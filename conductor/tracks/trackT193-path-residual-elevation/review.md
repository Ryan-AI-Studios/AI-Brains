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
| Codex cross-model | pending | — |

## DoD summary (Internal R1)
AC1–AC7, AC9–AC11, AC13–AC14 **Met**. AC8 deferred strike + AC12 full gate/cross-model = process (orchestrator).

## Findings disposition

| ID | Sev | Status | Notes |
|----|-----|--------|-------|
| Token no dedicated symlink FS test | P3 | **verified_fixed** | `http_token_file__symlink_leaf__write_refuses_target_intact` |
| ADR-0019 residual #12 stale | P3 | **verified_fixed** | Short amend: T193 elevated token/artifact/kit |
| deferred.md not struck | P3 | **deferred → ship** | Orchestrator strikes on merge |
| Token pre-check unwrap_or(false) | P3 | **verified_fixed** | Fail-closed on metadata I/O Err |
| Rename race replaces symlink entry | P2 residual | **accepted residual** | No target write-through; perfect TOCTOU non-claim (R-WIN-PERFECT) |
| Parent create_dir_all ambient | R | **residual** | F26 / R-WRITE-PARENT |

## Gates
- Targeted nextest (path+cli+api-server): 503 passed (implementer)
- Full workspace gate: pending before PR merge
- Codex: pending after polish commit
