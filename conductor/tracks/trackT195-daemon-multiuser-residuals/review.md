# T195 Review Log — Daemon Multi-User Residuals

## Scope

- Track: T195 Daemon Multi-User Residuals
- Branch: `agent/T195-daemon-multiuser-residuals`
- Category: SECURITY / ARCHITECTURE
- Ledger TX: `27065bef-3f35-4289-92c0-4fbd37c0aca7`

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| R1 | Internal (explore subagent) | **PASS WITH DEFERRED P3** | No P0–P2; three P3s |
| R1b | Orchestrator | Fixed easy P3s | CLI Protocol on empty path; `service_should_start_http` truth table |
| Codex R1 | gpt-5.6-luna | **FAIL** | P1 process closeout; P2 COMPATIBILITY stale `/tmp`; P3 foreign-owner test |
| Codex R2 | gpt-5.6-luna | **PASS WITH DEFERRED P3** | P2 fixed; P1 process residual to post-merge; P3 deferred |

## Findings

| ID | Sev | Status | Description |
|----|-----|--------|-------------|
| IR1-P3-01 | P3 | **verified_fixed** | CLI invalid socket override → empty path then NotRunning. Fixed: `request_unix` returns `Protocol` config error when path empty. |
| IR1-P3-02 | P3 | **verified_fixed** | Service HTTP gate composition only pure helpers. Fixed: `service_should_start_http` + truth-table unit test; `windows_service` uses it. |
| IR1-P3-03 / P3-01 | P3 | **deferred** | Foreign-owner UDS unlink not unit-tested (chown hard in unelevated CI). Code path present; file/dir refuse tests cover fail-closed. |
| Codex-P2-01 | P2 | **verified_fixed** | COMPATIBILITY/INSTALL XDG-first UDS docs (commit `44ce011`). |
| Codex-P1-01 | P1 process | **ship residual** | AC8 deferred strike + conductor Completed + ledger commit after CI/merge. |

## Soft residual

- **F13 Host-header rebinding:** not implemented (soft; not DoD). Bearer + loopback primary remain.

## Local full gate (orchestrator)

`cargo fmt --check` + `clippy -D warnings` + `nextest --workspace` **1870 passed** (1 skipped) + `deny` + `audit` → **FULL_GATE_CORE_OK**

## Ship process (orchestrator)

- AC8 deferred.md strike + conductor Completed — after CI green + merge
- Codex R2 engineering clearance: **PASS WITH DEFERRED P3**
