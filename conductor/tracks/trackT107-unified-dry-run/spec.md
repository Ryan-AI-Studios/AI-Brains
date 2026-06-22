# Track T107: Unified --dry-run for Mutating Commands

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — UX consistency; some mutating commands have --dry-run, others don't.
**Source:** Systematic command test 2026-06-22.

---

## Problem Statement

`backup restore`, `daemon schedule/unschedule`, `safety sync`, and `nightly schedule` all have `--dry-run`. But `pin`, `forget`, `ingest`, and `sync push` do not. Users who expect `--dry-run` on all mutating commands are surprised when it's missing. This is inconsistent UX.

## Acceptance Criteria

**AC1:** `ai-brains pin "content" --dry-run` prints what would be pinned (content preview, role, privacy, tags) without writing to the vault. Output: `[dry-run] Would pin: <truncated content> (role=assistant, privacy=LocalOnly, tags=[...])`.

**AC2:** `ai-brains forget --memory-id <id> --dry-run` prints what would be forgotten (memory ID, content preview) without marking it as forgotten. Output: `[dry-run] Would forget memory <id>: <truncated content>`.

**AC3:** `ai-brains forget --match "query" --dry-run` lists the memories that would be forgotten without marking any. Output: `[dry-run] Would forget N memory/memories matching "query":` followed by a list.

**AC4:** `ai-brains ingest --dry-run` reads the JSON from stdin, validates it, and prints what would be ingested (turn ID, session ID, role, content preview) without writing to the vault.

**AC5:** All `--dry-run` outputs go to stdout (not stderr) so they can be piped/redirected.

**AC6:** All `--dry-run` operations return exit code 0 and do not modify the vault, event log, or graph.

## Design Notes

- Add `dry_run: bool` to the option structs or function parameters for each command.
- The `--dry-run` flag should be added at the clap subcommand level (not as a global flag).
- For `pin`: parse and validate the content/args, build the event, print the preview, skip `event_store.append_event()`.
- For `forget`: query the memories that match, print the list, skip the `MemoryForgotten` event append.
- For `ingest`: read and parse stdin JSON, validate fields, print the preview, skip the event append.
- Truncate content preview to 100 chars with `...` indicator.
- **Event log purity:** The `EventStore::append_event` call must be physically isolated behind an explicit `if !dry_run { ... }` block, not gated by an early return deep inside a helper function. This makes the no-write guarantee structurally verifiable in code review. The pattern:
  ```rust
  if dry_run {
      println!("[dry-run] Would pin: ...");
  } else {
      event_store.append_event(&ev)?;
  }
  ```
- Keep the implementation simple — the dry-run path should be a few lines of preview output.

## Files

- `crates/ai-brains-cli/src/commands/pin.rs` — add `dry_run` parameter.
- `crates/ai-brains-cli/src/commands/forget.rs` — add `dry_run` parameter.
- `crates/ai-brains-cli/src/commands/ingest.rs` — add `dry_run` parameter.
- `crates/ai-brains-cli/src/main.rs` — add `--dry-run` flag to `Pin`, `Forget`, `Ingest` commands.

## Tests (TDD)

**Red:** `pin__dry_run__does_not_write_to_vault` — run `pin "test" --dry-run`, assert vault memory count is unchanged.

**Green:** Add dry-run flag. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains pin "test content" --dry-run` → preview, no vault change.
- Manual: `ai-brains forget --match "test" --dry-run` → list, no vault change.

## Out of Scope

- `sync push --dry-run` (sync push already returns "No insights to push" in most cases — low value).
- `context --dry-run` (context is not really mutating — it writes .env, but that's a config file, not vault state).
- `project set-alias --dry-run` (low frequency command).
