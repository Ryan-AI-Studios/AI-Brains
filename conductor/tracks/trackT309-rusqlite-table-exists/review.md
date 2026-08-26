# T309 Review Log — rusqlite `table_exists`

**Status:** **Completed** (full `dev-check.ps1` exit 0; Phase 6 publish follows)
**Category:** CHORE / REFACTOR
**Ledger:** CHORE TX `473e1069-374e-4a2d-96ba-38d64b417cd7`
**Date:** 2026-08-26
**Branch:** `track/T309-rusqlite-table-exists`

## Phase 0 evidence

| Check | Result |
|-------|--------|
| cwd | `C:\dev\AI-Brains` |
| docs.rs 0.40.2 | `Connection::table_exists(db_name: Option<N>, table_name: N) -> Result<bool>` |
| Pin | workspace `0.40.2`; lock checksum `23f2a97d…` unchanged |
| last-PR Cursor | `#226` / `#225` comments `[]` |
| T307 / T310 | not stolen |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| AC1 | — | sqlite_master probes in the two named fns | `verified_fixed` | `has_core_tables` / `has_graph_tables` call `table_exists`; docs omit sqlite_master |
| AC5 | — | Missing `has_core_tables__*` units | `verified_fixed` | `has_core_tables__empty_db__false` + `__both_tables__true` PASS |
| R1 | low-info | Test-local `fn table_exists` helpers still sqlite_master | deferred | Non-goal F1 / §11 |
| R2 | low-info | PATH binary until `cargo install` | deferred | Soft; source SoT |
| R3 | low-info | T310 placeholder not stolen | deferred | Out of scope |

No critical / high / medium. CHORE — no FEATURE/SECURITY/ARCHITECTURE cross-model required.

## DoD

| AC | Result |
|----|--------|
| AC1 | **Met** — both fns + their docs have no `sqlite_master` |
| AC2 | **Met** — `has_graph_tables__*` + workspace nextest 3531 passed |
| AC3 | **Met** — `backup.rs:252` / `:488` count probes unchanged |
| AC4 | **Met** — clippy `-D warnings` both crates + workspace exit 0 |
| AC5 | **Met** — named `has_core_tables__*` units PASS |
| F6 | **Met** — `has_graph_tables` docstring rewritten |
| Pin | **Met** — rusqlite 0.40.2 unchanged |

## Manual evidence

Happy path:

```
cargo nextest run -p ai-brains-brain --lib has_core_tables
```

`has_core_tables__both_tables__true` PASS; `has_core_tables__empty_db__false` PASS.

Error/false path: empty in-memory DB → `false` (same F4).

Prior regression: encrypt/backup key probes still `SELECT count(*) FROM sqlite_master` at `backup.rs:252` and `:488`.

```
cargo clippy -p ai-brains-brain --all-targets -- -D warnings  # exit 0
cargo clippy -p ai-brains-cli --all-targets -- -D warnings    # exit 0
cargo nextest run --lib --bins -p ai-brains-brain -p ai-brains-cli  # 962 passed
.\scripts\dev-check.ps1  # exit 0; 3531 passed, 1 skipped
ledgerful verify --scope full  # exit 0
```

## Closure

Conductor → **Completed**. Residuals → `conductor/deferred.md`. T307 / T310 not stolen. Phase 6 publish follows.
