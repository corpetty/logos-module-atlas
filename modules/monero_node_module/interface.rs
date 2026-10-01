// Extracted by logos-module-atlas from logos-co/logos-monero-node-module@ddaec9337b4d:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-monero-node-module/blob/ddaec9337b4d1c4d1ccc4cea6fdd4163b2dd6e0f/rust-lib/src/glue.rs

pub trait MoneroNodeModule: Send + Sync + 'static {
    /// Store one network's config from `{ url, username?, password?, proxy?, proxyRequired?, timeoutSecs?, trusted?, mode? }`.
    /// A full replace: `mode` is `remote` unless the config says `local`.
    fn set_node_config(&self, network: String, config_json: String) -> String;
    fn get_node_config(&self, network: String) -> String;
    /// What a wallet dials: the stored config, or in local mode monerod_module's loopback URL,
    /// trusted and unproxied. `{ ok, result }` like get_node_config.
    fn effective_node(&self, network: String) -> String;
    /// `{ ok, available, rpcUrl?, status?, error? }`: whether monerod_module can serve `network`.
    fn local_node(&self, network: String) -> String;
    fn remove_node_config(&self, network: String) -> bool;
    /// `{ ok, networks: [name, ...] }`.
    fn list_networks(&self) -> String;

    /// `{ ok, state: "unready"|"unconfigured"|"configured", source, networks }`.
    fn config_status(&self) -> String;
    /// Seed well-known defaults per-field-if-absent. `{ ok, applied: [...] }`.
    fn init_defaults(&self) -> String;

    /// `{ reachable, height, targetHeight, synced, restricted, rttMs, mode, local? }`.
    fn node_health(&self, network: String) -> String;

    fn get_info(&self, network: String) -> String;
    fn get_fee_estimate(&self, network: String) -> String;
    fn get_version(&self, network: String) -> String;
    fn hard_fork_info(&self, network: String) -> String;
    /// Broadcast a signed tx blob via `/send_raw_transaction`.
    fn send_raw_transaction(&self, network: String, tx_as_hex: String) -> String;
    /// regtest only: mine `amount` blocks to `address`.
    fn generateblocks(&self, network: String, address: String, amount: i64) -> String;

    /// Escape hatch: an arbitrary `/json_rpc` method. `params_json` is a JSON value.
    fn raw_json_rpc(&self, network: String, method: String, params_json: String) -> String;
    /// Escape hatch: an arbitrary "other" endpoint (POST `<url>/<path>`).
    fn raw_endpoint(&self, network: String, path: String, body_json: String) -> String;

    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
