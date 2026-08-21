# T274 — Pins and DECISION rows must beat harness session dumps

- **Track ID:** T274-PinRankVsIngest
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / UX / RETRIEVAL
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `recall`/`search` **10/4**, `--semantic` **8/4**, `preflight --pretty` Index **9/4**, `--summary` **9/5**, `memory list` **8/7**, `sync query` vault half **9/7**. Placeholder minted with T274–T284 (`deabae7`).
- **Depends on:** T260 ✅ symbol stubs; T211 ✅ `rerank_hits` F40; T218 ✅ dual floor; T216 ✅ memory-list recency; T264/T272 ✅ preflight caps/skip
- **Blocks / feeds:** Daily vault-first `recall` / `search` answers pins. Preflight Index/summary marker counts follow rank. `sync query` vault arm follows `recall_full`. Safety **section identity** stays **T279**. Grants **T275**. Leftover rebind **T276**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “recall/search/semantic/preflight/memory-list session dumps over pins”; T260 lesson that demote-only fails when `candidate_depth` is all noise; T211 F4 anywhere-in-body marker scan (lift to leading-line)
- **Not absorbed (DoD):** T279 Safety vs `safety sync` hotspots; T275 bootstrap; T276 leftover `7d97a456`; T284 #188 Work/samples; T263 H2 pin→Approved; T240 F2 silent Scope; T216 list ORDER flip; T211 F25 vault↔ledger RRF; T218 floors / ANN; T260 `--symbols`; clap 5 / rusqlite 0.40 / new crates / DTO keys
- **Research date:** 2026-08-21 (plan dogfood HEAD `deabae7`; product `src/` = T270 `#188` `14d42af`)
- **AI fold-in:** 2026-08-21 `agy-review.md` + `opencode-review.md`. **B 0 / M 0.** **Agree:** Agy m1 parameterized pass-2 `NOT IN` (F35 / AC17); Agy O1 authority GLOB helper (F36). **Already:** Agy m2 chrome-only dedupe (F10 / AC5); Agy O2 pass-1 full → skip pass 2 (F7 / §5.2). **Fold snapshot:** OpenCode m1 HEAD `9a99117`; o1 summary counts volatile; o2 hotspot score; o3 `classify_pin_kind` grep on go. Disposition **§13**.
- **Ledger:** planning DOCS TX `9c1049c0-5520-430d-a1f0-01aba355082e`. Fold-in DOCS TX `c483e45a-cf54-4d50-b15b-3d7128f9b5d0`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`, rewrite `.env`, pin-as-implement to the live vault, bootstrap live grants, rebind leftover paths, or live `retention apply --confirm`. Do **not** grow hotspot `project.rs` / CLI `preflight.rs` / `sync.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** raise `candidate_depth` (T261). Do **not** retune T218 cosine floors.

---

## 1. Objective

1. **Pins win daily recall.** `recall` / `search` (and `--semantic` lexical honesty) must return a leading `DECISION:` / `CONSTRAINT:` / `INVARIANT:` pin as hit **#1** when the query matches that pin — not a review-track `## Objective`, `# Track Plan Review`, or JSON session summary.
2. **Ingest recency must not bury a pin.** Harness capture is still `status='pinned'`. Session dumps stay recallable, but they must not occupy the whole `candidate_depth` (15 at `--limit 5`) or the preflight Index window.
3. **Near-identical reviews collapse.** Duplicate first-line session chrome (two `## Objective` copies) is one row after rank.
4. **Preflight Index/summary follow rank.** `--summary` `in_context_decisions` ≥ 1 when a leading `DECISION:` pin is in the budget window. Index lists that pin. Safety body vs Ledgerful hotspots is **T279**.
5. **North star.** Capture independence: ranking/retrieval only. No new events. No hidden CoT. Agents can `recall "what did we decide"` and get the pin, not the review-track prompt that mentioned the pin.

This unblocks the daily product: T260 stopped T70 stubs; T211’s +2 DECISION boost loses to BM25 of long dumps; T218 floors do not apply on lexical. The audit scored 10/4 because the vault’s 3k pins are invisible behind today’s ingest.

---

## 2. Live baseline (re-scan 2026-08-21)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `deabae7` mint T274–T284. **This fold-in:** `9a99117` (`docs(conductor): plan T274 pins vs harness ingest ranking`; parent `deabae7`). Product `src/` identical to T270 squash `14d42af` (`#188`) — `git diff 14d42af HEAD` on ranking/lexical/recall/preflight is empty. Tree **CLEAN** at fold-in. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-21 05:55**, 25 368 576 bytes, **0.1.1**. T270 is on PATH. **Do not `cargo install`.** |
| Source debug | `target\debug\ai-brains.exe` mtime **2026-08-20 22:16** — older than PATH. Plan dogfood used **PATH**. Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). **Plan:** Pinned **3297**, in-context **0/0/0**. **OpenCode review:** **1/1/1** (same 3297). **This fold-in:** Pinned **3324**, in-context **0/0/0**, word_count **805**. Counts are **volatile** (window of dumps). Hole stands: 3k pins vs ~0–1 in-context. Grants 0 of 3 (T275). |
| `recall "what did we decide about retention" --no-bridge --limit 5` | T248 reviews (BM25 **−8.67 / −8.65**), JSON `"decisions": [...]` summary, T270 `## Objective`, chat crumb. **No pin in top-5.** |
| `recall "T270" --no-bridge --limit 5` | Chat crumbs + JSON init summary + T270/T272 plan-review dumps. **No DECISION pin.** |
| `recall "DECISION: T270" --no-bridge --limit 5` | Five `## Objective` / T264-review dumps. Scores **−1.47 … −0.92** (weak BM25). Unique pin **not in candidate set**. |
| `recall "…retention" --semantic --no-bridge --limit 3` | JSON init / T270 agy-review / T251 JSON. Same hole on hybrid. |
| `memory list --limit 5` | All **18m**, previews `## Objective`, `## Objective`, `# Track Plan Review: T270`, two `### Track 248 Review`. `status=pinned`. Recency inventory is **honest** (T216) and **useless** as a decision index. |
| `preflight --pretty -m 400` | Safety = T272 **`## Objective`**. Sessions = T270/T271 Objective dumps. Index cut by budget. |
| Last GitHub PR | [#188](https://github.com/Ryan-AI-Studios/AI-Brains/pull/188) T270 (2026-08-21). Cursor Bugbot **2 Mediums** (Work table hides dispose; apply samples prefer inventory) — already **T284**. Open PRs: Dependabot remotes only. **No leftover to mint.** |
| Identity / doctor | Summary Scope `3581317d`. ledgerful doctor 5 warn (legacy `.changeguard` / sig-pin / timings / :8081). 0 pending / 0 drift. Hotspot **#1** `project.rs` (plan **3.990**; T270 review **3.999** — still #1). CLI `preflight.rs` **#7**. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why pins lose (code + BM25)

| Layer | Truth |
|-------|--------|
| Capture and `pin` share `status='pinned'` | Harness ingest is not `active`. T211 “pin-type” is a **content prefix**, not vault status. |
| T211 F4 is anywhere-in-body | `classify_pin_kind` takes **leftmost** `constraint:` / `decision:` / `hotspot:` after one `ASSISTANT: ` strip. Review-track bodies and JSON `"decisions": [` get **Decision +2**. |
| KIND_DECISION **+2** vs BM25 | Live dump `rank=-8.67` → composite base **8.67** + recency ~**1** ≈ **9.7**. A unique pin that barely MATCHes (`-1.47` → base **1.47** +2 +1 ≈ **4.5**) **loses**. Elastic Labs (2025-12): additive “+2” is brittle across BM25 scales. |
| `candidate_depth(5)=15` | T260 lesson: if MATCH LIMIT 15 is all chrome, `rerank_hits` cannot surface a pin that never entered. **`recall "DECISION: T270"` live: pin absent.** |
| T211 F9 recency | `RECENCY_SCALE=1.0` over 365 days. Same-day dump and same-day pin tie on recency; long dumps win on BM25. |
| PLAN_PENALTY **−3** | Reviews containing “expanded” / “planning” / “until go” get **demoted if classified Decision** — still not enough vs BM25 8–11, and leading pins that say “plan-only” are also hit. |
| Preflight Index | `retrieval/preflight.rs` `:437–507`: `status='pinned' ORDER BY updated_at DESC` until word budget. **No** `rerank_hits`. Newest ingest wins. |
| Preflight Safety | `LIKE '%CONSTRAINT:%' OR '%INVARIANT:%' OR '%HOTSPOT:%'` **anywhere** LIMIT 10/40. Skill text in `## Objective` matches. **T279** owns sourcing from `safety sync`; T274 does **not** retune this SQL. |
| Summary counts | CLI `preflight.rs` `:886–888` `text.matches("DECISION:")` on the **budget window**. Window of dumps → **0**. |
| `is_low_signal` | Word-count &lt;6 or short build-monitor phrases. Long `## Objective` is **high** signal. |
| T216 `memory list` | `ORDER BY updated_at DESC, memory_id ASC`. Correct inventory. **Not** a decision index. |
| FTS is one column | `memory_fts` has no title/body split. SQLite `bm25(fts, title_w, body_w)` **N/A**. Post-filter + candidate prefer-fill are the levers. |
| T260 analog | Default **exclude** stubs via GLOB. Session dumps **are** session memory — **do not** hard-exclude. Prefer-fill + penalty. |

Placeholder F “outrank” is resolved at this plan: **leading-marker classification + session-chrome penalty + lexical/Index two-pass** (pin enters the set) **+ first-line chrome collapse**. Boost-only is proven insufficient on this vault.

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Rank SOOT | `retrieval/src/ranking.rs` **858** lines | `classify_pin_kind` `:84`; `rerank_hits` `:248` F40. KIND_* `:15–32`. `INVARIANT:` **absent**. New chrome detector → **`session_chrome.rs`** (T260 pattern: do not grow ranking.rs). |
| Pipeline | `recall.rs` `:510–514` | `rerank_hits` then `dedupe_symbol_stubs` then truncate. Chrome dedupe **after** rerank, **with** stub dedupe (chrome then stubs, or one retain pass). |
| Lexical | `lexical.rs` `match_query` `:139–181` | `MATCH ? AND status='pinned'` + optional T260 `symbol_stub_sql_exclusion`. `ORDER BY rank LIMIT ?`. Two-pass **here** (same MATCH expr). |
| Depth | `hybrid.rs` `candidate_depth` | `limit*3` clamp **15..50**. **Do not raise.** |
| Semantic | `semantic.rs` | Same projection; T260 GLOB when exclude. In-memory prefer-fill after fetch (no second BLOB SQL). |
| Preflight Index | `retrieval/src/preflight.rs` `:437–507` **1003** lines | Recency scan. Two-pass **here**. CLI `preflight.rs` **2027**, hotspot **#7** — **do not grow** (summary scan stays; counts follow Index). |
| Safety SQL | `preflight.rs` `:288–303` | LIKE anywhere. **T279.** |
| Memory list | `store/projections/memory.rs` + CLI `memory.rs` | Recency. **T216 freeze.** |
| `search` | T243 alias of `recall` | Follows automatically. |
| `sync query` vault | `sync.rs` calls `recall_full` | Follows automatically. **Do not grow `sync.rs` (hotspot #2).** Ledger pane **T271**. |
| `forget --match` | uses `lexical_search` unfiltered | **Stay unfiltered** (T260 F). |
| Contracts | `RecallResult` | Additive optional fields only. **No** `is_session` / `is_authority` wire key. |
| `project.rs` | hotspot **#1** (3.990) | **Do not touch.** |
| CLI recall flags | `main.rs` Recall | `semantic`, `min_score`, `graph_boost`, `no_bridge`, `global`, `session_last`, `--symbols`. **No** `--transcripts` this track. |

### 2.4 Dependency / standards research (2026-08-21)

**Snapshot — re-verify at execute.**

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). **No clap 5.** | **No bump.** No new flags. |
| `serde_json` | lock **1.0.150** | — | **No bump.** JSON keys frozen. |
| `chrono` | workspace **0.4** / lock **0.4.44** | crates.io **0.4.45** (Dependabot #62) | **No bump.** Recency parse stays. |
| `rusqlite` | lock **0.39.0** | crates.io **0.40.2** (Dependabot #61; T213 L4) | **No bump.** MATCH + GLOB only. |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** No `regex` in retrieval (T211 F18). No simhash/minhash crate. |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| FTS5 BM25 more-negative-better; length-normalizes vs `avgdl` | [SQLite FTS5](https://www.sqlite.org/fts5.html) (bm25 section) | Long token-dense reviews beat short pins. One-column FTS → **no** `bm25(fts, title_w, body_w)`. |
| Column weights need a title/body split | Same; Engram #241 weighted BM25 (2026-04) | We have one `content` column. **Decline** FTS schema split this track. |
| Metadata/type filters belong **before** rank | Hybrid RAG + sqlite-vec (2026-05): pre-filter CTE by category | Two-pass: authority MATCH first, chrome fills remainder. Azure-style type filter analog. |
| Additive “+2” boosts are brittle vs BM25 scale | [Elasticsearch Labs 2025-12 multiplicative BM25](https://www.elastic.co/search-labs/blog/bm25-ranking-multiplicative-boosting-elasticsearch) | Do **not** bump KIND_DECISION. Add **−16** chrome penalty (T260 `SYMBOL_PENALTY` analog) **and** get the pin into the set. |
| `function_score` filter + weight | [ES function_score](https://www.elastic.co/docs/reference/query-languages/query-dsl/query-dsl-function-score-query) (current) | Filter “is authority pin” then score. Prefer-fill is the SQLite edition. |
| RRF already shipped | T215/T218; PatentLLM hybrid RAG (2026-05) | **No** second RRF. F40 stands. `--semantic` still fuses then `rerank_hits`. |
| clap 4 current | docs.rs/clap/4.6.6 | No new args. clap 5 not released. |
| Bind `NOT IN` lists; do not interpolate values | [SQLite expr parameters](https://www.sqlite.org/lang_expr.html); rusqlite `params_from_iter` (live `lexical.rs:184`) | Pass-2 ids are `?` placeholders (Agy m1 / F35). Empty list → skip the clause (SQLite allows `NOT IN ()` but we do not emit it). |

**N/A:** SQLCipher, schtasks, T180 2-key DTO, Windows service, llama.cpp `/health`.

**Could not verify:** exact live `DECISION:` pin COUNT for T270 (FTS never returned it). Hermetic unique needle is the proof, not live vault archaeology. Semantic e2e depends on :8083 (up this session); hermetic AC does **not** require HTTP.

**ledgerful / ai-brains:** fold-in `preflight --summary` 0/0/0 vs **3324** pins (plan 3297 / OpenCode 1/1/1 — volatile); `ledgerful ledger status --compact` 0 pending / 0 drift; `search "classify_pin_kind"` → `:84` + tests (all leading) + `sync.rs:523` (F2 lift inherits; **do not** edit `sync.rs`). Semantic `ask` skipped (search hit).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `9c1049c0`. Fold-in is DOCS TX `c483e45a`. Implement starts a **FEATURE** TX. |
| **F1 — Rank, do not delete** | Session ingest stays in the vault and stays recallable. **Do not** hard-exclude transcripts from default recall the way T260 excludes stubs. **Do not** forget/migrate them. |
| **F2 — Lift T211 F4 to leading-line** | `classify_pin_kind`: after `strip_assistant_prefix` + trim, inspect the **first contentful line** only. Marker must **start** that line (optional leading whitespace). Case-insensitive `constraint:` / `decision:` / `invariant:` / `hotspot:`. Buried JSON `"decisions": [` and skill-body mentions → **Other**. `ASSISTANT: DECISION:` still Decision (existing unit). |
| **F3 — INVARIANT ≡ Constraint** | Leading `INVARIANT:` → `PinKind::Constraint` (Safety SQL already treats it as a bearings marker). Kind boost stays `KIND_CONSTRAINT` (+4). |
| **F4 — KIND_* magnitudes frozen** | Do **not** change `KIND_DECISION` / `KIND_CONSTRAINT` / `KIND_HOTSPOT` / `PLAN_PENALTY` / `RECENCY_SCALE`. Hole is classification + candidate set + chrome penalty, not “+2 was too small.” |
| **F5 — Session chrome detector** | New `session_chrome.rs`. Closed list on the first contentful line after `strip_assistant_prefix`: (1) `## Objective`; (2) `# Track Plan Review`; (3) `### Track ` **and** `Review` (case-insensitive); (4) `# AI-Brains Onboarding`; (5) starts with ` ```json`. In-memory may also treat a line starting with `{` **and** containing `"decisions":` in the first 500 chars as chrome. **Not** every markdown heading. Detector is SoT. |
| **F6 — SESSION_CHROME_PENALTY** | `16.0` composite units (same scale as `SYMBOL_PENALTY`). Applied inside `rerank_hits` when detector is true. **F40 stands** — no second final sort. |
| **F7 — Lexical two-pass (required)** | Same MATCH expression. **Pass 1:** MATCH + authority GLOB (`DECISION:` / `CONSTRAINT:` / `INVARIANT:` at start, plus `ASSISTANT: ` variants) `ORDER BY rank LIMIT depth`. If `pass1.len() >= depth`, **return pass1** (Agy O2 — no second MATCH). **Pass 2:** MATCH excluding pass-1 ids, `ORDER BY rank LIMIT remainder`. Chrome may appear in pass 2. If pass 1 is empty, pass 2 is today’s behavior (chrome-only queries still return chrome — no T207 lie). T260 GLOB exclude still applies when `exclude_symbol_stubs`. |
| **F8 — GLOB is a subset** | SQL GLOB is case-sensitive prefix (`DECISION:*`, `ASSISTANT: DECISION:*`, …). Detector is case-insensitive SoT (T260 F19 analog). In-memory prefer-fill / penalty catch lowercase `decision:`. |
| **F9 — Semantic arm** | No second embedding SQL. After semantic fetch, **in-memory** prefer-fill: authority hits first, then others, cap `candidate_depth`. Then existing RRF / floors (T218 **untouched**). |
| **F10 — Near-dup chrome** | After `rerank_hits`, `dedupe_session_chrome` collapses rows the **detector** flags as chrome that share the **same first contentful line** (keep first / highest effective already sorted). Non-chrome rows **never** collapse, even if they share opening punctuation (`DECISION:`). T260 `dedupe_symbol_stubs` still runs. Distinct `DECISION:` pins never collapse. (Agy m2 — already this rule; helper name locked.) |
| **F11 — Preflight Index two-pass** | `retrieval/src/preflight.rs` Index: collect leading-marker pins (`classify_pin_kind` ≠ Other, or detector-false + leading GLOB) by `updated_at DESC`, then recency-fill other injectable pinned rows excluding those ids, until **existing** word budget / T264 global caps. **Do not** retune `GLOBAL_INDEX_FETCH` 80 (T272 F17). **Do not** change Safety SQL (T279). T272 `safety_ids` skip stands. |
| **F12 — Summary counts follow Index** | CLI `matches("DECISION:")` on the assembled window **stays**. Hermetic pin in Index → `in_context_decisions >= 1`. Do **not** grow CLI `preflight.rs`. |
| **F13 — Memory list recency freeze** | Default `memory list` stays `updated_at DESC` (T216). **Not DoD** to pin-first. Manual “first 5 **or** pin `updated` ≥ ingest” is honesty, not a sort change. No `--kind` flag. |
| **F14 — sync query vault** | Follows `recall_full`. **Do not** edit `sync.rs`. Ledger pane **T271**. Dash `--` **T273**. |
| **F15 — No new CLI flag** | No `--transcripts` / `--pins-only`. Prefer-fill is the remediator. `--symbols` stays T260. |
| **F16 — No DTO keys** | No `is_session` / `pin_kind` on `RecallResult`. Pretty optional `[session]` badge is **soft** (not DoD). JSON `content` raw. |
| **F17 — Depth / floors frozen** | `candidate_depth` 15..50 stays. T218 0.55/0.60 stay. T261 contentless stays. |
| **F18 — forget unfiltered** | `forget --match` does **not** take the two-pass / chrome GLOB. Same as T260 vs stubs. |
| **F19 — Capture independence** | Ranking + Index SQL only. No models on default FTS. No new events. No graph default-on. |
| **F20 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no new crates, workspace **0.1.1**. |
| **F21 — PATH** | Do not `cargo install` unless the user asks. |
| **F22 — Stop-before live vault** | Do not `pin` the live operator vault as implement proof. Hermetic needle only. No live grant bootstrap (T275). No leftover rebind (T276). |
| **F23 — Decline T279 Safety** | Pretty Safety `## Objective` vs `safety sync --dry-run` paths is **T279**. Index/summary only here. Session section may still show recent turns (capture honesty). |
| **F24 — Decline T275 / T276 / T284** | Grants, leftover `7d97a456`, #188 Work/samples. |
| **F25 — Decline T240 F2 / T263 H2 / T211 F25** | Standing. |
| **F26 — last-PR Cursor** | #188 two Mediums → **T284** (already minted). No T285. Open HEAD PR: none. |
| **F27 — Tests** | Naming `function_or_feature__condition__expected_result`. rstest for detector cases. No `unwrap`/`expect`/`panic` in production. `TempEnv` if tests touch ranking envs. |
| **F28 — Cross-model** | Retrieval ranking is FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F29 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F30 — File growth** | `session_chrome.rs` new (detector + F36 helper + F10 dedupe). `classify_pin_kind` + penalty call in `ranking.rs`. Two-pass + F35 binds in `lexical.rs` `match_query` (or a helper in that file). Index two-pass in `retrieval/preflight.rs`. Dedupe call in `recall.rs`. **Do not** grow `project.rs`, CLI `preflight.rs`, `sync.rs`. |
| **F31 — Existing tests stay green** | T211 leading `DECISION:` / `ASSISTANT: DECISION:` units; T260 GLOB/exclude; T216 list recency; T248/T270 retention; T272 Index skip; T207 empty; T218 floor units. Add **new** units for buried-marker → Other (today would be Decision). |
| **F32 — Docs** | CAPABILITIES pin-type row: leading-line + chrome penalty + two-pass. CHANGELOG T274. PROTOCOL-COMPAT: no new required keys (N−1 ignore). WORKFLOWS “what did we decide” still `recall`. |
| **F33 — PowerShell** | `;` not `&&`. |
| **F34 — substring fallback** | T105/T261: LIKE on small vaults. Apply detector penalty/prefer-fill in memory after substring; do **not** skip LIKE. `--global` leftover skip-LIKE (10k) stays. |
| **F35 — Pass-2 ids are bound** | Pass-2 `mp.memory_id NOT IN (?,?,…)` uses dynamically generated `?` placeholders and appends ids onto `params_vec` (same `params_from_iter` as live `match_query`). **Forbidden** to `format!` UUID strings into the SQL. `pass1` empty → omit the `NOT IN` clause (do not emit `NOT IN ()`). (Agy m1.) |
| **F36 — Authority GLOB helper** | `session_chrome.rs` owns `authority_glob_sql(column: &str) -> String` (T260 `symbol_stub_sql_exclusion` shape: identifier-checked, bind-free GLOB list). Callers pass `mp.content` / `m.content`. **Not** a hardcoded `mp.content` const (Index/lexical column names differ). (Agy O1 folded as helper, not a frozen `mp.` const.) |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `classify_pin_kind("DECISION: needle")` Decision; `"ASSISTANT: DECISION: x"` Decision; `"INVARIANT: rule"` Constraint; `"## Objective\n… decision: buried …"` **Other**; JSON `"decisions": [` body **Other**; `"hotspot: file"` at start Hotspot; `"just a chat turn"` Other. Case-insensitive leading `decision:`. **Required red.** |
| **AC2** | Unit rstest `#[case]`: detector true for the five closed prefixes (+ optional `{`+`"decisions":`); false for `DECISION: …` and `CONSTRAINT: …` and a normal chat sentence. |
| **AC3** | `rerank_hits`: chrome hit BM25 **−12** (better FTS) vs leading `DECISION:` BM25 **−2** → pin **first**. Penalty applied. **Required red.** |
| **AC4** | Retrieval hermetic: 15 chrome rows MATCH the query + 1 leading `DECISION: T274-rank-needle-<uuid>` that also MATCHes → `recall_full` `--limit 5` hit **#1** is the pin (`memory_id` + content starts with `DECISION:`). Proves two-pass, not only rerank. **Required red.** |
| **AC5** | Two chrome rows, identical first line `## Objective` → after pipeline one chrome row remains (plus the pin if present). Two leading `DECISION:` pins with **different** bodies stay two even if both start with `DECISION:` (Agy m2 / F10). |
| **AC6** | Hermetic preflight: pin `DECISION: T274-index-needle-<uuid>` + newer `## Objective` dump containing buried `CONSTRAINT:`. Index contains the pin; dump does **not** monopolize Index (pin present even if dump is newer). Safety SQL **unasserted** (T279). **Required red.** |
| **AC7** | Same fixture: `preflight --summary` (or in-process summary counts on `context.text`) `in_context_decisions >= 1`. Existing T220 AC5 still green. |
| **AC8** | T211 `rerank_hits__plan_below_shipped_same_track__ac1` still passes (leading DECISION unchanged). |
| **AC9** | T260 default exclude still drops `Function foo (src/a.rs:1)` without `--symbols`. |
| **AC10** | Empty / contentless recall still T207 / T261 (no chrome work). |
| **AC11** | Store `list_memories` recency order unit still `updated_at DESC, memory_id ASC`. |
| **AC12** | `forget --match` still finds a chrome row (unfiltered MATCH). Unit or hermetic. |
| **AC13** | Compact recall JSON: no new required keys; serde of a pin hit still has T180/T218 fields only. |
| **AC14** | Hermetic CLI: `ai-brains recall "T274-rank-needle-<uuid>" --limit 5 --format pretty --no-bridge` hit #1 is the pin (not `## Objective`). EXIT 0. **Required red** (CLI). |
| **AC15** | Hermetic `sync query "T274-rank-needle-<uuid>" --no-bridge --limit 5`: vault section top is the pin. Ledger pane may be empty — **do not** assert ledger. |
| **AC16** | Unit: semantic-arm prefer-fill (authority first in a shuffled `Vec<RecallHit>` before fuse) — **no** live HTTP. `--semantic` CLI e2e optional if :8083 down. |
| **AC17** | Lexical SQL for pass 1 contains `GLOB` + `LIMIT` (not unbounded SELECT + Rust-only filter). Pass-2 SQL, when `pass1` is non-empty, contains `NOT IN (` with **only** `?` placeholders (count == `pass1.len()`) — no UUID literals in the string (F35). Helper/unit reads the SQL string or a `#[cfg(test)]` fragment. Guard, not Phase-1 red. |

---

## 5. Design notes

### 5.1 Leading-line scan (F2)

```
strip ASSISTANT: once
trim
first non-empty line
trim_start
ascii-lowercase
starts_with constraint: | decision: | invariant: | hotspot:
```

Do **not** `find()` in the whole body. Preflight pretty may still strip `ASSISTANT:` for display (T224); classification uses raw stored content.

### 5.2 Two-pass MATCH (F7)

```
depth = candidate_depth(limit)
pass1 = MATCH expr AND authority_glob AND status=pinned [AND NOT symbol] ORDER BY rank LIMIT depth
if pass1.len() == depth: return pass1
pass2 = MATCH expr AND id NOT IN pass1 AND status=pinned [AND NOT symbol] ORDER BY rank LIMIT (depth - pass1.len())
return pass1 ++ pass2   -- rerank_hits will sort
```

Pass-2 ids: `format!(" AND mp.memory_id NOT IN ({})", (0..pass1.len()).map(|_| "?").collect::<Vec<_>>().join(", "))` then `params_vec.extend(pass1_ids)` (F35). Empty `pass1` → skip the clause (do not emit `NOT IN ()`). Parameterize MATCH expr once (T90 sanitize already done by caller).

Authority GLOB via `authority_glob_sql("mp.content")` (F36; bind-free, identifier-checked like `symbol_stub_sql_exclusion`):

```
col GLOB 'DECISION:*' OR col GLOB 'CONSTRAINT:*' OR col GLOB 'INVARIANT:*'
OR col GLOB 'ASSISTANT: DECISION:*' OR …
```

HOTSPOT is **not** in pass-1 GLOB (Index still has HOTSPOT via recency/Safety). Pass 1 is **decision-class authority** (DECISION/CONSTRAINT/INVARIANT). Leading HOTSPOT still gets kind boost if it appears in pass 2.

### 5.3 Index two-pass (F11)

Do **not** run FTS. Scan pinned injectable rows:

1. Push leading-marker rows (F2) first, `updated_at DESC`.
2. Then other rows `updated_at DESC` excluding ids in (1).
3. Stop at existing word-budget / global caps.

Reuse `classify_pin_kind` (after F2 lift). Session chrome with buried CONSTRAINT: is Other → pass 2.

### 5.4 Penalty vs multiplicative

Stay in T211 additive composite. Chrome `effective -= 16`. A BM25 −12 dump becomes ~−4 after polarity/recency vs a DECISION ~5. Pin wins **if present**. Two-pass is what makes it present.

### 5.5 `search` / daemon / sync

All `recall_full` callers inherit F7–F10. Construction sites that skip `recall_full` are out of scope unless they duplicate MATCH (forget stays unfiltered).

---

## 6. Non-goals

- Hard-exclude all session ingest (breaks “what did we discuss”).
- `source_tag` column + replay (T260 soft).
- Safety = Ledgerful hotspots (**T279**).
- Policy bootstrap / briefing authority (**T275** / T263 H2).
- Leftover `7d97a456` rebind (**T276**).
- Retention Work / apply samples (**T284**).
- Memory-list pin-first reorder (T216).
- `--transcripts` / `--pins-only`.
- FTS title/body split / `bm25()` weights / rusqlite 0.40.
- Raise 750 ms, clap 5, silent Scope switch, live `.env`, `cargo install`.
- LLM / cross-encoder re-ranker; ANN; T211 F25 blend.

---

## 7. Verification plan (TDD)

**Phase 1 red (required before green):** AC1, AC3, AC4, AC6, AC14.

Then green: `session_chrome.rs` + F2 lift + F6 penalty + F7 two-pass + F10 dedupe + F11 Index.

**Stay green:** AC8–AC13, T260, T216, T272 skip, T207/T261.

Targeted: `cargo nextest run -p ai-brains-retrieval --lib` + CLI hermetic filter `recall_pin_rank` / `preflight` Index test + `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`.

Full workspace gate only at implement closeout — **not** a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Two-pass doubles MATCH cost | LIMIT 15 each; same sanitized expr; no LIKE. |
| GLOB misses lowercase `decision:` | Detector SoT + in-memory prefer on blended hits (F8). |
| Chrome detector overfits | Closed list; rstest cases; markdown `# Heading` without those prefixes stays Other **without** penalty. |
| Index fills only old pins | Recency-fill after authority until **existing** budget. |
| Buried-marker tests fail | Intentional F2 lift. Fold-in grep: ranking units are all leading-line; production caller `sync.rs:523` inherits F2 (chrome-with-buried-`decision:` is no longer Plan-class — honest). Phase 0 re-grep `classify_pin_kind` (OpenCode o3). |
| `forget` would hide chrome | F18 — unfiltered. |
| Hotspot `preflight.rs` CLI | Retrieval crate only. |
| Semantic HTTP flake | AC16 unit-only. |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| recall/search/semantic/preflight Index/summary dumps over pins | **Absorb** F1–F12 / AC1–AC7 / AC14–AC15 |
| `memory list` just-now ingest | **Partial F13** — recency stays; Manual OR same-day pin |
| `sync query` vault dumps | **Absorb F14** via `recall_full` / AC15 |
| Preflight Safety = `## Objective` | **Decline F23 → T279** |
| briefing/progressive POLICY_DENIED | **Decline F24 → T275** |
| leftover `7d97a456` / `--global` junk | **Decline F24 → T276** |
| #188 Work hides CE / apply samples | **Decline F26 → T284** (last-PR Cursor) |
| `context --show` leftover shell | **T282** |
| `project list` leftover-first | **T283** |
| T260 symbol monopoly | **Affirm F / AC9** — do not reopen |
| T211 F4 leftmost-anywhere | **Lift F2** (this is the hole) |
| T211 F9 KIND_+2 | **Affirm F4** — do not bump |
| T211 F25 vault↔ledger RRF | **Decline F25** |
| T218 floors / ANN | **Affirm F17** |
| T216 list ORDER | **Affirm F13** |
| T264 leftover recall drop | **Decline** (F11 there) |
| T240 F2 / T263 H2 / T255 750 ms / T266 JSON | **Decline F25** |
| clap 5 / rusqlite 0.40 / DTO / `cargo install` | **Decline F20 / F21** |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `.changeguard` | **Decline** — not ranking |
| last-PR Cursor #188 | **T284** — still true on `14d42af`; **not this track** |
| Open PR on HEAD | **N/A** — none (Dependabot remotes) |
| Identity mismatch quiet (T242 analog) | **Not T274** — leftover data **T276**; adopt-path **T258** |

---

## 10. Implement order (on go)

1. Phase 0 re-verify (plan.md). FEATURE TX.
2. Red: AC1, AC3, AC4, AC6, AC14.
3. `session_chrome.rs` + AC2.
4. F2 `classify_pin_kind` + F3 INVARIANT + F6 penalty in `rerank_hits`.
5. F7 two-pass in `lexical.rs`; F10 dedupe in `recall.rs`.
6. F9 semantic in-memory prefer (AC16).
7. F11 Index two-pass.
8. Docs F32. Targeted clippy/nextest. Review loop. Full gate. Publish (implement-track Phase 6).

---

## 11. Soft residuals

| Residual | Notes |
|----------|--------|
| Pretty `[session]` badge | Like `[symbol]`; not DoD |
| Multiplicative BM25 boost | Elastic Labs; stay additive |
| `--transcripts` mix flag | Prefer-fill is enough |
| Memory-list pin-first | T216 |
| FTS title column + `bm25()` weights | Schema track |
| `source_tag` on projection | T260 soft |
| Session section still dumps | Capture honesty; not Index |
| PATH until `cargo install` | F21 |
| Lowercase GLOB | F8 detector |
| JSON `{` chrome without `"decisions":` | Closed list; may still rank as Other + BM25 |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/session_chrome.rs` | **New** detector + `authority_glob_sql` + `dedupe_session_chrome` |
| `crates/ai-brains-retrieval/src/ranking.rs` | F2 leading-line; F3 INVARIANT; F6 penalty; units AC1/AC3 |
| `crates/ai-brains-retrieval/src/lexical.rs` | F7 two-pass; F35 bound `NOT IN` |
| `crates/ai-brains-retrieval/src/recall.rs` | F10 after `rerank_hits`; F9 hook if semantic list is local |
| `crates/ai-brains-retrieval/src/lib.rs` | export detector if tests/CLI need it |
| `crates/ai-brains-retrieval/src/preflight.rs` | F11 Index two-pass |
| `crates/ai-brains-retrieval/src/semantic.rs` | F9 in-memory prefer only if hits are built here |
| `crates/ai-brains-cli/tests/…` | AC14 / AC6 hermetic (new or extend) |
| `Docs/CAPABILITIES.md` | pin-type + chrome row |
| `CHANGELOG.md` | T274 |
| `Docs/PROTOCOL-COMPAT.md` | no new required keys (one line) |
| `conductor/conductor.md` / `deferred.md` | Planned / absorb |

**Do not touch:** `project.rs`, CLI `preflight.rs`, `sync.rs`, `forget.rs` MATCH, contracts DTO required keys, `class_based_retention.rs`, live `.env`.

---

## 13. AI fold-in

Inputs: `agy-review.md` + `opencode-review.md` (2026-08-21). **Do not edit those files.** Product tree at fold-in = T270 `#188` `14d42af`. Plan commit `9a99117`. **B 0 / M 0.**

### Pins locked by fold-in

1. **F35 / AC17:** pass-2 `NOT IN` uses `?` placeholders + `params_vec` (Agy m1). Empty pass1 omits the clause.
2. **F36:** `authority_glob_sql(column)` in `session_chrome.rs` (Agy O1 as helper, not a frozen `mp.content` const).
3. **F10 / AC5:** `dedupe_session_chrome` collapses **detector** chrome only; two `DECISION:` pins stay two (Agy m2 already F10).
4. **F7 / §5.2:** `pass1.len() >= depth` → skip pass 2 (Agy O2 already).
5. **§2.1:** HEAD `9a99117`; summary counts volatile (plan 0/0/0 @ 3297; OpenCode 1/1/1; fold-in 0/0/0 @ 3324).
6. **Phase 0:** re-grep `classify_pin_kind` (OpenCode o3). Fold-in grep: ranking units leading-only; `sync.rs:523` inherits F2 — do not edit `sync.rs`.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** parameterized pass-2 `NOT IN` | **Folded** F35 / AC17 |
| Agy | **m2** chrome-only first-line dedupe | **Already** F10; **tightened** AC5 two-`DECISION:` stay two |
| Agy | **O1** `AUTHORITY_GLOB_SQL` const | **Folded as helper** F36 — column argument (`mp.content` vs `m.content`); not a hardcoded `mp.` const |
| Agy | **O2** skip pass 2 when pass1 full | **Already** F7 / §5.2 — F7 table now states it |
| OpenCode | B / M | None filed — live line/symbol table **affirmed** |
| OpenCode | **m1** HEAD `9a99117` vs plan `deabae7` | **Folded** §2.1 — product tree identical |
| OpenCode | **o1** summary 1/1/1 vs plan 0/0/0 | **Folded** §2.1 volatile; fold-in re-dogfood **0/0/0 @ 3324** |
| OpenCode | **o2** hotspot 3.999 vs 3.990 | **Folded** §2.1 snapshot; #1 unchanged |
| OpenCode | **o3** grep `classify_pin_kind` tests on go | **Folded** Phase 0 + §8; fold-in grep already leading-only + `sync.rs:523` |
| both | last-PR #188 Cursor | **Affirm T284** — no T285 |
| both | deferred table | **Affirm** — no new absorb |

No Blockers/Majors to decline. No new placeholder minted.
