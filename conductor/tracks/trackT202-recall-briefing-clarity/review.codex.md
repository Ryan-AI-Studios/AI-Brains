# Independent Completion Review

Verdict: **Not complete — needs fixes.** No P0 findings.

## P1

### P1-01 — Embedding diagnostics can leak secrets

`embedding_endpoint()` exposes the raw `AI_BRAINS_EMBEDDING_URL`, while `detail` contains raw provider/HTTP error text. These values reach JSON, TTY output, and tracing logs.

Evidence: [semantic.rs:25](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/semantic.rs:25), [semantic.rs:41](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/semantic.rs:41), [semantic.rs:190](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/semantic.rs:190).

Required fix: sanitize endpoint to scheme/host/port only and replace details with stable, non-sensitive classifications. Add tests covering credentials, query strings, paths, and provider response bodies.

### P1-02 — Error classification is not type-safe

`classify_embedding_error` scans the complete formatted error string. A provider HTTP 500 response containing “timeout” or “connection refused” can therefore be incorrectly reported as `unreachable` instead of `error`.

Evidence: [semantic.rs:41](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/semantic.rs:41), [semantic.rs:56](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/semantic.rs:56), [semantic.rs:201](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/semantic.rs:201).

Required fix: preserve the typed `ModelError` class through the retrieval boundary, and test collision cases.

## P2

### P2-01 — Budgeting can erase mandatory denied warnings

The new seed warning is removed by normal word-budget truncation. For example, a denied briefing with `--max-words 1` can return `denied=true` with no `warnings[].kind="denied"`.

Evidence: [budget.rs:89](C:/dev/AI-Brains/crates/ai-brains-control-plane/src/briefings/budget.rs:89), [budget.rs:176](C:/dev/AI-Brains/crates/ai-brains-control-plane/src/briefings/budget.rs:176).

Required fix: preserve at least one denied warning, even when the budget is too small, and add low-budget tests for both packet types.

### P2-02 — Required behavioral proof is incomplete and not fully hermetic

The T202 integration suite has only five tests. It lacks end-to-end proof for healthy `status=ok`, `no_stored_embeddings`, actual model propagation, and briefing TTY output. The connection-refused test relies on port 1 being unused.

Evidence: [recall_briefing_clarity.rs](C:/dev/AI-Brains/crates/ai-brains-cli/tests/recall_briefing_clarity.rs:1).

Required fix: add deterministic loopback/mock coverage for AC3, AC3b, AC5, and AC8; avoid fixed-port assumptions.

### P2-03 — F9 help/documentation remains inaccurate

`ai-brains briefing --help` still says “JSON stdout by default” at [main.rs:383](C:/dev/AI-Brains/crates/ai-brains-cli/src/main.rs:383), contradicting TTY markdown behavior. OPERATIONS also describes a combined briefing/query surface as TTY-markdown-capable even though progressive/expand remain JSON-only.

Required fix: update the parent help text and scope the format statement to briefing commands.

### P2-04 — Track Definition of Done and provenance are still open

`plan.md` leaves E4–E7 unchecked; the conductor marks T202 Proposed; the deferred row remains active; TX `82e6e899` is open.

Evidence: [plan.md:88](C:/dev/AI-Brains/conductor/tracks/trackT202-recall-briefing-clarity/plan.md:88), [conductor.md:148](C:/dev/AI-Brains/conductor/conductor.md:148), [deferred.md:29](C:/dev/AI-Brains/conductor/deferred.md:29).

`ledgerful verify` could not be run because Ledgerful’s database was inaccessible.

## P3

None proposed for deferral.

## Verification

- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- Prebuilt contracts tests: 38 passed.
- Dynamic Cargo tests were blocked by read-only access to `target\debug\.cargo-lock`.
- T202/CP runtime tests were blocked by sandbox denial when creating temporary vaults.
- No ranking changes, T203 scope work, daemon semantic work, or production stubs were found.