# T218 — Semantic recall quality v2

- **Track ID:** T218-SemanticQualityV2
- **Phase:** Post-audit CLI quality series (T217–T232) — **P1 ranking / display honesty** after T221
- **Status:** ✅ **Completed** (PR #116 squash `fc4d370`, 2026-08-09)
- **Depends on:** T215 RRF hybrid + cosine floor 0.55 + ScoreKind ✅; T202 embedding status ✅; T211 `rerank_hits` single entry point ✅; T217 FTS rescue (independent, already closed)
- **Blocks / feeds:** Operator/agent trust in `recall --semantic` scores + topic relevance; residual T224 role-prefix display; residual T231 unified search UX
- **Category:** FEATURE / RETRIEVAL / UX
- **Source:** Non-destructive CLI audit 2026-08-05 — semantic usefulness **6** · **output quality 4** (topic drift + unreadable ~0.016 scores); T215 soft residuals; series README T218 row
- **Deferred absorbed:** deferred.md T215 residuals (hermetic `recall_full` hybrid e2e; F25 JSON fusion metadata; score display); series “Semantic drift / scores (quality 4) → T218”; placeholder draft F1–F5; live re-scan 2026-08-09 semantic-only drift at floor 0.55
- **Not absorbed:** Full ANN / HNSW productization (F27 stays soft); LLM / cross-encoder re-ranker; vault↔ledger RRF blend (**T211 F25** soft elsewhere); default-on semantic for `sync query`; nomic `search_query`/`search_document` re-embed product cutover (soft residual — requires vault re-embed); raw-cosine single-arm ranking (O1 — pin boosts swamp risk); clap 5; MSI; new production crates; progressive retrieval rewrite; T224 role strip (separate track)
- **Research date:** 2026-08-09 (live dogfood + code map + hybrid/RRF / ES weighted RRF / nomic docs / clap pin)
- **AI fold-in:** 2026-08-09 — AI1 affirms dual floor + display + hermetic ACs (architecture diagram). AI2 **M1–M5** accepted hard; **L1–L5** elevated docs/soft; **O1** declined; **O2–O4** soft notes. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. **Readable scores:** Stop showing raw RRF contributions (~`0.016` = `1/(k+1)`) as if they were relevance probabilities. Humans and agents must interpret rank / kind / optional cosine without reverse-engineering k=60.
2. **Fusion honesty in JSON:** Expose additive `score_kind` (+ optional cosine / fusion context) so tools do not treat all `score` floats as comparable cosines.
3. **Semantic-only anti-drift:** When **no BM25/FTS candidates** (post T217 rescue; **substring-only does not count as FTS** — F37/M1) and the dense arm would contribute alone, apply a **stricter cosine floor** so weak neighbors (e.g. off-topic DECISION pins at ~0.55–0.57) do not surface as answers. Keep the hybrid arm floor at **0.55** when any `source=="fts"` candidate exists (T215 SOOT).
4. **Hermetic hybrid e2e:** Prove `recall_full` hybrid path against a temp vault with synthetic embedding BLOBs via **query-vector injection seam** (F12/M4; httpmock only as fallback).
5. **No second final sort:** All post-blend pin/recency ranking remains **`rerank_hits` only** (T211 F40 / T215 F40).
6. **Capture independence:** Ranking/semantic remain retrieval (+ models only when `--semantic`); capture path untouched.
7. **Zero new production crates** (httpmock test-only carve-out only if injection seam not chosen).

## 2. Live baseline (re-scan 2026-08-09)

### 2.1 Dogfood — residual after T215

| Probe | Live result | Verdict |
|-------|-------------|---------|
| `recall "authentication flow" --semantic` | **Empty** + hint; `embedding.status=ok` | Floor working for pure topic-less query ✅ |
| `recall "path TOCTOU openat cap-std" --semantic --no-bridge` | **3 off-topic DECISION pins** (T183/T154/T164), `source=semantic`, **score≈0.016** | Semantic-only drift + unreadable score ❌ |
| Same + `--min-score 0.58` | **Empty** | Stricter floor kills noise ✅ |
| Same + `--min-score 0.57` | **1 residual** off-topic pin | Boundary ~0.57–0.58 on this corpus |
| `recall "path TOCTOU" ` (no `--semantic`) | Correct **T190** DECISION, BM25 ~−13.7 | Lexical precision OK; hybrid should prefer FTS when present |
| `recall "T215 RRF hybrid cosine floor ScoreKind" --semantic --format pretty` | FTS hits with **score=0.016**, honesty line when zero post-threshold semantic | Hybrid path OK; **score display still lies** ❌ |
| Pretty line format | `[score=0.016 \| session=…]` | RRF raw printed as “score” |

### 2.2 Root-cause map

| Cause | Effect |
|-------|--------|
| RRF score = `Σ 1/(k+rank)` with k=60 | Rank-1 alone ≈ **0.0164**; both-list consensus ≈ **0.0328** — looks near-zero |
| Pretty/JSON print raw `RecallHit.score` | Agents treat 0.016 as “almost irrelevant cosine” |
| No `score_kind` / cosine on contracts | Cannot distinguish BM25 / RRF / cosine / bridge |
| Cosine discarded at fuse (replaced by RRF) | No way to show sim or re-filter post-fuse |
| Floor 0.55 on full-table cosine | Dense arm always returns top-K weak neighbors when anything ≥0.55 |
| Semantic-only identity RRF | Weak neighbors rank 1..n with scores ~0.016 even when topic-off |
| **Substring fallback fills `local_hits`** (T105) | Naïve `local_hits.is_empty()` gate **skips** semantic-only floor when only substring matches exist (AI2 **M1**) |
| `semantic_search` always HTTP-embeds | Hermetic positive hybrid e2e needs **injection seam** or httpmock (AI2 **M4**) |
| Hermetic denylist misses score/RRF envs | Ambient `AI_BRAINS_SEMANTIC_*` / `RRF_K` pollutes AC floors (AI2 **M5**) |
| No hermetic `recall_full` hybrid e2e | T215 residual P3 |

### 2.3 Code / touch map

| Site | Role |
|------|------|
| `ai-brains-retrieval/src/hybrid.rs` | Dual floor helper (`SEMANTIC_ONLY_MIN` + env); pure `rrf_fuse` preserves cosine; gate helper `has_fts_arm` |
| `ai-brains-retrieval/src/semantic.rs` | Preserve cosine on hit; **injection seam** `semantic_search_with_embedding` (or equiv.) for hermetic (F12/M4); public `semantic_search` wraps fetch |
| `ai-brains-retrieval/src/recall.rs` | F14 pipeline: if **no `source=="fts"`** in local arm → apply `SEMANTIC_ONLY_MIN` before RRF (F37); plumb cosine |
| `ai-brains-retrieval/src/ranking.rs` | No second sort; ScoreKind unchanged for effective_score |
| `ai-brains-retrieval/src/recall.rs` (`RecallHit`) | Additive optional `cosine: Option<f64>`; update fused literal + all constructors + test helpers (F36/L5) |
| `ai-brains-contracts/src/recall.rs` | Additive `score_kind` (**narrow** wire set F5/M3) + `cosine`; optional response-level fusion note |
| `ai-brains-cli/src/commands/recall.rs` | Pretty display SOOT **branch on ScoreKind** (F6/L2); map contracts; F11 honesty stays |
| `ai-brains-cli/src/main.rs` | `--min-score` help: applies to hybrid arm **and** semantic-only gate when set (M2) |
| `ai-brains-cli/tests/common/mod.rs` | `AMBIENT_DENYLIST` + `AI_BRAINS_SEMANTIC_MIN_SCORE`, `AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE`, `AI_BRAINS_RRF_K` (M5) |
| Hermetic / unit | Dual-floor + substring-only gate unit (AC18); injection-seam hybrid e2e (AC10) |
| Docs | CAPABILITIES dual floor + floors model-calibrated (O3); CHANGELOG; PROTOCOL-COMPAT additive |

### 2.4 Deps

| Item | Pin / note |
|------|------------|
| clap | Workspace **4.5** (crates.io **4.6.6** as of research) — **no bump** |
| serde_json | **1.0** — no bump |
| rusqlite | **0.39.0** SQLCipher — **no bump** |
| Embedding model | Default **nomic-embed-text-v1.5** @ `AI_BRAINS_EMBEDDING_URL` — **no model swap DoD** |
| Zero new crates | Required — pure f64 math; no sqlite-vss / faiss / candle |
| Capture independence | Unchanged |

## 3. Research summary (2026-08-09)

| Finding | Application |
|---------|-------------|
| RRF remains industry hybrid default (ES/OpenSearch/Azure; k=60 still default) | Keep k=60 + equal weights as baseline; do not replace RRF with raw score mix |
| Weighted RRF available in ES 9.2+ retrievers | Soft residual only — equal weights v1 unless free micro-env `AI_BRAINS_RRF_WEIGHT_SEMANTIC` |
| Small corpora can prefer lower k (10–20) | Soft doc note; **do not** change default k without labeled eval (T215 F5 stays) |
| Field / title boost improves hybrid NDCG (Turnbull 2025; product-name boost) | Soft evaluate **first-line / DECISION-line term overlap** only if free; not hard DoD |
| Nomic task types `search_query` / `search_document` | Soft residual: prefix + vault re-embed is a product cutover, not this track’s DoD |
| Cosine thresholds are corpus-specific | Dual floor with env knobs; hermetic units prove gate, not live model |
| Never linear-combine raw BM25 with raw cosine | Affirm T215 F3 |
| Cross-encoder stage-2 | **Out** DoD (latency + new deps) |
| Score display honesty (clig / agent UX) | Rank label + score_kind + optional cosine beats opaque 0.016 |

## 4. Frozen decisions (F1–F42)

| ID | Decision |
|----|----------|
| **F1 — Surface** | Primary: **`ai-brains recall --semantic`**. Shared `recall_full`. **`sync query` stays `semantic: false`**. Display/metadata improvements apply to recall JSON/pretty when scores are RRF/hybrid; BM25-only pretty may keep raw rank polarity (document). |
| **F2 — Dual cosine floor (hard)** | Default hybrid arm floor remains **`SEMANTIC_MIN_COSINE = 0.55`** (env `AI_BRAINS_SEMANTIC_MIN_SCORE`). When **no FTS arm** (F37) and semantic arm is non-empty after the 0.55 filter, apply **second gate** **`SEMANTIC_ONLY_MIN_COSINE = 0.60`** (env `AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE`; invalid/missing → 0.60). All-below → empty semantic contribution; status `ok` if embed succeeded. |
| **F2b — `--min-score` override semantics (hard, AI2 M2)** | **`override replaces`** the relevant default(s) — same pattern as today’s `effective_semantic_min_cosine`. When `--min-score X` is set: hybrid arm floor = **X** and semantic-only gate floor = **X** (not `max(X, 0.60)`). So `--min-score 0.57` **can** re-admit [0.55,0.60) residual (operator intent); defaults without override keep 0.55 / 0.60. Document in clap help + CAPABILITIES. |
| **F3 — Dual floor placement** | Apply semantic-only gate **after** the 0.55 filter and **before** `rrf_fuse`, gated on **F37** (not raw `local_hits.is_empty()`). Do **not** raise the hybrid arm floor globally when any FTS hit exists. |
| **F4 — Preserve cosine** | Semantic hits retain **pre-fuse cosine** on additive `RecallHit.cosine: Option<f64>`. RRF still writes fused rank-score into `score`. Hybrid prefer-FTS content rule unchanged; cosine from semantic arm when available. |
| **F5 — JSON fusion honesty (hard, elevates T215 F25; AI2 M3)** | Additive on `RecallResult` (`default` + `skip_serializing_if`): **`score_kind`** wire closed set **only** what the pipeline emits: **`bm25` \| `rrf` \| `bridge`**. Map: `Bm25LowerBetter`→`bm25`, `HigherIsBetter`→`rrf` (includes fused hybrid **and** single-list identity RRF), `BridgeHigherIsBetter`→`bridge`. **Do not** emit wire kinds `cosine` or `hybrid` (dead / collides with `source`). Existing **`source`** remains the origin label (`fts` / `semantic` / `hybrid` / `substring` / `bridge` / …). Optional **`cosine`** when known (separate field — not a score_kind). Soft response-level **`fusion`**: if free, `{ "method": "rrf", "k": <effective rrf_k()> }` (L1 — **not** bare constant 60). |
| **F6 — Pretty score display (hard; AI2 L2)** | Branch on **`ScoreKind`**, never unconditional: **`HigherIsBetter`** (RRF path): primary **`rank=#n`** + **`sim=0.XX`** when cosine known; do **not** primary-print raw ~0.016. **`Bm25LowerBetter`**: keep existing `score={:.3}` (negative rank) — **no BM25 polarity break**. **`BridgeHigherIsBetter`**: keep readable raw bridge score (`score={:.3}`) under mixed lists; document mixed display. Soft tertiary `rrf=0.016` only if free. |
| **F7 — Raw score stability** | JSON `score` remains the machine value used today (RRF under `--semantic` after fuse, BM25 rank without). Do **not** rescale JSON `score` into fake 0–1 “confidence”. Interpretation: `score_kind` + optional cosine + pretty rank. |
| **F8 — RRF constants** | Keep **k=60**, equal weights, missing-list omits term (T215 F5/F38). Soft env weighted residual only. |
| **F9 — No second final sort** | Extend pipeline only before `rerank_hits`; F40 holds. |
| **F10 — Soft-fail unchanged** | Embed unreachable/error/no_stored_embeddings: empty semantic list, classified status, FTS continues (T202). |
| **F11 — Pretty embedding honesty** | Keep T215 F11 threshold honesty line. Soft always-on ok line remains residual (T215 F24). Document: `semantic_post_threshold_count` is **post-0.55 / pre-dual-floor** (L3). |
| **F12 — Hermetic e2e (hard; AI2 M4)** | **Preferred mechanism:** refactor **injection seam** — e.g. `semantic_search_with_embedding(conn, query_vec, …)` used by production `semantic_search` after `fetch_embedding`. Hermetic tests call the seam with synthetic query vectors + stored f32 LE BLOBs; **no network**. Fallback only: httpmock + `AI_BRAINS_EMBEDDING_URL` (test-only; already in workspace desktop). Touch map + Phase 2/4 must name the seam. |
| **F13 — Zero new production crates** | Pure f64 + existing cosine. httpmock only if fallback path chosen (test-only, not a product dep). |
| **F14 — Capture independence** | Unchanged. |
| **F15 — Determinism** | Same vault + config → same order; ties `memory_id` asc in `rrf_fuse` and `rerank_hits` (AI1 §3); sort emitted collections. |
| **F16 — Deps** | clap 4.5 workspace; no rusqlite 0.40; no clap 5. |
| **F17 — Contracts** | Additive only; PROTOCOL-COMPAT / fixture notes for extra fields. No required field renames. |
| **F18 — Title / first-line soft boost** | Soft: if free after hard path, small post-RRF boost when query tokens hit first line / DECISION line. Cap boost. Else residual. |
| **F19 — Weighted RRF** | Soft residual (env weights). Equal weights DoD. |
| **F20 — ANN** | Soft residual deferred.md. |
| **F21 — Nomic task prefixes** | Soft residual: document floors are **model/corpus calibrated** (both sides currently unprefixed — symmetric); future prefix cutover requires re-embed **and** floor re-tune (O3). Out of DoD. |
| **F22 — High pre-ship** | Invert BM25; raise global floor silently; rescale JSON score to fake confidence; second final sort; hard-fail embed; production unwrap; break T215 pure RRF units; gate dual floor on `local_hits.is_empty()` (substring trap); emit dead `score_kind` values; skip hermetic denylist for new envs. |
| **F23 — Docs** | CAPABILITIES: dual floor + env names + **override-replaces**, score_kind wire set, pretty rank/sim, cosine field, floors model-calibrated (O3), nomic residual; CHANGELOG minor; PROTOCOL-COMPAT additive. |
| **F24 — Skill one-liner** | Soft (T215 F29). |
| **F25 — Adaptive threshold / query-type floor** | **Declined** beyond dual floor F2 (affirm T215 F26). **Also decline O1** raw-cosine single-arm ranking as DoD (would break pin-boost near-tie math unless new scale rule). |
| **F26 — Privacy** | Reorder/filter only; no content mutation (T224 strips display elsewhere). |
| **F27 — Review** | FEATURE. Primary required. Cross-model recommended. |
| **F28 — Bridge / graph** | Outside RRF; ScoreKind inherit/polarity unchanged (T215 AC16/AC17). |
| **F29 — Symbol noise honesty** | Docs: symbols share embedding space; dual floor reduces but does not eliminate. |
| **F30 — FTS rescue interaction** | T217 rescue feeds **FTS** arm (`source=="fts"`) before RRF; dual floor only when **post-rescue FTS arm still empty** (substring-only does not clear the gate — F37). |
| **F31 — Series order** | After T221. Parallel-friendly with T219/T224 if no file conflict on `recall.rs` pretty. |
| **F32 — Manual dogfood** | TOCTOU semantic-only empty or on-topic only at default floors; authentication flow stays empty; readable pretty ranks; hybrid lexical query still surfaces T215/T190-class pins. |
| **F33 — Pin boosts** | CONSTRAINT/DECISION kind boosts still apply post-fuse (T211); dual floor does not replace pin authority. |
| **F34 — Residual map** | Weighted RRF; ANN; nomic prefixes; always-on ok pretty; skill; vault↔ledger F25; O1 raw-cosine arm → deferred.md. |
| **F35 — No auto-forget** | Rank/filter/display only. |
| **F36 — Constructor discipline** | Update all `RecallHit` sites for `cosine`: constructors, fused literal (`hybrid.rs`), ranking/cli test helpers (L5); no production `unwrap`/`expect`. |
| **F37 — Gate = no FTS source (hard, AI2 M1)** | Semantic-only second floor fires when the pre-RRF local arm has **zero hits with `source == "fts"`** (after T217 rescue). **Substring-only** (`source=="substring"`) and empty local → gate **on**. True FTS (incl. rescue ranks) → gate **off**. Helper preferred: `fn has_fts_arm(hits: &[RecallHit]) -> bool`. **Do not** use `local_hits.is_empty()` alone. |
| **F38 — Hermetic ambient denylist (hard, AI2 M5)** | Add to `AMBIENT_DENYLIST`: `AI_BRAINS_SEMANTIC_MIN_SCORE`, `AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE`, `AI_BRAINS_RRF_K`. Tests needing non-default floors set via `.env(...)` after strip. |
| **F39 — Clap help (hard, M2)** | `--min-score` help text: one-shot cosine floor for `--semantic` (replaces both hybrid-arm default 0.55 and semantic-only default 0.60 when set). Parallel naming: `effective_semantic_only_min_cosine` next to `effective_semantic_min_cosine` (O4). |
| **F40 — Protocol serde** | New fields `#[serde(default, skip_serializing_if = "Option::is_none")]`; required fields unchanged (AI1 §2). |
| **F41 — RRF list for fuse when substring-only** | When gate fires (no FTS), **RRF FTS list is empty** even if substring hits exist; substring hits merge **outside** RRF the same way today’s non-FTS lexical path works after fuse (bridge wins id; then fused; ensure substring-only hits are not dropped — if currently only `local_hits` feed RRF, restructure: RRF(fts_only, sem) then merge remaining substring hits by id). Lock: **substring never disables semantic-only floor; substring still surfaces in final blend.** |
| **F42 — AI fold-in complete** | §14 disposition frozen; plan-only until go. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| deferred.md T215 fusion metadata / score display | **Absorb** F5, F6, F7 |
| deferred.md hermetic recall_full hybrid e2e | **Absorb** F12 |
| Series semantic quality 4 / T218 placeholder | **Absorb** F1–F12 |
| Live semantic-only TOCTOU drift | **Absorb** F2–F3 |
| AI2 M1 substring gate trap | **Absorb** F37, F41, AC18 |
| AI2 M2 min-score ambiguity | **Absorb** F2b, F39, AC16 rewrite |
| AI2 M3 dead score_kind values | **Absorb** F5 narrow wire set |
| AI2 M4 hermetic mechanism | **Absorb** F12 injection seam |
| AI2 M5 ambient denylist | **Absorb** F38 |
| Weighted RRF | **Soft F19** |
| ANN | **Soft F20** |
| Nomic task prefixes + re-embed | **Soft F21** + CAPABILITIES line |
| Always-on ok pretty (T215 F24) | Soft residual |
| Title/first-line boost | **Soft F18** |
| O1 raw-cosine single-arm | **Decline F25** (pin-boost scale risk) |
| T211 vault↔ledger RRF F25 | **Decline** (other surface) |
| Cross-encoder / LLM re-rank | **Decline** |
| Adaptive per-query-type floor beyond dual | **Decline F25** |
| T224 role strip | **Out** (separate track) |
| clap 5 / MSI / rusqlite 0.40 | **Out** |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | No FTS arm + semantic cosines in [0.55, SEMANTIC_ONLY_MIN) → **no** semantic results after dual floor | Unit |
| **AC2** | No FTS arm + cosine ≥ SEMANTIC_ONLY_MIN → semantic results retained | Unit |
| **AC3** | FTS arm non-empty + cosine in [0.55, SEMANTIC_ONLY_MIN) → still eligible for RRF (hybrid arm floor only) | Unit |
| **AC4** | Pretty for HigherIsBetter does not primary-display raw ~0.016; shows rank (and sim when known); BM25 keeps score= | Unit helper |
| **AC5** | JSON `score_kind` ∈ {`bm25`,`rrf`,`bridge`} only; optional `cosine`; never emits wire `cosine`/`hybrid` kinds | Contracts + CLI unit |
| **AC6** | JSON `score` still RRF under `--semantic` (no fake 0–1 rescale) | Unit |
| **AC7** | Pure RRF both-list beats single still green (T215 regression) | Existing units |
| **AC8** | BM25-only `semantic: false` ranking polarity regression green | Existing units |
| **AC9** | Soft-fail embed path + F11 honesty unchanged | Existing + touch |
| **AC10** | Hermetic `recall_full` hybrid path via **injection seam** + synthetic BLOBs (no network) exercises fuse | Hermetic |
| **AC11** | CAPABILITIES + CHANGELOG: dual floor, override-replaces, score_kind wire set, pretty display, cosine field, floors model-calibrated | Grep / review |
| **AC12** | Full CI gate green; no production unwrap/expect | Gate |
| **AC13** | Manual dogfood: TOCTOU semantic-only empty or non-noise; authentication flow empty; pretty ranks readable | Manual |
| **AC14** | Soft: first-line boost if shipped | Soft |
| **AC15** | Soft: response-level `fusion` with **effective** `rrf_k()` if shipped | Soft |
| **AC16** | `--min-score X` **replaces** both gate defaults with X (not max); unit for hybrid arm and semantic-only path | Unit |
| **AC17** | PROTOCOL-COMPAT / additive field honesty; serde default + skip_serializing_if | Docs / fixture |
| **AC18** | **M1:** substring-only local arm (no `source=="fts"`) **does** apply SEMANTIC_ONLY_MIN; weak [0.55,0.60) dropped; substring hits still merge into blend | Unit |
| **AC19** | **M5:** AMBIENT_DENYLIST includes the three score/RRF env keys | Unit / grep |
| **AC20** | **M4:** injection seam callable without HTTP (compile + hermetic) | Hermetic / unit |

## 7. Non-goals

- Packaging (MSI / notarization / App Store)
- clap 5 multi-heading
- LLM / cross-encoder re-ranking
- Default-on semantic for `sync query`
- Full vault↔ledger interleaved RRF (T211 F25)
- Weighted RRF as DoD
- ANN / vector index productization
- Embedding model replacement or nomic task-prefix re-embed cutover as DoD
- Raw-cosine single-arm ranking (O1) as DoD
- Progressive / governed claim ranking changes
- Auto-forget pins; privacy filter changes
- T224 role-prefix strip; T219 preflight pretty wall; T231 unified search product merge

## 8. Risk & blast radius

| Risk | Mitigation |
|------|------------|
| Semantic-only min too high → empty true hits | Env + `--min-score` replace; AC2; docs; 0.60 from live boundary ~0.57–0.58 |
| Breaking agents that parse only `score` | Keep raw `score`; additive fields only; CAPABILITIES note |
| Pretty BM25 regression | F6 branch on ScoreKind only |
| Losing cosine at fuse | F4 field before overwrite |
| Dual floor applied when true FTS present | F37 `has_fts_arm`; AC3 |
| Substring disables strict floor | F37/F41 + AC18 |
| Dead score_kind values confuse agents | F5 narrow wire set + AC5 |
| Constructor miss on new field | F36 enumerate sites |
| Hermetic needs live embed | F12 injection seam + AC10/AC20 |
| Ambient env pollutes floors | F38 + AC19 |

## 9. Verification plan

1. Red units: dual floor AC1–AC3, AC16, **AC18**; pretty helper AC4; contracts serde AC5–AC6; denylist AC19.
2. Green hybrid.rs / recall.rs (F37/F41) / semantic injection seam / CLI pretty / contracts.
3. Hermetic AC10 + AC20 via injection seam (no network).
4. Targeted: `cargo nextest run -p ai-brains-retrieval -p ai-brains-cli -p ai-brains-contracts --lib --bins` (+ hermetic tests).
5. Clippy those packages `-D warnings`.
6. Manual AC13.
7. Full gate + docs AC11/AC17.
8. Review convergence (primary + cross-model recommended).

## 10. Series / order

```text
… → ~~T217~~ → ~~T220~~ → ~~T221~~ → **T218** → T219 / T224 / peers
```

## 11. Open questions (resolved in freeze)

| Q | Resolution |
|---|------------|
| Raise global floor to 0.60? | **No** — dual floor only when **no FTS arm** (F2/F37). |
| Gate on `local_hits.is_empty()`? | **No** — gate on **no `source=="fts"`** (M1/F37); substring-only still strict. |
| Rescale JSON score to 0–1? | **No** — misleading; use score_kind + cosine + pretty rank (F6/F7). |
| `--min-score` max vs replace? | **Replace** defaults (F2b) — matches live dogfood / T215 helper. |
| Wire `score_kind` include cosine/hybrid? | **No** — narrow `bm25\|rrf\|bridge` (M3/F5); cosine field separate; source carries hybrid. |
| Hermetic mechanism? | **Injection seam preferred** (M4/F12); httpmock fallback only. |
| Weighted RRF DoD? | **No** — soft F19. |
| Nomic prefixes this track? | **No** — soft F21 (re-embed cost); CAPABILITIES calibration note. |
| Raw-cosine single-arm (O1)? | **Decline** — RELEVANCE_SCALE pin-boost risk. |
| Title boost hard? | **No** — soft F18 if free. |

## 14. AI fold-in disposition (2026-08-09)

| Source | Item | Disposition |
|--------|------|-------------|
| **AI1** | Dual floor + RRF preserve cosine + pretty rank/sim + hermetic e2e architecture | **Affirmed** (core F2–F12) |
| **AI1** | T217 post-rescue gate | **Affirmed** F30; tightened by F37 |
| **AI1** | Additive serde / required fields stable | **Affirmed** F40 |
| **AI1** | Deterministic memory_id ties | **Affirmed** F15 |
| **AI1** | CAPABILITIES dual-floor docs | **Affirmed** F23 |
| **AI2 M1** | Substring-only disables strict floor via `local_hits` non-empty | **Accept hard** F37, F41, **AC18** |
| **AI2 M2** | `--min-score` replace vs max ambiguity | **Accept hard** F2b (replace), F39 help, AC16 rewrite |
| **AI2 M3** | Dead wire kinds `cosine`/`hybrid` | **Accept hard** F5 narrow to `bm25\|rrf\|bridge` |
| **AI2 M4** | Hermetic needs injection seam or httpmock | **Accept hard** F12 seam preferred + AC10/AC20 |
| **AI2 M5** | AMBIENT_DENYLIST missing score/RRF envs | **Accept hard** F38 + AC19 |
| **AI2 L1** | fusion.k must be effective `rrf_k()` | Soft AC15 |
| **AI2 L2** | Pretty branch on ScoreKind; bridge keeps raw score | **F6 hard** |
| **AI2 L3** | semantic_post_threshold_count = post-0.55 pre-dual | Doc F11 |
| **AI2 L4** | CAPABILITIES/CHANGELOG/PROTOCOL-COMPAT | F23 / AC11 / AC17 |
| **AI2 L5** | Constructor + fused literal updates | F36 |
| **AI2 O1** | Raw-cosine single-arm ranking | **Decline** F25 (pin scale risk) |
| **AI2 O2** | Weighted RRF / rank_window | Soft F19 (already) |
| **AI2 O3** | Floors model-calibrated note | Soft F21 + CAPABILITIES line |
| **AI2 O4** | Naming `effective_semantic_only_min_cosine` | F39 |

**Not accepted as product change:** force max() for `--min-score`; emit dead score_kind values; gate on full `local_hits` empty; production new crates; O1 single-arm cosine DoD.
