# T211 — Sync query ranking + stale DECISION demotion

- **Track ID:** T211-SyncQueryRankingStaleness
- **Phase:** Post-T210 skill·CLI audit follow-ups (P1)
- **Status:** 📋 **Proposed / Expanded + AI fold-in** (plan-only until go)
- **Depends on:** T101/T207 recall pretty; T187 encrypt; T190/T193 path residuals (live stale sample); T204 help IA series; **T210** policy bootstrap closed (PR #93); progressive ranking precedent T152
- **Blocks / feeds:** Operator/agent trust in `sync query` + shared FTS recall order; **T215** owns semantic/embedding relevance (out here); `rerank_hits` is the single post-blend ranking entry point for T215 extension
- **Category:** FEATURE
- **Source:** Non-destructive skill/CLI audit 2026-08-04 — **sync query quality 5**; stale pre-T187 / plan-only DECISION pins outrank closed truth
- **Deferred absorbed:** deferred.md T211 placeholder; audit “stale DECISIONs”; T207 residual “ranking → T211/T215” (**T211 slice only**); soft T210 skill residual remains T210 (not ranking)
- **Not absorbed:** Semantic/hybrid embedding rank (**T215**); progressive query rewrite (already authority-ranked); LLM re-ranker; auto-forget/migrate DECISION pins; full vault↔ledger score fusion service; clap 5; MSI; **new crates into retrieval** (workspace `regex` not added); governed decision graph rewrite
- **Research date:** 2026-08-05 (expand + live re-scan + online)
- **AI fold-in:** 2026-08-05 — AI1 affirms F4–F12 core + invariants. AI2 **M1–M6** accepted; **L1–L2** elevated DoD; **L3** soft; **L4–L6** affirm; **L7** soft deferred.md. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. **Vault ranking honesty:** FTS hits are re-ranked with **pin-type authority** + **recency** so CONSTRAINT / closed DECISION pins beat plan-only / placeholder DECISION text and plain chat turns when topical match is similar.  
2. **Stale DECISION demotion/flag:** pin text that is plan-only, placeholder, or clearly superseded language is **demoted** and **flagged** in pretty (and structured when free) so agents do not treat draft decisions as current law.  
3. **Ledger preference when vault contradicts:** when `sync query` has ledger hits and the top vault hit is plan-class DECISION (or demoted), **prefer ledger section presentation** (ledger first or explicit “prefer ledger” banner) so shipped provenance is not buried under stale vault pins.  
4. **Shared pure helper:** ranking lives in `ai-brains-retrieval` (testable pure functions + optional `updated_at` from lexical), applied on the blended recall path so `sync query` and `recall` stay consistent.  
5. **No semantic work:** embedding blend, RRF, threshold — **T215**.  
6. **No progressive rewrite:** `query progressive` already ranks Decision/Conclusion by authority / valid-time / relevance (T152) — leave it.

## 2. Live baseline (re-scan 2026-08-05)

### 2.1 Audit signal — confirmed live

| Surface | Live result |
|---------|-------------|
| `sync query "path TOCTOU"` (main project) | Vault top = **plan-only / Expanded** DECISION (score better FTS); closed “ADR-0021 shipped” DECISION **second**; ledger has **T190 shipped** + **T193 residual** in a **separate section after** vault |
| `sync query` pretty path | Two hard sections: `--- AI-Brains Recall ---` then `--- Ledgerful Ledger Search ---` |
| Vault arm | `recall::run` with **limit 3**, `semantic: false`, `graph_boost: 0.1`, **`no_bridge: true`** always (ledger is the separate process, not bridge IPC) |
| NDJSON arm | `recall()` limit **5**, `no_bridge` flag respected |
| Lexical rank | `ORDER BY rank` (FTS5 BM25 only); **no** pin-type; **no** `updated_at` in `RetrievalMemory` |
| Progressive query | Authority 100 Decision / 90 Confirmed / 80 Active; **Stale conclusions never current truth** — different surface |
| Preflight legacy | Already uses `ORDER BY updated_at DESC` for DECISION/CONSTRAINT/HOTSPOT markers — pattern to mirror for type detection |
| Env friction | Local `.env` `AI_BRAINS_PROJECT_ID` (test-alias) overrides shell — out of T211 (T206 residual) |

### 2.2 Code / touch map (AI2-verified)

| Site | Role |
|------|------|
| `ai-brains-retrieval/src/lexical.rs` | SELECT `mp.updated_at` into `RetrievalMemory` (additive); substring path too (F16) |
| `ai-brains-retrieval/src/ranking.rs` (**new**) | Pure: `PinKind`, `StalenessClass`, `classify_pin_kind`, `classify_staleness`, **manual** `extract_track_tokens` (no regex crate), `rerank_hits` |
| `ai-brains-retrieval/src/recall.rs` | Plumb `updated_at` through **all** `RecallHit` constructors + graph SQL; after blend/graph, **before truncate**: `rerank_hits`; set plan flag / staleness on hit |
| `ai-brains-retrieval/src/lib.rs` | Export ranking helpers |
| `ai-brains-cli/src/commands/sync.rs` | **F37:** pretty path calls **`recall_full` directly** (not `recall::run`); inspect top hit for F12; ledger `--json` probe; section order; `--limit` (F27 DoD) |
| `ai-brains-cli/src/commands/recall.rs` | **L1/F37:** extract shared pretty render helper; F11 badge; preserve T207 empty-state path |
| `ai-brains-contracts` recall DTO | **Soft F26** additive `staleness` with `skip_serializing_if`; pretty badge is hard DoD |
| Hermetic | New `sync_query_ranking` + pure ranking units; isolation regression AC7 |
| Docs | CAPABILITIES § unified search; CHANGELOG minor; soft OPERATIONS |

### 2.3 Deps

| Item | Pin / note |
|------|------------|
| clap | Workspace **4.5** (resolves ~4.6.x; crates.io 4.6.5) — **no bump** |
| rusqlite | **0.39.0** SQLCipher bundled — **no bump** (0.40 residual elsewhere) |
| chrono | **0.4** — already used (`parse_from_rfc3339` in preflight) |
| regex | Workspace pin **1.12** exists, **not** a retrieval dep — **do not add** (F6 manual scan / F18) |
| Zero new crates | Required — pure heuristics; no reranker; no `regex` in retrieval |
| Capture independence | Ranking is retrieval-only; no models/embeddings |

## 3. Research summary (2026-08-05)

| Finding | Application |
|---------|-------------|
| [SQLite FTS5](https://www.sqlite.org/fts5.html) — `ORDER BY rank` is BM25; lower (more negative) is better; auxiliary `bm25()` exists | Keep FTS as **primary relevance**; re-rank is **post-filter**, not custom FTS C aux |
| Epistemic IR (2026 arXiv org-memory) — topical match ≠ epistemic state; **SUPERSEDES demotes**, DECISION stable until superseded | Staleness class + demotion; do not delete pins |
| RAG/outdated docs practice — boost recent timestamps; demote archived/superseded | Recency boost + plan-only demotion |
| Hybrid FTS+vector RRF (2026 patterns) | **Out → T215** (semantic track) |
| T152 progressive order | policy → lifecycle → valid_time → authority → relevance | Mirror **authority-then-relevance** spirit for **pins**, not claim governed Decision identity |
| clig.dev — human-first TTY; next action; honesty | Pretty badges + ledger-first when vault top is plan-class |
| Preflight marker convention | `DECISION:` / `CONSTRAINT:` / `HOTSPOT:` string classes already in product |
| Bridge vs ledger section | Pretty path does **not** use IPC bridge; ledger is `ledgerful ledger search` — keep that split; only reorder sections |

## 4. Frozen decisions (F1–F42)

| ID | Decision |
|----|----------|
| **F1 — Surface** | Primary user-visible surface is **`ai-brains sync query`**. Shared ranking also applies to **`recall`** via `recall_full` so order is consistent (same FTS arm). |
| **F2 — No semantic** | Do **not** enable semantic by default on sync query; do **not** implement RRF/embedding blend. **T215**. |
| **F3 — No progressive rewrite** | Leave `query progressive` / T152 authority ranking alone. |
| **F4 — Pin kinds (authority)** | Detect from content markers (case-insensitive, first match wins): `CONSTRAINT:` → Constraint; `DECISION:` → Decision; `HOTSPOT:` → Hotspot; else Other. **DoD:** strip leading `ASSISTANT: ` (same pattern as preflight) before marker match. |
| **F5 — Staleness classes (Decision only)** | Heuristic on DECISION text (ASCII lower): **Plan** if matches any of: `plan-only`, `placeholder`, `expanded`, `until go`, `not dod`, `planning`; **Shipped** if matches any of: `shipped`, `complete`, `closed by`, `pr #`, `squash-merged`, `verified_fixed`; **Unknown** otherwise. Plan **demotes**; Shipped **boosts** slightly; Unknown neutral. |
| **F6 — Sibling demotion (M1)** | Within one result set, if ≥1 **Shipped** Decision hit shares a **track token** with a **Plan** Decision hit, apply **extra demotion** to those Plan hit(s). Track tokens = `T` + ≥1 ASCII digits at word boundaries (**manual scan** in ranking.rs — **no `regex` crate** in retrieval; keeps F18). Case-insensitive `T`/`t`. |
| **F7 — Recency** | Lexical + substring include `mp.updated_at`. Recency is **tie-break / small boost** only — never sole authority. Parse RFC3339/ISO via existing chrono patterns; unparseable → neutral (boost 0). |
| **F8 — Composite order (M2)** | **Single composite sort replaces** the old “Some scores first, None last” bucket sort. For each hit: `base = score.unwrap_or(0.0)` (document: **None→0.0**); `effective = base + kind_boost + shipped_boost − plan_penalty − sibling_penalty + recency_boost`. Sort: **effective desc**, then `updated_at` desc (missing last), then **`memory_id` asc**. Kind/stale adjustments apply to substring/graph/bridge the same way. |
| **F9 — Boost magnitudes** | Kind: Constraint **+4.0**, Decision **+2.0**, Hotspot **+0.5**, Other **0**. Shipped Decision **+1.0**. Plan Decision **−3.0**. Sibling Plan **−2.0** additional. Recency: clamp age days `d` to [0, 365]; boost `1.0 * (1 - d/365)` when `updated_at` parses (not gated on FTS Some). Const block in ranking.rs. **AC1b** proves swing beats FTS gap. |
| **F10 — Flag, do not delete** | Never forget/mutate vault pins. Demotion is **rank + presentation only**. |
| **F11 — Pretty badge** | Demoted Plan (or sibling-demoted) Decision lines show **`[plan/stale?]`** before content. Shipped optional `[shipped]` only if free. |
| **F12 — Ledger preference (M4)** | When **not** `--no-bridge`: (1) run vault `recall_full` first; (2) probe ledger non-empty via **`ledgerful ledger search --json <query>`** — non-empty = process success **and** stdout parses as JSON array/object with ≥1 entry **or** ≥1 non-empty JSON line (implementer picks one and locks in AC); (3) if ledger non-empty **and** top vault hit is Plan-class Decision (or every vault Decision hit is Plan), **emit ledger section first**, then vault. Human ledger display may re-run without `--json` **or** pretty-print the JSON probe once (single call preferred if free). Banner (DoD when reorder fires): `Note: vault top hit is plan/stale; ledger results shown first.` Fail/empty/missing CLI → vault-only as today (no panic). |
| **F13 — Keep section model** | Do **not** require full interleave of ledger rows into vault hits for DoD. Section reorder + badge is enough. Soft full blend residual F25. |
| **F14 — Limits** | Default vault limit **5** (pretty was 3; ndjson already 5). Truncate **after** re-rank. |
| **F15 — JSON / contracts** | Pretty badge is hard DoD. Optional additive `staleness` on contracts RecallResult stays **soft F26**. |
| **F16 — All hit sources (M5)** | Re-rank all sources. Plumb `updated_at`: FTS/substring from SELECT; graph = second column in content `query_row` (or same SELECT); bridge = omit / neutral (BridgeRecord timestamp is not memory `updated_at` — do not fake). |
| **F17 — Graph/bridge scores** | Existing scores feed `base`; kind/stale still apply. |
| **F18 — Zero new crates / no clap bump** | clap 4.5, rusqlite 0.39 stay. **Do not** add `regex` to retrieval. |
| **F19 — Capture independence** | No models, no embeddings, no graph DB required for ranking. |
| **F20 — Series** | After T210. Parallel-ish with T215; **do not block** on T215. Before T212 labels. |
| **F21 — Hermetic (≥7)** | (1) Plan below Shipped same track + keyword; (2) CONSTRAINT above Other; (3) pretty badge; (4) sibling demotion; (5) memory_id ties; (6) `--no-bridge` ranks vault; (7) **AC1b** score-gap. Soft: ledger-first AC10. |
| **F22 — High pre-ship** | Auto-forget pins; hide all Decisions; non-deterministic order; production unwrap; progressive authority change; accidental semantic; isolation leak; dual sort (composite vs old None-last) left ambiguous. |
| **F23 — Docs** | CAPABILITIES: pin-type + recency + plan demotion + ledger-first + heuristic honesty; CHANGELOG minor. Soft OPERATIONS. |
| **F24 — Soft skill** | Optional agent skill one-liner. |
| **F25 — Soft full blend** | Structured ledger RRF with vault — residual deferred.md. |
| **F26 — Soft recall JSON staleness** | Additive `staleness: Option<String>` with skip_serializing_if — ship if free after pretty; not blocking. |
| **F27 — `--limit` DoD (L2)** | `SyncCommands::Query` gains `--limit` / `-l` default **5** (clap). Pretty + ndjson both use it. Unifies hardcoded 3/5 split. |
| **F28 — Soft age-only demotion** | Do **not** demote old Shipped by age alone. |
| **F29 — Soft governed migrate** | Pin → Approved decision conversion — out. |
| **F30 — Determinism** | Fixed boosts; final `memory_id` asc (changes equal-score insertion order — AC5). |
| **F31 — Review category** | FEATURE. Primary review required. Cross-model soft. |
| **F32 — Privacy** | Reorder only; no privacy filter change. |
| **F33 — FTS base** | Use raw FTS rank as `base` (typically negative; more negative = better BM25). Additive offsets (F9) are large enough to reorder near-ties and score-gaps (AC1b). Do not invert BM25. |
| **F34 — False positive honesty** | Badge `plan/stale?`; docs: content heuristic ≠ lifecycle fact. |
| **F35 — Residual map** | Semantic → T215; full blend → deferred; auto-forget → out; progressive → out. Residuals → **deferred.md** (ISSUES.md absent — L7 soft). |
| **F36 — T207 AC10** | Non-empty pretty Scope residual **not** absorbed. |
| **F37 — sync pretty path (M3+L1)** | **Required:** `run_query` pretty path calls **`recall_full` (or `recall`) directly** — **not** `recall::run` — so hits are inspectable for F12. Extract **shared pretty render** helper used by `recall::run` and `sync::run_query` (badge single-site). **Must preserve T207 empty pretty** (Scope + hint, no TTY gate). NDJSON path already direct-calls `recall`. |
| **F38 — updated_at plumbing (M5)** | Enumerate: `RetrievalMemory.updated_at`; `RecallHit.updated_at: Option<String>`; constructors fts/substring/graph/bridge; graph SQL fetches `updated_at` with content; bridge leaves None. |
| **F39 — Score-gap proof (M6)** | **AC1b / AC11:** Shipped Decision with **worse** FTS base (e.g. −0.5) outranks Plan Decision with **better** FTS base (e.g. −3.0) under F9 magnitudes. Unit property. |
| **F40 — T215 forward-compat** | Document in ranking.rs / CAPABILITIES: **`rerank_hits` is the single post-blend ranking entry point**; T215 extends it rather than adding a second final sort. |
| **F41 — AI1 affirm** | Pure ranking module, badges, ledger-first, no auto-forget, no semantic, no progressive rewrite — all above. |
| **F42 — Preflight strip parity** | F4 strip uses same `ASSISTANT: ` prefix rule as preflight (no shared crate dep required — duplicate one-liner OK). |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| deferred.md T211 placeholder / quality 5 | **Absorb** F1–F14, F21 |
| Stale plan-only DECISION outranks closed | **Absorb** F5–F6, F11, F39 |
| Prefer ledger when vault contradicts | **Absorb** F12–F13, F37 |
| F6 regex vs F18 zero crates | **Absorb** F6 manual scan |
| Composite None-score ambiguity | **Absorb** F8 single composite + None→0.0 |
| Semantic topic drift 6/5 | **Decline → T215** (F40 entry point note) |
| Progressive authority ranking | **Decline** (already shipped T152) |
| Full vault+ledger RRF blend | **Soft F25** residual → deferred.md |
| Auto-forget superseded pins | **Decline** F10 |
| T207 AC10 non-empty Scope | **Out** F36 |
| clap 5 / MSI / rusqlite 0.40 | **Out** |
| ISSUES.md missing | **Soft** — residuals in deferred.md (L7) |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Hermetic: Plan Decision (`plan-only` + `T999`) + Shipped (`shipped` + `PR #1` + `T999`) + shared keyword → **Shipped ranks above Plan** | Unit + hermetic |
| **AC1b / AC11** | **Score-gap (M6):** pure unit — Shipped with base **−0.5** outranks Plan with base **−3.0** under F9 (proves magnitudes) | Unit |
| **AC2** | CONSTRAINT outranks plain Other for same keyword | Unit / hermetic |
| **AC3** | Pretty demoted Plan Decision contains `plan/stale?` | Hermetic |
| **AC4** | Sibling demotion: Plan+Shipped same track token → Plan after Shipped | Unit |
| **AC5** | Equal composite scores → `memory_id` asc | Unit |
| **AC6** | `sync query --no-bridge` still re-ranks vault | Hermetic |
| **AC7** | Project isolation tests still pass | Regression |
| **AC8** | CAPABILITIES + CHANGELOG mention re-rank / plan demotion / ledger-first | Grep / review |
| **AC9** | Full CI gate green; no production unwrap/expect | Gate |
| **AC10** | Soft: ledger-first when `--json` probe non-empty + vault top Plan — hermetic if fixtureable; else manual | Soft / manual |
| **AC12** | Soft: `sync query --limit 1` respected (F27) | Soft hermetic |

## 7. Non-goals

- Packaging (MSI / notarization / App Store)
- clap 5 multi-heading
- Semantic / RRF / embedding threshold (**T215**)
- LLM re-ranking
- Rewriting governed progressive query
- Auto-forgetting or bulk-migrating DECISION pins
- Changing deny-by-default / policy / bootstrap
- Project list labels (**T212**), graph density (**T213**), preflight global (**T214**), forget list (**T216**)

## 8. Risk & blast radius

| Risk | Mitigation |
|------|------------|
| Re-rank changes recall order tests | Pure units for constants; isolation asserts presence not full order |
| False-positive plan demotion | Badge `plan/stale?`; F34 honesty; Decision-kind only |
| Ledger-first flaky without ledgerful | `--json` probe F12; AC10 soft |
| FTS score + offsets too weak | F9 + **AC1b** property |
| Pretty path restructure (M3) | Shared helper F37; preserve T207 empty path |
| `updated_at` plumbing misses graph | F16/F38 enumerate all constructors |
| Touching lexical SQL | Additive column only |
| Equal-score order change | F30/AC5 document memory_id sort |

## 9. Verification plan

1. Red: pure ranking units AC1/AC1b/AC2/AC4/AC5 fail.  
2. Green: ranking.rs + lexical/recall plumb + sync F37/F12/F27.  
3. Hermetic ranking + isolation AC7 + T207 empty pretty regression if touched.  
4. Manual: `sync query "path TOCTOU"` — closed / ledger preference visible.  
5. `ledgerful verify --scope fast` during; full gate before finalize.  
6. Review log + soft cross-model if heuristics grow.

## 10. Manual test script (on implement)

```powershell
# Prefer main project id if .env points at test-alias
ai-brains sync query "path TOCTOU" --format pretty
ai-brains sync query "path TOCTOU" --no-bridge --format pretty
ai-brains sync query "path TOCTOU" --limit 2 --format pretty
ai-brains recall "T190 TOCTOU" --limit 5 --format pretty
```

Expect: shipped/closed language and/or ledger section preferred over plan-only DECISION at top; badge on plan hits.

## 11. Out of band notes

- Pre-T187 “wrong key / plain vault” DECISION text may still exist as historical pins — content heuristics + recency reduce harm without wipe.  
- Governed Approved decisions remain on progressive/briefing path; this track is **pin/memory_projection** ranking only.

## 12. Suggested order note

… → T210 closed → **T211** / T215 → T212–T214/T216.

## 14. AI fold-in disposition (2026-08-05)

| ID | Source | Action |
|----|--------|--------|
| **AI1 #1** | Pure ranking.rs + updated_at + F9 boosts | **Affirm** F4–F9, F33, touch map |
| **AI1 #2** | Pretty `[plan/stale?]` badge | **Affirm** F11, AC3 |
| **AI1 #3** | Ledger section first when vault top plan | **Affirm** F12 (method clarified M4) |
| **AI1 #4** | No auto-forget / no semantic / no progressive | **Affirm** F2, F3, F10 |
| **M1** | regex vs zero-crates contradiction | **Accept** F6 **manual** track-token scan; F18 no regex in retrieval |
| **M2** | None-score + single vs bucket sort | **Accept** F8: **None→0.0**, **single composite** replaces old None-last sort |
| **M3** | run_query must not call recall::run | **Accept** F37 direct `recall_full` + shared pretty helper; preserve T207 empty |
| **M4** | ledger non-empty detection | **Accept** F12: `ledgerful ledger search --json` probe (≥1 entry) |
| **M5** | updated_at through all constructors | **Accept** F16 + F38 plumbing map |
| **M6** | F9 magnitude property | **Accept** AC1b/AC11 score-gap unit |
| **L1** | Shared pretty helper | **Accept** elevated DoD via F37 |
| **L2** | `--limit` on sync query | **Accept** elevated **DoD** F27 |
| **L3** | contracts staleness field | **Soft** F26 (pretty remains hard DoD) |
| **L4** | ASSISTANT: strip like preflight | **Accept** F4/F42 DoD |
| **L5** | Banner wording actionable | **Accept** F12 banner string |
| **L6** | memory_id tie-break | **Affirm** F8/F30/AC5 |
| **L7** | ISSUES.md missing | **Soft** residuals → deferred.md |
| **T215 note** | Single ranking entry point | **Accept** F40 |
