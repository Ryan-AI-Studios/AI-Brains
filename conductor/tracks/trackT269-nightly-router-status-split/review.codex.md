Product verdict: **PASS**

- **P0:** None.
- **P1:** None. CX1’s P1 items are process/publish gates and are excluded from this product DoD audit.
- **P2:** None. The CX1 adjacency fix is real: the hermetic test now requires `Nightly: AI-Brains-Nightly` at exactly the line after `=== Nightly Status ===`.
- **P3:** None.

Regression sweep:

- T247 `--quick` remains `probe=skipped` with no HTTP probe.
- T247 750 ms budget and parallel probes are unchanged.
- T255 JSON retains raw probe tokens and schema keys.
- T255 Router formatting and schedule helpers are unchanged.
- Human timeout labeling is exact-match only: `timeout → timeout (750ms)`.
- `after_help` remains additive and includes `TCP` and `/health`.

Evidence: [`nightly.rs`](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/nightly.rs:156), [`nightly_status.rs`](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/nightly_status.rs:8), [`nightly_status.rs` hermetic test](C:/dev/AI-Brains/crates/ai-brains-cli/tests/nightly_status.rs:88). Known targeted clippy, nextest, hermetic AC8, and manual AC10 checks are green.