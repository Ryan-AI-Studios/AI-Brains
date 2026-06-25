# Track T136: `--log-format minimal` Mode

**Status:** Complete
**Started:** —
**Owner:** —
**Priority:** P3 — UX; compact format still has full timestamps, off is all-or-nothing.
**Source:** T119-T132 non-destructive command audit.

---

## Problem Statement

`--log-format compact` uses `tracing_subscriber::fmt().compact()` which removes the target but still shows full ISO timestamps:
```
2026-06-23T22:38:24.994305Z  WARN Could not read backup metadata...
```

`--log-format off` suppresses everything, but there's no middle ground for interactive use where users want to see WARN/ERROR messages without the verbose timestamp noise.

A `minimal` format that shows just `WARN message` (level + message, no timestamp, no target) would be ideal for terminal use.

## Acceptance Criteria

**AC1:** `--log-format minimal` shows tracing output as `LEVEL message` with no timestamp and no target:
```
WARN Could not read backup metadata path=...
INFO Creating vault backup...
```

**AC2:** `RUST_LOG` still controls the level filter in minimal mode (same as other modes).

**AC3:** The default remains `compact` (not `minimal`).

**AC4:** `--help` shows `minimal` as a valid value for `--log-format`.

## Design Notes

- **File:** `crates/ai-brains-cli/src/main.rs` — add `"minimal"` arm to the log format match.
- `tracing_subscriber` doesn't have a built-in "minimal" formatter. Options:
  1. Use `.with_target(false)` + `.with_timer(tracing_subscriber::fmt::time::NoTime)` + `.compact()`. This removes both timestamp and target while keeping the compact format.
  2. Use `.with_target(false).with_level(true).with_timer(tracing_subscriber::fmt::time::NoTime)` without `.compact()` for a slightly different layout.
- Approach 1 is simplest: `.compact().with_target(false).with_timer(NoTime)`.
- Need to import `tracing_subscriber::fmt::time::NoTime` or use `.with_timer(tracing_subscriber::fmt::time::Uptime)` or `.without_time()` if available in the version used.
- Check `tracing-subscriber` version in workspace Cargo.toml — `.without_time()` may be available.

## Files

- `crates/ai-brains-cli/src/main.rs` — add `minimal` to the log format match.

## Tests (TDD)

**Red:** `log_format_minimal__no_timestamp` — run a command with `--log-format minimal`, capture stderr, assert no line contains an ISO timestamp pattern (`YYYY-MM-DDTHH:MM:SS`).

**Green:** Add the minimal format arm. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains --log-format minimal backup list` → `WARN Could not read...` without timestamps.

## Out of Scope

- Custom format templates.
- File logging.
- Changing the default format.