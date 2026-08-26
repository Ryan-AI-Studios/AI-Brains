# T309 Plan — `table_exists`

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger (planning):** series DOCS `c62396f6`.
**Ledger (fold-in):** DOCS `04a90ce4-f45e-43ca-875a-f2d8324ff2a7`.
**Implement:** **CHORE** TX on go.

## Preflight (fold-in — 2026-08-26)

| Check | Result |
|-------|--------|
| HEAD / tree | `bf04c91` CLEAN; `main...origin/main` |
| Pin | rusqlite **0.40.2** (`Cargo.toml` + lock `23f2a97d…`) |
| `has_core_tables` | `backup.rs:615–631` sqlite_master + `.unwrap_or(false)` |
| `has_graph_tables` | `graph_density.rs:288–304`; docstring `:287` sqlite_master |
| Count probes | `backup.rs:252` / `:488` — **out of scope** |
| Graph units | `has_graph_tables__empty_db__false` `:646`; `__both_tables__true` `:652` |
| Core units | **none** — AC5 on go |
| docs.rs | `Connection::table_exists`; `ConnectionRef` **404** |
| Views | SQLite metadata API errors on views; probe `events_view=false` |
| Last PR Cursor | `#226` / `#225` comments **`[]`** — N/A; no T311 |
| Open PRs | none |
| Ledger | 0 pending / 0 drift at scan; this TX `04a90ce4` |
| `ISSUES.md` | **Does not exist** |

## Phase 0 (on go)

- [ ] Re-read [docs.rs rusqlite 0.40.2 `Connection::table_exists`](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.table_exists) (not ConnectionRef)
- [ ] Confirm lock still 0.40.2
- [ ] Rescan deferred + last-PR Cursor
- [ ] CHORE TX

## Tasks

- [ ] **Not a behavioral red.** Existing `has_graph_tables__*` + backup/doctor tests already pass on sqlite_master. AC1 red = those two fns still contain `sqlite_master`.
- [ ] AC5: `has_core_tables__empty_db__false` + `has_core_tables__both_tables__true` in `backup.rs` tests (in-memory; stay-green on current probe)
- [ ] Green: `table_exists(None, name).unwrap_or(false)` in both fns (F3/F4)
- [ ] F6: rewrite `has_graph_tables` docstring (and `has_core_tables` if it gains sqlite_master wording)
- [ ] Do not touch encrypt/rotate/backup count probes (`:252` / `:488`)
- [ ] CHANGELOG
- [ ] clippy `-D warnings` on `ai-brains-brain` + `ai-brains-cli`; nextest those packages
- [ ] PR → CI → squash (never `git push origin main`)

## DoD

- [ ] AC1–AC5; pin unchanged 0.40.2
- [ ] F6 docstring
- [ ] T310 / T307 not stolen
