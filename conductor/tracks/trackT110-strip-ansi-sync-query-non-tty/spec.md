# Track T110: Strip ANSI in `sync query` When Not TTY

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — output cleanliness; ANSI codes in piped output is garbage.
**Source:** Systematic command test 2026-06-22.

---

## Problem Statement

`ai-brains sync query "term" --format pretty` outputs the ChangeGuard ledger search results with ANSI color codes (bold, colored table cells). This looks great in a terminal but produces raw escape sequences when piped to another command or redirected to a file. The `ledgerful` CLI (called internally) detects TTY and adds colors, but `sync query` calls it without TTY context.

Additionally, T101 already added TTY detection for `recall` format defaulting, but `sync query` doesn't apply the same logic to its ChangeGuard subprocess output.

## Acceptance Criteria

**AC1:** When stdout is NOT a TTY, `sync query` passes a `--no-color` flag (or equivalent) to the `ledgerful search` subprocess to suppress ANSI codes.

**AC2:** When stdout IS a TTY, ANSI codes are preserved (current behavior).

**AC3:** If `ledgerful` doesn't support `--no-color`, the `sync query` output is post-processed to strip ANSI codes using the existing `strip_ansi` function (from T91).

**AC4:** The `--format json` path is unaffected (JSON output never has ANSI codes).

**AC5:** No regression in existing `sync query` tests.

## Design Notes

- Check if `ledgerful search` supports `--no-color` or `--color never` (check `ledgerful search --help`).
- If supported: add `--no-color` to the `ledgerful search` args in `sync.rs` when `!stdout.is_terminal()`.
- If not supported: apply `strip_ansi` (already exists from T91 in `crates/ai-brains-cli/src/commands/sync.rs`) to the ChangeGuard output before printing.
- The TTY check uses `is_terminal::IsTerminal` (already a dependency): `std::io::stdout().is_terminal()`.
- This is the same pattern as T101's `resolve_format` for `recall`.

## Files

- `crates/ai-brains-cli/src/commands/sync.rs` — add TTY check, pass `--no-color` to ledgerful or strip ANSI from output.

## Tests (TDD)

**Red:** `sync_query__piped_output__no_ansi_codes` — run `sync query "test" --format pretty` piped (not TTY), assert output contains no ANSI escape sequences (no `\x1b[` patterns).

**Green:** Add TTY-gated ANSI stripping. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains sync query "backup" --format pretty | cat` → no ANSI codes.
- Manual: `ai-brains sync query "backup" --format pretty` → colored output in terminal.

## Out of Scope

- Adding `--color`/`--no-color` as an explicit CLI flag (TTY detection is sufficient).
- Stripping ANSI from `recall` output (recall doesn't emit ANSI).
- Modifying `ledgerful` itself.
