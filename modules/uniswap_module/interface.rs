// Extracted by logos-module-atlas from logos-co/logos-evm-uniswap-module@cc677c301f45:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-evm-uniswap-module/blob/cc677c301f45a00a1baac0358082df1fb271a650/rust-lib/src/glue.rs

pub trait UniswapModule: Send + Sync + 'static {
    /// Add or override a chain's Uniswap config (JSON of `ChainUniswap`).
    fn configure(&self, chain_json: String) -> bool;
    /// All configured chains (defaults + overrides).
    fn get_chains(&self) -> String;
    /// Token→ETH and token→USD prices for `{ "tokens": [{address, decimals}] }`,
    /// best-rate across V2/V3/V4, batched into one Multicall3 `eth_call`.
    fn get_prices(&self, chain_id: i64, tokens_json: String) -> String;
    /// Best swap quote for `{ tokenIn, tokenOut, amountIn, owner? }` (native = "ETH"):
    /// the route, its output, the price impact, a gas hint, and — with `owner` — the
    /// account's balance and whether an approval must go first.
    fn quote_swap(&self, chain_id: i64, params_json: String) -> String;
    /// The quote plus the calls that make the swap, in order, in the shape
    /// `tx_sender_module` takes: `{ calls: [{ kind, to, value, data, gasLimitHint, label }] }`.
    fn build_swap(&self, chain_id: i64, params_json: String) -> String;

    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
