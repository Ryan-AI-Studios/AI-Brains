# T218 Plan — Semantic recall quality v2

**Status:** ✅ **Completed** (PR #116 `fc4d370`)
**Category:** FEATURE / RETRIEVAL / UX  
**Depends:** T215 ✅, T202 ✅, T211 ✅, T217 closed (independent)  
**Spec:** [spec.md](./spec.md) — includes AI fold-in **§14**

## Goal

1. **Dual cosine floor:** hybrid arm **0.55**; when **no FTS arm** (`source=="fts"` empty post-rescue), apply **SEMANTIC_ONLY_MIN = 0.60** before RRF.  
2. **Substring-only does not disable the strict floor** (M1/F37); substring still merges outside RRF (F41).  
3. **Readable pretty scores:** branch on ScoreKind — RRF → rank + sim; BM25/bridge keep honest raw.  
4. **JSON honesty:** additive `score_kind` ∈ {`bm25`,`rrf`,`bridge`} + optional `cosine` (no dead kinds).  
5. **`--min-score` replaces** both gate defaults when set (F2b).  
6. **Hermetic e2e** via **query-vector injection seam** (no network).  
7. Hermetic **AMBIENT_DENYLIST** includes score/RRF envs (M5).  
8. Zero new production crates; no second final sort; no fake JSON 0–1 confidence.

## Absorbed deferred / audit / research / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| deferred.md T215 residuals | fusion metadata, score display, hermetic e2e | Hard F5, F6, F12 |
| Series README T218 | quality 4 topic drift + scores | This track |
| Placeholder draft F1–F5 | fusion honesty, floor, e2e, display | Expanded F1–F42 |
| Live dogfood 2026-08-09 | TOCTOU noise at 0.55; empty at ≥0.58; scores 0.016 | F2–F3, F6 |
| Hybrid research 2026 | RRF k=60; no raw BM25+cosine mix | F8, F19 soft |
| clap crates.io 4.6.6 | workspace 4.5 | **no bump** F16 |
| **AI1** | Architecture dual floor + display + hermetic ACs | Affirmed |
| **AI2 M1** | Substring trap on `local_hits` | **F37, F41, AC18 hard** |
| **AI2 M2** | min-score replace vs max | **F2b replace + F39 help + AC16** |
| **AI2 M3** | Dead score_kind cosine/hybrid | **F5 narrow bm25\|rrf\|bridge** |
| **AI2 M4** | Hermetic mechanism | **F12 injection seam + AC10/AC20** |
| **AI2 M5** | Ambient denylist | **F38 + AC19** |
| **AI2 L1–L5** | fusion effective k; ScoreKind pretty; count docs; constructors | F5 soft / F6 / F11 / F36 |
| **AI2 O1** | Raw-cosine single-arm | **Decline** F25 |
| **AI2 O3** | Floors model-calibrated | F21 CAPABILITIES line |

**Not absorbed as DoD:** ANN; weighted RRF; nomic re-embed cutover; O1 single-arm cosine; cross-encoder; vault↔ledger F25; T224 role strip; clap 5; MSI; force max() on min-score.

## Live dogfood freeze (2026-08-09)

| Command | Observed |
|---------|----------|
| `recall "authentication flow" --semantic` | Empty (floor working) |
| `recall "path TOCTOU openat cap-std" --semantic --no-bridge` | Off-topic DECISIONs, score≈0.016 |
| Same + `--min-score 0.58` | Empty |
| Same + `--min-score 0.57` | 1 residual noise pin (override-replaces — F2b) |
| `recall "path TOCTOU"` (lexical) | Correct T190 DECISION |
| Pretty hybrid FTS hits | `score=0.016` unreadable |

## Research freeze (2026-08-09)

| Topic | Note |
|-------|------|
| RRF | Keep k=60 equal weights; missing list omits term |
| Dual floor | SEMANTIC_ONLY 0.60 when **no FTS source**; hybrid arm 0.55 |
| Gate | `has_fts_arm` not `local_hits.is_empty()` (M1) |
| min-score | **Replace** defaults (not max) |
| score_kind wire | `bm25\|rrf\|bridge` only; cosine is separate field |
| Display | rank + sim for RRF; branch ScoreKind |
| Hermetic | Injection seam preferred |
| Denylist | SEMANTIC_MIN / SEMANTIC_ONLY_MIN / RRF_K |
| clap | 4.5 pin; 4.6.6 latest — no bump |

## Implementation sketch (on go)

```rust
// hybrid.rs
pub const SEMANTIC_ONLY_MIN_COSINE: f64 = 0.60;
pub fn semantic_only_min_cosine() -> f64 { /* env AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE */ }
pub fn effective_semantic_only_min_cosine(override_min: Option<f64>) -> f64 {
    // F2b: override REPLACES default (same as effective_semantic_min_cosine)
    override_min.unwrap_or_else(semantic_only_min_cosine)
}
pub fn has_fts_arm(hits: &[RecallHit]) -> bool {
    hits.iter().any(|h| h.source == "fts")
}

// recall_full when semantic:
let fts_only: Vec<RecallHit> = local_hits.iter()
    .filter(|h| h.source == "fts").take(depth).cloned().collect();
let substring_rest: Vec<RecallHit> = local_hits.iter()
    .filter(|h| h.source != "fts").cloned().collect();
let mut sem = semantic_hits; // already post-0.55 floor
if !has_fts_arm(&local_hits) {
    let only_floor = effective_semantic_only_min_cosine(options.min_semantic_score);
    sem = filter_by_cosine_floor(sem, only_floor);
}
let fused = rrf_fuse(&fts_only, &sem, rrf_k()); // preserves cosine from sem arm
// blend: bridge → fused → substring_rest (id-dedupe) → graph → rerank
```

```rust
// semantic.rs — injection seam (F12/M4)
pub fn semantic_search_with_embedding(
    conn, query_vec: &[f32], limit, …, min_score_override
) -> Result<SemanticOutcome> { /* cosine vs stored BLOBs; set hit.cosine */ }

pub fn semantic_search(...) -> Result<SemanticOutcome> {
    let q = fetch_embedding(query)?;
    semantic_search_with_embedding(conn, &q, …)
}
```

```rust
// contracts RecallResult — additive:
// score_kind: Option<String>  // "bm25" | "rrf" | "bridge" only
// cosine: Option<f64>
// map: Bm25LowerBetter→bm25, HigherIsBetter→rrf, BridgeHigherIsBetter→bridge
// pretty: HigherIsBetter → rank=#n [sim=0.xx]; BM25/bridge keep score=
```

```rust
// cli/tests/common/mod.rs AMBIENT_DENYLIST +=
// AI_BRAINS_SEMANTIC_MIN_SCORE, AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE, AI_BRAINS_RRF_K
```

## Phases

### Phase 0 — Plan freeze

- [x] Live dogfood residual re-scan
- [x] Online hybrid/RRF/score-display research
- [x] Spec F1–F42 + AC1–AC20
- [x] AI fold-in M1–M5 / L1–L5 / O1–O4 disposition §14
- [x] Deferred / series rollup
- [x] User **go** before code / ledger TX

### Phase 1 — Ledger + red tests

- [x] `ledgerful doctor` / ledger status
- [x] `ledgerful ledger start T218-semantic-quality-v2 --category FEATURE --message "Dual semantic-only floor (no-FTS gate) + score_kind honesty + injection-seam hermetic e2e"`
- [x] Red pure units: AC1–AC3 dual-floor; **AC18 substring-only gate**; AC16 override-replaces
- [x] Red pretty helper AC4 (ScoreKind branch); contracts serde AC5–AC6
- [x] Red denylist AC19
- [x] Red hermetic scaffold AC10/AC20 (injection seam; may fail until green)

### Phase 2 — Green core (retrieval)

- [x] `SEMANTIC_ONLY_MIN_COSINE` + `effective_semantic_only_min_cosine` (replace semantics)
- [x] `has_fts_arm` + F37/F41: RRF list = FTS-only; substring merge outside
- [x] `RecallHit.cosine` + constructor + fused literal updates (F36)
- [x] `semantic_search_with_embedding` seam; production wraps fetch (F12)
- [x] `rrf_fuse` preserves cosine when semantic arm present
- [x] Green AC1–AC3, AC7, AC16, AC18, AC20 unit

### Phase 3 — Contracts + CLI pretty + docs

- [x] Additive `RecallResult.score_kind` + `cosine` (narrow wire set F5)
- [x] Map ScoreKind → `bm25|rrf|bridge` only
- [x] Pretty SOOT branch on ScoreKind (F6); sync query BM25 path unchanged
- [x] clap `--min-score` help (F39)
- [x] CAPABILITIES + CHANGELOG + PROTOCOL-COMPAT (F23; floors model-calibrated O3)
- [ ] Soft F18 first-line / AC15 fusion object if free — **skipped** (not free after hard path)

### Phase 4 — Hermetic e2e + denylist + verify + review + ship

- [x] F38 AMBIENT_DENYLIST three keys + AC19
- [x] Hermetic AC10 via injection seam + synthetic BLOBs
- [x] Targeted nextest retrieval + cli + contracts
- [x] Clippy those packages `-D warnings`
- [x] Manual AC13
- [x] Full gate
- [x] Internal review → fix ≥medium
- [x] Cross-model review (FEATURE) — final PASS WITH DEFERRED P3
- [x] PR; conductor Completed; deferred.md strike T218 + residuals
- [ ] `ai-brains pin`; ledger commit

## Touch map

| File | Change |
|------|--------|
| `retrieval/src/hybrid.rs` | SEMANTIC_ONLY_MIN + env; has_fts_arm; preserve cosine in fuse |
| `retrieval/src/recall.rs` | F37/F41 dual-floor gate; FTS-only RRF list; substring merge; plumb cosine |
| `retrieval/src/semantic.rs` | Set cosine; **injection seam** `semantic_search_with_embedding` |
| `retrieval/src/lib.rs` | Export new helpers / seam as needed |
| `contracts/src/recall.rs` | Additive score_kind (narrow) + cosine |
| `cli/src/commands/recall.rs` | Map fields; pretty ScoreKind branch |
| `cli/src/main.rs` | `--min-score` help F39 |
| `cli/tests/common/mod.rs` | AMBIENT_DENYLIST F38 |
| `Docs/CAPABILITIES.md` | Dual floor + override + score_kind + calibration |
| `Docs/PROTOCOL-COMPAT.md` | Additive note |
| `CHANGELOG.md` | Minor |
| tests | AC1–AC20 pure + hermetic |

## Regression guards

- T215 pure RRF / floor / ScoreKind / bridge polarity units
- T211 BM25 ranking + plan demotion
- T202 embedding status suite
- T207 empty pretty suite
- T217 rescue still feeds FTS (`source=="fts"`) before RRF
- `sync query` stays `semantic: false`; pretty BM25 polarity
- Substring fallback still surfaces when present (F41)

## High pre-ship (F22)

- [x] No second final sort
- [x] No global silent floor raise
- [x] No fake JSON 0–1 confidence rescale
- [x] Gate is `has_fts_arm` not `local_hits.is_empty()`
- [x] Wire score_kind only bm25|rrf|bridge
- [x] `--min-score` replaces (not max)
- [x] BM25-only polarity green
- [x] Embed soft-fail exit 0
- [x] No production unwrap
- [x] Cosine preserved when available
- [x] Denylist includes three new envs

## Evidence on close

| AC | Evidence |
|----|----------|
| AC1–AC3, AC16, AC18 | unit names |
| AC4–AC6 | pretty + contracts units |
| AC7–AC9 | T215/T211/T202 regressions |
| AC10, AC20 | hermetic injection-seam name |
| AC11, AC17 | docs grep |
| AC12 | full gate / CI |
| AC13–AC15 | manual / soft |
| AC19 | denylist unit/grep |

## Notes

- PowerShell: `;` not `&&`.
- Synthetic embedding bytes + injection seam — no live model for DoD.
- Do not edit `.ledgerful/` by hand.
- If live dogfood still noisy at 0.60 semantic-only, document env — do not silently change without unit proof.
- Coordinate with T224 if both edit `format_pretty_hit_line` — prefer ScoreKind-aware signature / wrapper.
