// Extracted by logos-module-atlas from logos-co/logos-evm-eth-rpc-module@42cc465e0cbd:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-evm-eth-rpc-module/blob/42cc465e0cbd748117a0af0cf983404348683335/rust-lib/src/glue.rs

pub trait EthRpcModule: Send + Sync + 'static {
    /// Store config for a chain. `config_json`: `{ endpoint, proxy?, proxyRequired?, timeoutSecs? }`.
    /// An OMITTED `verifiedProxyMode` / `verifiedTimeoutSecs` preserves what is stored — only an
    /// explicit `"off"` lowers the mode, so a sibling wallet cannot revoke it by silence.
    fn set_chain_config(&self, chain_id: i64, config_json: String) -> bool;
    fn get_chain_config(&self, chain_id: i64) -> String;
    fn remove_chain_config(&self, chain_id: i64) -> bool;
    /// `{ ok, chains: [chainId, ...] }`.
    fn list_chains(&self) -> String;
    /// The chain registry, including metadata, enabled state and device-wide scope.
    fn list_chain_configs(&self) -> String;
    fn set_chain_enabled(&self, chain_id: i64, enabled: bool) -> String;
    /// Patch name/nativeSymbol/nativeDecimals/testnet. Omitted keeps; explicit null clears.
    fn patch_chain_metadata(&self, chain_id: i64, metadata_json: String) -> String;
    fn get_network_scope(&self) -> String;
    fn set_network_scope(&self, scope: String) -> String;

    /// `eth_chainId` round-trip → `{ ok, chainId }`.
    fn verify_chain_id(&self, chain_id: i64) -> String;
    fn block_number(&self, chain_id: i64) -> String;
    fn get_balance(&self, chain_id: i64, address: String) -> String;
    /// `eth_call` — `call_json` is a `{ to, data }` object (ERC20 reads).
    ///
    /// `deadline_ms` is the CALLER's own wall budget for this call, measured from when this
    /// module receives it. Absent or `0` leaves the chain's `timeoutSecs` in charge, so a
    /// caller with no budget of its own is unchanged. A supplied value may only SHORTEN what
    /// the chain already permits, never lengthen it: `chains.json` is shared with other
    /// wallets on this device, and a caller must not be able to widen its own exposure.
    ///
    /// The rule, so the next method to take one is an instance rather than a second
    /// exception: a method gains a deadline when the caller has a budget this module cannot
    /// infer. `call` is that method — it is the read behind a wallet's balance screen, and a
    /// caller racing it against a timeout it could neither set nor read is what this fixes.
    fn call(&self, chain_id: i64, call_json: String, deadline_ms: Option<i64>) -> String;
    fn get_transaction_count(&self, chain_id: i64, address: String) -> String;
    fn gas_price(&self, chain_id: i64) -> String;
    fn fee_history(&self, chain_id: i64, blocks: i64, reward_percentiles_json: String) -> String;
    fn estimate_gas(&self, chain_id: i64, tx_json: String) -> String;
    fn send_raw_transaction(&self, chain_id: i64, raw_hex: String) -> String;
    fn get_transaction_receipt(&self, chain_id: i64, hash_hex: String) -> String;
    fn get_transaction_by_hash(&self, chain_id: i64, hash_hex: String) -> String;
    /// Escape hatch for any standard JSON-RPC method. `params_json` is a JSON array.
    fn raw_rpc(&self, chain_id: i64, method: String, params_json: String) -> String;
    /// Like [`Self::raw_rpc`] but POSTs to an explicit `url` (not the chain's
    /// configured endpoint), reusing `chain_id`'s fail-closed proxied client. For
    /// off-chain JSON-RPC tied to a chain — e.g. an ERC-4337 bundler
    /// (`eth_sendUserOperation`) — so it too goes through net-proxy. `params_json`
    /// is a JSON array. On a chain set to `required` the proof-backed reads are REFUSED here:
    /// an arbitrary url cannot prove them, and answering anyway is a silent downgrade.
    fn raw_rpc_url(&self, chain_id: i64, url: String, method: String, params_json: String) -> String;

    /// Seed a chain only where it is ABSENT, per field, returning which fields were written.
    /// `chains.json` is shared with other wallets on this device: a blanket overwrite silently
    /// retunes theirs, a blanket skip leaves a stale value we own.
    fn ensure_chain_config(&self, chain_id: i64, config_json: String) -> String;
    /// Overwrite only the transport timeouts this module owns. Lowering a default is useless
    /// without this — an existing chains.json already carries the old value. 0 leaves a field.
    fn patch_chain_transport(&self, chain_id: i64, timeout_secs: i64, verified_timeout_secs: i64) -> String;
    /// Overwrite ONLY the endpoint, creating the chain with defaults if it is absent.
    /// `chains.json` is shared with other wallets on this device, so a user retyping an
    /// endpoint must not silently reset their verified-proxy mode or timeouts.
    fn patch_chain_endpoint(&self, chain_id: i64, endpoint: String) -> String;
    /// `"off"` talks to the endpoint; `"required"` routes through the light-client proxy and
    /// REFUSES rather than falling back. There is no `preferred`: answering from an unverified
    /// source when verification was asked for is the failure this prevents. This module is the
    /// single owner of the mode — a consumer keeping its own copy can disagree with the truth.
    fn set_verified_proxy_mode(&self, chain_id: i64, mode: String) -> String;
    /// The verified-proxy verdict for one chain: what state it is in, whether a verified call
    /// would answer now, and what the user has to do about it.
    /// `{ ok, chainId, mode, state, usable, blocking, message, action, detail }`.
    fn verified_proxy_status(&self, chain_id: i64) -> String;

    /// Whether a config has been SET, and what it is — no network I/O, no call to another
    /// module, so it is cheap enough for a consumer's startup path. `state` is the machine
    /// discriminator (`unready` / `unconfigured` / `configured`); never match on the message.
    /// `{ ok, state, source, chains: [{ chainId, state, source, endpoint?, verifiedProxyMode? }] }`.
    fn config_status(&self) -> String;
    /// Seed the built-in chains, per chain and per FIELD, only where absent. This module never
    /// calls it itself; any consumer may, unconditionally and at any time. A missing default
    /// chain is seeded at most once per device (`registry.json` records each one offered), so
    /// one removed with `remove_chain_config` stays removed. A second call from any consumer
    /// writes nothing and answers `applied: false`, which is not an error.
    /// `{ ok, applied, seeded: { "<chainId>": ["*" | "endpoint", ...] } }`.
    fn init_defaults(&self) -> String;

    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
