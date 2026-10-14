# Compatibility: loading a third-party module next to the Basecamp 0.3.2 catalog

What has to line up for your module to build, load and be callable in **Logos Basecamp 0.3.2** (tag `0.3.2` = [`f2fae63`][bc], released 2026-10-06) alongside the six bundled modules and the 46 catalog packages. It also covers how to check what generation a module was built on, how to pin one, and how to install a local build for testing.

Companion guide: [`calling-official-modules.md`](calling-official-modules.md), which covers how to call a module once it loads. All facts were checked on 2026-10-14 against the commits linked here, unless marked **unverified**. The Muster observations (§2, §3, §6) date from 2026-10-01 and were not re-checked. This guide was first written against the 0.3.1 release (`aeb8192`); where 0.3.2 changed something, the text says so.

---

## 0. The rules, in the order they bite

| # | Rule | Enforced by | Failure mode |
|---|---|---|---|
| 1 | Same logos-protocol **MAJOR** as the host | liblogos load gate ([module_manager.cpp][ll-gate]) | "Refusing to load module … incompatible protocol majors". Everything is `0.x` today, so **this gate passes everything and protects nobody** |
| 2 | Your language backend **defines every module-impl export the generated glue calls**. The glue calls exports conditionally on the protocol MINOR it was generated for | Nothing at build time. Linux `dlopen` with `-z now` | `undefined symbol: logos_module_…` and "Module process crashed" on **Linux only**. macOS links with `-undefined dynamic_lookup` and hides it ([logos_module_impl.h][abi-teardown]) |
| 3 | **One** logos-protocol and **one** logos-qt-host on every link line | `follows` in the flakes | Two protocols on one link line give an 8-byte heap overrun on every `getClient()`: silent on macOS, fatal on glibc ([Basecamp flake.nix][bc-one-proto]). A plugin-qt older than the protocol fails to **compile**, because `logos_consumer.h` upper-bounds the MINOR ([builder flake.nix][b-upper]) |
| 4 | A `ui_qml` view plugin is built on the **same SDK generation** as Basecamp's `ui-host` and view-module runtime | Nothing | `std::bad_alloc` in `ui-host`, or "Timeout waiting for ui-host" ([muster labbook][m-lab-ui]) |
| 5 | Your builder's **LIDL parser understands your dependencies' contracts** | Build | Parse error on e.g. `optional_depends` (logos-lidl ≥ [`c0d9f9c`][lidl-optdeps], 2026-09-07) |
| 6 | The package **variant** matches the host build (`linux-amd64` for the release AppImage, `linux-amd64-dev` for a nix dev build) | lgpm / logos-module variant lists | "has no variant matching this platform: tried […], package provides […]" ([lgpm][lgpm-mismatch]) |
| 7 | Each declared dependency `version` range is satisfied by the installed version | liblogos dependency gate ([dependency_gate.h][dep-gate]) | Load refused, failing closed |

Rule 2 is the one that broke Muster, and the protocol header says it has broken the platform twice: `grant_host_services` at 0.3 and the teardown pair at 0.5.

---

## 1. What Basecamp 0.3.2 and the builder release are built from

**Basecamp 0.3.2 root pins** (`git show 0.3.2:flake.lock`; the 0.3.1 rev is in the notes where it moved):

| Input | Rev | Notes |
|---|---|---|
| logos-protocol | `9d59f5de` | **0.9.0**, still 12 module-impl exports. 0.3.1: `8bbc027c`, also 0.9.0. Since then an unknown method name is answered with the `unknown_method` error object, not NULL (#462), and a replica keeps only the completions it may be waiting for (#475) |
| logos-cpp-sdk | `3bf5365c` | Exactly builder `0.3.3`'s. Adds "fix(lidl-gen): refuse an unknown method name instead of answering NULL". 0.3.1: `c24c4ab9`, one commit past builder 0.3.1's `3f34c0b2` |
| logos-plugin-qt | `2d36a088` | qt-host + the cdylib glue generator. The glue now answers `unknown_method` when a backend returns NULL for a name it does not list, and makes deferred-call ids unique per module instance. 0.3.1: `3a471be1` |
| logos-qt-sdk | `b0c57af8` | 0.3.1: `03e489b1` |
| logos-liblogos | `00f19ef8` | Loader, protocol gate, dependency gate. Now registers a module's token with capability_module before the child holds it, so a call from the module's own init is not refused. 0.3.1: `db45024f` |
| logos-module | `f71d16dd` | Unchanged. Module directory and variant resolution, `lm` |
| logos-module-loader-qt | `dc2b3dfa` | Relock only. 0.3.1: `888da92c` |
| logos-view-module-runtime | `4c429723` | `ui-host`, `LogosQmlBridge`, the frozen intents surface. A provider's refusal now reaches QML as an error, not a value. On Windows, `ui-host` now dies with the app (#467). 0.3.1: `91dc59b3` |
| logos-package-manager | `4771a648` | lgpm library. Newest-version-wins scan (#479), see §5. 0.3.1: `3133786e` |
| logos-package | `c25a1167` | lgx library. Streams `Package::load` (#456). 0.3.1: `d759f34b` |
| logos-nix | `7c1eb8bc` | Unchanged, so nixpkgs, which follows it, and Qt are unchanged |
| nix-bundle-logos-module-install | `4ebf721f` | Relock only. 0.3.1: `19b27700` |
| logos-qt-mcp | `8885d24a` | UI-test inspector only (#464). Shipping builds set `enableInspector = false`. 0.3.1: `1fea5094` |

**Bundled modules** (`installedDev` for the dev `app`, `installPortable` for `appDistributed`, which is what the AppImage is built from; [flake.nix][bc-installed]):

| Module | Repo@rev | Builder | Protocol |
|---|---|---|---|
| package_manager | logos-package-manager-module@`4c5fa074` | `62861b94` | 0.9.0 (`46386344`) |
| package_downloader | logos-package-downloader-module@`7b0ca8b7` | `3a64943f` (`0.3.3`) | 0.9.0 (`9d59f5de`) |
| capability_module | logos-capability-module@`1a1b8b5a` | `de169fd4` | 0.9.0 (`8bbc027c`) |
| modules_state | logos-modules-state-module@`c70da49a` | `3818566d` | 0.9.0 (`d8389858`) |
| package_manager_ui | logos-package-manager-ui@`8b67bf02` | `fc31329d` | 0.9.0 (`8bbc027c`) |
| storage_module | logos-storage-module@`7548f41a` (v3.0.2) | `fb8d5513` | 0.9.0 (`fdc09ff5`) |

Since 0.3.1, every bundled module except capability_module has moved:

- package_downloader moved to builder `0.3.3` and gained `start()`, which Basecamp calls as soon as the module is loaded (#469, #474).
- modules_state moved to builder `3818566d` and package_manager_ui to `fc31329d`.
- package_manager was relocked onto the newest-version-wins scan, with the same builder.
- storage_module moved from v3.0.0 (`a9c14b8c`) to v3.0.2, with the same builder.

The six modules still span four protocol revisions, all 0.9.0.

**logos-module-builder release tags:**

| Tag | Commit | cpp-sdk | protocol | plugin-qt | qt-sdk | rust-sdk | view-module-runtime |
|---|---|---|---|---|---|---|---|
| `0.3.1` | `16e2f6bd` | `3f34c0b2` | `8bbc027c` (0.9.0) | `3a471be1` | `03e489b1` | `b572e817` | `42d91408` |
| `0.3.2` | `4b799827` | `3f34c0b2` | `8bbc027c` (0.9.0) | `3a471be1` | `03e489b1` | `569ecc5e` | `42d91408` |
| `0.3.3` | `3a64943f` | `3bf5365c` | `9d59f5de` (0.9.0) | `2d36a088` | `b0c57af8` | `040e94e8` | `fd902f81` |

Builder `0.3.3` (commit of 2026-10-03, released 2026-10-06, the same day as Basecamp 0.3.2) matches Basecamp 0.3.2 on protocol, cpp-sdk, plugin-qt, qt-sdk and logos-nix. Only view-module-runtime differs: `fd902f81` is the parent of Basecamp's `4c429723`, and the one commit between them is the Windows `ui-host` fix. **This is "the 0.3.2 generation".** Despite its name, builder `0.3.2` is the 0.3.1 generation with a newer rust-sdk. No builder release is newer than `0.3.3`. Master (`5f4c4f2`, 2026-10-13) has unreleased commits.

**What 0.3.2 changes for a third-party module:**

- **Loading: nothing.** The protocol MAJOR is still 0 and the version still 0.9.0, so rule 1 admits what it admitted before. The module-impl ABI still declares the same 12 exports, including the unload-callback pair. The glue at plugin-qt `2d36a088` calls the same 12 symbols as at `3a471be1`, so rule 2 asks nothing new. A module built for 0.3.1 loads in 0.3.2.
- **Unknown method names.** C++ and Rust backends built on builder `0.3.3` answer an unknown name with `{"code":"unknown_method","message":…,"origin":…}` as the result ([logos_module_impl.h][abi]). Older-generation modules still answer a bare null, the same bytes as a method that legitimately returns null, and that is most of the catalog (below). So a caller can rely on the error object only from a `0.3.3`-generation callee. QML views get the refusal as an error.
- **Calls across a module reload.** Since #475, a replica keeps only the completions it may be waiting for, so a call made after a module reload no longer gets another call's old result. Glue built on `0.3.3` also stops a reloaded module reusing `lc-0`-style call ids. Rebuilding on `0.3.3` gets you both halves. Whether a module on older glue is fully covered by the host-side half alone is **unverified**.

### The catalog is not one generation

Resolved at today's catalog (index 2026-10-14T12:39Z), from each package's pinned source (`registry.json` → `.modules[].source`, including its `subdir`). Each one's `flake.lock` → its builder → that builder's SDK inputs. [`data/catalog-pins.tsv`](../data/catalog-pins.tsv) is the 2026-10-01 snapshot. Since then, 18 of the 46 pins have moved.

| Generation | protocol (rev) | cpp-sdk | Catalog modules |
|---|---|---|---|
| 0.9, Oct 3 (builder `0.3.3`) | 0.9.0 (`9d59f5de`) | `3bf5365c` | **lez_core**, lez_wallet_ui, storage_ui |
| 0.9, Sept 17–23 | 0.9.0 (`8bbc027c`) | `3f34c0b2` | blockchain module+ui, chat module+ui, delivery_module, delivery_demo, lez_explorer_ui, libp2p, monerod module+ui, openmetrics, uniswap_backend |
| 0.9, Sept 8–16 | 0.9.0 (`46386344`, `fdc09ff5`) | `e1d6bf20`, `fb88c7d5`, `50017552`, `7e29573e` | **every EVM wallet module** (keystore, tx_sender, eth_rpc, fee, token_list, uniswap, evm_assets, eth_wallet backend+ui, evm signer/keystore UIs and CLIs, eth_rpc_ui, token_list_ui), **every Monero wallet module** (core, backend, node, cli, ui), storage_module, the RLN modules (`liblogos_rln_module`, `liblogos_lez_rln_module`: builder `9f420c29` in each subdir's lock), accounts_ui, uniswap_ui, verified_proxy module+ui |
| 0.2 | — | — | None. In 0.3.1: lez_core and openmetrics |
| 0.1 | 0.1.0 (`9de4165a`) | `1bc101df` | lez_indexer_module |
| Pre-protocol | None. Builder `d5ac0dce` (2026-06-10) has no logos-protocol input | `40e76314` | logos_execution_zone 1.0.0 |
| ? | not resolvable from the lock | — | amm module+ui (builder nested under `lez_programs`). **Unverified** |

Consequences:

- Rule 1 admits all of these. The gate loads a module with no protocol stamp "permissively with a warning". Whether logos_execution_zone's plugin carries a stamp is **unverified**.
- Only three catalog packages were rebuilt on the 0.3.2 generation. Core modules at 0.1 and pre-protocol ship next to 0.9 ones. Whether each has been exercised against the 0.3.2 host is **unverified**.
- A 0.2-generation caller works against a 0.9-generation callee in at least one tested case: Muster (0.2 glue) → `delivery_module` v0.3.0 (0.9) passed Muster's `scripts/ui-parity.sh` 8/8 in the standalone runner on 2026-10-01. The catalog now ships delivery_module 0.3.2, still on the Sept 17–23 generation.
- **UI modules all sit on 0.9 builders from Sept 8 to Oct 3.** Their view-module-runtime is `3ab072ea`, `42d91408`, `db31d118`, `da1069ae` or `fd902f81`, never Basecamp's exact `4c429723`. Rule 4 is therefore "same generation", not "same hash".

---

## 2. The "unload-callback crash": what actually happened

Muster's labbook ([`basecamp-sdk-skew-unload-callback.md`][m-lab-unload]) describes it as a host/module disagreement over who exports `logos_module_set_unload_done_callback`. The protocol sources show a more precise mechanism.

1. A `cdylib` module's `.so` holds two halves:
   - The **generated Qt glue**, emitted by `logos-plugin-qt`'s `qt-host-generator` from your `.lidl`.
   - Your **language backend's** module-impl C ABI: the cpp-sdk scaffold, the rust-sdk scaffold, or logos-nim-sdk's `lidl-gen provider`.
2. The glue makes **direct calls**, with no `dlsym` and no null check, guarded on the protocol MINOR it was compiled against ([glue generator][glue-guards]):

   | Export(s) | Glue calls it from MINOR |
   |---|---|
   | `grant_host_services` | ≥ 0.3 |
   | `set_unload_done_callback`, `about_to_unload` | ≥ 0.5 |
   | `set_call_caller` | ≥ 0.6 |
   | `accept_inbound_token` | ≥ 0.8 |

3. At 0.9.0 the ABI has **12** exports ([logos_module_impl.h][abi], list via `#module-impl-abi`, §3). The C++ and Rust backends define all of them, and each runs a CI diff against the protocol's export list.
4. **logos-nim-sdk `6077eb7` defines 7.** These are the 0.2 set ([gen.nim][nim-gen-provider]). It also hard-codes `get_protocol_version` to `"0.1.0"`.
5. Muster's module builder at the time was the fork `720ac2f` (PR #226), which is **protocol 0.9.0** (`48afc01c`, checked). Its glue therefore called `logos_module_set_unload_done_callback`, the Nim backend did not define it, and the `.so` linked with `U logos_module_set_unload_done_callback`. On Linux, `dlopen` failed.
6. The fix that shipped was to rebuild on a **protocol 0.2.0** builder: `corpetty/logos-module-builder@c10a94c` with every SDK input overridden to `cpp-sdk c3fa1b5a`, `protocol 6401e30a`, and so on. That glue calls only the 7 exports, so the module loads.

**The general rule.** Pick the builder generation by **what your language backend can define**, not by what the host runs. The host accepts any `0.x` module (rule 1). The two cpp-sdk revisions named in the comment, `58c65737` and `8554ecd7`, both have the C++ backend *defining* the symbol ([lidl_gen_cdylib.cpp][cpp-unload-def]). The ABI declares it a **module** export, so the host is never meant to provide it. "Make the host export it" is therefore not a fix.

**On 0.3.2 it is still there**, for any backend that does not define the pair. 0.3.2 fixed nothing in the mechanism:

- Protocol `9d59f5de` declares the same 12 exports.
- The glue at plugin-qt `2d36a088` still calls `logos_module_set_unload_done_callback` directly under MINOR ≥ 5 ([glue][glue-unload]).
- logos-nim-sdk is still at `6077eb7`, with no commits since 2026-09-29, and defines 7.
- The C++ backend at Basecamp's cpp-sdk `3bf5365c` defines the symbol ([lidl_gen_cdylib.cpp][cpp-unload-def-032]).

The fix is still on the module side (§6). One related change: when a backend's dispatch returns NULL, `0.3.3` glue checks `logos_module_get_methods`, one of the 7. If the name is not listed, it answers `unknown_method`; otherwise it returns null as before. The glue's own comment names a hand-written Nim backend as the case it covers. A protocol-0.2 builder like Muster's does not get this.

---

## 3. How to check a module's generation

**From source (`flake.lock`).** Follow `root → logos-module-builder → logos-protocol`:

```bash
jq -r '.nodes as $n | $n[$n.root.inputs["logos-module-builder"]].inputs["logos-protocol"] as $p
       | if ($p|type)=="string" then $n[$p].locked.rev else "follows: \($p)" end' flake.lock
# then map rev → version:
gh api "repos/logos-co/logos-protocol/contents/cpp/logos_protocol.h?ref=<rev>" \
   -H 'Accept: application/vnd.github.raw' | grep '#define LOGOS_PROTOCOL_VERSION_STRING'
```

- Repeat the same walk for `logos-cpp-sdk`, `logos-plugin-qt`, `logos-qt-sdk` and `logos-view-module-runtime`.
- `nix flake metadata --json` shows the same graph.
- A builder with no `logos-protocol` input at all is pre-protocol (logos_execution_zone's `d5ac0dce`, above).
- In a monorepo the package's lock is the one in its `subdir` (the RLN modules), not the root one.
- A gitignored `flake.lock` (Muster's `module/` ignores it) means the effective set is whatever `flake.nix` pins by rev, plus each input's own lock for everything not overridden.

**From a built plugin:**

```bash
nix build github:logos-co/logos-module#lm && ./result/bin/lm metadata <plugin.so> --json      # embedded metadata
strings <plugin.so> | grep -o 'logos_protocol_version.\{0,7\}'   # e.g. "logos_protocol_versione0.2.0" (CBOR; 'e' is a length byte)
nm -D --undefined-only <plugin.so> | grep logos_module_           # MUST print nothing — any line here is a dlopen failure on Linux
nm <plugin.so> | grep -c ' [Tt] logos_module_'                    # 7 = 0.2 set, 10 = 0.5, 12 = 0.8+ (static symtab; not present if stripped)
```

**Authoritative export list for a protocol revision** (Basecamp 0.3.2's):

```bash
nix build 'github:logos-co/logos-protocol/9d59f5de526624b120d42f4c422a3d83c0b9d64f#module-impl-abi'
cat result/version result/exports.txt   # 0.9.0 + the 12 names; result/bin/logos-module-impl-diff <declared> <defined> <label>
```

**Observed values** (checked 2026-10-01):

| Plugin | Protocol stamp | `logos_module_*` defined |
|---|---|---|
| Muster's `muster_module_plugin.so` (Sept 4 build) | `0.2.0` | 7 (`t`, local) |
| The `package_manager_plugin.so` baked into the Sept 4 Basecamp | `0.5.0` | 10 (`T`) |

---

## 4. Variants: portable vs dev

| | Dev | Portable |
|---|---|---|
| Builder output | `.#lgx`, `.#install` (nix-bundle-lgx default, nix-bundle-logos-module-install `.dev`) | `.#lgx-portable`, `.#install-portable` ([mkLogosModule][mk-outputs]) |
| Variant key in `manifest.json#main` | `linux-amd64-dev` (`darwin-arm64-dev`, …) | `linux-amd64`, `darwin-arm64`, `linux-arm64`, `windows-x86_64` |
| Runtime deps | `/nix/store` paths: runs only where that store exists | Self-contained: `nix-bundle-dir` co-locates the libraries |
| Host that accepts it | A nix-built **dev** Basecamp (`nix build` → `app`, `LOGOS_PORTABLE_BUILD=OFF`), dev `lgpm`, dev `logoscore` | The **release AppImage/DMG** (`appDistributed`, `portable = true`), portable `lgpm` (`cli-portable`), `logoscore`'s default resolver |
| What the catalog ships | — | Portable only (checked against the 2026-10-14 index: no `-dev` keys) |

**Why a dev-built module is rejected by the release AppImage.**

1. lgpm builds its candidate variant list from the host platform. In a non-portable build it **appends `-dev`** to every spelling ([package_manager_lib.cpp][lgpm-dev]). A portable host therefore tries `linux-amd64` and its arch aliases only.
2. A dev `.lgx` contains only `linux-amd64-dev`, so the install fails. The message is "package '…' has no variant matching this platform: tried [linux-amd64, …], package provides [linux-amd64-dev]" ([lgpm][lgpm-mismatch]).
3. A hand-placed dev module is "present-but-rejected" ("variant linux-amd64-dev not supported"). Observed when baking a dev-keyed module into the AppImage set.
4. In the other direction, a dev host reports a portable module as "Module not found in known modules" ([muster labbook][m-lab-ui]).
5. Even with the key forced, a dev plugin's `/nix/store` RUNPATH does not exist on a user's machine.

Ship and test against the AppImage with **`.#lgx-portable`**. Test against a nix-built Basecamp with **`.#lgx`**.

**Windows.**

- Basecamp 0.3.2 ships a Windows installer (`LogosBasecamp-Desktop-v0.3.2-f2fae6-x86_64-setup.exe`) next to the AppImages and the DMG.
- 34 of the 46 catalog packages ship a `windows-x86_64` variant.
- New in 0.3.2: Basecamp's UI doc-tests have Windows legs (#470, #471), and `ui-host` dies with the app on Windows (#467).
- Whether a third-party module's `windows-x86_64` variant installs and loads in the Windows build is **unverified**.

---

## 5. `.lgx` packaging and installing a local package

- **Build:**
  - `nix build .#lgx-portable` (or `.#lgx`).
  - `nix build github:logos-co/logos-package#lgx` provides the CLI. `lgx manifest X.lgx` prints name, version, manifest version, **variants** and dependencies. `lgx extract X.lgx --assets-only` gives `assets/lidl/<name>.lidl` on current bundlers ([lgx README][lgx-readme]).
  - Example check of Muster's portable `.lgx` (built 2026-09-23): manifest `0.3.0`, variant `linux-amd64`, deps `delivery_module`, `lez_core`, unsigned, no root assets. Of the catalog's 46 current releases, 42 are manifest `0.6.0`. amm module+ui are `0.3.0`, and lez_indexer_module and logos_execution_zone are `0.2.0`.
- **Install into Basecamp, GUI (release AppImage or dev build):**
  1. Install every dependency first, from the catalog. **A local `.lgx` install does not resolve dependencies** ([PMUI][pmui-local]).
  2. In Package Manager, use the "install local" action. It opens a file dialog titled "Select LGX Package to Install" ([PackageManager.qml][pmui-dialog]).
  3. PMUI inspects the manifest and decides between install, upgrade, downgrade and reinstall by version.
  4. Basecamp confirms, then installs.
- **A dependency only Basecamp ships.** If your package is in a catalog and depends on a bundled module, such as storage_module, that no enabled catalog lists, 0.3.2's App Manager keeps the installed copy if it satisfies the range ([#472][bc-472]). In 0.3.1 the install could not start, with "no candidate matches '<dependency>'".
- **Version decides which copy loads.** Since 0.3.2, when the same package name is in the app's embedded directory and the user directory, the scan keeps the **newest version**, and takes the user copy only on a tie ([package_manager_lib.cpp][lgpm-scan], #479). In 0.3.1 the user copy always won. A local build that replaces a bundled module therefore needs a version at least as high as the bundled one, for example storage_module ≥ 3.0.2.
- **Install into Basecamp, CLI:** run lgpm against the session directories. On Linux these are `~/.local/share/Logos/LogosBasecamp/{modules,plugins}`. A dev build adds a `Dev` suffix, and `--user-dir` or `LOGOS_USER_DIR` moves them ([README][bc-dirs]).
  ```bash
  lgpm --modules-dir ~/.local/share/Logos/LogosBasecamp/modules \
       --ui-plugins-dir ~/.local/share/Logos/LogosBasecamp/plugins install --file ./muster-module.lgx
  ```
  - Use **portable** lgpm for an AppImage's directories and dev lgpm for a dev build's.
  - Portable lgpm comes from `nix build github:logos-co/logos-package-manager#cli-portable`. There is also the prebuilt [lgpm `0.3.1` release][lgpm-rel] (2026-10-06): Basecamp 0.3.2's pin `4771a648` plus a version bump, with portable builds for Linux, macOS and Windows (`lgpm-x86_64-windows.zip`).
  - `--allow-unsigned` silences the unsigned-package warning, and nothing in the catalog is signed.
  - Whether a running Basecamp picks up a CLI install without a restart is **unverified**. Restart it.
- **Isolated test instance:** `LogosBasecamp --user-dir /tmp/bc-a` gives an instance with its own `plugins/`, `modules/`, `module_data/` and `logs/`. Logs go to `<session>/logs/`.
- **Baking into a Basecamp build (dev only):** add the module's flake as an input and put its `.install` output (dev set) or `.install-portable` output (AppImage set) into `installedDev` or `installedDistributed`. Pair each module's `.install` with the dev set and `.install-portable` with the AppImage set. Mixing them gives the variant rejection above. Keep such bakes out of upstream commits.

---

## 6. Pinning a third-party module to the 0.3.2 generation

**C++ (`universal`) or Rust (`cdylib` + `codegen.rust`): do not override anything.**

```nix
inputs.logos-module-builder.url = "github:logos-co/logos-module-builder/0.3.3";   # 3a64943f
# no inputs.logos-module-builder.inputs.* overrides — the tag's own lock IS the generation
```

1. Run `nix flake lock`.
2. Check (§3) that you resolved `protocol 9d59f5de`, `cpp-sdk 3bf5365c`, `plugin-qt 2d36a088`, `qt-sdk b0c57af8`.
3. Builder `0.3.3` already matches Basecamp 0.3.2's cpp-sdk, so the `logos-cpp-sdk` override this guide suggested for 0.3.1 is no longer needed. If you override anything, override a full coherent set, never a single input (rule 3).
4. Pin dependency module flakes by commit. The current pins are in `registry.json` → `.modules[].source`. [`data/catalog-pins.tsv`](../data/catalog-pins.tsv) is the 2026-10-01 snapshot.

Staying on builder `0.3.1` or `0.3.2` still loads in 0.3.2: the protocol and exports are the same. You lose `unknown_method` answers and per-instance call ids (§1).

**Nim (`codegen.nim`): blocked today.** Muster would need all of the following.

| Step | Where | What |
|---|---|---|
| 1 | logos-nim-sdk `lidl-gen/gen.nim` `providerExports` + `src/logos_sdk/ffi.nim` | Define the 5 missing exports:<br>• `logos_module_grant_host_services(json)` → forward to `lp_grant_host_services`<br>• `logos_module_set_unload_done_callback(cb, ud)` and `logos_module_about_to_unload()` → store `cb`, return `0`<br>• `logos_module_set_call_caller(json)` → a per-thread push/pop stack, plus a `currentCaller()` accessor<br>• `logos_module_accept_inbound_token(caller, token)` → forward to `lp_token_save_inbound`<br>Return `$lp_protocol_version()` from `get_protocol_version` instead of the hard-coded `"0.1.0"`. Bind `lp_grant_host_services` and `lp_token_save_inbound` ([logos_protocol.h][lp-h]). Answer an unknown name from `logos_module_dispatch` with the `unknown_method` error object: at protocol `9d59f5de`, NULL means a structural failure ([logos_module_impl.h][abi]) |
| 2 | logos-nim-sdk CI | Diff the generated provider against `#module-impl-abi`'s `exports.txt`, as the C++ and Rust SDKs do |
| 3 | logos-module-builder | Rebase PR #226's `nim.packages` and RUNPATH commits (`165839f`, `720ac2f`) onto tag `0.3.3`. `codegen.nim` itself (#202) is already upstream |
| 4 | Muster `module/flake.nix` | Point at the rebased builder, **drop every `inputs.logos-module-builder.inputs.*` override**, and drop `deliveryForModule`, because the `0.3.3` parser reads `optional_depends` |
| 5 | Muster `ui/flake.nix` | Builder `4717b9af` → `0.3.3` (tracked as muster `exo-817`). Rebuild and run `scripts/ui-parity.sh` |
| 6 | Verify | `nm -D --undefined-only muster_module_plugin.so \| grep logos_module_` is empty. Load in the release AppImage from `.#lgx-portable` |

**Until then**, Muster stays on its protocol-0.2 module builder. It is loadable (rule 1) and has been shown to call a 0.9 module (delivery v0.3.0, runner). Its UI should still move to a 0.9-generation builder for rule 4. Moving the UI is independent of the Nim module, because the UI is C++ (`universal` + `.rep`).

---

## 7. Checklist before you ship a module next to 0.3.2

- [ ] `protocol` resolves to `0.9.0` (`9d59f5de`, builder `0.3.3`), **or** you have a reason to stay older and know which exports your backend defines.
- [ ] `nm -D --undefined-only <plugin> | grep logos_module_` is empty, checked on **Linux**.
- [ ] One protocol and one qt-host in the closure: no partial SDK overrides.
- [ ] `ui_qml`: built on builder `0.3.3`. Builders `0.3.1` and `0.3.2` are the same 0.9 generation, one SDK wave older. The view renders in the release AppImage (not only in `logos-standalone-app`, which loads in-process with the access policy off).
- [ ] Every dependency's `.lidl` parses with your builder, and every dependency has a `version` range.
- [ ] `.#lgx-portable` installs through PMUI "install local" after its dependencies are installed from the catalog.
- [ ] If it replaces a bundled module, its version is at least the bundled one, or the bundled copy wins the scan.
- [ ] Callers handle an `unknown_method` error object, and do not count on one from an older-generation callee, which still answers null.
- [ ] It works under `--access-policy enforce`: every module you call is in `dependencies`.
- [ ] Outbound calls are attributed to your module name: callee `caller_identity()` or `current_caller()` shows `module/<you>`, not `host` or `unknown`.

---

[bc]: https://github.com/logos-co/logos-basecamp/tree/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9
[bc-472]: https://github.com/logos-co/logos-basecamp/commit/6b6f2e89e67c2f0dff5bf51726e03aa96dd95f26
[bc-one-proto]: https://github.com/logos-co/logos-basecamp/blob/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9/flake.nix#L15-L30
[bc-installed]: https://github.com/logos-co/logos-basecamp/blob/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9/flake.nix#L221-L239
[bc-dirs]: https://github.com/logos-co/logos-basecamp/blob/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9/README.md#L94-L165
[b-upper]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/flake.nix#L50-L62
[mk-outputs]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/lib/mkLogosModule.nix#L1160-L1176
[ll-gate]: https://github.com/logos-co/logos-liblogos/blob/00f19ef8e2ffd54e2cf97616a960cd19232ad4ba/src/logos_core/module_manager.cpp#L837-L870
[dep-gate]: https://github.com/logos-co/logos-liblogos/blob/00f19ef8e2ffd54e2cf97616a960cd19232ad4ba/src/logos_core/dependency_gate.h#L8-L24
[abi]: https://github.com/logos-co/logos-protocol/blob/9d59f5de526624b120d42f4c422a3d83c0b9d64f/cpp/logos_module_impl.h#L52-L297
[abi-teardown]: https://github.com/logos-co/logos-protocol/blob/9d59f5de526624b120d42f4c422a3d83c0b9d64f/cpp/logos_module_impl.h#L162-L190
[lp-h]: https://github.com/logos-co/logos-protocol/blob/9d59f5de526624b120d42f4c422a3d83c0b9d64f/cpp/logos_protocol.h#L739-L899
[glue-guards]: https://github.com/logos-co/logos-plugin-qt/blob/2d36a08819b8df36094036d02d130aa4bd3a6132/qt-host-generator/lidl_gen_cdylib_glue.cpp#L165-L195
[glue-unload]: https://github.com/logos-co/logos-plugin-qt/blob/2d36a08819b8df36094036d02d130aa4bd3a6132/qt-host-generator/lidl_gen_cdylib_glue.cpp#L670-L690
[cpp-unload-def]: https://github.com/logos-co/logos-cpp-sdk/blob/58c657375185b2768239d495ea8defab237b608e/cpp-generator/experimental/lidl_gen_cdylib.cpp#L1023
[cpp-unload-def-032]: https://github.com/logos-co/logos-cpp-sdk/blob/3bf5365c5412f89bbbf30e3f46237cd68cff0599/cpp-generator/experimental/lidl_gen_cdylib.cpp#L1055-L1062
[nim-gen-provider]: https://github.com/corpetty/logos-nim-sdk/blob/6077eb70a8709dccf3d60601121dbf66e794b89c/lidl-gen/gen.nim#L135-L180
[lidl-optdeps]: https://github.com/logos-co/logos-lidl/commit/c0d9f9c
[lgpm-dev]: https://github.com/logos-co/logos-package-manager/blob/4771a648952c8e3b991ed800ba551c6d72d6650b/src/package_manager_lib.cpp#L1450-L1459
[lgpm-mismatch]: https://github.com/logos-co/logos-package-manager/blob/4771a648952c8e3b991ed800ba551c6d72d6650b/src/package_manager_lib.cpp#L1544-L1569
[lgpm-scan]: https://github.com/logos-co/logos-package-manager/blob/4771a648952c8e3b991ed800ba551c6d72d6650b/src/package_manager_lib.cpp#L742-L809
[lgpm-rel]: https://github.com/logos-co/logos-package-manager/releases/tag/0.3.1
[lgx-readme]: https://github.com/logos-co/logos-package/blob/c25a1167578aef5cbd9a9b6f822ffe2ae4fd6a89/README.md
[pmui-local]: https://github.com/logos-co/logos-package-manager-ui/blob/8b67bf0213596e021d441e5d1d43f857142003b6/src/PackageManagerBackend.cpp#L1242-L1256
[pmui-dialog]: https://github.com/logos-co/logos-package-manager-ui/blob/8b67bf0213596e021d441e5d1d43f857142003b6/src/qml/PackageManager.qml#L302-L311
[m-lab-unload]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/docs/labbook/basecamp-sdk-skew-unload-callback.md
[m-lab-ui]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/docs/labbook/nim-cdylib-modules-cannot-load-in-basecamp-sdk-skew.md
