# Track T143: Fix `--run-as-system` Nightly Scheduling for SYSTEM Context

**Status:** Complete (merged on main; conductor closeout 2026-07-24)
**Started:** 2026-07-01
**Owner:** Antigravity (closeout: Grok; live confirm via T145)
**Priority:** P1 — the nightly task has been failing since 2026-06-25 because `--run-as-system` registers a task without the env vars and flags SYSTEM needs.
**Source:** Manager investigation 2026-07-01; T132 (`--run-as-system` flag) shipped a task that runs `ai-brains.exe nightly` as SYSTEM with no working directory, no vault path, no LLM env vars, and no `--skip-import` / `--no-project-context`.

---

## Problem Statement

T132 added `--run-as-system` to `ai-brains nightly --schedule` and `ai-brains daemon --schedule`. It registers a Windows scheduled task with `/RU SYSTEM` so the task runs without login. However, the task is registered with bare arguments:

```
Command:    "C:\Users\RyanB\.cargo\bin\ai-brains.exe"
Arguments:  nightly
Run As:     SYSTEM
Start In:   (none — defaults to C:\Windows\System32)
```

When SYSTEM runs this:

1. **No vault path** — `AI_BRAINS_VAULT_PATH` is a User-level env var; SYSTEM doesn't inherit User env vars. The CLI falls back to `vault.db` in cwd (`C:\Windows\System32\vault.db`), which doesn't exist → "Database error: Query returned no rows".
2. **No LLM/embedding URLs** — `AI_BRAINS_MODEL_URL`, `AI_BRAINS_COMPLETION_MODEL`, `AI_BRAINS_EMBEDDING_URL`, `AI_BRAINS_EMBEDDING_MODEL` are all User-level env vars. Without them, the nightly's LLM summarization calls fail or use wrong defaults.
3. **Antigravity import fails** — the import scans the user's Antigravity session DB, which SYSTEM can't access at the user-profile path. The import errors with "Query returned no rows" and the nightly continues, but this is wasted work and noise.
4. **No `--no-project-context`** — the CLI tries to auto-discover project context from a `.env` in cwd (`C:\Windows\System32`), finds none, and wastes cycles or fails.
5. **No `--skip-import`** — the nightly attempts the Antigravity import which is guaranteed to fail as SYSTEM.

**Last successful nightly run:** 2026-06-25T11:46:18 UTC. The task has been firing daily but exiting with code 1 since then.

## Acceptance Criteria

**AC1 — `--run-as-system` bakes env vars into the task:**
When `nightly --schedule --run-as-system` is invoked, the scheduled task command includes the current `AI_BRAINS_VAULT_PATH`, `AI_BRAINS_MODEL_URL`, `AI_BRAINS_COMPLETION_MODEL`, `AI_BRAINS_EMBEDDING_URL`, `AI_BRAINS_EMBEDDING_MODEL` values from the user's env (or `.env`) as part of the task action — either via a wrapper script (bat/sh) or via `cmd /c "set VAR=val && ..."` syntax in the task command. SYSTEM does not inherit User env vars, so these must be explicit.

**AC2 — `--run-as-system` adds `--no-project-context --skip-import`:**
When `--run-as-system` is passed, the task arguments automatically include `--no-project-context` (SYSTEM has no `.env` to discover) and `--skip-import` (SYSTEM can't access the user's Antigravity session DB). These are not required when `--run-as-system` is NOT passed (user context has access to both).

**AC3 — Working directory set:**
The scheduled task's "Start In" (working directory) is set to the vault's parent directory (or the repo root if discoverable), so relative paths resolve correctly.

**AC4 — `--run-as-system --dry-run` shows the full command:**
When `--dry-run` is also passed (if it exists for `nightly --schedule`), the printed schtasks command includes all env vars and flags so the user can verify before registering. If `--dry-run` doesn't exist for nightly, add it.

**AC5 — `daemon --schedule --run-as-system` gets the same treatment:**
The daemon scheduled task has the same env-var problem. Apply the same env-var-baking and `--no-project-context` to the daemon task.

**AC6 — Tests:**
- Test that `nightly --schedule --run-as-system` (in a temp env) produces a task XML/command string containing `--no-project-context`, `--skip-import`, and all required env var names.
- Test that `nightly --schedule` WITHOUT `--run-as-system` does NOT include `--no-project-context` or `--skip-import` (no regression).
- Follow AI-Brains test conventions.

**AC7 — Migration path:**
- Existing `AI-Brains-Nightly` and `AI-Brains-Daemon` SYSTEM tasks should be re-scheduled. Document the migration: `ai-brains nightly --unschedule` then `ai-brains nightly --schedule --run-as-system`.
- The current manual workaround (`scripts/nightly-task.bat`) is replaced by the in-CLI fix.

**AC8 — CI gate passes:**
`cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit ; ledgerful verify --scope full`

## Design Notes

- **Wrapper script approach vs inline env vars:** A wrapper `.bat` file in `scripts/` is the simplest cross-platform-ish approach on Windows. The CLI can generate it at schedule time, parameterized with the current env values. Alternatively, `cmd /c "set VAR=val && ai-brains.exe ..."` works but is fragile with quoting. Prefer the wrapper-script approach: CLI writes `scripts\nightly-task.bat` (or `%TEMP%\ai-brains-nightly.bat`) with baked-in env vars, then registers the task to run that script.
- **Env var sourcing:** At schedule time, read from (in priority order): CLI args > User env vars > `.env` file in cwd. Bake the resolved values into the wrapper script. Warn if any required var is missing.
- **No Machine-level env vars:** Setting Machine-level env vars requires elevation, which the CLI can't assume. The wrapper script sidesteps this entirely.
- **Log output:** The nightly should log to a known path so failures are diagnosable. Consider `--log-format json` piped to `$env:USERPROFILE\.ai-brains\logs\nightly.log` or `.ledgerful\logs\nightly.log`.

## Files

- `crates/ai-brains-cli/src/commands/nightly.rs` — schedule logic
- `crates/ai-brains-cli/src/commands/daemon.rs` — schedule logic (same fix)
- `crates/ai-brains-cli/src/main.rs` — add `--dry-run` to nightly if missing
- `scripts/nightly-task.bat` — current manual workaround (to be replaced)
- `Docs/OPERATIONS.md` — update nightly scheduling docs
- `conductor/deferred.md` — update with T143 reference

## Out of Scope

- The Antigravity import failure when run as a different user (that's an Antigravity path-access issue, not a scheduling issue).
- Machine-level env var management (requires elevation; out of scope).
- The daemon's actual runtime behavior as SYSTEM (only the scheduling is fixed here).