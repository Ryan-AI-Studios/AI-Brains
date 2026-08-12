# T240 Review Log — Project identity convergence

## Scope
- Product squash: PR #144 → `29b9b59` on `main`
- Product: whoami + path-first detect + once mismatch warn + doctor soft `project_identity` + docs

## Internal review rounds

### R1 — Spec completeness + correctness (explore subagents)
- **Verdict:** CLEAN (no open P0–P2 product defects)
- **Findings fixed:**
  - P3 Detect clap one-liner stale → path-first help + after_help
  - P2/P3 AC9 subdir hermetic + F6 0-mem e2e note hermetic added

### Disposition table

| ID | Severity | Status | Notes |
|----|----------|--------|-------|
| Detect help path-first | P3 | verified_fixed | main.rs Detect doc |
| AC9 subdir hermetic | P2 test gap | verified_fixed | project_identity_convergence |
| F6 0-mem hermetic | P3 | verified_fixed | same suite |
| F13 detect --json | — | deferred | not DoD |
| F14 project use | — | deferred | not DoD |

## Cross-model Codex

### R1 (`review.codex.md`) — FAIL process P2 (no code P0–P1)
1. P2 governance stale → fixed (Implementing + evidence)
2. P2 AC6 unproven → fixed (temp rebind live proof; restored)

### R2 (`review.codex.r2.md`) — no P0–P2
- Only P3 trailing whitespace → fixed

### Final (`review.codex.final.md`) — **PASS**
- P0–P3: none

## Gates (observed)
- Local: fmt, clippy -D, nextest **2647**, deny, audit allowed warnings
- Hermetics: project_identity_convergence 9/9; T206 honesty green
- Live: whoami triangle; detect → path owner 7d97a456; AC6 rebind Scope main / mismatch false
- CI: PR #144 Win/Linux/macOS green (run 31550998087)
- Codex final: **PASS**

## Soft deferred (not DoD)
- F13 detect `--json` schemaVersion + source
- F14 `project use`

## Completion decision
**Completed** after final Codex PASS, CI green, squash merge `29b9b59`, conductor/deferred/coordinated updates, pin.
