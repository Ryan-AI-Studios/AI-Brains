# Track T111: Recall No-Results Hint

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — UX polish; "No results" message is unhelpful.
**Source:** Systematic command test 2026-06-22.

---

## Problem Statement

When `ai-brains recall` returns zero results, it prints: `No results for '<query>'. Try shorter terms or check spelling.` This message is unhelpful — it assumes the query is misspelled, not that the vault is sparse or the search mode is wrong. Users on small vaults or new projects see this frequently and don't know what to try next.

## Acceptance Criteria

**AC1:** When recall returns zero results, the message is contextual based on the search configuration:
- If `--semantic` was NOT used: `No results for '<query>'. Try --semantic for embedding-based search, or --global to search across all projects.`
- If `--semantic` WAS used: `No results for '<query>' (semantic search). Try --global to search across all projects, or check if the embedding model is running.`
- If `--global` WAS used: `No results for '<query>' across all projects. The vault may be empty or the query may not match any memories.`

**AC2:** When the project has fewer than 10 memories, an additional hint is shown: `This project has only N memories — results may be limited. Consider importing more sessions.`

**AC3:** The hint goes to stderr (not stdout) so it doesn't interfere with JSON parsing when piped.

**AC4:** In JSON format, the response envelope includes a `hint` field (optional, `skip_serializing_if = "Option::is_none"`) with the same hint text, so programmatic consumers can surface it.

**AC5:** No regression in existing tests that check for the "No results" message — update them to match the new message format.

## Design Notes

- The memory count for the project can be obtained from `QueryStore` (there's likely a `count_memories` or similar method, or use `SELECT count(*) FROM memories WHERE project_id = ?`).
- The hint logic should be in `recall.rs` (CLI layer), not in the retrieval layer — the retrieval layer should just return results, the CLI layer handles user-facing messages.
- Keep the hint concise — one line, not a paragraph.
- The `hint` field in `RecallResponse` is backward-compatible (optional, skipped when None).

## Files

- `crates/ai-brains-cli/src/commands/recall.rs` — replace the "No results" eprintln with contextual hints, add memory count check.
- `crates/ai-brains-contracts/src/recall.rs` — add `hint: Option<String>` to `RecallResponse`.

## Tests (TDD)

**Red:** `recall__no_results__hint_suggests_semantic_and_global` — run recall with no results, assert stderr contains "--semantic" and "--global".

**Green:** Implement contextual hints. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- `cargo nextest run -p ai-brains-contracts`
- Manual: `ai-brains recall "nonexistent" --no-bridge` → hint suggests --semantic and --global.
- Manual: `ai-brains recall "nonexistent" --semantic --no-bridge` → hint suggests --global and embedding model check.

## Out of Scope

- Implementing the substring fallback (T105 covers that).
- Adding a `--suggest` flag for query suggestions.
- Auto-suggesting similar terms (did-you-mean).
