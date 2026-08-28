# T313 Plan — sync query provenance (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T313`.
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

## Phase 0 (on `/plan-track T313`)

- [ ] Re-read `sync_query_ledger.rs` F7 banner + `sync.rs` `print_ledger`
- [ ] Dogfood `sync query "graph backend"` pretty vs pipe
- [ ] Confirm whether banner is stdout, stderr, or dropped on JSON

## DoD (after full plan + go)

Phrase-miss + token rescue cannot be mistaken for a phrase hit. Vault vs ledger sections labeled.

## Isolation

Do not grow `sync.rs` hotspot. No `cargo install`. Never `git push origin main`.
