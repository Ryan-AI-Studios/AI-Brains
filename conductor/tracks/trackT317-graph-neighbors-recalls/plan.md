# T317 Plan — neighbors RECALLS filter (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T317`.
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

## Phase 0 (on `/plan-track T317`)

- [ ] Dogfood `graph neighbors <live-id> --format human` and `graph hierarchy`
- [ ] Re-read T293 JSON freeze + T67 RECALLS meaning
- [ ] Decide cap vs exclude vs group without breaking rebuild tests

## DoD (after full plan + go)

Human neighbors of a real memory is not RECALLS-only spam. Empty hierarchy has a named next-step.

## Isolation

No live rebuild. No floor retune. No `cargo install`. Never `git push origin main`.
