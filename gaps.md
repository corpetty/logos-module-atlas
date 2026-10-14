# Roadmap vs. what ships in Basecamp 0.3.2

Checked **2026-10-14**. This compares the v0.3 testnet scope in
[`logos-co/roadmap` `content/testnets/v03.md`](https://github.com/logos-co/roadmap/blob/d8eec6ed449a17dd56e2eb43787f618bdc3c8bd2/content/testnets/v03.md)
(branch `v5` @ `d8eec6e`; the page itself is unchanged since `f6da798`, 2026-07-20) with Basecamp
[0.3.2](https://github.com/logos-co/logos-basecamp/releases/tag/0.3.2) (tag `f2fae63`, 2026-10-06):
its 6 bundled modules (`package_manager`, `package_downloader`, `capability_module`,
`modules_state`, `package_manager_ui`, `storage_module`) and the 46 packages in
[`logos-co/logos-modules-release`](https://github.com/logos-co/logos-modules-release)
([index](https://github.com/logos-co/logos-modules-release/releases/download/index/index.json),
2026-10-14T12:39Z). The 0.3.2 notes are fixes and relocks, and none touches a roadmap item here. The
[v0.3 release notes](https://github.com/logos-co/roadmap/blob/d8eec6ed449a17dd56e2eb43787f618bdc3c8bd2/content/testnets/v03-release.md)
(last changed 2026-09-29, no 0.3.2 section) are cited where they differ. None of these, the
[dev catalog](https://github.com/logos-co/logos-modules-dev) (11 packages, 2026-10-14) or
[`logos-release-set`](https://github.com/logos-co/logos-release-set) (`6f19129`; its only wallet is
`lez_wallet_ui`) has a Bitcoin, Zcash, token, oracle or mix entry.

**Since the 2026-10-01 check:**

- **Bitcoin:** 11 `logos-co` repos for a watch-only wallet stack appeared on 2026-10-13, and the keys
  app has Bitcoin wallets on `main`. Still no release or catalog entry. [§ Bitcoin](#bitcoin)
- **Zcash:** 8 `logos-co` repos for a Tor light-client wallet stack appeared on 2026-10-08. Same: no
  release or catalog entry. [§ Zcash](#zcash)
- **RFP-003 (swaps):** M1 is clear for payment and M3 is accepted at v0.2.6 (2026-10-09). M2 is not
  submitted, and M6 stays open for ZEC.
- **Mix correction:** Delivery's presets turn mix on, but Chat's source never asks to send through
  it. [§ Mix](#mix)
- **LEZ testnet reset:** the testnet moved to LEZ v0.3.1, and the RLN modules were bumped for it.
  Whether `token`, `amm` and `twap_oracle` are deployed after the reset is **unverified**.
  [§ RLN Membership](#rln-membership), [§ Token](#token-lez-program)
- **Oracle:** the AnonComms oracle deliverables were closed and re-planned for v0.4, and RFP-019
  gained milestone issues. [§ Oracle](#oracle)

## Summary

| Roadmap item | Status in 0.3.2 catalog | Where the work lives | Evidence |
|---|---|---|---|
| Chat, Delivery, LEZ, LEZ Indexer, Blockchain, Storage (modules) | Shipped | `chat_module` 0.3.0, `delivery_module` 0.3.2, `lez_core` 0.5.1 + `logos_execution_zone` 1.0.0, `lez_indexer_module` 1.2.1, `blockchain_module` 0.3.2, `storage_module` 3.0.2 | Catalog index 2026-10-14 |
| RLN Membership | Shipped. Proof validation is still off | `liblogos_rln_module` 0.10.0, `liblogos_lez_rln_module` 4.2.1 | [§ RLN Membership](#rln-membership) |
| Service Discovery | **Partial**: inside `libp2p_module`, opt-in, not a standalone package | [logos-libp2p-module](https://github.com/logos-co/logos-libp2p-module), [logos-delivery-module](https://github.com/logos-co/logos-delivery-module) | [§ Service Discovery](#service-discovery) |
| Mix | **Partial**: mix routing is on in Delivery's presets, but Chat does not opt in. The standalone mix-node module is in no catalog | [logos-libp2p-mix-rln](https://github.com/logos-co/logos-libp2p-mix-rln) | [§ Mix](#mix) |
| Wallet: Ethereum | Shipped as its own app and stack | `eth_wallet_ui`, `eth_wallet_backend`, plus 17 other EVM packages (keystore, signer, RPC, verified proxy, assets, fees, tx sender, token lists, Uniswap), all unchanged | Catalog |
| Wallet: Monero | Shipped as its own app and stack | `monero_wallet_ui`, `monero_wallet_backend`, `monero_wallet_core_module`, `monero_wallet_cli`, `monero_node_module`, `monerod_module`, `monerod_ui` (all 0.1.1) | Catalog. [stacks/monero-wallet.md](stacks/monero-wallet.md) |
| Wallet: **Bitcoin** | **Not in the release.** A wallet stack exists as 11 unreleased repos (0.1.0, no tags). A BTC↔LEZ *swap* app is in a third-party catalog | [logos-bitcoin-wallet-core-module](https://github.com/logos-co/logos-bitcoin-wallet-core-module) and 10 siblings; [gateway-fm/lez-atomic-swaps](https://github.com/gateway-fm/lez-atomic-swaps) (RFP-003) | [§ Bitcoin](#bitcoin) |
| Wallet: **Zcash** | **Not in the release.** A wallet stack exists as 8 unreleased repos (0.1.0, no tags). Nothing installable | [logos-zcash-wallet-core-module](https://github.com/logos-co/logos-zcash-wallet-core-module) and 7 siblings; RFP-003 M2, not submitted | [§ Zcash](#zcash) |
| One "Wallet" module/app covering all four chains | **Absent.** ETH and XMR ship as two separate apps; BTC and ZEC are two more separate stacks | — | Release notes list "Monero wallet" and "EVM wallets" separately |
| LEZ program: Token | **No Basecamp package** in any catalog. On-chain status after the testnet reset is **unverified** | [lez-programs](https://github.com/logos-blockchain/lez-programs) | [§ Token](#token-lez-program) |
| LEZ program: DEX / App: DEX | Shipped | `amm_module` 0.1.0, `amm_ui` 0.1.0 ("Logos DEX App"). `amm` was deployed on testnet v0.2.4; after the reset, **unverified** | Catalog. [§ Token](#token-lez-program) |
| LEZ program: Oracle | **No module or app.** On-chain status after the reset is **unverified** | lez-programs `twap_oracle`, [logos_oracle_network](https://github.com/logos-co/logos_oracle_network) | [§ Oracle](#oracle) |
| Apps: Blockchain Dashboard, LEZ Wallet, LEZ Explorer, Chat, Storage | Shipped | `blockchain_ui` 0.3.2, `lez_wallet_ui` 1.2.2, `lez_explorer_ui` 1.2.3, `chat_ui` 0.3.0, `storage_ui` 3.0.2 | Catalog. Release notes say Chat uses mix through Delivery; the source does not show that ([§ Mix](#mix)) |

## Bitcoin

**Not in the release.** No Bitcoin package is in the default catalog, the dev catalog or the release
set. Two bodies of work exist outside them: an unreleased wallet stack, and atomic-swap work.

- **Wallet stack** (11 public `logos-co` repos, all created 2026-10-13): wallet core
  ([`logos-bitcoin-wallet-core-module`](https://github.com/logos-co/logos-bitcoin-wallet-core-module)),
  `-wallet-backend`, `-wallet-cli`, `-wallet-ui`, `logos-bitcoin-node-module`, `logos-bitcoind-module`,
  `logos-bitcoind-ui`, `logos-bitcoin-core-nix`, `logos-bitcoin-peer-list`,
  `logos-bitcoin-privacy-tests` and `logos-bitcoin-windows-tests`. The modules'
  [`metadata.json`](https://github.com/logos-co/logos-bitcoin-wallet-core-module/blob/90c2e1a5b1e860e3e334e96c23a139b060522021/metadata.json#L4)
  files say 0.1.0. None has a release or tag, and none is in a catalog.
  - The core is a watch-only wallet on
    [`bdk_wallet` 3.2.0 and `bdk_kyoto` 0.18.0](https://github.com/logos-co/logos-bitcoin-wallet-core-module/blob/90c2e1a5b1e860e3e334e96c23a139b060522021/rust-lib/Cargo.toml#L18-L23).
    Its [README](https://github.com/logos-co/logos-bitcoin-wallet-core-module/blob/90c2e1a5b1e860e3e334e96c23a139b060522021/README.md#L3-L6)
    says it "holds no keys" and signs through `signer_manager_module`, which
    [`metadata.json`](https://github.com/logos-co/logos-bitcoin-wallet-core-module/blob/90c2e1a5b1e860e3e334e96c23a139b060522021/metadata.json#L12)
    lists as a dependency. `signer_manager_module` is in no catalog either.
  - Networks: [mainnet, testnet4, signet and regtest](https://github.com/logos-co/logos-bitcoin-wallet-core-module/blob/90c2e1a5b1e860e3e334e96c23a139b060522021/rust-lib/src/network.rs#L13-L19)
    (regtest is for test harnesses).
  - The node module's [README](https://github.com/logos-co/logos-bitcoin-node-module/blob/7462bce96676bad4be296155117bd6ea6c643a4d/README.md#L102-L105)
    says its peer list is signed with the crawler's "TEST key" and "the release key is not chosen
    yet", and that the shipped lists are "provisional".
- **Signer stack** (public parts created 2026-10-10):
  [`logos-signer-interface`](https://github.com/logos-co/logos-signer-interface) (`logos.signer/1`),
  [`logos-signer-manager-module`](https://github.com/logos-co/logos-signer-manager-module), and the
  keycard modules and UIs ([`logos-keycard-module`](https://github.com/logos-co/logos-keycard-module),
  [`-keycard-shell-module`](https://github.com/logos-co/logos-keycard-shell-module),
  [`-keycard-shell-ui`](https://github.com/logos-co/logos-keycard-shell-ui),
  [`-keycard-ui`](https://github.com/logos-co/logos-keycard-ui)). All 0.1.0, none in a catalog. The
  [interface README](https://github.com/logos-co/logos-signer-interface/blob/a80206330ae18be12710fac83c1b2feb73a96dc9/README.md)
  lists PSBTs among what its test signer handles.
- **Keystore:** Bitcoin support is on `main` of the EVM keystore repos but not in the release.
  [logos-evm-keystore-module#26](https://github.com/logos-co/logos-evm-keystore-module/pull/26)
  (BIP84/BIP86 keys and PSBT signing, "spike", merged 2026-10-10) and
  [logos-evm-keystore-ui#13](https://github.com/logos-co/logos-evm-keystore-ui/pull/13) (Bitcoin
  wallets in the keys app, merged 2026-10-10). The released `keystore_module` is 0.1.0 (2026-09-28),
  built from `2318c67`.
- **[gateway-fm/lez-atomic-swaps](https://github.com/gateway-fm/lez-atomic-swaps)** (created
  2026-08-20; latest release [v0.2.6](https://github.com/gateway-fm/lez-atomic-swaps/releases/tag/v0.2.6),
  2026-10-08, docs plus an example) is the RFP-003 grantee Gateway's repo
  ([RFP](https://github.com/logos-co/rfp/blob/master/RFPs/RFP-003-atomic-swaps.md); proposal
  [rfp#112](https://github.com/logos-co/rfp/issues/112), accepted 2026-07-10).
  It does native BTC↔LEZ swaps with Taproot/MuSig2 adaptor signatures. Its two `ui_qml` swap desks,
  `lez_atomic_swap_maker` and `lez_atomic_swap_taker` ("LEZ / BTC Maker/Taker"), are still 0.2.5 and
  published only in the third-party catalog
  [mandrigin/logos-modules-release-base](https://github.com/mandrigin/logos-modules-release-base).
  They depend on `chat_module` and `delivery_module`, and drive Maker/Taker Node services and a
  Bitcoin Core node that run outside Basecamp.
  - Its [README](https://github.com/gateway-fm/lez-atomic-swaps/blob/4b76e2f7f31616ca1415f3ea0599610a68ad2bb4/README.md#L18-L22)
    still says "Public-network operation, production key management, and an independent
    cryptographic audit are still in progress."
  - [Issue #106](https://github.com/gateway-fm/lez-atomic-swaps/issues/106) (2026-10-12) asks to
    move that catalog to a `gateway-fm` org repo. [Issue #107](https://github.com/gateway-fm/lez-atomic-swaps/issues/107)
    says the escrow program "needs rewriting, not porting" for LEZ 0.4.
- **Milestones** (tracker [ecosystem#182](https://github.com/logos-co/ecosystem/issues/182)):
  - M1 ([rfp#121](https://github.com/logos-co/rfp/issues/121)): ["M1 is clear for payment"](https://github.com/logos-co/rfp/issues/121#issuecomment-6080097469) (2026-10-09).
  - M3 BTC leg ([rfp#123](https://github.com/logos-co/rfp/issues/123)): [accepted at v0.2.6](https://github.com/logos-co/rfp/issues/123#issuecomment-6079909704)
    (`4b76e2f7`, 2026-10-09).
  - M6 GUIs ([rfp#126](https://github.com/logos-co/rfp/issues/126)): the tracker's
    [9 October update](https://github.com/logos-co/ecosystem/issues/182#issuecomment-5616692159)
    says "M6 stays open for the ZEC app flow" and the native Bitcoin proof "is still blocked by the
    local LEZ chain freeze" ([logos-blockchain#3643](https://github.com/logos-blockchain/logos-blockchain/issues/3643), open).
    It also says "M2, M4, M5 and M7 are not yet submitted". M7 audit is
    [rfp#127](https://github.com/logos-co/rfp/issues/127) (open).
- **Related:**
  - [ecosystem#206](https://github.com/logos-co/ecosystem/issues/206) "[RFP] Wrapped Bitcoin":
    open since 2026-08-11, one-line placeholder, unchanged.
  - Draft maker-operator L-Prizes: [lambda-prize#165](https://github.com/logos-co/lambda-prize/pull/165)
    and [#166](https://github.com/logos-co/lambda-prize/pull/166) (open, untouched since 2026-09-25).
    LP-0018 BTC-LEZ swap ([lambda-prize#28](https://github.com/logos-co/lambda-prize/pull/28)):
    closed unmerged 2026-08-05.
  - [logos-co/atomic-swaps-poc](https://github.com/logos-co/atomic-swaps-poc) (formerly
    `eth-lez-atomic-swaps`): ETH↔LEZ only.
  - [ecosystem#254](https://github.com/logos-co/ecosystem/issues/254) (docs planning, 2026-10-07)
    lists a "Bitcoin wallet module developer doc" as "Agreed: we should have one".
  - Adjacent: [corpetty/muster](https://github.com/corpetty/muster) (personal repo), a multisig
    coordination client. Its wallet layer has a Bitcoin Core adapter. It is in neither the default
    nor the dev catalog; it keeps its own (`catalog/index.json`).
- **None found:** a Bitcoin package in any catalog. The old claims that no Bitcoin branches or
  `metadata.json` exist no longer hold: the repos above exist, and `gh search code bitcoin
  --filename metadata.json --owner logos-co` lists them plus `logos-evm-keystore-cli` and
  `logos-evm-keystore-ui`. BDK is now used for real by the wallet core; [ecosystem#238](https://github.com/logos-co/ecosystem/issues/238)
  was re-scoped (updated 2026-10-13) away from a "BDK-shaped" kit, and
  [#236](https://github.com/logos-co/ecosystem/issues/236) no longer mentions BDK.

## Zcash

**Not in the release.** Nothing Zcash-related is installable from any catalog.

- **Wallet stack** (8 public `logos-co` repos, all created 2026-10-08):
  [`logos-zcash-wallet-core-module`](https://github.com/logos-co/logos-zcash-wallet-core-module),
  `-wallet-backend`, `-wallet-cli`, `-wallet-ui`, `logos-zcash-node-module` (the lightwalletd servers
  a wallet talks to), `logos-zebrad-module`, `logos-zebrad-ui` and `logos-zebra-nix`. All 0.1.0 per
  their `metadata.json`; no releases or tags; in no catalog.
  - The core's [README](https://github.com/logos-co/logos-zcash-wallet-core-module/blob/cd7a032a5edfa6cfb510d2ef7cdeb5b6f4a67271/README.md#L3-L5)
    calls it a light client "built on librustzcash's pre-release NU7 cohort"
    ([`zcash_client_backend` `=0.25.0-pre.1`](https://github.com/logos-co/logos-zcash-wallet-core-module/blob/cd7a032a5edfa6cfb510d2ef7cdeb5b6f4a67271/rust-lib/Cargo.toml#L16)).
    It syncs [over Tor](https://github.com/logos-co/logos-zcash-wallet-core-module/blob/cd7a032a5edfa6cfb510d2ef7cdeb5b6f4a67271/README.md#L18-L24).
  - The same README says a send [spends from "one shielded pool at a time, Ironwood first"](https://github.com/logos-co/logos-zcash-wallet-core-module/blob/cd7a032a5edfa6cfb510d2ef7cdeb5b6f4a67271/README.md#L35)
    and describes a regtest that [runs "the ZIP 318 migration of the Orchard coinbase"](https://github.com/logos-co/logos-zcash-wallet-core-module/blob/cd7a032a5edfa6cfb510d2ef7cdeb5b6f4a67271/README.md#L112-L115).
    Transparent shielding is also supported.
- [logos-workspace#107](https://github.com/logos-co/logos-workspace/pull/107) (open) adds Zcash apps
  to the workspace, per its title ("add EVM wallet, keycard, monero, zcash, uplink apps").
- **RFP-003 M2, the ZEC-LEZ leg** ([rfp#122](https://github.com/logos-co/rfp/issues/122)): open
  since 2026-07-20, with no comments, and the tracker says M2 is "not yet submitted".
  - Scope: BIP-199 HTLCs on transparent `t1…` addresses, using `librustzcash`/`zcash_primitives`.
  - Shielded swaps (Sapling/Orchard) are out of scope; see
    [appendix/zcash-atomic-swap-primitives.md](https://github.com/logos-co/rfp/blob/master/appendix/zcash-atomic-swap-primitives.md).
- **Code in gateway-fm/lez-atomic-swaps:** `zec-swap-sdk`, `zec-reference-actor` and
  `zebra-node-adapter` sit behind the `pair-zec` feature, which is off by default.
  [docs/m6-zec-reconciliation.md](https://github.com/gateway-fm/lez-atomic-swaps/blob/4b76e2f7f31616ca1415f3ea0599610a68ad2bb4/docs/m6-zec-reconciliation.md#L32-L48)
  ([#85](https://github.com/gateway-fm/lez-atomic-swaps/pull/85), merged 2026-09-21) records:
  - ZEC was proven only at the service layer, against Zebra Regtest, on 2026-08-04.
  - ZEC was removed from the desks on 2026-08-26, and the shipped Nodes are built without it.
  - There has "never been one run from a desk click to a ZEC chain effect".
- **M6:** per the tracker's 9 October update, "M6 stays open for the ZEC app flow".
- [ecosystem#254](https://github.com/logos-co/ecosystem/issues/254) notes that the planned-scope
  page lists Zcash wallet support but the release notes don't.

## Service Discovery

**Partial.** Shipped inside `libp2p_module` 1.1.0 (catalog pin `49f7743`), not as a separate package,
and off by default.

- `libp2p_module` has `mountServiceDiscovery`, [default `false`](https://github.com/logos-co/logos-libp2p-module/blob/49f77430fbe6cc8dbb8eee2361670d7396565e5b/src/config.h#L45),
  and `disco*` methods, added in logos-libp2p-module #40 (2026-04-27) and #53 (2026-06-16).
- `delivery_module` 0.3.2 lists `libp2p_module` as an [`optional_dependency`](https://github.com/logos-co/logos-delivery-module/blob/c25b7868099949c375e7affdeb1502b354fd92e3/metadata.json#L15).
  It installs the libp2p discovery plugin [only when the node reports `externalServiceDiscovery: true`](https://github.com/logos-co/logos-delivery-module/blob/c25b7868099949c375e7affdeb1502b354fd92e3/src/delivery_module_plugin.cpp#L750-L782)
  ([parsing](https://github.com/logos-co/logos-delivery-module/blob/c25b7868099949c375e7affdeb1502b354fd92e3/src/discovery_config.h#L102-L139)).
  Wired in `ba4c5a7` (#97, 2026-09-27), building on [logos-delivery#4178](https://github.com/logos-messaging/logos-delivery/pull/4178).
- Both presets list "Kademlia discovery" as on in
  [networks.md](https://github.com/logos-co/logos-delivery-module/blob/c25b7868099949c375e7affdeb1502b354fd92e3/docs/pages/networks.md#L34-L38),
  which is Delivery's internal discovery, not the plugin. `libp2p_module` has
  [no build for Windows](https://github.com/logos-co/logos-delivery-module/blob/c25b7868099949c375e7affdeb1502b354fd92e3/flake.nix#L158).
- 0.3.2 includes [#155](https://github.com/logos-co/logos-delivery-module/pull/155) (merged
  2026-10-05): libp2p gets every bootstrap peer, not only the first.
  [#157](https://github.com/logos-co/logos-delivery-module/issues/157) (open, 2026-10-02) proposes
  using the plugin in `logos.dev`.
- [anoncomms-pm#64](https://github.com/logos-co/anoncomms-pm/issues/64) and
  [#36](https://github.com/logos-co/anoncomms-pm/issues/36) were closed on 2026-10-12. The v0.4
  [discovery page](https://github.com/logos-co/roadmap/blob/d8eec6ed449a17dd56e2eb43787f618bdc3c8bd2/content/anoncomms/roadmap/testnet_v0.4/discovery_v0.4.md)
  starts with [anoncomms-pm#89](https://github.com/logos-co/anoncomms-pm/issues/89) "Simplify the
  service discovery module API" (open, 2026-10-13).
- Not in the catalog: [logos-libp2p-module v1.1.1](https://github.com/logos-co/logos-libp2p-module/releases/tag/v1.1.1)
  (2026-10-06, adds `kadWaitBootstrap`). Also open: [logos-libp2p-module#113](https://github.com/logos-co/logos-libp2p-module/pull/113)
  and [logos-docs#561](https://github.com/logos-co/logos-docs/issues/561), a "run a Delivery node
  with service discovery" journey that requires `delivery_module` 0.3.2 or later.

## Mix

**Partial.** Delivery has mix routing on in both network presets, but Chat's sends do not use it.

- Mix routing is "on" for `logos.dev` and `logos.test` in
  [networks.md](https://github.com/logos-co/logos-delivery-module/blob/c25b7868099949c375e7affdeb1502b354fd92e3/docs/pages/networks.md#L34-L38)
  and in [`networks_config.nim`](https://github.com/logos-messaging/logos-delivery/blob/3a478e90a4813685af47b4f77e1f6232ac5b50bb/logos_delivery/waku/factory/networks_config.nim#L89)
  ([L127](https://github.com/logos-messaging/logos-delivery/blob/3a478e90a4813685af47b4f77e1f6232ac5b50bb/logos_delivery/waku/factory/networks_config.nim#L127)),
  the logos-delivery revision `delivery_module` 0.3.2 locks.
- **The release notes say "Chat uses RLN and mix through Delivery"; the source does not show it.**
  `chat_module` 0.3.0 creates its node with only
  [`mode`, `preset` and a log level](https://github.com/logos-co/logos-chat-module/blob/5053dd5062817f34a93055435781c1c448c7ce6c/rust-lib/src/actions.rs#L260-L264),
  and sets no `anonymityLevel`. The default is `None` ([delivery_module doc](https://github.com/logos-co/logos-delivery-module/blob/c25b7868099949c375e7affdeb1502b354fd92e3/src/delivery_module_plugin.h#L85-L87)),
  and the mix send processor is added only for
  [`Preferred` or `Required`](https://github.com/logos-messaging/logos-delivery/blob/3a478e90a4813685af47b4f77e1f6232ac5b50bb/logos_delivery/messaging/delivery_service/send_service/send_service.nim#L117-L129).
  So the node has mix mounted, but Chat's sends take the plain path. This held on 2026-10-01 too;
  Chat's use of RLN was not checked.
- The release notes' "Mix module: Enables running mix middle nodes" is
  [logos-co/logos-libp2p-mix-rln](https://github.com/logos-co/logos-libp2p-mix-rln)
  (`libp2p_mix_rln_module` 0.1.0, created 2026-08-13, last commit `9ebf684`, 2026-10-12).
  It is in neither the default nor the dev catalog and has no releases or tags, so it must be built
  from source. [logos-modules-release#74](https://github.com/logos-co/logos-modules-release/pull/74)
  (open draft, 2026-09-29) would add it, the first sign of a catalog entry.
- Integration landed on `master` branches, unreleased: [logos-delivery#4282](https://github.com/logos-messaging/logos-delivery/pull/4282)
  (merged 2026-10-08, in no tag yet) and [logos-delivery-module#148](https://github.com/logos-co/logos-delivery-module/pull/148)
  (merged 2026-10-12, after v0.3.2), which reapplies mix routing with a shared RLN backend.
- v0.4: [anoncomms-pm#91](https://github.com/logos-co/anoncomms-pm/issues/91) (open, 2026-10-13)
  plans one Logos Mix module that acts as intermediate, entry or exit node; see the
  [v0.4 mix page](https://github.com/logos-co/roadmap/blob/d8eec6ed449a17dd56e2eb43787f618bdc3c8bd2/content/anoncomms/roadmap/testnet_v0.4/mix_v0.4.md).

## RLN Membership

**Shipped** as `liblogos_rln_module` 0.10.0 and `liblogos_lez_rln_module` 4.2.1, with validation off.

- The release notes say "RLN proof validation: Activation is planned approximately two weeks after
  release" ([v03-release.md](https://github.com/logos-co/roadmap/blob/d8eec6ed449a17dd56e2eb43787f618bdc3c8bd2/content/testnets/v03-release.md),
  dated 2026-09-29). As of 2026-10-14, 15 days on, it is still off: the `logos.test` preset has
  `.enableValidation = false` in `delivery_module` 0.3.2
  ([`rln_presets.cpp`](https://github.com/logos-co/logos-delivery-module/blob/c25b7868099949c375e7affdeb1502b354fd92e3/src/rln_presets.cpp#L57))
  and on `main` at [`7d2e057`](https://github.com/logos-co/logos-delivery-module/blob/7d2e057687c170e93a34f519d682b1d84294b2d1/src/rln_presets.cpp#L57).
  The networks page agrees: "RLN | off | on — LEZ testnet registry, 600 s epochs, validation off".
- [logos-modules-release#115](https://github.com/logos-co/logos-modules-release/pull/115) (merged
  2026-10-14T18:25Z) bumps them to `liblogos_rln_module` 0.12.0 and `liblogos_lez_rln_module` 5.1.0
  for LEZ v0.3.1. Not published yet: the index above (12:39Z) predates it and shows 0.10.0 and 4.2.1.
  Its notes say `delivery_module` 0.4.0 ([logos-delivery-module#160](https://github.com/logos-co/logos-delivery-module/pull/160))
  follows in its own bump.
- [logos-rln-modules#47](https://github.com/logos-co/logos-rln-modules/pull/47) (merged 2026-10-14)
  points the registry at `testnet-v03-z3` in place of `testnet-v03-z1`, "gone with the reset".
- v0.4: [anoncomms-pm#95](https://github.com/logos-co/anoncomms-pm/issues/95) (a standalone
  membership allocation service module) and [#96](https://github.com/logos-co/anoncomms-pm/issues/96)
  (on-chain memberships and accounting compatible with Logos zones and LEZ), in the
  [v0.4 RLN page](https://github.com/logos-co/roadmap/blob/d8eec6ed449a17dd56e2eb43787f618bdc3c8bd2/content/anoncomms/roadmap/testnet_v0.4/rln_v0.4.md).

## Token (LEZ program)

**No Basecamp package.** On-chain status after the testnet reset is **unverified**.

- [lez-programs DEPLOYMENTS.md](https://github.com/logos-blockchain/lez-programs/blob/64c01210adecd6b557dbd021a3cc94553f243fd8/DEPLOYMENTS.md)
  is unchanged since `52ec5d7` (2026-09-10). It lists `token`, `amm` and `twap_oracle` under
  "Testnet (v0.2.4)" at `https://testnet.lez.logos.co/`.
- The testnet has since been reset. LEZ [v0.3.1](https://github.com/logos-blockchain/logos-execution-zone/releases/tag/v0.3.1)
  was released 2026-10-11, and [v0.2.4-lez-legacy](https://github.com/logos-blockchain/logos-execution-zone/releases/tag/v0.2.4-lez-legacy)
  on 2026-10-13. [logos-rln-modules#47](https://github.com/logos-co/logos-rln-modules/pull/47)
  moves to a v0.3.1 registry after the reset. Whether the three programs were redeployed is
  **unverified**. [lez-programs#385](https://github.com/logos-blockchain/lez-programs/issues/385)
  "Upgrade programs to LEZ v0.2.5-rc2" is open.
- lez-programs `main` (`64c0121`, 2026-10-01) has `token_module` 0.1.0 and `token_ui` 0.1.0
  ([module](https://github.com/logos-blockchain/lez-programs/blob/64c01210adecd6b557dbd021a3cc94553f243fd8/modules/token/metadata.json),
  [app](https://github.com/logos-blockchain/lez-programs/blob/64c01210adecd6b557dbd021a3cc94553f243fd8/apps/token/metadata.json)).
  Neither is in any catalog checked (default, dev, mandrigin, 3esmit, 0x-r4bbit).
- New there: [`stablecoin_module`](https://github.com/logos-blockchain/lez-programs/blob/64c01210adecd6b557dbd021a3cc94553f243fd8/modules/stablecoin/metadata.json)
  0.1.0 (since 2026-08-26), also in no catalog.
- `logos-blockchain/logos-token-module` and `logos-token-ui-module` are still empty repos.
- For comparison, the DEX does ship, although lez-programs has a newer `amm_module`/`amm_ui`
  (0.3.0) than the catalog (0.1.0).

## Oracle

**No module or app.** The `twap_oracle` program was in DEPLOYMENTS.md for testnet v0.2.4 (see
[§ Token](#token-lez-program)); whether it exists after the reset is **unverified**.

- [RFP-019](https://github.com/logos-co/rfp/blob/master/RFPs/RFP-019-twap-oracle.md) (TWAP) and
  [RFP-020](https://github.com/logos-co/rfp/blob/master/RFPs/RFP-020-redstone-oracle-adaptor.md)
  (RedStone) are still `status: closed` to proposals.
- RFP-019 milestones [rfp#215](https://github.com/logos-co/rfp/issues/215) (M1, "LEZ testnet v0.2.4
  target") and [#216](https://github.com/logos-co/rfp/issues/216) (M2, "LEZ testnet v0.4 target")
  opened 2026-10-13. Tracker: [ecosystem#188](https://github.com/logos-co/ecosystem/issues/188).
- RFP-020: M0 ([rfp#146](https://github.com/logos-co/rfp/issues/146)) closed 2026-10-09; M1 to M5
  ([#147](https://github.com/logos-co/rfp/issues/147) to [#151](https://github.com/logos-co/rfp/issues/151))
  are open, including M2 ([rfp#148](https://github.com/logos-co/rfp/issues/148)), with no comments.
- The AnonComms [Logos Oracle Network](https://github.com/logos-co/logos_oracle_network) has no
  Logos module. [logos_oracle_network#8](https://github.com/logos-co/logos_oracle_network/issues/8)
  "Few blockers on being a module" (opened 2026-09-18, last comment 2026-10-07) notes that oracle
  nodes would be background services and that the Blockchain module has no publish surface (its
  C bindings export only `get_channel_state`).
- [anoncomms-pm#57](https://github.com/logos-co/anoncomms-pm/issues/57) ("Implementing Oracle
  Zone") and [#59](https://github.com/logos-co/anoncomms-pm/issues/59) were closed on 2026-10-12.
  The v0.4 [oracle page](https://github.com/logos-co/roadmap/blob/d8eec6ed449a17dd56e2eb43787f618bdc3c8bd2/content/anoncomms/roadmap/testnet_v0.4/oracle_v0.4.md)
  tracks [#92](https://github.com/logos-co/anoncomms-pm/issues/92) to
  [#94](https://github.com/logos-co/anoncomms-pm/issues/94) in
  [milestone 29](https://github.com/logos-co/anoncomms-pm/milestone/29), due 2026-11-23.

## How this was checked

Every "none found" above comes from these searches. The 2026-10-14 re-run covers 2026-10-01 to 2026-10-14:

- **Orgs:** logos-co (278 repos), logos-blockchain (36), logos-storage (118), vacp2p (53), status-im
  (847) and logos-messaging (88): 1,420 in all, up from 1,382. `gh repo list` covered all of them,
  grepped for btc|bitcoin|zec|zcash|zebra|orchard|sapling|bdk|electrum|wallet|signer|keycard|swap|mix|discover|oracle|token.
- **`gh search repos`:** bitcoin, btc, zcash and zec in each org. On all of GitHub: "lez atomic
  swap" and logos/lez/basecamp × bitcoin/zcash, also `in:readme`.
- **`gh search issues --include-prs`:** bitcoin and zcash in all six orgs. btc, zec, sapling,
  orchard, zebrad, zingo, lightwalletd, bitcoind, librustzcash, bdk, electrum, "bitcoin wallet" and
  "zcash wallet" in logos-co and/or logos-blockchain.
- **`gh search code`:** zcash, librustzcash, bdk_wallet, zebra and electrum in logos-co; zcash and
  bitcoin `filename:metadata.json` in logos-co and logos-blockchain.
- **Third-party catalogs:** `gh search code` for `logos-repo.json` found 20. This pass re-read 16 of
  their indexes (the other candidates had no index URL there); none lists a Zcash, token or
  oracle package. The only Bitcoin packages are the two swap desks (read from the `mandrigin` index
  directly). `atomic-swaps-poc`'s `swap` and `swap_ui` are ETH↔LEZ.
- **Also read:** the default and dev catalog indexes (2026-10-14), `release-set.json` at `6f19129`,
  the gateway catalog index, and the roadmap's v0.3 pages and v0.4 deliverable pages
  ([roadmap PR #557](https://github.com/logos-co/roadmap/pull/557) and [#576](https://github.com/logos-co/roadmap/pull/576),
  merged 2026-10-13). There is no v0.4 testnet page yet.
- **Code claims:** for repos without releases, the links above are permalinks at the commit `HEAD`
  pointed to on 2026-10-14. Source reads for Delivery and Chat use the commits their catalog packages pin.
- **Roadmap revision read:** `logos-co/roadmap` branch `v5` (`d8eec6e`).
