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
| Internal R2 | — | — | pending after commit |
| Cross-model | — | — | pending (F24 hard) |

---

## Notes

- No production `unwrap`/`expect` on capability Option.  
- Capture independence: recall/legacy preflight never require grants.  
- Live F25 dogfood recorded in plan.md evidence log.
