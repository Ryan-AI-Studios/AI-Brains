# Track T77: forget — clear error on unknown --memory-id

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P0 — silent failure in a critical "hygiene" surface.

---

## Problem Statement

`ai-brains forget --memory-id=<X>` is the only way to forget a memory by
its UUID directly (e.g., from a script or from `recall` output). When `<X>`
does not match any row in `memory_projection`, the command:

1. Parses the UUID successfully.
2. Appends a `MemoryForgotten` event to the event log.
3. Calls the `MemoryProjection` handler, whose `UPDATE ... WHERE memory_id = ?`
   matches **zero rows** because no projection row exists with that ID.
4. Prints `"Memory <X> marked as forgotten."` and exits **0**.

The user is left believing the memory was forgotten, but the data is still
served by `recall`, still in `memory_projection`, and only visible as a
dead event in the `events` table.

The audit (June 2026) confirmed this by re-testing the full forget
workflow on a fresh vault: when given a valid UUID (e.g., discovered via
`forget --match`), the projection correctly transitions to `forgotten` and
`recall` excludes the memory. The bug only manifests when the supplied
UUID is wrong.

## Acceptance Criteria

**AC1:** `ai-brains forget --memory-id=<unknown-uuid> --force` exits **1**
with a structured JSON error envelope. The error message clearly states
that the memory is not found and points to the alternative commands
(`forget --match`, `forget --list-forgotten`).

**AC2:** `ai-brains forget --memory-id=<known-uuid> --force` continues to
work as before — appends the event, updates the projection, prints success.

**AC3:** The new validation runs *only* on the `--memory-id` branch.
`forget --match` and `forget --list-forgotten` and `forget --restore` are
unaffected.

## Design Notes

- Added a `memory_exists(memory_id: &str) -> Result<bool>` method to the
  `QueryStore` trait (and its `VaultConnection` impl). The query is a
  `SELECT COUNT(*) FROM memory_projection WHERE memory_id = ?` — cheap
  and indexed.
- Validation runs *after* `MemoryId::from_str` (so the user gets a clear
  "invalid UUID" error if the format is wrong) but *before* the event is
  appended.
- The error message mirrors the `T73 init` style: a single `Err(Box<dyn Error>)`
  string that the CLI's main() wraps in `ApiError::new("COMMAND_FAILED", ...)`.

## Files

- `crates/ai-brains-store/src/lib.rs` — added `memory_exists` to the
  `QueryStore` trait.
- `crates/ai-brains-store/src/query_store.rs` — added the implementation.
- `crates/ai-brains-cli/src/commands/forget.rs` — precheck in the
  `--memory-id` branch.
- `crates/ai-brains-cli/tests/smoke.rs` — `test_forget_unknown_memory_id_errors`.

## Tests (TDD)

Red phase: added `test_forget_unknown_memory_id_errors` to
`crates/ai-brains-cli/tests/smoke.rs`. The test runs `init`, then
`forget --memory-id=00000000-...-0000 --force`, and asserts non-zero exit
plus a stderr message containing "not found" or "not in".

Green phase: implemented the `memory_exists` precheck in
`forget.rs`. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli test_forget_unknown_memory_id_errors`
  — passes.
- `cargo nextest run -p ai-brains-cli` — all 32 tests pass (31 pre-existing
  + 1 new).

## Out of Scope

- The deeper question of *which* UUIDs are forgettable (e.g., should we
  allow forgetting memory_projection rows created by `TurnProjection`
  for non-`pin` events?) is a T85/T86 follow-up. T77 only closes the
  silent-failure gap.
