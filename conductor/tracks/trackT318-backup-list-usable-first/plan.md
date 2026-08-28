# T318 Plan — backup list usable-first (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T318`.
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

## Phase 0 (on `/plan-track T318`)

- [ ] Re-read `ListMode` / F6 / verify Default vs Verbose
- [ ] Dogfood `backup list` and `backup verify` (non-destructive)

## DoD (after full plan + go)

Default list leads with usable encrypted backups; residuals are a summary unless `--verbose`.

## Isolation

No live prune/create as planning. No `cargo install`. Never `git push origin main`.
