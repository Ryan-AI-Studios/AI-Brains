# T309 — rusqlite `Connection::table_exists`

- **Track ID:** T309-RusqliteTableExists
- **Status:** **Planned** (Pending until **go**)
- **Category:** CHORE / REFACTOR
- **Owner:** Grok
- **Source:** T213 L4 (deferred while pin was 0.39.0); T305 R2 — pin is now **0.40.2**.
- **Depends on:** T305 `#222`. [docs.rs 0.40.2 `Connection::table_exists`](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.table_exists): `conn.table_exists(None, name) -> Result<bool>`.
- **F0:** Plan-only until go.
- **AI fold-in:** 2026-08-26 `agy-review.md` + `opencode-review.md` (HEAD `bf04c91`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** Disposition **§13**. Fold-in DOCS TX `04a90ce4-f45e-43ca-875a-f2d8324ff2a7`.
- **Ledger:** series DOCS `c62396f6`. Implement **CHORE** on go.

## 1. Objective

Replace **production** sqlite_master existence probes in `has_core_tables` and `has_graph_tables` with `Connection::table_exists`. No pin bump. No unwrap. Test-local `fn table_exists` helpers may stay.

## 2. Live baseline (fold-in re-verify 2026-08-26)

| Site | Probe |
|------|--------|
| `ai-brains-brain/src/backup.rs:615–631` `has_core_tables` | `sqlite_master` `type='table'` `events` + `memory_projection`; `.unwrap_or(false)` |
| `ai-brains-cli/src/graph_density.rs:288–304` `has_graph_tables` | same pattern `graph_node` + `graph_edge`; docstring **`:287`** still says sqlite_master |
| `backup.rs:252` / `:488` `SELECT count(*) FROM sqlite_master` | **Key/open probes — out of scope** |
| Store tests `fn table_exists` | File-local wrappers — **out of scope** unless compile-forced |
| `has_graph_tables__empty_db__false` / `__both_tables__true` | `graph_density.rs:646` / `:652` — stay-green |
| `has_core_tables__*` | **Absent** — coverage via `classify_backup_read__*` + doctor `schema_readable` |

**Research (snapshot — re-read docs.rs at execute):**

- Pin: workspace `Cargo.toml` rusqlite **0.40.2**; lock checksum `23f2a97d…`.
- docs.rs 0.40.2 method lives on **`Connection`**, not the deprecated `ConnectionRef` alias (that docs page **404s**). Signature: `pub fn table_exists<N: Name>(&self, db_name: Option<N>, table_name: N) -> Result<bool>`. `db_name = None` searches all.
- SQLite [`sqlite3_table_column_metadata`](https://www.sqlite.org/c3ref/table_column_metadata.html): NULL column name = table existence; **views return an error**. OpenCode throwaway probe (rusqlite 0.40.2 bundled): `table_exists(events)=true`, `table_exists(events_view)=false`, `table_exists(nope)=false` — identical to today's `type='table'` probe. **F4 holds.**

**last-PR Cursor:** [#226](https://github.com/Ryan-AI-Studios/AI-Brains/pull/226) (merged 2026-08-26T17:22:22Z) + [#225](https://github.com/Ryan-AI-Studios/AI-Brains/pull/225). `pulls/226/comments` and `pulls/225/comments` both **`[]`**. Plan-write cited `#222` (stale). Disposition unchanged: **N/A empty. No T311.**

HEAD fold-in: `bf04c91` `main...origin/main` CLEAN.

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Only `has_core_tables` + `has_graph_tables` (T213 L4 named probes). |
| **F2** | Pin stays **0.40.2**. No clap/tokio/reqwest steal. |
| **F3** | No `unwrap`/`expect` on `table_exists`. Map `Result` with `.unwrap_or(false)`. |
| **F4** | Behavior: missing table **or** view **or** probe err → `false` (same as today’s `type='table'` + `.unwrap_or(false)`). |
| **F5** | Never `git push origin main`. |
| **F6** | After green, the two fns’ **doc comments** must not say `sqlite_master`. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | `has_core_tables` / `has_graph_tables` call `table_exists`; **no** `sqlite_master` string in those two fns **or their doc comments**. |
| **AC2** | Existing `has_graph_tables__*` + schema_readable / backup tests green. |
| **AC3** | encrypt/rotate/backup `SELECT count(*) FROM sqlite_master` key probes unchanged (`backup.rs:252` / `:488` included). |
| **AC4** | clippy `-D warnings` on touched crates; nextest those packages. |
| **AC5** | New hermetic units in `backup.rs` tests: `has_core_tables__empty_db__false` and `has_core_tables__both_tables__true` (in-memory; same shape as `has_graph_tables__*`). **Regression net — not a behavioral red** (current sqlite_master probe already satisfies them). |

## 5. Design notes

```rust
pub fn has_core_tables(conn: &rusqlite::Connection) -> bool {
    conn.table_exists(None, "events").unwrap_or(false)
        && conn.table_exists(None, "memory_projection").unwrap_or(false)
}
```

Same for `has_graph_tables` with `graph_node` / `graph_edge`. `None` = search all attached DBs (docs.rs).

## 6. Non-goals

Migrating every test helper; `column_exists`; T306 install; T307 dual tower-http; T310 `run_update` / PATH daemon; clap 5.

## 7. Verification plan

TDD: **no behavioral red** distinguishes `table_exists` from the current probe (OpenCode m3; empirically same on missing table and view). On go:

1. **Red = source-level:** the two fns still contain `sqlite_master` (AC1 fail).
2. **Green:** `table_exists` + F6 docstring + AC5 tests (may be written green; they lock F4).
3. Stay-green AC2/AC3.

## 8. Risk

Low. Internal refactor. Capture-independent. Contracts/events untouched. View mismatch was the only semantic risk — **disproven** (SQLite docs + probe).

## 9. Deferred absorb/decline

| Item | Disposition |
|------|-------------|
| T213 L4 / T305 R2 `table_exists` | **Absorb** F1 / AC1 / AC2 |
| SQLCipher `SELECT count(*) FROM sqlite_master` key probes | **Decline** AC3 / F1 |
| Test-local `fn table_exists` helpers | **Decline** (non-goal) |
| T310 `run_update` + PATH daemon 4.14 | **Decline steal → T310** |
| T307 dual tower-http | **Not stolen** (Blocked) |
| last-PR Cursor `#226` / `#225` | **N/A empty** — no T311 |
| clap 5 | **Decline** |

## 10. Implement order (on go)

1. Phase 0: re-read docs.rs `Connection::table_exists`; confirm lock **0.40.2**; CHORE TX.
2. AC5 tests (stay-green on current probe).
3. Green AC1/F6.
4. Stay-green AC2/AC3; AC4 clippy + nextest.
5. CHANGELOG. PR → CI → squash. Never `git push origin main`.

## 11. Soft residuals

| Residual | Note |
|----------|------|
| Test-local `fn table_exists` helpers still sqlite_master | **By design** non-goal |
| PATH binary until `cargo install` | Soft — source SoT |
| T310 placeholder | Not stolen |

## 12. Touch map

| Path | Role |
|------|------|
| `crates/ai-brains-brain/src/backup.rs` | `has_core_tables` + AC5 tests; **do not** touch `:252`/`:488` |
| `crates/ai-brains-cli/src/graph_density.rs` | `has_graph_tables` + F6 docstring; stay-green `__*` |
| `CHANGELOG.md` | Unreleased Changed |
| `conductor/conductor.md` | Pending until go |

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` (HEAD `bf04c91`). Fold-in verify: `has_core_tables` `backup.rs:615–631`; `has_graph_tables` `graph_density.rs:288–304` docstring `:287`; tests `:646`/`:652`; no `has_core_tables__*` in `backup.rs` tests; `backup.rs:252`/`:488` count probes; rusqlite **0.40.2**; docs.rs `Connection::table_exists` (ConnectionRef **404**); `#226`/`#225` comments `[]`.

### Pins locked by fold-in

1. **Live lines (OpenCode m1):** `has_graph_tables` is **`:288`**, not `:281`.
2. **last-PR (OpenCode m2):** `#226` (and `#225`) empty; **no T311**. `#222` was stale.
3. **No behavioral red (OpenCode m3):** AC1 is source-level; AC5 is regression net.
4. **docs.rs (OpenCode m4):** `Connection`, not `ConnectionRef`.
5. **Views (OpenCode O1):** `table_exists` false for views — F4.
6. **Agy m1:** AC5 named `has_core_tables__*` units.
7. **Agy m2:** F6 docstring.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** hermetic `has_core_tables__*` | **Folded** AC5 |
| Agy | **m2** docstring sqlite_master | **Folded** F6 / AC1 |
| Agy | **O1** `.unwrap_or(false)` | **Already** F3 / F4 |
| Agy | **O2** count probes out of scope | **Already** AC3 |
| OpenCode | B / M | None filed |
| OpenCode | **m1** line drift `:281` → `:288` | **Folded** §2 |
| OpenCode | **m2** last-PR `#222` → `#226` | **Folded** §2 / §9 |
| OpenCode | **m3** Red is not behavioral | **Folded** §7 / plan |
| OpenCode | **m4** ConnectionRef docs anchor | **Folded** Depends-on + §2 |
| OpenCode | **O1** view-behavior research | **Folded** §2 / F4 |
| both | deferred map; no T311 | **Affirm** |

No Blockers/Majors to decline. No new placeholder. Do **not** edit `*-review.md`. Do **not** execute until go.
