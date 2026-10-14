# Calling an official module from your own module

How a third-party Logos module depends on, and calls, a module from the Basecamp 0.3.2 catalog: what to declare, how to get the contract, the call and event patterns in each SDK, what the callee learns about you, and when to hand the user to another app's UI instead. It ends with Muster as a worked example.

**Verified against** (2026-10-14): Basecamp `0.3.2` ([f2fae63][bc]), logos-module-builder `0.3.3` (`3a64943f`, which builds Basecamp 0.3.2's bundled package_downloader; its package-manager-ui and modules_state build from the pre-tag commits `fc31329d`/`3818566d`; 0.3.2 → 0.3.3 is lock-only), logos-cpp-sdk `3bf5365c`, logos-rust-sdk `040e94e8`, logos-protocol `0.9.0` (`9d59f5de`), liblogos `00f19ef8`, corpetty/logos-nim-sdk `6077eb7`, capability_module `1a1b8b5a` (unchanged), keystore_module `2318c679` (unchanged). Everything marked **unverified** was not checked. First written 2026-10-01 against Basecamp 0.3.1 (`aeb8192`).

Related pages:

- [`compatibility.md`](compatibility.md): which SDK and builder revisions must line up before any of this works.
- [`../stacks/evm-wallet.md`](../stacks/evm-wallet.md), [`../stacks/monero-wallet.md`](../stacks/monero-wallet.md), [`../stacks/key-custody.md`](../stacks/key-custody.md): the wallet stacks themselves.
- `../modules/<name>/README.md` and `../modules/<name>/<name>.lidl`: per-module cards and contracts.

---

## 0. Direct call, intent, or both

| You need… | Use | Why |
|---|---|---|
| Data or an action from another backend | **Direct call**: list it in `dependencies` and call it through a typed or raw lp client | "A backend that needs another backend should make that call, not raise an intent" ([intents doc §2][int-providers]) |
| The user to decide, approve or see something in another app's UI | An **intent** from your `ui_qml` (`uses`, then `logos.request`) | Only `ui_qml` modules may provide or use intents. A core module cannot ([builder `provides`][cfg-provides]) |
| A signature that a human has to approve (keystore, Monero send) | **Both**: the backend calls `request_approval`/`send`/`prepare_send` directly, and the UI raises the approver's intent with the returned handle | The official wallets split signing into a *requester* (any named module) and an *approver* (one UI). See §5 |

---

## 1. `metadata.json` fields that matter

Field reference: [builder `docs/configuration.md`][cfg]. Parsing: [`lib/parseMetadata.nix`][parse].

| Field | What it does | Gotchas |
|---|---|---|
| `dependencies` | Generates a typed `modules().<name>` wrapper from the dep's published `.lidl`. The dep is auto-loaded before you. If it fails to load, you fail to load ([cfg][cfg-deps]) | Entries are `"name"` or `{name, version?, signer?}`. The flake input passed in `flakeInputs` must be keyed by the **module name**, e.g. `keystore_module = inputs.logos-evm-keystore-module`. Every dep must publish `packages.<sys>.lidl`, or the build refuses it by name ([common.nix][common-deps]) |
| `version` (in a dep entry) | npm-style range, e.g. `"~0.1.0"`. **liblogos refuses to load you** if the installed dep does not satisfy it ([dependency_gate.h][dep-gate]) | Fails closed. Supports `^ ~ x * comparator-sets`. Hyphen ranges and prereleases (`1.0.0-dev` against a caret range) are refused. An unversioned dep is not gated at all |
| `signer` (in a dep entry) | A `did:jwk:` publisher pin. Carried into the manifest, checked by lgpm (`SignerMismatch`/`SignerUnknown`) | **Not enforced at load** ([dependency_gate.h][dep-gate]). The catalog has zero signatures today ([intents doc §6][int-limits]) |
| `optional_dependencies` | Same typed wrapper ([cfg][cfg-optdeps]). Never fails your load. **Loaded with you when it is installed**: both hosts load a module's installed optional deps along with it ([liblogos `LOGOS_LOAD_REQUIRED_AND_OPTIONAL`][ll-loaddeps], measured in [#6][a6]). `logosctl package install` does not install it, but Basecamp's install dialog offers it pre-ticked | This row used to say "never auto-loaded". That was wrong on 0.3.1 as well. See "Optional dependencies at install and load" below. Bound calls: a call to a module that is installed but not loaded waits the caller's full deadline, then fails. Check for `object_unavailable` |
| `interface_dependencies` | Code against a contract shape and bind the provider at runtime (`bind_<iface>(name)`) | Objects only: `{name, file, input?, impl_class?}` |
| `dependency_overrides` | `{ "<dep>": { file, input?, impl_class? } }` forces the contract source, e.g. a vendored `.lidl` ([parse][parse-overrides]) | Satisfies the "must publish a `.lidl`" check. Bundling the dep into a standalone runner still needs its flake input |
| `include` | Extra files staged into the package (licenses, side libraries) from the build's `lib/`. Can be overlaid per platform ([cfg §platforms][cfg-platforms]) | Keep the base `[]` and put per-OS files in `platforms` overlays |
| `capabilities` | **Decorative.** Nothing reads it ([parse][parse-hostsvc]) | The real privilege key is `host_services` (`token_registry`, `token_delivery`). It is restricted to `capability_module` at build time |
| `interface` | `universal` (C++ impl header → derived contract), `cdylib` (your code exports the module-impl C ABI: Rust, Nim), `legacy` (no glue). A core module with `main` + `legacy` is refused ([cfg][cfg-interface]) | Selects which consumer surface you get (next row) |
| `codegen` | `impl_header`/`impl_class` (universal), `lidl` (contract-first cdylib), `rust {crate, trait?, source?}`, `rep` (ui_qml backend), `consumer_api_style` (`lp`/`qt`), `nim {crate, main, staticlib, link}` ([cfg][cfg-codegen], [mkLogosModule nim][mk-nim]) | `codegen.nim.packages` (pinned nimble deps) is **not upstream**. It lives in [logos-module-builder#226][pr226], which is still open |
| `concurrency` / `max_workers` | `"single"` (default, one handler at a time) or `"multi"` (bounded pool) ([cfg][cfg-conc]) | A blocking outbound `lp_invoke` inside a `single` handler stalls every other caller of your module |
| `provides` / `uses` | App-to-app intents, `ui_qml` only (§6) | `uses` entries must be **objects**. `["x"]` parses but declares nothing, so every request fails `not_declared` |

### How a dependency gets installed

1. **Bundle time.** `nix build .#lgx` copies `dependencies` (with `version`/`signer`) into the signed `manifest.json`. The catalog index carries it, e.g. `tx_sender_module` depends on `keystore_module ~0.1.0` in [`data/catalog-index.snapshot.json`](../data/catalog-index.snapshot.json).
2. **Catalog install (Package Manager UI).** `package_downloader.resolveDependencies` and `downloadResolvedDependencies` resolve the manifest list against the merged catalog. Given the installed list, an installed dep that satisfies the range is kept; otherwise the newest matching version is used ([downloader impl.h][dl-resolve]). Basecamp shows `basecamp.packages.confirm_install` (the "Install Package?" dialog), then installs in dependency order. What Basecamp 0.3.2 adds to this:
   - **Optional packages are offered, pre-ticked.** The dialog lists the subject's optional dependencies, transitively. An available one is ticked by default on a fresh install, and an unavailable one is unticked and disabled ([#452][bc-452], [#453][bc-453], [OptionalDependencyPreview.h][bc-optsel]). The 0.3.1 release branch already carried #452 and #453; this guide missed them. New in 0.3.2: an installed optional that no catalog lists shows as installed, not unavailable ([#455][bc-455]). So a default Basecamp install **does** install optional dependencies. Installing `monero_wallet_backend` this way took 4 packages, `monerod_module` included ([#6][a6]).
   - **A dependency only Basecamp ships.** The App Manager dialog resolves against the catalog alone, then skips what is already on disk. A dependency that no enabled catalog lists used to fail that resolve (`no candidate matches`) even when it was installed and in range. It now falls back to the installed copy ([#472][bc-472]). The install gate (an install another app asks for) already resolved against the full installed list.
   - **`package_downloader` must be started.** It does nothing on load, and every method except `start()` answers "not started" until a consumer calls `start()` ([downloader impl.h][dl-start]). Basecamp starts it as soon as it loads, and again before each operation ([#469][bc-469], [#474][bc-474]). A module that calls `package_downloader` itself should call `start()` first; it is idempotent.
3. **Local `.lgx` install (PMUI file picker).** This still installs **only that file** at 0.3.2. Dependencies are not resolved ([PMUI performInstall][pmui-local]). Install the deps first, from the catalog or from local files.
4. **Load.** liblogos loads the deps first and applies the protocol-major gate and the version-range gate ([module_manager.cpp][ll-gate]). A missing dep surfaces as Basecamp's "Missing Dependencies" dialog. Since 0.3.2 the newest version wins: when Basecamp's bundle carries a newer version of a module than the user's copy, the bundled one loads, and installing a version older than the bundled one succeeds but has no effect ([#479][bc-479]).
5. **Variants.** The catalog publishes only portable variants: `darwin-arm64`, `linux-amd64`, `linux-arm64`, and `windows-x86_64` for most packages. See [`compatibility.md` §4](compatibility.md#4-variants-portable-vs-dev).

**Nothing loads a core module after it is installed** ([#6][a6]). Not the install, not a Basecamp restart, and not a call to it. Basecamp loads core modules when a `ui_qml` app that depends on them opens. The Modules tab also has a manual load. [`../stacks/platform.md`](../stacks/platform.md) covers the lifecycle.

### Optional dependencies at install and load

Measured on Basecamp 0.3.2 and logosctl 0.3.0 ([#6][a6]). The source agrees, and it was the same at 0.3.1.

| Step | What happens to an optional dependency |
|---|---|
| `logosctl package install` | Not installed |
| Basecamp "Install Package?" | Offered, and ticked by default when available (above) |
| Load, logosctl | Loaded if installed, transitively. `module load monero_wallet_backend` printed "Dependencies loaded: monero_wallet_core_module, monero_node_module, monerod_module". `monerod_module` is an optional dependency of `monero_node_module` |
| Load, Basecamp | `CoreModuleManager::loadModule` uses `LoadPolicy::RequiredAndOptional` ([CoreModuleManager.cpp][bc-loadpolicy]). Opening a `ui_qml` app runs `PluginLoader::loadCoreDependencies`: required dependencies, then the installed optional ones ([PluginLoader.cpp][bc-coredeps]) |
| Load fails or dep is absent | Logged and skipped. Your load still succeeds. An optional dep is admitted only if all of its own required deps are installed ([liblogos `LogosLoadDeps`][ll-loaddeps]) |

### When the callee is not loaded

Measured on Basecamp 0.3.2 ([#6][a6]):

- **A module-to-module lp call** to a module that is installed but not loaded blocks for the **caller's** timeout, then fails with `object_unavailable`. At the default it took 19.97 s. Under a 1.5 s budget it took 1.47 s. Pass a short timeout on any call to a dependency that may not be up.
- **`logosctl call`** instead fails fast with `MODULE_NOT_LOADED` (0.04 s), because the daemon checks first. CLI timing does not predict module-to-module timing.
- **`lp_get_methods` takes no deadline.** In a standalone runner it blocked about 20 s on a module the host did not have.
- **Unverified:** logos-protocol [`0ec3cbe`][lp-absent] makes a QtRO acquire for a module that nothing listens for give up after a 1 s grace instead of the full budget. Each module embeds the protocol its builder pinned, and builder 0.3.3 pins one that carries this fix (`9d59f5de`). [#6][a6] does not record which protocol the measuring caller embedded. Whether a caller built with builder 0.3.3 fails in about 1 s was not measured.

> **Pin a range on anything you call through raw lp.** An unversioned dep resolves to the newest catalog version. On 2026-10-14 that is `delivery_module 0.3.2` and `lez_core 0.5.1`, and nothing type-checks a raw `lp_invoke` against the new contract. delivery `0.3.0` added a `source` field before `timestamp` in `messageReceived` ([muster delivery.nim][m-deliv-hdr]). The field is still there at `0.3.2` ([`delivery_module.lidl`](../modules/delivery_module/delivery_module.lidl)). A range does not protect you from a breaking patch release. `lez_core 0.5.1` dropped `statistics_path` from `create_new` and `open`, and removed `token_elf`, `amm_elf` and `ata_elf`, so `~0.5.0` admits it (`git diff 335c097 HEAD -- modules/lez_core/`).

---

## 2. Getting a dependency's contract (`.lidl`)

A `.lidl` file is the language-neutral module contract. It declares the module name, version, `depends`, `method`s and `event`s. Its type vocabulary is `tstr`, `bstr`, `int`, `uint`, `bool`, `[T]`, `{tstr: T}`, `any`, `result` and `?T`. Each SDK has a codegen backend over the shared frontend ([logos-lidl][lidl-readme]), so a consumer generates typed callers **without building the dependency**.

```bash
# Every builder-made module flake exposes packages.<system>.lidl
nix build 'github:logos-co/logos-evm-keystore-module/2318c679e2b7176967bd45052f3d64b2a8b06931#lidl'
cat result/keystore_module.lidl          # → 36 methods, 3 events (verified 2026-10-01)
```

- **Where it comes from** ([mkLogosModule][mk-lidl]):
  - `universal` modules: derived from the impl header (`logos-cpp-generator --header-to-lidl`).
  - contract-first `cdylib` modules: the committed `codegen.lidl`, normalised.
  - rust-first modules: derived from the trait.
  - Current bundlers also ship it as a root asset of the `.lgx`: `lgx extract X.lgx --assets-only` gives `assets/lidl/<name>.lidl` ([lgx README][lgx-readme]). Muster's July-generation `.lgx` (manifest `0.3.0`) carries no assets (checked).
- **This repo vendors them** at `modules/<name>/<name>.lidl`, built from each module's pin in `registry.json` (`.modules[].source.commit`, stamped in `modules/<name>/.contract-commit`). [`data/catalog-pins.tsv`](../data/catalog-pins.tsv) is the 2026-10-01 snapshot, not the current pins. Use these to read a contract, or as a `dependency_overrides.file`.
- **Inspect a built plugin.** `nix build github:logos-co/logos-module#lm`, then `lm methods <plugin.so> --json` or `lm metadata <plugin.so> --json` ([logos-module README][lm]).
- **Parser skew.** Your builder parses every dep's `.lidl`, even deps you only call raw. `optional_depends` entered logos-lidl on 2026-09-07 ([c0d9f9c][lidl-optdeps]). An older builder cannot read a contract that uses it, so Muster strips the line with `sed` ([muster module/flake.nix][m-flake-sed]). The keystore, monero-core, monero-backend and tx_sender contracts parse with the June frontend (`logos-lidl@8c95d4f`) that Muster's builder resolves. This was checked on 2026-10-01 by running the Nim generator linked against that build. The monero `0.1.1` contracts differ from `0.1.0` only in their `version` line, so this carries over (not re-run).

---

## 3. Calling a dependency, per language

### C++ — `interface: "universal"` (logos-cpp-sdk)

Write a plain impl class deriving `LogosModuleContext`. The builder emits `logos_sdk.h` with a `LogosModules` aggregate that has one member per dependency ([cpp-sdk README][cpp-universal]). A universal core module gets the Qt-free `lp` surface: `std::string`, `int64_t`, `LogosMap`.

```cpp
// metadata.json: "interface":"universal", "dependencies":[{"name":"keystore_module","version":"~0.1.0"}]
logos::CallError err;
std::string r = modules().keystore_module.request_approval(intentJson, &err, /*timeout_ms=*/5000);
if (!err.code.empty()) { /* err.code: object_unavailable | dispatch_failed | invalid_args | unknown_method … */ }

modules().keystore_module.request_approvalAsyncResult(intentJson,
    [](logos::AsyncResult<std::string> r) { if (r.ok()) use(r.value); });
```

- There are three entry points per method: `foo(…, CallError* = nullptr, timeout)`, `fooAsync(…, cb)` and `fooAsyncResult(…, cb)` ([README][cpp-surfaces]). Prefer `AsyncResult`, because `fooAsync` cannot tell a failure from a legitimate zero value.
- **`unknown_method` is now real.** A provider refuses a call by returning `{code, message, origin}` as its result, and the generated wrappers fold the closed set `dispatch_failed`, `invalid_args` and `unknown_method` into `CallError` ([generator_lib.cpp][cpp-reject]). Until the 0.3.2 relock no provider emitted `unknown_method`. A call to a method the target lacks returned a bare null, which a typed caller read as the return type's default with an ok `CallError`. Providers built on cpp-sdk `3bf5365c`, rust-sdk `040e94e8` or the matching plugin-qt now refuse an unknown name ([Basecamp #462][bc-462], [logos_module_impl.h][abi-dispatch]). A provider built on an older builder still answers null. Basecamp 0.3.2's bundled capability_module, package_manager and storage_module are among these ([#462][bc-462]).
- **Untyped escape hatch:** `modules().dynamic("x")` returns `logos::LpClient`. Use `nlohmann::json invoke(method, args, CallError*, int timeout_ms)` ([logos_lp_client.h][cpp-lpclient]).
- **Who called you:** `logos::currentCaller()` returns a `LogosCaller` with `isHost()` and `isModule(name)` ([logos_caller.h][cpp-caller]).
- The snake_case wrapper names (`request_approvalAsyncResult`) follow the README's `fooAsyncResult` rule. Not compiled here, so **unverified** for snake_case contracts.

### Rust — `interface: "cdylib"` + `codegen.rust` (logos-rust-sdk)

The builder supplies the SDK crate, runs `logos-lidl-gen` and emits `generated/provider_gen.rs`. There is no `build.rs` ([rust-sdk README][rs-modules]).

```rust
// The official tx_sender does exactly this (glue.rs):
let raw = modules().keystore_module
    .request_approval_with_timeout(&intent.to_string(), t)?;   // Result<String, LogosError>
modules().keystore_module.request_approval_async(&intent, |res| { /* Result<String, _> */ });
```

- The twins are `_async`, `_with_timeout` and `_async_with_timeout`, with a 20 s default deadline ([README][rs-modules]).
- A provider's refusal reaches you as `Err(LogosError::PluginCallFailed)` ([plugin.rs][rs-fold]). The SDK's closed set is `dispatch_failed`, `invalid_args` and `unknown_method`, and a Rust provider now answers an unknown name with `unknown_method` ([args.rs][rs-reject]). The caveat about older providers in the C++ section applies here too.
- To depend on a module whose contract you hold as a file, use `dependency_overrides: {"keystore_module": {"file": "contracts/keystore_module.lidl"}}`. Pass the flake input as well if a standalone runner must bundle it.
- **Origin is automatic.** The scaffold bakes `LOGOS_MODULE_NAME` from the contract ([README "announced origin"][rs-origin]). Before that fix the Rust SDK announced `"core"`, which made a module look like the host anchor.
- **Who called you:** `logos_rust_sdk::current_caller()` returns `Unknown | HostAnchor | Module{name, instance} | Derived | Operator` ([README][rs-caller]).

### Nim — `codegen.nim` + corpetty/logos-nim-sdk

Status: builder `0.3.x` builds `codegen.nim` with the Nim **stdlib only** ([mkLogosModule nim][mk-nim]). Third-party Nim packages, including the SDK itself, need the `codegen.nim.packages` hook from [PR #226][pr226]. Muster pins a fork that carries it.

| SDK piece | What you use it for |
|---|---|
| `logos_sdk/ffi` | Raw `lp_*` bindings: `lp_client_create(target, origin, nil, nil)`, `lp_invoke`, `lp_invoke_async`, `lp_subscribe`, `lp_token_save` ([ffi.nim][nim-ffi]) |
| `logos_sdk/plugin` | `newPluginProxy(target, origin = "core")` with `callSync`, `callAsync`, `subscribe` and `methodsOf` ([plugin.nim][nim-plugin]) |
| `logos_sdk/api` | `saveToken`, `context()`, `emit` |
| `logos_sdk/bytes` | The `{"_bytes":"<b64url>"}` codec, shared with the Rust SDK |
| `lidl-gen` | `provider` (your module's export surface), `client` (typed wrapper per dependency), `driver` (Muster manifests) |

Generate a typed client the way Muster's `tools/regen.sh` does:

```bash
lidl=$(nix build --no-link --print-out-paths github:logos-co/logos-lidl#logos-lidl)
git clone https://github.com/corpetty/logos-nim-sdk && git -C logos-nim-sdk checkout 6077eb70a8709dccf3d60601121dbf66e794b89c
nim c -d:LIDL_INC:$lidl/include/lidl -d:LIDL_C_A:$lidl/lib/liblogos_lidl_c.a -d:LIDL_A:$lidl/lib/liblogos_lidl.a \
      --out:lidl-gen logos-nim-sdk/lidl-gen/lidl_gen.nim
./lidl-gen client keystore_module.lidl nim-lib/keystore_client.nim keystore_module
```

```nim
let ks = newKeystoreClient(origin = "muster_module")   # NEVER rely on the default
let reply = ks.request_approvalOrRaise($intent)        # raising twin; request_approval() is lenient ("" on failure)
```

**Verified 2026-10-01** with `nim check` against the SDK at `6077eb7`. The SDK has not moved since. On 2026-10-14 the keystore and tx_sender contracts are unchanged, and the monero `0.1.1` contracts differ only in their `version` line, so the results stand (not re-run):

| Contract | Result |
|---|---|
| `keystore_module` | Generates 36 methods and compiles |
| `tx_sender_module` | Generates 9 methods and compiles |
| `monero_wallet_backend` | Generates 32 methods and compiles |
| `monero_wallet_core_module` | Generates 19 methods but **does not compile**. The parameter `addr` (in `addressValid`) is a Nim keyword, and the generator does not escape it ([gen.nim][nim-gen-client]). Rename it by hand or patch the generator |

Gaps to know about:

- **The default origin is `"core"`** in both `newPluginProxy` and every generated `new<X>Client` ([plugin.nim][nim-plugin], [gen.nim][nim-gen-client]). The Rust SDK had the same default before its fix, and its README says such a module "authorizes as the host at every callee" ([rust-sdk README][rs-origin]). The keystore and monero backend refuse the host anchor (§5). Always pass `origin = "<your module name>"`. What the 0.3.2 runtime does with a Nim module's `"core"` origin was not tested here.
- The generated client has **no typed events and no async wrappers**. Use `proxy.subscribe` and `callAsync`, or the raw `lp_*` calls.
- The provider surface exports only the protocol-0.2 module-impl ABI (7 symbols) and hard-codes `logos_module_get_protocol_version` to `"0.1.0"` ([gen.nim][nim-gen-provider]). This is the root of the Nim SDK skew in [`compatibility.md` §2](compatibility.md#2-the-unload-callback-crash-what-actually-happened).
- There is no `current_caller()` in Nim, because `logos_module_set_call_caller` is not exported. A Nim provider cannot tell who is calling it.

### QML UI plugins (`ui_qml`)

| Shape | Call | Notes |
|---|---|---|
| QML only | `logos.callModuleAsync("keystore_module", "list_accounts", [], function(json) {...}, 30000)` | Holds the call until the module appears, and the deadline also bounds that wait. The callback receives a JSON **string**. `logos.callModule(...)` is the synchronous form: avoid it at startup ([LogosQmlBridge.h][qml-call]). Since 0.3.2 a provider's refusal comes back as the bridge's error payload, not as a value: "Method not found" for `unknown_method`, "Call refused" for `invalid_args`/`dispatch_failed` ([LogosQmlBridge.cpp][qml-refusal]) |
| C++ backend (`interface: "universal"` + `codegen.rep`) | `modules().keystore_module.list_accounts()` | Qt-typed (`QString`, `qlonglong`). The dep must be in **the UI's** `dependencies` and flake inputs. Muster's backend does this for `muster_module` ([muster_ui_backend.cpp][m-ui-backend]) |

A view's bridge calls out "as an ADMITTED consumer, not as its host", under the view's own identity ([LogosQmlBridge.h][qml-consumer]). This is why `evm_signer_ui` can hold the keystore's approver role by name.

---

## 4. Events

| Language | Subscribe | Delivery thread / caveats |
|---|---|---|
| C++ | `modules().dep.onEventName([](const std::string& a, int64_t b) {...})`. Raw form: `LpClient::subscribe(event, cb)` returns an `LpSubscription` ([README][cpp-events], [lp_client.h][cpp-lpsub]) | At protocol 0.9, `onSubscriptionStatus` reports `Lost`, then `Armed` with a higher generation. That means the provider restarted, and the events in between are gone |
| Rust | `let sub = modules().dep.on_event()?;` then `for ev in sub { DepClient::decode_event(&ev) }` in a spawned thread ([README][rs-events]) | The subscription owns its client share. It unsubscribes when dropped |
| Nim | `proxy.subscribe("approval_settled", cb, userData)` with `cb: proc(eventName, dataJson: cstring, userData: pointer) {.cdecl, gcsafe.}` | `cb` runs on a foreign thread. Do no Nim GC work there: copy `dataJson` into a queue and drain it on the module thread. `GC_ref` the object behind `userData`. This is Muster's `inbound_queue` pattern ([delivery.nim][m-deliv-cb]). The payload is a JSON array |
| QML | `logos.onModuleEvent("keystore_module", "accounts_changed")`, then `Connections { target: logos; function onModuleEventReceived(module, event, data) {...} }` ([LogosQmlBridge.h][qml-events]) | Accepted is not the same as live. `logos.pendingEventSubscriptions()` lists the ones not yet armed |

The event plane carries **no auth token**, so payloads must not hold secrets. The keystore's `approval_offered`/`approval_settled` carry the handle only, and `expired_no_ack`/`cancelled` are **never announced**. Always back an event with polling ([keystore specs §Events][ks-events]).

---

## 5. Caller identity, `capability_module`, and permissions

### How a call gets attributed (Basecamp 0.3.2)

1. **Anchor token.** The host loads your module and seeds its own token through `logos_module_accept_token`. In Nim this is `saveToken` → `lp_token_save`. If you drop this step, every outbound call is rejected ([nim-sdk CLAUDE.md][nim-claude]).
2. **First call to a target.** Your lp client asks `capability_module.requestModule(fromModuleName, target)`. capability_module then works through these checks ([impl.cpp][cap-impl]):
   - It takes **your name from the platform's caller document** (`logos::currentCaller()`), **not** from `fromModuleName`. That argument is dead ABI and only triggers a warning.
   - It refuses an unnamed caller.
   - It checks that the target is loaded.
   - It applies the access policy.
   - It mints a token and pushes it **to the target**, filed under your name (`informModuleTokenTo(…, moduleName=callerName, …)`).
3. **The call.** The target's generated glue resolves the token you present to your name. It pushes `{"kind":"module","name":"<you>"}` through `logos_module_set_call_caller` (protocol ≥ 0.6) for the duration of that dispatch ([logos_module_impl.h][abi-caller]). The handler reads this with `currentCaller()` or `current_caller()`.

The `caller_json` arms are `unknown`, `host`, `module{name, instance?}`, `derived{parent, leaf}` and `operator{name}`. An unrecognised arm reads as `unknown`. The ABI itself says "Nothing here is spelled 'verified'". A module name is **token-bound**, and the name is self-declared in unsigned metadata.

**Access policy.** Basecamp 0.3.2 installs **none** by default, so enforcement is off. `--access-policy enforce` (or `LOGOS_ACCESS_POLICY=enforce`) is deny-by-default: a module may only call the modules it declares as dependencies ([main.cpp][bc-policy]). capability_module's own gate is still fail-open for a target with no registered restriction. Declare every module you call in `dependencies`, or your module breaks under `enforce`. `optional_dependencies` count as declared: under `enforce`, liblogos allows each target's loaded direct dependents to call it, optional dependents included ([module_manager.cpp][ll-derived]). That was read from the source and not exercised here. Since 0.3.2 liblogos registers a module's token before the module holds it, and counts a dependent that is still loading. Before that, a call a module made from its own init could be refused until a token re-exchange won a race ([liblogos 7d372f5][ll-inflight]).

### What the official wallet modules do with the caller

| Module | Gate | What a third-party module (e.g. `muster_module`) can do |
|---|---|---|
| `keystore_module` ([specs §tiers][ks-tiers]) | **A** (approver, default `evm_signer_ui`): `pending`/`acknowledge`/`approve`/`reject`. **B** (any *named* module): `request_approval`/`approval_status`/`fetch_result`/`ack_result`/`cancel_approval`. **C** (ungated): reads, `caller_identity`, `configure`. **D** (custodian, default `evm_keystore_ui`): every account mutation | Read accounts (C). **Request** signatures (B): at most 4 pending per requester, 16 in total, 64 KiB per intent ([specs][ks-req]). It can **never** approve, sign silently, or create keys |
| `tx_sender_module` | Records the attested caller on every row and appends it to the human-facing claim line: "the module that asked as the runtime attested it" ([glue.rs][txs-send], [caller_name][txs-caller]) | `send(request_json)` reserves nonces and asks the keystore for **one** approval. It returns `{ok, pending:true, requestId, handle}` and **never a hash**. Poll `send_status`: "this call IS the broadcast" |
| `monero_wallet_backend` | custodian/approver default `monero_wallet_ui`. `prepare_send` and `cancel_send` are open to any **named** module. `confirm_send` is approver-only. The host anchor is refused ([gate.rs][xmrb-gate]) | `prepare_send({address, amountXmr\|amount, priority?, accountIndex?})` returns a `requestId`. Poll `send_status` until `previewed`. A human confirms. At most one send is in flight ([glue.rs][xmrb-send]) |
| `monero_wallet_core_module` | **None.** The source at the pin has no caller check ([wallet_runtime.cpp startJob][xmrc-jobs]) | Anything, including `startJob("create_transaction")` and `startJob("commit_transaction")` on whatever wallet is open. It is **one engine with one open wallet** ("a wallet is already open; close it first", [wallet_runtime.cpp][xmrc-one]). See the warning in §7.3 |

**What this means for a module that requests signatures.**

- Your module name is what the human sees on the keystore's `acknowledge` (`requester`) and in tx_sender's claim line. Your `purpose` text is shown as *your claim*, never as fact.
- **Announce your real name.** `"core"` and `"capability_module"` are role labels for the host anchor. A module that announces one of them "authorizes as the host at every callee" ([rust-sdk README][rs-origin]). The keystore and monero backend refuse the host anchor at their request tiers.
- **Do not call `configure`.** It is ungated, but it is *total*: naming yourself custodian strips `evm_keystore_ui` unless you restate it. It is a documented, deliberate exposure, not an API ([specs][ks-tiers]).
- **The `receipt` is returned exactly once.** It alone authorises `fetch_result`, so persist it with the request.
- **Check attribution first.** Call `keystore_module.caller_identity()` and expect `{"kind":"module","identity":"<your name>",…}` ([specs §caller identity][ks-caller]). If you see `host` or `unknown`, your origin or token plumbing is wrong.

---

## 6. App-to-app intents (handing the user to another app)

Sources: [`docs/app-to-app-intents.md` @ 0.3.2][int] (unchanged since 0.3.1) and the Basecamp `CLAUDE.md` "App-to-app intents" section.

```json
"uses": [ { "intent": "evm.signing.approve" } ]
```

```qml
logos.request("evm.signing.approve", { handle: h }, function (res) {
    // res = { ok, data, error } — always all three keys, fires exactly once, always async
    // error ∈ not_declared | unavailable | bad_request | cancelled | timeout | failed
})
```

- **Routing:**
  - A request names a capability, never a provider.
  - The user confirms every cross-app dispatch.
  - The broker mints a separate dispatch id, so a provider cannot forge or enumerate requests.
- **No enumeration:** `unavailable` deliberately merges "nothing installed" with "denied", on a 400 ms floor.
- **Deadlines:**
  - A dispatch that no handler accepts times out at 20 s.
  - An accepted one has a **10-minute backstop**. Answer when the work has *started*, not when a chain confirms it.
- **Navigation:** when the provider answers, the shell returns the user to the requester. `"handoff": true` on the provider side opts out of that.
- **Params:** the provider's `params` are enforced just before dispatch (`bad_request`). Extra fields pass through.
- **Not signed.** A name is a claim, and the provider renders what is signed ([§6 limits][int-limits]).

**Wallet intents in the 0.3.2 catalog** (from each pinned `metadata.json`, via `registry.json` on 2026-10-14):

| Intent | Provider | Params | Handoff | Used today by |
|---|---|---|---|---|
| `evm.signing.approve` | `evm_signer_ui` | `handle` (keystore approval handle) | no | `eth_wallet_ui`, `uniswap_ui` |
| `evm.transactions.send` | `eth_wallet_ui` | `calls[{to, value?, data?, gasLimit?, label?, meta?}]`, `purpose`, `chainId?`, `from?`, `tier?` | no | — |
| `evm.accounts.manage` | `evm_keystore_ui` | — | yes | `eth_wallet_ui`, `uniswap_ui` |
| `monero.wallet.unlock` | `monero_wallet_ui` | `wallet` | no | — |
| `monero.accounts.manage` | `monero_wallet_ui` | — | yes | — |
| `monero.node.configure` / `evm.rpc.configure` / `evm.token_lists.configure` / `evm.verified_routing.operate` | `monerod_ui` / `eth_rpc_ui` / `token_list_ui` / `verified_proxy_ui` | `network?` / — / — / — | yes | wallet UIs (and `eth_rpc_ui` for verified routing) |

The copyable pattern is **backend requests, UI escorts**: the backend calls `request_approval` (or `tx_sender.send`), receives a `handle`, and the UI raises `evm.signing.approve {handle}`. If the user ignores the escort, `evm_signer_ui` still lists the request through `pending` and `approval_offered`.

---

## 7. Worked example: Muster

Muster is `corpetty/muster`. It has a Nim core module (`module/`) and a QML UI (`ui/`). The citations below are for `origin/main` [`fff01af`][m] (2026-10-01).

### 7.1 How Muster calls `lez_core` and `delivery_module` today

| File | What it does |
|---|---|
| [`module/metadata.json`][m-meta] | `type: core`, `interface: cdylib`, `codegen.lidl: src/api/muster.lidl`, `codegen.nim {crate: nim-lib, main: muster_module.nim, staticlib, link: [sodium], packages: [logos-nim-sdk@6077eb7, nim-eth, nim-web3, …]}`. `dependencies: ["delivery_module", "lez_core"]`, **unversioned** |
| [`module/flake.nix`][m-flake] | Builder is the fork `corpetty/logos-module-builder@c10a94c`, with every SDK input overridden to the protocol-**0.2.0** generation (`cpp-sdk c3fa1b5a`, `protocol 6401e30a`, …). `flakeInputs = { delivery_module = …; lez_core = inputs.lez_core; } // inputs` maps flake inputs to module names. `deliveryForModule` strips `optional_depends` from delivery v0.3.0's `.lidl` |
| [`nim-lib/muster_gen.nim`][m-gen] | Provider surface generated by `lidl-gen provider` (`tools/regen.sh`): the 7 `logos_module_*` exports. `accept_token` calls `saveToken` |
| [`src/transport/delivery.nim`][m-deliv] | `lp_client_create("delivery_module", "muster_module", nil, nil)` with an explicit origin. It runs `createNode` and `start` synchronously via `lp_invoke`, then `lp_subscribe("messageReceived", cb, self)`. The callback copies into an `InboundQueue`, and `poll()` drains it on the module thread. `send` and `storeQuery` use `lp_invoke_async` |
| [`src/wallet/lez_lp.nim`][m-lez] | `lp_client_create("lez_core", origin, …)`. `rawCall` wraps `lp_invoke` with a 15 s read budget. Proving transfers use `lp_invoke_async` with a 900 s budget, and the result lands in a queue that a pump drains. It is gated behind `MUSTER_LEZ_REAL` |
| [`ui/metadata.json`][m-uimeta] | `ui_qml` + `universal` + `codegen.rep`. Deps: `muster_module`, `delivery_module`, `lez_core`, the RLN modules. `provides: coordinate.request`. `uses: []` |
| [`ui/src/muster_ui_backend.cpp`][m-ui-backend] | `modules().muster_module.health()` / `.describe()` / `.propose(…)`: Qt-typed generated client |

The pattern: Muster calls its dependencies with **untyped JSON over raw `lp_*`**. It passes an explicit origin, never touches the Nim GC on foreign threads, and pumps long work from the intents tick. Nothing checks those JSON calls against the dependency's contract. That is why the delivery v0.3 move needed hand-written parsers for both event shapes (`received.nim`).

### 7.2 Sketch: adding `keystore_module` (EVM)

These steps edit files in the Muster repo. Do them there, not here.

1. **`module/metadata.json`**: add `{"name":"keystore_module","version":"~0.1.0"}` to `dependencies`. At the same time, pin ranges on the existing deps.
2. **`module/flake.nix`**:
   - Add `logos-evm-keystore-module.url = "github:logos-co/logos-evm-keystore-module/2318c679e2b7176967bd45052f3d64b2a8b06931";`.
   - Add `keystore_module = inputs.logos-evm-keystore-module;` to `flakeInputs`.
   - Its `.lidl` parses with Muster's builder frontend (checked, §2).
3. **`ui/flake.nix` and `ui/metadata.json`**: add `keystore_module` **and `evm_signer_ui`** (`logos-co/logos-evm-signer-ui@62f89d5`) so the standalone runner contains an approver. Without `evm_signer_ui`, `request_approval` never settles. Add `"uses": [{"intent":"evm.signing.approve"}]`.
4. **Client**: pick one.
   - Typed: `lidl-gen client keystore_module.lidl nim-lib/keystore_client.nim keystore_module`, then `newKeystoreClient(origin = "muster_module")`.
   - Raw: a new `src/wallet/keystore_lp.nim` shaped like `lez_lp.nim`, with `lp_client_create("keystore_module", "muster_module", nil, nil)`.
5. **Seam change.** Muster's `Keystore` seam has a synchronous `sign`, but the keystore has none. The flow is request, then a human approves, then fetch. Model it like the LEZ proving pump:
   - `requestSignature(intent)` → `request_approval` → persist `{handle, receipt}`.
   - On the intents tick: `approval_status(handle, receipt)`.
   - When the status is `approved`: `fetch_result` → `{ok, signed:[…]}`, then `ack_result`.
   - Also `subscribe("approval_settled")` for a faster wake-up, but keep the polling (§4).
6. **Leg mapping** (`request_approval` intent `{address, purpose, legs}`, [specs][ks-req]):

   | Muster signature | Keystore leg | Why |
   |---|---|---|
   | Safe owner approval (`safeTxHash`, EIP-712) | `typed_data` | The keystore computes the hash itself and renders every SafeTx field. Not `digest`, which shows the human an opaque hash |
   | EIP-155 transaction | `tx` with `chain_id` | Prefer `tx_sender_module.send` (next step) |
   | EIP-191 message | `message` | — |

7. **Broadcasting.** The keystore never touches the network. `tx_sender_module` is "the one EVM transaction sender on the device: one nonce ledger", so broadcasting Muster's own keystore-signed transactions bypasses that ledger. For transactions, call `tx_sender.send` as the requester. It returns a `handle`: escort the user with `evm.signing.approve`, then poll `send_status`. Alternatively, the UI hands the calls to `eth_wallet_ui` with `evm.transactions.send`. Keep keystore `request_approval` for non-transaction signatures (Safe owner signatures, EIP-712).
8. **UI flow.** The `muster_module` method returns the handle. QML calls `logos.request("evm.signing.approve", {handle}, cb)`. The shell brings `evm_signer_ui` forward, the user enters the vault password, and the shell returns the user to Muster. The module's pump picks up the result.
9. **First test.** Call `keystore_module.caller_identity()` from `muster_module` inside Basecamp and expect `identity: "muster_module"`. Muster (protocol-0.2 glue) calling a protocol-0.9 module is proven for **delivery v0.3.0 in the standalone runner** (`scripts/ui-parity.sh` 8/8, 2026-10-01). Keystore attribution from a 0.2-generation caller is **unverified**, in the runner and in Basecamp.

### 7.3 Sketch: Monero

- **Go through `monero_wallet_backend`, not the core.** Add `{"name":"monero_wallet_backend","version":"~0.1.0"}` (the catalog has `0.1.1`, which `~0.1.0` admits). Its `0.1.1` manifest pulls `monero_wallet_core_module ~0.1.1` and `monero_node_module ~0.1.0`. `monero_node_module` names `monerod_module` as an optional dependency, which Basecamp offers pre-ticked and both hosts load when it is installed (§1). Its Nim client compiles (§3).
- **Calls:**
  - Reads (open to any caller): `wallet_status`, `balances(account)`, `receive_info(account)`.
  - Spend: `prepare_send({address, amountXmr, accountIndex?})` returns `{ok, requestId}`. Poll `send_status(requestId)` until it reaches `previewed`.
  - The approver (`monero_wallet_ui`) runs `confirm_send`.
  - Events: `send_status_changed(request_id, state)`, `balance_changed`, `wallet_state_changed`.
- **Unlock.** The wallet must be open. Opening it is a custodian method (`open_wallet`), so the UI escorts the user with `monero.wallet.unlock {wallet}`. **Unverified:** whether `monero_wallet_ui` shows a send that a third party prepared for confirmation. There is no confirm intent; `list_sends` is documented as what "a headless approver polls".
- **Why not `monero_wallet_core_module` directly.**
  - It has no caller gate: any loaded module can create and commit transactions on the open wallet, which bypasses the backend's approver. Treat that as a platform gap, not a feature.
  - It holds one open wallet shared with the backend, so two drivers race each other.
  - Its Nim client does not compile as generated (`addr`).

---

[bc]: https://github.com/logos-co/logos-basecamp/tree/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9
[int]: https://github.com/logos-co/logos-basecamp/blob/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9/docs/app-to-app-intents.md
[int-providers]: https://github.com/logos-co/logos-basecamp/blob/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9/docs/app-to-app-intents.md#L52
[int-limits]: https://github.com/logos-co/logos-basecamp/blob/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9/docs/app-to-app-intents.md#L203-L217
[bc-policy]: https://github.com/logos-co/logos-basecamp/blob/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9/app/main.cpp#L165-L216
[cfg]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/docs/configuration.md
[cfg-interface]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/docs/configuration.md#L88-L115
[cfg-codegen]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/docs/configuration.md#L117-L149
[cfg-conc]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/docs/configuration.md#L151-L212
[cfg-deps]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/docs/configuration.md#L273-L310
[cfg-optdeps]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/docs/configuration.md#L311-L365
[cfg-provides]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/docs/configuration.md#L367-L481
[cfg-platforms]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/docs/configuration.md#L683-L720
[parse]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/lib/parseMetadata.nix
[parse-overrides]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/lib/parseMetadata.nix#L295-L314
[parse-hostsvc]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/lib/parseMetadata.nix#L368-L400
[common-deps]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/lib/common.nix#L167-L200
[mk-nim]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/lib/mkLogosModule.nix#L647-L692
[mk-lidl]: https://github.com/logos-co/logos-module-builder/blob/3a64943fe6827bab4c4686fb35c37d24d5df0710/lib/mkLogosModule.nix#L958-L985
[pr226]: https://github.com/logos-co/logos-module-builder/pull/226
[dep-gate]: https://github.com/logos-co/logos-liblogos/blob/00f19ef8e2ffd54e2cf97616a960cd19232ad4ba/src/logos_core/dependency_gate.h#L8-L24
[ll-gate]: https://github.com/logos-co/logos-liblogos/blob/00f19ef8e2ffd54e2cf97616a960cd19232ad4ba/src/logos_core/module_manager.cpp#L838-L898
[ll-derived]: https://github.com/logos-co/logos-liblogos/blob/00f19ef8e2ffd54e2cf97616a960cd19232ad4ba/src/logos_core/module_manager.cpp#L466-L504
[ll-inflight]: https://github.com/logos-co/logos-liblogos/commit/7d372f567f69e1946668966cd759e473fd15117a
[ll-loaddeps]: https://github.com/logos-co/logos-liblogos/blob/00f19ef8e2ffd54e2cf97616a960cd19232ad4ba/src/logos_core/logos_core.h#L72-L94
[bc-loadpolicy]: https://github.com/logos-co/logos-basecamp/blob/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9/app/CoreModuleManager.cpp#L92-L98
[bc-coredeps]: https://github.com/logos-co/logos-basecamp/blob/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9/app/PluginLoader.cpp#L137-L222
[bc-optsel]: https://github.com/logos-co/logos-basecamp/blob/f2fae630737bc56ef8c4bb33ce92ae365ba8abb9/app/OptionalDependencyPreview.h#L215-L220
[bc-452]: https://github.com/logos-co/logos-basecamp/pull/452
[bc-453]: https://github.com/logos-co/logos-basecamp/pull/453
[bc-455]: https://github.com/logos-co/logos-basecamp/pull/455
[bc-462]: https://github.com/logos-co/logos-basecamp/pull/462
[bc-469]: https://github.com/logos-co/logos-basecamp/pull/469
[bc-472]: https://github.com/logos-co/logos-basecamp/pull/472
[bc-474]: https://github.com/logos-co/logos-basecamp/pull/474
[bc-479]: https://github.com/logos-co/logos-basecamp/pull/479
[a6]: https://github.com/corpetty/logos-module-atlas/issues/6
[dl-resolve]: https://github.com/logos-co/logos-package-downloader-module/blob/7b0ca8b74acde01c277431a1372092980a3f31df/src/package_downloader_impl.h#L64-L83
[dl-start]: https://github.com/logos-co/logos-package-downloader-module/blob/7b0ca8b74acde01c277431a1372092980a3f31df/src/package_downloader_impl.h#L85-L94
[pmui-local]: https://github.com/logos-co/logos-package-manager-ui/blob/8b67bf0213596e021d441e5d1d43f857142003b6/src/PackageManagerBackend.cpp#L1243-L1262
[lidl-readme]: https://github.com/logos-co/logos-lidl/blob/2043d8bf94c6bfee3781f96ad088e8c13ec36038/README.md
[lidl-optdeps]: https://github.com/logos-co/logos-lidl/commit/c0d9f9c
[lm]: https://github.com/logos-co/logos-module/blob/f71d16ddc5779b50b188b3a3928a37c9ce90b2a1/README.md
[lgx-readme]: https://github.com/logos-co/logos-package/blob/c25a1167578aef5cbd9a9b6f822ffe2ae4fd6a89/README.md
[cpp-surfaces]: https://github.com/logos-co/logos-cpp-sdk/blob/3bf5365c5412f89bbbf30e3f46237cd68cff0599/README.md#L273-L345
[cpp-universal]: https://github.com/logos-co/logos-cpp-sdk/blob/3bf5365c5412f89bbbf30e3f46237cd68cff0599/README.md#L347-L427
[cpp-events]: https://github.com/logos-co/logos-cpp-sdk/blob/3bf5365c5412f89bbbf30e3f46237cd68cff0599/README.md#L477-L508
[cpp-lpclient]: https://github.com/logos-co/logos-cpp-sdk/blob/3bf5365c5412f89bbbf30e3f46237cd68cff0599/cpp/logos_lp_client.h#L268-L299
[cpp-lpsub]: https://github.com/logos-co/logos-cpp-sdk/blob/3bf5365c5412f89bbbf30e3f46237cd68cff0599/cpp/logos_lp_client.h#L376-L415
[cpp-caller]: https://github.com/logos-co/logos-cpp-sdk/blob/3bf5365c5412f89bbbf30e3f46237cd68cff0599/cpp/logos_caller.h#L86-L110
[cpp-reject]: https://github.com/logos-co/logos-cpp-sdk/blob/3bf5365c5412f89bbbf30e3f46237cd68cff0599/cpp-generator/generator_lib.cpp#L840-L896
[rs-modules]: https://github.com/logos-co/logos-rust-sdk/blob/040e94e83873249dc5f27da8c2a01c3dcaaa630d/README.md#L40-L125
[rs-events]: https://github.com/logos-co/logos-rust-sdk/blob/040e94e83873249dc5f27da8c2a01c3dcaaa630d/README.md#L127-L142
[rs-caller]: https://github.com/logos-co/logos-rust-sdk/blob/040e94e83873249dc5f27da8c2a01c3dcaaa630d/README.md#L162-L189
[rs-origin]: https://github.com/logos-co/logos-rust-sdk/blob/040e94e83873249dc5f27da8c2a01c3dcaaa630d/README.md#L191-L216
[rs-reject]: https://github.com/logos-co/logos-rust-sdk/blob/040e94e83873249dc5f27da8c2a01c3dcaaa630d/src/args.rs#L270-L306
[rs-fold]: https://github.com/logos-co/logos-rust-sdk/blob/040e94e83873249dc5f27da8c2a01c3dcaaa630d/src/plugin.rs#L554-L566
[nim-ffi]: https://github.com/corpetty/logos-nim-sdk/blob/6077eb70a8709dccf3d60601121dbf66e794b89c/src/logos_sdk/ffi.nim
[nim-plugin]: https://github.com/corpetty/logos-nim-sdk/blob/6077eb70a8709dccf3d60601121dbf66e794b89c/src/logos_sdk/plugin.nim#L54-L148
[nim-gen-provider]: https://github.com/corpetty/logos-nim-sdk/blob/6077eb70a8709dccf3d60601121dbf66e794b89c/lidl-gen/gen.nim#L135-L180
[nim-gen-client]: https://github.com/corpetty/logos-nim-sdk/blob/6077eb70a8709dccf3d60601121dbf66e794b89c/lidl-gen/gen.nim#L253-L290
[nim-claude]: https://github.com/corpetty/logos-nim-sdk/blob/6077eb70a8709dccf3d60601121dbf66e794b89c/CLAUDE.md
[qml-call]: https://github.com/logos-co/logos-view-module-runtime/blob/4c4297239f876802ae02e7810fe956967ea75a39/include/LogosQmlBridge.h#L153-L181
[qml-consumer]: https://github.com/logos-co/logos-view-module-runtime/blob/4c4297239f876802ae02e7810fe956967ea75a39/include/LogosQmlBridge.h#L108-L135
[qml-events]: https://github.com/logos-co/logos-view-module-runtime/blob/4c4297239f876802ae02e7810fe956967ea75a39/include/LogosQmlBridge.h#L320-L362
[qml-refusal]: https://github.com/logos-co/logos-view-module-runtime/blob/4c4297239f876802ae02e7810fe956967ea75a39/src/LogosQmlBridge.cpp#L71-L96
[abi-caller]: https://github.com/logos-co/logos-protocol/blob/9d59f5de526624b120d42f4c422a3d83c0b9d64f/cpp/logos_module_impl.h#L192-L290
[abi-dispatch]: https://github.com/logos-co/logos-protocol/blob/9d59f5de526624b120d42f4c422a3d83c0b9d64f/cpp/logos_module_impl.h#L61-L66
[lp-absent]: https://github.com/logos-co/logos-protocol/commit/0ec3cbef39d7eda31543a7c40f9e6fb1faee61a7
[cap-impl]: https://github.com/logos-co/logos-capability-module/blob/1a1b8b5a1167931afbf41ebaafbe76034e6357a0/src/capability_module_impl.cpp#L60-L168
[ks-tiers]: https://github.com/logos-co/logos-evm-keystore-module/blob/2318c679e2b7176967bd45052f3d64b2a8b06931/docs/specs.md#L1064-L1107
[ks-caller]: https://github.com/logos-co/logos-evm-keystore-module/blob/2318c679e2b7176967bd45052f3d64b2a8b06931/docs/specs.md#L1109-L1150
[ks-req]: https://github.com/logos-co/logos-evm-keystore-module/blob/2318c679e2b7176967bd45052f3d64b2a8b06931/docs/specs.md#L1354-L1395
[ks-events]: https://github.com/logos-co/logos-evm-keystore-module/blob/2318c679e2b7176967bd45052f3d64b2a8b06931/docs/specs.md#L1479-L1520
[txs-send]: https://github.com/logos-co/logos-evm-tx-sender-module/blob/7cd2fead60a78ac9ac8a4337fd9f01e1ab5515e2/rust-lib/src/glue.rs#L60-L80
[txs-caller]: https://github.com/logos-co/logos-evm-tx-sender-module/blob/7cd2fead60a78ac9ac8a4337fd9f01e1ab5515e2/rust-lib/src/glue.rs#L271-L283
[xmrb-gate]: https://github.com/logos-co/logos-monero-wallet-backend/blob/a17d0df6e7c89bb5a82ae196e7eb2f02599ae29a/rust-lib/src/gate.rs#L102-L131
[xmrb-send]: https://github.com/logos-co/logos-monero-wallet-backend/blob/a17d0df6e7c89bb5a82ae196e7eb2f02599ae29a/rust-lib/src/glue.rs#L88-L99
[xmrc-jobs]: https://github.com/logos-co/logos-monero-wallet-core-module/blob/c513e58d05a08abfdb789e82c94bab052f3eb7e5/src/wallet_runtime.cpp#L153-L156
[xmrc-one]: https://github.com/logos-co/logos-monero-wallet-core-module/blob/c513e58d05a08abfdb789e82c94bab052f3eb7e5/src/wallet_runtime.cpp#L314
[m]: https://github.com/corpetty/muster/tree/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a
[m-meta]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/module/metadata.json#L1-L35
[m-flake]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/module/flake.nix#L22-L86
[m-flake-sed]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/module/flake.nix#L64-L86
[m-gen]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/module/nim-lib/muster_gen.nim#L310-L356
[m-deliv]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/module/src/transport/delivery.nim#L73-L190
[m-deliv-hdr]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/module/src/transport/delivery.nim#L1-L12
[m-deliv-cb]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/module/src/transport/delivery.nim#L96-L105
[m-lez]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/module/src/wallet/lez_lp.nim#L39-L75
[m-uimeta]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/ui/metadata.json
[m-ui-backend]: https://github.com/corpetty/muster/blob/fff01afa45c5a0ff1bd18a533e0a7bd90f51b65a/ui/src/muster_ui_backend.cpp#L20-L70
