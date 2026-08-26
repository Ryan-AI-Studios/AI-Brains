# T309 — rusqlite `Connection::table_exists`

- **Track ID:** T309-RusqliteTableExists
- **Status:** **Planned** (Pending until **go**)
- **Category:** CHORE / REFACTOR
- **Owner:** Grok
- **Source:** T213 L4 (deferred while pin was 0.39.0); T305 R2 — pin is now **0.40.2**.
- **Depends on:** T305 `#222`. [docs.rs 0.40 `table_exists`](https://docs.rs/rusqlite/0.40.0/rusqlite/functions/struct.ConnectionRef.html#method.table_exists): `conn.table_exists(None, name) -> Result<bool>`.
- **F0:** Plan-only until go.

## 1. Objective

Replace **production** sqlite_master existence probes in `has_core_tables` and `has_graph_tables` with `Connection::table_exists`. No pin bump. No unwrap. Test-local `fn table_exists` helpers may stay.

## 2. Live baseline (2026-08-26)

| Site | Probe |
|------|--------|
| `ai-brains-brain/src/backup.rs:615` `has_core_tables` | `sqlite_master` `events` + `memory_projection` |
| `ai-brains-cli/src/graph_density.rs:281` `has_graph_tables` | `sqlite_master` `graph_node` + `graph_edge` |
| encrypt/rotate/connection `SELECT count(*) FROM sqlite_master` | **Key/open probes — out of scope** (not table existence) |
| Store tests `fn table_exists` | File-local wrappers — **out of scope** unless compile-forced |

**Research:** rusqlite 0.40.2 `table_exists(db_name, table_name)`; `db_name = None` searches all. Returns `Result<bool>` — map err → false **or** propagate; **no** `unwrap`/`expect`. Snapshot — re-read docs.rs at execute.

last-PR `#222` Cursor **empty**. **T310** minted from T306 plan (daemon + T84), not Cursor.

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Only `has_core_tables` + `has_graph_tables` (T213 L4 named probes). |
| **F2** | Pin stays **0.40.2**. No clap/tokio/reqwest steal. |
| **F3** | No `unwrap`/`expect` on `table_exists`. |
| **F4** | Behavior: missing table → `false` (same as today’s `unwrap_or(false)`). |
| **F5** | Never `git push origin main`. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | `has_core_tables` / `has_graph_tables` call `table_exists`; no sqlite_master in those two fns. |
| **AC2** | Existing `has_graph_tables__*` + schema_readable / backup tests green. |
| **AC3** | encrypt/rotate sqlite_master **count** probes unchanged. |
| **AC4** | clippy `-D warnings` on touched crates; nextest those packages. |

## 5–12

**Non-goals:** Migrating every test helper; `column_exists`; T306 install.

**§9:** Absorb T213 L4 + T305 R2. Decline encrypt sqlite_master key probes. last-PR `#222` N/A.

**Touch:** `backup.rs`, `graph_density.rs`, tests if signatures force; CHANGELOG.

**Isolation:** Hermetic tempdir only.
