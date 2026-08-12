# T241 Review Log — Policy cold-start bootstrap

**Track:** T241-PolicyColdStartBootstrap  
**Category:** FEATURE / UX / GOVERNED  
**Review requirement:** Hard cross-model (F24) — doctor matrix + contracts `denial_hint`

---

## Findings

| ID | Severity | Status | Summary | Notes |
|----|----------|--------|---------|-------|
| H1 | high | verified_fixed | Stale exit_contract hermetic locked clap-required capability text | Renamed/rewrote to fail_usage catalog; forbids clap required-arg English |
| M1 | medium | verified_fixed | CLI-EXIT-CODES + CAPABILITIES still said clap-required capability | Updated to T241 optional + catalog fail_usage |
| CX1-P1 | high (process) | out_of_scope | Track not yet marked Completed / cross-model open at review time | Expected mid-loop; cleared by final CX + closeout |
| CX1-P2a | medium | verified_fixed | Explicit empty `--capability` collapsed to usage catalog | Only `None` → fail_usage; empty/whitespace → INVALID_PAYLOAD exit 6 + hermetic |
| CX1-P2b | medium | verified_fixed | Test coverage gaps (preflight wired, personal denial_hint, next_step omit) | Hermetics added; next_step omit requires key absent not null |
| CX2-H1 | high | verified_fixed | Preflight grants probe used ambient soft-resolve, ignoring `--project-id` | Probe uses `Repository:{project_id}`; hermetic clears env + flag path |
| L1 | low | deferred | after_help catalog string dual-site vs CAPABILITY_CATALOG | Sync comment added on main.rs; clap after_help cannot call runtime fn easily |
| L2 | low | deferred | Dual short-SOOT constants CLI vs CP | Substring `policy bootstrap` locked by tests; cross-crate const not free |

### Status legend

- `open` — not yet fixed  
- `fixed_pending_verification` — implementer claims fix; needs re-verify  
- `verified_fixed` — reviewer confirmed  
- `deferred` — medium/low only; justification + ISSUES.md  

---

## Review passes

| Pass | Reviewer | Date | Result |
|------|----------|------|--------|
| Internal R1 | explore subagent | 2026-08-12 | NEEDS_FIX (H1 high, M1 medium) |
| Internal R1 fix | orchestrator | 2026-08-12 | H1+M1 fixed; exit_contract + policy_bootstrap hermetics PASS |
| Internal R2 | explore subagent | 2026-08-12 | CLEAN |
| Cross-model CX1 | Codex gpt-5.4 high | 2026-08-12 | FAIL (P2 empty-cap + test coverage; P1 process) |
| Cross-model CX1 fix | orchestrator | 2026-08-12 | P2 fixed; hermetics PASS |
| Cross-model CX2 | Codex gpt-5.4 high | 2026-08-12 | FAIL (preflight `--project-id` scope mismatch) |
| Cross-model CX2 fix | orchestrator | 2026-08-12 | Probe uses summary project_id; hermetic flag path PASS |
| Cross-model CX3 | Codex gpt-5.4 high | 2026-08-12 | **PASS** (final clean gate; no findings) |

## Final decision

**PASS** — engineering DoD met; CX3 clean final cross-model; soft F20–F22 / L1–L2 deferred only.

---

## Notes

- No production `unwrap`/`expect` on capability Option.  
- Capture independence: recall/legacy preflight never require grants.  
- Live F25 dogfood recorded in plan.md evidence log.
