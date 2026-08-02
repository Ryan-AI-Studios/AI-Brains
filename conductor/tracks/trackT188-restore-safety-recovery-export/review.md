# T188 Canonical Review — Restore Safety + Recovery Operator Surface

**Track:** T188-RestoreSafetyRecoveryExport  
**Date:** 2026-08-02  
**Branch:** `agent/T188-restore-safety-recovery-export`  
**Final engineering verdict:** **PASS WITH DEFERRED P3** (product complete; deferred P3 residuals only)

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| Internal R1 | `review.internal.r1.md` | PASS WITH DEFERRED P3 |
| Internal fixes | event assert + DECISION pin honesty | P3-2/P3-3 fixed |
| Codex R1 | `review.codex.r1.md` | FAIL (P1 process closeout + P2 F8b symlink) |
| Fix Codex P2 | reparse/symlink refuse on passphrase-file + kit output | P2 fixed |
| Local full gate | fmt / clippy -D warnings / nextest **1749** / deny / audit | **green** |
| Codex R2 | `review.codex.r2.md` (final gate) | pending at write time → re-run after closeout commit |

## DoD matrix (AC1–AC14)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 hard-fail no overwrite | Met | unit `backup_restore__daemon_running__fails_no_overwrite` |
| AC2 daemon down + force | Met | unit + integration |
| AC3 dry-run notice | Met* | printed; *P3-1 stdout capture residual |
| AC4 unlockable kit + schema_version=1 | Met | recovery_drills + crypto tests |
| AC5 leakage | Met | assert_no_secret_leakage + no kit dump |
| AC6 0600 / public refuse | Met | code + Windows unit |
| AC7 R-DOC-CLI partial | Met | export yes, doctor no |
| AC8 rpassword only + deny | Met | 7.5.4 Apache-2.0 |
| AC9 full gate + SECURITY | Met (local) | 1749 nextest; CI on PR |
| AC10 deferred #1/#6 struck | Met | deferred.md promotion table + §59 residuals + §66 |
| AC11 T181-F-03 hard-fail | Met | RECOVERY-DRILLS |
| AC12 no key material in errors | Met | unit test |
| AC13 no migrate while daemon up | Met | pre-AppContext export; open_without_migrate |
| AC14 output exists refuse | Met | unit + integration |

## Findings disposition

| ID | Sev | Disposition |
|----|-----|-------------|
| Internal P3-1 dry-run stdout capture | P3 | **deferred** (test hardening) |
| Internal P3-2 RecoveryKitCreated assert | P3 | **verified_fixed** (unit test) |
| Internal P3-3 DECISION pin | P3 | **verified_fixed** |
| Internal P3-4 live-daemon drill | P3 | **deferred** (unit-injected covers safety) |
| Internal P3-5 AppContext before restore probe | P3 | **deferred** (export F16b only; overwrite still blocked) |
| Codex R1 P1 process closeout | P1 | **fixed** by AC9 gate + AC10 deferred/conductor |
| Codex R1 P2 F8b symlink | P2 | **verified_fixed** (`ai_brains_path` reparse refuse) |

## Deferred residuals (append-only register)

See `conductor/deferred.md` §66. Doctor (#2) remains open. No new mediums.

## Gate evidence (local 2026-08-02)

```text
cargo fmt --check                          OK
cargo clippy --workspace --all-targets -- -D warnings  OK
AI_BRAINS_ALLOW_ZERO_KEY=1 cargo nextest run --workspace --profile ci
  → 1749 passed, 1 skipped
cargo deny check                           OK (exit 0; pre-existing wildcard/duplicate warnings)
cargo audit                                OK (exit 0)
```

## Completion decision

Engineering DoD met for product ACs. Governance closeout (conductor ✅, deferred #1/#6 struck) included in this track. Fresh Codex R2 is the final cross-model gate before merge.
