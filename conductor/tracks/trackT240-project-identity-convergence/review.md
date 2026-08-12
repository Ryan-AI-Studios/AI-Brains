# T240 Review Log — Project identity convergence

## Scope
- Branch: `agent/T240-project-identity-convergence`
- Product: whoami + path-first detect + once mismatch warn + doctor soft `project_identity` + docs

## Internal review rounds

### R1 — Spec completeness + correctness (explore subagents)
- **Verdict:** CLEAN (no open P0–P2 product defects)
- **Findings fixed:**
  - P3 Detect clap one-liner stale → updated path-first help + after_help
  - P2/P3 AC9 subdir hermetic + F6 0-mem e2e note hermetic added
- **Deferred process:** F13/F14 soft (already plan), F22 Codex, AC6 operator rebind, full gate

### Disposition table

| ID | Severity | Status | Notes |
|----|----------|--------|-------|
| Detect help path-first | P3 | verified_fixed | main.rs Detect doc |
| AC9 subdir hermetic | P2 test gap | verified_fixed | project_identity_convergence |
| F6 0-mem hermetic | P3 | verified_fixed | same suite |
| F13 detect --json | — | deferred | not DoD |
| F14 project use | — | deferred | not DoD |

## Cross-model
- Pending Codex (F22 required).

## Gates
- Targeted: clippy ai-brains-cli + project_* nextest (implementer) PASS
- Full workspace gate: pending before PR merge
