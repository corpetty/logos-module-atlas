// Extracted by logos-module-atlas from logos-co/logos-monero-wallet-backend@769098441db3:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-monero-wallet-backend/blob/769098441db339f7541ba8804722ae3f20614309/rust-lib/src/glue.rs

pub trait MoneroWalletBackendModule: Send + Sync + 'static {
    /// Name who holds the two roles: `{ approvers?, custodians? }` → `{ ok, approvers, custodians }`.
    /// TOTAL — a role the document does not name is held by nobody — and in force at once, on a
    /// module serving the defaults (`monero_wallet_ui` holds both roles: Monero's password is a
    /// once-per-session unlock, so the surface that spends is the surface that unlocks) since
    /// it loaded. A malformed document is refused and the roles in force stay.
    ///
    /// UNGATED, deliberately and for now — the same standing gap keystore_module documents: any
    /// caller can name itself custodian. Protecting it is deferred, not refuted.
    fn configure(&self, config_json: String) -> String;
    /// `{ ok, kind, identity, approvers, custodians }` — who this call authenticated as, and who
    /// holds the roles. A headless relay uses it to say exactly which `configure` would admit it.
    fn caller_identity(&self) -> String;

    /// `{ ok, networks: [...], active }`.
    fn list_networks(&self) -> String;
    /// CUSTODIAN. Refused while a wallet is open or a send is in flight.
    fn set_active_network(&self, network: String) -> String;

    /// `{ ok, wallets: [{ name, network, label, viewOnly, restoreHeight, address }] }` — the
    /// registry merged with what the engine finds on disk; unverified files have network="".
    fn list_wallets(&self) -> String;
    /// CUSTODIAN. Open on the selected network; a verified wallet on another network is refused.
    /// `{ ok, jobId }`; poll job_status. The core verifies the encrypted wallet before connecting.
    fn open_wallet(&self, name: String, password: String) -> String;
    /// CUSTODIAN. `{ ok, jobId }`. The wallet is registered on the active network.
    fn create_wallet(&self, name: String, password: String, label: String) -> String;
    /// `params_json`: `{ name, password, seed, restoreHeight, seedOffset?, label? }` → `{ ok, jobId }`.
    fn restore_from_seed(&self, params_json: String) -> String;
    /// `params_json`: `{ name, password, address, viewKey, spendKey?, restoreHeight, label? }` → `{ ok, jobId }`.
    fn restore_from_keys(&self, params_json: String) -> String;
    /// `{ ok, jobId }`.
    fn close_wallet(&self) -> String;
    fn change_password(&self, old_password: String, new_password: String) -> String;
    /// Pass-through to the engine, which re-checks the password. Never cached here.
    fn reveal_seed(&self, password: String) -> String;
    fn reveal_view_key(&self, password: String) -> String;
    /// `{ ok, state: queued|running|done|failed, result?, error? }` for a backend job id.
    fn job_status(&self, job_id: String) -> String;

    /// The engine's status plus `syncPercent` and the registry's meta for the open wallet.
    fn wallet_status(&self) -> String;
    /// `{ ok, balance, unlocked, balanceXmr, unlockedXmr }` — atomic units as decimal strings.
    /// A read the engine could not serve answers `{ ok: false, busy: true }` while it is building
    /// or committing: unread, not zero, and not a failure a poller should report.
    fn balances(&self, account_index: i64) -> String;
    /// `{ ok, address, subaddresses: [{ index, address, label }] }`.
    fn receive_info(&self, account_index: i64) -> String;
    fn create_subaddress(&self, account_index: i64, label: String) -> String;
    /// Rename a subaddress, or clear the label with an empty string. `{ ok, index, label }`.
    fn set_subaddress_label(&self, account_index: i64, address_index: i64, label: String) -> String;
    /// `{ ok, rows: [...] }`, newest first, amounts as decimal strings + XMR strings.
    fn history(&self) -> String;
    fn address_valid(&self, address: String) -> bool;
    /// The node module's health for the active network.
    fn node_health(&self) -> String;
    /// The active network's node config, with the RPC password REDACTED to `hasPassword`.
    /// A wallet surface needs to know a password is set, never what it is.
    fn node_config(&self) -> String;
    /// Whether monerod_module can serve the active network: `{ ok, available, rpcUrl?, status?, error? }`.
    fn local_node(&self) -> String;
    /// CUSTODIAN. Point the active network at a different daemon.
    /// `{ url, username?, password?, proxy?, proxyRequired?, trusted?, mode? }` — omitting `password`
    /// KEEPS the stored one; send `""` to clear it. `mode` is `remote` unless it says `local`.
    /// Refused while a wallet is open, because wallet2 binds its daemon at init.
    fn set_node_config(&self, config_json: String) -> String;

    /// Build a transaction for review. `send_json`: `{ address, amountXmr | amount, priority?, accountIndex? }`.
    /// `{ ok, requestId }`; poll send_status for the preview. At most one send in flight.
    fn prepare_send(&self, send_json: String) -> String;
    /// `{ ok, requestId, state: preparing|previewed|committing|sent|failed|cancelled, preview?, txids?, error? }`.
    fn send_status(&self, request_id: String) -> String;
    /// `{ ok, sends: [{ requestId, state }] }` — every request not yet acked away; what a headless
    /// approver polls to find previews awaiting a decision.
    fn list_sends(&self) -> String;
    /// APPROVER. Broadcast a previewed transaction. This governs BROADCAST — the engine already
    /// signed when it built the preview.
    fn confirm_send(&self, request_id: String) -> String;
    fn cancel_send(&self, request_id: String) -> String;

    fn format_xmr(&self, atomic: String) -> String;
    fn parse_xmr(&self, xmr: String) -> String;

    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
