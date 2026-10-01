// Extracted by logos-module-atlas from logos-co/logos-evm-signer-cli@861c2edafee2:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-evm-signer-cli/blob/861c2edafee23fd9322772d79c25a0ff5545e9e6/rust-lib/src/glue.rs

pub trait EvmSignerCliModule: Send + Sync + 'static {
    /// `{ ok, held, identity, approvers, custodians, rendered, pending_count, last_error, hint }`.
    fn status(&self) -> String;
    /// The keystore's queue summaries — never leg detail.
    fn list(&self) -> String;
    /// Claim `handle` for display: the keystore's lines, verbatim, this signer's own
    /// reading of them, and the prompt text.
    fn show(&self, handle: String) -> String;
    /// The human said yes to the request on screen. `bundle_id` must be the value shown.
    fn approve(&self, handle: String, bundle_id: String, password: String) -> String;
    /// The human said no.
    fn reject(&self, handle: String) -> bool;
    /// Re-read the queue now, then answer as `status` does.
    fn refresh(&self) -> String;
    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
