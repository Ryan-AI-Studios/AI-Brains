# T320 Plan — unified status (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T320`.
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

## Phase 0 (on `/plan-track T320`)

- [ ] Inventory existing summary JSON keys (doctor, nightly, graph update, daemon)
- [ ] Name the command so it does not collide with `daemon status`
- [ ] Decide compose vs subprocess (prefer in-process helpers)

## DoD (after full plan + go)

One command returns four named sections; a single timeout does not blank the rest.

## Isolation

Do not grow `doctor.rs`. No `cargo install`. Never `git push origin main`.
