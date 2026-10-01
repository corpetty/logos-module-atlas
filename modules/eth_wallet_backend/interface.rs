// Extracted by logos-module-atlas from logos-co/logos-eth-wallet-backend@2866409f0450:rust-lib/src/glue.rs
// https://github.com/logos-co/logos-eth-wallet-backend/blob/2866409f045096e7f9eb8bc01841555b42e7a167/rust-lib/src/glue.rs

pub trait EthWalletBackendModule: Send + Sync + 'static {
    /// Enabled chains selected by the device scope, each carrying its current verdict.
    /// `{ ok, scope, networks: [{ chainId, name, nativeSymbol, testnet, rpcUrl,
    /// verifiedProxyMode, verifiedProxy }] }`.
    /// `rpcUrl`, `verifiedProxyMode` and `verifiedProxy` all come from `eth_rpc_module`, which
    /// owns them. All three are read-only here — a device-wide store shared with every wallet
    /// on the machine is configured in the `eth_rpc_ui` app, not from inside one wallet.
    ///
    /// Answers within a fixed budget however slow `eth_rpc` is. A network whose reads did
    /// not fit reports `verifiedProxyMode: "unknown"` and an empty `rpcUrl`.
    fn list_networks(&self) -> String;

    /// Relay the chain registry's enable switch.
    fn set_chain_enabled(&self, chain_id: i64, enabled: bool) -> String;
    /// Relay the device-wide `mainnets` | `testnets` | `both` scope selector.
    fn set_network_scope(&self, scope: String) -> String;

    /// `eth_rpc`'s verified-proxy verdict for every enabled in-scope chain:
    /// `{ ok, chains: [{ chainId, verdict }] }`.
    fn verified_proxy_state(&self) -> String;

    /// Tokens OFFERED on `chain_id`, native first: the built-in rows plus whatever
    /// the user turned on. `{ ok, chainId, tokenSort, tokens: [{ symbol, name, decimals,
    /// address?, native, builtin, inTokenList, metadataSource, logoURI? }] }`.
    ///
    /// `builtin` says the asset layer pins the address rather than accepting a user snapshot.
    /// `metadataSource` says who decorated the row:
    /// `native` | `allowlist` (ours, undecorated) | `custom` | `downloaded` | `embedded`
    /// (`token_list`'s own bucket labels, relayed rather than inferred) | `unknown` | `enabled`
    /// (a persisted snapshot the list no longer holds).
    fn list_tokens(&self, chain_id: i64) -> String;

    /// Every token that COULD be offered on `chain_id`: what the wallet offers now, plus
    /// everything `token_list_module` holds for that chain. The token picker's one read.
    ///
    /// `{ ok, chainId, tokenSort, total, shown, listed, tokens: [{ symbol, name, decimals,
    /// address?, native, enabled, builtin, logoURI?, source }], listError? }`. `source` uses
    /// the same vocabulary as `list_tokens`'s `metadataSource`, and `builtin` is true for the
    /// native row and the verified WETH row — the two that cannot be turned off.
    ///
    /// `query` matches a symbol or name (case-insensitive substring) or an exact address; an
    /// empty query matches everything. The answer comes in pages: `offset` skips that many
    /// matches and `limit` caps the rest (zero or less is no limit). `total` counts every
    /// match, `shown` the rows in this page, and `hasMore` says whether another page follows,
    /// so a view loads the list as it scrolls instead of presenting a slice as the whole.
    ///
    /// The embedded Uniswap list is overwhelmingly mainnet, so on sepolia and hoodi `listed`
    /// is legitimately 0 and the reply carries the built-in rows alone. That is an ANSWER:
    /// `ok` stays true, and `listError` — present only when the `token_list` call itself
    /// failed — is what tells an empty catalogue from an unread one.
    fn list_available_tokens(&self, chain_id: i64, query: String, offset: i64, limit: i64) -> String;

    /// Turn a token on or off for `chain_id`. `{ ok }` or `{ ok: false, error }`.
    ///
    /// Enabling SNAPSHOTS the whole record from `token_list_module` and refuses an address it
    /// does not hold on that chain: `decimals` scales every amount this wallet renders or
    /// signs, and there is no honest way to invent one. Enabling a built-in row succeeds and
    /// stores nothing — it is already offered. Disabling one is refused outright: the native
    /// currency pays every fee and WETH is this wallet's own assertion, not the user's.
    ///
    /// The change persists, so an enabled token is still offered after a restart, and emits
    /// `tokens_changed(chain_id)` once it is on disk — but only when the offered set actually
    /// moved. Turning on a token already enabled with the same snapshot, or a built-in row,
    /// changes nothing and says nothing.
    fn set_token_enabled(&self, chain_id: i64, address: String, enabled: bool) -> String;

    /// The order `get_balances` returns its rows in — `alpha` or `balance`. `{ ok, tokenSort }`.
    ///
    /// `balance` orders by each token's OWN amount. This wallet has no fiat price and will
    /// not fetch one, because a price feed discloses the user's IP — so across two different
    /// tokens this is NOT a value order, and nothing rendering it may imply that it is.
    ///
    /// Device-wide, and emits `token_sort_changed` on a move: the same rows come back in a
    /// new order on every network at once, so there is no chain to scope it to.
    fn set_token_sort(&self, order: String) -> String;

    /// Accounts the keystore holds. Read-only: this module can never create, import or
    /// export one — those are the custodian's, and reach the keystore only via `evm_keystore_ui`.
    fn list_accounts(&self) -> String;

    /// Account names, `{ ok, labels: { "<lowercase hex, no 0x>": "<name>" } }`, relayed from
    /// the keystore verbatim. `keystore_module.get_labels` is ungated — a label is not a
    /// secret — and this passthrough keeps a view's dependency list at exactly one module.
    ///
    /// The keys are `vault_name` form and `list_accounts` answers EIP-55 checksummed
    /// addresses, so the two never match textually and a lookup must normalise.
    fn get_account_labels(&self) -> String;

    /// The WALLET each account was derived under, and where in it:
    /// `{ ok, wallets: { "<address>": { "wallet": "<name>", "index": <n> } } }`.
    ///
    /// Separate from `get_account_labels` because they are different things and a view must
    /// tell them apart: an account's own name identifies THAT account, a wallet's name is
    /// shared by every account under it. `index` is the DERIVATION index, straight off
    /// `m/44'/60'/0'/0/<index>`, and it is stable for the life of the account. Absent for an
    /// account whose wallet has no name, and `index` is absent for one that was imported
    /// rather than derived — both are ordinary, and an empty map is the normal state.
    fn get_account_wallets(&self) -> String;

    /// The address book: `{ ok, contacts: [{ address, name }] }`, named rows first and then
    /// unnamed, each ordered by name and address so a picker can show them without sorting
    /// and the order does not move when an unrelated contact is added.
    ///
    /// These are COUNTERPARTIES and live here rather than in the keystore, which names
    /// accounts it holds keys for. A contact carries no key material and is not a secret.
    fn list_contacts(&self) -> String;

    /// Add a contact, or rename one already there — an UPSERT, because a user who saves an
    /// address they already have meant to name it. The address is stored EIP-55 and matched
    /// case-insensitively. `{ ok, contact: { address, name } }`. An empty name is allowed.
    fn save_contact(&self, address: String, name: String) -> String;

    /// Forget a contact. Removing one that is not there SUCCEEDS — the caller's goal is that
    /// the address is not in the book, and that is already true.
    fn forget_contact(&self, address: String) -> String;

    /// Native and token balances for `address` on every enabled chain in the device scope.
    /// The per-chain reads fan out concurrently, so another configured chain cannot multiply
    /// the interaction deadline. `{ ok, address, tokenSort, chains: [{ ok, chainId,
    /// balances: [{ symbol, address?, raw, decimals, native, builtin, display, exact,
    /// amountExact }], route } | { ok: false, chainId, error }] }`. `display` is bounded;
    /// `amountExact` carries every digit as a plain decimal string (`exact` is the older name
    /// for the same digits). All three are absent when a sub-call failed, so a view renders an
    /// em-dash and never a zero. A caller must not scale `raw` itself — a JS number loses
    /// digits above 2^53.
    ///
    /// EVERY offered token gets a row, including one the account holds none of. The array
    /// arrives ALREADY SORTED by the persisted `tokenSort` — comparing 18-decimal amounts is
    /// exact `U256` work and belongs where it is testable, not in QML.
    ///
    /// `route` is `eth_rpc`'s own label for the read — `verified` (proof-backed), `proxied`
    /// (forwarded on trust), `direct` (never touched the proxy) or `unknown`. Badge the
    /// balances on `route`, never on the network's mode.
    fn get_balances(&self, address: String) -> String;

    /// Transactions `tx_sender_module` broadcast for `address` on every chain in the current
    /// device scope, newest first — this wallet's own sends and any other app's calls from the
    /// same account. Only transactions the sender broadcast: there is no indexer.
    ///
    /// `{ ok, address, stillDue, stillDueAnyChain, unstored, unresolved, blockedChains,
    /// strandedNonces, transactions, decorationErrors? }`. A row this wallet sent as an ERC-20
    /// transfer reads back as one: `kind: "erc20"`, `to` the recipient, `value` the token
    /// amount at the token's decimals, `txTo` the contract. Another app's call keeps
    /// `kind: "call"` with its `label`, `origin` and `purpose`. Each row carries `stalled`,
    /// `unresolved` and `verificationBlocked`; `stillDue` covers the rows in THIS reply.
    fn get_history(&self, address: String) -> String;

    /// Fee tiers for `chain_id`, from `fee_module`. `{ ok, chainId, baseFeePerGas,
    /// source, tiers: { slow, normal, fast } }`; `source` distinguishes a real EIP-1559
    /// suggestion from the legacy `gasPrice` fallback.
    fn suggest_fees(&self, chain_id: i64) -> String;

    /// Quote a send without doing anything: resolves the token, checks an ERC-20 balance
    /// here, and has `tx_sender_module` price the fee, check the ether and read the nonce.
    ///
    /// `request_json`: `{ from, to, amount | amountUnits, token?, tokenAddress?, tier?,
    /// maxFeePerGas?, maxPriorityFeePerGas?, gasLimit?, nonce? }`. `amount` is base units,
    /// `amountUnits` is what the user typed in TOKEN units ("0.1" ETH, not 10^17 wei);
    /// exactly one of the two. `tokenAddress` names the contract exactly and wins over
    /// `token`, a symbol that is refused when two offered contracts share it. Any explicit
    /// fee field is used verbatim — the user overrules the suggestion, never the other way.
    ///
    /// Returns `{ ok, chainId, from, to, amount, amountDisplay, amountExact, amountSymbol,
    /// amountDecimals, nativeSymbol, token?, tokenAddress, nonce, gasLimit, maxFeePerGas,
    /// maxPriorityFeePerGas, maxCostWei(+Display/Exact), feeCeilingWei(+Display/Exact),
    /// feeSource, replaces, route, feeRoute }`. `feeCeilingWei` is `maxFeePerGas × gasLimit` —
    /// a ceiling, never a price, so a view must say "at most". `replaces` is the sender's word
    /// on a pinned nonce outbidding a transaction still pending there, else null. No approval
    /// is requested and no nonce is reserved, so it is safe to call on every keystroke.
    fn prepare_send(&self, request_json: String) -> String;

    /// Ask a human to approve a send. Takes the same `request_json` as `prepare_send`.
    ///
    /// Returns `{ ok, pending: true, requestId, handle }` and **never a transaction hash** —
    /// nothing has been signed or broadcast at this point. `tx_sender_module` reserved the
    /// nonce and registered the approval; the human approves in `evm_signer_ui`; drive the
    /// rest with `send_status`. `handle` is the KEYSTORE's name for the approval record, for
    /// pointing a signer at this specific request.
    fn send(&self, request_json: String) -> String;

    /// Advance a pending send and report where it got to. Poll this — the sender broadcasts
    /// on this call, exactly once, and records the row before the transaction leaves.
    /// `{ ok, requestId, handle, status, final, hash?, hashes, route?, reason?, origin,
    /// purpose, legs }` where `status` is `awaitingApproval` | `broadcasting` | `stuck` |
    /// `broadcast` | `rejected` | `cancelled` | `failed`. A reply carrying `blocked: true` is a
    /// send being HELD by the verified-proxy gate, not a failed one: keep polling, or
    /// `cancel_send`.
    ///
    /// Poll until `final` is true, refusals included: `{ ok: false, error, final: false }` is
    /// a poll that may yet succeed. `final` is the sender's own, read off `status` for a
    /// sender that predates it; a sender that did not answer is never final.
    fn send_status(&self, request_id: String) -> String;

    /// Withdraw a send that has not been approved yet, releasing its reserved nonce.
    fn cancel_send(&self, request_id: String) -> String;

    /// Re-read one recorded transaction's receipt on ITS OWN chain and update the stored
    /// status. `{ ok, hash, chainId, status, route }` — `pending` | `confirmed` | `failed`.
    fn refresh_tx_status(&self, address: String, hash_hex: String) -> String;

    /// The transaction- and block-level fields a RECEIPT does not carry, for one recorded
    /// transaction on ITS OWN chain. `{ ok, hash, chainId, route, fetchedAt, gasPriceUnit,
    /// block?, transaction?, blockError?, transactionError? }`; `ok` is true when EITHER leg
    /// landed. Every reply names the `hash` it is about, refusals included.
    fn get_tx_details(&self, address: String, hash_hex: String) -> String;

    /// Poll receipts for this address's still-pending transactions, on each row's OWN chain,
    /// and update their stored status. `{ ok, address, polled, changed, blocked,
    /// blockedChains, stillDue }`. `stillDue` is false once no row can move again, which is
    /// when a caller's poll timer should stop.
    fn refresh_pending(&self, address: String) -> String;

    fn on_context_ready(&self, _ctx: &RustModuleContext) {}
}
