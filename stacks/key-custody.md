# Key custody across chains

Pinned at Basecamp **0.3.1**, catalog index 2026-10-01. This page summarizes the
evidence in [evm-wallet.md](evm-wallet.md), [monero-wallet.md](monero-wallet.md) and
[blockchain-lez.md](blockchain-lez.md). Those pages carry the permalinks, so follow
them before relying on a detail.

Read this before using the official wallet modules (EVM, Monero, LEZ) as the key-management backend of
another module (the motivating case is Muster).

## What exists

| Chain | Key holder (the only module with secrets) | Coordinator a caller should use | Human approval surface | Headless approval | Ships in 0.3.1 |
|---|---|---|---|---|---|
| Ethereum / EVM | [`keystore_module`](../modules/keystore_module/README.md): scrypt vaults, BIP39/32 HD, secp256k1 | [`tx_sender_module`](../modules/tx_sender_module/README.md) for transactions; `keystore_module.request_approval` for messages, EIP-712 and digests | [`evm_signer_ui`](../modules/evm_signer_ui/README.md) (intent `evm.signing.approve`) | [`evm_signer_cli`](../modules/evm_signer_cli/README.md) + an operator | catalog |
| Monero | [`monero_wallet_core_module`](../modules/monero_wallet_core_module/README.md): wallet2 via monero_c | [`monero_wallet_backend`](../modules/monero_wallet_backend/README.md) (`prepare_send` → `confirm_send`) | [`monero_wallet_ui`](../modules/monero_wallet_ui/README.md) (default approver; no "review this send" intent) | [`monero_wallet_cli`](../modules/monero_wallet_cli/README.md) | catalog |
| LEZ | [`lez_core`](../modules/lez_core/README.md): BIP39 key tree, **stored unencrypted** (`password` ignored upstream) | `lez_core` directly. One wallet per process, shared by every caller | **none**: no consent step; every spending method is ungated | n/a | catalog |
| Bitcoin | none | — | — | — | **no**, see [gaps.md](../gaps.md) |
| Zcash | none | — | — | — | **no**, see [gaps.md](../gaps.md) |

## The shared model (EVM and Monero)

Both wallet stacks were built to the same pattern. Learn it once.

1. **One module holds secrets, and nothing hands them out.** The EVM keystore never
   returns a private key or an xpub. The Monero core never returns the spend key, and
   it reveals the seed or view key only on a password re-check.
2. **Roles, not ACLs.** A coordinator gates its methods by role:
   - a *custodian* creates, imports, exports and deletes;
   - an *approver* answers pending requests;
   - any *named module* may ask;
   - reads are open to anyone.

   The default holders are the official UIs. `configure` sets the roles. It is
   **total** (a role you don't name is held by nobody), **ungated** (upstream calls it
   "deferred, not refuted") and **in memory only**, so roles revert to the defaults
   whenever the module restarts. Monero can also read a persistent `roles.json`.
3. **The runtime attests the caller, not the arguments.**
   `logos_rust_sdk::current_caller()` gives one of
   `Unknown | HostAnchor | Module{name} | Derived{parent,leaf} | Operator{name}`.
   Only a plain `Module{name}` can request or hold a role. `HostAnchor` (`logosctl`,
   the shell, `core`) is refused on purpose, so you **cannot approve from `logosctl call`
   directly**; you go through the `*_cli` relay modules. Both coordinators expose
   `caller_identity()`. Call it from your own module first to check what you
   authenticate as.
4. **Asynchronous, with a human in the loop.** A request returns a handle (and on EVM
   a one-time receipt) immediately, never a result. You poll a status method until it
   reports final. Events exist but don't cover every terminal state, so poll anyway.
5. **Intents, not module calls, put a human in front of a request.** Only `ui_qml`
   code can raise intents (`logos.request(...)`). A core module has to pass the handle
   up to its own QML. Declare intents in the UI's `metadata.json` as **objects**
   (`"uses": [{"intent": "evm.signing.approve"}]`). A bare string array is silently
   ignored and every request fails with `not_declared`.

## Where they differ

| | EVM | Monero |
|---|---|---|
| When signing happens | Only inside `approve()`, after a human types the vault password into the signer | When the preview is **built**. Review gates the broadcast, not the signature |
| Who sees the password | Only the keystore, typed into `evm_signer_ui` | Passed as a plain argument UI → backend → core. Anyone acting as custodian handles it |
| Can a third party trigger a human review? | Yes: `evm.signing.approve {handle}` opens the signer | **No.** The GUI follows only the sends it started. A third party's `prepare_send` sits unseen until it expires (120 s) and blocks the single global send slot |
| Can a third party create accounts? | No (custodian only, no xpub). Hand off with `evm.accounts.manage` | No (custodian only). Unlock hand-off: `monero.wallet.unlock {wallet}` |
| Fresh receive address per request | Not without the custodian role | Yes: `create_subaddress` is ungated |
| Incoming-payment signal | Not in scope (no indexer; `tx_sender` history covers only its own sends) | No event. Poll `history()` and match the subaddress |
| Concurrency limits | 4 live approvals per requester, shared by every app sending through `tx_sender_module` | One open wallet per instance and one send in flight globally |
| Expiry | 60 s if no signer claims it; no deadline once claimed | Preview expires after 120 s |

## LEZ is different: no custody boundary at all

`lez_core` has none of the shared model above. There are no roles, no approval and
no per-caller scope. Any module that can reach it can spend from the shared wallet
(`transfer_*`, `send_generic_*`, `bridge_withdraw`), and even
`--access-policy enforce` only requires that the module *declares* `lez_core`. Keys
sit in a plain-JSON `storage.json`. There is one open wallet per process, and it may
be the user's LEZ Wallet UI wallet. So **probe before you open**: if
`get_sequencer_addr()` or `list_accounts()` is non-empty, a wallet is already open.
An "`open()` failed, so `create_new()`" fallback silently operates on someone
else's wallet. Proving calls take minutes, so use `lp_invoke_async` rather than the
20 s default timeout. Details and the full checklist:
[blockchain-lez.md → Using `lez_core` from another module](blockchain-lez.md#using-lez_core-from-another-module).

## Security posture you are inheriting

The role gates separate the **honest** surfaces. They are **not a boundary against
other loaded modules**:

- Basecamp 0.3.1 runs with `--access-policy` **off by default**, so any loaded module
  can call any other. `--access-policy enforce` limits each module to its declared
  `metadata.json` dependencies. It is off by default because `ui_qml` callers are not
  yet in the derived allow-list (Basecamp README, "Inter-module access enforcement").
- Any module can `configure` itself into a role on either stack.
- **Monero:** while a wallet is open, the core's `startJob("create_transaction")` /
  `startJob("commit_transaction", {txHandle})` take no password and no approval.
  `txHandle`s are sequential (`tx1`, `tx2`, …). `monero_node_module.get_node_config`
  returns the daemon RPC password unredacted, and `set_node_config` bypasses the
  backend's custodian gate.
- **EVM:** `eth_rpc_module` has no caller gating: it can repoint endpoints, turn off
  verified-proxy mode, and broadcast raw transactions. Even so, signing still needs the
  vault password, typed by a human into the signer.
- The `*_cli` relays (`evm_keystore_cli`, `evm_signer_cli`, `monero_wallet_cli`) don't
  check their own callers. Once an operator enrols one in a role, anyone can drive it.

If Muster becomes the place people approve multi-party transactions, design for this:
assume a co-resident module can see and cancel requests, and never let a status reply
stand in for the user's own confirmation.

## Recipes for a third-party module (Muster)

- **EVM transaction:** `tx_sender_module.prepare` → `send` gives `{requestId, handle}`.
  `muster_ui` raises `evm.signing.approve {handle}`, then you poll
  `send_status(requestId)` until `final`. **The poll is what broadcasts.** Never sign
  `tx` legs yourself; that bypasses the device's only nonce ledger.
- **EVM message / EIP-712 / digest:** `keystore_module.request_approval({address, purpose, legs})`
  gives `{handle, receipt}`. Raise `evm.signing.approve`, then
  `approval_status` → `fetch_result` → `ack_result`. The receipt is returned **once**,
  so keep it in memory and never log it. `personal_sign` takes printable text only
  (≤ 8 KiB).
- **Monero payment:** either Muster becomes approver (`configure`, or ship a
  `roles.json`) and draws its own review screen, or a human uses `monero_wallet_cli`.
  To keep Muster away from the password, have `muster_ui` raise
  `monero.wallet.unlock {wallet}` so the user unlocks in `monero_wallet_ui`.
- **Monero receive:** `create_subaddress` per request, then poll `history()` and match
  on `account` + `subaddrIndex`.
- **Before any of this:** check `caller_identity()` from the Nim module. Whether
  `lp_client_create(…)` from Nim yields `Module{muster_module}` is **unverified**. Then
  check [guides/compatibility.md](../guides/compatibility.md) for the SDK generation
  Muster must be built against to load next to these modules.
