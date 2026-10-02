# Logos Module Atlas

An AI-readable registry of the official Logos modules: everything the **latest Logos
Basecamp release** ships with or can install from its default catalog. For each
module it has a card, the canonical LIDL contract and the annotated interface source,
all pinned to the exact commit that was released. On top of that are hand-written
docs on how the modules fit together.

It exists so a coding agent can answer "what can I call, and how?" in minutes, and
not by spelunking through 50 repos. AI agents should start with [AGENTS.md](AGENTS.md).

## Using it with Claude Code

Pick one:

- **As a plugin** (works in any project, nothing to clone):

  ```
  /plugin marketplace add corpetty/logos-module-atlas
  /plugin install logos-module-atlas@logos-module-atlas
  ```

  This installs the `logos-module-atlas` skill, which Claude loads whenever a task
  involves Logos modules. The registry refreshes daily, but Claude Code doesn't
  auto-update plugins from third-party marketplaces unless you turn it on: in `/plugin`,
  open the **Marketplaces** tab, select `logos-module-atlas` and choose **Enable
  auto-update**. To update by hand, run `claude plugin marketplace update
  logos-module-atlas`, then `claude plugin update logos-module-atlas@logos-module-atlas`.

  On Claude Code 2.1.287 or later, the plugin is also a
  [mod](https://code.claude.com/docs/en/plugins/mods/overview) (`hooks/`), which adds:

  - **Tools for Claude:** `module`, `method` and `search` (as
    `mcp__logos-module-atlas__*`) answer from the registry and contracts in one call.
  - **Context:** a prompt that names a module carries a line on where it ships and
    where its contract is, once per conversation.
  - **Edit checks:** an edit that calls a method or event a module's contract doesn't
    declare is refused with the closest real names, and so is a `metadata.json` mistake
    that breaks a module (a string in `uses`, a version range no release satisfies). The same edit made
    again goes through, so a wrong check can't block Claude. Risks such as an
    unversioned dependency are passed to Claude as a note instead.
  - **`/atlas`:** a pane to browse and search the modules, with no Claude turn.
    `/atlas <module>` or `/atlas <words>` answers in the transcript, and works while
    Claude is busy.

  A mod is code that runs with your permissions. This one reads the plugin's own files
  and any `metadata.json` Claude edits, and makes no network or process calls:
  `claude plugin validate .claude-plugin/plugin.json` lists every call it makes. Tests
  run with `claude plugin test`. To turn the mod off and keep the skill, set
  `"disableAllHooks": true` in your settings, or disable the plugin in `/plugin`.

- **From a project's `CLAUDE.md`**, given a local clone (e.g. Muster):

  ```markdown
  Logos module reference: ~/src/logos-module-atlas — read its AGENTS.md
  before calling or depending on any official Logos module.
  ```

  Start the session with `claude --add-dir ~/src/logos-module-atlas` so it
  can read the files without prompting.

- **Without Claude Code:** [`llms.txt`](llms.txt) indexes everything by raw URL, and
  [`registry.json`](registry.json) is the whole registry in one file.

## What's covered and where it comes from

| Source | What it gives |
|---|---|
| Latest [`logos-co/logos-basecamp`](https://github.com/logos-co/logos-basecamp/releases/latest) release | The modules bundled in the app (`installedDistributed` in its `flake.nix`, pinned by its `flake.lock`) |
| `kDefaultRepositoryUrl` in that release's package downloader | The default catalog. Today: [`logos-co/logos-modules-release`](https://github.com/logos-co/logos-modules-release) |
| The catalog's `index.json` | Every installable package, with versions, platforms and dependencies |
| `publisherRef` tag → catalog commit → submodule gitlink | The exact source commit each package was built from (same method as [`logos-release-set`](https://github.com/logos-co/logos-release-set)) |
| Each source repo at that commit | `metadata.json`, the interface source, the docs, and the `.lidl` contract via `nix build #lidl` |

## Layout

```
AGENTS.md            how an AI agent should use this repo (start here)
registry.json        every module, machine-readable
llms.txt             index of everything by raw URL
modules/<name>/      README.md card · <name>.lidl contract · interface.* source · NOTES.md
stacks/              EVM wallet, Monero wallet, blockchain & LEZ, messaging, storage, platform, key custody
guides/              calling official modules from your own module; SDK/variant compatibility
gaps.md              roadmap items not in the release (e.g. Bitcoin, Zcash)
scripts/             build_registry.py, fetch_contracts.py, refresh.sh · check_mod.mjs, check_drift.py, issue.sh (CI checks and reports)
hooks/               the Claude Code mod: register.js (events, tools, /atlas pane) · atlas.js (logic)
tests/               the mod's tests, for `claude plugin test`
```

## Refreshing

```bash
scripts/refresh.sh
```

You need `python3`, `gh` (or `GITHUB_TOKEN`), and `nix` with flakes for the
contracts. A GitHub Action runs this daily. Before it commits, it checks that the
refresh left the hand-written files alone and that the mod still validates, passes its
tests and reads the new data (`node scripts/check_mod.mjs`). The `Mod` workflow runs
the same checks on pull requests.

What the automation can't fix, it reports as GitHub issues that open and close
themselves, one per label:

| Label | Opened when | Closed when |
|---|---|---|
| `atlas:drift` | A stack doc, guide or `gaps.md` is behind the release, or a module has no stack of its own in `data/stacks.json` (`scripts/check_drift.py`) | A refresh finds nothing behind |
| `atlas:refresh-failed` | The daily refresh fails | A refresh passes |
| `atlas:canary` | The weekly canary finds the newest Claude Code breaks the mod | The canary passes |
| `atlas:raise-pin` | A newer Claude Code passes, so CI's pin in `.github/claude-code-version` can move | The pin is the newest |

Both scheduled workflows also re-enable each other through the GitHub API, so the
daily refresh keeps running through quiet stretches upstream when it has nothing to
commit.

## License

The atlas's own scripts and docs are MIT ([LICENSE](LICENSE)). Vendored
`modules/*/interface.*` and `modules/*/*.lidl` files come from the upstream module
repos and remain under their terms. Each one names its source repo and commit.

## Modules

<!-- registry:start -->
_Generated 2026-10-02 from Basecamp [0.3.1](https://github.com/logos-co/logos-basecamp/releases/tag/0.3.1) and its default catalog [Logos Official](https://github.com/logos-co/logos-modules-release) (index generated 2026-10-02T15:49)._

### [Blockchain & LEZ](stacks/blockchain-lez.md)

| Module | Type | Version | Where | Contract | Description |
|---|---|---|---|---|---|
| [`amm_module`](modules/amm_module/README.md) | core | 0.1.0 | catalog | — | Core module for the Logos DEX (AMM) — on-chain pool resolution, swaps, and liquidity. |
| [`amm_ui`](modules/amm_ui/README.md) | ui_qml | 0.1.0 | catalog | — | Trade tokens and provide liquidity on the Logos DEX. |
| [`blockchain_module`](modules/blockchain_module/README.md) | core | 0.3.0 | catalog | [lidl](modules/blockchain_module/blockchain_module.lidl) | Logos blockchain node for logos-core |
| [`blockchain_ui`](modules/blockchain_ui/README.md) | ui_qml | 0.3.1 | catalog | — | Blockchain UI module for the Logos application |
| [`lez_core`](modules/lez_core/README.md) | core | 0.5.0 | catalog | [lidl](modules/lez_core/lez_core.lidl) | Logos Execution Zone Core Module for Logos Core |
| [`lez_explorer_ui`](modules/lez_explorer_ui/README.md) | ui_qml | 1.2.0 | catalog | — | Logos Execution Zone Block Explorer |
| [`lez_indexer_module`](modules/lez_indexer_module/README.md) | core | 1.2.0 | catalog | [lidl](modules/lez_indexer_module/lez_indexer_module.lidl) | Logos Execution Zone Indexer Module for Logos Core |
| [`lez_wallet_ui`](modules/lez_wallet_ui/README.md) | ui_qml | 1.2.0 | catalog | — | Execution Zone Wallet UI module for the Logos application |
| [`logos_execution_zone`](modules/logos_execution_zone/README.md) | core | 1.0.0 | catalog | [lidl](modules/logos_execution_zone/logos_execution_zone.lidl) | Logos Execution Zone Module for Logos Core |

### [EVM wallet](stacks/evm-wallet.md)

| Module | Type | Version | Where | Contract | Description |
|---|---|---|---|---|---|
| [`eth_rpc_module`](modules/eth_rpc_module/README.md) | core | 0.1.0 | catalog | [lidl](modules/eth_rpc_module/eth_rpc_module.lidl) | Proxyable, fail-closed Ethereum JSON-RPC client. Per-chain config (endpoint + proxy); RPC calls keyed by chainId. socks5h/Tor-ready. |
| [`eth_rpc_ui`](modules/eth_rpc_ui/README.md) | ui_qml | 0.1.0 | catalog | — | Device-wide JSON-RPC endpoints and light-client verified routing, shared by every Logos wallet on this device. |
| [`eth_wallet_backend`](modules/eth_wallet_backend/README.md) | core | 0.1.0 | catalog | [lidl](modules/eth_wallet_backend/eth_wallet_backend.lidl) | EVM wallet composer over reusable chain, asset, token, account, fee, and transaction-sender modules, with multi-chain balances and activity. |
| [`eth_wallet_ui`](modules/eth_wallet_ui/README.md) | ui_qml | 0.1.0 | catalog | — | View EVM assets and activity across enabled networks, and send on an explicitly selected chain. Holds no key material. |
| [`evm_assets_module`](modules/evm_assets_module/README.md) | core | 0.1.0 | catalog | [lidl](modules/evm_assets_module/evm_assets_module.lidl) | Reusable native and ERC-20 asset rows, balances, transfer building, and transaction-history decoration. |
| [`evm_keystore_cli`](modules/evm_keystore_cli/README.md) | core | 0.1.0 | catalog | [lidl](modules/evm_keystore_cli/evm_keystore_cli.lidl) | Headless custodian for keystore_module: creates, imports, exports and deletes accounts over `logosctl call`. Every other headless surface… |
| [`evm_keystore_ui`](modules/evm_keystore_ui/README.md) | ui_qml | 0.1.0 | catalog | — | The one place accounts are created, imported, exported and deleted. Every other surface only reads which accounts exist. |
| [`evm_signer_cli`](modules/evm_signer_cli/README.md) | core | 0.1.0 | catalog | [lidl](modules/evm_signer_cli/evm_signer_cli.lidl) | Headless approver for keystore_module: shows what a module asked to sign, in the keystore's own words, over `logosctl watch`, and takes t… |
| [`evm_signer_ui`](modules/evm_signer_ui/README.md) | ui_qml | 0.1.0 | catalog | — | The signing approval surface: shows what a module asked to sign, in the keystore's own words, and takes the vault password. |
| [`fee_module`](modules/fee_module/README.md) | core | 0.1.0 | catalog | [lidl](modules/fee_module/fee_module.lidl) | EIP-1559 fee suggestion for EVM chains: slow/normal/fast tiers derived from eth_feeHistory, with custom overrides. |
| [`keystore_module`](modules/keystore_module/README.md) | core | 0.1.0 | catalog | [lidl](modules/keystore_module/keystore_module.lidl) | Keystore: scrypt vaults, BIP39/BIP32 HD derivation, secp256k1 signing. No network; private keys never leave the module. |
| [`token_list_module`](modules/token_list_module/README.md) | core | 0.1.0 | catalog | [lidl](modules/token_list_module/token_list_module.lidl) | Reusable EVM token catalogue, enabled snapshots, pinned assets and paged picker. Proxyable, fail-closed. |
| [`token_list_ui`](modules/token_list_ui/README.md) | ui_qml | 0.1.0 | catalog | — | Device-wide token catalogue and enabled set: built-in defaults, extra list URLs, custom tokens, and the ERC-20s shared by every Logos wal… |
| [`tx_sender_module`](modules/tx_sender_module/README.md) | core | 0.1.0 | catalog | [lidl](modules/tx_sender_module/tx_sender_module.lidl) | The one EVM transaction sender on the device: one nonce ledger, call bundles approved as one decision in the keystore, ordered broadcast,… |
| [`uniswap_backend`](modules/uniswap_backend/README.md) | core | 0.1.0 | catalog | [lidl](modules/uniswap_backend/uniswap_backend.lidl) | The Uniswap app's backend: quotes, swaps and swap history composed from reusable EVM chain, token, asset, account, fee, quote and sender … |
| [`uniswap_module`](modules/uniswap_module/README.md) | core | 0.1.0 | catalog | [lidl](modules/uniswap_module/uniswap_module.lidl) | Uniswap price oracle + swap router: V2/V3/V4 token prices (best-rate, Multicall3-batched) and V2/V3 swap building. Multi-chain, configura… |
| [`uniswap_ui`](modules/uniswap_ui/README.md) | ui_qml | 0.1.0 | catalog | — | Swap EVM assets on Uniswap. A view over uniswap_backend, which composes the reusable EVM modules. |
| [`verified_proxy_module`](modules/verified_proxy_module/README.md) | core | 0.1.3 | catalog | [lidl](modules/verified_proxy_module/verified_proxy_module.lidl) | Light-client-verified Ethereum JSON-RPC, wrapping nimbus libverifproxy |
| [`verified_proxy_ui`](modules/verified_proxy_ui/README.md) | ui_qml | 0.1.0 | catalog | — | Configure and operate the light-client-verified Ethereum proxy |

### [Messaging](stacks/messaging.md)

| Module | Type | Version | Where | Contract | Description |
|---|---|---|---|---|---|
| [`chat_module`](modules/chat_module/README.md) | core | 0.3.0 | catalog | [lidl](modules/chat_module/chat_module.lidl) | Chat module for Logos |
| [`chat_ui`](modules/chat_ui/README.md) | ui_qml | 0.3.0 | catalog | — | Chat App for Logos - Private messaging interface |
| [`delivery_demo`](modules/delivery_demo/README.md) | ui_qml | 0.3.0 | catalog | — | Educational UI demo for logos-delivery-module: subscribe to content topics, send and receive messages, see which delivery_module API call… |
| [`delivery_module`](modules/delivery_module/README.md) | core | 0.3.0 | catalog | [lidl](modules/delivery_module/delivery_module.lidl) | Logos Delivery Module - High-level message-delivery API |
| [`liblogos_lez_rln_module`](modules/liblogos_lez_rln_module/README.md) | core | 4.2.1 | catalog | [lidl](modules/liblogos_lez_rln_module/liblogos_lez_rln_module.lidl) | RLN registry provider (LEZ chain reads + registration/funding txs) |
| [`liblogos_rln_module`](modules/liblogos_rln_module/README.md) | core | 0.10.0 | catalog | [lidl](modules/liblogos_rln_module/liblogos_rln_module.lidl) | Registry-agnostic RLN membership management (RLN-MEMBERSHIP-MANAGEMENT) |
| [`libp2p_module`](modules/libp2p_module/README.md) | core | 1.1.0 | catalog | [lidl](modules/libp2p_module/libp2p_module.lidl) | Libp2p network protocol module for Logos |

### [Monero wallet](stacks/monero-wallet.md)

| Module | Type | Version | Where | Contract | Description |
|---|---|---|---|---|---|
| [`monero_node_module`](modules/monero_node_module/README.md) | core | 0.1.0 | catalog | [lidl](modules/monero_node_module/monero_node_module.lidl) | Proxyable, fail-closed monerod JSON-RPC client. Per-network config (endpoint + proxy); socks5h/Tor-ready. Never runs a node itself; local… |
| [`monero_wallet_backend`](modules/monero_wallet_backend/README.md) | core | 0.1.0 | catalog | [lidl](modules/monero_wallet_backend/monero_wallet_backend.lidl) | Monero wallet coordinator: wallet registry, sync poller, balances and history, send orchestration (build → review → broadcast). Holds no … |
| [`monero_wallet_cli`](modules/monero_wallet_cli/README.md) | core | 0.1.0 | catalog | [lidl](modules/monero_wallet_cli/monero_wallet_cli.lidl) | Headless Monero wallet for monero_wallet_backend: open or create a wallet, read the balance and addresses, build a transfer, review it an… |
| [`monero_wallet_core_module`](modules/monero_wallet_core_module/README.md) | core | 0.1.0 | catalog | [lidl](modules/monero_wallet_core_module/monero_wallet_core_module.lidl) | In-process Monero wallet engine: wraps monero_c (wallet2 C ABI, LGPL-3.0, dynamically linked, built from source). Keys and the wallet pas… |
| [`monero_wallet_ui`](modules/monero_wallet_ui/README.md) | ui_qml | 0.1.0 | catalog | — | The Monero wallet: balances, a reviewed send, receive addresses with a QR, activity, and the wallet management (create, restore, open, cl… |
| [`monerod_module`](modules/monerod_module/README.md) | core | 0.1.0 | catalog | [lidl](modules/monerod_module/monerod_module.lidl) | Runs a Monero node in-process: per-network config, start, stop, status and log tail. |
| [`monerod_ui`](modules/monerod_ui/README.md) | ui_qml | 0.1.0 | catalog | — | Run and manage a local Monero node: sync progress, peers, settings and the node log. |

### [Platform](stacks/platform.md)

| Module | Type | Version | Where | Contract | Description |
|---|---|---|---|---|---|
| [`accounts_ui`](modules/accounts_ui/README.md) | ui_qml | 0.3.0 | catalog | — | Create and manage Logos accounts and their keys. |
| [`capability_module`](modules/capability_module/README.md) | core | 1.0.0 | bundled | [lidl](modules/capability_module/capability_module.lidl) | Coordinates permissions between modules |
| [`modules_state`](modules/modules_state/README.md) | core | 0.1.0 | bundled | [lidl](modules/modules_state/modules_state.lidl) | Read-only registry of module lifecycle state |
| [`openmetrics`](modules/openmetrics/README.md) | core | 0.1.1 | catalog | [lidl](modules/openmetrics/openmetrics.lidl) | Serves an OpenMetrics /metrics endpoint by scraping modules that implement collectMetrics() |
| [`package_downloader`](modules/package_downloader/README.md) | core | 1.0.0 | bundled | [lidl](modules/package_downloader/package_downloader.lidl) | Online package catalog and download service |
| [`package_manager`](modules/package_manager/README.md) | core | 1.0.0 | bundled | [lidl](modules/package_manager/package_manager.lidl) | Plugin manager for the Logos system |
| [`package_manager_ui`](modules/package_manager_ui/README.md) | ui_qml | 1.0.0 | bundled | — | Package Manager UI plugin for managing plugins and packages |

### [Storage](stacks/storage.md)

| Module | Type | Version | Where | Contract | Description |
|---|---|---|---|---|---|
| [`storage_module`](modules/storage_module/README.md) | core | 3.0.0 | bundled + catalog | [lidl](modules/storage_module/storage_module.lidl) | Storage module |
| [`storage_ui`](modules/storage_ui/README.md) | ui_qml | 3.0.0 | catalog | — | Storage interface for the Logos application |

<!-- registry:end -->
