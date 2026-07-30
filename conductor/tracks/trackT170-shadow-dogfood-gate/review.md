# T170 Review Log

## R1 — Internal review (2026-07-30)

**Verdict:** NEEDS_FIX → fixed in R1 fix commit  
**Reviewer:** explore subagent (read-only)

| ID | Severity | Status | Summary |
|----|----------|--------|---------|
| R1-01 | medium | verified_fixed (R2) | Stage B fixture project_id → briefing |
| R1-02 | medium | verified_fixed (R2) | D20 evaluate report overwrite |
| R1-03 | medium | verified_fixed (R2) | Stage B evidence |
| R1-04 | medium | verified_fixed (R2) | plan.md Phase E honesty |
| R1-05 | medium | verified_fixed (R2) | BOM-less UTF-8 JSON writes |
| R1-06 | low | verified_fixed (R2) | claim_ids_sample sorted for hash |
| R1-07 | low | verified_fixed (R2) | migrate_report path wired |
| R1-08 | low | deferred | Optional Stage B seed integration assert |

## R2 — Internal re-review (2026-07-30)

**Verdict:** CLEAN  
All R1 mediums + actionable lows verified_fixed. Residual easy docs lows (manual `>` encoding, Stage A overwrite in runbook examples) fixed in follow-up docs commit before Codex.

## Codex R1 — External review (2026-07-30)

**Verdict:** FAIL → fixed in this commit  
**Source:** `review.codex.r1.md`

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| Codex-P1-01 | P1 | fixed_pending_verification | D24 honesty: live path + unreadable hashes → `live_checksum_unchanged=false`, `live_checksum_verified=false`, `D24_UNREADABLE` limitation; orchestrator exit non-zero. True N/A only when no live vault resolves. Evidence re-run with env cleared for honest N/A; fail-closed path smoke-tested with locked vault. |
| Codex-P1-02 | P1 | fixed_pending_verification | Stage A required: `-SkipEvaluate` needs `-AllowIncomplete` and exits 2; missing/unparseable evaluate report **throws**; missing `hard_gates_passed` fails; CLI `t169_passed` requires `hard == Some(true)` (no `unwrap_or(true)`). |
| Codex-P1-03 | P1 | fixed_pending_verification | `refuse_unsafe_dogfood_out_path`: refuse `.db`/`.sqlite`, live same-location, reparse/hardlink, existing without `--allow-out-overwrite`. Unit + integration tests. |
| Codex-P1-04 | P1 | fixed_pending_verification | Filled `evidence/stage-b-human-checklist.md` (synthetic seed ids, risk refs, sign-off); linked from `stage-b-notes.md`. |
| Codex-P2-01 | P2 | fixed_pending_verification | Stricter compare parse: require decisions/conclusions/warnings arrays; legacy `text` string; stage B\|C only; denied type check. Tests for rejection. |
| Codex-P2-02 | P2 | fixed_pending_verification | `-ProjectId` for Stage C; `stage-a-report-hash.txt` baseline + drift warn; runbook documents multi-project project-id requirement. |
| Codex-P2-03 | P2 | fixed_pending_verification | Runbook `Write-CliStdoutNoBom` no longer merges stderr (`2>&1` removed); matches orchestrator stdout-only pattern. |
| Codex-P2-04 | P2 | fixed_pending_verification | `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` clean; dogfood unit (23) + integration (5) tests pass. TDD red/green history residual noted as process-only (not product bug). Full workspace gate deferred to orchestrator. |

### Residuals

- Stage C still operator-deferred (no test vault in CI).
- Stage D deferred (no approval).
- Fixture Stage B briefing remains `denied=true` (grant gap honesty — not a regression).
- `$HOME` PS automatic-variable bug in live-resolve path fixed when env is unset (`$userHome`).
