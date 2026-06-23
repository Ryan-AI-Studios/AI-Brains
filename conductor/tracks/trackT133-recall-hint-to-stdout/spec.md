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

**AC1:** The no-results hint in pretty format is printed via `println!` (stdout), not `eprintln!` (stderr).

**AC2:** PowerShell no longer wraps the hint in error formatting — it appears as normal output.

**AC3:** The JSON format behavior is unchanged (hint is already a JSON field on stdout).

**AC4:** Other `eprintln!` calls in `recall.rs` that are user-facing informational output (not errors) are also migrated to `println!`. Error messages that indicate actual failures (exit 1) remain on `eprintln!`.

## Design Notes

- **File:** `crates/ai-brains-cli/src/commands/recall.rs` — line 268, change `eprintln!("{}", hint)` to `println!("{}", hint)`.
- Scan other `eprintln!` calls in recall.rs for similar issues — only migrate informational output, not genuine errors.

## Files

- `crates/ai-brains-cli/src/commands/recall.rs` — line 268.

## Tests (TDD)

**Red:** `recall__no_results_pretty__hint_on_stdout` — run `recall "zzzz" --format pretty --no-bridge`, capture stdout and stderr separately, assert hint appears on stdout and NOT on stderr.

**Green:** Change `eprintln!` to `println!`. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains recall "zzzz" --format pretty --no-bridge` → hint on stdout, no PowerShell error wrapping.

## Out of Scope

- Migrating `eprintln!` in other command files (that's a broader effort).
- Changing the JSON format (already correct).