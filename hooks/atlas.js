// Pure helpers for the atlas mod. Nothing here touches the mods API, so tests call these
// directly and register.js does all the reading, drawing and event handling.

export const TOOL_PREFIX = 'mcp__logos-module-atlas__'

// Stacks whose modules hold or request keys; their context also points at key-custody.md.
const KEY_STACKS = new Set(['evm-wallet', 'monero-wallet'])

// ---------------------------------------------------------------------------
// Contracts

// One `method` or `event` line of a .lidl contract. Descriptions are one quoted string
// with escaped newlines, so every declaration fits on one line.
const LIDL_DECL =
  /^\s*(method|event)\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*(.+?))?(?:\s+description\s+"((?:[^"\\]|\\.)*)")?\s*$/

function unescapeLidl(raw) {
  try {
    return JSON.parse('"' + raw + '"')
  } catch {
    return raw.replace(/\\n/g, '\n').replace(/\\"/g, '"').replace(/\\\\/g, '\\')
  }
}

/** Methods and events of a .lidl contract, each { name, signature, description }. */
export function parseLidl(text) {
  const methods = new Map()
  const events = new Map()
  for (const line of text.split('\n')) {
    const m = LIDL_DECL.exec(line)
    if (!m) continue
    const [, kind, name, params, ret, desc] = m
    const signature = `${name}(${params.trim()})` + (ret ? ` -> ${ret.trim()}` : '')
    const entry = { name, signature, description: desc == null ? '' : unescapeLidl(desc) }
    ;(kind === 'method' ? methods : events).set(name, entry)
  }
  return { methods, events }
}

// ---------------------------------------------------------------------------
// Calls in source code

// The call shapes guides/calling-official-modules.md documents. Only shapes that name the
// module and the method in one expression can be checked; a proxy held in a variable can't.
const CALL_PATTERNS = [
  // C++ and Rust typed clients: modules().keystore_module.request_approval(...)
  { kind: 'typed', re: /\bmodules\(\)\s*\.\s*(\w+)\s*\.\s*(\w+)\s*\(/g },
  // C++ untyped client: modules().dynamic("keystore_module").invoke("list_accounts", ...)
  { kind: 'dynamic', re: /\bmodules\(\)\s*\.\s*dynamic\(\s*"(\w+)"\s*\)\s*\.\s*invoke\(\s*"(\w+)"/g },
  // QML bridge: logos.callModule[Async]("keystore_module", "list_accounts", ...)
  { kind: 'qml', re: /\blogos\s*\.\s*callModule(?:Async)?\(\s*["'](\w+)["']\s*,\s*["'](\w+)["']/g },
  // QML events: logos.onModuleEvent("keystore_module", "accounts_changed")
  { kind: 'event', re: /\blogos\s*\.\s*onModuleEvent\(\s*["'](\w+)["']\s*,\s*["'](\w+)["']/g },
]

// Wrapper names the SDKs generate around a contract method: C++ fooAsync/fooAsyncResult,
// Rust foo_async/foo_with_timeout/foo_async_with_timeout.
const WRAPPER_SUFFIXES = ['AsyncResult', 'Async', '_async_with_timeout', '_with_timeout', '_async']

/** Every checkable module call in `text`, as { kind, module, member }. */
export function findCalls(text) {
  const calls = []
  if (!text) return calls
  for (const { kind, re } of CALL_PATTERNS) {
    for (const m of text.matchAll(re)) {
      if (kind === 'typed' && m[1] === 'dynamic') continue
      calls.push({ kind, module: m[1], member: m[2] })
    }
  }
  return calls
}

function methodCandidates(member) {
  const names = [member]
  for (const suffix of WRAPPER_SUFFIXES) {
    if (member.endsWith(suffix) && member.length > suffix.length) names.push(member.slice(0, -suffix.length))
  }
  return names
}

function editDistance(a, b) {
  const row = Array.from({ length: b.length + 1 }, (_, i) => i)
  for (let i = 1; i <= a.length; i++) {
    let prev = row[0]
    row[0] = i
    for (let j = 1; j <= b.length; j++) {
      const cur = row[j]
      row[j] = Math.min(row[j] + 1, row[j - 1] + 1, prev + (a[i - 1] === b[j - 1] ? 0 : 1))
      prev = cur
    }
  }
  return row[b.length]
}

/**
 * Up to `limit` names from `entries` (a contract's methods or events) closest to `name`:
 * by spelling, by shared words, and by words of `name` that a description uses, so that
 * `sign_tx` finds the keystore's `request_approval` ("approve signing").
 */
export function closest(name, entries, limit = 3) {
  const words = name.split(/_|(?=[A-Z])/).map((w) => w.toLowerCase()).filter((w) => w.length >= 3)
  return [...entries.values()]
    .map(({ name: c, description }) => {
      const spelling = editDistance(name.toLowerCase(), c.toLowerCase()) / Math.max(name.length, c.length)
      const inName = words.filter((w) => c.toLowerCase().split('_').includes(w)).length
      const inText = words.filter((w) => new RegExp(`\\b${w}`, 'i').test(description ?? '')).length
      return { c, score: 10 * spelling - 3 * inName - 1.5 * inText }
    })
    .sort((x, y) => x.score - y.score || x.c.localeCompare(y.c))
    .slice(0, limit)
    .map((x) => x.c)
}

/**
 * Calls that name a module the atlas has a contract for, with a method or event that
 * contract doesn't declare. `contracts` maps a module name to its parsed contract; a module
 * missing from it (third-party, or no contract) isn't checked.
 */
export function checkCalls(calls, contracts) {
  const problems = []
  const seen = new Set()
  for (const call of calls) {
    const contract = contracts.get(call.module)
    if (!contract) continue
    const key = `${call.kind}:${call.module}.${call.member}`
    if (seen.has(key)) continue
    seen.add(key)
    if (call.kind === 'event') {
      if (!contract.events.has(call.member)) {
        problems.push({ ...call, suggestions: closest(call.member, contract.events) })
      }
      continue
    }
    // A typed client also has event subscriptions (onAccountsChanged, on_event), which
    // aren't contract methods.
    if (call.kind === 'typed' && /^on[A-Z_]/.test(call.member)) continue
    const names = call.kind === 'typed' ? methodCandidates(call.member) : [call.member]
    if (!names.some((n) => contract.methods.has(n))) {
      problems.push({ ...call, suggestions: closest(names[names.length - 1], contract.methods) })
    }
  }
  return problems
}

/** One line per problem, for the text Claude reads. */
export function describeCallProblems(problems, byName, root) {
  return problems.map((p) => {
    const what = p.kind === 'event' ? 'event' : 'method'
    const near = p.suggestions.length ? ` Closest: ${p.suggestions.join(', ')}.` : ''
    return `${p.module} declares no ${what} \`${p.member}\`.${near} Contract: ${modulePaths(byName.get(p.module), root).contract}`
  })
}

// ---------------------------------------------------------------------------
// Version ranges, as liblogos's dependency gate reads them

function parseVersion(v) {
  const m = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/.exec(String(v).trim())
  return m ? { major: +m[1], minor: +m[2], patch: +m[3], pre: m[4] ?? '' } : null
}

function compare(a, b) {
  return a.major - b.major || a.minor - b.minor || a.patch - b.patch
}

// A partial version such as 1, 1.2, 1.x or 1.2.* : the numbers before the first wildcard.
function parsePartial(s) {
  const parts = []
  for (const p of s.replace(/^v/, '').split('.')) {
    if (p === '' || /^[xX*]$/.test(p)) break
    if (!/^\d+$/.test(p)) return null
    parts.push(+p)
  }
  return parts.length > 3 ? null : parts
}

function lower(parts) {
  return { major: parts[0] ?? 0, minor: parts[1] ?? 0, patch: parts[2] ?? 0 }
}

// The predicate one comparator such as ^0.1.0, ~1.2, >=1.0.0 or 1.x stands for.
function comparator(token) {
  const m = /^(\^|~|>=|<=|>|<|=)?(.*)$/.exec(token)
  const op = m[1] ?? ''
  const parts = parsePartial(m[2])
  if (parts == null) return null
  if (parts.length === 0) return op === '' || op === '>=' ? () => true : null
  const lo = lower(parts)
  let hi = null
  if (op === '^') {
    if (parts[0] > 0 || parts.length === 1) hi = { major: parts[0] + 1, minor: 0, patch: 0 }
    else if ((parts[1] ?? 0) > 0 || parts.length === 2) hi = { major: 0, minor: parts[1] + 1, patch: 0 }
    else hi = { major: 0, minor: 0, patch: parts[2] + 1 }
  } else if (op === '~') {
    hi = parts.length === 1 ? { major: parts[0] + 1, minor: 0, patch: 0 } : { major: parts[0], minor: parts[1] + 1, patch: 0 }
  } else if (op === '' || op === '=') {
    if (parts.length === 3) return (v) => compare(v, lo) === 0
    hi = parts.length === 1 ? { major: parts[0] + 1, minor: 0, patch: 0 } : { major: parts[0], minor: parts[1] + 1, patch: 0 }
  } else {
    return {
      '>=': (v) => compare(v, lo) >= 0,
      '>': (v) => compare(v, lo) > 0,
      '<=': (v) => compare(v, lo) <= 0,
      '<': (v) => compare(v, lo) < 0,
    }[op]
  }
  return (v) => compare(v, lo) >= 0 && compare(v, hi) < 0
}

/**
 * Whether `version` satisfies the npm-style `range`: true, false, or 'unsupported' for a
 * range liblogos refuses outright (hyphen ranges) or can't be read. A prerelease version
 * never satisfies, as the gate refuses 1.0.0-dev against a caret range.
 */
export function satisfies(version, range) {
  const v = parseVersion(version)
  if (!v) return false
  const alternatives = String(range).split('||').map((a) => a.trim())
  let unsupported = false
  for (const alt of alternatives) {
    if (/\s-\s/.test(alt)) {
      unsupported = true
      continue
    }
    const tests = alt.split(/\s+/).filter(Boolean).map(comparator)
    if (tests.some((t) => t == null)) {
      unsupported = true
      continue
    }
    if (!v.pre && tests.every((t) => t(v))) return true
  }
  return unsupported ? 'unsupported' : false
}

// ---------------------------------------------------------------------------
// metadata.json

/** Every version of a module that the release ships, catalog and bundled. */
export function releasedVersions(m) {
  const versions = new Set(m.catalog?.versions ?? [])
  if (m.catalog?.latest) versions.add(m.catalog.latest)
  if (m.availability?.bundledVersion) versions.add(m.availability.bundledVersion)
  return [...versions]
}

/**
 * Problems in a Logos module's metadata.json that guides/calling-official-modules.md
 * describes. `errors` make the module fail to load or misbehave; `warnings` are risks.
 */
export function checkMetadata(meta, byName) {
  const errors = []
  const warnings = []
  for (const u of Array.isArray(meta.uses) ? meta.uses : []) {
    if (typeof u === 'string') {
      errors.push(
        `"uses" entry "${u}" is a string. It parses but declares nothing, so every request fails with not_declared. Write it as { "intent": "${u}" }.`,
      )
    }
  }
  for (const d of Array.isArray(meta.dependencies) ? meta.dependencies : []) {
    const name = typeof d === 'string' ? d : d?.name
    const m = byName.get(name)
    if (!m) continue
    const released = releasedVersions(m)
    if (!released.length) continue
    const range = typeof d === 'string' ? undefined : d?.version
    if (!range) {
      const newest = m.catalog?.latest ?? released[released.length - 1]
      warnings.push(
        `Dependency ${name} has no version range, so it resolves to the newest catalog version and nothing checks raw lp calls against a newer contract. Pin one, such as "~${newest}".`,
      )
      continue
    }
    const verdict = released.map((v) => satisfies(v, range))
    if (verdict.includes('unsupported')) {
      errors.push(`Dependency ${name}: liblogos refuses the range "${range}". Use ^, ~, x-ranges or comparators, not a hyphen range.`)
    } else if (!verdict.includes(true)) {
      errors.push(
        `Dependency ${name}: the range "${range}" matches no version this release ships (${released.join(', ')}), and liblogos refuses to load a module whose dependency doesn't satisfy its range.`,
      )
    }
  }
  if (Array.isArray(meta.capabilities) && meta.capabilities.length) {
    warnings.push(
      `"capabilities" is decorative: nothing reads it. The real privilege key is host_services, which only capability_module may hold.`,
    )
  }
  return { errors, warnings }
}

/** The text an Edit would leave in a file, or null when old_string isn't in it. */
export function applyEdit(current, e) {
  if (!current.includes(e.old_string)) return null
  return e.replace_all ? current.split(e.old_string).join(e.new_string) : current.replace(e.old_string, () => e.new_string)
}

// ---------------------------------------------------------------------------
// Text for Claude and for the /atlas command

/** Absolute paths of a module's files in the atlas. */
export function modulePaths(m, root) {
  const dir = `${root}/modules/${m.name}`
  return {
    card: `${dir}/README.md`,
    contract: m.contract?.file ? `${dir}/${m.contract.file}` : null,
    interface: m.interfaceSource?.file ? `${dir}/${m.interfaceSource.file}` : null,
    stack: `${root}/stacks/${m.stack}.md`,
  }
}

/** "v0.1.0 in the default catalog", "bundled in Basecamp (v1.0.0)", or both. */
export function whereLine(m) {
  const parts = []
  if (m.availability?.bundledInBasecamp) parts.push(`bundled in Basecamp (v${m.availability.bundledVersion})`)
  if (m.availability?.inDefaultCatalog) parts.push(`v${m.catalog?.latest} in the default catalog`)
  return parts.join(', ') || 'not shipped'
}

/** "keystore_module" or "tx_sender_module: EVM Transaction Sender"; not every module has a display name. */
export function title(m) {
  return m.displayName ? `${m.name}: ${m.displayName}` : m.name
}

function depLine(d) {
  return typeof d === 'string' ? d : d.version ? `${d.name} ${d.version}` : d.name
}

/** Names of known modules that `text` mentions, in the order they first appear. */
export function findMentions(text, byName) {
  const found = new Set()
  for (const word of String(text).match(/\b[a-z][a-z0-9_]*\b/g) ?? []) if (byName.has(word)) found.add(word)
  return [...found]
}

/** Whether `generatedAt` is more than `days` days before `now` (milliseconds). */
export function isStale(generatedAt, now, days = 14) {
  const t = Date.parse(generatedAt)
  return Number.isFinite(t) && now - t > days * 86_400_000
}

/** The context a prompt that mentions these modules carries to Claude. */
export function mentionContext(modules, registry, root, stale) {
  const rel = registry.basecamp
  const lines = [`Logos Module Atlas (Basecamp ${rel.tag}, registry generated ${registry.generatedAt.slice(0, 10)}):`]
  for (const m of modules) {
    const p = modulePaths(m, root)
    const contract = p.contract ? `contract ${p.contract}` : `no LIDL contract; interface ${p.interface ?? p.card}`
    lines.push(`- ${title(m)}, ${m.type} module in the ${m.stack} stack, ${whereLine(m)}. Card ${p.card}; ${contract}.`)
  }
  lines.push(
    `Take method names and payload shapes only from the contract, or from the ${TOOL_PREFIX}module and ${TOOL_PREFIX}method tools. ` +
      `Before writing integration code, read ${root}/guides/calling-official-modules.md and ${root}/guides/compatibility.md.`,
  )
  if (modules.some((m) => KEY_STACKS.has(m.stack))) {
    lines.push(`Who holds keys and what is ungated: ${root}/stacks/key-custody.md. Design around the custodian; never copy key material into the caller.`)
  }
  if (stale) lines.push(`The registry is more than two weeks old; versions may have moved. Update the plugin before relying on them.`)
  return lines.join('\n')
}

/** Everything the atlas knows about one module, as plain text. */
export function moduleSummary(m, contract, root) {
  const p = modulePaths(m, root)
  const lines = [
    `${title(m)} (${m.type}, ${m.language ?? 'unknown language'}, ${m.stack} stack)`,
    m.description ?? '',
    `Ships: ${whereLine(m)}. Platforms: ${(m.catalog?.variants ?? []).join(', ') || 'see card'}.`,
    `Source: https://github.com/${m.source?.repo} at ${m.source?.commit} (read this commit, not HEAD).`,
    `Depends on: ${(m.dependencies ?? []).map(depLine).join(', ') || 'nothing'}.`,
    `Required by: ${(m.requiredBy ?? []).join(', ') || 'nothing in this release'}.`,
  ]
  const methods = contract ? [...contract.methods.values()] : null
  const apiMethods = m.api?.methods ?? []
  if (methods) {
    lines.push('', `Methods (${methods.length}):`)
    const summaries = new Map(apiMethods.map((x) => [x.name, x.summary]))
    for (const x of methods) lines.push(`  ${x.signature}` + (summaries.get(x.name) ? ` : ${summaries.get(x.name)}` : ''))
    const events = [...contract.events.values()]
    lines.push('', events.length ? `Events (${events.length}):` : 'Events: none.')
    for (const x of events) lines.push(`  ${x.signature}`)
  } else if (apiMethods.length) {
    lines.push('', `No LIDL contract. Methods from its interface source (${apiMethods.length}):`)
    for (const x of apiMethods) lines.push(`  ${x.signature}` + (x.summary ? ` : ${x.summary}` : ''))
  }
  lines.push(
    '',
    `Card: ${p.card}`,
    p.contract ? `Contract: ${p.contract}` : 'Contract: none',
    `Interface source: ${p.interface ?? 'none'}`,
    `Stack doc: ${p.stack}`,
  )
  if (methods?.some((x) => x.signature.includes('tstr'))) {
    lines.push(`Methods that take or return JSON in a tstr document its shape only in their description: call ${TOOL_PREFIX}method for it.`)
  }
  return lines.join('\n')
}

/** One method or event with its full description, or null. */
export function methodDetail(m, contract, name) {
  const x = contract?.methods.get(name) ?? contract?.events.get(name)
  if (!x) return null
  const kind = contract.methods.has(name) ? 'method' : 'event'
  return `${m.name} ${kind} ${x.signature}\n\n${x.description || '(no description in the contract)'}`
}

/** Modules and methods matching every word of `query`, best first. */
export function searchAtlas(registry, query) {
  const words = String(query).toLowerCase().split(/\s+/).filter(Boolean)
  if (!words.length) return []
  const has = (s) => words.every((w) => String(s ?? '').toLowerCase().includes(w))
  const results = []
  for (const m of registry.modules) {
    // A method matches when the words are in it or in its module's name ("monero send"),
    // and at least one is in the method itself.
    const methods = (m.api?.methods ?? [])
      .filter((x) => {
        const own = `${x.name} ${x.summary ?? ''}`.toLowerCase()
        return words.some((w) => own.includes(w)) && words.every((w) => own.includes(w) || m.name.includes(w))
      })
      .map((x) => x.name)
    const byModule = has(`${m.name} ${m.displayName ?? ''} ${m.description ?? ''} ${m.stack}`)
    if (!byModule && !methods.length) continue
    const score = (m.name.includes(words[0]) ? 0 : 2) + (byModule ? 0 : 1)
    results.push({ module: m, methods, score })
  }
  return results.sort((a, b) => a.score - b.score || a.module.name.localeCompare(b.module.name))
}

/** Search results as text. */
export function searchText(registry, query) {
  const results = searchAtlas(registry, query)
  if (!results.length) return `Nothing in the atlas matches "${query}". If you expected a module, check gaps.md: it lists roadmap items that aren't in this release.`
  return results
    .slice(0, 25)
    .map(({ module: m, methods }) => {
      const d = m.description ?? ''
      const head = `${m.name} (${m.stack}): ${d.length > 120 ? d.slice(0, 117) + '…' : d}`
      return methods.length ? `${head}\n  methods: ${methods.join(', ')}` : head
    })
    .join('\n')
}

/**
 * Modules grouped by stack, as [title, modules] in the registry's order. A module the
 * generator placed in no known stack (a new catalog package nobody has assigned yet)
 * goes in a last "Other" group.
 */
export function groupByStack(registry) {
  const groups = Object.entries(registry.stacks).map(([key, stack]) => [stack.title, registry.modules.filter((m) => m.stack === key)])
  groups.push(['Other', registry.modules.filter((m) => !registry.stacks[m.stack])])
  return groups.filter(([, modules]) => modules.length)
}

/** Every module, grouped by stack, as text. */
export function overviewText(registry) {
  const lines = [`Logos Module Atlas: ${registry.modules.length} modules in Basecamp ${registry.basecamp.tag}, registry generated ${registry.generatedAt.slice(0, 10)}.`]
  for (const [title, modules] of groupByStack(registry)) lines.push(`${title}: ${modules.map((m) => m.name).join(', ')}`)
  lines.push('Run /atlas <module> for one module, or /atlas <words> to search.')
  return lines.join('\n')
}
