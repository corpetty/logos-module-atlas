# Roadmap vs. what ships in Basecamp 0.3.2

Checked **2026-11-04**. This compares the v0.3 testnet scope in
[`logos-co/roadmap` `content/testnets/v03.md`](https://github.com/logos-co/roadmap/blob/ca8f30ede6958e431e322b3c330cb03eed2f7ae7/content/testnets/v03.md)
(branch `v5` @ `ca8f30e`; the page itself is unchanged since `f6da798`, 2026-07-20) with Basecamp
[0.3.2](https://github.com/logos-co/logos-basecamp/releases/tag/0.3.2) (tag `f2fae63`, 2026-10-06),
still the latest release: its 6 bundled modules (`package_manager`, `package_downloader`,
`capability_module`, `modules_state`, `package_manager_ui`, `storage_module`) and the 49 packages in
[`logos-co/logos-modules-release`](https://github.com/logos-co/logos-modules-release)
([index](https://github.com/logos-co/logos-modules-release/releases/download/index/index.json),
2026-11-04T12:26Z, the same contents as the 2026-11-02 index the registry was built from). The 0.3.2
notes are fixes and relocks, and none touches a roadmap item here. The
[v0.3 release notes](https://github.com/logos-co/roadmap/blob/ca8f30ede6958e431e322b3c330cb03eed2f7ae7/content/testnets/v03-release.md)
(last changed 2026-09-29, no 0.3.2 section) are cited where they differ. The default catalog now has
token and oracle packages. Neither it, the [dev catalog](https://github.com/logos-co/logos-modules-dev)
(11 packages, index 2026-11-04T12:27Z) nor [`logos-release-set`](https://github.com/logos-co/logos-release-set)
(`6f19129`, unchanged; its only wallet is `lez_wallet_ui`) has a Bitcoin, Zcash or mix entry.

**Since the 2026-10-14 check:**

- **Token and Oracle ship:** `token_module`, `token_ui` and `oracle_module` 0.1.0 joined the catalog on
  2026-10-30, all wrapping lez-programs `lez-v0.2.x` @ `543563a`. [§ Token](#token-lez-program), [§ Oracle](#oracle)
- **LEZ split:** those three and the DEX (now 0.3.0) need `lez_core` 0.4.3 exactly, the legacy LEZ line.
  `lez_wallet_ui` 1.2.2 needs 0.5.1 or later, so one install likely cannot satisfy both. [§ Token](#token-lez-program)
- **Mix:** `delivery_module` 0.4.0 ships mix routing on a shared RLN backend, but Chat still does not
  ask for it. [§ Mix](#mix)
- **RLN:** 0.12.0 and 5.1.0 are published for LEZ v0.3.1. Proof validation is still off, about five
  weeks after the release notes. [§ RLN Membership](#rln-membership)
- **Bitcoin and Zcash:** still no release or catalog entry. Both wallets now use the new `tor_module`,
  and RFP-003 M6 reached its native Bitcoin proof on 2026-10-15. [§ Bitcoin](#bitcoin), [§ Zcash](#zcash)

## Summary

| Roadmap item | Status in 0.3.2 catalog | Where the work lives | Evidence |
|---|---|---|---|
| Chat, Delivery, LEZ, LEZ Indexer, Blockchain, Storage (modules) | Shipped | `chat_module` 0.3.0, `delivery_module` 0.4.0, `lez_core` 0.5.1 + `logos_execution_zone` 1.0.0, `lez_indexer_module` 1.2.1, `blockchain_module` 0.3.4, `storage_module` 3.0.2 | Catalog index 2026-11-04 |
| RLN Membership | Shipped. Proof validation is still off | `liblogos_rln_module` 0.12.0, `liblogos_lez_rln_module` 5.1.0 | [§ RLN Membership](#rln-membership) |
| Service Discovery | **Partial**: inside `libp2p_module`, opt-in, not a standalone package | [logos-libp2p-module](https://github.com/logos-co/logos-libp2p-module), [logos-delivery-module](https://github.com/logos-co/logos-delivery-module) | [§ Service Discovery](#service-discovery) |
| Mix | **Partial**: `delivery_module` 0.4.0 ships mix routing and its presets mount mix, but Chat does not opt in. The standalone mix-node module is in no catalog | [logos-delivery-module](https://github.com/logos-co/logos-delivery-module), [logos-libp2p-mix-rln](https://github.com/logos-co/logos-libp2p-mix-rln) | [§ Mix](#mix) |
| Wallet: Ethereum | Shipped as its own app and stack | `eth_wallet_ui`, `eth_wallet_backend`, plus 17 other EVM packages (keystore, signer, RPC, verified proxy, assets, fees, tx sender, token lists, Uniswap). Only `eth_rpc_ui` 0.1.1 and `verified_proxy_module` 0.1.4 changed | Catalog |
| Wallet: Monero | Shipped as its own app and stack | `monero_wallet_ui`, `monero_wallet_backend`, `monero_wallet_core_module`, `monero_wallet_cli`, `monero_node_module`, `monerod_module`, `monerod_ui` (all 0.1.1) | Catalog. [stacks/monero-wallet.md](stacks/monero-wallet.md) |
| Wallet: **Bitcoin** | **Not in the release.** A wallet stack exists as 11 unreleased repos (0.1.0, no tags). A BTC↔LEZ *swap* app is in a third-party catalog | [logos-bitcoin-wallet-core-module](https://github.com/logos-co/logos-bitcoin-wallet-core-module) and 10 siblings; [gateway-fm/lez-atomic-swaps](https://github.com/gateway-fm/lez-atomic-swaps) (RFP-003) | [§ Bitcoin](#bitcoin) |
| Wallet: **Zcash** | **Not in the release.** A wallet stack exists as 8 unreleased repos (0.1.0, no tags). Nothing installable | [logos-zcash-wallet-core-module](https://github.com/logos-co/logos-zcash-wallet-core-module) and 7 siblings; RFP-003 M2, not submitted | [§ Zcash](#zcash) |
| One "Wallet" module/app covering all four chains | **Absent.** ETH and XMR ship as two separate apps; BTC and ZEC are two more separate stacks | — | Release notes list "Monero wallet" and "EVM wallets" separately |
| LEZ program: Token | **Shipped, for the legacy LEZ line**: `token_module` 0.1.0, `token_ui` 0.1.0. They need the legacy `lez_core`, which `lez_wallet_ui` 1.2.2 rules out. The programs are recorded only on the legacy testnet | lez-programs `lez-v0.2.x` (`543563a`), via [logos-token-module](https://github.com/logos-blockchain/logos-token-module) and [logos-token-ui-module](https://github.com/logos-blockchain/logos-token-ui-module) | [§ Token](#token-lez-program) |
| LEZ program: DEX / App: DEX | Shipped, for the legacy LEZ line | `amm_module` 0.3.0, `amm_ui` 0.3.0 ("Logos DEX App"), with the same legacy `lez_core` pin as Token | Catalog. [§ Token](#token-lez-program) |
| LEZ program: Oracle | **Module shipped, no app**: `oracle_module` 0.1.0, legacy LEZ line. RFP-019 M1, which asks for that module, is not accepted | lez-programs `twap_oracle`, via [logos-twap-oracle-module](https://github.com/logos-blockchain/logos-twap-oracle-module); [logos_oracle_network](https://github.com/logos-co/logos_oracle_network) | [§ Oracle](#oracle) |
| Apps: Blockchain Dashboard, LEZ Wallet, LEZ Explorer, Chat, Storage | Shipped | `blockchain_ui` 0.3.5, `lez_wallet_ui` 1.2.2, `lez_explorer_ui` 1.2.3, `chat_ui` 0.3.0, `storage_ui` 3.0.2 | Catalog. Release notes say Chat uses mix through Delivery; the source does not show that ([§ Mix](#mix)) |

## Bitcoin

**Not in the release.** No Bitcoin package is in the default catalog, the dev catalog or the release
set. Two bodies of work exist outside them: an unreleased wallet stack, and atomic-swap work.

- **Wallet stack** (11 public `logos-co` repos, all created 2026-10-13, all pushed to since 2026-10-28):
  wallet core ([`logos-bitcoin-wallet-core-module`](https://github.com/logos-co/logos-bitcoin-wallet-core-module)),
  `-wallet-backend`, `-wallet-cli`, `-wallet-ui`, `logos-bitcoin-node-module`, `logos-bitcoind-module`,
  `logos-bitcoind-ui`, `logos-bitcoin-core-nix`, `logos-bitcoin-peer-list`,
  `logos-bitcoin-privacy-tests` and `logos-bitcoin-windows-tests`. The modules'
  [`metadata.json`](https://github.com/logos-co/logos-bitcoin-wallet-core-module/blob/226707c52b44f782a8007637066f48fcf2c51b08/metadata.json#L4)
  files still say 0.1.0. None has a release or tag, and none is in a catalog.
  - The core is a watch-only wallet on
    [`bdk_wallet` 3.2.0, `bdk_electrum` 0.24.1, a patched `bdk_kyoto` 0.18.1 and `bdk_bitcoind_rpc` 0.22.1](https://github.com/logos-co/logos-bitcoin-wallet-core-module/blob/226707c52b44f782a8007637066f48fcf2c51b08/rust-lib/Cargo.toml#L17-L24).
    Its [README](https://github.com/logos-co/logos-bitcoin-wallet-core-module/blob/226707c52b44f782a8007637066f48fcf2c51b08/README.md#L3-L7)
    says it "holds no keys" and signs through `signer_manager_module`, which
    [`metadata.json`](https://github.com/logos-co/logos-bitcoin-wallet-core-module/blob/226707c52b44f782a8007637066f48fcf2c51b08/metadata.json#L12)
    lists as a dependency. `signer_manager_module` is in no catalog either.
  - Networks: [mainnet, testnet4, signet and regtest](https://github.com/logos-co/logos-bitcoin-wallet-core-module/blob/226707c52b44f782a8007637066f48fcf2c51b08/rust-lib/src/network.rs#L13-L19)
    (regtest is for test harnesses).
  - Since 2026-10-14 the stack can sync from a Bitcoin Core RPC source
    ([core #1](https://github.com/logos-co/logos-bitcoin-wallet-core-module/pull/1)), and the node module
    uses `tor_module` as its default proxy ([node #3](https://github.com/logos-co/logos-bitcoin-node-module/pull/3),
    merged 2026-10-30; see [§ Zcash](#zcash) for `tor_module`).
  - The node module's [README](https://github.com/logos-co/logos-bitcoin-node-module/blob/46d8209b083c83e7ce370ddfd7bea72d00ce3718/README.md#L170-L180)
    now says the peer lists are compiled in and not signed, and still calls them "provisional".
- **Signer stack:** [`logos-signer-interface`](https://github.com/logos-co/logos-signer-interface)
  (`logos.signer/1`), [`logos-signer-manager-module`](https://github.com/logos-co/logos-signer-manager-module),
  the keycard modules and UIs ([`logos-keycard-module`](https://github.com/logos-co/logos-keycard-module),
  [`-keycard-shell-module`](https://github.com/logos-co/logos-keycard-shell-module),
  [`-keycard-shell-ui`](https://github.com/logos-co/logos-keycard-shell-ui),
  [`-keycard-ui`](https://github.com/logos-co/logos-keycard-ui)), all created 2026-10-10. New since the
  last check: an air-gapped QR signer ([`logos-qr-signer-module`](https://github.com/logos-co/logos-qr-signer-module),
  2026-10-14, and `logos-qr-signer-ui`, with `logos-qr-scanner-module`) and
  [`logos-account-kit`](https://github.com/logos-co/logos-account-kit) (2026-10-16). All are 0.1.0,
  untagged, and in no catalog. The
  [interface README](https://github.com/logos-co/logos-signer-interface/blob/2cbefe1a5d1e5704f4250ec07cf909669477d1b6/README.md#L22-L23)
  lists PSBTs among what its test signer and its `psbt` feature handle.
- **Keystore:** Bitcoin support is on `main` of the EVM keystore repos but not in the release.
  [logos-evm-keystore-module#26](https://github.com/logos-co/logos-evm-keystore-module/pull/26)
  (BIP84/BIP86 keys and PSBT signing, merged 2026-10-10) and
  [logos-evm-keystore-ui#13](https://github.com/logos-co/logos-evm-keystore-ui/pull/13) (Bitcoin
  wallets in the keys app, merged 2026-10-10) were followed by
  [logos-evm-keystore-module#39](https://github.com/logos-co/logos-evm-keystore-module/pull/39)
  (2026-11-01), which stops the keystore keeping a Bitcoin wallet's birthday. Neither repo has a tag.
  The released `keystore_module` is still 0.1.0 (2026-09-28), built from `2318c67`.
- **[gateway-fm/lez-atomic-swaps](https://github.com/gateway-fm/lez-atomic-swaps)** (created
  2026-08-20; latest release still [v0.2.6](https://github.com/gateway-fm/lez-atomic-swaps/releases/tag/v0.2.6),
  2026-10-08) is the RFP-003 grantee Gateway's repo
  ([RFP](https://github.com/logos-co/rfp/blob/master/RFPs/RFP-003-atomic-swaps.md); proposal
  [rfp#112](https://github.com/logos-co/rfp/issues/112), accepted 2026-07-10).
  It does native BTC↔LEZ swaps with Taproot/MuSig2 adaptor signatures. Its two `ui_qml` swap desks,
  `lez_atomic_swap_maker` and `lez_atomic_swap_taker` ("LEZ / BTC Maker/Taker"), are still 0.2.5 and
  published only in the third-party catalog
  [mandrigin/logos-modules-release-base](https://github.com/mandrigin/logos-modules-release-base).
  They depend on `chat_module` and `delivery_module`, and drive Maker/Taker Node services and a
  Bitcoin Core node that run outside Basecamp.
  - Its [README](https://github.com/gateway-fm/lez-atomic-swaps/blob/4b76e2f7f31616ca1415f3ea0599610a68ad2bb4/README.md#L18-L22)
    at v0.2.6 says public-network operation, production key management and an independent
    cryptographic audit "are still in progress".
  - [Issue #106](https://github.com/gateway-fm/lez-atomic-swaps/issues/106), to move that catalog to a
    `gateway-fm` org repo, is still open; the catalog has not moved.
    [Issue #107](https://github.com/gateway-fm/lez-atomic-swaps/issues/107) (LEZ 0.4 port) is open.
  - [Issue #115](https://github.com/gateway-fm/lez-atomic-swaps/issues/115) (2026-10-16) records the
    decision to move the Docker stack into Logos modules before M2. A
    [2026-10-28 comment](https://github.com/gateway-fm/lez-atomic-swaps/issues/115#issuecomment-6465959583) asks Gateway to use
    `logos-bitcoind-module` and `logos-zebrad-module` for the chain nodes.
- **Milestones** (tracker [ecosystem#182](https://github.com/logos-co/ecosystem/issues/182)):
  - The tracker's [15 October update](https://github.com/logos-co/ecosystem/issues/182#issuecomment-5616692159)
    says "M6 Bitcoin part: native proof reached": a BTC to LEZ swap through the 0.2.5 desks in native
    Basecamp. It adds that "M6 still waits on the ZEC app flow", that M2, M4, M5 and M7 remain
    unsubmitted, and that M1 and M3 are unchanged since 9 October. There is no later update.
  - M1 ([rfp#121](https://github.com/logos-co/rfp/issues/121)): ["M1 is clear for payment"](https://github.com/logos-co/rfp/issues/121#issuecomment-6080097469) (2026-10-09).
  - M3 BTC leg ([rfp#123](https://github.com/logos-co/rfp/issues/123)): [accepted at v0.2.6](https://github.com/logos-co/rfp/issues/123#issuecomment-6079909704)
    (`4b76e2f7`, 2026-10-09). Both issues are still open.
  - M6 GUIs ([rfp#126](https://github.com/logos-co/rfp/issues/126)): a
    [hosting question](https://github.com/logos-co/rfp/issues/126#issuecomment-6193369291) (2026-10-15)
    notes that a Taker on the public LEZ testnet needs a Bedrock node, an indexer, a proxy and Bitcoin
    Core beside it. The local chain froze again on 14 to 15 October:
    [logos-blockchain#3643](https://github.com/logos-blockchain/logos-blockchain/issues/3643) is open,
    and its only comment is the reviewer's new evidence. M7 audit is
    [rfp#127](https://github.com/logos-co/rfp/issues/127) (open, no comments).
- **Related:**
  - [ecosystem#206](https://github.com/logos-co/ecosystem/issues/206) "[RFP] Wrapped Bitcoin": still a
    one-line placeholder, though it was assigned and added to a project board on 2026-10-28.
  - Draft maker-operator L-Prizes: [lambda-prize#165](https://github.com/logos-co/lambda-prize/pull/165)
    and [#166](https://github.com/logos-co/lambda-prize/pull/166) are open, with no review beyond the
    automated check. LP-0018 BTC-LEZ swap ([lambda-prize#28](https://github.com/logos-co/lambda-prize/pull/28)):
    closed unmerged 2026-08-05.
  - [logos-co/atomic-swaps-poc](https://github.com/logos-co/atomic-swaps-poc) (formerly
    `eth-lez-atomic-swaps`): ETH↔LEZ only.
  - [ecosystem#254](https://github.com/logos-co/ecosystem/issues/254) (docs planning) still lists a
    "Bitcoin wallet module developer doc" as agreed. Its 2026-10-16 comment says R&D teams will drive
    the journeys for their own features.
  - [ecosystem#258](https://github.com/logos-co/ecosystem/issues/258) "Blockchain Wallet API" (open,
    updated 2026-11-03) is the Logos Blockchain (L1) wallet, not Bitcoin.
  - Adjacent: [corpetty/muster](https://github.com/corpetty/muster) (personal repo), a multisig
    coordination client. Its wallet layer has a Bitcoin Core adapter. It is in neither the default
    nor the dev catalog; it keeps its own (`catalog/index.json`).
- **None found:** a Bitcoin package in any catalog. `gh search code bitcoin --filename metadata.json
  --owner logos-co` lists the wallet repos above plus `logos-evm-keystore-cli` and
  `logos-evm-keystore-ui`; the same search in logos-blockchain finds nothing.
  [ecosystem#238](https://github.com/logos-co/ecosystem/issues/238) stays re-scoped away from a
  "BDK-shaped" kit, and [#236](https://github.com/logos-co/ecosystem/issues/236) does not mention BDK.

## Zcash

**Not in the release.** Nothing Zcash-related is installable from any catalog.

- **Wallet stack** (8 public `logos-co` repos, all created 2026-10-08):
  [`logos-zcash-wallet-core-module`](https://github.com/logos-co/logos-zcash-wallet-core-module),
  `-wallet-backend`, `-wallet-cli`, `-wallet-ui`, `logos-zcash-node-module` (the lightwalletd servers
  a wallet talks to), `logos-zebrad-module`, `logos-zebrad-ui` and `logos-zebra-nix`. All still 0.1.0
  per their `metadata.json`; no releases or tags; in no catalog.
  - The core's [README](https://github.com/logos-co/logos-zcash-wallet-core-module/blob/e684d5fb6d8dc5165c8dc33a9c94c7bccd56dd91/README.md#L3-L5)
    now calls it a light client "built on librustzcash's NU7 release"
    ([`zcash_client_backend` `=0.25.0`](https://github.com/logos-co/logos-zcash-wallet-core-module/blob/e684d5fb6d8dc5165c8dc33a9c94c7bccd56dd91/rust-lib/Cargo.toml#L16)).
    It left the `0.25.0-pre.1` pre-release in [#16](https://github.com/logos-co/logos-zcash-wallet-core-module/pull/16)
    (merged 2026-10-22).
  - It [syncs over Tor](https://github.com/logos-co/logos-zcash-wallet-core-module/blob/e684d5fb6d8dc5165c8dc33a9c94c7bccd56dd91/README.md#L21-L24):
    by default through `tor_module`, or through the user's own SOCKS proxy.
  - A send [spends from one shielded pool at a time, Ironwood first](https://github.com/logos-co/logos-zcash-wallet-core-module/blob/e684d5fb6d8dc5165c8dc33a9c94c7bccd56dd91/README.md#L56-L57).
    Transparent shielding is also supported.
- **Tor:** [`logos-tor-module`](https://github.com/logos-co/logos-tor-module) (`tor_module`, created
  2026-10-27) runs Arti in-process behind a loopback SOCKS port, and
  [`logos-tor-ui`](https://github.com/logos-co/logos-tor-ui) manages it. Both are 0.1.0, untagged, and in
  no catalog. The Zcash node module and the Bitcoin node module use it by default.
- [logos-workspace#107](https://github.com/logos-co/logos-workspace/pull/107), which adds Zcash apps
  to the developer workspace, was merged on 2026-10-15. The workspace is not a catalog.
- **RFP-003 M2, the ZEC-LEZ leg** ([rfp#122](https://github.com/logos-co/rfp/issues/122)): open
  since 2026-07-20. Its one comment ([2026-10-16](https://github.com/logos-co/rfp/issues/122#issuecomment-6211332992))
  says Gateway moves its Docker stack into Logos modules before starting M2
  ([gateway-fm#115](https://github.com/gateway-fm/lez-atomic-swaps/issues/115)). The tracker says M2
  is unsubmitted.
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
- **M6:** per the tracker's 15 October update, "M6 still waits on the ZEC app flow".
- [ecosystem#254](https://github.com/logos-co/ecosystem/issues/254) still notes that the planned-scope
  page lists Zcash wallet support but the release notes don't.

## Service Discovery

**Partial.** Shipped inside `libp2p_module` 1.1.0 (catalog pin `49f7743`), not as a separate package,
and off by default.

- `libp2p_module` has `mountServiceDiscovery`, [default `false`](https://github.com/logos-co/logos-libp2p-module/blob/49f77430fbe6cc8dbb8eee2361670d7396565e5b/src/config.h#L45),
  and `disco*` methods, added in logos-libp2p-module #40 (2026-04-27) and #53 (2026-06-16).
- `delivery_module` 0.4.0 lists `libp2p_module` as an [`optional_dependency`](https://github.com/logos-co/logos-delivery-module/blob/1e1fb56aaf4bb6b80e8d1fef043ce64eba6bd28f/metadata.json#L15).
  It installs the libp2p discovery plugin [only when the node asks for one](https://github.com/logos-co/logos-delivery-module/blob/1e1fb56aaf4bb6b80e8d1fef043ce64eba6bd28f/src/delivery_module_plugin.cpp#L810-L842)
  ([parsing of `externalServiceDiscovery`](https://github.com/logos-co/logos-delivery-module/blob/1e1fb56aaf4bb6b80e8d1fef043ce64eba6bd28f/src/discovery_config.h#L99-L140)).
  Wired in `ba4c5a7` (#97, 2026-09-27), building on [logos-delivery#4178](https://github.com/logos-messaging/logos-delivery/pull/4178).
- Both presets list "Kademlia discovery" as on in
  [networks.md](https://github.com/logos-co/logos-delivery-module/blob/1e1fb56aaf4bb6b80e8d1fef043ce64eba6bd28f/docs/pages/networks.md#L34-L38),
  which is Delivery's internal discovery, not the plugin. `libp2p_module` has
  [no build for Windows](https://github.com/logos-co/logos-delivery-module/blob/1e1fb56aaf4bb6b80e8d1fef043ce64eba6bd28f/flake.nix#L162).
- [logos-delivery-module#157](https://github.com/logos-co/logos-delivery-module/issues/157) (open,
  last comment 2026-10-07) proposes using the plugin in `logos.dev`.
- The v0.4 [discovery page](https://github.com/logos-co/roadmap/blob/ca8f30ede6958e431e322b3c330cb03eed2f7ae7/content/anoncomms/roadmap/testnet_v0.4/discovery_v0.4.md)
  starts with [anoncomms-pm#89](https://github.com/logos-co/anoncomms-pm/issues/89) "Simplify the
  service discovery module API" (open, no comments).
- Still not in the catalog: [logos-libp2p-module v1.1.1](https://github.com/logos-co/logos-libp2p-module/releases/tag/v1.1.1)
  (2026-10-06, adds `kadWaitBootstrap`), which is still its latest release. Also open:
  [logos-libp2p-module#113](https://github.com/logos-co/logos-libp2p-module/pull/113) (publish
  caller-provided records) and [logos-docs#561](https://github.com/logos-co/logos-docs/issues/561), a
  "run a Delivery node with service discovery" journey.

## Mix

**Partial.** `delivery_module` 0.4.0 ships mix routing, and both network presets mount mix, but
Chat's sends do not use it.

- 0.4.0 (2026-10-14) ships [logos-delivery-module#148](https://github.com/logos-co/logos-delivery-module/pull/148),
  which reapplies mix routing with a shared RLN backend, and
  [#160](https://github.com/logos-co/logos-delivery-module/pull/160). It adds
  [`getLocalMixPeerRecord` and `addMixPeer`](https://github.com/logos-co/logos-delivery-module/blob/1e1fb56aaf4bb6b80e8d1fef043ce64eba6bd28f/src/delivery_module_plugin.h#L297-L298),
  so a host can connect its node to standalone mix intermediates.
- Mix routing is "on" for `logos.dev` and `logos.test` in
  [networks.md](https://github.com/logos-co/logos-delivery-module/blob/1e1fb56aaf4bb6b80e8d1fef043ce64eba6bd28f/docs/pages/networks.md#L34-L38)
  and in [`networks_config.nim`](https://github.com/logos-messaging/logos-delivery/blob/37b4b800acf18c7069c0de7beb61a8b91f274d10/logos_delivery/waku/factory/networks_config.nim#L89)
  ([L127](https://github.com/logos-messaging/logos-delivery/blob/37b4b800acf18c7069c0de7beb61a8b91f274d10/logos_delivery/waku/factory/networks_config.nim#L127)),
  at `37b4b80`, the logos-delivery revision 0.4.0 locks. That revision includes
  [logos-delivery#4282](https://github.com/logos-messaging/logos-delivery/pull/4282). Upstream, #4282 is
  in the pre-release `v0.40.0-rc.0` (2026-10-27) but not in `v0.39.2`, the latest release.
- **The release notes say "Chat uses RLN and mix through Delivery"; the source does not show it.**
  `chat_module` is still 0.3.0, and it creates its node with only
  [`mode`, `preset` and a log level](https://github.com/logos-co/logos-chat-module/blob/5053dd5062817f34a93055435781c1c448c7ce6c/rust-lib/src/actions.rs#L259-L263).
  It sets no `anonymityLevel`. In 0.4.0 the default is `None`, and only a higher level
  [routes sends through mix](https://github.com/logos-co/logos-delivery-module/blob/1e1fb56aaf4bb6b80e8d1fef043ce64eba6bd28f/src/delivery_module_plugin.h#L86-L90).
  The mix send processor is added only for
  [`Preferred` or `Required`](https://github.com/logos-messaging/logos-delivery/blob/37b4b800acf18c7069c0de7beb61a8b91f274d10/logos_delivery/messaging/delivery_service/send_service/send_service.nim#L117-L129).
  So Chat's sends take the plain path. Chat's `main` (`8e5b68d`, 2026-10-23) sets no level either.
  Chat's use of RLN was not checked.
- The release notes' "Mix module: Enables running mix middle nodes" is
  [logos-co/logos-libp2p-mix-rln](https://github.com/logos-co/logos-libp2p-mix-rln)
  (`libp2p_mix_rln_module` 0.1.0, created 2026-08-13, last commit still `9ebf684`, 2026-10-12).
  It is in neither the default nor the dev catalog and has no releases or tags, so it must be built
  from source. [logos-modules-release#74](https://github.com/logos-co/logos-modules-release/pull/74),
  which would add it, is still an open draft with no activity since 2026-09-29.
- v0.4: [anoncomms-pm#91](https://github.com/logos-co/anoncomms-pm/issues/91) (open) plans one Logos
  Mix module that acts as intermediate, entry or exit node; see the
  [v0.4 mix page](https://github.com/logos-co/roadmap/blob/ca8f30ede6958e431e322b3c330cb03eed2f7ae7/content/anoncomms/roadmap/testnet_v0.4/mix_v0.4.md).

## RLN Membership

**Shipped** as `liblogos_rln_module` 0.12.0 and `liblogos_lez_rln_module` 5.1.0, with validation off.

- [logos-modules-release#115](https://github.com/logos-co/logos-modules-release/pull/115) bumped them
  for LEZ v0.3.1, and both were published on 2026-10-14, built from logos-rln-modules `d9f8b3d`.
  `liblogos_lez_rln_module` 5.1.0 adds `check_registry`, which tells a caller whether a registry is
  deployed on the chain it is using. Its
  [`networks.json`](https://github.com/logos-co/logos-rln-modules/blob/d9f8b3d5ff93f6fd6fb363f5689098c07e81ac47/logos-lez-rln-module/rust-lib/networks.json#L21-L25)
  points the `logos.test` registry at deployment `testnet-v03-z3` on `testnet.lez.logos.co`.
- The release notes say "RLN proof validation: Activation is planned approximately two weeks after
  release" ([v03-release.md](https://github.com/logos-co/roadmap/blob/ca8f30ede6958e431e322b3c330cb03eed2f7ae7/content/testnets/v03-release.md),
  dated 2026-09-29). As of 2026-11-04, 36 days on, it is still off: the `logos.test` preset has
  `.enableValidation = false` in `delivery_module` 0.4.0
  ([`rln_presets.cpp`](https://github.com/logos-co/logos-delivery-module/blob/1e1fb56aaf4bb6b80e8d1fef043ce64eba6bd28f/src/rln_presets.cpp#L57))
  and on `main` at [`23d724d`](https://github.com/logos-co/logos-delivery-module/blob/23d724da0f982400acb51114f7906c04c3151d57/src/rln_presets.cpp#L57)
  (2026-10-29). The networks page agrees: "RLN | off | on — LEZ testnet registry, 600 s epochs, validation off".
- The catalog also has `delivery_module` 0.3.3, published 2026-10-26. Per
  [logos-modules-release#118](https://github.com/logos-co/logos-modules-release/pull/118) it is the
  same code as 0.4.0 under a lower version, for the `logos.test` fleet. That PR planned to unpublish
  0.4.0 afterwards, but 0.4.0 is still in the index.
- v0.4: [anoncomms-pm#95](https://github.com/logos-co/anoncomms-pm/issues/95) (a standalone
  membership allocation service module) and [#96](https://github.com/logos-co/anoncomms-pm/issues/96)
  (on-chain memberships and accounting compatible with Logos zones and LEZ), both open, in the
  [v0.4 RLN page](https://github.com/logos-co/roadmap/blob/ca8f30ede6958e431e322b3c330cb03eed2f7ae7/content/anoncomms/roadmap/testnet_v0.4/rln_v0.4.md).

## Token (LEZ program)

**Shipped, for the legacy LEZ line.** `token_module` 0.1.0 and `token_ui` 0.1.0 were published on
2026-10-30. They target the legacy v0.2.4 testnet, not the reset v0.3 one, and their `lez_core`
requirement clashes with the LEZ wallet's.

- **Packaging.** [`logos-token-module`](https://github.com/logos-blockchain/logos-token-module) and
  [`logos-token-ui-module`](https://github.com/logos-blockchain/logos-token-ui-module), empty on
  2026-10-14, are now thin wrappers. Each [`flake.nix`](https://github.com/logos-blockchain/logos-token-module/blob/2afe8a796a67b1a5ce807851b3f74660114e1ee2/flake.nix#L19)
  re-exports lez-programs at `543563a`, the merge of lez-programs#476 on branch `lez-v0.2.x`
  (2026-10-29). [logos-modules-release#126](https://github.com/logos-co/logos-modules-release/pull/126)
  released them together with the DEX and Oracle wrappers. The module's API (16 methods: reads,
  fungible and non-fungible creation, transfer, mint, burn, authorities) is on its
  [card](modules/token_module/README.md).
- **The LEZ split.**
  - [`token_module`](https://github.com/logos-blockchain/logos-token-module/blob/2afe8a796a67b1a5ce807851b3f74660114e1ee2/metadata.json#L9),
    [`token_ui`](https://github.com/logos-blockchain/logos-token-ui-module/blob/255f0cac9e530dc0e1587f83a4a2d57185b5f5eb/metadata.json#L11),
    [`oracle_module`](https://github.com/logos-blockchain/logos-twap-oracle-module/blob/a964dc4d32780adec3e6fa63ee549f151d3871b2/metadata.json#L9-L11)
    and [`amm_module`](https://github.com/logos-blockchain/logos-amm-module/blob/0efc26a6012e86f252c1b99a4964aac4a96d96c9/metadata.json#L11)
    (and `amm_ui`) declare `lez_core` version `"0.4.3"`, which allows only that version.
  - `lez_core` 0.4.3 (published 2026-10-16) builds LEZ at
    [`80a648a`](https://github.com/logos-blockchain/logos-execution-zone-module/blob/e02937cc9248dabc684c3b1d7daaa8431be27bff/flake.nix#L8-L9),
    the head of LEZ's `release/lez-legacy` branch and 12 commits past
    [v0.2.4-lez-legacy](https://github.com/logos-blockchain/logos-execution-zone/releases/tag/v0.2.4-lez-legacy)
    (2026-10-13). `lez_core` 0.5.1 builds
    [LEZ v0.3.1](https://github.com/logos-blockchain/logos-execution-zone-module/blob/51a6fee385eedfdc65ba71f01447c5411a54bdf7/flake.nix#L14-L15).
  - [`lez_wallet_ui` 1.2.2](https://github.com/logos-blockchain/logos-execution-zone-wallet-ui/blob/486379ec41ea0bd4e59cf3c82632429b0fed393b/metadata.json#L12)
    declares `lez_core` `>=0.5.1`, and the RLN modules moved to LEZ v0.3.1 ([§ RLN Membership](#rln-membership)).
    The catalog offers `lez_core` 0.4.0 to 0.5.1, latest 0.5.1.
  - liblogos's [dependency gate](https://github.com/logos-co/logos-liblogos/blob/00f19ef8e2ffd54e2cf97616a960cd19232ad4ba/src/logos_core/dependency_gate.h#L8-L24)
    refuses to load a module whose installed dependency is outside its declared range
    ([guides/compatibility.md](guides/compatibility.md), rule 7). With one `lez_core` installed,
    either the Token, DEX and Oracle group or the LEZ wallet should then be refused. **This is
    inferred from the gate's rules, not tested in Basecamp.**
- **Testnets.** [lez-programs#423](https://github.com/logos-blockchain/lez-programs/pull/423) (merged
  2026-10-19) says `testnet.lez.logos.co` now runs LEZ v0.3.0, so the v0.2.4 programs were redeployed
  to `testnet-legacy.lez.logos.co`. DEPLOYMENTS.md, the same at
  [`543563a`](https://github.com/logos-blockchain/lez-programs/blob/543563af88b5638e308a981106fd821690227011/DEPLOYMENTS.md)
  and on [`main`](https://github.com/logos-blockchain/lez-programs/blob/6674d7833a8f5d618126d992eed5cfe21bd86ba4/DEPLOYMENTS.md)
  (`6674d78`), now lists `token`, `amm`, `twap_oracle` and `token_mint_authority` only under
  "Testnet legacy (v0.2.4)". No deployment on the reset testnet is recorded.
  - The RFP-019 review on 2026-10-19 ([rfp#215](https://github.com/logos-co/rfp/issues/215#issuecomment-6262951133))
    found nothing from DEPLOYMENTS.md live on the legacy testnet that day. Whether the programs are
    live there now is **unverified**: this check did not query the chain.
- **`main` is a separate line.** lez-programs `main` builds against LEZ v0.2.5-rc2
  ([#385](https://github.com/logos-blockchain/lez-programs/pull/385), a PR the last check called an
  open issue, merged 2026-10-20), and its module metadata declares `lez_core` with no range.
  [#478](https://github.com/logos-blockchain/lez-programs/pull/478) (open) ports the packaging fixes to
  `main`.
- `stablecoin_module` 0.1.0 is still in lez-programs (at `543563a` and on `main`) and in no catalog.

## Oracle

**Module shipped, no app.** `oracle_module` 0.1.0 (2026-10-30) wraps the `twap_oracle` program from
lez-programs `lez-v0.2.x` @ `543563a`, through
[`logos-twap-oracle-module`](https://github.com/logos-blockchain/logos-twap-oracle-module) (created
2026-10-21). Like Token, it needs the legacy `lez_core` 0.4.3 and targets the legacy testnet
([§ Token](#token-lez-program)).

- It exposes [nine methods](https://github.com/logos-blockchain/lez-programs/blob/543563af88b5638e308a981106fd821690227011/modules/oracle/src/oracle_module_impl.h#L22-L57):
  program and feed lookups, creating the tick, observation and price accounts, `updateCurrentTick`,
  `recordTick` and `publishPrice`. The price-feed dashboard app that RFP-019 asks for does not exist.
- **RFP-019 M1** ([rfp#215](https://github.com/logos-co/rfp/issues/215), "LEZ testnet v0.2.4 target")
  is open and not accepted. Its second deliverable is a "Logos module wrapping the program".
  - The [2026-10-19 review](https://github.com/logos-co/rfp/issues/215#issuecomment-6262951133)
    requested changes, including M1-019-06: no Logos module wrapped the oracle.
  - A [2026-10-23 re-review](https://github.com/logos-co/rfp/issues/215#issuecomment-6361006305),
    against a local v0.2.5-rc2 chain, found M1-019-01 to -04 fixed. It still listed the module as
    partial, along with M1-019-05 (permissionless recording) and -07 (no decimals field).
  - On [2026-11-04](https://github.com/logos-co/rfp/issues/215#issuecomment-6629925757) the team
    replied that the module is implemented for the legacy testnet on `lez-v0.2.x`, the branch
    `oracle_module` 0.1.0 is built from, and argued that -05 and -07 are by design. No reviewer has answered.
    **Whether `oracle_module` 0.1.0 meets the deliverable is therefore not settled.**
  - lez-programs tracks the module's remaining work in
    [#436](https://github.com/logos-blockchain/lez-programs/issues/436) to
    [#444](https://github.com/logos-blockchain/lez-programs/issues/444) (opened 2026-10-21, all open),
    and further oracle fixes in [#481](https://github.com/logos-blockchain/lez-programs/issues/481)
    to [#484](https://github.com/logos-blockchain/lez-programs/issues/484) (opened 2026-11-02).
- RFP-019 M2 ([rfp#216](https://github.com/logos-co/rfp/issues/216), "LEZ testnet v0.4 target") is
  open with no comments. Tracker: [ecosystem#188](https://github.com/logos-co/ecosystem/issues/188)
  (no comments).
- [RFP-019](https://github.com/logos-co/rfp/blob/master/RFPs/RFP-019-twap-oracle.md) (TWAP) and
  [RFP-020](https://github.com/logos-co/rfp/blob/master/RFPs/RFP-020-redstone-oracle-adaptor.md)
  (RedStone) are still `status: closed` to proposals. RFP-020 is unchanged: M0
  ([rfp#146](https://github.com/logos-co/rfp/issues/146)) closed 2026-10-09, and M1 to M5
  ([#147](https://github.com/logos-co/rfp/issues/147) to [#151](https://github.com/logos-co/rfp/issues/151))
  are open with no comments.
- The AnonComms [Logos Oracle Network](https://github.com/logos-co/logos_oracle_network) still has no
  Logos module. In [logos_oracle_network#8](https://github.com/logos-co/logos_oracle_network/issues/8)
  "Few blockers on being a module" (open), the maintainers agreed on 2026-10-23 to wait for the next
  version of Logos modules rather than build on the current one.
- The v0.4 [oracle page](https://github.com/logos-co/roadmap/blob/ca8f30ede6958e431e322b3c330cb03eed2f7ae7/content/anoncomms/roadmap/testnet_v0.4/oracle_v0.4.md)
  tracks [anoncomms-pm#92](https://github.com/logos-co/anoncomms-pm/issues/92) to
  [#94](https://github.com/logos-co/anoncomms-pm/issues/94) in
  [milestone 29](https://github.com/logos-co/anoncomms-pm/milestone/29), due 2026-11-23. All three
  are open, with no comments.

## How this was checked

Every "none found" above comes from these searches. The 2026-11-04 re-run covers 2026-10-14 to 2026-11-04:

- **Orgs:** logos-co (287 repos), logos-blockchain (40), logos-storage (119), vacp2p (53), status-im
  (851) and logos-messaging (92), as `gh repo list` returns them: 1,442 in all, up from 1,420.
  `gh repo list` covered all of them, grepped for
  btc|bitcoin|zec|zcash|zebra|orchard|sapling|bdk|electrum|wallet|signer|keycard|swap|mix|discover|oracle|twap|token|stablecoin|tor.
  New since 2026-10-14 and relevant here: `logos-twap-oracle-module`, the QR signer repos,
  `logos-account-kit` and the two Tor repos.
- **`gh search repos`:** bitcoin, btc, zcash and zec in each org. On all of GitHub: "lez atomic
  swap" and logos/lez/basecamp × bitcoin/zcash, updated since 2026-10-14. The only hits are the
  Gateway repo and the `logos-co` wallet repos above.
- **`gh search issues --include-prs`:** bitcoin and zcash in all six orgs, updated since 2026-10-14.
  The hits are the repos and issues named above. All issue and PR states, comments and milestones
  cited here were re-read on 2026-11-04.
- **`gh search code`:** bitcoin and zcash `filename:metadata.json` in logos-co and logos-blockchain;
  `stablecoin_module` in logos-co (no hits).
- **Third-party catalogs:** `gh search code` for `logos-repo.json` found 22 files in 21 repos. This
  pass read 19 of their indexes, plus the `mandrigin` index directly. `logos-ecodev-catalog` has no
  published index, and `logos-modules-release-base` is a template. None lists a Zcash, oracle or LEZ
  token-program package; one (`hackyguru/logos-modules`) has a shop app with its own LEZ token. The
  only Bitcoin packages are the two swap desks. `vpavlin`'s catalogs list a `keycard` smartcard-login
  package, not the signer stack's `keycard_module`. `atomic-swaps-poc`'s `swap` and `swap_ui` are
  ETH↔LEZ.
- **Also read:** the default and dev catalog indexes (2026-11-04), `release-set.json` at `6f19129`,
  lez-programs `lez-v0.2.x` and `main` (DEPLOYMENTS.md and module metadata), the `lez_core` 0.4.3
  and 0.5.1 sources the catalog pins, LEZ releases (latest
  [v0.3.2](https://github.com/logos-blockchain/logos-execution-zone/releases/tag/v0.3.2) and
  `v0.4.0-rc1`, both 2026-10-29), and the roadmap's v0.3 pages and v0.4 deliverable pages. There is
  still no v0.4 testnet page.
- **Code claims:** for repos without releases, the links above are permalinks at the commit `HEAD`
  pointed to on 2026-11-04. Source reads for Delivery, Chat, RLN, LEZ, Token, Oracle and DEX use the
  commits their catalog packages pin.
- **Roadmap revision read:** `logos-co/roadmap` branch `v5` (`ca8f30e`).
