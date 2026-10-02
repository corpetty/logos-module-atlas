---
name: logos-module-atlas
description: Look up official Logos modules (Logos Core / Basecamp) before calling, depending on, or reimplementing one — what exists, which stack it belongs to, its exact LIDL contract, who holds keys, how to call it from C++/Rust/Nim/QML, and which SDK generation must match. Use whenever a task touches a Logos module by name (keystore_module, tx_sender_module, monero_wallet_core_module, lez_core, delivery_module, chat_module, storage_module, package_manager, capability_module, …), a module's metadata.json dependencies, lp_* / LogosAPI calls between modules, .lidl contracts, or asks "is there a Logos module that does X".
---

# Logos Module Atlas

The atlas is a registry of every module the latest Logos Basecamp release makes
available, both bundled and installable from the default catalog. Everything is
pinned to the commit that was released.

**Atlas root:** `${CLAUDE_PLUGIN_ROOT}`. If that placeholder was not substituted,
the root is two directories above this skill's base directory (`../../`). If neither
is readable, fetch the same paths from
`https://raw.githubusercontent.com/corpetty/logos-module-atlas/main/<path>`.

## How to answer with it

If the tools `mcp__logos-module-atlas__search`, `mcp__logos-module-atlas__module` and
`mcp__logos-module-atlas__method` are available (the plugin's mod, Claude Code
2.1.287 or later), use them for steps 2 and 3. They read the same registry and
contracts, and `method` returns the full description, which is where JSON payload
shapes are documented. Read the stack docs and guides from the paths they return.

1. **Check freshness.** Read `registry.json` → `.basecamp.tag` and `.generatedAt`. If
   it is more than a couple of weeks old and the task depends on recent versions, tell
   the user. A local clone refreshes with `git pull` or `scripts/refresh.sh`; the
   plugin refreshes with `/plugin marketplace update logos-module-atlas`.
2. **Find the module(s)** in `registry.json`:
   - one module: `jq '.modules[] | select(.name=="<name>")' registry.json`
   - a whole stack: `jq -r '.modules[] | select(.stack=="evm-wallet") | "\(.name)\t\(.description)"' registry.json`
   - by capability: `jq -r '.modules[] | .name as $n | .api.methods[]? | select(.name|test("<word>")) | "\($n).\(.name): \(.summary)"' registry.json`,
     or grep `modules/*/*.lidl`
3. **Read, in this order:**
   1. `modules/<name>/README.md`, the card: availability, version, platforms, deps
      both ways, source commit, API table.
   2. `modules/<name>/<name>.lidl`, the canonical contract. Take method names,
      parameter types and payload shapes from here **only**. JSON-in-`tstr` payloads
      are described in each method's `description`.
   3. `modules/<name>/interface.*`: annotated source when you need more than the contract.
   4. `stacks/<stack>.md`: trust boundaries (who holds keys, what is ungated),
      flows, and "using this from another module".
   5. `stacks/key-custody.md` for anything about keys, signing or approval across chains.
4. **Before writing integration code**, read `guides/calling-official-modules.md`
   (the call pattern per language, `metadata.json` deps, events, caller identity,
   intents) and `guides/compatibility.md` (SDK/builder generation and package
   variant that must match Basecamp, or the module crashes or is rejected on load).
5. **If it seems missing**, read `gaps.md` before saying so. Bitcoin and Zcash wallets,
   for example, are on the v0.3 roadmap but not in the release.

## Rules

- Cite the pinned commit, not the source repo's HEAD. HEAD may already be past
  the released version.
- Don't guess an API. If the contract and interface don't answer the question,
  clone the source at the pinned commit (the card links it) and read it.
- Keep custody boundaries intact. When a stack doc says a module is the only holder
  of keys or passwords, design around it. Don't propose copying key material into
  the caller.
