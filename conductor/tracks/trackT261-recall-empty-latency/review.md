# T261 review log — Recall empty-query latency

**Track:** `conductor/tracks/trackT261-recall-empty-latency`
**Category:** FEATURE / UX / RETRIEVAL
**FEATURE TX:** `4a317118-21c5-4667-9f8d-ae10157f20e2`
**Date:** 2026-08-17

## Scope

Zero-contentful `recall` / `search` / `sync query` vault arm is the existing T207
empty envelope. Gate at the top of `recall_full` plus `substring_fallback` before
`COUNT(*)`. `--semantic` emits closed-set `skipped` / `contentless_query` /
`endpoint=None`. Piped `recall -` trim-empty → `""`. `forget --match` stays
unfiltered. No DTO field, no `RecallOptions` field, no clap 5, no live `.env`,
no `cargo install`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC18 / F0–F19 / DoD | PASS |
| R1b | Independent explore | **PASS** (0 findings) |
| CX1 | Codex FEATURE `gpt-5.4` high | Product **PASS**. Sole P1 was process-timing (Phase 4 / registry still In Progress at review time). Same class as T257 CX1. |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| CX1-P1 | high (process) | Track artifacts still In Progress / Phase 4 unchecked at review time | `verified_fixed` | Product code had no correctness gap (Codex: “no product-scope correctness bug”). Full gate + `ledgerful verify --scope full` now recorded; registry Completed. |

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 empty `recall_full` no hits / embedding None | met | `recall_full__empty_query__no_hits__ac1` |
| AC2 whitespace no LIKE match-all | met | `recall_full__whitespace__no_hits__ac2` (triple-space fixture) |
| AC3 all-stopword / `what is the` empty | met | `recall_full__all_stopword__no_hits__ac3` |
| AC4 contentful still searches | met | `recall_full__contentful_still_searches__ac4` + T105 `"llo worl"` green |
| AC5 semantic skipped wire + no embed HTTP | met | `recall_full__semantic_contentless__embedding_skipped__ac5` (dead `:1` URL → skipped not unreachable) |
| AC6 CLI empty pretty Scope + `No results for ''` | met | `recall__empty_pretty__hint_no_hits__ac6` |
| AC7 CLI whitespace no hit lines | met | `recall__whitespace_pretty__no_hit_lines__ac7` |
| AC8 CLI stopword No results | met | `recall__stopword_pretty__no_hits__ac8` |
| AC9 CLI empty JSON `results==[]` + hint | met | `recall__empty_json__results_empty__ac9` |
| AC10 `search ""` alias chrome | met | `search__empty_pretty__alias__ac10` |
| AC11 piped `recall -` empty/whitespace exit 0 | met | `recall_stdin__piped_empty__short_circuit__ac11` |
| AC12 TTY `recall -` refuse | met | T86 hang guard left intact; no existing TTY hermetic to weaken |
| AC13 helper units + F19 contractions | met | three `is_contentless_query__*` unit tests |
| AC14 substring empty before COUNT | met | `substring_fallback__whitespace__empty_before_count__ac14`; T105 10k skip still green |
| AC15 `lexical_search` still MATCH stopwords | met | `lexical_search__all_stopword__still_matches__ac15` |
| AC16 no bridge/embed/graph on contentless | met | early return before sanitize/bridge; AC5 dead-port skipped |
| AC17 `--symbols` does not override F1 | met | `recall_full__symbols_contentless__still_empty__ac17` |
| AC18 live timing note | met | debug bin after green: `""` 533 ms empty; `"   "` 502 ms empty (was hits); `"the the the"` 430 ms empty (was hits); `"" --semantic` 410 ms `Embedding: skipped`; `"forget list"` 218 ms still hits. SQLCipher open ≥500 ms — live bar is join the `""` band, not stretch `<500`. |

## Targeted gates (observed)

- `cargo fmt --check` after rustfmt of `recall.rs` early-return
- `cargo clippy -p ai-brains-core -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings` exit 0
- core `is_contentless_query` 3/3; retrieval `recall_empty_latency` 8/8; CLI `recall_empty_latency` 6/6
- keep-green: T105 substring + 10k; T217 `lexical_rescue` 8/8; T260 retrieval+CLI; T207 `recall_empty_pretty_scope`; T86 `test_recall_reads_query_from_stdin`

## Full gate (observed)

- `.\scripts\dev-check.ps1` **[SUCCESS] CI Gate passed!** nextest **3064** (1 skipped); deny 0.20.2; audit 0.22.2; 19 allowed warnings
- `ledgerful verify --scope fast` passed (fmt/clippy/workspace nextest/deny/audit)
- `ledgerful verify --scope full` passed (fmt 2.1s / clippy 1.9s / nextest 133.4s / deny 2.5s / audit 2.7s)

## Residual / decline

- Skip CLI graph-vault open on contentless — soft §11 (SQLCipher open dominates)
- Skip T207 `<10` memory COUNT — keep small-vault sentence (F9)
- Leftover-project `--global` / preflight blender — **T264**
- Graph live projection — **T262**
- PATH `cargo install` — operator / F12
