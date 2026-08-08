# T217 — FTS multi-token / natural-phrase rescue

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit 2026-08-05 — `recall "what did we decide about forget list"` → empty; short token / `--global` hits same vault
- **Scores:** usefulness 7 · **output quality 4**
- **Category:** FEATURE / UX
- **Depends on:** T90 FTS sanitize; T111 empty hint; T112 scope; T140 bridge sanitize
- **Absorbs:** Natural-language empty trap; optional OR/AND strategy docs

## Objective

Stop long natural-language FTS queries from returning **false empty** when shorter tokenizations would hit. Keep capture independence (no embeddings required on default path).

## Frozen direction (draft)

| ID | Decision |
|----|----------|
| F1 | On empty FTS with ≥3 tokens, run **fallback tokenization** (drop stopwords; try progressive token subsets or OR-join) before empty hint |
| F2 | Empty hint must mention “try fewer keywords” when multi-token empty after rescue |
| F3 | Do **not** auto-enable semantic on empty (T202 soft-fail stays separate) |
| F4 | Hermetic: multi-token empty under old path → hits under rescue |
| F5 | Scores/ranking remain BM25-honest after rescue |

## Non-goals

Semantic floor changes (T218); pretty default (T101 already TTY); ANN.

## Acceptance sketch

AC1 multi-token rescue finds known pin; AC2 single-token unchanged; AC3 empty truly empty still empty; AC4 hermetic + CHANGELOG.
