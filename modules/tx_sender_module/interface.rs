// Extracted by logos-module-atlas from logos-co/logos-evm-tx-sender-module@7cd2fead60a7:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-evm-tx-sender-module/blob/7cd2fead60a78ac9ac8a4337fd9f01e1ab5515e2/rust-lib/src/glue.rs

pub trait TxSenderModule: Send + Sync + 'static {
    /// Price a bundle of calls without doing anything: one `fee_module` bundle estimate,
    /// the account's ether against the value plus the fee ceiling, and the nonce the first
    /// call would take. Reserves nothing and requests no approval, so it is safe on every
    /// keystroke.
    ///
    /// `request_json`: `{ chainId, from, calls: [{ to, value?, data?, gasLimit?, label?,
    /// meta? }], tier?, maxFeePerGas?, maxPriorityFeePerGas?, nonce?, deadlineMs? }`.
    /// `value` is wei, `data` is `0x`-hex calldata (absent for a plain transfer). A call with
    /// no `gasLimit` is estimated through `fee_module` as the chain will find it — an ERC-20
    /// approve in an earlier call is applied to the calls after it — and one the estimator
    /// cannot model must carry its own limit. `nonce` pins a
    /// single call onto a number to REPLACE a transaction that already left. `deadlineMs`
    /// shrinks this method's own allowance to what the caller will wait.
    ///
    /// Returns `{ ok, chainId, from, nonce, legs: [{ to, value, data, gasLimit, gasSource,
    /// label }], maxFeePerGas, maxPriorityFeePerGas, feeSource, feeCeilingWei(+Display/Exact),
    /// maxCostWei(+Display/Exact), assumptions, nativeSymbol?, replaces?, route, feeRoute }`.
    /// `feeCeilingWei` is the sum of every call's `maxFeePerGas × gasLimit` as `fee_module`
    /// answered it — a ceiling, never a price. `replaces` is `{ nonce, raised,
    /// pendingMaxFeePerGas, pendingMaxPriorityFeePerGas }` when a pinned nonce outbids a
    /// transaction still pending there.
    fn prepare(&self, request_json: String) -> String;

    /// Ask a human to approve a bundle. The same `request_json` as `prepare`, plus `purpose`:
    /// one line the signer shows as the requester's CLAIM, suffixed with the module that
    /// asked as the runtime attested it.
    ///
    /// Reserves one nonce per call under the one ledger, asks `keystore_module` for ONE
    /// approval over every call, and answers `{ ok, pending: true, requestId, handle }` —
    /// **never a transaction hash**. Nothing has been signed or broadcast. `handle` is the
    /// keystore's name for the approval record, for pointing a signer at it. Drive the rest
    /// with `send_status`.
    fn send(&self, request_json: String) -> String;

    /// Advance a pending send and report where it got to. Poll this — there is no
    /// background advancer, so this call IS the broadcast.
    ///
    /// Once the human has approved, this collects the signatures, broadcasts them in call
    /// order — each recorded before it leaves, each exactly once — and stops at the first
    /// that does not land. `{ ok, requestId, handle, status, final, origin, purpose, legs:
    /// [{ to, nonce, label, hash? }], hashes, hash?, route?, reason? }` where `status` is
    /// `awaitingApproval` | `broadcasting` | `stuck` | `broadcast` | `rejected` |
    /// `cancelled` | `failed`. `hashes` are the calls that answered, in order; `hash` is the
    /// last. A `failed` bundle whose earlier calls landed still lists them.
    ///
    /// `final` is when to stop polling: false for `awaitingApproval` and `broadcasting`, true
    /// for every other status, `stuck` included. A refusal is `{ ok: false, error, final }`,
    /// final only for a request id this module does not hold — any other may pass on the next
    /// poll, and a poller that stops on it strands an approved send.
    ///
    /// A reply carrying `blocked: true` is a send being HELD by the verified-proxy gate, not
    /// a failed one: `ok` stays true, the nonces stay reserved, and the next poll sends it
    /// once the proxy is usable.
    fn send_status(&self, request_id: String) -> String;

    /// Withdraw a send that has not been approved yet, releasing its reserved nonces.
    fn cancel_send(&self, request_id: String) -> String;

    /// Every send that could still move and is not stuck: `{ ok, sends: [{ requestId,
    /// handle, chainId, from, status, origin, purpose }] }`. What a consumer that must not
    /// change the ground under a pending approval — a wallet switching networks — asks first.
    fn live_sends(&self) -> String;

    /// Locally recorded transactions for `address`, newest first, on `chain_id` — or on every
    /// chain when it is 0. Only transactions this module broadcast: there is no indexer.
    ///
    /// `{ ok, chainId, address, transactions, stillDue, stillDueAnyChain, unstored,
    /// unresolved, blockedChains, strandedNonces }`. Each row carries its stored fields plus
    /// `stalled`, `unresolved` and `verificationBlocked`; ether figures are decorated at 18
    /// places and gas prices in gwei. A row's `label`, `purpose`, `origin` and `meta` are the
    /// requester's own, returned verbatim — a token amount inside `meta` is the consumer's to
    /// render. Sweeps due receipts first.
    fn history(&self, address: String, chain_id: i64) -> String;

    /// Poll receipts for this address's still-pending transactions, on each row's OWN chain,
    /// and update their stored status. `{ ok, address, polled, changed, blocked,
    /// blockedChains, stillDue }`. `stillDue` is false once no row can move again.
    fn refresh_pending(&self, address: String) -> String;

    /// Re-read one recorded transaction's receipt on ITS OWN chain and update the stored
    /// status. `{ ok, hash, chainId, status, route }`.
    fn refresh_tx_status(&self, address: String, hash_hex: String) -> String;

    /// The transaction- and block-level fields a RECEIPT does not carry, for one recorded
    /// transaction. `{ ok, hash, chainId, route, fetchedAt, gasPriceUnit, block?,
    /// transaction?, blockError?, transactionError? }`; `ok` is true when EITHER leg landed.
    fn tx_details(&self, address: String, hash_hex: String) -> String;

    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
