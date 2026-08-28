# T312 — Recall rank v3: pins and synth over review dumps

- **Track ID:** T312-RecallRankV3
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / UX / RETRIEVAL
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — finding #2 (“ranking is the product”); `recall` FTS 9/8 but dump-first; `--semantic` 9/**7**. Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T285 ✅ rank v2 (envelope + chrome prefixes + TAGS-or-authority + chrome-seed skip); T274 ✅ −16 chrome; T211 ✅ `rerank_hits` F40; T218 ✅ dual floor (do **not** retune); T260 ✅ stub skip; T217 ✅ R0-empty OR rescue (does **not** fire when dumps AND-hit)
- **Blocks / feeds:** Daily `recall` / `search` / `--semantic` lexical fallback / `sync query` vault half. Preflight Index/summary **T315** / T286 (not this DoD). Neighbors **T317**. Ledger pane **T313**.
- **Absorbs:** Audit ranking gap; opportunity (a) prefer `DECISION:` / `CONSTRAINT:` / structured synth; T285 closeout “live vault still dumps” **reopened** (PATH **is** 0.1.3 / T285 and still dump-first on `graph backend`); T285 closeout “more chrome prefixes as vault grows”
- **Not absorbed (DoD):** T315 summary 0/0/0; T313 ledger pane; T317 RECALLS spam; T218 floors / ANN; raise `candidate_depth`; clap 5; T263 H2; T240 F2; FTS title/body schema split (T285 declined); KIND_* bump; pretty score = effective (raw BM25 display stays)
- **Research date:** 2026-08-27. SQLite FTS5 still one-column `bm25` (https://www.sqlite.org/fts5.html, fetched 2026-08-27; last updated 2026-08-01). clap lock **4.6.1** / crates.io **4.6.6**. rusqlite **0.40.2**. Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `8b1b418b-acbb-4398-b867-7ea297d10e41`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement. Do **not** rewrite `.env`. Do **not** grow hotspot `project.rs` / `sync.rs` / CLI `preflight.rs`. Touch `ranking.rs` + `session_chrome.rs` (#6) + `lexical.rs` as this plan names. Do **not** print or commit `AI_BRAINS_KEY`. Frozen: `candidate_depth`, T218 floors, `KIND_*`.

---

## 1. Objective

1. **Pins and structured synth beat raw review dumps** for topical queries (`graph backend`, `GPU driver fix`) when a matching `DECISION:` / `CONSTRAINT:` / `INVARIANT:` pin or nightly synth exists in-scope — including when the pin does **not** contain every AND token.
2. **T285 is not enough on this vault.** Envelope + closed-list chrome + KIND_DECISION **+2** still lose because (a) pass-1 AND-retain is empty so the candidate set is dumps only, (b) long prose dumps are **not** in the chrome prefix list (live #1 was the audit dump, BM25 **−4.06**), (c) T217 OR rescue never fires when dumps AND-hit.
3. **Keep dumps recallable.** Do not hard-exclude transcripts. Authority-OR fill (when AND-retain is empty) + verbose-Other penalty + ATX-heading token detector are the levers.
4. **North star.** Capture independence: ranking/retrieval only. No new events. No hidden CoT. Agents can `recall "what did we decide"` / `recall "graph backend"` and get the pin when one MATCH-ORs in-scope.

---

## 2. Live baseline (re-scan 2026-08-27)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `27731be` `docs(conductor): mint T312-T324…`. Product `src/` has T285 + T311. Tree **CLEAN**. `origin/main...HEAD` **ahead 1** (mint). Branch `track/T312-T324-cli-dogfood`. |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. (Mint snapshot was 26,842,112 B / 05:52 — owner reinstalled after T311 `#229`.) Ranking hole is **source + PATH**. **Do not `cargo install`.** |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4513**. In-context hotspots/decisions/constraints **0/0/0**. Word count **587**. Grants **3 of 3**. Summary hole is **T315**. |
| `recall "graph backend" --no-bridge --limit 3` | **#1** audit dump `ASSISTANT: All non-destructive commands tested…` score **−4.060**. **#2** `ASSISTANT: ## Objective` (T309 OpenCode plan-audit) **−3.824**. **#3** `ASSISTANT: # Review of Track 253` **−1.325**. No leading `DECISION:`. |
| Same query `--format json --limit 15` | **`results.len() == 3`** — AND MATCH only produced **3** rows (not a JSON cap). Pass-1 authority retain **empty**. Candidate set **is** those three dumps. |
| JSON first lines | #1 not chrome (prose). #2 `## Objective` **is** chrome (−16 applied; still #2 because nothing better is in the set). #3 `# Review of Track` **is** chrome. |
| `recall "GPU driver fix"` | #1 same audit dump **−2.599**. #2 review-track chatter. #3 `# AI-Brains Onboarding` (already chrome). |
| `recall "DECISION: graph"` | Still dumps (`Here's where we are:`, T300 review, T285 `## Objective`). Query containing `DECISION:` does **not** force pass-1 if no pin GLOB-matches both tokens. |
| `recall "what did we decide"` | Dump-first, BM25 **−18.137** (review chatter that repeats the phrase). Strong FTS, no pin in top-5. |
| `--semantic "graph backend"` | Mixed: lexical #1 audit dump + semantic sim **0.56** / **0.61** (T262 agy-review + ````json` `"decisions":`). T218 floors stand; lexical fallback + semantic dumps are this track’s inherit. |
| Last GitHub PR | [#229](https://github.com/Ryan-AI-Studios/AI-Brains/pull/229) T311. `mergedAt` **2026-08-27T23:50:34Z**. `pulls/229/comments`, `/reviews`, `issues/229/comments` all **`[]`**. **last-PR Cursor: N/A. No T325.** Open PRs: **none**. |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX). Hotspot **#1** `project.rs` (3.749). `sync.rs` #2. `session_chrome.rs` **#6** (2.382) — **this is the detector file**; do not grow `project.rs` / `sync.rs` / CLI `preflight.rs`. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why T285 still loses (code + live)

| Layer | Truth |
|-------|--------|
| T285 two-pass is on | `lexical.rs` `prefer_authority: true` → GLOB-or-TAGS `LIMIT depth`, in-memory retain, recency retry, then pass-2 fill. Hermetic `recall_rank_v2` **green**. |
| Pass-1 AND-retain empty | `match_and(["graph","backend"])` + authority GLOB returns **no** pin. Recency retry uses the **same** AND expr → still empty. Pass-2 fills dumps. |
| T217 rescue does not fire | `lexical_search` returns early when R0 AND is **non-empty** (`:85–87`). Dumps AND-hit → no R2 OR. T217 also requires **≥3** tokens (`:90`); `"graph backend"` is **2**. |
| Detector closed list | True for `#2` `## Objective` and `#3` `# Review of Track`. **False** for #1 prose (`All non-destructive commands tested…`). Those take BM25 −4 with **no −16**. |
| Chrome in top-3 is a set problem | After −16, chrome effective ≈ −12. They still occupy rank 2–3 because the truncated window **is** 3 dumps. Rerank cannot invent a pin that never entered `candidate_depth`. |
| KIND_+2 frozen | Dump BM25 −4.06 → effective ≈ 4.06. Pin BM25 −1 + KIND 2 + recency 1 ≈ 4 **without** `LEADING_QUERY_BONUS`. Bonus +16 wins **if the pin is in the set**. Bumping `KIND_DECISION` does **not** pull a pin into an empty pass-1. **Keep frozen.** |
| `candidate_depth(3)=15` | Frozen (T261). Live AND only **had** 3 rows. Depth cannot create MATCH hits. |
| FTS one column | `bm25(fts, title_w, body_w)` **N/A**. SQLite FTS5 (current): column weights need a title/body split. k1=1.2, b=0.75 length-normalize **inside** BM25 — long dumps still win on **term frequency**, not missing `b`. **Decline** schema split. |
| `--semantic` | T218 floor → lexical honesty + RRF dumps. `prefer_authority_hits` inherits envelope classify. Dual floors **untouched**. |
| Pretty score | Displayed `score=` is **raw BM25** (`RecallHit.score`), not composite effective. Chrome dumps still *look* strong. **Not this DoD** (display). |

Placeholder “outrank” is resolved at this plan: **authority-OR fill when AND-retain is empty (T217-class, but pass-1-only, ≥2 contentful tokens) + verbose-Other dump-length penalty (same −16 scale, no stack with chrome) + ATX-heading **token** detector (not substring `contains("review")`) + verbose-Other must not seed graph**. KIND bump, depth raise, and FTS schema split are proven the wrong lever.

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Rank SOOT | `retrieval/src/ranking.rs` | `classify_pin_kind` `:122`; `first_contentful_line` `:102`; `rerank_hits_with_query` `:293` F40. KIND_* / `SESSION_CHROME_PENALTY` 16 / `LEADING_QUERY_BONUS` 16 **frozen**. Verbose-Other penalty **here** (same sort). |
| Detector / GLOB | `session_chrome.rs` **hotspot #6** | Closed list `:14–44`; `authority_glob_sql` `:58`; `tags_envelope_sql` `:81`; `prefer_authority_hits` `:185`; `parent_seeds_graph_neighbors` `:180` = `!is_session_chrome`. **Extend this file** (ATX token set, `is_verbose_other_dump`, seed skip). |
| Lexical two-pass | `lexical.rs` `:159–221` | Pass-1 Prefer → retain → recency retry → pass-2 `ExcludeIds`. **Add** authority-OR MATCH after recency-empty, before pass-2. Thread raw query into `match_query`. |
| T217 ladder | `lexical.rs` `:74–140` | R0 AND → return if non-empty; R1/R2 only if empty **and** tokens ≥ 3. **Do not change** this gate. T312 OR is inside `match_query` Prefer. |
| OR helpers | `ai-brains-core/src/fts.rs` `match_or` `:71`; `select_or_tokens` `:80` cap 8 | Reuse. No new crate. |
| Pipeline | `recall.rs` `:291–357` / rerank `:570` | `candidate_depth` then `prefer_authority: true` then graph then `rerank_hits_with_query(&mut blended, Some(query))`. Pass **query** already. Seed skip reads `blended` content (T285 F10). |
| Depth | `hybrid.rs` `candidate_depth` `:20` | `limit*3` clamp **15..50**. **Do not raise.** |
| Semantic floors | `hybrid.rs` `:13–17` | 0.55 hybrid / 0.60 semantic-only. **Do not retune.** |
| CLI pin | `pin.rs` TAGS envelope | **Do not change pin write.** |
| CLI recall flags | `main.rs` `:1404–1434` | `limit` default **5**; `graph_hop_depth` default **1**. **No new flags.** |
| `search` | T243 alias of `recall` | Follows automatically. |
| `sync query` vault | `sync.rs` `recall_full` | Follows automatically. **Do not grow `sync.rs` (hotspot #2).** Ledger pane **T313**. |
| `forget --match` | unfiltered MATCH | **Stay unfiltered.** |
| Preflight Index | `retrieval/src/preflight.rs` | **T315** / T286. **Do not edit as DoD.** |
| Contracts | `RecallResult` | Additive optional only. **No** `is_session` / `pin_kind` / `verbose_dump` wire key. |
| Hermetic T285 | `tests/recall_rank_v2.rs` + `recall_rank_v2_graph.rs`; retrieval `recall_pin_rank.rs` | Chrome-first-line dumps **stay green**. New tests for **non-chrome long dumps** + **AND-miss OR-hit pin**. |
| `project.rs` | hotspot **#1** | **Do not touch.** |

### 2.4 Dependency / standards research (2026-08-27) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (`cargo search`). **No clap 5** in top results. | **No bump.** No new flags. |
| `rusqlite` | exact **0.40.2** / lock **0.40.2** | — | **No bump.** MATCH + GLOB only. |
| `serde_json` | lock present | — | **No bump.** JSON keys frozen. |
| `uuid` | workspace **1.13** | — | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| workspace | **0.1.3** | — | **No bump** |
| New crates | — | — | **Zero.** No `regex` in retrieval (T211 F18). |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| FTS5 BM25 more-negative-better; `bm25(table, w…)` needs **per-column** weights; k1=1.2, b=0.75 hardcoded | [SQLite FTS5](https://www.sqlite.org/fts5.html) §5.1.1 (page last updated 2026-08-01; fetched 2026-08-27) | One `content` column → **no** title weight. Length norm **already** in `b`. Long dumps win on TF, not missing `b`. Decline schema split. |
| Implicit AND of whitespace tokens | [SQLite FTS5](https://www.sqlite.org/fts5.html) grammar (fetched 2026-08-27): `MATCH 'one two'` = AND | `"graph backend"` requires **both** tokens. Short pins miss AND → empty pass-1. |
| Title-weighted BM25 is the Engram pattern | [Engram #241](https://github.com/Gentleman-Programming/engram/issues/241) (2026-04, approved) | 6-column FTS. We cannot copy without a migration. First-line overlap bonus already shipped (T285). |
| Query-independent authority / quality prior | ACM 2026 “Towards a Relevance Posterior” (neural priors); CAR 2026 two-stage authority-then-relevance | SQLite edition: pass-1 authority OR is the cheap prior. Do not add an LLM reranker. |
| Authority ≠ textual richness | AuthorityBench 2026: adding webpage text **degrades** authority judgment | Length penalty on **Other**, not on `DECISION:`/`CONSTRAINT:`. Do not treat fluency as authority. |
| Filter then score | ES `function_score` filter+weight (T285 research; still current class) | Pass-1 GLOB-or-TAGS + OR-fill + in-memory retain is the SQLite edition. |
| Additive KIND is brittle vs BM25 scale | Elasticsearch Labs multiplicative BM25 (T285; still stands) | Do **not** bump KIND_DECISION. Dump BM25 −18 beats +2. Penalty −16 on dumps + leading-query +16 on pins is the matching scale. |
| T217 OR rescue | Live `lexical.rs` + CHANGELOG T217 | R0-empty **and** ≥3 tokens. Live hole is R0-**non-empty** dumps + **2** tokens. New Prefer-OR, not a T217 gate change. |
| clap 4 current | docs.rs/clap/4.6.6; `cargo search` 4.6.6 | No new args. clap 5 not this track. |

**N/A:** SQLCipher page encrypt, schtasks, T180 new required keys, Windows service, llama.cpp `/health`, Safety GLOB (T279 Completed), T311 in-force (governed, not FTS).

**Could not verify:** COUNT of in-scope `DECISION:` rows MATCH-ORing `graph` without vault SQL (do not print `AI_BRAINS_KEY`). Hermetic unique needle + optional live Manual canary are the proof, not live archaeology.

**ledgerful / ai-brains:** `preflight --summary` 0/0/0 vs **4513** pins; live recall still audit/Objective/Review-of-Track; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "rerank_hits_with_query"` → `ranking.rs:293` + `recall.rs:570`; `scan --impact` CLEAN at `27731be`; hotspots `session_chrome.rs` #6. Semantic/`sync query` still dump-first — evidence of the hole, not SoT for decisions.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `8b1b418b`. Implement starts a **FEATURE** TX. |
| **F1 — Rank, do not delete** | Session ingest stays recallable. **Do not** hard-exclude transcripts. **Do not** forget/migrate them. |
| **F2 — Envelope stands** | T285 F2 `first_contentful_line` / role+TAGS strip. **Do not** rewrite. |
| **F3 — Leading-line stands** | Marker must **start** the first contentful line. Buried JSON `"decisions":` stays Other (and stays chrome via the `{`+`"decisions":` detector). |
| **F4 — KIND_* / chrome −16 / depth / floors frozen** | Do **not** change `KIND_*`, `PLAN_PENALTY`, `RECENCY_SCALE`, `SESSION_CHROME_PENALTY` (16), `SYMBOL_PENALTY`, `LEADING_QUERY_BONUS` (16), `candidate_depth` 15..50, T218 0.55/0.60. |
| **F5 — ATX heading tokens (additive)** | `is_session_chrome` is true when the first contentful line is an ATX heading (`#` after envelope) **and** `extract_fts_tokens(line)` contains one of **`review` / `objective` / `onboarding` / `audit` / `ratings`** (ASCII case-insensitive token set — **not** substring `contains("review")`, which would false-hit `# Preview of graph`). Existing exact prefixes (`## objective`, `# review of track`, `` ```json ``, `{`+`"decisions":`, …) stay as regression. **False** for `# Heading without chrome prefixes` (T285 AC2). **Do not** add prose-sentence prefixes (`Here's where we are`) — that is F6. |
| **F6 — Verbose-Other penalty** | Inside **the same** `rerank_hits_with_query` sort (F40 — **no** second final sort): when `classify_pin_kind` is **Other**, `is_session_chrome` is **false**, and `content.chars().count() >= DUMP_OTHER_CHAR_FLOOR` (**800**), subtract `DUMP_OTHER_PENALTY` = `SESSION_CHROME_PENALTY` (16.0). Decision / Constraint / Hotspot never take this penalty (authority richness is not a dump). Short Other chat crumbs (**&lt; 800** chars) stay unpenalized. |
| **F7 — No double dump penalty** | Chrome already −16. Do **not** also apply F6 on chrome rows. A long `## Objective` dump is −16 once. |
| **F8 — Authority-OR fill** | Inside `match_query` when `prefer_authority` and post-AND + post-recency **retain is empty**: if `contentful_tokens(extract_fts_tokens(raw_query)).len() >= 2`, one more Prefer MATCH using `match_or(select_or_tokens(contentful))` (T217 helpers), then in-memory retain. Then pass-2 AND as today (`ExcludeIds`). This is a **retry of pass-1** with OR — **not** a T217 R2 change, **not** substring_fallback, **not** a third MATCH family on unfiltered dumps. |
| **F9 — T217 ladder unchanged** | R0 non-empty still returns. Rescue still requires tokens ≥ 3. `forget --match` stays `rescue: false` + `prefer_authority: false`. |
| **F10 — Verbose-Other must not seed graph** | `parent_seeds_graph_neighbors` is false when `is_session_chrome` **or** `is_verbose_other_dump`. T260 stub-seed skip stays. Do **not** default `graph_hop_depth` to 0. Read **`hit.content` from `blended`**. |
| **F11 — Semantic arm** | No second embedding SQL. `prefer_authority_hits` + F6/F5 inherit. Dual floors **untouched**. Hermetic AC **no** HTTP. |
| **F12 — Near-dup chrome** | `dedupe_session_chrome` after rerank stands. Distinct `DECISION:` pins never collapse. |
| **F13 — Decline Index / summary** | Preflight Index / `--summary` 0/0/0 are **T315** (T286 shipped renderer). **Do not** edit CLI `preflight.rs` (hotspot #9). |
| **F14 — Decline memory-list preview** | **T316**. T287 ORDER frozen. |
| **F15 — sync query vault** | Follows `recall_full`. **Do not** edit `sync.rs`. Ledger pane **T313**. |
| **F16 — No new CLI flag** | No `--transcripts` / `--pins-only` / `--no-graph`. |
| **F17 — No DTO keys** | No `is_session` / `pin_kind` / `verbose_dump` on `RecallResult`. JSON `content` raw. PROTOCOL-COMPAT: N−1 ignore. |
| **F18 — forget unfiltered** | `forget --match` does **not** take envelope GLOB / two-pass / chrome skip / verbose skip / OR fill. |
| **F19 — Capture independence** | Ranking only. No models on default FTS. No new events. **Do not rewrite** `pin.rs` stored shape. |
| **F20 — Pins / crates** | No clap 5, no rusqlite bump, no new crates, workspace **0.1.3**. |
| **F21 — PATH** | Do not `cargo install` unless the user asks. Tests/manual AC use `cargo run` / hermetic bin. |
| **F22 — Live vault pin** | Do **not** pin production DECISIONs as implement. Hermetic unique needle is SoT. Manual DoD **unique canary** (uuid in the string) is allowed on go. |
| **F23 — Decline leftover F39** | T276 preferred-fill skip when cwd fills depth stays. |
| **F24 — Decline T279 / T263 H2 / T240 F2 / T211 F25 / ANN / floor retune** | Standing. |
| **F25 — last-PR Cursor** | #229 empty → **N/A**. **No T325.** |
| **F26 — Tests** | Naming `function_or_feature__condition__expected_result`. rstest `#[case]` for new heading tokens. No `unwrap`/`expect`/`panic` in production. |
| **F27 — Cross-model** | Retrieval ranking is FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F28 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F29 — File growth** | F6 penalty apply in `ranking.rs`. F5 token heading + `is_verbose_other_dump` + F10 seed helper in `session_chrome.rs`. F8 OR fill in `lexical.rs` (`match_query` threads `raw_query`). New CLI hermetic `tests/recall_rank_v3.rs`. **Do not** grow `project.rs`, CLI `preflight.rs`, `sync.rs`, `pin.rs` write path, `.github/workflows/ci.yml`. **Do not** require a production `recall.rs` edit if seed helper lives in `session_chrome.rs` (call site already uses `parent_seeds_graph_neighbors`). |
| **F30 — Existing tests stay green** | T285 AC1–AC17; T274 pin-rank; T211 leading DECISION; T260 exclude; T217 rescue units; T216 list recency; T207/T261 empty; T218 floors; T276 prefer-fill. |
| **F31 — Docs** | CAPABILITIES pin-type row: authority-OR fill + verbose-Other −16 + ATX token headings. CHANGELOG T312. PROTOCOL-COMPAT: no new required keys. WORKFLOWS “what did we decide” still `recall`. |
| **F32 — PowerShell** | `;` not `&&`. |
| **F33 — substring fallback** | T105/T261 stands. Envelope prefer-fill after substring stands. |
| **F34 — Pass-2 / retry ids bound** | T274 F35 / T285 F34 stand. OR-fill Prefer has **no** `NOT IN`. Pass-2 still `bound_not_in_sql` + `?` only. **Forbidden** to `format!` UUID strings into SQL. |
| **F35 — search alias** | T243: `search` is `recall`. One hermetic covers both **or** document argv0. Manual DoD still runs **both**. |
| **F36 — Seed helper** | `parent_seeds_graph_neighbors` unit-tested **without** `--features graph`. |
| **F37 — Graph-on CLI** | T285 AC17 stays green. New verbose-Other seed is a **graph-off unit** (F36). Do **not** add a retrieval graph CI line. |
| **F38 — Pretty score display** | Raw BM25 on pretty `score=` stays. Do **not** replace it with composite effective this track. |
| **F39 — DUMP_OTHER_CHAR_FLOOR** | Const **800** next to the penalty. Count `content.chars()` (Unicode scalar values), not bytes, not tokens. |
| **F40 — OR only when retain empty** | Do **not** OR-fill when AND-retain already has ≥1 authority pin (T285 pass-1 stands). |
| **F41 — Two-token OR** | Fire at **≥2** contentful tokens (T217 rescue stays ≥3). `"graph backend"` is the live 2-token hole. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | rstest `#[case]`: detector **true** for `# Review of Track 253`, `## Objective`, `## Ratings\n…`, `# AI-Brains Onboarding`, `# Track plan audit`. **False** for `DECISION: …`, `# Heading without chrome prefixes`, `# Preview of graph backend` (must not substring-hit `review`). Existing T285 prefixes stay true. **Required red.** |
| **AC2** | `rerank_hits_with_query`: Other dump 2000 chars, first line `All non-destructive commands tested against the live vault.`, BM25 **−4.06**, query `"graph backend"` vs leading `DECISION: graph backend stays sqlite` BM25 **−1.0** → pin **first** (F6 −16). **Required red.** |
| **AC3** | Same sort: Other dump **200** chars (short crumb) BM25 −4 vs pin BM25 −1 **without** query bonus → dump may still lead (F6 does **not** fire). Guard that short Other is not penalized (`effective` of the crumb is **not** reduced by 16). |
| **AC4** | Retrieval hermetic: 15 dumps whose **first line is non-chrome prose** (`Here's the assessment.`) and whose **body repeats the unique needle 12×** + 1 `ASSISTANT: TAGS: t312\nDECISION: {needle}` → `recall_full` `--limit 5` hit **#1** is the pin. Graph may be `None`. **Required red** (T285 chrome-first-line tests would **not** catch this). |
| **AC5** | Retrieval hermetic **AND-miss / OR-hit**: query `"t312or {uuid} backend"` (two contentful tokens); pin `DECISION: t312or {uuid} sqlite graph` (**no** `backend`); 15 dumps whose body contains **both** tokens + chrome or prose first line → pin **#1**. Proves F8. **Required red.** |
| **AC6** | Unit (graph-off): `parent_seeds_graph_neighbors` **false** for verbose-Other (800+ chars, non-chrome) and for `## Objective`; **true** for `DECISION: …` and for short Other &lt; 800. T260 stub-seed skip stays. **Required red.** |
| **AC7** | T285 `recall_rank_v2` / `classify_pin_kind__tags_envelope` / `rerank_hits_with_query__onboarding_chrome_loses_to_pin` / CLI graph AC17 **stay green**. |
| **AC8** | T260 default exclude still drops `Function foo (src/a.rs:1)` without `--symbols`. |
| **AC9** | Empty / contentless still T207 / T261. |
| **AC10** | `forget --match` still finds a verbose-Other dump row (unfiltered). |
| **AC11** | Compact recall JSON: no new required keys; `content` still includes `ASSISTANT:` / `TAGS:` raw. |
| **AC12** | Hermetic CLI `tests/recall_rank_v3.rs`: `ai-brains recall "{needle}" --limit 5 --format pretty --no-bridge` **and** `search` hit #1 is the tagged pin, **not** the prose dump / `## Objective`. EXIT **0**. **Required red** (CLI). |
| **AC13** | Hermetic `sync query "{needle}" --no-bridge --limit 5`: vault section top is the pin. Ledger pane may be empty — **do not** assert ledger. |
| **AC14** | `--semantic` hermetic: if embed skipped/floor-empty, lexical fallback list still has the pin in **top-3**. No live HTTP required. |
| **AC15** | Unit: authority-OR SQL (when retain-empty) contains ` OR ` + `GLOB` + `TAGS:` + `LIMIT`; **only** `?` placeholders for ids/limit — no UUID literals (F34). Guard, not Phase-1 red. |
| **AC16** | Unit: `KIND_DECISION == 2.0`, `KIND_CONSTRAINT == 4.0`, `SESSION_CHROME_PENALTY == 16.0`, `candidate_depth(5) == 15`, `SEMANTIC_MIN_COSINE == 0.55`, `SEMANTIC_ONLY_MIN_COSINE == 0.60`. Stay-green freeze. |
| **AC17** | Chrome long dump: applying F6 must **not** change effective vs chrome-only (−16 once). Unit: `## Objective` + 2000 chars, BM25 −3.8, query `None` → same effective as the chrome penalty path today. |
| **AC18** | Store `list_memories` recency order unit still `updated_at DESC` (T216 / T287 freeze). |

---

## 5. Design notes

### 5.1 Why AND-retain is empty (live `graph backend`)

FTS5 implicit AND (`MATCH 'graph backend'`) requires both tokens. T285 pass-1 adds GLOB-or-TAGS. Short pins often contain `graph` **or** `backend`, not both. Recency retry uses the same AND. T217 R2 OR never runs because dumps already AND-hit (and the query has only two tokens).

**Fill:** Prefer MATCH with `match_or` of up to 8 contentful tokens, retain authority, then pass-2 AND dumps as today. Pins that match **any** query token enter `candidate_depth`. `LEADING_QUERY_BONUS` then prefers pins whose first line overlaps the query.

### 5.2 Verbose-Other vs chrome

```
if is_session_chrome { effective -= 16 }           // F5 / T274 / T285
else if is_verbose_other_dump { effective -= 16 }  // F6 / F7
```

`is_verbose_other_dump` = `PinKind::Other && !is_session_chrome && chars >= 800`.

Live #1 (`All non-destructive commands tested…`) is this class. Live #2/#3 are chrome. Both sink relative to a pin in the set.

### 5.3 ATX token set (F5)

Split the first contentful line with `extract_fts_tokens` (alphanumeric runs). Do **not** use `str::contains("review")` (`Preview` would match). Token set is closed and tiny. Exact-prefix detectors remain so `` ```json `` and `{`+`"decisions":` still fire without being headings.

### 5.4 What we will not do

- Bump `KIND_DECISION` (dump BM25 −18 still wins; pin-not-in-set still wins).
- Raise `candidate_depth` (live AND had 3 rows).
- Split FTS into title/body (migration; T285 declined).
- LLM / neural reranker (capture independence; AuthorityBench says text richness ≠ authority).
- Pretty composite score (display; F38).

---

## 6. Non-goals

Hard-delete dumps from FTS. Pin→Approved (H2). New FTS columns. `--semantic` HTTP in hermetic tests. clap 5. T218 floor retune. ANN. T315 / T313 / T317 / T316 steal. `KIND_*` bump. Pretty `score=` rewrite. Silent `.env` rewrite (T240 F2). Graph Cargo default-on.

---

## 7. Verification plan (TDD)

**Red first (on go):**

1. `is_session_chrome__atx_tokens__ac1` (rstest) — `# Preview of graph` false.
2. `rerank_hits_with_query__verbose_other_dump_loses_to_pin__ac2`
3. `recall_full__prose_dump_body_match__pin_first__ac4` (retrieval hermetic)
4. `match_query__and_retain_empty__authority_or_fills_pin__ac5`
5. `parent_seeds_graph_neighbors__verbose_other__false__ac6`
6. CLI `recall_rank_v3.rs` AC12

Then green the production paths. Stay-green AC7/AC16/T285. Full gate before Completed.

Manual (optional canary, not architecture SoT): unique `DECISION: t312-canary-{uuid} sqlite graph` then `recall "t312-canary-{uuid} backend" --format pretty --no-bridge --limit 5` → pin #1. Do **not** require live `graph backend` to become a pin (corpus volatile).

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Authority-OR too broad (`graph` matches many pins) | Retain is authority-only; `LEADING_QUERY_BONUS` prefers first-line overlap; depth still 15; dumps still fill remainder. |
| Long nightly synths (Other) sink | They are not leading-marker pins. Pins still win. Synth still recallable via unique tokens. Soft residual if a synth **should** lead a topical query — do not invent `KIND_SYNTH` this track. |
| `# Preview` false-hit | Token set, not substring (AC1). |
| Double −16 on chrome | F7 / AC17. |
| T217 rescue regression | F9; existing `lexical_rescue.rs` stay-green. |
| forget starts OR-filling | `prefer_authority: false` default; AC10. |
| Hotspot growth | Edits stay in `session_chrome.rs` / `ranking.rs` / `lexical.rs`. No `project.rs`. |

---

## 9. Deferred absorb/decline

| Item | Disposition |
|------|-------------|
| T312 mint / audit recall rank dump-first | **Absorb** this plan |
| T285 closeout live dumps / more chrome prefixes | **Absorb** F5/F6/F8 |
| T217 R0-empty OR | **Partial** — reuse helpers; **do not** change T217 gate (F9) |
| T286 / T315 Index/summary 0/0/0 | **Decline steal → T315** |
| T313 `sync query` ledger pane | **Decline steal → T313** |
| T317 neighbors RECALLS | **Decline steal → T317** |
| T316 memory list preview | **Decline steal → T316** |
| T218 floors / ANN / `candidate_depth` | **Decline** F4 |
| T263 H2 / T240 F2 / clap 5 | **Decline** F24 / F20 |
| T307 Blocked dual tower-http | **Not stolen** |
| T308 density floors | **Decline** standing |
| T311 R1–R7 | **Not this DoD** (T322–T324 / declined) |
| last-PR Cursor `#229` | **N/A empty** — no T325 |
| conductor/archive specs / cargo-audit allowlist / T147 TempEnv | **Not related** — not retrieval rank |
| Pretty composite score | **Decline** F38 |

---

## 10. Implement order (on go)

1. Phase 0: re-read `match_query` retain gate, `is_session_chrome`, `rerank_hits_with_query`; rescan deferred; FEATURE TX.
2. Red: AC1/AC2/AC4/AC5/AC6/AC12 tests.
3. Green: F5 detector + F6 penalty + F8 OR fill + F10 seed.
4. Stay-green: T285 / T217 / T218 / T260 / AC16/AC17.
5. Docs: CAPABILITIES + CHANGELOG.
6. Full gate + codex-review (FEATURE).
7. Conductor Completed + deferred. Phase 6: `track/T312-*` → PR → watch `CI` → squash-merge. Never `git push origin main`.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| PATH until `cargo install` | F21 |
| Live `graph backend` may still dump-first if **no** in-scope pin MATCH-ORs either token | Honest; hermetic AC5 is SoT |
| Long Other synths demoted | F6 by design |
| Pretty `score=` still raw BM25 | F38 |
| More heading tokens as vault grows | Closed token set; extend only with evidence |
| Semantic dumps above floor | T218 freeze; inherit F5/F6 only |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/session_chrome.rs` | F5 ATX tokens; `is_verbose_other_dump`; F10 seed |
| `crates/ai-brains-retrieval/src/ranking.rs` | Apply F6 in `rerank_hits_with_query`; freeze tests AC16 |
| `crates/ai-brains-retrieval/src/lexical.rs` | F8 OR fill inside `match_query`; thread `raw_query` |
| `crates/ai-brains-retrieval/src/recall.rs` | **Avoid** unless compile-forced (seed helper already called) |
| `crates/ai-brains-retrieval/tests/*` | AC4/AC5 hermetics |
| `crates/ai-brains-cli/tests/recall_rank_v3.rs` | AC12/AC13 |
| `Docs/CAPABILITIES.md` | Pin-type row additive |
| `CHANGELOG.md` | T312 Unreleased |
| `conductor/conductor.md` / `deferred.md` | Registry + absorb table |

**Do not touch:** `project.rs`, `sync.rs`, CLI `preflight.rs`, `pin.rs` write, `hybrid.rs` floors, `ci.yml`, PROTOCOL-COMPAT required keys.

---

## 13. Fold-in

*(empty until `/fold-in 312`)*
