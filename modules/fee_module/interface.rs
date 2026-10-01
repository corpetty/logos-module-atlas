// Extracted by logos-module-atlas from logos-co/logos-evm-fee-module@5bf49b768cf1:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-evm-fee-module/blob/5bf49b768cf178d2e8c58267f17822060f813fbf/rust-lib/src/glue.rs

pub trait FeeModule: Send + Sync + 'static {
    /// Slow/normal/fast suggestions for a chain, derived from `eth_feeHistory`.
    ///
    /// `{ ok, chainId, baseFeePerGas, source, tiers: { slow, normal, fast } }`
    /// where each tier is `{ maxFeePerGas, maxPriorityFeePerGas }` as decimal
    /// wei strings. `source` is `"feeHistory"`, `"maxPriorityFee"` (a 1559 chain
    /// whose reward rows were blank, priced off the node's own tip) or
    /// `"gasPrice"` (a chain with no base fee). A fee history that cannot be read
    /// is `ok: false`.
    fn suggest_fees(&self, chain_id: i64) -> String;

    /// Resolve a concrete fee for ONE call, honouring any override.
    ///
    /// `request_json` accepts:
    ///   `{ "tier": "slow"|"normal"|"fast" }`            — pick a suggested tier
    ///   `{ "maxFeePerGas": "...", "maxPriorityFeePerGas": "..." }` — override
    ///   `{ "gasLimit": "..." }`                          — override the limit
    ///   `{ "tx": { ... } }`                              — estimate the limit
    ///   `{ "deadlineMs": 5000 }`                         — bound this call
    ///
    /// Returns `{ ok, chainId, maxFeePerGas, maxPriorityFeePerGas, gasLimit,
    /// gasSource, feeCeilingWei(+Display/Exact), totalWei, baseFeePerGas,
    /// source }`. `totalWei` equals `feeCeilingWei` and stays for older callers.
    /// An explicit fee field is used verbatim — this module advises, it does not
    /// overrule the user. With one set, the other comes from the tier.
    fn estimate(&self, chain_id: i64, request_json: String) -> String;

    /// Price a bundle of calls that will leave in order, from one account.
    ///
    /// `request_json`: `{ from, calls: [{ to, value?, data?, gasLimit?, label? }],
    /// tier? | maxFeePerGas? + maxPriorityFeePerGas?, deadlineMs? }`.
    ///
    /// Each call is estimated as the chain will find it: an ERC-20 `approve` in an
    /// earlier call becomes a state override on that token's allowance slot for
    /// every later call, so a swap behind its approval gets a real estimate and a
    /// USDT-style reset-then-set is estimated with the reset applied. A call with
    /// its own `gasLimit` is taken as given. The first call that cannot be
    /// estimated refuses the bundle, naming it.
    ///
    /// Returns `{ ok, chainId, source, baseFeePerGas, maxFeePerGas,
    /// maxPriorityFeePerGas, gasLimit, feeCeilingWei(+Display/Exact),
    /// calls: [{ gasLimit, gasSource: "given"|"estimated"|"simulated",
    /// feeCeilingWei(+Display/Exact) }], assumptions: [{ call, after, token,
    /// spender, allowance }], nativeDecimals }`. One fee for the bundle; every
    /// ceiling is `maxFeePerGas × gasLimit`, in wei and in the native unit.
    fn estimate_bundle(&self, chain_id: i64, request_json: String) -> String;

    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
