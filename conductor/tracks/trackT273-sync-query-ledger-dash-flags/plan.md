# T273 Plan — sync query dash-leading Ledgerful flags

**Status:** **Pending** (placeholder)
**Spec:** [spec.md](./spec.md)
**Category:** BUGFIX

- [ ] Full `/plan-track 273` before go (this is a mint stub from T268 last-PR Cursor)
- [ ] Re-read `sync_query_ledger.rs` `run_ledger_search` + live `ledgerful ledger search --help`
- [ ] Confirm Ledgerful accepts `--` before QUERY
- [ ] Red: dash-leading query is not parsed as a flag
- [ ] Green: POSIX `--` (or documented equivalent) before query
- [ ] T271 AC suite stays green (no T90 re-wrap)
- [ ] Gate + review
