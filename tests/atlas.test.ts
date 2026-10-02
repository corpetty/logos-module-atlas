// Tests for the atlas mod, run with `claude plugin test` from the repository root. They read
// a small fixture atlas through stubs, so they don't depend on the daily registry refresh.
import { expect, mock, test } from 'claude-code/testing'
import { checkCalls, checkMetadata, findCalls, parseLidl, satisfies } from '../hooks/atlas.js'

const KEYSTORE_LIDL = String.raw`module keystore_module {
  version "0.1.0"
  depends []

  method list_accounts() -> tstr description "${'`'}{ ok, accounts: [address, ...] }${'`'}."
  method request_approval(intent_json: tstr) -> tstr description "Ask a human to approve signing.\nReturns ${'`'}{ ok, handle, receipt }${'`'}."
  event approval_offered(handle: tstr) description "A new request is waiting."
}
`

const TX_SENDER_LIDL = String.raw`module tx_sender_module {
  version "0.1.0"
  depends [keystore_module]

  method send(request_json: tstr) -> tstr description "Ask a human to approve a bundle."
}
`

const keystore = {
  name: 'keystore_module',
  displayName: 'EVM Keystore',
  description: 'Keystore: scrypt vaults and secp256k1 signing.',
  type: 'core',
  language: 'cpp',
  stack: 'evm-wallet',
  availability: { bundledInBasecamp: false, bundledVersion: null, inDefaultCatalog: true },
  catalog: { latest: '0.1.0', versions: ['0.1.0'], variants: ['linux-amd64'] },
  source: { repo: 'logos-co/logos-evm-keystore-module', commit: '2318c679' },
  dependencies: [],
  requiredBy: ['tx_sender_module'],
  contract: { file: 'keystore_module.lidl' },
  interfaceSource: { file: 'interface.h' },
  api: {
    methods: [
      { name: 'list_accounts', signature: 'list_accounts() -> tstr', summary: 'Every account.' },
      { name: 'request_approval', signature: 'request_approval(intent_json: tstr) -> tstr', summary: 'Ask a human to approve signing.' },
    ],
  },
}

const txSender = {
  ...keystore,
  name: 'tx_sender_module',
  displayName: 'EVM Transaction Sender',
  description: 'The one EVM transaction sender on the device.',
  language: 'rust',
  dependencies: [{ name: 'keystore_module', version: '~0.1.0' }],
  requiredBy: [],
  contract: { file: 'tx_sender_module.lidl' },
  api: { methods: [{ name: 'send', signature: 'send(request_json: tstr) -> tstr', summary: 'Ask a human to approve a bundle.' }] },
}

const signerUi = {
  ...keystore,
  name: 'evm_signer_ui',
  // Not every module has a display name in the registry
  displayName: null,
  description: 'The signing approval surface.',
  type: 'ui_qml',
  requiredBy: [],
  contract: null,
  interfaceSource: { file: 'interface.rep' },
  api: { methods: [{ name: 'approve', signature: 'approve(handle: QString)', summary: 'Approve a request.' }] },
}

const REGISTRY = {
  generatedAt: '2026-10-01T19:03:15+00:00',
  basecamp: { tag: '0.3.1' },
  stacks: { 'evm-wallet': { title: 'EVM wallet' } },
  modules: [keystore, txSender, signerUi],
}

// Answer the mod's file reads from the fixture, and start the session, as Claude Code would
async function start($, on, files: Record<string, string> = {}) {
  on('fs.read', ($, e) => {
    if (e.path.endsWith('/registry.json')) return { value: JSON.stringify(REGISTRY) }
    if (e.path.endsWith('/keystore_module.lidl')) return { value: KEYSTORE_LIDL }
    if (e.path.endsWith('/tx_sender_module.lidl')) return { value: TX_SENDER_LIDL }
    if (e.path in files) return { value: files[e.path] }
    return { deny: 'no such file: ' + e.path }
  })
  on('tool.register', () => ({ value: undefined }))
  on('command.register', () => ({ value: undefined }))
  on('session.start', () => ({ cwd: '/work' }))
  mock.clock(on, { now: Date.parse('2026-10-02T00:00:00Z') })
  await $.session.start({ surface: 'terminal', isInteractive: true, cwd: '/work' })
}

// ---------------------------------------------------------------------------
// Pure helpers

test('parseLidl reads methods, events and escaped descriptions', async () => {
  const c = parseLidl(KEYSTORE_LIDL)
  expect([...c.methods.keys()]).toEqual(['list_accounts', 'request_approval'])
  expect([...c.events.keys()]).toEqual(['approval_offered'])
  expect(c.methods.get('request_approval').signature).toBe('request_approval(intent_json: tstr) -> tstr')
  expect(c.methods.get('request_approval').description).toBe('Ask a human to approve signing.\nReturns `{ ok, handle, receipt }`.')
})

test('satisfies reads ranges the way the dependency gate does', async () => {
  const cases: [string, string, boolean | string][] = [
    ['0.1.0', '~0.1.0', true],
    ['0.1.5', '^0.1.0', true],
    ['0.2.0', '^0.1.0', false],
    ['1.4.0', '^1.2.3', true],
    ['2.0.0', '^1.2.3', false],
    ['0.0.4', '^0.0.3', false],
    ['3.0.0', '>=2.0.0 <4.0.0', true],
    ['3.1.0', '1.x || 3.x', true],
    ['1.0.0-dev', '^1.0.0', false],
    ['1.0.0', '1.0.0 - 2.0.0', 'unsupported'],
    ['0.1.0', '*', true],
  ]
  for (const [version, range, want] of cases) expect(satisfies(version, range)).toBe(want)
})

test('checkCalls flags unknown methods and events in every call shape, and only for known contracts', async () => {
  const code = `
    modules().keystore_module.request_approvalAsyncResult(intent, cb);
    let raw = modules().keystore_module.request_approval_with_timeout(&i, t)?;
    modules().keystore_module.onAccountsChanged(cb);
    modules().keystore_module.sign_tx(intent);
    modules().dynamic("keystore_module").invoke("list_acounts", args, &err, 5000);
    logos.callModuleAsync("keystore_module", "list_accounts", [], cb, 30000)
    logos.onModuleEvent("keystore_module", "approval_done")
    modules().third_party_module.anything(1);
  `
  const contracts = new Map([['keystore_module', parseLidl(KEYSTORE_LIDL)]])
  const problems = checkCalls(findCalls(code), contracts)
  expect(problems.map((p) => `${p.kind} ${p.member}`)).toEqual(['typed sign_tx', 'dynamic list_acounts', 'event approval_done'])
  expect(problems[1].suggestions[0]).toBe('list_accounts')
})

test('checkMetadata separates load failures from risks', async () => {
  const byName = new Map(REGISTRY.modules.map((m) => [m.name, m]))
  const { errors, warnings } = checkMetadata(
    {
      name: 'muster_module',
      uses: ['evm.signing.approve', { intent: 'evm.accounts.manage' }],
      dependencies: [{ name: 'keystore_module', version: '~0.2.0' }, 'tx_sender_module', { name: 'third_party' }],
      capabilities: ['sign'],
    },
    byName,
  )
  expect(errors.length).toBe(2)
  expect(errors[0]).toContain('"uses" entry "evm.signing.approve" is a string')
  expect(errors[1]).toContain('"~0.2.0" matches no version')
  expect(warnings.length).toBe(2)
  expect(warnings[0]).toContain('tx_sender_module has no version range')
  expect(warnings[1]).toContain('"capabilities" is decorative')
})

// ---------------------------------------------------------------------------
// Tools

test('the module tool describes a module from its contract', async ($, on) => {
  await start($, on)
  const out = await $.tool.call({ tool: 'mcp__logos-module-atlas__module', name: 'keystore_module' })
  expect(out.result).toContain('keystore_module: EVM Keystore (core, cpp, evm-wallet stack)')
  expect(out.result).toContain('Methods (2):')
  expect(out.result).toContain('approval_offered(handle: tstr)')
  expect(out.result).toContain('/modules/keystore_module/keystore_module.lidl')
})

test('the method tool returns the payload description, and lists the real names for a wrong one', async ($, on) => {
  await start($, on)
  const ok = await $.tool.call({ tool: 'mcp__logos-module-atlas__method', module: 'keystore_module', method: 'request_approval' })
  expect(ok.result).toContain('Returns `{ ok, handle, receipt }`')
  const wrong = await $.tool.call({ tool: 'mcp__logos-module-atlas__method', module: 'keystore_module', method: 'sign' })
  expect(wrong.result).toContain('declares no method or event "sign"')
  expect(wrong.result).toContain('list_accounts, request_approval, approval_offered')
})

test('the search tool finds modules by method', async ($, on) => {
  await start($, on)
  const out = await $.tool.call({ tool: 'mcp__logos-module-atlas__search', query: 'approve' })
  expect(out.result).toContain('keystore_module (evm-wallet)')
  expect(out.result).toContain('methods: request_approval')
})

// ---------------------------------------------------------------------------
// Prompt context

test('a prompt that names a module carries its context once per conversation', async ($, on) => {
  const seen: (readonly string[] | undefined)[] = []
  on('prompt.submit', ($, e) => {
    seen.push(e.context)
    return { text: e.text }
  })
  await start($, on)
  await $.prompt.submit({ text: 'call keystore_module from my module', wait: false, origin: { kind: 'user' } })
  await $.prompt.submit({ text: 'and keystore_module again', wait: false, origin: { kind: 'user' } })
  await $.prompt.submit({ text: 'nothing to see here', wait: false, origin: { kind: 'user' } })
  expect(seen[0]?.[0]).toContain('- keystore_module: EVM Keystore, core module in the evm-wallet stack, v0.1.0 in the default catalog.')
  expect(seen[0]?.[0]).toContain('stacks/key-custody.md')
  expect(seen[0]?.[0]).not.toContain('more than two weeks old')
  expect(seen[1]).toBeUndefined()
  expect(seen[2]).toBeUndefined()
})

// ---------------------------------------------------------------------------
// Edit checks

test('an edit that calls an undeclared method is refused once, then goes through with a warning', async ($, on) => {
  on('tool.call', () => ({ result: 'written' }))
  await start($, on)
  const edit = { tool: 'Write', file_path: '/work/ui/Main.qml', content: 'logos.callModuleAsync("keystore_module", "list_acounts", [], f, 1)' }
  const first = await $.tool.call(edit)
  expect(first.deny).toContain('keystore_module declares no method `list_acounts`. Closest: list_accounts')
  const second = await $.tool.call(edit)
  expect(second.result).toBe('written')
  expect(second.context?.[0]).toContain('list_acounts')
})

test('an edit with only known calls goes through untouched', async ($, on) => {
  on('tool.call', () => ({ result: 'written' }))
  await start($, on)
  const out = await $.tool.call({
    tool: 'Edit',
    file_path: '/work/src/glue.rs',
    old_string: 'todo!()',
    new_string: 'modules().keystore_module.request_approval_with_timeout(&intent, t)?',
  })
  expect(out).toEqual({ result: 'written' })
})

test('a metadata.json that declares nothing in "uses" is refused', async ($, on) => {
  on('tool.call', () => ({ result: 'written' }))
  await start($, on)
  const out = await $.tool.call({
    tool: 'Write',
    file_path: '/work/metadata.json',
    content: JSON.stringify({ name: 'muster_ui', type: 'ui_qml', uses: ['evm.signing.approve'] }),
  })
  expect(out.deny).toContain('"uses" entry "evm.signing.approve" is a string')
})

test('an Edit to metadata.json is checked against the whole file, and a risk is a warning', async ($, on) => {
  on('tool.call', () => ({ result: 'edited' }))
  const current = JSON.stringify({ name: 'muster_module', dependencies: [{ name: 'keystore_module', version: '~0.1.0' }] }, null, 2)
  await start($, on, { '/work/metadata.json': current })
  const out = await $.tool.call({
    tool: 'Edit',
    file_path: '/work/metadata.json',
    old_string: '"version": "~0.1.0"\n    }',
    new_string: '"version": "~0.1.0"\n    },\n    "tx_sender_module"',
  })
  expect(out.result).toBe('edited')
  expect(out.context?.[0]).toContain('tx_sender_module has no version range')
})

// ---------------------------------------------------------------------------
// /atlas

test('/atlas with a module name answers in the transcript', async ($, on) => {
  await start($, on)
  const out = await $.command.run({ command: 'atlas', args: 'tx_sender_module' })
  expect(out.text).toContain('Depends on: keystore_module ~0.1.0.')
})

test('a module with no display name and no contract still reads well', async ($, on) => {
  await start($, on)
  const out = await $.command.run({ command: 'atlas', args: 'evm_signer_ui' })
  expect(out.text).toMatch(/^evm_signer_ui \(ui_qml, cpp, evm-wallet stack\)/)
  expect(out.text).toContain('No LIDL contract. Methods from its interface source (1):')
  expect(out.text).not.toContain('null')
})

test('/atlas with words searches', async ($, on) => {
  await start($, on)
  const out = await $.command.run({ command: 'atlas', args: 'bundle' })
  expect(out.text).toContain('tx_sender_module (evm-wallet)')
})

const PANE = {
  plugin: 'logos-module-atlas',
  component: 'Pane',
  requestId: 'atlas',
  viewport: { columns: 120, rows: 40 },
  props: { title: 'Logos Module Atlas', isFocused: true, bodyColumns: 80, placement: 'inline', scroll: { offset: 0, bodyRows: 30 }, view: {} },
} as const

test('the pane lists, searches, opens a module, follows a dependency and drafts a prompt', async ($, on) => {
  const filled: string[] = []
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('ui.close', () => ({ value: undefined }))
  on('prompt.fill', ($, e) => {
    filled.push(e.text)
    return { isFilled: true }
  })
  await start($, on)
  await $.command.run({ command: 'atlas', args: '' })

  for (const surface of ['terminal', 'desktop'] as const) {
    const ui = await $.ui.mount({ ...PANE, surface })
    expect(await ui.find({ type: 'Text', text: 'EVM wallet (3)' })).toBeDefined()
    await ui.input({ key: 'search', text: 'approve' })
    expect(await ui.find({ type: 'Text', text: 'methods: request_approval' })).toBeDefined()
    await ui.input({ key: 'search', text: '' })
    await ui.press({ key: 'm-tx_sender_module' })
    expect(await ui.find({ type: 'Text', text: 'tx_sender_module: EVM Transaction Sender' })).toBeDefined()
    await ui.press({ key: 'dep-keystore_module' })
    expect(await ui.find({ type: 'Text', text: 'Events (1)' })).toBeDefined()
    await ui.press({ key: 'back' })
    expect(await ui.find({ key: 'search' })).toBeDefined()
    await ui.unmount()
  }

  const ui = await $.ui.mount({ ...PANE, surface: 'terminal' })
  await ui.press({ key: 'm-keystore_module' })
  await ui.press({ key: 'draft' })
  expect(filled).toEqual(['Using keystore_module from the Logos Module Atlas, '])
})

test('/atlas with nothing to draw on prints the overview', async ($, on) => {
  on('fs.read', ($, e) => ({ value: JSON.stringify(REGISTRY) }))
  on('tool.register', () => ({ value: undefined }))
  on('command.register', () => ({ value: undefined }))
  on('session.start', () => ({ cwd: '/work' }))
  await $.session.start({ surface: null, isInteractive: false, cwd: '/work' })
  const out = await $.command.run({ command: 'atlas', args: '' })
  expect(out.text).toContain('EVM wallet: keystore_module, tx_sender_module, evm_signer_ui')
})
