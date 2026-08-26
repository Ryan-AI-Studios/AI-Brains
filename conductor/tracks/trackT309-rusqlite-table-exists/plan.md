# T309 Plan — `table_exists`

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `c62396f6`. Implement **CHORE** on go.

## Phase 0

- [ ] Re-read docs.rs rusqlite 0.40.2 `Connection::table_exists`
- [ ] Confirm lock still 0.40.2
- [ ] CHORE TX

## Tasks

- [ ] Red: `has_core_tables` / `has_graph_tables` tests still fail if sqlite_master string remains required
- [ ] Green: `table_exists(None, name)` with F4 false-on-err
- [ ] Do not touch encrypt/rotate count probes
- [ ] CHANGELOG
- [ ] PR → CI → squash (never `git push origin main`)

## DoD

- [ ] AC1–AC4; pin unchanged 0.40.2
