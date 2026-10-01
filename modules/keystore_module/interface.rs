// Extracted by logos-module-atlas from logos-co/logos-evm-keystore-module@2318c679e2b7:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-evm-keystore-module/blob/2318c679e2b7176967bd45052f3d64b2a8b06931/rust-lib/src/glue.rs

/// The keystore module's IPC contract. Each non-defaulted method is a callable
/// module method. Private keys never appear in any signature — only addresses,
/// signed payloads, and (re-encrypted) keystore JSON cross the boundary.
pub trait KeystoreModule: Send + 'static {
    /// Name who holds the two roles: `{ approver?, custodian? }` → `{ ok, approver,
    /// custodian }`. TOTAL — a role the document does not name is held by nobody — and it
    /// takes effect at once, on a module that has been serving the built-in defaults
    /// (`evm_signer_ui`, `evm_keystore_ui`) since it loaded. A malformed document is refused and
    /// the roles in force are left untouched.
    ///
    /// UNGATED, deliberately and for now: any caller can name itself custodian and then
    /// mutate the keystore. Protecting it is deferred, not refuted — see docs/specs.md.
    fn configure(&mut self, config_json: String) -> String;
    /// Generate a fresh BIP-39 mnemonic of `words` (12/15/18/21/24) — `{ ok, phrase }`.
    fn create_mnemonic(&mut self, words: i64) -> String;
    /// Derive + persist an account from a mnemonic, creating its derivation group. params
    /// JSON: `{ phrase, passphrase?, accountIndex?, password, storage?, bip44Account?,
    /// change?, groupPassword?, groupLabel? }` → `{ ok, address, path, group, storage,
    /// index, origin }`. `accountIndex` is the ADDRESS index; `bip44Account` is the
    /// hardened BIP-44 account level. `storage` defaults to `"plain"` — keeping nothing —
    /// so the pre-HD call shape behaves exactly as it did.
    fn import_mnemonic(&mut self, params_json: String) -> String;
    /// Add the next account of a derivation group, without the phrase. params JSON:
    /// `{ group?, groupPassword, password, change? }` → `{ ok, address, path, group,
    /// index, origin }`. `group` may be omitted only when exactly one group can derive.
    fn derive_next_account(&mut self, params_json: String) -> String;
    /// Add one account at a chosen index. params JSON:
    /// `{ group, groupPassword, password, bip44Account?, change?, index }`.
    fn derive_account_at(&mut self, params_json: String) -> String;
    /// Addresses a group would derive, without writing anything and without a network.
    /// params JSON: `{ group, groupPassword, change?, from?, count? }` →
    /// `{ ok, group, addresses: [{ index, path, address, present }] }`.
    fn preview_addresses(&mut self, params_json: String) -> String;
    /// The ONE way to obtain a random key — a key no recovery phrase covers. params JSON:
    /// `{ password, acknowledgeUnrecoverable }`; the refusal says what an unrelated account
    /// IS rather than that a flag is missing.
    ///
    /// It replaced `new_account`, which minted one silently on a keystore that looked empty.
    /// The acknowledgement is enforced by construction: it generates the key, so a path that
    /// skipped it has nothing to persist.
    fn create_unrelated_account(&mut self, params_json: String) -> String;
    /// Stop keeping a group's derivation key. params JSON: `{ group }`.
    /// One-way: the material is gone, and re-importing the phrase is the only way back.
    /// Keyed on the FILES and needing no password — deletion must not require the ability to
    /// READ what it deletes, or a key whose password is lost becomes permanent while it goes
    /// on refusing every new account. Removes every path the id can occupy, the staging one
    /// included. Accounts already derived keep working; they just stop being extendable.
    fn forget_derivation(&mut self, params_json: String) -> String;
    /// Remove a wallet's record and its name. params JSON: `{ group }` →
    /// `{ ok, group, recordRemoved, nameRemoved }`. Tier D.
    ///
    /// Refuses while the wallet holds a derivation key (live or staged) or an account, so it
    /// never deletes key material — `forget_derivation` and `delete_account` stay the only
    /// writers that do, each keeping its own acknowledgement. Because "holds nothing" is the
    /// precondition, nothing signable is at stake and no password is asked for.
    fn remove_group(&mut self, params_json: String) -> String;
    /// Derivation groups, `{ ok, groups: [...] }`. UNGATED, like `get_labels`: none of it
    /// is a secret, and a wallet showing an account picker needs it. Includes STRANDED
    /// groups — a derivation key on disk that no record names.
    fn list_groups(&mut self) -> String;
    /// What the key directory holds: `{ ok, groups, staged, unexplained }`. UNGATED. Reads
    /// the directory ONLY, so a wallet whose bookkeeping is unreadable can still be named —
    /// and therefore still deleted with `forget_derivation`. `staged` are ids an interrupted
    /// import left a copy of; `unexplained` are paths the layout does not account for. That
    /// last list REPORTS; it no longer refuses anything.
    fn list_derivation_keys(&mut self) -> String;
    /// Where each account came from, `{ ok, accounts: { <address>: {...} } }`. UNGATED.
    fn get_provenance(&mut self) -> String;
    /// Wallet ownership for account pickers,
    /// `{ ok, wallets: { <address>: { wallet, index? } } }`. UNGATED. This is the
    /// canonical join of account provenance and wallet names, so every consumer groups
    /// accounts the same way.
    fn get_account_wallets(&mut self) -> String;
    /// Bring the keystore directory to a state the layout explains, and report both what it
    /// DID and what is left: `{ ok, swept, promoted, unexplained, links, staged, importStages }`.
    /// Tier D — it removes things. A leftover only named after it has been swept was never
    /// nameable, so the reply says what went.
    /// Callable by name rather than only as a side effect of listing, so a stage a crash
    /// left behind is not waiting on something happening to call `list_accounts`.
    fn settle(&mut self) -> String;
    /// Remove one path `settle`/`list_accounts` reported as unexplained. params JSON:
    /// `{ path, acknowledgeMayBeKeyMaterial }` → `{ ok, removed }`. Tier D. Only a string
    /// the scan itself produced is accepted, so nothing outside `<ks>/` can be named; the
    /// acknowledgement is required because unidentified material may BE a live key.
    fn remove_unexplained(&mut self, params_json: String) -> String;
    /// Import a raw private key (hex), persisted under `password` → `{ ok, address }`.
    fn import_private_key(&mut self, priv_hex: String, password: String) -> String;
    /// Import a scrypt keystore JSON, re-encrypted under `new_password` → `{ ok, address }`.
    fn import_keystore_json(&mut self, key_json: String, password: String, new_password: String) -> String;
    /// Export an account's scrypt keystore JSON (requires its password) → `{ ok, keystore }`.
    fn export_keystore_json(&mut self, address: String, password: String) -> String;
    /// `{ ok, accounts: [address, ...] }`.
    fn list_accounts(&mut self) -> String;
    fn has_address(&mut self, address: String) -> bool;
    fn delete_account(&mut self, address: String, password: String) -> bool;
    /// Re-encrypt a vault under a new password. Tier D. Crash-safe: the new vault is staged
    /// and renamed, so a failure cannot leave the account with no readable copy.
    /// `{ ok, address }`, or `{ ok: false }` if the old password is wrong.
    fn change_password(&mut self, address: String, old_password: String, new_password: String) -> String;
    /// Name an account, or clear the name with an empty string. Tier D, plus the account's
    /// own vault password when a name is being SET: a label is what a wallet shows in place
    /// of an address, so writing one is a claim of custody and now has to prove it. Clearing
    /// needs no password — it can only move the display toward the raw address, and it is
    /// the one way to strip a stale name off an account whose password is lost. `{ ok }`.
    fn set_label(&mut self, address: String, label: String, password: String) -> String;
    /// Account names, `{ ok, labels: { <address>: <name> } }`. UNGATED: a label is not a
    /// secret, and a wallet showing an account picker needs it.
    fn get_labels(&mut self) -> String;
    /// Name a wallet, or clear the name with an empty string. params JSON:
    /// `{ group, label, address?, password? }` → `{ ok }`. Tier D, plus the credential of
    /// whatever the wallet HOLDS when a name is being set: one of its own accounts (named by
    /// `address`) where it has accounts, because the name is a claim about them; the
    /// derivation key's password where it has only a key, because the name will come to
    /// stand over what that key mints. Free only where it holds neither. No uniqueness rule.
    fn set_group_label(&mut self, params_json: String) -> String;
    /// Wallet names, `{ ok, labels: { <groupId>: <name> } }`. UNGATED, like `get_labels`,
    /// and answered from its own document — so a wallet whose record is gone is still
    /// nameable on screen.
    fn get_group_labels(&mut self) -> String;
    // ── Tier B: any NAMED module may ask ────────────────────────────────
    /// Ask a human to approve signing. Returns immediately with
    /// `{ ok, handle, receipt }` — it does NOT block on the human. `handle` is
    /// announced on the event plane; `receipt` is returned exactly once and is
    /// what authorises collecting the result.
    fn request_approval(&mut self, intent_json: String) -> String;
    /// Bare state for the requester — never the intent, never the results.
    /// `{ ok, state, reason? }`.
    fn approval_status(&mut self, handle: String, receipt: String) -> String;
    /// Collect the signatures. Idempotent until `ack_result`, so a dropped
    /// reply does not cost the human a second password entry.
    fn fetch_result(&mut self, handle: String, receipt: String) -> String;
    /// The requester has the signatures; erase them.
    fn ack_result(&mut self, handle: String, receipt: String) -> bool;
    /// The requester gave up.
    fn cancel_approval(&mut self, handle: String, receipt: String) -> bool;

    // ── Tier A: the configured approver only ────────────────────────────
    /// Queue summaries — never leg detail. `{ ok, pending: [...] }`.
    fn pending(&mut self) -> String;
    /// Claim a request for display. Returns the lines to show VERBATIM plus the
    /// commitment to echo back. Demotes any other rendered request, so exactly
    /// one thing can be on screen. `claim_lines` is the requester's own account of
    /// what this is for; `render_lines` is what is actually signed. An approver must
    /// keep them apart. `{ ok, handle, bundle_id, requester, claim_lines, render_lines }`.
    fn acknowledge(&mut self, handle: String) -> String;
    /// The human said yes. One key derivation, every leg signed, then wiped.
    /// `bundle_id` must be the value that was displayed. `{ ok, signed }`.
    fn approve(&mut self, handle: String, bundle_id: String, password: String) -> String;
    /// The human said no.
    fn reject(&mut self, handle: String) -> bool;

    /// Observability: what this module currently sees as its caller. Ungated
    /// and side-effect-free — identity cannot report its own absence.
    fn caller_identity(&mut self) -> String;
    /// Framework hook — defaulted, so it is NOT part of the IPC contract.
    fn on_context_ready(&mut self, _ctx: &RustModuleContext) {}
}
