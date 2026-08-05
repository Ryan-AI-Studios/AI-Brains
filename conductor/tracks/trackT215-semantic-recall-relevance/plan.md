# T215 Plan — Semantic recall relevance

Status: **Proposed / Expanded + AI fold-in** (plan-only until **go**). Spec: [spec.md](./spec.md).

## Goal

1. Fix `recall --semantic` topic drift (cosine floor + RRF hybrid).  
2. Fix **live bridge polarity demotion** (M1): Tantivy higher-is-better must not be negated as FTS5 rank.  
3. Extend T211 `rerank_hits` via `ScoreKind` only (F40 — no second final sort).  
4. FTS-only BM25 path stays correct; `sync query` stays `semantic: false`.

## Absorbed deferred / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| deferred.md T215 | audit semantic 6/5 | Hard DoD |
| T211 residual | RRF + polarity | F2, F6, F40 |
| AI1 **M1** | Bridge HigherIsBetter | F6 + AC17 |
| AI1 **M2** | F14 order wording | F14 rewrite |
| AI1 **M3** | semantic candidate depth | F9 |
| AI1 **M4** | semantic updated_at | F16 DoD |
| AI1 **M5** | FTS-empty semantic path | AC14 |
| AI1 **M6** | 5 constructors + `::semantic` | F42 |
| AI1 L1/L5 | hybrid pin AC + graph kind | AC15, AC16 |
| AI2 | Affirms floor/RRF/ScoreKind/pretty | Already F* |

**Not absorbed:** T211 F25 vault↔ledger blend; weighted RRF; ANN; LLM re-rank; default semantic on sync.

## Phases

### Phase 0 — Plan freeze

- [x] Live dogfood topic drift
- [x] Online RRF research
- [x] Spec F1–F42 + AC1–AC17
- [x] AI fold-in M1–M6 / L1–L8 disposition §14
- [x] User **go** before code / ledger TX

### Phase 1 — Ledger + red tests

- [x] `ledgerful doctor` / ledger status
- [x] `ledgerful ledger start T215-semantic-recall-relevance --category FEATURE --message "RRF hybrid + cosine floor + ScoreKind (bridge polarity + semantic topic drift)"`
- [x] Red pure units: AC1 RRF; AC2 floor; AC3 HigherIsBetter; AC4 FTS BM25; AC5 hybrid content; AC9 plan demotion; **AC14** FTS-empty semantic; **AC15** pin in hybrid space; **AC16** graph inherit; **AC17** bridge polarity
- [x] Red F11 pretty honesty helper

### Phase 2 — Green core (retrieval)

#### ScoreKind (all 5 sites — M6 / F42)

| # | Site | score_kind |
|---|------|------------|
| 1 | `RecallHit::fts` | `Bm25LowerBetter` |
| 2 | `RecallHit::substring` | `Bm25LowerBetter` |
| 3 | `RecallHit::graph` | **inherit parent** |
| 4 | `RecallHit::bridge` | **`HigherIsBetter` (M1)** |
| 5 | **`RecallHit::semantic` (new)** | `HigherIsBetter` — replace semantic.rs inline literal |

- [x] Add `ScoreKind` on `RecallHit`
- [x] Update all constructors + graph inherit parent kind
- [x] `effective_score(..., score_kind)` — Bm25: `-score`; HigherIsBetter: F6 rules (bridge raw = score; RRF/cosine = score × RELEVANCE_SCALE)
- [x] Update **all** `effective_score` unit call sites with `ScoreKind::Bm25LowerBetter` (L7) — keep T211 green
- [x] Green **AC17** bridge polarity

#### Hybrid + semantic

- [x] Prefer **`hybrid.rs`**: pure `rrf_fuse`, depth helper (F37)
- [x] `semantic_search`: floor F4; **pass candidate_depth** F9 (not final limit); **updated_at** F16 DoD
- [x] `recall_full` when semantic: **RRF(fts, semantic) → merge bridge (cap, bridge wins id) → graph → rerank_hits → truncate** (F14)
- [x] `recall_full` when !semantic: bridge → FTS → graph → rerank (no RRF); ScoreKind still correct
- [x] Green AC1–AC6, AC9, AC14–AC16

### Phase 3 — CLI pretty + docs

- [x] F11 pretty honesty line
- [x] Soft F32 `--min-score` if free
- [x] CAPABILITIES: RRF, k, floor, env, score meaning, **missing-from-list**, **bridge polarity**, recency on semantic
- [x] CHANGELOG minor
- [ ] Soft F29 (deferred) skill if free

### Phase 4 — Verify + review + ship

- [x] Targeted nextest retrieval + cli (+ contracts if any)
- [x] Clippy those packages `-D warnings`
- [x] Manual AC12
- [x] Full gate
- [x] Internal review → fix ≥medium
- [x] Cross-model review (Claude PASS WITH DEFERRED P3; Codex rate-limited)
- [ ] PR; conductor (in progress) Completed; deferred.md strike T215 + residuals
- [ ] `ai-brains pin`; ledger commit

## Touch map

| File | Change |
|------|--------|
| `retrieval/src/recall.rs` | ScoreKind; 5 constructors; F14 pipeline |
| `retrieval/src/ranking.rs` | kind-aware effective_score / rerank_hits |
| `retrieval/src/hybrid.rs` | **new** rrf_fuse (+ helpers) |
| `retrieval/src/semantic.rs` | floor; depth; updated_at; use `::semantic` |
| `retrieval/src/lib.rs` | mod hybrid; exports |
| `cli/.../recall.rs` | F11; soft --min-score |
| `Docs/CAPABILITIES.md` | F23 |
| `CHANGELOG.md` | Minor |
| tests | AC1–AC17 pure + hermetic |

## Regression guards

- T211 ranking tests + ScoreKind::Bm25LowerBetter args
- T202 embedding status suite
- T207 empty pretty suite
- sync query pretty + ndjson: `semantic: false` unchanged (L4 note: if semantic ever added, use `recall_full`)

## High pre-ship (F22)

- [x] No second final sort
- [x] Bridge not demoted (AC17)
- [x] FTS-only BM25 order green
- [x] Embed soft-fail exit 0
- [x] No production unwrap
- [x] All 5 constructors set kind
- [x] semantic_search depth ≠ final limit only

## Evidence on close

| AC | Evidence |
|----|----------|
| AC1–AC6, AC9, AC14–AC17 | unit names |
| AC7–AC8 | T202 + hermetic / helper |
| AC10 | CAPABILITIES + CHANGELOG |
| AC11 | full gate / CI |
| AC12–AC13 | manual / soft |

## Notes

- PowerShell: `;` not `&&`.
- Synthetic embedding bytes in tests — no live model for DoD.
- Do not edit `.ledgerful/` by hand.
- If live dogfood still noisy at 0.55, document env — do not silently change floor without unit proof.
