# Track T101: Default `recall` to Pretty Format When stdout is a TTY

**Status:** Complete
**Started:** 2026-06-22
**Owner:** Claude
**Priority:** P3 — UX polish; JSON by default is unfriendly to interactive users.
**Source:** Non-destructive command test 2026-06-22.

---

## Problem Statement

`ai-brains recall "query"` outputs raw JSON by default. A new user expecting readable output sees a wall of escaped JSON. The `--format pretty` flag exists but is not the default. The `preflight` command already implements smart TTY detection (defaulting to human format when stdout is a TTY), but `recall` does not.

## Acceptance Criteria

**AC1:** When stdout is a TTY and `--format` is not explicitly set, `recall` defaults to `pretty` format (human-readable with scores and memory IDs).

**AC2:** When stdout is NOT a TTY (piped to another command, redirected to a file), `recall` defaults to `json` format (machine-readable).

**AC3:** Explicitly passing `--format pretty` or `--format json` overrides the TTY detection.

**AC4:** No regression in existing tests. All CLI tests that assert output format must explicitly pass `--format json` or `--format pretty` to avoid TTY-dependent flakes between local and CI runs.

**AC5:** Pretty format truncates memory content to a reasonable display length (e.g. 500 chars) with a `...` indicator. This prevents terminal lockup when a memory contains a large code dump.

## Design Notes

- Use `std::io::IsTerminal` (stable since Rust 1.70) to check `std::io::stdout().is_terminal()`.
- Follow the same pattern as `preflight.rs:51-53`: 
  ```rust
  let format_str = format.unwrap_or_else(|| {
      if std::io::stdout().is_terminal() { "pretty".to_string() } else { "json".to_string() }
  });
  ```
- The change is in `crates/ai-brains-cli/src/commands/recall.rs:run()` where `options.format` is used.
- Currently `RecallRunOptions.format` is a `String` with no default. The CLI clap definition at `main.rs` has `format: Option<String>` which becomes `"json"` if not specified (check the exact default).

## Files

- `crates/ai-brains-cli/src/commands/recall.rs` — add TTY detection for default format.
- `crates/ai-brains-cli/src/main.rs` — check if the clap default needs updating.

## Tests (TDD)

**Red:** `recall__tty_stdout__defaults_to_pretty` — simulate a TTY (or test the format selection logic directly) and verify `pretty` is chosen when no `--format` is passed.

**Green:** Add TTY detection. Test passes.

> Note: Testing TTY detection in a unit test is tricky. The simplest approach: extract the format selection into a testable function `fn default_format(explicit: Option<&str>, is_tty: bool) -> &str` and test that directly.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains recall "test"` from a terminal — outputs pretty format.
- Manual: `ai-brains recall "test" | cat` — outputs JSON.
- Manual: `ai-brains recall "test" --format json` from a terminal — outputs JSON (explicit override).

## Out of Scope

- Changing other commands' default formats.
- Adding color/output styling.
- A `--format table` option.