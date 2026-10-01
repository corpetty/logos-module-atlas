// Extracted by logos-module-atlas from logos-co/logos-evm-keystore-cli@47554a1725fd:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-evm-keystore-cli/blob/47554a1725fd654f0e4a19a9fa520492e28e2b2d/rust-lib/src/glue.rs

pub trait EvmKeystoreCliModule: Send + 'static {
    /// `{ ok, held, identity, approvers, custodians, hint }`.
    fn status(&mut self) -> String;

    // Tier D — admitted only once `configure` names evm_keystore_cli a custodian.
    fn create_mnemonic(&mut self, words: i64) -> String;
    fn import_mnemonic(&mut self, params_json: String) -> String;
    fn derive_next_account(&mut self, params_json: String) -> String;
    fn derive_account_at(&mut self, params_json: String) -> String;
    fn preview_addresses(&mut self, params_json: String) -> String;
    fn create_unrelated_account(&mut self, params_json: String) -> String;
    fn forget_derivation(&mut self, params_json: String) -> String;
    fn remove_group(&mut self, params_json: String) -> String;
    fn settle(&mut self) -> String;
    fn remove_unexplained(&mut self, params_json: String) -> String;
    fn set_group_label(&mut self, params_json: String) -> String;
    fn import_private_key(&mut self, priv_hex: String, password: String) -> String;
    fn import_keystore_json(&mut self, key_json: String, password: String, new_password: String) -> String;
    fn export_keystore_json(&mut self, address: String, password: String) -> String;
    /// `{ ok }` rather than the keystore's bare bool, so a refusal can say why.
    fn delete_account(&mut self, address: String, password: String) -> String;
    fn change_password(&mut self, address: String, old_password: String, new_password: String) -> String;
    fn set_label(&mut self, address: String, label: String, password: String) -> String;

    // Ungated reads, relayed so one module covers the whole headless session.
    fn list_accounts(&mut self) -> String;
    fn get_labels(&mut self) -> String;
    fn get_group_labels(&mut self) -> String;
    fn list_groups(&mut self) -> String;
    fn list_derivation_keys(&mut self) -> String;
    fn get_provenance(&mut self) -> String;
    fn caller_identity(&mut self) -> String;

    fn on_context_ready(&mut self, _ctx: &RustModuleContext) {}
}
