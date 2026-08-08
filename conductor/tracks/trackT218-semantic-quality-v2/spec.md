# T218 — Semantic recall quality v2

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit — `recall "authentication flow" --semantic` → SQLCipher/DataKey/pipe hits; scores ~0.016 unreadable
- **Scores:** usefulness 6 · **output quality 4**
- **Category:** FEATURE / RETRIEVAL
- **Depends on:** T215 RRF + floor 0.55; T202 embedding status
- **Absorbs:** T215 soft residuals (fusion metadata, weighted RRF, hermetic e2e)

## Objective

Raise **topic relevance** of semantic/hybrid results and make scores human/agent interpretable without claiming ANN productization.

## Frozen direction (draft)

| ID | Decision |
|----|----------|
| F1 | Pretty/JSON expose fusion honesty: `score_kind`, optional `rrf_rank` / cosine when available (T215 F25) |
| F2 | Evaluate **title/first-line bias** or DECISION/CONSTRAINT soft boost in hybrid post-RRF (not full T211 blend) |
| F3 | Revisit floor vs over-drop; document; optional query-type adaptive floor **declined** unless data proves |
| F4 | Hermetic e2e `recall_full` hybrid fixture (T215 residual) |
| F5 | Human score display: map BM25/RRF to 0–1 or “rank #n” for pretty |

## Non-goals

Full ANN index; cloud embeddings; vault↔ledger RRF (T211 F25 stays soft elsewhere).
