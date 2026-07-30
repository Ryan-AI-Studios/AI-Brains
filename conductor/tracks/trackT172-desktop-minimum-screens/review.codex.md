Verdict: **FAIL**

Working tree is clean; no files or Git state were modified.

### P0

None.

### P1

- **Live screens do not propagate scope, so core flows fail against the daemon.** T161 rejects missing scope for query, evidence, source, review list/resolve, and erasure. The `ScopeIndicator` resolves scope only for display; it is not wired into screen requests. Review has no scope field, Evidence/Source omit scope, and Erasure labels scope optional.

  Evidence: [ReviewScreen.tsx](<C:/dev/AI-Brains-wt-t172/apps/desktop/src/screens/ReviewScreen.tsx:27>), [EvidenceScreen.tsx](<C:/dev/AI-Brains-wt-t172/apps/desktop/src/screens/EvidenceScreen.tsx:22>), [SourceScreen.tsx](<C:/dev/AI-Brains-wt-t172/apps/desktop/src/screens/SourceScreen.tsx:22>), [T161 services.rs](<C:/dev/AI-Brains-wt-t172/crates/ai-brainsd/src/services.rs:420>).

  Impact: SC4 and SC5 are not end-to-end reachable; Review is not dogfood-usable; default erasure-ticket submission also fails.

- **Required claim route is mismatched.** Spec requires `#/claims/:kind/:id`; implementation registers only `#/claim/:kind/:id`. The required plural deep link falls through to the home redirect.

  Evidence: [spec.md](<C:/dev/AI-Brains-wt-t172/conductor/tracks/trackT172-desktop-minimum-screens/spec.md:142>), [App.tsx](<C:/dev/AI-Brains-wt-t172/apps/desktop/src/App.tsx:26>).

### P2

- **Tests do not prove the adapter path.** The `httpmock` tests construct separate raw reqwest clients and never call `post_json`, `get_json`, or the Tauri command handlers. Missing-token, authenticated adapter, 401 propagation, and frontend offline/denied behavior are untested. The required manual CSP/offline smoke evidence remains unchecked in [plan.md](<C:/dev/AI-Brains-wt-t172/conductor/tracks/trackT172-desktop-minimum-screens/plan.md:51>).

- **Token zeroization is incomplete.** `read_to_string` stores the bearer in an ordinary `String`, then clones it into `Zeroizing`; request-header formatting creates another ordinary copy. This does not leak the token to JS, but does not fully satisfy the repository’s sensitive-key zeroization mandate. [http_client.rs](<C:/dev/AI-Brains-wt-t172/apps/desktop/src-tauri/src/commands/http_client.rs:141>)

### P3

- Track closeout is incomplete: `plan.md` leaves manual smoke, `deferred.md` absorption, and ledger pinning unchecked; `conductor.md` still marks T172 **In Progress**. The review log references missing `review.codex.md` and `review.codex.r2.md`. [plan.md](<C:/dev/AI-Brains-wt-t172/conductor/tracks/trackT172-desktop-minimum-screens/plan.md:110>), [conductor.md](<C:/dev/AI-Brains-wt-t172/conductor/conductor.md:118>)

The reported package gates are positive, and invoke-first transport, production CSP strictness, retry disabling, capabilities, honest unavailable surfaces, and token non-exposure to JS are implemented. They do not overcome the P1 reachability defects and incomplete acceptance evidence.