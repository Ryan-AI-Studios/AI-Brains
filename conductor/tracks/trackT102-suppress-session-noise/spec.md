# Track T102: Suppress Session-ID Noise on recall and sync query

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — UX polish; every recall prints a session-ID warning to stderr.
**Source:** Non-destructive command test 2026-06-22.

---

## Problem Statement

Every `ai-brains recall` (and `sync query`) without `--session-id` prints to stderr:

```
No session id supplied for recall; using generated session <uuid> for graph provenance.
```

This fires on every invocation from a non-interactive context. It clutters stderr and breaks the `recall | jq` idiom (stderr noise). The `--quiet` flag suppresses it, but it's not the default. This message is informational (graph provenance uses the session ID for `RECALLS` edges), not an error — it should not be printed by default.

## Acceptance Criteria

**AC1:** The "No session id supplied" message is suppressed by default. It only prints when `RUST_LOG=debug` is set (via `tracing::debug!`).

**AC2:** The existing `--quiet` flag continues to suppress all stderr output.

**AC3:** The generated session ID is still used for graph provenance — only the message is suppressed, not the behavior.

**AC4:** The generated `session_id` is included in the JSON response envelope and the pretty format output as metadata, so users debugging graph provenance can see which session a recall was bucketed under without needing `RUST_LOG=debug`. Add a `session_id` field to `RecallResponse` in `ai-brains-contracts`.

**AC5:** No regression: existing tests that check for the message (if any) are updated. Tests that parse JSON output are updated to handle the new `session_id` field.

## Design Notes

- The message is at `crates/ai-brains-cli/src/commands/recall.rs:26-29`:
  ```rust
  eprintln!("No session id supplied for recall; using generated session {} for graph provenance.", generated);
  ```
- Simplest fix: change `eprintln!` to `tracing::debug!` so it only prints when `RUST_LOG=debug` is set. This avoids adding a new `--verbose` flag.
- Alternatively: gate on `!options.quiet` — but this is the current `--quiet` behavior. The user wants it suppressed by default, not just with `--quiet`.
- The `tracing::debug!` approach is cleanest: it's already initialized via `tracing_subscriber::fmt::init()` in `main.rs:455`, and defaults to `WARN` level. The message won't print unless the user sets `RUST_LOG=debug`.
- Do NOT add a `--verbose` flag — `tracing::debug!` is sufficient and consistent with the codebase.

## Files

- `crates/ai-brains-cli/src/commands/recall.rs` — change `eprintln!` to `tracing::debug!`, include `session_id` in response.
- `crates/ai-brains-contracts/src/recall.rs` — add `session_id` field to `RecallResponse`.

## Tests (TDD

**Red:** `recall__no_session_id__does_not_print_warning_by_default` — run `recall` without `--session-id`, capture stderr, assert it does NOT contain "No session id supplied". Fails because the `eprintln!` fires.

**Green:** Change to `tracing::debug!`. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains recall "test" 2>stderr.txt; cat stderr.txt` — empty (or no session-ID message).
- Manual: `RUST_LOG=debug ai-brains recall "test" 2>stderr.txt; cat stderr.txt` — message appears.

## Out of Scope

- Adding a `--verbose` flag.
- Changing other eprintln messages.
- Modifying the graph provenance behavior.