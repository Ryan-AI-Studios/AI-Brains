# T215 Review Log — Semantic recall relevance

## Scope

- **Track:** T215-SemanticRecallRelevance
- **Branch:** `track/T215-semantic-recall-relevance`
- **Category:** FEATURE
- **Implemented:** ScoreKind polarity (bridge M1); RRF hybrid when `--semantic`; cosine floor 0.55; F9 candidate depth; F16 semantic `updated_at`; F14 pipeline; F11 pretty honesty; soft `--min-score`

## Internal review rounds

| Round | Verdict | Notes |
|-------|---------|-------|
| R1 | NEEDS_FIX | F-01 HIGH graph boost swamp; F-02 MED AC8 honesty gate; F-03 MED floor helper not in prod |
| R2 | **CLEAN** | F-01/F-02/F-03 verified fixed; residual lows only (spec text lag, CAPABILITIES graph row, substring honesty unit) |

### R1 findings disposition

| ID | Sev | Disposition |
|----|-----|-------------|
| F-01 | High | **verified_fixed** — `graph_neighbor_stored_score` divides HigherIsBetter boost by RELEVANCE_SCALE |
| F-02 | Medium | **verified_fixed** — `should_show_semantic_threshold_honesty` requires lexical fts/substring/hybrid |
| F-03 | Medium | **verified_fixed** — `semantic_search` uses `filter_by_cosine_floor` |
| F-04 | Low | open residual — no full `recall_full` e2e unit (wiring reviewed) |

## Cross-model review

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| R1 | Codex gpt-5.4 | rate-limited | Usage limit until ~2026-08-07 |
| R1 | Claude Sonnet 4.6 high | **PASS WITH DEFERRED P3** | 0 P0–P2; P3 process (conductor/deferred closeout) + hermetic e2e residual |

Raw: `review.claude.md`. Fresh final gate — no code findings above low.

## Gates

| Gate | Result |
|------|--------|
| nextest retrieval+cli (post-fix) | 688 passed |
| clippy workspace -D warnings | clean |
| nextest --workspace | **2153 passed**, 1 skipped |
| cargo deny check | ok |
| cargo audit | 19 allowed warnings only (pre-existing) |
| Manual AC12 | topic-drift query empty post-floor; `--min-score` in help |
| CI (PR #96) | **Win/Linux/macOS green** → squash-merged `b5cdc98` |

## Residual lows / soft deferrals

- Spec F13 text lag (score formula wording vs ScoreKind-aware stored delta)
- CAPABILITIES graph-boost row could note F-01 scaling
- Substring-only honesty unit not added (fts/hybrid covered)
- Soft: F24 always-on ok pretty; F25 JSON fusion metadata; weighted RRF; ANN; T211 F25 vault↔ledger blend
