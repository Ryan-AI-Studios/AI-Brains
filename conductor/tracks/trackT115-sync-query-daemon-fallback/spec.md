# Track T115: Sync Query Daemon Fallback — Local-Only on Daemon Down

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P2 — sync query is unusable when daemon is down, even for local-only search.
**Source:** Non-destructive command audit 2026-06-23.

---

## Problem Statement

`ai-brains sync query "term"` requires the daemon to be running and returns an error if it's not:

```
Error: AI-Brains daemon is not running or unreachable.
```

The local recall portion of `sync query` (the "AI-Brains Recall" section) does NOT need the daemon — it reads directly from the vault via `recall()`. Only the ChangeGuard ledger search section calls `ledgerful` (which is a separate process, not the daemon). The daemon is only used for... actually, looking at the code, the daemon check is a gate that prevents the entire command from running, but the command itself doesn't call the daemon API at all.

This is an unnecessary gate that makes `sync query` fail in environments where the daemon isn't running (CI, fresh boots, etc.).

## Acceptance Criteria

**AC1:** When the daemon is NOT running, `sync query` proceeds with local-only recall and ChangeGuard search, printing a warning to stderr: `Warning: AI-Brains daemon is not running. Local recall and ChangeGuard search only.`

**AC2:** When the daemon IS running, `sync query` works as before (no warning).

**AC3:** The `--quiet` flag suppresses the daemon-down warning.

**AC4:** The `--format ndjson` path also works without the daemon.

**AC5:** No regression in existing `sync query` tests.

## Design Notes

- **Fix:** In `sync.rs:run_query()`, remove the `ensure_running` gate. Instead, probe the daemon and emit a warning if down. The command proceeds regardless.

- **Current code (sync.rs:395-402):**
  ```rust
  let client = crate::daemon_client::DaemonClient::new();
  if !client.ensure_running(&ctx.vault_path, &ctx._key).await {
      if quiet {
          return Ok(());
      }
      return Err("AI-Brains daemon is not running or unreachable.".into());
  }
  ```

- **New code:**
  ```rust
  let client = crate::daemon_client::DaemonClient::new();
  let daemon_up = client.ensure_running(&ctx.vault_path, &ctx._key).await;
  if !daemon_up && !quiet {
      eprintln!("Warning: AI-Brains daemon is not running. Local recall and ChangeGuard search only.");
  }
  // Proceed regardless — local recall doesn't need the daemon.
  ```

- **Note:** The `ensure_running` call may attempt to start the daemon. If we want to avoid that, use `client.probe()` instead (just checks if running, doesn't start). Check the DaemonClient API.

- **Alternative:** If `ensure_running` starts the daemon, change to a lightweight probe:
  ```rust
  let daemon_up = client.probe(std::time::Duration::from_millis(500)).await;
  ```

## Files

- `crates/ai-brains-cli/src/commands/sync.rs` — Remove the hard gate, replace with warning.

## Tests (TDD)

**Red:** `sync_query__daemon_down__returns_local_results` — ensure daemon is not running, run `sync query "test"`, assert it returns local recall results with a warning on stderr.

**Red:** `sync_query__daemon_down_quiet__no_warning` — same but with `--quiet`, assert no warning on stderr.

**Green:** Remove the hard gate. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: Stop daemon (`ai-brains daemon stop`), run `ai-brains sync query "backup" --format pretty` → local recall works, warning on stderr, ChangeGuard search runs.
- Manual: Start daemon (`ai-brains daemon start`), run same command → no warning.

## Out of Scope

- Making the ChangeGuard `ledgerful` search independent of the daemon (it already is — `ledgerful` is a separate CLI).
- Removing the daemon dependency from `sync push` or `sync pull` (those may genuinely need it).
- Adding a `--no-daemon-check` flag (unnecessary if we just warn).