// Extracted by logos-module-atlas from logos-co/logos-evm-assets-module@3235e6bfaa63:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-evm-assets-module/blob/3235e6bfaa63c85f5857591da290bb1631ece183/rust-lib/src/glue.rs

pub trait EvmAssetsModule: Send + Sync + 'static {
    /// Native currency followed by the given ERC-20 descriptors, as asset rows.
    fn list_assets(&self, chain_id: i64, tokens_json: String) -> String;
    /// One Multicall3 read for native and every given token balance.
    fn get_balances(
        &self,
        chain_id: i64,
        address: String,
        tokens_json: String,
        token_sort: String,
    ) -> String;
    /// Build exactly one unsigned native or ERC-20 call, resolved among native and the
    /// request's `tokens`. Never signs or broadcasts.
    fn build_transfer(&self, chain_id: i64, request_json: String) -> String;
    /// Resolve native or one given token by exact address or unambiguous symbol.
    fn resolve_asset(&self, chain_id: i64, key: String, tokens_json: String) -> String;
    /// Decorate sender history with `{ "<chainId>": [descriptors] }`, native only for a chain
    /// the map omits. A chain-registry failure leaves that chain's rows intact, reported in
    /// `decorationErrors`; it never erases usable activity from the other chains.
    fn decorate_history(&self, history_json: String, tokens_json: String) -> String;
    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
