# Track T96: SQLCipher `busy_timeout` for Concurrent CLI Access

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P2 — transient "unable to open database file" errors under concurrent access.
**Source:** `C:\dev\testing_report.md` Item 3; root cause confirmed by code audit 2026-06-22.

---

## Problem Statement

AI-Brains' SQLCipher store (`crates/ai-brains-store/src/pragmas.rs`) sets `journal_mode = WAL` and `synchronous = NORMAL` but does NOT set `PRAGMA busy_timeout`. As a result, when two CLI invocations (or a CLI invocation and the daemon) contend on the write lock, SQLite returns `SQLITE_BUSY` immediately instead of retrying. The user sees:

```
ChangeGuard signals were unavailable: changeguard ledger status --compact failed with unable to open database file.
```

ChangeGuard's own store already sets `PRAGMA busy_timeout = 5000` (`src/state/storage.rs:71`). AI-Brains should do the same — or a higher value for read-only paths — to match the robustness of the rest of the toolchain.

## Acceptance Criteria

**AC1:** `apply_pragmas` in `crates/ai-brains-store/src/pragmas.rs` sets `PRAGMA busy_timeout = 5000` (5 seconds) on every connection. SQLite retries internally for up to 5s before returning `SQLITE_BUSY`.

**AC2:** A new test `pragma_busy_timeout_is_set__new_connection__returns_5000` opens a connection via the standard path and queries `PRAGMA busy_timeout`, asserting the value is `5000`.

**AC3:** The daemon's single-writer queue continues to work. The `busy_timeout` does not replace the queue — it is a safety net for the case where a CLI tool opens a read connection while the daemon holds the write lock (or vice versa).

**AC4:** No regression in existing store tests. All tests in `ai-brains-store` pass.

## Design Notes

- The change is one line: add `conn.execute_batch("PRAGMA busy_timeout = 5000;")?;` after the `synchronous` pragma in `pragmas.rs:19`.
- WAL mode allows concurrent readers + one writer. The `busy_timeout` ensures that a reader that hits the lock waits up to 5s instead of failing instantly.
- The value `5000` matches ChangeGuard's store. If AI-Brains finds this insufficient in practice (heavier write loads), it can be raised to 10000 in a follow-up. 5000 is the proven baseline.
- This is NOT a replacement for the daemon's single-writer queue. It is a defense-in-depth measure for CLI tools that open their own connections (e.g. `ai-brains recall` reading while `ai-brains nightly` writes).

## Files

- `crates/ai-brains-store/src/pragmas.rs` — add `busy_timeout` pragma.
- `crates/ai-brains-store/tests/pragmas.rs` (or appropriate test module) — new test.

## Tests (TDD)

**Red:** `pragma_busy_timeout_is_set__new_connection__returns_5000` — opens a connection, queries `PRAGMA busy_timeout`, asserts `5000`. Fails because the pragma is not set.

**Green:** Add the pragma line. Test passes.

## Verification

- `cargo nextest run -p ai-brains-store`
- Manual: open two terminals, run `ai-brains nightly` (long write) in one and `ai-brains recall "test"` in the other simultaneously. Confirm no "unable to open database file" error.

## Out of Scope

- Changing the daemon's single-writer queue architecture.
- Adding a configurable busy_timeout via env var (hardcode 5000 for now; config can come later if needed).
- Changes to ChangeGuard's store (it already has busy_timeout).