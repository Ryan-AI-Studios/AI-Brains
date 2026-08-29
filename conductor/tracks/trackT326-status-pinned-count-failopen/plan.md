# T326 Plan — status/graph PinnedCountFailed fail-open (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T326`.
**Ledger (planning):** minted with T316 DOCS `66b597f7-faf9-4f3e-bb06-6af72811bdc6`

## Phase 0 (on `/plan-track T326`)

- [ ] Re-read `status.rs:329–340` vs doctor skip `:901–904`
- [ ] Re-read `graph.rs:445–458` same arm
- [ ] Re-read `graph_density.rs` `GatherResult::PinnedCountFailed` docs `:63–68` / gather `:321–333`
- [ ] Confirm Cursor `#237` still true on that day’s HEAD
- [ ] Hermetic: inject `PinnedCountFailed` → glance graph `error`, not `pinned=0` + assess

## DoD (after full plan + go)

Pinned COUNT fail does not invent `pinned=0`. Doctor skip stays. Floors frozen. T316 not stolen.

## Isolation

Do not implement from T316. No floor retune. No `cargo install`. Never `git push origin main`.
