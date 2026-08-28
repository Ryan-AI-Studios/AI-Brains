# T322 Plan — decision as-of (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T322`.
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

## Phase 0 (on `/plan-track T322`)

- [ ] Re-read T311 `in_force.rs` + `decision_valid_at`
- [ ] Prove whether `updated_at` is sufficient for as-of (R4)

## DoD (after full plan + go)

`--as-of` returns the historically ruling Approved (or none). Default now unchanged.

## Isolation

No daemon DTO. No H2. No `cargo install`. Never `git push origin main`.
