# T210 Review Log — Governed policy bootstrap for discovery

## Round 1 — Internal review (2026-08-05)

**Reviewer:** subagent (read-only)  
**Verdict:** NEEDS_FIX

### Findings

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| T210-R1 | medium | AC6 dry-run did not lock zero-append for `register_principal` (false-green) | verified_fixed |
| T210-R2 | low_info | `exit_contract` hint assert soft-OR (AC7 locked in policy_bootstrap) | verified_fixed |
| T210-R3 | low_info | No negative hermetic for Erase/Approve* after bootstrap | verified_fixed |
| T210-R4 | low_info | Soft-resolve success path not hermetically locked (AC8 fail path is DoD) | deferred — fail path meets AC8 |
| T210-R5 | low_info | Issued grant privacy not asserted | verified_fixed |

### Fixes applied (orchestrator)

1. **R1:** AC6 now re-runs dry-run (still `would_register`) + first real bootstrap must still `registered` (proves no dry-run register append).
2. **R2:** `exit_contract` requires `bootstrap` substring.
3. **R3/R5:** New `policy_bootstrap__after__dangerous_caps_still_denied` — Erase/Approve*/Export stay exit 3; show lists only three Read*; privacy LocalOnly.

### Evidence

```
cargo nextest run -p ai-brains-cli --test policy_bootstrap
# 9 passed
```

## Round 2 — Internal re-review (2026-08-05)

**Reviewer:** subagent (read-only)  
**Verdict:** CLEAN  
R1–R3/R5 verified_fixed; R4 deferred (AC8 fail path). No new findings.

## Cross-model Round 1 — Claude Sonnet (Codex rate-limited)

**Reviewer:** Claude Sonnet 4.6 read-only (Codex usage limit until ~2026-08-07)  
**Artifact:** `review.claude.md`  
**Verdict:** **PASS**

- No P0/P1/P2 findings.
- Optional P3 skill one-liner (F23 soft) — not DoD; leave deferred residual.
- Internal R1–R5 dispositions accepted.
- Fresh matrix: F1–F40 required + AC1–AC11 met.

### Manual evidence

```
cargo build -p ai-brains-cli
.\target\debug\ai-brains.exe policy bootstrap --scope Repository:441837f6-5c55-d075-0000-000000000000 --dry-run --format json
# exit 0; would_register + three would_issue; dry_run true
```

### Targeted gates observed

- `policy_bootstrap` 9/9
- `exit_contract` + `policy_bootstrap` 20/20
- `ai-brainsd` policy_denied units 2/2
- `cargo fmt --check` + workspace clippy `-D warnings` green

### Final decision

Engineering DoD clear for PR after full workspace nextest + deny + audit.

## Closeout (2026-08-05)

- PR #93 squash-merged: `d52df25`
- CI: gate-windows / gate-linux / gate-macos **pass** (run 30965515085)
- Full local nextest: **2090 passed**
- Cross-model final: Claude **PASS** (Codex rate-limited; fresh PASS is final gate)
- Conductor → Completed; deferred.md T210 struck
- Coordinated `coordination.md` T210 note appended
