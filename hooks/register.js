// The Logos Module Atlas as a Claude Code mod (Claude Code 2.1.287 or later):
//   - tools Claude calls to look up modules, contracts and methods
//   - context on any prompt that names a module
//   - checks on edits that call a module or declare dependencies on one
//   - /atlas, a command and a pane for browsing the atlas
// The logic lives in ./atlas.js; this file reads the atlas, handles events and draws.
import {
  TOOL_PREFIX,
  applyEdit,
  checkCalls,
  checkMetadata,
  describeCallProblems,
  findCalls,
  findMentions,
  groupByStack,
  isStale,
  mentionContext,
  methodDetail,
  moduleSummary,
  overviewText,
  parseLidl,
  searchAtlas,
  searchText,
  title,
  whereLine,
} from './atlas.js'

const PANE = 'atlas'
// Files whose module calls the edit check reads: C++, Rust, Nim, QML and JavaScript.
const CODE_FILE = /\.(c|cc|cpp|cxx|h|hh|hpp|hxx|rs|nim|qml|js|mjs|ts)$/

let registry = null // registry.json, read at session start
let byName = new Map()
let root = ''
let surface = null // 'terminal' or 'desktop', or null where nothing draws
const contracts = new Map() // module name → parsed contract, read on first use
const described = new Set() // modules whose context Claude already has in this conversation
const refused = new Set() // edits refused once: the same edit again goes through

// What the pane shows: a search, or one module
let query = ''
let selected = null

async function loadRegistry($) {
  root = $.plugin.root
  registry = JSON.parse(await $.fs.read(`${root}/registry.json`))
  byName = new Map(registry.modules.map((m) => [m.name, m]))
  contracts.clear()
}

async function contractOf($, name) {
  const m = byName.get(name)
  if (!m?.contract?.file) return null
  if (!contracts.has(name)) {
    let contract = null
    try {
      contract = parseLidl(await $.fs.read(`${root}/modules/${name}/${m.contract.file}`))
    } catch {}
    contracts.set(name, contract)
  }
  return contracts.get(name)
}

// The text a Write would leave, or an Edit would leave in the file it changes
async function editedText($, e) {
  if (e.tool === 'Write') return e.content
  try {
    return applyEdit(await $.fs.read(e.file_path), e)
  } catch {
    return null
  }
}

export function register(on) {
  on('session.start', async ($, e, next) => {
    surface = e.surface
    await loadRegistry($)
    await $.tool.register({
      name: 'module',
      description:
        'Everything the Logos Module Atlas knows about one official Logos module (Logos Core / Basecamp): ' +
        'what it is, where it ships, its version, source commit, dependencies both ways, every contract method ' +
        'and event, and the paths of its card, LIDL contract, interface source and stack doc.',
      inputSchema: {
        type: 'object',
        properties: { name: { type: 'string', description: 'Module name, such as keystore_module' } },
        required: ['name'],
      },
    })
    await $.tool.register({
      name: 'method',
      description:
        "One method or event of a Logos module's LIDL contract, with its full description. Methods that take " +
        'or return JSON in a tstr document the JSON shape only here, so read it before building a payload.',
      inputSchema: {
        type: 'object',
        properties: {
          module: { type: 'string', description: 'Module name, such as keystore_module' },
          method: { type: 'string', description: 'Method or event name, such as request_approval' },
        },
        required: ['module', 'method'],
      },
    })
    await $.tool.register({
      name: 'search',
      description:
        'Search the official Logos modules and their methods by words, such as "approval" or "monero send". ' +
        'Use it to find out whether a module that does something already exists.',
      inputSchema: {
        type: 'object',
        properties: { query: { type: 'string', description: 'Words that must all appear' } },
        required: ['query'],
      },
    })
    // Last, because it throws if the name is taken, which would skip what follows it
    await $.command.register({
      name: 'atlas',
      description: 'Browse the Logos Module Atlas, or look up a module or search it without a turn',
      argumentHint: '[module | words]',
      immediate: true,
    })
    return next(e)
  })

  // A new conversation hasn't seen the context earlier prompts carried
  on('classic.SessionStart', async ($, e, next) => {
    described.clear()
    return next(e)
  })

  on('tool.call', { tool: 'mcp__logos-module-atlas__module' }, async ($, e) => {
    const m = byName.get(String(e.name ?? '').trim())
    if (!m) return { result: `No module named "${e.name}". ${searchText(registry, String(e.name ?? ''))}` }
    return { result: moduleSummary(m, await contractOf($, m.name), root) }
  })

  on('tool.call', { tool: 'mcp__logos-module-atlas__method' }, async ($, e) => {
    const m = byName.get(String(e.module ?? '').trim())
    if (!m) return { result: `No module named "${e.module}". ${searchText(registry, String(e.module ?? ''))}` }
    const contract = await contractOf($, m.name)
    if (!contract) return { result: `${m.name} has no LIDL contract in the atlas. Its interface source: ${root}/modules/${m.name}/${m.interfaceSource?.file ?? 'README.md'}` }
    return {
      result:
        methodDetail(m, contract, String(e.method ?? '').trim()) ??
        `${m.name} declares no method or event "${e.method}". It declares: ${[...contract.methods.keys(), ...contract.events.keys()].join(', ')}`,
    }
  })

  on('tool.call', { tool: 'mcp__logos-module-atlas__search' }, async ($, e) => {
    return { result: searchText(registry, String(e.query ?? '')) }
  })

  // Name a module in a prompt, and Claude reads where it ships and where its contract is
  on('prompt.submit', async ($, e, next) => {
    if (!registry) return next(e)
    const fresh = findMentions(e.text, byName).filter((n) => !described.has(n))
    if (!fresh.length) return next(e)
    for (const n of fresh) described.add(n)
    const stale = isStale(registry.generatedAt, await $.clock.now())
    const note = mentionContext(fresh.map((n) => byName.get(n)), registry, root, stale)
    return next({ ...e, context: [...(e.context ?? []), note] })
  })

  // Check an edit against the contracts before it lands: calls to methods a module doesn't
  // declare, and metadata.json mistakes that keep a module from loading. A refused edit
  // made again goes through, with the problem in Claude's context, so a check that's wrong
  // can't block Claude for good.
  on('tool.call', { tool: ['Edit', 'Write'] }, async ($, e, next) => {
    const path = String(e.file_path ?? '')
    // The atlas's own files quote wrong calls on purpose, in tests and examples
    if (!registry || path.startsWith(root + '/')) return next(e)
    let errors = []
    let warnings = []
    if (/(^|\/)metadata\.json$/.test(path)) {
      const text = await editedText($, e)
      let meta = null
      try {
        meta = JSON.parse(text)
      } catch {}
      if (meta && typeof meta === 'object' && typeof meta.name === 'string') ({ errors, warnings } = checkMetadata(meta, byName))
    } else if (CODE_FILE.test(path)) {
      const calls = findCalls(e.tool === 'Edit' ? e.new_string : e.content)
      const known = new Map()
      for (const name of new Set(calls.map((c) => c.module))) {
        const contract = await contractOf($, name)
        if (contract) known.set(name, contract)
      }
      errors = describeCallProblems(checkCalls(calls, known), byName, root)
    }

    if (errors.length) {
      const key = path + '\n' + errors.join('\n')
      if (!refused.has(key)) {
        refused.add(key)
        return {
          deny:
            'Logos Module Atlas refused this edit:\n- ' +
            errors.join('\n- ') +
            `\nCheck the contract, or call ${TOOL_PREFIX}module or ${TOOL_PREFIX}method, and fix the edit. ` +
            'If you have checked and the edit is right, make the same edit again and it will go through.',
        }
      }
      warnings = [...errors, ...warnings]
    }
    if (!warnings.length) return next(e)
    const result = await next(e)
    if (result.deny || result.isError) return result
    const note = 'Logos Module Atlas, about the edit just made:\n- ' + warnings.join('\n- ')
    return { ...result, context: [...(result.context ?? []), note] }
  })

  // /atlas opens the pane; /atlas <module> or /atlas <words> answers in the transcript
  on('command.run', { command: 'atlas' }, async ($, e) => {
    if (!registry) return { text: 'The atlas registry did not load. Check the debug log.' }
    const arg = String(e.args ?? '').trim()
    if (byName.has(arg)) return { text: moduleSummary(byName.get(arg), await contractOf($, arg), root) }
    if (arg) return { text: searchText(registry, arg) }
    if (!surface) return { text: overviewText(registry) }
    query = ''
    selected = null
    await $.ui.open({ id: PANE, title: 'Logos Module Atlas', focus: true, closeOnEscape: true })
    return {}
  })

  on('ui.render', { component: 'Pane' }, async ($, e, next) => {
    if (e.requestId !== PANE) return next(e)
    const { Box, Text, Button, Input } = $.ui.resolve(e)
    const redraw = () => $.ui.invalidate('ui.render')
    const show = (name) => () => {
      selected = name
      redraw()
    }
    const line = (children, props = {}) => Text({ wrap: 'truncate-end', ...props, children })
    const blank = () => Text({ children: [' '] })
    const moduleRow = (m, note) =>
      Box({
        flexDirection: 'row',
        columnGap: 2,
        children: [Button({ key: 'm-' + m.name, label: m.name, plain: true, onPress: show(m.name) }), line([note], { dimColor: true })],
      })

    if (!registry) return line(['The atlas registry did not load.'])

    if (selected && byName.has(selected)) {
      const m = byName.get(selected)
      const contract = await contractOf($, m.name)
      const deps = (m.dependencies ?? []).map((d) => (typeof d === 'string' ? { name: d } : d))
      const methods = contract ? [...contract.methods.values()] : (m.api?.methods ?? [])
      const summaries = new Map((m.api?.methods ?? []).map((x) => [x.name, x.summary]))
      const events = contract ? [...contract.events.values()] : []
      const related = (title, prefix, list) =>
        list.length
          ? [
              line([title], { bold: true }),
              ...list.map((d) =>
                byName.has(d.name)
                  ? Button({ key: prefix + d.name, label: d.version ? `${d.name} ${d.version}` : d.name, plain: true, onPress: show(d.name) })
                  : line([d.version ? `${d.name} ${d.version}` : d.name]),
              ),
            ]
          : []
      return Box({
        flexDirection: 'column',
        children: [
          Box({
            flexDirection: 'row',
            columnGap: 3,
            children: [
              Button({ key: 'back', label: 'Back', hotkey: 'b', plain: true, autoFocus: true, onPress: show(null) }),
              Button({
                key: 'draft',
                label: 'Draft a prompt',
                hotkey: 'd',
                plain: true,
                onPress: async () => {
                  await $.prompt.fill({ text: `Using ${m.name} from the Logos Module Atlas, ` })
                  await $.ui.close({ id: PANE })
                },
              }),
            ],
          }),
          blank(),
          line([title(m)], { bold: true }),
          Text({ children: [m.description ?? ''] }),
          line([`${m.type} · ${m.language ?? '?'} · ${m.stack} stack · ${whereLine(m)}`], { dimColor: true }),
          blank(),
          ...related('Depends on', 'dep-', deps),
          ...related('Required by', 'req-', (m.requiredBy ?? []).map((name) => ({ name }))),
          blank(),
          line([contract ? `Methods (${methods.length})` : `Methods from its interface source (${methods.length}), no LIDL contract`], { bold: true }),
          ...methods.flatMap((x) => [line([x.signature]), ...(summaries.get(x.name) ? [line(['  ' + summaries.get(x.name)], { dimColor: true })] : [])]),
          ...(events.length ? [blank(), line([`Events (${events.length})`], { bold: true }), ...events.map((x) => line([x.signature]))] : []),
          blank(),
          line([`Contract: ${contract ? `${root}/modules/${m.name}/${m.contract.file}` : 'none'}`], { dimColor: true }),
        ],
      })
    }

    const search = Input({
      key: 'search',
      label: 'Find',
      placeholder: 'a module, a method or a word, then Enter',
      value: query,
      submitLabel: 'search',
      autoFocus: true,
      onSubmit: (value) => {
        query = value.trim()
        redraw()
      },
    })

    if (query) {
      const results = searchAtlas(registry, query)
      return Box({
        flexDirection: 'column',
        children: [
          search,
          blank(),
          ...(results.length
            ? results.map(({ module: m, methods }) => moduleRow(m, methods.length ? 'methods: ' + methods.join(', ') : m.description ?? ''))
            : [line([`Nothing matches "${query}". Roadmap items that aren't released are in gaps.md.`])]),
        ],
      })
    }

    const stacks = groupByStack(registry).flatMap(([title, mods]) => [
      blank(),
      line([`${title} (${mods.length})`], { bold: true }),
      ...mods.map((m) => moduleRow(m, m.description ?? '')),
    ])
    return Box({
      flexDirection: 'column',
      children: [
        search,
        line([`${registry.modules.length} modules in Basecamp ${registry.basecamp.tag}, registry generated ${registry.generatedAt.slice(0, 10)}`], { dimColor: true }),
        ...stacks,
      ],
    })
  })
}
