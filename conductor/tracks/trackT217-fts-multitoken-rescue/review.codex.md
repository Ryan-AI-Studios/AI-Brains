## Verdict

Not complete. No P0 findings, but completion cannot be certified because required verification, manual dogfood, cross-model review, provenance checks, and track closeout remain incomplete.

## Scope

Audited the working tree on `feat/T217-fts-multitoken-rescue`, including implementation, tests, docs, `spec.md`, `plan.md`, `review.md`, and governance files. No files or Git state were modified.

## Requirement matrix

| Requirements | Result |
|---|---|
| D1–D12 | Implemented |
| D13 SQL LIMIT/privacy ordering | Implemented; R2-specific proof is insufficient |
| D14 underscore split | Implemented |
| F1–F18 | Implemented; testing gaps noted below |
| F19 LIMIT proof | Code implemented; regression proof incomplete |
| F20–F22 | Implemented |
| AC1–AC10 | Code/wiring satisfied; limited end-to-end proof |
| AC11 focused verification | User-reported green, not independently rerun |
| AC12 docs | Satisfied |
| AC13 full gate/review clean | Not satisfied |
| AC14 rescue=false forget behavior | Hermetically covered |
| AC15 LIMIT | SQL implementation correct; test only exercises R0 |
| AC16–AC17 | Pure tests present |

## Findings

### P1-01 — Required completion gates are still open

`plan.md` leaves focused verification, manual dogfood, full gate, cross-model review, PR/CI, and conductor closeout unchecked: [plan.md](C:/dev/AI-Brains/conductor/tracks/trackT217-fts-multitoken-rescue/plan.md:121).

The conductor still marks T217 `Planning`, and `spec.md` still says “Plan-only until go.”

Required: run and record all Phase 5 gates, update the track/conductor status, and complete independent/cross-model review.

### P1-02 — Ledger/provenance verification is unavailable

These read-only commands failed with `unable to open database file`:

- `ledgerful doctor`
- `ledgerful ledger status --compact`
- `ai-brains preflight --summary`

The plan claims these completed, but current provenance and pending-transaction state cannot be independently verified. `conductor/ISSUES.md` is also absent.

Required: repair/reconcile the ledger environment, then rerun and record `ledgerful verify --scope full` and ledger status.

### P2-01 — Required rescue test is untracked

`crates/ai-brains-retrieval/tests/lexical_rescue.rs` is untracked. It contains most of the claimed AC proof but will not be included unless explicitly staged/committed.

Required: ensure the test is intentionally included in the final change set.

### P2-02 — R2 LIMIT/privacy/scope regression proof is incomplete

The implementation correctly applies privacy predicates before `ORDER BY rank LIMIT` in [lexical.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/lexical.rs:127), and R2 uses the same `match_query` path. However, the LIMIT test only searches a single-token R0 query: [lexical_rescue.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/tests/lexical_rescue.rs:156).

There is no hermetic test proving that:

- R2 OR is bounded;
- sealed/NeverInject rows cannot consume the bound or appear;
- project/session scoping remains identical during rescue;
- recall wiring uses raw query plus rescue while bridge receives sanitized AND.

Required: add an R2-only fixture with broad matches, privacy-excluded rows, and scope separation.

### P2-03 — New test violates repository test conventions

The new LIMIT test uses a `for` loop at [lexical_rescue.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/tests/lexical_rescue.rs:166). Project rules require independent `rstest` cases rather than loops.

Required: replace the loop with compliant parameterized cases or explicit fixture setup.

### P3-01 — Changed documentation has trailing whitespace

`git diff --check` fails in the changed README/spec files. `cargo fmt --check` passes, but the working tree is not whitespace-clean.

Required: clean trailing whitespace before finalization.

## Completeness

The production implementation is substantially complete:

- R0/R1/R2 ladder is reachable.
- `recall_full` passes raw query, `rescue=true`, and `candidate_depth`.
- Bridge receives sanitized primary AND.
- Forget remains strict with `rescue=false`.
- SQL LIMIT is applied after SQL privacy filtering.
- No migration, dependency bump, semantic auto-enable, or capture-path dependency was introduced.
- Docs and public exports are aligned.
- No T217 production stubs or no-op paths found.

The completion process and proof package are incomplete.

## Wiring

Verified end-to-end statically:

`CLI recall → recall_full → bridge sanitized AND + lexical raw query → R0/R1/R2 → T105 fallback → semantic only when requested`

Forget uses `LexicalSearchOptions::default()` in both match and UUID-preview paths.

## Verification Evidence

- `cargo fmt --check`: passed.
- `cargo metadata --no-deps`: passed.
- `git diff --check`: failed.
- Ledgerful doctor/status and AI-Brains preflight: blocked by database-open failure.
- User-reported focused nextest/clippy results were not independently rerun.
- Full workspace gate has not been run.

## Deferred Candidates

None. The listed bridge/control-plane/NEAR/semantic residuals are valid non-goals, but the P1/P2 items above are completion blockers and should not be deferred.

## Completion Decision

Reject completion for now. The implementation looks functionally sound, but T217 should remain open until the untracked test, R2 privacy/LIMIT proof, test hygiene, ledger verification, manual dogfood, full gate, cross-model review, and track closeout are completed.