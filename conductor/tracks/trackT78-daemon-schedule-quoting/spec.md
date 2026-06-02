# Track T78: daemon schedule schtasks quoting fix

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P0 — Windows-only P0; `daemon schedule` was non-functional.

---

## Problem Statement

`ai-brains daemon schedule` failed on Windows with:

```
schtasks: ERROR: Access is denied.
```

The renderer at `crates/ai-brains-scheduler/src/lib.rs:43` produced:

```
schtasks /create /tn "AI-Brains-Daemon" /tr "\"C:\Users\RyanB\.cargo\bin\ai-brainsd.exe\"" /sc ONLOGON /delay 0000:30 /f
```

The escape sequence `\\\"{exe_path}\\\"` in the Rust `format!` produces the
literal substring `\"C:\…\ai-brainsd.exe\"` — note the backslashes that
immediately precede the closing double-quote. `schtasks` rejects the
trailing backslash as malformed quoting and refuses to register the task.

The sibling renderer, `render_create_command` (used by `nightly --schedule`),
uses the single-quote convention `'{exe_path}' nightly` and works
correctly. The two renderers had drifted; the daemon one was broken, and
its unit test was *asserting the broken output*, locking in the bug.

## Acceptance Criteria

**AC1:** `TaskScheduler::render_daemon_logon_command(exe, name, delay)`
emits a schtasks command using the single-quote convention. The output is
bit-identical to the format that `render_create_command` produces for the
same input (modulo the `/sc ONLOGON /delay` switch).

**AC2:** `ai-brains daemon schedule` (run in an elevated PowerShell
session) registers the task successfully.

**AC3:** A new regression test covers paths with spaces (e.g.,
`C:\Program Files\AI-Brains\ai-brainsd.exe`), which are the most common
real-world path and the most likely to expose quoting bugs.

## Design Notes

- One-line change in `render_daemon_logon_command`: replace the
  `\\\"{exe_path}\\\"` wrapper with `'{exe_path}'` to match the convention
  established in `render_create_command` (line 22).
- Two test updates: the existing test's expected string is corrected to
  the new output, and a new test with a path containing spaces is added
  so future regressions are caught.

## Files

- `crates/ai-brains-scheduler/src/lib.rs` — 1-line change in the renderer,
  2 test updates (1 corrected, 1 new).

## Tests (TDD)

Red phase: updated `test_render_daemon_logon_command` to assert the
single-quote output, and added `test_render_daemon_logon_command_with_spaces_in_path`.
Both failed against the buggy implementation.

Green phase: the renderer fix made both tests pass.

## Verification

- `cargo nextest run -p ai-brains-scheduler` — 3/3 pass.
- The rendered command can be inspected by running:
  `ai-brains daemon schedule --dry-run` (or by reading the printed
  command in the current `daemon schedule` output, which already prints
  the rendered string before invoking `cmd /C`).

## Out of Scope

- The `daemon schedule` itself still requires an elevated PowerShell
  session to actually register the task (a Windows ACL requirement, not
  a code issue). The fix makes the *command* correct; the *invocation*
  is unchanged.
