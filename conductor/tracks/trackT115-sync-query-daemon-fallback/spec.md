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

**AC1:** When the daemon is NOT running, `sync query` proceeds with local-only recall and ChangeGuard search without any warning or delay. The command does not probe or attempt to start the daemon.

**AC2:** When the daemon IS running, `sync query` works as before (no change).

**AC3:** The `--quiet` flag is unaffected (it already suppresses ChangeGuard errors).

**AC4:** The `--format ndjson` path also works without the daemon.

**AC5:** No regression in existing `sync query` tests.

**AC6:** No daemon probe latency — the command starts immediately without waiting for a named pipe connection timeout.

## Design Notes

- **Fix:** Remove the `ensure_running` gate entirely from `run_query()`. The daemon is irrelevant to `sync query` execution — local recall reads the vault directly, and ChangeGuard search calls `ledgerful` (a separate CLI). Probing adds latency (named pipe timeout) and `ensure_running` may attempt to spawn the daemon (adding seconds of delay). The cleanest fix is to simply not check.

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

- **New code:** Delete those lines entirely. No probe, no warning, no spawn attempt. The command proceeds directly to local recall + ChangeGuard search.

- **Why no warning:** The daemon's presence or absence has no impact on `sync query`'s ability to function. Warning about it would be noise. If a future feature adds a daemon-dependent code path to `sync query`, only that path should warn on failure.

## Files

- `crates/ai-brains-cli/src/commands/sync.rs` — Remove the hard gate, replace with warning.

## Tests (TDD)

**Red:** `sync_query__daemon_down__returns_local_results` — ensure daemon is not running, run `sync query "test"`, assert it returns local recall results without error or delay.

**Red:** `sync_query__daemon_down__no_spawn_attempt` — ensure daemon is not running, run `sync query "test"`, assert the command completes in <1s (proves no spawn attempt or probe timeout).

**Green:** Remove the hard gate. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: Stop daemon (`ai-brains daemon stop`), run `ai-brains sync query "backup" --format pretty` → local recall works, warning on stderr, ChangeGuard search runs.
- Manual: Start daemon (`ai-brains daemon start`), run same command → no warning.

## Out of Scope

- Making the ChangeGuard `ledgerful` search independent of the daemon (it already is — `ledgerful` is a separate CLI).
- Removing the daemon dependency from `sync push` or `sync pull` (those may genuinely need it).
- Adding a `--no-daemon-check` flag (unnecessary if we just warn).