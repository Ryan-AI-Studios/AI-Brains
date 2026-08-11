PASS WITH DEFERRED P3 — F17 only.

Verified:

- Product commit remains `927b8db`; no product-code drift.
- T225 hard DoD and P2 fixes are satisfied.
- Governance reconciled: deferred item closed, ledger TX marked committed, P1-1 `verified_fixed`.
- Recorded full gate: 2521 tests passed; fmt, clippy, deny, and audit green.
- `cargo fmt --check` passes locally.

Targeted tests and Ledgerful re-checks were blocked by local access errors; no files were modified. The stale “final pending” sentence is superseded by the current PASS row and completion decision, not a new finding.