# Roadmap vs. what ships in Basecamp 0.3.1

Checked **2026-10-01**. This compares the v0.3 testnet scope in
[`logos-co/roadmap` `content/testnets/v03.md`](https://github.com/logos-co/roadmap/blob/v5/content/testnets/v03.md)
(branch `v5` @ `668bf39`) with Basecamp 0.3.1: its 6 bundled modules (`package_manager`,
`package_downloader`, `capability_module`, `modules_state`, `package_manager_ui`, `storage_module`)
and the 46 packages in [`logos-co/logos-modules-release`](https://github.com/logos-co/logos-modules-release)
(`data/catalog-index.snapshot.json`, 2026-10-01T12:42Z). The
[v0.3 release notes](https://github.com/logos-co/roadmap/blob/v5/content/testnets/v03-release.md)
(2026-09-29) are cited where they differ. None of these, the
[dev catalog](https://github.com/logos-co/logos-modules-dev) (11 packages, 2026-10-01T12:43Z) or
[`logos-release-set`](https://github.com/logos-co/logos-release-set) (which lists no wallets)
has a Bitcoin or Zcash entry.

## Summary

| Roadmap item | Status in 0.3.1 catalog | Where the work lives | Evidence |
|---|---|---|---|
| Chat, Delivery, LEZ, LEZ Indexer, Blockchain, Storage (modules) | Shipped | `chat_module` 0.3.0, `delivery_module` 0.3.0, `lez_core` 0.5.0 + `logos_execution_zone` 1.0.0, `lez_indexer_module` 1.2.0, `blockchain_module` 0.3.0, `storage_module` 3.0.0 | Catalog snapshot |
| RLN Membership | Shipped | `liblogos_rln_module` 0.10.0, `liblogos_lez_rln_module` 4.2.1 | Catalog. Release notes: proof validation turns on ~2 weeks after release |
| Service Discovery | **Partial**: inside `libp2p_module`, opt-in, not a standalone package | [logos-libp2p-module](https://github.com/logos-co/logos-libp2p-module), [logos-delivery-module](https://github.com/logos-co/logos-delivery-module) | [§ Service Discovery](#service-discovery) |
| Mix | **Partial**: mix routing in Delivery. The standalone mix-node module is in no catalog | [logos-libp2p-mix-rln](https://github.com/logos-co/logos-libp2p-mix-rln) | [§ Mix](#mix) |
| Wallet: Ethereum | Shipped as its own app and stack | `eth_wallet_ui`, `eth_wallet_backend`, plus 17 other EVM packages (keystore, signer, RPC, verified proxy, assets, fees, tx sender, token lists, Uniswap) | Catalog |
| Wallet: Monero | Shipped as its own app and stack | `monero_wallet_ui`, `monero_wallet_backend`, `monero_wallet_core_module`, `monero_wallet_cli`, `monero_node_module`, `monerod_module`, `monerod_ui` (all 0.1.0) | Catalog. [stacks/monero-wallet.md](stacks/monero-wallet.md) |
| Wallet: **Bitcoin** | **Absent.** No wallet exists anywhere. A BTC↔LEZ *swap* app is in a third-party catalog | [gateway-fm/lez-atomic-swaps](https://github.com/gateway-fm/lez-atomic-swaps) (RFP-003) | [§ Bitcoin](#bitcoin) |
| Wallet: **Zcash** | **Absent.** Nothing installable anywhere | RFP-003 M2, not delivered | [§ Zcash](#zcash) |
| One "Wallet" module/app covering all four chains | **Absent.** ETH and XMR ship as two separate apps | — | Release notes list "Monero wallet" and "EVM wallets" separately |
| LEZ program: Token | **On-chain only**: no Basecamp package in any catalog | [lez-programs](https://github.com/logos-blockchain/lez-programs) | [§ Token](#token-lez-program) |
| LEZ program: DEX / App: DEX | Shipped | `amm_module` 0.1.0, `amm_ui` 0.1.0 ("Logos DEX App") | Catalog. The `amm` program is deployed on testnet |
| LEZ program: Oracle | **On-chain only**: no module or app | lez-programs `twap_oracle`, [logos_oracle_network](https://github.com/logos-co/logos_oracle_network) | [§ Oracle](#oracle) |
| Apps: Blockchain Dashboard, LEZ Wallet, LEZ Explorer, Chat, Storage | Shipped | `blockchain_ui` 0.3.0, `lez_wallet_ui` 1.2.0, `lez_explorer_ui` 1.2.0, `chat_ui` 0.3.0, `storage_ui` 3.0.0 | Catalog. Release notes: Chat uses mix through Delivery |

## Bitcoin

**Not in the release.** There is no Bitcoin wallet module or app in any Logos-org repo or any
catalog checked. What exists is atomic-swap work:

- **[gateway-fm/lez-atomic-swaps](https://github.com/gateway-fm/lez-atomic-swaps)** (created
  2026-08-20; latest release [v0.2.5](https://github.com/gateway-fm/lez-atomic-swaps/releases/tag/v0.2.5),
  2026-09-28) is the RFP-003 grantee Gateway's repo
  ([RFP](https://github.com/logos-co/rfp/blob/master/RFPs/RFP-003-atomic-swaps.md); proposal
  [rfp#112](https://github.com/logos-co/rfp/issues/112), accepted 2026-07-10).
  It does native BTC↔LEZ swaps with Taproot/MuSig2 adaptor signatures (`btc-swap-sdk`,
  `btc-core-adapter`, …). Its two `ui_qml` swap desks, `lez_atomic_swap_maker` and
  `lez_atomic_swap_taker` ("LEZ / BTC Maker/Taker", 0.2.5), are published only in the third-party
  catalog [mandrigin/logos-modules-release-base](https://github.com/mandrigin/logos-modules-release-base).
  The desks depend on `chat_module` and `delivery_module`, and drive Maker/Taker Node services and
  a Bitcoin Core node that run outside Basecamp. Per its README, public-network operation,
  production key management and an audit are "still in progress".
- **Milestones** (tracker [ecosystem#182](https://github.com/logos-co/ecosystem/issues/182)):
  M1 design ([rfp#121](https://github.com/logos-co/rfp/issues/121)) accepted 2026-09-30.
  M3 BTC leg ([rfp#123](https://github.com/logos-co/rfp/issues/123)) is "ready to accept once
  three items are closed" (2026-09-30). M6 GUIs ([rfp#126](https://github.com/logos-co/rfp/issues/126))
  are being reviewed for the Bitcoin part only. M7 audit ([rfp#127](https://github.com/logos-co/rfp/issues/127))
  is open.
- **Related:**
  - [ecosystem#206](https://github.com/logos-co/ecosystem/issues/206) "[RFP] Wrapped Bitcoin":
    open since 2026-08-11, one-line placeholder.
  - Draft maker-operator L-Prizes: [lambda-prize#165](https://github.com/logos-co/lambda-prize/pull/165)
    and [#166](https://github.com/logos-co/lambda-prize/pull/166) (open, 2026-09-25).
  - LP-0018 BTC-LEZ swap ([lambda-prize#28](https://github.com/logos-co/lambda-prize/pull/28)):
    closed unmerged 2026-08-05.
  - [logos-co/atomic-swaps-poc](https://github.com/logos-co/atomic-swaps-poc) (formerly
    `eth-lez-atomic-swaps`): ETH↔LEZ only. No BTC/ZEC in its 92 branches or its source.
  - Adjacent: [corpetty/muster](https://github.com/corpetty/muster) (personal repo), a multisig
    coordination client. Its wallet layer has a Bitcoin Core adapter. It is not in the default or dev catalog.
- **None found:** BTC/UTXO branches in 13 wallet repos (eth-wallet, monero-wallet, evm-keystore,
  wallet, accounts, LEZ-wallet, status-im/go-wallet-sdk, whose `pkg/` is EVM only), or a Bitcoin
  `metadata.json` in logos-co or logos-blockchain. The "BDK" hits ([ecosystem#236](https://github.com/logos-co/ecosystem/issues/236),
  [#238](https://github.com/logos-co/ecosystem/issues/238)) use the Bitcoin Development Kit only
  as a design model for the LEZ SDK.

## Zcash

**Not in the release.** No Zcash wallet exists in any form, and nothing Zcash-related is
installable from any catalog.

- **RFP-003 M2, the ZEC-LEZ leg** ([rfp#122](https://github.com/logos-co/rfp/issues/122)): open
  since 2026-07-20, with no comments or submission.
  - Scope: BIP-199 HTLCs on transparent `t1…` addresses, using `librustzcash`/`zcash_primitives`.
  - Shielded swaps (Sapling/Orchard) are out of scope; see
    [appendix/zcash-atomic-swap-primitives.md](https://github.com/logos-co/rfp/blob/master/appendix/zcash-atomic-swap-primitives.md).
- **Code in gateway-fm/lez-atomic-swaps:** `zec-swap-sdk`, `zec-reference-actor` and
  `zebra-node-adapter` sit behind the `pair-zec` feature, which is off by default.
  [docs/m6-zec-reconciliation.md](https://github.com/gateway-fm/lez-atomic-swaps/blob/main/docs/m6-zec-reconciliation.md)
  ([#85](https://github.com/gateway-fm/lez-atomic-swaps/pull/85), merged 2026-09-18) records:
  - ZEC was proven only at the service layer, against Zebra Regtest, on 2026-08-04.
  - ZEC was removed from the desks on 2026-08-26, and the shipped Nodes are built without it.
  - There has "never been one run from a desk click to a ZEC chain effect".
- **M6:** in [rfp#126](https://github.com/logos-co/rfp/issues/126), the ZEC app flow and the
  shield-after-swap guidance stay open until M2 is delivered (2026-09-25).
- **None found:** a Zcash wallet module or app, or a ZEC branch in the 13 wallet repos.
  Searches for zcash, zec, librustzcash, orchard, sapling, zebrad, zingo and lightwalletd returned
  only the RFP-003 material above, research repos, and unrelated threads from 2023.

## Service Discovery

**Shipped inside `libp2p_module` 1.1.0** (catalog pin `49f7743`), not as a separate package.

- `libp2p_module` has a `mountServiceDiscovery` flag (default `false`) and `disco*` methods,
  added in logos-libp2p-module #40 (2026-04-27) and #53 (2026-06-16).
- `delivery_module` 0.3.0 lists `libp2p_module` as an `optional_dependency` and uses it for
  discovery only when `pluginKadDiscovery` is set. It was wired in `ba4c5a7` (#97, 2026-09-27),
  building on [logos-delivery#4178](https://github.com/logos-messaging/logos-delivery/pull/4178)
  (merged 2026-09-22).
- Otherwise, and on Windows (where `libp2p_module` has no build), Delivery uses its internal discovery.
- Still open: [anoncomms-pm#64](https://github.com/logos-co/anoncomms-pm/issues/64) and
  [#36](https://github.com/logos-co/anoncomms-pm/issues/36).

## Mix

**Partial.** Delivery has mix routing on in both network presets, and Chat uses mix through Delivery.

- The release notes' "Mix module: Enables running mix middle nodes" is
  [logos-co/logos-libp2p-mix-rln](https://github.com/logos-co/logos-libp2p-mix-rln)
  (`libp2p_mix_rln_module` 0.1.0, created 2026-08-13, last commit 2026-10-01).
- It is in neither the default nor the dev catalog and has no releases, so it must be built from source.
- Integration PRs are open: [logos-delivery#4282](https://github.com/logos-messaging/logos-delivery/pull/4282)
  (2026-09-18) and [logos-delivery-module#148](https://github.com/logos-co/logos-delivery-module/pull/148)
  (2026-09-30).

## Token (LEZ program)

**Deployed on chain; no Basecamp package.**

- The `token` program is live on LEZ testnet v0.2.4
  ([lez-programs DEPLOYMENTS.md](https://github.com/logos-blockchain/lez-programs/blob/main/DEPLOYMENTS.md),
  2026-09-10).
- lez-programs also contains `token_module` 0.1.0 and `token_ui` 0.1.0, but neither is in any
  catalog.
- `logos-blockchain/logos-token-module` and `logos-token-ui-module` are empty repos (2026-09-18).
- For comparison, the DEX does ship, although lez-programs has a newer `amm_module`/`amm_ui`
  (0.3.0) than the catalog (0.1.0).

## Oracle

**Deployed on chain; no module or app.**

- A `twap_oracle` program is deployed on LEZ testnet (same DEPLOYMENTS.md).
- [RFP-019](https://github.com/logos-co/rfp/blob/master/RFPs/RFP-019-twap-oracle.md) (TWAP) and
  [RFP-020](https://github.com/logos-co/rfp/blob/master/RFPs/RFP-020-redstone-oracle-adaptor.md)
  (RedStone) are closed to proposals. RFP-020 M2 ([rfp#148](https://github.com/logos-co/rfp/issues/148))
  is open.
- The AnonComms [Logos Oracle Network](https://github.com/logos-co/logos_oracle_network)
  (2026-07-24) has no Logos module. Its tracking issue,
  [anoncomms-pm#57](https://github.com/logos-co/anoncomms-pm/issues/57), is open.

## How this was checked

Every "none found" above comes from these searches:

- **Orgs:** logos-co, logos-blockchain, logos-storage, vacp2p, status-im and logos-messaging.
  `gh repo list` covered all 1,382 of their repos, grepped for
  btc|bitcoin|zec|zcash|orchard|sapling|bdk|electrum|wallet|swap|mix|discover|oracle|token.
- **`gh search repos`:** bitcoin, btc, zcash and zec in each org. On all of GitHub: "lez atomic
  swap" and logos/lez/basecamp × bitcoin/zcash, also `in:readme`.
- **`gh search issues --include-prs`:** bitcoin and zcash in all six orgs. btc, zec, sapling,
  orchard, zebrad, zingo, lightwalletd, bitcoind, librustzcash, bdk, electrum, "bitcoin wallet" and
  "zcash wallet" in logos-co and/or logos-blockchain.
- **`gh search code`:** zcash, librustzcash, bdk_wallet, zebra and electrum in logos-co; zcash and
  bitcoin `filename:metadata.json` in logos-co and logos-blockchain.
- **Also read:** the dev catalog index, `release-set.json`, the gateway catalog index, and the
  roadmap's v0.3 pages and weekly updates (to 2026-09-28).
- **Roadmap revision read:** `logos-co/roadmap` branch `v5` (668bf39).
