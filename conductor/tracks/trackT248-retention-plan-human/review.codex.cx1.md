# T248 Independent Completion Audit

Scope: commit `920c78c` vs `origin/main` `008099a`, plus working-tree `spec.md`, `plan.md`, and conductor planning files.

## Verdict

**NOT CLEAR FOR COMPLETION**

The product implementation satisfies F1–F15 and AC1–AC15 based on the supplied gates and source audit. Completion is blocked by missing mandatory verification/provenance evidence and unfinished track finalization.

## P0

None.

## P1

### P1-1 — Mandatory full verification and ledger verification are not evidenced

The supplied checks cover targeted CLI/control-plane tests, clippy, hermetic tests, and live non-mutating behavior, but do not show:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
ledgerful verify --scope full
```

`ledgerful ledger status --compact` could not complete:

```text
rusqlite_migration error while executing query 'PRAGMA user_version;': unable to open database file
```

Required fix: run and record the mandatory full gate and ledger verification in a writable, healthy Ledgerful environment.

## P2

### P2-1 — Track finalization remains incomplete

The working-tree planning state is still `Planning`; all implementation and verification checklist items remain unchecked. There is no canonical:

```text
conductor/tracks/trackT248-retention-plan-human/review.md
```

The internal `review.internal.r1.md` and `review.internal.r1b.md` files are useful evidence but do not replace the required review log and conductor completion update.

Required fix: after verification, finalize the review log, check off `plan.md`, update conductor/deferred/series status to Completed, and close the Ledgerful transaction.

### P2-2 — Required skill documentation is only present in ignored local state

F14 requires the AI-Brains skill one-liner. It exists in the current ignored file:

[SKILL.md](C:\dev\AI-Brains\.agents\skills\ai-brains\SKILL.md:111)

However, `.agents/` is ignored and the path is not part of commit `920c78c`. The implementation commit therefore does not reproducibly deliver this required documentation.

Required fix: either deliver the required line through the project’s supported tracked documentation surface or explicitly revise F14 to classify this ignored local skill as non-deliverable.

## P3

### P3-1 — Apply `auto` wiring lacks an end-to-end regression test

The production path correctly forces apply through:

[retention.rs](C:\dev\AI-Brains\crates\ai-brains-cli\src\commands\retention.rs:81)

```rust
resolve_retention_format(&options.format, false)
```

The unit tests cover the resolver and the hermetic tests cover explicit `json` and `human`, but no test invokes `retention apply --confirm --format auto` and verifies JSON output. A future change to the apply call site could regress this while the current tests remain green.

This is non-blocking; add a focused regression test.

## Requirement audit

- **F1–F4:** Pass. Format parser, TTY plan behavior, apply JSON default, case-sensitive clap validation, and `is_tty: false` apply behavior are wired correctly.
- **F5–F6:** Pass. JSON DTO and keys remain unchanged; pretty output fills the nine-class matrix locally and preserves horizons.
- **F7–F11:** Pass. `memory_legacy` zero rows use `skip`; samples are joined safely; honesty labels, totals, ordering, and `next:` behavior match the spec.
- **F12–F13:** Pass. No new crates, DTO/API changes, planner rewrite, nightly restyle, or live apply.
- **F14:** Product documentation passes. Skill requirement is present only in ignored local state; see P2-2.
- **F15:** Supplied targeted suites pass.
- **F16–F18:** Correctly deferred residuals; no improper implementation scope detected.

## Verification evidence supplied

- CLI clippy: PASS
- CLI retention unit tests: PASS
- Hermetic retention tests: 5/5 PASS
- Control-plane class retention: 30/30 PASS
- CLI help tests: 7/7 PASS
- Live TTY pretty output: PASS
- Piped JSON parsing: PASS
- Invalid format exit 2: PASS
- Live `retention apply --confirm`: correctly not run

No implementation placeholders, production stubs, JSON schema drift, planner/contract rewrites, or capture-independence regressions were found.