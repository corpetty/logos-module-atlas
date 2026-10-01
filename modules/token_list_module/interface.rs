// Extracted by logos-module-atlas from logos-co/logos-evm-token-list-module@213e62853fe8:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-evm-token-list-module/blob/213e62853fe817bad02db8588b8be6ecd1e72211/rust-lib/src/glue.rs

pub trait TokenListModule: Send + Sync + 'static {
    /// Set configuration: `{ listUrls?, proxy?, proxyRequired?, refreshSecs?,
    /// timeoutSecs?, useEmbeddedList? }`. Omitted keys keep their stored value.
    /// Emits `config_changed`, plus `tokens_updated` for any chain the new
    /// config re-serves (`useEmbeddedList` is the one that moves rows).
    fn configure(&self, config_json: String) -> bool;
    /// Whether a config has been SET, and what it is → `{ ok, state, source,
    /// config?, counts }`. No network I/O, so it is cheap on a startup path.
    fn config_status(&self) -> String;
    /// Apply the built-in offline defaults, only where nothing is configured →
    /// `{ ok, applied, state, source, config }`. Idempotent; `applied: false`
    /// means somebody got there first and is not an error. Emits
    /// `config_changed` when it applied, and normally no `tokens_updated`.
    fn init_defaults(&self) -> String;
    /// Fetch all configured lists through the fail-closed proxy → `{ ok, tokenCount }`.
    fn refresh_now(&self) -> String;
    /// Merged builtin + custom + downloaded + embedded tokens for a chain, each row
    /// labelled with the bucket it came from → `{ ok, tokens: [...] }`.
    fn get_tokens(&self, chain_id: i64) -> String;
    /// Metadata for specific tokens only; `addresses_json` is a JSON array of
    /// hex addresses matched case-insensitively → `{ ok, tokens: [...] }`.
    fn get_tokens_by_address(&self, chain_id: i64, addresses_json: String) -> String;
    fn get_all_tokens(&self) -> String;
    /// Enable snapshots the catalogue row; disabling removes the snapshot. Pinned rows cannot
    /// be disabled. `{ ok, chainId, address, enabled, changed }`.
    fn set_token_enabled(&self, chain_id: i64, address: String, enabled: bool) -> String;
    /// Persisted snapshots for one chain, each with `resolved` saying whether a current
    /// catalogue bucket still describes the address.
    fn get_enabled_tokens(&self, chain_id: i64) -> String;
    /// Pinned rows followed by enabled snapshots. ERC-20 only.
    fn list_offered(&self, chain_id: i64) -> String;
    /// Offered rows first, then the rest of the catalogue, with search and pagination.
    fn list_available(
        &self,
        chain_id: i64,
        query: String,
        offset: i64,
        limit: i64,
    ) -> String;
    /// Add a user token: `{ chainId, address, name, symbol, decimals, logoURI? }`.
    fn add_custom_token(&self, token_json: String) -> bool;
    /// Bulk-ingest one Uniswap-schema document into the CUSTOM list; the caller
    /// supplies the bytes, so this fetches nothing. `replace` swaps the list
    /// wholesale, else entries merge → `{ ok, tokenCount }`.
    fn import_custom_tokens(&self, list_json: String, replace: bool) -> String;
    fn remove_custom_token(&self, chain_id: i64, address: String) -> bool;
    fn get_custom_tokens(&self) -> String;
    fn get_list_sources(&self) -> String;
    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
