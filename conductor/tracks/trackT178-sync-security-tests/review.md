# T178 Review Log — Sync Security Tests

## Scope
P11.3 security suite: F1–F28, Must-id matrix, WRAP KATs, F19 side-effects, honesty gates.

## Rounds

### Internal R1 (2026-07-31) — dual explore subagents
**Verdict: NEEDS_FIX** → all High/Medium **verified_fixed** in R2.

| ID | Sev | Title | Status |
|----|-----|-------|--------|
| IR1-H1 | High | Seeded wrap helper is public crate API (F20 containment) | verified_fixed |
| IR1-M1–M8 | Medium | L3/L7/L4/F19/F27 completeness | verified_fixed |
| IR1-L1 | Low | R-ack-attestation behavioral thin | deferred |

### Internal R2 (2026-07-31)
**Verdict: CLEAN WITH DEFERRED LOWS** — no open Medium/High.

### Codex R1 (2026-07-31)
**Verdict: FAIL** (`review.codex.r1.md`)

| ID | Sev | Title | Status |
|----|-----|-------|--------|
| CR1-P1 | P1 | Multi-device revoke omit wrap rows | verified_fixed |
| CR1-P2 | P2 | Revoke-past must AEAD-open plaintext on B | verified_fixed |
| CR1-DoD | Process | Full gate incomplete at time of R1 | closed after gate |

### Codex R2 (2026-07-31)
**Verdict: FAIL** on process DoD only; engineering CR1-P1/P2 **verified**. (`review.codex.r2.md`)
- P3 F21 Cargo.toml-only gate (non-blocking; direct dep parse; `cargo tree -p ai-brains-capture` clean)

### Gate evidence (2026-07-31)
```
cargo clippy --workspace --all-targets -- -D warnings  → clean
cargo nextest run --workspace --no-fail-fast          → 1651 passed, 1 skipped
cargo deny check                                       → advisories/bans/licenses/sources ok
cargo audit                                            → 19 allowed warnings (pre-existing)
cargo fmt --check                                      → pre-existing Windows newline_style vs LF workspace noise (unchanged on main)
```
Ambient `AI-Brains-Daemon` Session-0 process can flake `cli_erasure_request__daemon_down__exit_code_5` (POLICY_DENIED vs exit 5); stopped service for gate; **unrelated to T178**.

### Codex R3 (final gate, 2026-07-31)
**Verdict: PASS WITH DEFERRED P3** (`review.codex.r3.md`)

| ID | Sev | Disposition |
|----|-----|-------------|
| CR1-P1 / CR1-P2 | — | verified fixed |
| CR2-P3 F21 Cargo.toml-only | P3 | deferred → deferred.md §55 |
| Engineering Must matrix | — | met |

Fresh final cross-model review clean above Low (only deferred P3).

## Deferred lows → deferred.md §55
- IR1-L1 / R2-L1 / R2-L2 ceremony / attestation thinness
- CR2-P3 F21 transitive edge residual
