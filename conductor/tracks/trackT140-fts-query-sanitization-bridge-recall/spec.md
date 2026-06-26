# Track T140: FTS Query Sanitization for Bridge and Recall

**Status:** Complete
**Owner:** Codex
**Priority:** P1 - punctuation-heavy prompts should never crash FTS-backed recall.
**Category:** BUGFIX
**Source:** ledgerful-web bootstrap feedback, 2026-06-25.

## Problem Statement

The AI-Brains bridge/query path can surface `fts5: syntax error near ","` when
natural-language prompts contain punctuation. Earlier tracks hardened dots,
hyphens, and colons, but comma-heavy prompts still exposed duplicated or
incomplete sanitization paths.

## Acceptance Criteria

- AC1: Comma/colon-heavy queries do not produce FTS5 syntax errors.
- AC2: `recall` and lexical retrieval use the shared sanitizer from
  `ai-brains-retrieval`.
- AC3: The sanitizer produces safe quoted terms from alphanumeric/underscore
  runs and drops punctuation operators.
- AC4: Tests cover comma-heavy bridge-style prompts.

## Implementation Notes

- Modify `crates/ai-brains-retrieval/src/fts_utils.rs`.
- Modify `crates/ai-brains-retrieval/src/lexical.rs` to remove the duplicate
  sanitizer.
- Add regression coverage in
  `crates/ai-brains-retrieval/tests/lexical_search_fts5_safety.rs`.

## Verification

- `cargo test -p ai-brains-retrieval fts_utils`
- `cargo nextest run -p ai-brains-retrieval lexical_search_survives_comma_heavy_prompt`
- `cargo clippy -p ai-brains-retrieval --all-targets -- -D warnings`
