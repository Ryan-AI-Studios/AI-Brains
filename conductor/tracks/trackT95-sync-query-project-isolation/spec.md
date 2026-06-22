# Track T95: Strict Project Isolation in `sync query` (Pretty Path)

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P1 — cross-project context pollution in the default code path.
**Source:** `C:\dev\testing_report.md` Item 1; confirmed by code audit 2026-06-22.

---

## Problem Statement

`ai-brains sync query "<q>"` has two code paths:

- **ndjson path** (`sync.rs:432`): passes `project_id: Some(project_id)` to `recall` — correctly scoped.
- **pretty path** (`sync.rs:478`): passes `project_id: None` to `recall` — **global, unscoped**.

The pretty path is the default (runs when `--format ndjson` is not specified). As reported, this causes memories from unrelated projects (e.g. `/home/ryan/dev/Sneaky-Browse/...`) to leak into the current project's query results.

`lexical_search` and `semantic_search` already filter on `project_id` when it is `Some(pid)`. The bug is that the pretty path passes `None`.

ChangeGuard's bridge already provides `project_id` to ai-brains (`src/bridge/notify.rs:25`, `BridgeRecord.project_id`). The fix is on the ai-brains side: resolve the current `project_id` in the pretty path the same way the ndjson path does, and pass `Some(project_id)` instead of `None`.

## Acceptance Criteria

**AC1:** `ai-brains sync query "cozo"` (pretty, default) scopes vault recall to the current `AI_BRAINS_PROJECT_ID`. Memories from other projects do not appear in results.

**AC2:** `ai-brains sync query "cozo" --format ndjson` continues to scope correctly (no regression).

**AC3:** A `--global` flag is added to `sync query` that explicitly opts in to unscoped recall (passes `project_id: None`). When `--global` is not set, the pretty path uses `Some(project_id)`.

**AC4:** When `AI_BRAINS_PROJECT_ID` is unset and `--global` is not set, the pretty path falls back to the `default-project` behavior (same as ndjson path) — it does NOT silently go global.

## Design Notes

- The fix is in `crates/ai-brains-cli/src/commands/sync.rs:run_query`, pretty branch (~line 470-487). The ndjson branch (~line 415-422) already shows the correct pattern: read `AI_BRAINS_PROJECT_ID` env var, parse to `ProjectId`, pass `Some(project_id)`.
- The `--global` flag should be a new clap arg on the `sync query` subcommand. When set, pass `project_id: None` to `recall`.
- The ChangeGuard `ledger search` call (line 494) is inherently repo-scoped (it indexes only the current working tree) — no change needed there.

## Files

- `crates/ai-brains-cli/src/commands/sync.rs` — fix pretty path, add `--global` flag, plumb through.
- `crates/ai-brains-cli/src/main.rs` — add `--global` arg to `SyncQuery` variant (if not already a global).

## Tests (TDD)

**Red:**
- `test_sync_query_pretty_scoped_to_project`: ingest a memory under project A, set `AI_BRAINS_PROJECT_ID=B`, run `sync query "test"` (pretty), assert no results from project A.
- `test_sync_query_global_flag_returns_all`: same setup, run `sync query "test" --global`, assert results from project A appear.

**Green:** Fix the pretty path to resolve and pass `project_id`. Add `--global` flag.

## Verification

- `cargo nextest run -p ai-brains-cli test_sync_query_pretty test_sync_query_global`
- Manual: `ai-brains sync query "cozo"` from the AI-Brains repo — confirm no `/home/ryan/dev/Sneaky-Browse/` paths in results.
- Manual: `ai-brains sync query "cozo" --global` — confirm cross-project results appear (opt-in).

## Out of Scope

- Changes to ChangeGuard's bridge layer (already provides `project_id`).
- Changes to `recall` or `preflight` project scoping (those already scope correctly when `project_id` is `Some`).
- A unified query interface (Item 4 of the report — product decision, not a bug).