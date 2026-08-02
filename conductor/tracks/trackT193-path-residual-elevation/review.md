# T193 Review Log — Path Residual Elevation

## Scope
- Branch: `track/T193-path-residual-elevation`
- Commits: `f45a37b` (B SOOT), `cb18d2b` (C P0), `a6b0233` (D/E P1+docs), + P3 polish
- Ledger TX: `a52b3a65-fe17-4553-a919-2494a1c56426` (SECURITY)

## Reviewers / rounds
| Round | Source | Verdict |
|-------|--------|---------|
| Internal R1 | explore subagent (DoD matrix) | **PASS WITH DEFERRED P3** |
| Security wire R1 | explore subagent (threat matrix) | **PASS** |
| P3 polish | orchestrator | ADR-0019 honesty; token pre-check fail-closed; token symlink FS test |
| Codex R1 | gpt-5.6-luna high | **FAIL** (P1 process pending; P2 kit leaf test; P3 whitespace) |
| Codex R1 fixes | orchestrator | Kit leaf AC13 test + whitespace; full gate observed green |
| Codex R2 | gpt-5.6-luna high | **PASS WITH DEFERRED P3** (ship-process only; engineering clean) |
| Ship | PR #77 squash-merge `2183127` | CI gate-linux/macos/windows **pass** |

## DoD summary (Internal R1 + Codex R1 fixes)
AC1–AC7, AC9–AC11, AC13–AC14 **Met**. AC5 kit leaf proof added. AC8 deferred strike + conductor Completed = ship process. AC12 engineering gate green locally; cross-model R2 final gate pending.

## Findings disposition

| ID | Sev | Status | Notes |
|----|-----|--------|-------|
| Token no dedicated symlink FS test | P3 | **verified_fixed** | `http_token_file__symlink_leaf__write_refuses_target_intact` |
| ADR-0019 residual #12 stale | P3 | **verified_fixed** | Short amend: T193 elevated token/artifact/kit |
| deferred.md not struck | P3 | **deferred → ship** | Orchestrator strikes on merge |
| Token pre-check unwrap_or(false) | P3 | **verified_fixed** | Fail-closed on metadata I/O Err |
| Codex T193-P2-001 kit leaf symlink proof | P2 | **verified_fixed** | `recovery_write_kit_file__symlink_leaf__refuses_target_intact` |
| Codex T193-P3-001 trailing whitespace | P3 | **verified_fixed** | ADR-0019 + spec.md |
| Codex T193-P1-001 process closeout | P1 process | **verified_fixed** | deferred struck; conductor Completed; PR #77 merged; ledger commit on closeout |
| Rename race replaces symlink entry | P2 residual | **accepted residual** | No target write-through; perfect TOCTOU non-claim (R-WIN-PERFECT) |
| Parent create_dir_all ambient | R | **residual** | F26 / R-WRITE-PARENT |

## Gates (local + CI, 2026-08-02)
- `cargo fmt --check` — pass
- `cargo clippy --workspace --all-targets -- -D warnings` — pass
- `cargo nextest run --workspace` — **1852 passed**, 1 skipped
- `cargo deny check` — pass
- `cargo audit` — pass (allowed warnings only)
- Targeted: token security 18/18; kit leaf symlink 1/1
- GitHub Actions PR #77 run `30764924158`: gate-windows / gate-linux / gate-macos **pass**

## Ship decision
**Completed.** Engineering DoD met; residual honesty retained (soft-canon, parent mkdir, P2 ambient long-tail, perfect Windows TOCTOU, soft-skip). Next residuals: T195 → T196.
