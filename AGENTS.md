# Logos Module Atlas — guide for AI agents

This repo describes **every Logos module the latest Logos Basecamp release makes
available**: the modules bundled inside the app, plus every package its default
catalog ([`logos-co/logos-modules-release`](https://github.com/logos-co/logos-modules-release))
offers to install. Every fact is pinned to the source commit that was actually
released. It is regenerated daily from the release and the catalog, so treat it as
current unless `registry.json` → `generatedAt` is old.

Use it when you are writing code that **calls, depends on, or replaces** a Logos
module and need to know what exists, what it does, and its exact API.

## Finding what you need

In Claude Code 2.1.287 or later with this plugin installed, the tools
`mcp__logos-module-atlas__search`, `__module` and `__method` answer steps 1–3 from
the same files in one call each. Everything below still applies to reading the
files directly.

1. **Find candidates in `registry.json`.** It has one record per module: availability
   (bundled and/or in the catalog), versions, platforms, dependencies both ways, the
   source repo and commit, and an API summary.

   ```bash
   jq -r '.modules[] | "\(.stack)\t\(.name)\t\(.type)\t\(.description)"' registry.json   # everything
   jq '.modules[] | select(.name=="keystore_module")' registry.json                       # one module
   jq -r '.modules[] | .name as $n | .api.methods[]? | select(.name|test("sign|approv"))
          | "\($n).\(.name)  \(.summary)"' registry.json                                  # search methods
   grep -l 'method .*address' modules/*/*.lidl                                            # search contracts
   ```

2. **Read the module card**, `modules/<name>/README.md`: what it is, where it ships,
   its version and platforms, what it depends on and what depends on it, an API table,
   and links to everything below.

3. **Read the contract**, `modules/<name>/<name>.lidl`. It is the canonical interface,
   built from the pinned commit with `nix build <flake>#lidl`, and it is what the C++,
   Rust and Nim SDKs generate typed clients from. **Never invent method names or
   payload fields.** Many methods take and return JSON in a `tstr`. That JSON's shape
   is documented only in the method's `description`, so read it.

4. **Read the annotated source**, `modules/<name>/interface.{rs,h,lidl,rep}`. It is
   the Rust trait, C++ impl header, hand-written `.lidl` or Qt `.rep` that the contract
   was derived from, with its full doc comments. Use it when the contract is missing
   or you need the surrounding commentary.

5. **Read the stack doc**, `stacks/<stack>.md`. It covers how modules compose, who
   holds keys, what is ungated, step-by-step flows, and a section on using the stack
   from another module.

6. **Before adding a dependency**, read `guides/calling-official-modules.md` (how to
   call from C++/Rust/Nim/QML) and `guides/compatibility.md` (which SDK/builder
   generation and package variant must match, or your module will not load).

7. **Need more depth?** Clone the source at the pinned commit given on the card:
   `git clone --filter=blob:none https://github.com/<repo> && git -C <dir> checkout <commit>`.
   The source repo's HEAD may already be past the released version, so read the
   pinned commit.

If a module you expected is missing, check `gaps.md` before concluding it doesn't
exist. It records roadmap items, such as the Bitcoin and Zcash wallets, that are not
in the release, and where any work on them lives.

## Layout

| Path | What | Who writes it |
|---|---|---|
| `registry.json` | Machine-readable registry | `scripts/build_registry.py` |
| `llms.txt` | Index of everything with raw URLs ([llmstxt.org](https://llmstxt.org)) | `scripts/build_registry.py` |
| `modules/<name>/README.md` | Module card | `scripts/build_registry.py` |
| `modules/<name>/interface.*` | Annotated interface source, extracted | `scripts/build_registry.py` |
| `modules/<name>/<name>.lidl` | Canonical contract (+ `.contract-commit` stamp) | `scripts/fetch_contracts.py` |
| `modules/<name>/NOTES.md` | Curated per-module notes (optional) | humans / agents |
| `stacks/*.md` | How modules compose, trust boundaries, flows | humans / agents |
| `guides/*.md` | Calling modules, compatibility | humans / agents |
| `gaps.md` | Roadmap vs. what ships | humans / agents |
| `data/stacks.json` | Module → stack assignment | humans |
| `data/contracts.json` | Per-module contract build result | `scripts/fetch_contracts.py` |
| `hooks/register.js`, `hooks/atlas.js` | Claude Code mod: atlas tools, prompt context, edit checks, `/atlas` | humans / agents, never the generator |
| `tests/*.test.ts` | The mod's tests (`claude plugin test`) | humans / agents |
| `scripts/check_mod.mjs` | Checks the mod against the real registry and contracts | humans / agents |
| `scripts/check_drift.py`, `scripts/issue.sh` | Reports hand-written docs the release has left behind, as an `atlas:drift` issue | humans / agents |
| `.github/claude-code-version` | The Claude Code version CI runs the mod's checks on | humans / agents |

Generated files are overwritten on every refresh, so never hand-edit them. Put
corrections in `NOTES.md`, a stack doc, or the generator.

## Refreshing

```bash
scripts/refresh.sh          # build_registry → fetch_contracts (nix) → build_registry
```

`.github/workflows/refresh.yml` runs this daily and commits any changes. A new
Basecamp release, a new catalog package or a version bump all flow in automatically.
The hand-written docs do not. After each refresh, `scripts/check_drift.py` compares the
stack docs' "Pinned at" lines, the guides' "Verified against" lines, the release each doc
names, and `data/stacks.json` with the registry. It keeps one GitHub issue labelled
`atlas:drift` open listing what's behind. To work through it, run
`python3 scripts/check_drift.py` locally: it prints the same list and exits 0 once
nothing is left.
