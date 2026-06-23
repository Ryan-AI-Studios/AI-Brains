# Track T133: Recall Hint to Stdout

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — UX friction; no-results hint prints to stderr, PowerShell treats as error.
**Source:** T119-T132 non-destructive command audit.

---

## Problem Statement

When `recall` returns zero results in pretty format, the hint message is printed via `eprintln!`:

```
$ ai-brains recall "zzzz" --format pretty --no-bridge
Session: 72b13e5d-...
ai-brains : No results for 'zzzz'. Try --semantic for embedding-based search...
```

PowerShell interprets stderr output as a native error, wrapping it in `RemoteException` formatting that looks like the command crashed. The hint is informational output, not an error — it should go to stdout via `println!`.

The JSON format already handles this correctly (hint is a field in the JSON object on stdout).

## Acceptance Criteria

**AC1:** The no-results hint in pretty format is printed via `println!` (stdout), not `eprintln!` (stderr), BUT only when stdout is an interactive terminal (`std::io::stdout().is_terminal()`). When stdout is NOT a terminal (piped/redirected), the hint is suppressed entirely to avoid polluting downstream pipes (e.g., `ai-brains recall "query" | jq`).

**AC2:** PowerShell no longer wraps the hint in error formatting — it appears as normal output on an interactive terminal.

**AC3:** The JSON format behavior is unchanged (hint is already a JSON field on stdout, always emitted regardless of TTY).

**AC4:** Other `eprintln!` calls in `recall.rs` that are user-facing informational output (not errors) are also migrated to `println!` with the same TTY guard. Error messages that indicate actual failures (exit 1) remain on `eprintln!`.

## Design Notes

- **File:** `crates/ai-brains-cli/src/commands/recall.rs` — line 268, change `eprintln!("{}", hint)` to a TTY-guarded `println!`:
  ```rust
  use is_terminal::IsTerminal;
  if std::io::stdout().is_terminal() {
      println!("{}", hint);
  }
  ```
  This prevents the hint from polluting piped output while still showing it in interactive terminals.
- The crate already depends on `is-terminal` (used elsewhere in the CLI).
- Scan other `eprintln!` calls in recall.rs for similar issues — only migrate informational output with the same TTY guard, not genuine errors.

## Files

- `crates/ai-brains-cli/src/commands/recall.rs` — line 268.

## Tests (TDD)

**Red:** `recall__no_results_pretty__hint_on_stdout_tty` — run `recall "zzzz" --format pretty --no-bridge` with stdout as a pipe, assert hint does NOT appear on stdout or stderr (TTY guard suppresses it when not interactive). Run with stdout as terminal, assert hint appears on stdout.

**Green:** Add TTY-guarded `println!`. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains recall "zzzz" --format pretty --no-bridge` → hint on stdout, no PowerShell error wrapping.

## Out of Scope

- Migrating `eprintln!` in other command files (that's a broader effort).
- Changing the JSON format (already correct).