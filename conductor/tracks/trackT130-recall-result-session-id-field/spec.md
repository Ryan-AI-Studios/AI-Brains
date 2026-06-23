# Track T130: `recall` Result Session-ID Field

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P2 — data completeness; results don't show which session they came from.
**Source:** v0.1.1 verification opportunity #9.

---

## Problem Statement

T112 made `recall` default to project-wide search (across all sessions). However, the JSON results don't include `session_id` per hit:

```json
{"results": [{"memory_id": "...", "content": "...", "source": "fts", "score": -2.0}], "session_id": "..."}
```

The top-level `session_id` is the effective session for graph provenance, not the session of each result. With cross-session search as the default, users need to know which session each result came from. Without per-result `session_id`, users can't:
- Distinguish results from different sessions in the output.
- Filter results by session after the fact.
- Build session-aware UIs on top of recall.

## Acceptance Criteria

**AC1:** Each result in the `results` array of the JSON output includes a `session_id` field with the session ID of the memory that matched.

**AC2:** The pretty format output shows the session ID (shortened to 8-char prefix) for each result: `[score=-2.0 | session=a09b9b05] content...`

**AC3:** The NDJSON output (for `sync query` and `recall --format ndjson` if it exists) includes `session_id` per record.

**AC4:** The `RecallHit` struct in `ai-brains-retrieval` and the `RecallResponse` in `ai-brains-contracts` are updated to include `session_id: Option<String>` per hit.

**AC5:** The top-level `session_id` in the JSON response (the effective session for graph provenance) is renamed to `effective_session_id` to disambiguate from per-hit session IDs. This is a contract change — update consumers.

**AC6:** No regression in recall test assertions (update tests that check the JSON shape).

## Design Notes

- **Files:**
  - `crates/ai-brains-retrieval/src/` — `RecallHit` struct: add `session_id: Option<SessionId>`. Populate it from the memory projection query (the `session_id` column is already in the projection).
  - `crates/ai-brains-contracts/src/recall.rs` — `RecallResponse` result item: add `session_id: Option<String>`.
  - `crates/ai-brains-cli/src/commands/recall.rs` — Map `RecallHit.session_id` to the response. Update pretty format to show session prefix.
  - `crates/ai-brains-cli/src/commands/sync.rs` — NDJSON output: include `session_id` from the hit.
- **Contract change:** This changes the `RecallResponse` JSON shape. Per AGENTS.md Contracts section: update affected crate docs, `ai-brains-contracts` types, CLI consumers, and `Docs/` references.
- The `effective_session_id` rename is a breaking change for any consumer parsing the top-level `session_id`. Document in changelog.
- The `session_id` field in `RecallHit` should be `Option` because some memories (e.g., system-level memories) might not have a session.

## Files

- `crates/ai-brains-retrieval/src/` — `RecallHit`: add `session_id`.
- `crates/ai-brains-contracts/src/recall.rs` — Response item: add `session_id`, rename top-level to `effective_session_id`.
- `crates/ai-brains-cli/src/commands/recall.rs` — Map field, update pretty format.
- `crates/ai-brains-cli/src/commands/sync.rs` — NDJSON: include per-hit `session_id`.
- `Docs/` — Update any API/contract references.

## Tests (TDD)

**Red:** `recall_json__each_result_has_session_id` — ingest memories in 2 sessions, run `recall --format json`, parse JSON, assert each result has a `session_id` field matching the source session.

**Red:** `recall_pretty__shows_session_prefix` — run `recall --format pretty`, assert each result line contains `session=xxxxxxxx` (8-char prefix).

**Green:** Add `session_id` to `RecallHit` and response. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- `cargo nextest run -p ai-brains-retrieval`
- `cargo nextest run -p ai-brains-contracts` (if tests exist)
- Manual: `ai-brains recall "backup" --format json | jq '.results[0].session_id'` → returns a UUID.

## Out of Scope

- Adding `project_id` per hit (it's already implied by the project-scoped search; could be future track).
- Adding `timestamp` per hit (separate track if needed).
- Changing the recall scoring algorithm.
- Adding session-level grouping in the output (flatten is fine for now).