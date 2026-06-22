# Track T105: Recall Fallback for Small Vaults

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P2 — empty results on common terms erodes user trust in recall.
**Source:** Systematic command test 2026-06-22.

---

## Problem Statement

`ai-brains recall "backup"` on a project with 54 memories returns zero results. FTS5 uses BM25 ranking which requires sufficient corpus size for meaningful token matching. On small vaults, common terms that exist in the corpus may not match because FTS5's internal tokenization or stemming doesn't find them. Users see "No results for 'backup'" and assume recall is broken.

The current flow: FTS5 query → if empty, return empty. There is no fallback to substring search or relaxed matching.

## Acceptance Criteria

**AC1:** When FTS5 returns zero results, the recall pipeline falls back to a `LIKE '%query%'` substring search on the `content` column of the memory projection table.

**AC2:** The fallback search is limited to the same `limit` and `project_id` scope as the original query.

**AC3:** When the fallback fires, a `tracing::debug!` message is emitted: "FTS5 returned 0 results, falling back to substring search for '<query>'". No stderr noise by default.

**AC4:** The response includes a `source` field on fallback results set to `"substring"` (instead of `"fts"` or `"semantic"`) so callers can distinguish match quality.

**AC5:** If both FTS5 and substring search return zero results, the "No results" message suggests `--global` and `--semantic` as alternatives: `No results for '<query>'. Try --global to search all projects, or --semantic for embedding-based search.`

**AC6:** No regression in existing recall tests. The fallback only fires when FTS5 returns empty.

## Design Notes

- The fallback should be in `ai-brains-retrieval/src/lexical.rs` (the FTS5 search module), not in the CLI layer. The `recall` function in `ai-brains-retrieval` should try FTS5 first, then substring if empty.
- Substring search: `SELECT memory_id, content, source FROM memories WHERE content LIKE '%' || ? || '%' AND project_id = ? LIMIT ?`
- Sanitize the query for LIKE (escape `%` and `_` literals in the query string) to prevent wildcard injection.
- Performance: on small vaults (<1000 memories) this is fast. On large vaults, the fallback may be slow — add a `tracing::warn!` if the vault has >5000 memories and the fallback fires.
- The fallback is a separate query, not a modification of the FTS5 query. This keeps the CQRS boundary clean.

## Files

- `crates/ai-brains-retrieval/src/lexical.rs` — add `substring_fallback` function, call it when FTS5 returns empty.
- `crates/ai-brains-cli/src/commands/recall.rs` — update the "No results" message with suggestions.

## Tests (TDD)

**Red:** `recall__fts5_empty__substring_fallback_finds_match` — create a vault with a memory containing "backup" in content but not as an FTS5 token match (e.g. inside a code block), run recall, assert the memory is found via fallback.

**Green:** Implement substring fallback. Test passes.

## Verification

- `cargo nextest run -p ai-brains-retrieval`
- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains recall "backup" --no-bridge` on the current project returns results via fallback.

## Out of Scope

- Fuzzy matching (Levenshtein, trigram).
- Embedding-based fallback (already covered by `--semantic`).
- Tuning FTS5 tokenizer settings.
