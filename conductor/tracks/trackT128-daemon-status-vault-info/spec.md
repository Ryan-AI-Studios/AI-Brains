# Track T128: `daemon status` Vault Info

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — observability; can't tell which vault the daemon is serving.
**Source:** v0.1.1 verification opportunity #7.

---

## Problem Statement

`ai-brains daemon status` shows:
```
Status: Running
LLM backend 127.0.0.1:8081: Open
Embedding backend 127.0.0.1:8083: Open
PID: 15940
```

It doesn't show which vault the daemon is serving. This is a problem when:
- Multiple vaults exist (different projects, test vaults).
- The daemon was started in a different directory and may be serving a stale vault.
- Debugging "why isn't my recall finding memories" — you might be hitting a different vault than you think.

## Acceptance Criteria

**AC1:** `daemon status` shows the vault path the daemon is serving: `Vault: C:\dev\AI-Brains\vault.db`

**AC2:** `daemon status` shows the vault file size: `Vault size: 14.0 MB`

**AC3:** `daemon status` shows the memory count in the vault: `Memories: 1721`

**AC4:** If the daemon is not running, the vault info is NOT shown (it would require opening the vault independently, which could conflict with the daemon's lock).

**AC5:** The vault path is read from the daemon's PID file or the same env var resolution the daemon uses, not from the caller's `AI_BRAINS_VAULT_PATH` (which might differ).

## Design Notes

- **File:** `crates/ai-brains-cli/src/commands/daemon.rs` — `run_status` function.
- The daemon PID file (in the vault directory or temp) may contain the vault path. Check how the daemon stores its runtime state.
- Alternatively, the daemon could respond to a status RPC that includes vault info. Check if `DaemonClient` has a status/ping RPC that returns metadata.
- If no RPC exists, read the vault path from the same resolution the daemon uses (`AI_BRAINS_VAULT_PATH` env var → `.env` → `~/.ai-brains/.env` → default). Open the vault read-only to get size + memory count.
- Be careful not to deadlock: opening the vault while the daemon has it open in WAL mode should be fine for read-only, but use a short `busy_timeout`.
- Format vault size human-readably: `14.0 MB`, `1.2 GB`, etc.

## Files

- `crates/ai-brains-cli/src/commands/daemon.rs` — `run_status`: add vault info when daemon is running.

## Tests (TDD)

**Red:** `daemon_status__shows_vault_path` — start daemon, run `daemon status`, assert output contains `Vault:` and the vault path.

**Red:** `daemon_status__shows_memory_count` — run `daemon status`, assert output contains `Memories:` and a number > 0.

**Green:** Implement vault info display. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains daemon status` → shows Vault path, size, memory count.

## Out of Scope

- Showing vault encryption status (key presence is implied by successful open).
- Showing daemon uptime (would require daemon to track start time).
- Showing active sessions in the daemon (that's a separate query).
- Adding vault info to `daemon status --format json` (should be included but the format change is straightforward).