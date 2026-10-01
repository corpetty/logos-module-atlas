// Extracted by logos-module-atlas from logos-co/logos-uniswap-backend@b561baac2d32:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-uniswap-backend/blob/b561baac2d32e7f9aa234c8529deb66eb344ec00/rust-lib/src/glue.rs

pub trait UniswapBackendModule: Send + Sync + 'static {
    /// The chains a swap can happen on: eth_rpc's enabled, in-scope chains that uniswap_module
    /// holds a deployment for. `{ ok, scope, networks: [{ chainId, name, nativeSymbol,
    /// nativeDecimals, testnet, verifiedProxyMode }], unsupported: [chainId] }`; `unsupported`
    /// names enabled chains Uniswap is not deployed on.
    fn networks(&self) -> String;

    /// eth_rpc's verified-proxy verdict for `chain_id`: `{ ok, chainId, mode, state, usable,
    /// blocking, message, action, detail }`. A verdict that cannot be read is a blocking
    /// `mode: "unknown"` one, never `off`.
    fn verdict(&self, chain_id: i64) -> String;

    /// The native coin, then the tokens offered on `chain_id` (pinned and enabled), as asset
    /// rows: `{ ok, chainId, tokens: [{ symbol, name, decimals, address?, native, … }] }`.
    fn tokens(&self, chain_id: i64) -> String;

    /// The token picker: token_list's catalogue for `chain_id`, the native row first on the
    /// first page. `{ ok, chainId, total, offset, shown, hasMore, listed, tokens, listError? }`.
    fn catalogue(&self, chain_id: i64, query: String, offset: i64, limit: i64) -> String;

    /// Balances of `address` on `chain_id` for the native coin, every offered token and any
    /// extra token rows the caller names (the pair on screen, say). evm_assets' reply:
    /// `{ ok, chainId, address, tokenSort, balances, route }`.
    fn balances(&self, chain_id: i64, address: String, tokens_json: String) -> String;

    /// fee_module's tiers for `chain_id`, relayed verbatim.
    fn fee_tiers(&self, chain_id: i64) -> String;

    /// The keystore's accounts, their names and the wallets they were derived under, in one
    /// reply: `{ ok, accounts, labels?, wallets? }`. Read-only: this module can never create,
    /// import or sign.
    fn accounts(&self) -> String;

    /// Price a swap without side effects. `request_json`: `{ chainId, from, tokenIn, tokenOut,
    /// amountUnits | amountIn, decimalsIn?, decimalsOut?, symbolIn?, symbolOut?, slippageBps?,
    /// deadlineMins?, tier?, recipient?, maxFeePerGas?, maxPriorityFeePerGas?, gasLimits?,
    /// nonce? }`; a token is an address, "ETH", or an offered symbol, and missing decimals are
    /// looked up. The fee fields are the user's, in wei, relayed as given, `gasLimits` one per
    /// call in build order. The reply is uniswap_module's `build_swap` with the display figures
    /// and `tx_sender_module`'s `prepare` under `fee`.
    fn quote(&self, request_json: String) -> String;

    /// Build the swap afresh and ask the sender to make it: `{ ok, pending, requestId, handle,
    /// chainId, from, purpose, amountOutMin, deadline }`. Nothing is signed yet; poll
    /// `swap_status`.
    fn swap(&self, request_json: String) -> String;

    /// Advance a swap and report where it got to: the sender's `send_status` and its `final`,
    /// read off `status` for a sender that predates it. Polling IS the broadcast. `final:
    /// false` means ask again, refusals included.
    fn swap_status(&self, request_id: String) -> String;

    /// Withdraw a swap nobody has approved yet, releasing its nonces.
    fn cancel_swap(&self, request_id: String) -> String;

    /// This app's swaps for `address` on `chain_id`, newest first, one entry per bundle:
    /// `{ ok, chainId, address, stillDue, swaps: [{ requestId, status, timestamp, hashes, legs,
    /// swap, label, origin, via }] }`. Receipts due are swept first.
    fn swaps(&self, address: String, chain_id: i64) -> String;

    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
