# T221 Review Log — Governed first-run + deny exit honesty

**Track:** T221-GovernedFirstRunDenyExit  
**Category:** FEATURE / CONTRACT / BUGFIX  
**Ledger TX:** `eef6a954-03c8-4166-a07b-691cb6235b83`

## Rounds

| Round | Source | Verdict | Notes |
|-------|--------|---------|-------|
| R1 | Internal subagent | NEEDS_FIX | P2 AC5 Denied hermetic missing |
| R2 | Internal re-review | CLEAN | Expand Denied hermetic verified; 9/9 nextest |
| R3 | Codex gpt-5.4 high | **Product PASS** | No P0–P3 product findings; process-only open gates (DoD checkboxes / deferred strike / full CI) owned by orchestrator closeout |
| R4 | Closeout | **Completed** | PR #114 squash `b3c4b0f`; CI green; conductor/deferred/series/coordinated updated; soft residuals only |
| R5 | Codex final (gpt-5.4 high) | **PASS WITH DEFERRED P3** | Fresh post-merge product audit: no open P0–P2; deferred soft F12/F32/F18/F36 |

## Findings

### R1-P2 — AC5 expand Denied untested
- **Severity:** medium
- **Status:** `verified_fixed`
- **Description:** `run_expand` maps `kind == "Denied"` → exit 3, but suite only locked Unknown → 0.
- **Fix:** Hermetic `expand__seeded_no_grants__exit_3_kind_denied` seeds in-scope evidence via AllowAll, expands without Read* grants, asserts exit 3 + kind Denied + CODE/bootstrap stderr.
- **Evidence:** `crates/ai-brains-cli/tests/governed_first_run_deny_exit.rs`; nextest 9/9 including this case.

### Codex process notes (not product defects)
- Open DoD checkboxes / full gate / deferred strike until PR merge + closeout (expected mid-track).
- Series README / deferred still said Planning during In Progress — updated at closeout.

## Soft residuals (not open findings)

| Item | Disposition |
|------|-------------|
| F12 doctor `policy_grants` | Soft skip — matrix/cwd risk (M4) |
| F32 `--principal-id` progressive/expand | Soft skip — not DoD |
| F18 daemon HTTP 200+denied | Document residual |
| F36 trace `applied_policy` string | Out of DoD |
| Triple-site POLICY_DENIED_HINT drift | Dual-site comments; CLI backfill |

## Explicit non-findings

- Briefing soft exit 0 kept (AC10)
- Progressive packet preserved on deny (no fail_api replace)
- Unknown expand stays exit 0
- AC3 System principal (bootstrap omits `--principal-id`)
- Progressive/expand CP Err via `fail_cp`
- No production unwrap on new paths
- Contract `denial_hint` additive serde defaults
