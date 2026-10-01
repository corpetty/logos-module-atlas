# Compatibility: loading a third-party module next to the Basecamp 0.3.1 catalog

What has to line up for your module to build, load and be callable in **Logos Basecamp 0.3.1** (tag `0.3.1` = [`aeb8192`][bc], released 2026-10-01) alongside the six bundled modules and the 46 catalog packages. It also covers how to check what generation a module was built on, how to pin one, and how to install a local build for testing.

Companion guide: [`calling-official-modules.md`](calling-official-modules.md), which covers how to call a module once it loads. All facts were checked on 2026-10-01 against the commits linked here, unless marked **unverified**.

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

## 1. What Basecamp 0.3.1 and the builder release are built from

**Basecamp 0.3.1 root pins** (`git show 0.3.1:flake.lock`):

| Input | Rev | Notes |
|---|---|---|
| logos-protocol | `8bbc027c` | **0.9.0**. Requires 12 module-impl exports |
| logos-cpp-sdk | `c24c4ab9` | One commit after `3f34c0b2`: "fix(lidl-gen): answer name/version/lidl before the impl is touched" |
| logos-plugin-qt | `3a471be1` | qt-host + the cdylib glue generator |
| logos-qt-sdk | `03e489b1` | |
| logos-liblogos | `db45024f` | Loader, protocol gate, dependency gate |
| logos-module | `f71d16dd` | Module directory and variant resolution, `lm` |
| logos-module-loader-qt | `888da92c` | |
| logos-view-module-runtime | `91dc59b3` | `ui-host`, `LogosQmlBridge`, the frozen intents surface |
| logos-package-manager | `3133786e` | lgpm library |
| logos-nix | `7c1eb8bc` | nixpkgs follows this |
| nix-bundle-logos-module-install | `19b27700` | |

**Bundled modules** (`installedDev` for the dev `app`, `installPortable` for `appDistributed`, which is what the AppImage is built from; [flake.nix][bc-installed]):

| Module | Repo@rev | Builder | Protocol |
|---|---|---|---|
| package_manager | logos-package-manager-module@`41df424f` | `62861b94` | 0.9.0 (`46386344`) |
| package_downloader | logos-package-downloader-module@`d40d4ab9` | `4b799827` | 0.9.0 (`8bbc027c`) |
| capability_module | logos-capability-module@`1a1b8b5a` | `de169fd4` | 0.9.0 (`8bbc027c`) |
| modules_state | logos-modules-state-module@`ed0f4ba5` | `de169fd4` | 0.9.0 (`8bbc027c`) |
| package_manager_ui | logos-package-manager-ui@`3174edd0` | `16e2f6bd` | 0.9.0 (`8bbc027c`) |
| storage_module | logos-storage-module@`a9c14b8c` | `fb8d5513` | 0.9.0 (`fdc09ff5`) |

**logos-module-builder release tags:**

| Tag | Commit | cpp-sdk | protocol | plugin-qt | qt-sdk | rust-sdk | view-module-runtime |
|---|---|---|---|---|---|---|---|
| `0.3.1` | `16e2f6bd` | `3f34c0b2` | `8bbc027c` (0.9.0) | `3a471be1` | `03e489b1` | `b572e817` | `42d91408` |
| `0.3.2` | `4b799827` | `3f34c0b2` | `8bbc027c` (0.9.0) | `3a471be1` | `03e489b1` | `569ecc5e` | `42d91408` |

Builder `0.3.1` matches Basecamp 0.3.1 on protocol, plugin-qt, qt-sdk and logos-nix. **This is "the 0.3.1 generation".**

### The catalog is not one generation

Resolved from each catalog pin's `flake.lock` → its builder → that builder's SDK inputs ([`data/catalog-pins.tsv`](../data/catalog-pins.tsv)):

| Generation | protocol (rev) | cpp-sdk | Catalog modules |
|---|---|---|---|
| 0.9, Sept 17+ | 0.9.0 (`8bbc027c`) | `3f34c0b2` | blockchain module+ui, chat module+ui, delivery_module, delivery_demo, lez_explorer_ui, lez_wallet_ui, libp2p, monerod module+ui, uniswap_backend |
| 0.9, Sept 8–16 | 0.9.0 (`46386344`, `fdc09ff5`) | `e1d6bf20`, `fb88c7d5`, `50017552`, `7e29573e` | **every EVM wallet module** (keystore, tx_sender, eth_rpc, fee, token_list, uniswap, evm_assets, eth_wallet backend+ui, evm signer/keystore UIs and CLIs, eth_rpc_ui, token_list_ui), **every Monero wallet module** (core, backend, node, cli, ui), storage_ui, json_rpc_bridge, accounts_ui, uniswap_ui, verified_proxy module+ui |
| 0.2 | 0.2.0 (`976bc7a9`, `ae2f7e1b`) | `d12a7bbb`, `2e31eeb1` | **lez_core** (logos-execution-zone-module, builder `6ef42ea8` of 2026-07-01), openmetrics |
| 0.1 | 0.1.0 (`9de4165a`) | `1bc101df` | lez_indexer_module |
| ? | not resolvable from the lock | — | amm module+ui (builder nested under `lez_programs`), the RLN modules (pin cpp-sdk `25c88f4d` directly). **Unverified** |

Consequences:

- Rule 1 admits all of these.
- Core modules at 0.1 and 0.2 are shipped next to 0.9 ones. Whether each has been exercised against the 0.3.1 host is **unverified**.
- A 0.2-generation caller works against a 0.9-generation callee in at least one tested case: Muster (0.2 glue) → `delivery_module` v0.3.0 (0.9) passed Muster's `scripts/ui-parity.sh` 8/8 in the standalone runner on 2026-10-01.
- **UI modules all sit on Sept 0.9 builders.** Their view-module-runtime is `3ab072ea`, `42d91408`, `db31d118` or `da1069ae`, never Basecamp's exact `91dc59b3`. Rule 4 is therefore "same generation", not "same hash".

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
- A gitignored `flake.lock` (Muster's `module/` ignores it) means the effective set is whatever `flake.nix` pins by rev, plus each input's own lock for everything not overridden.

**From a built plugin:**

```bash
nix build github:logos-co/logos-module#lm && ./result/bin/lm metadata <plugin.so> --json      # embedded metadata
strings <plugin.so> | grep -o 'logos_protocol_version.\{0,7\}'   # e.g. "logos_protocol_versione0.2.0" (CBOR; 'e' is a length byte)
nm -D --undefined-only <plugin.so> | grep logos_module_           # MUST print nothing — any line here is a dlopen failure on Linux
nm <plugin.so> | grep -c ' [Tt] logos_module_'                    # 7 = 0.2 set, 10 = 0.5, 12 = 0.8+ (static symtab; not present if stripped)
```

**Authoritative export list for a protocol revision:**

```bash
nix build 'github:logos-co/logos-protocol/8bbc027c99505c3c2f043265f8c1ae8ff899bdff#module-impl-abi'
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
| What the catalog ships | — | Portable only (verified from the index: no `-dev` keys) |

**Why a dev-built module is rejected by the release AppImage.**

1. lgpm builds its candidate variant list from the host platform. In a non-portable build it **appends `-dev`** to every spelling ([package_manager_lib.cpp][lgpm-dev]). A portable host therefore tries `linux-amd64` and its arch aliases only.
2. A dev `.lgx` contains only `linux-amd64-dev`, so the install fails. The message is "package '…' has no variant matching this platform: tried [linux-amd64, …], package provides [linux-amd64-dev]" ([lgpm][lgpm-mismatch]).
3. A hand-placed dev module is "present-but-rejected" ("variant linux-amd64-dev not supported"). Observed when baking a dev-keyed module into the AppImage set.
4. In the other direction, a dev host reports a portable module as "Module not found in known modules" ([muster labbook][m-lab-ui]).
5. Even with the key forced, a dev plugin's `/nix/store` RUNPATH does not exist on a user's machine.

Ship and test against the AppImage with **`.#lgx-portable`**. Test against a nix-built Basecamp with **`.#lgx`**.

---

## 5. `.lgx` packaging and installing a local package

- **Build:**
  - `nix build .#lgx-portable` (or `.#lgx`).
  - `nix build github:logos-co/logos-package#lgx` provides the CLI. `lgx manifest X.lgx` prints name, version, manifest version, **variants** and dependencies. `lgx extract X.lgx --assets-only` gives `assets/lidl/<name>.lidl` on current bundlers ([lgx README][lgx-readme]).
  - Example check of Muster's portable `.lgx` (built 2026-09-23): manifest `0.3.0`, variant `linux-amd64`, deps `delivery_module`, `lez_core`, unsigned, no root assets. The catalog's current releases are manifest `0.6.0`.
- **Install into Basecamp, GUI (release AppImage or dev build):**
  1. Install every dependency first, from the catalog. **A local `.lgx` install does not resolve dependencies** ([PMUI][pmui-local]).
  2. In Package Manager, use the "install local" action. It opens a file dialog titled "Select LGX Package to Install" ([PackageManager.qml][pmui-dialog]).
  3. PMUI inspects the manifest and decides between install, upgrade, downgrade and reinstall by version.
  4. Basecamp confirms, then installs.
- **Install into Basecamp, CLI:** run lgpm against the session directories. On Linux these are `~/.local/share/Logos/LogosBasecamp/{modules,plugins}`. A dev build adds a `Dev` suffix, and `--user-dir` or `LOGOS_USER_DIR` moves them ([README][bc-dirs]).
  ```bash
  lgpm --modules-dir ~/.local/share/Logos/LogosBasecamp/modules \
       --ui-plugins-dir ~/.local/share/Logos/LogosBasecamp/plugins install --file ./muster-module.lgx
  ```
  Use **portable** lgpm (`nix build github:logos-co/logos-package-manager#cli-portable`) for an AppImage's directories and dev lgpm for a dev build's. `--allow-unsigned` silences the unsigned-package warning, and nothing in the catalog is signed. Whether a running Basecamp picks up a CLI install without a restart is **unverified**. Restart it.
- **Isolated test instance:** `LogosBasecamp --user-dir /tmp/bc-a` gives an instance with its own `plugins/`, `modules/`, `module_data/` and `logs/`. Logs go to `<session>/logs/`.
- **Baking into a Basecamp build (dev only):** add the module's flake as an input and put its `.install` output (dev set) or `.install-portable` output (AppImage set) into `installedDev` or `installedDistributed`. Pair each module's `.install` with the dev set and `.install-portable` with the AppImage set. Mixing them gives the variant rejection above. Keep such bakes out of upstream commits.

---

## 6. Pinning a third-party module to the 0.3.1 generation

**C++ (`universal`) or Rust (`cdylib` + `codegen.rust`): do not override anything.**

```nix
inputs.logos-module-builder.url = "github:logos-co/logos-module-builder/0.3.1";   # 16e2f6bd
# no inputs.logos-module-builder.inputs.* overrides — the tag's own lock IS the generation
```

1. Run `nix flake lock`.
2. Check (§3) that you resolved `protocol 8bbc027c`, `cpp-sdk 3f34c0b2`, `plugin-qt 3a471be1`, `qt-sdk 03e489b1`.
3. If you must match Basecamp exactly, add `inputs.logos-module-builder.inputs.logos-cpp-sdk.url = "github:logos-co/logos-cpp-sdk/c24c4ab9…"`. Override only a full coherent set, never a single input (rule 3).
4. Pin dependency module flakes by commit. The atlas pins are in [`data/catalog-pins.tsv`](../data/catalog-pins.tsv).

**Nim (`codegen.nim`): blocked today.** Muster would need all of the following.

| Step | Where | What |
|---|---|---|
| 1 | logos-nim-sdk `lidl-gen/gen.nim` `providerExports` + `src/logos_sdk/ffi.nim` | Define the 5 missing exports:<br>• `logos_module_grant_host_services(json)` → forward to `lp_grant_host_services`<br>• `logos_module_set_unload_done_callback(cb, ud)` and `logos_module_about_to_unload()` → store `cb`, return `0`<br>• `logos_module_set_call_caller(json)` → a per-thread push/pop stack, plus a `currentCaller()` accessor<br>• `logos_module_accept_inbound_token(caller, token)` → forward to `lp_token_save_inbound`<br>Return `$lp_protocol_version()` from `get_protocol_version` instead of the hard-coded `"0.1.0"`. Bind `lp_grant_host_services` and `lp_token_save_inbound` ([logos_protocol.h][lp-h]) |
| 2 | logos-nim-sdk CI | Diff the generated provider against `#module-impl-abi`'s `exports.txt`, as the C++ and Rust SDKs do |
| 3 | logos-module-builder | Rebase PR #226's `nim.packages` and RUNPATH commits (`165839f`, `720ac2f`) onto tag `0.3.1`. `codegen.nim` itself (#202) is already upstream |
| 4 | Muster `module/flake.nix` | Point at the rebased builder, **drop every `inputs.logos-module-builder.inputs.*` override**, and drop `deliveryForModule`, because the 0.3.1 parser reads `optional_depends` |
| 5 | Muster `ui/flake.nix` | Builder `4717b9af` → `0.3.1` (tracked as muster `exo-817`). Rebuild and run `scripts/ui-parity.sh` |
| 6 | Verify | `nm -D --undefined-only muster_module_plugin.so \| grep logos_module_` is empty. Load in the release AppImage from `.#lgx-portable` |

**Until then**, Muster stays on its protocol-0.2 module builder. It is loadable (rule 1) and has been shown to call a 0.9 module (delivery v0.3.0, runner). Its UI should still move to a Sept-generation builder for rule 4. Moving the UI is independent of the Nim module, because the UI is C++ (`universal` + `.rep`).

---

## 7. Checklist before you ship a module next to 0.3.1

- [ ] `protocol` resolves to `0.9.0` (`8bbc027c`), **or** you have a reason to stay older and know which exports your backend defines.
- [ ] `nm -D --undefined-only <plugin> | grep logos_module_` is empty, checked on **Linux**.
- [ ] One protocol and one qt-host in the closure: no partial SDK overrides.
- [ ] `ui_qml`: built on builder `0.3.1`/`0.3.2`. The view renders in the release AppImage (not only in `logos-standalone-app`, which loads in-process with the access policy off).
- [ ] Every dependency's `.lidl` parses with your builder, and every dependency has a `version` range.
- [ ] `.#lgx-portable` installs through PMUI "install local" after its dependencies are installed from the catalog.
- [ ] It works under `--access-policy enforce`: every module you call is in `dependencies`.
- [ ] Outbound calls are attributed to your module name: callee `caller_identity()` or `current_caller()` shows `module/<you>`, not `host` or `unknown`.

---

[bc]: https://github.com/logos-co/logos-basecamp/tree/aeb819216c1a54563d758e82fb2264359987dd2c
[bc-one-proto]: https://github.com/logos-co/logos-basecamp/blob/aeb819216c1a54563d758e82fb2264359987dd2c/flake.nix#L15-L30
[bc-installed]: https://github.com/logos-co/logos-basecamp/blob/aeb819216c1a54563d758e82fb2264359987dd2c/flake.nix#L225-L243
[bc-dirs]: https://github.com/logos-co/logos-basecamp/blob/aeb819216c1a54563d758e82fb2264359987dd2c/README.md#L94-L165
[b-upper]: https://github.com/logos-co/logos-module-builder/blob/4b7998272c5ec014bcac4bf1c7dbe7602c63a3c1/flake.nix#L50-L62
[mk-outputs]: https://github.com/logos-co/logos-module-builder/blob/4b7998272c5ec014bcac4bf1c7dbe7602c63a3c1/lib/mkLogosModule.nix#L1160-L1176
[ll-gate]: https://github.com/logos-co/logos-liblogos/blob/db45024f5c45ce5fc00ab76a6921985839ad7355/src/logos_core/module_manager.cpp#L807-L840
[dep-gate]: https://github.com/logos-co/logos-liblogos/blob/db45024f5c45ce5fc00ab76a6921985839ad7355/src/logos_core/dependency_gate.h#L8-L24
[abi]: https://github.com/logos-co/logos-protocol/blob/8bbc027c99505c3c2f043265f8c1ae8ff899bdff/cpp/logos_module_impl.h#L51-L295
[abi-teardown]: https://github.com/logos-co/logos-protocol/blob/8bbc027c99505c3c2f043265f8c1ae8ff899bdff/cpp/logos_module_impl.h#L160-L188
[lp-h]: https://github.com/logos-co/logos-protocol/blob/8bbc027c99505c3c2f043265f8c1ae8ff899bdff/cpp/logos_protocol.h#L740-L900
[glue-guards]: https://github.com/logos-co/logos-plugin-qt/blob/3a471be14af66d099827ee712ec8c40ead701340/qt-host-generator/lidl_gen_cdylib_glue.cpp#L160-L190
[cpp-unload-def]: https://github.com/logos-co/logos-cpp-sdk/blob/58c657375185b2768239d495ea8defab237b608e/cpp-generator/experimental/lidl_gen_cdylib.cpp#L1023
[nim-gen-provider]: https://github.com/corpetty/logos-nim-sdk/blob/6077eb70a8709dccf3d60601121dbf66e794b89c/lidl-gen/gen.nim#L135-L180
[lidl-optdeps]: https://github.com/logos-co/logos-lidl/commit/c0d9f9c
[lgpm-dev]: https://github.com/logos-co/logos-package-manager/blob/3133786ea86821e251fccc82a2eefbfc4db7605e/src/package_manager_lib.cpp#L1446-L1455
[lgpm-mismatch]: https://github.com/logos-co/logos-package-manager/blob/3133786ea86821e251fccc82a2eefbfc4db7605e/src/package_manager_lib.cpp#L1540-L1565
[lgx-readme]: https://github.com/logos-co/logos-package/blob/d759f34b095db14945ab0a38f6e1e924eafc3758/README.md
[pmui-local]: https://github.com/logos-co/logos-package-manager-ui/blob/3174edd016f8464c6bbdb667db54d0f7b117a62c/src/PackageManagerBackend.cpp#L1230-L1244
[pmui-dialog]: https://github.com/logos-co/logos-package-manager-ui/blob/3174edd016f8464c6bbdb667db54d0f7b117a62c/src/qml/PackageManager.qml#L302-L311
[m-lab-unload]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/docs/labbook/basecamp-sdk-skew-unload-callback.md
[m-lab-ui]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/docs/labbook/nim-cdylib-modules-cannot-load-in-basecamp-sdk-skew.md
