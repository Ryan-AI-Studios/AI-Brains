# Track T112: Recall Scope Overhaul — Default to Project-Wide

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P1 — the single biggest UX issue. Recall silently scopes to one session, missing most memories.
**Source:** Non-destructive command audit 2026-06-23.

---

## Problem Statement

`ai-brains recall "backup"` returns zero results even when the project has 1721 memories containing that word. The root cause: `recall` in the CLI always passes `session_id` from the `AI_BRAINS_SESSION_ID` env var to the retrieval layer. Both `lexical_search` (FTS5) and `substring_fallback` (T105 LIKE) filter by `session_id` when it's `Some`, limiting results to the current session only.

The `--global` flag only clears `project_id` to `None` but does NOT clear `session_id`. So `recall --global` is still session-scoped — the T111 hint says "across all projects" but it's actually searching one session. This is misleading and breaks the core use case of recall.

The `effective_session_id` variable in `recall.rs:39-46` is computed but never passed to the retrieval layer — it's only used for graph provenance (MemoryPinned events). The retrieval layer receives `options.session_id` directly.

## Acceptance Criteria

**AC1:** By default (no `--global`, no `--session`), `recall` searches all memories in the current project (no `session_id` filter). This is the most intuitive default — users think "search my project's vault", not "search this session".

**AC2:** `--global` searches across ALL projects AND all sessions (both `project_id = None` AND `session_id = None`). The T111 hint for `--global` is now accurate.

**AC3:** A new `--session <SESSION_ID>` flag explicitly scopes recall to a specific session. This replaces the implicit env-var-based session scoping.

**AC4:** When `AI_BRAINS_SESSION_ID` is set, it does NOT automatically scope recall. The user must explicitly pass `--session` to scope by session. The env var is used only for graph provenance (MemoryPinned events), not for search filtering.

**AC5:** `sync query` (both pretty and NDJSON paths) is updated to match: default is project-scoped (no session filter), `--global` is project+session unscoped.

**AC6:** The `substring_fallback` (T105) now actually finds results when the project has memories containing the query string, because the `session_id` filter is no longer applied by default.

**AC7:** No regression in existing recall tests. Tests that relied on session-scoped search must be updated to use `--session` explicitly.

**AC8:** The T111 hint text is accurate: `--global` says "across all projects" and means it (no session filter).

## Design Notes

- **Files to change:**
  - `crates/ai-brains-cli/src/main.rs` — Add `--session <SESSION_ID>` flag to `Recall`. Remove `session_id` from the `Recall` env var binding (it should NOT auto-load from env for search purposes). Change the match arm: pass `session_id: None` by default, pass `Some(id)` only when `--session` is explicitly provided. For `--global`, pass both `project_id: None` and `session_id: None`.
  - `crates/ai-brains-cli/src/commands/recall.rs` — `RecallRunOptions.session_id` is already `Option<SessionId>`. The `effective_session_id` for graph provenance is computed from `options.session_id` OR generated — keep this for graph edges. But the search call to `recall()` should use `options.session_id` (which is now `None` by default).
  - `crates/ai-brains-cli/src/commands/sync.rs` — In the NDJSON path (line 426-432), don't auto-set `session_id` from env var. In the pretty path (line 487), already passes `session_id: None` — no change needed.

- **Backward compatibility:** Users who relied on session-scoped search (if any) must now pass `--session $AI_BRAINS_SESSION_ID`. This is a breaking change but the previous behavior was a bug — session-scoped search was never documented as the default.

- **Test updates:** Tests that set `AI_BRAINS_SESSION_ID` and expect recall to filter by it need to pass `--session` explicitly instead.

- **The `--session` flag should NOT have an env var binding** (`#[arg(long)]` only, no `env = "AI_BRAINS_SESSION_ID"`). The existing `session_id` field on `Recall` currently has `#[arg(long, env = "AI_BRAINS_SESSION_ID")]` — remove the `env` attribute and rename to `--session`.

## Files

- `crates/ai-brains-cli/src/main.rs` — Add `--session` flag, remove env binding from recall's session_id, clear session_id on --global.
- `crates/ai-brains-cli/src/commands/recall.rs` — Keep `effective_session_id` for graph provenance only.
- `crates/ai-brains-cli/src/commands/sync.rs` — NDJSON path: don't auto-set session_id from env.
- `crates/ai-brains-cli/tests/smoke.rs` — Update tests that rely on session-scoped recall.

## Tests (TDD)

**Red:** `recall__default_scope__searches_all_project_memories` — create a vault with memories in 2 different sessions (same project), run recall WITHOUT --session, assert results from both sessions are returned.

**Red:** `recall__global_flag__searches_all_projects_and_sessions` — create memories in 2 projects x 2 sessions, run `recall --global`, assert results from all 4 combinations are returned.

**Red:** `recall__session_flag__scopes_to_specified_session` — run `recall --session <id>`, assert only that session's memories are returned.

**Green:** Implement the scope changes. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- `cargo nextest run -p ai-brains-retrieval`
- Manual: `ai-brains recall "backup" --no-bridge` on the current vault returns results (previously returned empty).
- Manual: `ai-brains recall "audit" --no-bridge --global` returns results from all projects.
- Manual: `ai-brains recall "T96" --no-bridge --session a09b9b05-...` scopes to that session only.

## Out of Scope

- Changing the ChangeGuard bridge query (already ignores session_id — it's global by design).
- Adding a `--project` flag (project scoping is already handled by `AI_BRAINS_PROJECT_ID` env var).
- Rebuilding the FTS5 index.