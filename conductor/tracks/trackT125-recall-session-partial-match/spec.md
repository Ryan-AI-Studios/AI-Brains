# Track T125: `recall --session` Partial/Name Matching

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P2 — usability; users don't know full session UUIDs.
**Source:** v0.1.1 verification opportunity #4.

---

## Problem Statement

T112 added the `--session <SESSION_ID>` flag for explicit session scoping. However, users must provide a full UUID:

```
$ ai-brains recall "backup" --session a09b9b05-...
```

Users rarely know full session UUIDs offhand. They would need to query the vault or inspect `.env` to find the session ID. This makes session-scoped recall impractical for interactive use.

## Acceptance Criteria

**AC1:** `--session <prefix>` accepts a UUID prefix (first N characters) and resolves it to the full session ID by matching against sessions in the vault. e.g. `--session a09b` matches `a09b9b05-...` if only one session starts with `a09b`.

**AC2:** A minimum prefix length of 4 characters is required. Prefixes shorter than 4 chars are rejected with: `Session prefix too short; provide at least 4 characters to avoid accidental matches.` This prevents 1-2 char prefixes from accidentally matching a single session.

**AC3:** If multiple sessions match the prefix, `recall` prints an error listing up to 5 matching session IDs (capped) and asks the user to provide a longer prefix: `Ambiguous session prefix 'a09b'. Matching sessions (5 of 12 shown): a09b9b05-..., a09b1234-..., .... Provide more characters.` If more than 5 match, the count is shown but only 5 are listed.

**AC4:** If no sessions match the prefix, `recall` prints: `No session matching 'a09b'. Use 'ai-brains project list' to see sessions.`

**AC5:** `--session <full-uuid>` continues to work exactly as before (backward compatible). Full UUIDs (36 chars, valid format) skip prefix resolution.

**AC6:** `--session-last` (new flag) scopes recall to the most recently active session in the current project. This is a convenience for the common case of "search in my last session".

**AC7:** Session resolution queries the vault's session list (from the memory projection or event log), not just the current project's `.env`.

**AC8:** `--session` and `--session-last` are mutually exclusive (`conflicts_with`). Using both exits 1 with a clap conflict error.

## Design Notes

- **Files:** `crates/ai-brains-cli/src/main.rs` (add `--session-last` flag), `crates/ai-brains-cli/src/commands/recall.rs` (session resolution logic), `crates/ai-brains-store/src/` (query for session list).
- Session resolution flow:
  1. If `--session` value is a full UUID (36 chars, valid format) → use directly, skip prefix resolution.
  2. If `--session` value is < 4 chars → reject with "prefix too short" error.
  3. If `--session` value is a prefix (4-35 chars) → query vault for all sessions starting with this prefix.
  4. If exactly 1 match → use it. If >1 → ambiguous error (cap display at 5). If 0 → not found error.
- For `--session-last`: query the vault for the most recent session by `last_turn_at` or max event timestamp in the current project.
- The session list query can use: `SELECT DISTINCT session_id FROM memory_projection WHERE session_id LIKE 'prefix%' ORDER BY session_id`
- `--session` and `--session-last` are mutually exclusive (`conflicts_with`).

## Files

- `crates/ai-brains-cli/src/main.rs` — Add `--session-last` flag, `conflicts_with` on `--session`.
- `crates/ai-brains-cli/src/commands/recall.rs` — Session resolution logic.
- `crates/ai-brains-store/src/` — Session list query (may need a new method).

## Tests (TDD)

**Red:** `recall__session_prefix__resolves_to_full_id` — vault with session `a09b9b05-1234-...`, run `recall --session a09b`, assert results are scoped to that session.

**Red:** `recall__session_prefix_ambiguous__errors_with_capped_matches` — vault with 12 sessions starting `a09`, run `recall --session a09b`, assert error listing at most 5 matches and showing "12 of 12 shown" count (or "5 of 12 shown").

**Red:** `recall__session_prefix_too_short__rejected` — run `recall --session ab`, assert error "prefix too short" and at least 4 characters required.

**Red:** `recall__session_prefix_no_match__errors` — run `recall --session zzzz`, assert "No session matching" error.

**Red:** `recall__session_last__scopes_to_most_recent` — vault with 2 sessions, last activity in session 2, run `recall --session-last`, assert results from session 2 only.

**Green:** Implement resolution. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains recall "backup" --session a09b` → resolves and scopes.
- Manual: `ai-brains recall "backup" --session-last` → scopes to most recent session.

## Out of Scope

- `--project` prefix matching (project IDs are already short and in `.env`).
- Tab completion for session IDs (separate shell integration track).
- Listing sessions as a command (could be `project sessions` in a future track).