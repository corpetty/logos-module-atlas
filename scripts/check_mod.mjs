// Checks the mod's logic (hooks/atlas.js) against the real atlas, which its tests don't
// read: every contract the registry names parses to the same methods and events the
// generator found, every module renders, and the edit check flags a bad call. CI runs
// this on every change to the mod and after each refresh, before committing.
//
//   node scripts/check_mod.mjs
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { checkCalls, findCalls, findMentions, mentionContext, moduleSummary, overviewText, parseLidl, searchText } from '../hooks/atlas.js'

const root = fileURLToPath(new URL('..', import.meta.url)).replace(/\/$/, '')
const registry = JSON.parse(readFileSync(`${root}/registry.json`, 'utf8'))
const failures = []
const fail = (msg) => failures.push(msg)

for (const key of ['generatedAt', 'basecamp', 'stacks', 'modules']) if (!registry[key]) fail(`registry.json has no "${key}"`)
const modules = registry.modules ?? []
const byName = new Map(modules.map((m) => [m.name, m]))
const contracts = new Map()

for (const m of modules) {
  for (const key of ['name', 'type', 'stack', 'availability']) if (m[key] == null) fail(`${m.name}: no "${key}"`)
  if (!registry.stacks?.[m.stack]) fail(`${m.name}: stack "${m.stack}" is not in registry.stacks`)
  if (m.contract?.file) {
    let contract
    try {
      contract = parseLidl(readFileSync(`${root}/modules/${m.name}/${m.contract.file}`, 'utf8'))
    } catch (err) {
      fail(`${m.name}: can't read its contract: ${err.message}`)
      continue
    }
    contracts.set(m.name, contract)
    // The generator reads the same .lidl, so any difference is the mod's parser at fault
    for (const kind of ['methods', 'events']) {
      const ours = [...contract[kind].keys()].sort().join(' ')
      const theirs = (m.api?.[kind] ?? []).map((x) => x.name).sort().join(' ')
      if (ours !== theirs) fail(`${m.name}: the mod reads ${kind} [${ours}], the registry lists [${theirs}]`)
    }
  }
  try {
    moduleSummary(m, contracts.get(m.name) ?? null, root)
  } catch (err) {
    fail(`${m.name}: moduleSummary threw: ${err.message}`)
  }
}

try {
  overviewText(registry)
  searchText(registry, 'approval')
  mentionContext(findMentions(modules.map((m) => m.name).join(' '), byName).map((n) => byName.get(n)), registry, root, false)
} catch (err) {
  fail(`rendering threw: ${err.message}`)
}

// The edit check passes every real method and flags a made-up one
for (const [name, contract] of contracts) {
  const real = [...contract.methods.keys()].map((x) => `logos.callModule("${name}", "${x}")`).join('\n')
  const bad = checkCalls(findCalls(real + `\nlogos.callModule("${name}", "no_such_method_x")`), contracts)
  if (bad.length !== 1 || bad[0].member !== 'no_such_method_x') {
    fail(`${name}: the edit check flagged [${bad.map((p) => p.member).join(', ')}], expected [no_such_method_x]`)
  }
}

if (failures.length) {
  for (const f of failures) console.error(`::error::${f}`)
  process.exit(1)
}
const methods = [...contracts.values()].reduce((n, c) => n + c.methods.size, 0)
const events = [...contracts.values()].reduce((n, c) => n + c.events.size, 0)
console.log(`mod check passed: ${modules.length} modules, ${contracts.size} contracts, ${methods} methods, ${events} events`)
