// Extracted by logos-module-atlas from logos-co/logos-monero-wallet-cli@9f5958f7172a:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-monero-wallet-cli/blob/9f5958f7172a2f12c7fa16996da1ea3fdf1a7905/rust-lib/src/glue.rs

pub trait MoneroWalletCliModule: Send + Sync + 'static {
    /// `{ ok, held, custodian, approver, identity, approvers, custodians, awaiting, hint }`.
    /// `held` is "holds every role this module needs"; `hint` names the exact `configure`.
    fn status(&self) -> String;

    // ---- the wallet: custodian-gated (open/create/restore/password/seed/network) ----
    /// CUSTODIAN. Open a registered wallet on the active network. `{ ok, jobId }`; poll job_status.
    fn open_wallet(&self, name: String, password: String) -> String;
    /// CUSTODIAN. `{ ok, jobId }`. The wallet is registered on the active network and left open.
    fn create_wallet(&self, name: String, password: String, label: String) -> String;
    /// CUSTODIAN. `{ name, password, seed, restoreHeight, seedOffset?, label? }` as one quoted
    /// document or `@file`. A restore height of 0 scans from genesis — hours.
    fn restore_from_seed(&self, params_json: String) -> String;
    /// CUSTODIAN. `{ name, password, address, viewKey, spendKey?, restoreHeight, label? }`.
    /// Omit `spendKey` for a view-only wallet.
    fn restore_from_keys(&self, params_json: String) -> String;
    /// CUSTODIAN. `{ ok, jobId }`.
    fn change_password(&self, old_password: String, new_password: String) -> String;
    /// CUSTODIAN. The 25-word seed, after the backend re-checks the password. Never stored.
    fn reveal_seed(&self, password: String) -> String;
    /// CUSTODIAN. The private view key, after the backend re-checks the password.
    fn reveal_view_key(&self, password: String) -> String;
    /// CUSTODIAN. Refused while a wallet is open or a send is in flight.
    fn set_active_network(&self, network: String) -> String;
    /// Either role. Stores and closes the open wallet. `{ ok, jobId }`.
    fn close_wallet(&self) -> String;

    // ---- spending: build, review, decide ----
    /// Build a send for review: `{ address, amountXmr | amount, priority?, accountIndex? }` as one
    /// quoted document. `{ ok, requestId }`; the preview arrives as a `prompt` event.
    fn prepare_send(&self, send_json: String) -> String;
    /// `transfer <address> <amount>` — prepare_send without writing a JSON document by hand.
    fn transfer(&self, address: String, amount_xmr: String) -> String;
    /// The block a human reads for one previewed send, plus the backend's status.
    fn show(&self, request_id: String) -> String;
    /// APPROVER: broadcast the previewed send. Governs BROADCAST — the engine signed at build.
    fn confirm(&self, request_id: String) -> String;
    /// Withdraw an unbroadcast send.
    fn cancel(&self, request_id: String) -> String;
    /// `{ ok, sends: [{ requestId, state }] }`.
    fn list(&self) -> String;

    // ---- reads, relayed so one module covers a headless session ----
    fn send_status(&self, request_id: String) -> String;
    /// `{ ok, state: queued|running|done|failed, result?, error? }` for a lifecycle job id.
    fn job_status(&self, job_id: String) -> String;
    fn wallet_status(&self) -> String;
    fn list_wallets(&self) -> String;
    fn list_networks(&self) -> String;
    fn balances(&self, account_index: i64) -> String;
    /// The primary address and every subaddress of an account.
    fn receive_info(&self, account_index: i64) -> String;
    /// `address new [<label>]` — derive a fresh subaddress on the account.
    fn address_new(&self, account_index: i64, label: String) -> String;
    fn history(&self) -> String;
    fn address_valid(&self, address: String) -> bool;
    fn format_xmr(&self, atomic: String) -> String;
    fn parse_xmr(&self, xmr: String) -> String;
    fn caller_identity(&self) -> String;

    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
