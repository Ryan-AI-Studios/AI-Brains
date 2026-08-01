# T181 Review Log — Backup Recovery Drills

## R1 — Internal (explore subagent) — FAIL

| ID | Severity | Status | Notes |
|----|----------|--------|-------|
| T181-R-001 | medium | fixed → verified R2 | SQLCipher residual docs |
| T181-R-002 | medium | fixed → verified R2 | Vacuous matchers |
| T181-R-003 | medium | fixed → verified R2 | CLI secret helper |
| T181-R-004 | process | verified_fixed | Phase D closeout |
| T181-R-005 | low | deferred | rstest preference |
| T181-R-006 | low | deferred | duplicate dry-run |
| T181-R-007 | low | deferred | store Online Backup mirror |
| T181-R-008 | low | verified_fixed | status flip |

## R2 — Internal re-review — PASS WITH DEFERRED LOWS

Mediums R-001..003 **verified_fixed**. Lows deferred to residual list.

## Codex R1 — FAIL

| ID | Severity | Disposition |
|----|----------|-------------|
| P1 | process/DoD open | Fixed in closeout (gate + statuses) |
| P2 | E-01 `fs::copy` | **Fixed** — Online Backup API only |
| P3 | F34 byte-display | **Fixed** — Debug forms in helper |

## Codex R2 (final gate) — **PASS WITH DEFERRED P3**

- Prior P2/P3 fixed; no new >P3.
- Deferred P3: SQLCipher-inactive wrong-key residual (honest docs + dual-mode tests).
- See `review.codex.r2.md`.

## Gate evidence (2026-08-01)

```
cargo fmt --check → pass
cargo clippy --workspace --all-targets -- -D warnings → pass
cargo nextest run --workspace → 1704 passed, 1 skipped
cargo deny check → advisories/bans/licenses/sources ok
cargo audit → exit 0 (allowed warnings only)
```

Targeted recovery suites: crypto + store + cli recovery_drills → 20/20 (pre-full-gate).

## Residual list (deferred.md §59)

SQLCipher page encryption inactive (`bundled` plain SQLite); no recovery export/doctor CLI; Argon2 param opacity; #34.2; F-REC-03/04; daemon restore hard-fail; lows (rstest, BackupService mirror, duplicate dry-run).
