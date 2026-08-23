# T285 — Recall/search must surface pins, not review-track dumps

- **Track ID:** T285-RecallRankV2
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / UX / RETRIEVAL
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-22 PATH **0.1.2** — `recall`/`search` **10/4**, `--semantic` **8/4**, `sync query` vault **9/7**. Placeholder minted with T285–T300 (`76c4db9`).
- **Depends on:** T274 ✅ leading-line + two-pass + chrome −16 (live still Q=4); T260 ✅ stub seed skip analog; T211 ✅ `rerank_hits` F40; T218 ✅ dual floor; T276 ✅ `--global` prefer-fill (do **not** reopen F39)
- **Blocks / feeds:** Daily `recall` / `search` / `--semantic` lexical honesty / `sync query` vault half. Preflight Index/summary **T286**. `memory list` ORDER **T287**. Graph neighbors CLI **T293**. Leftover dest upsert **T294**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “recall/search/semantic/sync-vault still chrome Q=4”; T274 closeout “live vault still dumps until install” **reopened** (PATH **is** 0.1.2 / T274 and still Q=4); T274 I1 `ASSISTANT:` + CLI `TAGS:` envelope; live detector miss (`# AI-Brains Session Onboarding Complete`, `# Review of Track`); default `graph_hop_depth=1` chrome seed; T274 AC4 needle-in-both-bodies (insufficient)
- **Not absorbed (DoD):** T286 Index/summary renderer; T287 list ORDER; T293 `graph neighbors` CLI; T276 F39 leftover skip; T294 vault upsert; T279 Safety; T263 H2; T240 F2; T211 F25 blend; T218 floors / ANN; clap 5 / rusqlite 0.40 / DTO keys; raise `candidate_depth`
- **Research date:** 2026-08-22 (plan dogfood HEAD `76c4db9` mint; product `src/` = 0.1.2 `#200` `ae5f6fd`). Fold-in against `ee7dab2`.
- **AI fold-in:** 2026-08-22 `agy-review.md` + `opencode-review.md` (HEAD `ee7dab2`). **Agy B 0 / M 0.** **OpenCode B 0 / M 3.** **Agree:** OpenCode M1 AC4/AC5 needle-in-body redness; M2 CLI `test(graph)` (option a) + pure helper; M3 skip reads `blended` content not the 3-tuple; Agy m1 recency-retry binds; Agy m2 role+TAGS no-panic / trim_start. **Already:** Agy O1 F31; OpenCode m1 F7 post-retain; m3 F2. **Decline:** Agy O2 “dumps without needle” (conflicts with M1); OpenCode O1 retrieval CI extra line; O3 share `ROLE_PREFIXES` via core. Disposition **§13**.
- **Ledger:** planning DOCS TX `515b984b-7f5e-4386-9566-a292efd3afe1`. Fold-in DOCS TX `3a598eff-b7e5-4158-970b-be5e331006a7`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement (hermetic needle is SoT; Manual DoD unique canary is allowed on go). Do **not** rewrite `.env`. Do **not** grow hotspot `project.rs` / `sync.rs` / CLI `preflight.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** raise `candidate_depth`. Do **not** retune T218 floors. Do **not** bump `KIND_*`.

---

## 1. Objective

1. **Pins win daily recall.** `recall` / `search` (and `--semantic` when it falls back to lexical) must return a leading `DECISION:` / `CONSTRAINT:` / `INVARIANT:` pin in **top-3**, and hit **#1** must **not** be session chrome, when the query matches that pin — including the live `ASSISTANT: TAGS: …\nDECISION:` storage shape.
2. **T274 is not enough on this vault.** Two-pass + −16 chrome penalty shipped. Live 0.1.2 still leads with `# AI-Brains Session Onboarding Complete`, `# Review of Track 254`, and `## Objective` because (a) the detector closed list misses those prefixes, (b) tagged pins miss authority GLOB / classify as Other, (c) T274 hermetic put the needle in **both** dump bodies and the pin, (d) default graph hop-1 lets dumps seed more dumps.
3. **Keep dumps recallable.** Do not hard-exclude transcripts. Prefer-fill + envelope + detector + chrome-seed skip + leading-line query bonus.
4. **North star.** Capture independence: ranking/retrieval only. No new events. No hidden CoT. Agents can `recall "what did we decide"` and get the pin.

This unblocks the daily product: T274 proved pin-vs-identical-needle-dumps in an empty vault. The operator vault is 3648 dumps with `status='pinned'`, tagged `ai-brains pin` rows, and graph edges between review sessions.

---

## 2. Live baseline (re-scan 2026-08-22)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `76c4db9` `docs(conductor): mint T285-T300…` (ahead of `origin/main` `ae5f6fd` `#200` by **1**, docs only). Product `src/` = **0.1.2**. Tree **CLEAN**. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. **Has T274.** Hole is in **source + PATH**. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3648**. In-context hotspots **5** / decisions **0** / constraints **0**. Word count **319**. Grants **3 of 3**. Index/summary hole is **T286**. |
| `recall "capture independence event log" --no-bridge --limit 5` | **#1** `# AI-Brains Session Onboarding Complete` score **−12.222**. **#2** `# Review of Track 254`. **#3** chat crumb. **#4** ` ```json` `"decisions": [` (detector **does** flag this; still in top-5). **No leading `DECISION:` in top-5.** |
| `search "DECISION:" --no-bridge --limit 5` | Chat crumbs / review bodies that **mention** `DECISION` (`ASSISTANT: \`query_ledgerful\` … includes DECISION`). **None** start with `DECISION:` / `ASSISTANT: DECISION:`. |
| `recall "DECISION: Capture Independence"` | `# Review of Track 253`, `# Review of Track 255`, ` ```json`. Same hole on an exact-looking query. |
| `recall "SQLCipher page encryption" --semantic --no-bridge --limit 3` | Honesty: `Embedding: ok (no semantic hits above threshold; showing lexical)`. Then T254 review / JSON / `## Objective`. T218 floors stand; lexical fallback is this track. |
| `--global search "DECISION:"` | Same cwd dumps first (T276 F39: preferred fills depth → skip leftover MATCH). Leftover 18k is **T294** / T276 F9, not a rank-v2 SQL exclude. |
| `memory list --limit 5` | Just-now ingest previews. **T287.** |
| Last GitHub PR | [#200](https://github.com/Ryan-AI-Studios/AI-Brains/pull/200) version bump (2026-08-22). `gh pr view --comments`, `/reviews`, `/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081 unreachable). **0 pending / 0 drift** at scan. Hotspot **#1** `project.rs` (3.995). `sync.rs` #2. `session_chrome.rs` **#10** (1.931) — **this is the touch file**; do not grow `project.rs` / `sync.rs`. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why T274 still loses (code + live)

| Layer | Truth |
|-------|--------|
| T274 two-pass is on | `lexical.rs` `prefer_authority: true` → MATCH + `authority_glob_sql` LIMIT depth, then fill. Hermetic `recall_pin_rank` **green**. |
| Pass-1 GLOB is prefix-only | `DECISION:*` / `ASSISTANT: DECISION:*` (and CONSTRAINT/INVARIANT). Live `pin --tag` stores `ASSISTANT: TAGS: t285-canary\nDECISION: …` (`pin.rs` `:53–57`; CLI `memory.rs` `:44` already documents this). GLOB **miss**. `classify_pin_kind` first line is `TAGS: …` → **Other**. Placeholder Manual DoD **`--tag t285-canary` would fail T274 as-is.** |
| Detector closed list | True for `## Objective`, `# Track Plan Review`, `### Track`+`Review`, `# AI-Brains Onboarding`, ` ```json`, `{`+`"decisions":`. **False** for live #1 `# AI-Brains Session Onboarding Complete` and #2 `# Review of Track 254`. Those take BM25 −12 with **no −16**. |
| T274 AC4 needle-in-both | Hermetic dumps **repeat** the unique needle 12× in the `## Objective` body so MATCH hits them **and** the pin. Live topical queries (`capture independence event log`) MATCH long dumps; unique pins may not contain every AND token, so pass-1 is **empty** and pass-2 is chrome. |
| `search "DECISION:"` pass-1 empty | JSON hits are `ASSISTANT:` chat, not `ASSISTANT: DECISION:`. So either this project has **no** untagged leading-marker rows that MATCH, or they lose BM25 LIMIT 15 to dumps. Tagged pins never enter pass-1. |
| Default graph hop-1 | clap `graph_hop_depth` default **1**, `graph_boost` **0.1**. `recall.rs` `:493` expands neighbors whenever hop ≥ 1 (**including lexical**). Retrieval hermetics set `graph_hop_depth: 0` (`recall_pin_rank.rs` `:46`). T260 already drops **stub** seeds before graph; **chrome still seeds**. CAPABILITIES “expansion when `--semantic`” is **stale vs src**. |
| KIND_+2 frozen | T274 F4. A −12.222 dump without chrome penalty still beats a short pin (`base≈2 + KIND 2 + recency 1 ≈ 5`). |
| `candidate_depth(5)=15` | Frozen (T261). Do not raise. Get the pin **into** 15, then rank. |
| T276 F39 | `--global` preferred-fill skip when cwd fills 15. Cwd dumps hide leftover pins. **T294** / not this DoD. |
| FTS one column | `bm25(fts, title_w, body_w)` **N/A**. SQLite FTS5 current docs: column weights need a title/body split. **Decline** schema split. Post-filter + envelope + first-line overlap are the levers. |
| `--semantic` | T218 floor → lexical honesty. Same rank-v2 on the fallback list. No HTTP in hermetic AC. |

Placeholder F “outrank” is resolved at this plan: **envelope strip (role + TAGS) + detector prefixes from live first-lines + pass-1 TAGS-or-authority with in-memory authority retain + recency retry when that retain is empty + chrome must not seed graph (T260 analog) + leading-line query-token bonus inside `rerank_hits` (F40 single sort)**. Boost-only and two-pass-only are proven insufficient.

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Rank SOOT | `retrieval/src/ranking.rs` | `classify_pin_kind` `:101`; `first_contentful_line` `:85` strips **only** `ASSISTANT: `; `rerank_hits` `:260` F40. KIND_* / `SESSION_CHROME_PENALTY` frozen. Envelope + query bonus **here** (first_line + rerank). |
| Detector / GLOB | `session_chrome.rs` **hotspot #10** | Closed list `:14–38`; `authority_glob_sql` `:52`; `prefer_authority_hits` `:139`; `dedupe_session_chrome` `:127`. **Extend this file** (prefixes, TAGS GLOB helper, envelope-aware `is_authority`). |
| Lexical two-pass | `lexical.rs` `:156–207` / `:291–309` | Pass-1 `AuthorityFilter::Prefer` → GLOB + `ORDER BY rank LIMIT`. **Add** TAGS-or-authority SQL; **in-memory retain** authority after envelope; **recency retry** (`ORDER BY mp.updated_at DESC`) only when retain is empty. F35 bound `NOT IN` stands. |
| Pipeline | `recall.rs` `:250–573` | lexical (`prefer_authority: true`) → optional LIKE prefer → blend → **graph hop-1** → `rerank_hits` → chrome dedupe → stub dedupe → truncate. Pass **query** into rerank. Skip chrome parents in the graph loop (T260 `:484` analog). |
| Depth | `hybrid.rs` `candidate_depth` | `limit*3` clamp **15..50**. **Do not raise.** |
| Semantic | `semantic.rs` `:329` `prefer_authority_hits` | Inherits envelope classify. No second BLOB SQL. T218 floors untouched. |
| CLI pin | `pin.rs` `:53–57` | Tags prepend `TAGS: …\n`. Default role **assistant** → stored `ASSISTANT: TAGS: …\nDECISION:`. **Do not change pin write** (would rewrite history). Rank must understand the envelope. |
| CLI recall flags | `main.rs` `:1042–1088` | `graph_boost` default **0.1**; `graph_hop_depth` default **1**. **No new flags.** |
| `search` | T243 alias of `recall` | Follows automatically. |
| `sync query` vault | `sync.rs` `recall_full`; `classify_pin_kind` **`:532`** (ledger-first probe; plan `:529` was off-by-3) | Follows automatically. **Do not grow `sync.rs` (hotspot #2).** Ledger pane **T271**. |
| Graph loop | `recall.rs` **`:492–552`** `#[cfg(feature = "graph")]` | Snapshot tuple is `(id, score, score_kind)` **no content**. Hits in `blended` **do** carry `content` (`RecallHit` `:40–60`). F10 skip **must** read parent content from `blended` (OpenCode M3). Default retrieval tests pass `graph: None` / hop **0**. CI graph job is **CLI-only** (`ci.yml` `:109` / `:173` `-p ai-brains-cli --features graph -E 'test(graph)'`). |
| `forget --match` | unfiltered MATCH | **Stay unfiltered** (T260/T274 F18). |
| Preflight Index | `retrieval/src/preflight.rs` | Two-pass already. Renderer/summary counts **T286**. **Do not edit as DoD.** |
| Contracts | `RecallResult` `:18–36` | Additive optional only. **No** `is_session` / `pin_kind` / `envelope` wire key. |
| Hermetic T274 | `crates/ai-brains-cli/tests/recall_pin_rank.rs`; `crates/ai-brains-retrieval/tests/recall_pin_rank.rs` | Untagged pin vs dumps **with needle in dump bodies**; `graph_hop_depth: 0`. **Stay green.** New tests for envelope / asymmetric needle / hop-1. |
| `project.rs` | hotspot **#1** | **Do not touch.** |

### 2.4 Dependency / standards research (2026-08-22) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (`cargo search`). **No clap 5.** | **No bump.** No new flags. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** JSON keys frozen. |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot #62) | **No bump.** Recency parse stays. |
| `rusqlite` | lock **0.39.0** | crates.io **0.40.2** (Dependabot #61) | **No bump.** MATCH + GLOB only. |
| `uuid` | lock **1.23.1** | crates.io **1.25.0** | **No bump.** |
| `tokio` | workspace **1.52** / lock **1.52.3** | crates.io **1.53.1** (`#59`) | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.2** | — | **No bump** |
| New crates | — | — | **Zero.** No `regex` in retrieval (T211 F18). No simhash. |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| FTS5 BM25 more-negative-better; `bm25(table, w…)` needs **per-column** weights | [SQLite FTS5](https://www.sqlite.org/fts5.html) (fetched 2026-08-22; bm25 section) | One `content` column → **no** title weight. Decline schema split. |
| Title-weighted BM25 is the Engram pattern | [Engram #241](https://github.com/Gentleman-Programming/engram/issues/241) (2026-04, approved) | 6-column FTS. We cannot copy without a migration. First-line overlap bonus is the one-column analog. |
| Phrase **position** beats BM25 bag-of-words for titles | [APSW `position_rank`](https://rogerbinns.github.io/apsw/textsearch.html) (docs current 2026-07) | BM25 ignores “early in the document.” Leading-line token overlap is that signal. |
| Additive “+2” is brittle vs BM25 scale | [Elasticsearch Labs 2025-12 multiplicative BM25](https://www.elastic.co/search-labs/blog/bm25-ranking-multiplicative-boosting-elasticsearch) | Do **not** bump KIND_DECISION. Stay on T211 additive composite. Query-line bonus **+16** (same scale as `SESSION_CHROME_PENALTY` / `SYMBOL_PENALTY`). Multiplicative `function_score` would need ES or a second sort — **decline**. |
| Filter “is authority” then score | [ES `function_score` filter+weight](https://www.elastic.co/docs/reference/query-languages/query-dsl/query-dsl-function-score-query) (current) | Pass-1 GLOB-or-TAGS + in-memory retain is the SQLite edition. |
| Do not let noise seed expansion | T260 F8 live `recall.rs`: drop stub-shaped hits **before** graph | Chrome-shaped hits must not seed neighbors. |
| clap 4 current | docs.rs/clap/4.6.6 | No new args. clap 5 not released. |
| Bind `NOT IN`; do not interpolate UUIDs | [SQLite expr parameters](https://www.sqlite.org/lang_expr.html); live F35 | Recency retry and pass-2 stay `?` placeholders. |

**N/A:** SQLCipher page encrypt, schtasks, T180 new required keys, Windows service, llama.cpp `/health`, Safety GLOB (T279 Completed).

**Could not verify:** exact COUNT of leading-marker (post-envelope) rows in `3581317d` without vault SQL (do not print `AI_BRAINS_KEY`). Hermetic unique needle + live Manual canary are the proof, not live archaeology.

**ledgerful / ai-brains:** `preflight --summary` 0/0 decisions vs **3648** pins; live recall still onboarding/review; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "classify_pin_kind"` → `ranking.rs:101` + `session_chrome.rs:41` + `sync.rs:529` (inherit envelope; **do not edit `sync.rs`**); `search "is_session_chrome"` → `session_chrome.rs`; `scan --impact` CLEAN at `76c4db9`; hotspots `session_chrome.rs` #10. Semantic/`sync query` returned T274 `## Objective` review-track dumps — evidence of the hole, not SoT for decisions.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `515b984b`. Fold-in is DOCS TX `3a598eff`. Implement starts a **FEATURE** TX. |
| **F1 — Rank, do not delete** | Session ingest stays recallable. **Do not** hard-exclude transcripts. **Do not** forget/migrate them. |
| **F2 — Envelope strip (new)** | `first_contentful_line` / `classify_pin_kind` / `is_session_chrome` / `is_authority_pin_content` inspect the body **after** a pin envelope: (1) strip one leading role token `ASSISTANT:` / `USER:` / `SYSTEM:` via `strip_prefix(token)` then **`trim_start`** (newlines/tabs count — CLI `display_text::strip_role_prefix` shape; live `strip_assistant_prefix("ASSISTANT: ")` is **too strict** for `ASSISTANT:\nDECISION:`); (2) if the next contentful line starts with `tags:` (ASCII case-insensitive), **skip that line**; (3) first remaining contentful line is the marker/chrome line. Empty / shorter-than-prefix / all-whitespace → `""` / `PinKind::Other` — **no panic, no unwrap**. Duplicate the three tokens next to the envelope helper in `ranking.rs` (retrieval cannot import CLI `pub(crate)` `ROLE_PREFIXES`; **do not** move them into `ai-brains-core` this track — OpenCode O3 declined). `strip_assistant_prefix` stays as a primitive (T274 units that call it directly stay green). Stored JSON `content` stays raw (`ASSISTANT: TAGS: …`). |
| **F3 — T274 leading-line stands** | After the envelope, marker must **start** that line. Buried JSON `"decisions": [` and skill-body mentions stay **Other**. Leading `INVARIANT:` stays Constraint. |
| **F4 — KIND_* / chrome −16 / depth / floors frozen** | Do **not** change `KIND_*`, `PLAN_PENALTY`, `RECENCY_SCALE`, `SESSION_CHROME_PENALTY` (16), `SYMBOL_PENALTY`, `candidate_depth` 15..50, T218 0.55/0.60. |
| **F5 — Detector prefixes (additive)** | Closed list **adds** (first contentful line, case-insensitive `starts_with`): `# ai-brains session onboarding`; `# review of track`. Existing five prefixes + `{`+`"decisions":` stay. **Not** every `# Heading`. |
| **F6 — Leading-line query bonus** | Inside **the same** `rerank_hits` sort (F40 — **no** second final sort): when the hit is authority after envelope **and** the first contentful line contains ≥1 **contentful** query token (`contentful_tokens(extract_fts_tokens(query))`, ASCII case-insensitive substring), add `LEADING_QUERY_BONUS = 16.0`. Chrome still −16. Call site `recall_full` passes the raw query. Existing `rerank_hits(hits)` keeps `query=None` (bonus 0) so T211/T274 units stay green. |
| **F7 — Pass-1 GLOB-or-TAGS** | Pass-1 SQL is existing `authority_glob_sql` **OR** `tags_envelope_sql(column)` (`col GLOB 'TAGS:*' OR col GLOB 'ASSISTANT: TAGS:*'`; identifier-checked like T274 F36). Then **in-memory retain** `is_authority_pin_content`. Live `match_query` `:180` returns early when **SQL** `pass1.len() >= limit` **before** retain — **move the gate** to post-retain (OpenCode m1). `retain.len() >= depth` uses **post-retain** len (do not skip pass-2 / F8 because 15 tagged dumps filled SQL). Dropped dumps do not occupy the authority slot. |
| **F8 — Recency retry** | If post-retain pass-1 is **empty**, one more MATCH with the same Prefer filter but `ORDER BY mp.updated_at DESC, mp.memory_id ASC LIMIT depth` (not `rank`), then in-memory retain again. This is a **retry of pass-1** (same MATCH expr, different ORDER) — **not** a third MATCH family, **not** `substring_fallback`, **not** a second `match_limit_bound` on pass-2 (OpenCode m2). Newest matching tagged/untagged pins enter when BM25-of-TAGS was all dumps. **Do not** use recency as the primary pass-1 when retain is already full. ExcludeIds on pass-2 (and on retry if ids are excluded) use `bound_not_in_sql` + `params_from_iter` only (F34). |
| **F9 — GLOB still a subset** | SQL GLOB stays case-sensitive. Envelope + in-memory retain are SoT for lowercase `decision:` and TAGS (T274 F8 analog). |
| **F10 — Chrome must not seed graph** | After blend, **before** neighbor expansion: skip parent hits where `parent_seeds_graph_neighbors` is false (F36). Read **`hit.content` from `blended`** (or snapshot a 4th `content` field). **Forbidden** to skip using only the live `(id, score, score_kind)` tuple — that tuple has no content and a stub would always expand (OpenCode M3). T260 stub-seed skip stays. Do **not** set default `graph_hop_depth` to 0 (would disable T66). Do **not** add `--no-graph`. |
| **F11 — Semantic arm** | No second embedding SQL. `prefer_authority_hits` inherits envelope classify. Dual floors **untouched**. Hermetic AC **no** HTTP. `--semantic` CLI e2e optional if :8083 down; lexical fallback list must still put the pin in top-3 when the floor yields “showing lexical.” |
| **F12 — Near-dup chrome** | `dedupe_session_chrome` after rerank stands. Distinct `DECISION:` pins never collapse. |
| **F13 — Decline Index / summary** | Preflight Index two-pass / `--summary` in-context counts are **T286**. Retrieval `preflight.rs` Index SQL **not** DoD here. |
| **F14 — Decline memory-list ORDER** | T216 / **T287**. |
| **F15 — sync query vault** | Follows `recall_full`. **Do not** edit `sync.rs`. Ledger pane **T271**. |
| **F16 — No new CLI flag** | No `--transcripts` / `--pins-only` / `--no-graph`. `--symbols` stays T260. Graph defaults stay. |
| **F17 — No DTO keys** | No `is_session` / `pin_kind` / `envelope` on `RecallResult`. JSON `content` raw. PROTOCOL-COMPAT: N−1 ignore. |
| **F18 — forget unfiltered** | `forget --match` does **not** take envelope GLOB / two-pass / chrome skip. |
| **F19 — Capture independence** | Ranking only. No models on default FTS. No new events. No graph default-on Cargo. **Do not rewrite** `pin.rs` stored shape. |
| **F20 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no new crates, workspace **0.1.2**. |
| **F21 — PATH** | Do not `cargo install` unless the user asks. |
| **F22 — Live vault pin** | Do **not** pin production DECISIONs as implement. Hermetic unique needle is SoT. Manual DoD **unique canary** (uuid in the string) is allowed on go — that is the command test, not architecture. |
| **F23 — Decline leftover F39** | T276 preferred-fill skip when cwd fills depth stays. Leftover dest-missing **T294**. This track ranks **within** the retrieved set. |
| **F24 — Decline T279 / T263 H2 / T240 F2 / T211 F25** | Standing. |
| **F25 — last-PR Cursor** | #200 empty → **N/A**. Dependabot remotes not this track. **No T301.** |
| **F26 — Tests** | Naming `function_or_feature__condition__expected_result`. rstest `#[case]` for **new** detector prefixes (absorb T274 closeout low). No `unwrap`/`expect`/`panic` in production. `TempEnv` if tests touch ranking envs. |
| **F27 — Cross-model** | Retrieval ranking is FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F28 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F29 — File growth** | Envelope + `LEADING_QUERY_BONUS` in `ranking.rs`. Detector prefixes + `tags_envelope_sql` + F36 helper in `session_chrome.rs`. Pass-1 OR-TAGS + retain + recency retry in `lexical.rs`. Graph chrome-seed skip (read `blended` content) + query-aware rerank call in `recall.rs`. New CLI hermetic `tests/recall_rank_v2.rs` + graph-on `test(graph)` file (do **not** weaken T274 `recall_pin_rank.rs`). **Do not** grow `project.rs`, CLI `preflight.rs`, `sync.rs`, `pin.rs` write path, `.github/workflows/ci.yml`. |
| **F30 — Existing tests stay green** | T274 AC1–AC5/AC14 (untagged + needle-in-dump-bodies); T211 leading DECISION; T260 exclude; T216 list recency; T207/T261 empty; T218 floors; T276 prefer-fill. Add **new** units for TAGS envelope → Decision; live prefixes → chrome; first-line-asymmetric / body-MATCH dumps; F36 helper; CLI `test(graph)` AC17. |
| **F31 — Docs** | CAPABILITIES pin-type row: envelope + new prefixes + leading-query bonus + chrome-seed skip; fix the stale “graph expansion only when `--semantic`” sentence to match src (hop-1 default, chrome parents skipped). CHANGELOG T285. PROTOCOL-COMPAT: no new required keys. WORKFLOWS “what did we decide” still `recall`. |
| **F32 — PowerShell** | `;` not `&&`. |
| **F33 — substring fallback** | T105/T261: LIKE on small vaults. Envelope prefer-fill after substring stands. `--global` leftover skip-LIKE (10k) stays. |
| **F34 — Pass-2 / retry ids bound** | T274 F35 stands. Recency-retry **and** pass-2 `NOT IN` use `bound_not_in_sql` + dynamically generated `?` placeholders + `params_from_iter` (Agy m1). **Forbidden** to `format!` UUID strings into SQL. Empty retain → omit `NOT IN` (do not emit `NOT IN ()`). |
| **F35 — search alias** | T243: `search` is `recall`. One hermetic covers both binary names **or** document that argv0 alias is clap-level (AC may call `recall` only). Manual DoD still runs **both** commands. |
| **F36 — Seed helper** | `parent_seeds_graph_neighbors(content: &str) -> bool` is `!is_session_chrome(content)` **after envelope**. Lives in `session_chrome.rs` (or `recall.rs` next to the loop). Unit-tested **without** `--features graph` (graph block is `#[cfg(feature = "graph")]` `:492`). |
| **F37 — Graph-on CLI hermetic** | AC17 lives in `crates/ai-brains-cli/tests/` with `graph` in the test name (nextest `-E 'test(graph)'`) and `#[cfg(feature = "graph")]`, same pattern as `graph_live_projection.rs`. Reuses existing CI job (`ci.yml` `:109` / `:173`). **Do not** add a retrieval `--features graph` CI line (OpenCode O1 declined as DoD). **Do not** add a new GHA job. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `classify_pin_kind("ASSISTANT: TAGS: t285-canary\nDECISION: needle")` **Decision**; `"TAGS: x\nCONSTRAINT: rule"` **Constraint**; `"ASSISTANT: TAGS: x\n## Objective\nburied decision: y"` **Other**; `"ASSISTANT:\nDECISION: nl"` **Decision** (token + `trim_start`); `""` / `"ASSISTANT:"` → **Other** (no panic). Untagged `ASSISTANT: DECISION: x` still Decision (T274). **Required red.** |
| **AC2** | rstest `#[case]`: detector **true** for `# AI-Brains Session Onboarding Complete`, `# Review of Track 254: …`, plus T274 closed prefixes. **False** for `DECISION: …`, `CONSTRAINT: …`, `# Heading without chrome prefixes`. |
| **AC3** | `rerank_hits_with_query`: chrome `# AI-Brains Session Onboarding Complete\n… capture independence …` BM25 **−12** vs leading `DECISION: Capture independence remains` BM25 **−2**, query `"capture independence"` → pin **first** (detector −16 **and/or** leading-query +16). **Required red.** |
| **AC4** | Retrieval hermetic **first-line asymmetric, body MATCH** (OpenCode M1): 15 dumps whose **first line** is `# Review of Track` / onboarding and whose **body repeats the unique needle** (so FTS MATCH hits them with high BM25 — T274-class) + 1 `ASSISTANT: TAGS: t285\nDECISION: {needle}` → `recall_full` `--limit 5` hit **#1** is the pin (`memory_id` + envelope-stripped content starts with `DECISION:`). Graph may be `None` (graph-off crate). Dumps **without** the needle in the body are **not** this AC (would be a sole-candidate pass). **Required red.** |
| **AC5** | Same MATCH dumps as AC4 (needle **in dump bodies**, chrome first line) but pin is **untagged** `DECISION: {needle}` → pin still #1 (regression vs “we only fixed TAGS”). |
| **AC6** | Unit (graph-off): `parent_seeds_graph_neighbors` is **false** for `# Review of Track …` / onboarding / `## Objective` (post-envelope, including `ASSISTANT: TAGS: x\n## Objective`) and **true** for `DECISION: …` / `ASSISTANT: TAGS: t\nDECISION: …`. T260 stub-seed skip stays. **Required red.** |
| **AC7** | T274 `rerank_hits__plan_below_shipped_same_track` / classify buried-Other / CLI `recall_pin_rank` **stay green**. |
| **AC8** | T260 default exclude still drops `Function foo (src/a.rs:1)` without `--symbols`. |
| **AC9** | Empty / contentless still T207 / T261. |
| **AC10** | `forget --match` still finds a chrome row (unfiltered). |
| **AC11** | Compact recall JSON: no new required keys; serde of a tagged-pin hit still has T180/T218 fields only; `content` still includes `ASSISTANT:` / `TAGS:` raw. |
| **AC12** | Hermetic CLI: `ai-brains recall "{needle}" --limit 5 --format pretty --no-bridge` **and** `ai-brains search "{needle}" --limit 5 --format pretty --no-bridge` hit #1 is the tagged pin, **not** `# AI-Brains Session Onboarding` / `# Review of Track` / `## Objective`. EXIT **0**. **Required red** (CLI). |
| **AC13** | Hermetic `sync query "{needle}" --no-bridge --limit 5`: vault section top is the pin. Ledger pane may be empty — **do not** assert ledger. |
| **AC14** | `--semantic` hermetic: if embed skipped/floor-empty, lexical fallback list still has the pin in **top-3**. No live HTTP required. |
| **AC15** | Unit: pass-1 SQL string for Prefer contains `GLOB` + `TAGS:` + `LIMIT` (not unbounded SELECT). Recency-retry SQL contains `updated_at` and **only** `?` placeholders for ids/limit — no UUID literals in the string (F34 / Agy m1). Guard, not Phase-1 red. |
| **AC16** | Store `list_memories` recency order unit still `updated_at DESC` (T216 / T287 freeze). |
| **AC17** | CLI `#[cfg(feature = "graph")]` hermetic whose **test name contains `graph`** (CI `-E 'test(graph)'`): a chrome dump that MATCHES the query is a hit; a neighbor memory that does **not** MATCH is **absent** from results (chrome did not seed). An authority pin parent **may** still add a neighbor. Pattern: `graph_live_projection.rs`. Not required red on graph-off workspace nextest. |

---

## 5. Design notes

### 5.1 Envelope (F2)

```
strip_prefix one of USER:|ASSISTANT:|SYSTEM:  (no required space)
trim_start  (newlines/tabs ok)
if remainder empty → Other
if first contentful line starts_with tags: (ascii lower): skip that line
first remaining contentful line → classify / chrome / authority
```

Do **not** `find("DECISION:")` in the whole body. Do **not** rewrite `pin.rs`. Pretty already strips role for display; JSON stays raw.

### 5.2 Pass-1 (F7 / F8)

```
depth = candidate_depth(limit)
sql_pass1 = MATCH + (authority_glob OR tags_envelope) ORDER BY rank LIMIT depth
retain = authority after envelope          -- gate uses THIS len, not SQL len
if retain.len() >= depth: return retain
if retain is empty:
    sql_retry = same MATCH+filter ORDER BY updated_at DESC, memory_id ASC LIMIT depth
    retain = authority(retry)              -- pass-1 retry, not substring_fallback
pass2 = MATCH excluding retain ids ORDER BY rank LIMIT (depth - retain.len())
return retain ++ pass2
```

Pass-2 may still contain chrome (queries with no matching pin stay honest — no T207 lie). F8 does **not** re-run LIKE / `substring_fallback`.

### 5.3 Chrome-seed skip (F10)

T260: stubs do not seed graph. T285: iterate `blended` (content on the hit); `if !parent_seeds_graph_neighbors(&hit.content) { continue; }` before `get_neighbors`. Do **not** rebuild skip from the `(id, score, kind)` snapshot alone. Authority parents still expand (neighbors then take chrome −16 / query bonus in the **same** `rerank_hits` sort).

### 5.4 Query bonus vs multiplicative

Stay in T211 additive composite. `LEADING_QUERY_BONUS = 16` is the one-column analog of FTS5 title weight / APSW `position_rank`. Elastic Labs multiplicative `function_score` is **not** this track (no ES, no second sort, F40).

### 5.5 `search` / daemon / sync

All `recall_full` callers inherit F2–F10. `forget` stays unfiltered.

---

## 6. Non-goals

- Hard-exclude all session ingest.
- FTS title/body split / `bm25()` column weights / rusqlite 0.40.
- Raise `candidate_depth` / change T218 floors / ANN / T211 F25 vault↔ledger RRF.
- Preflight Index/summary (**T286**).
- `memory list` pin-first (**T287**).
- `graph neighbors` CLI order (**T293**).
- T276 F39 leftover skip / T294 `.env` dest upsert / live leftover rebind.
- Rewrite `pin` storage (drop TAGS line / drop ASSISTANT:).
- Default `graph_hop_depth=0` / new `--no-graph`.
- `--transcripts` / `--pins-only`.
- T263 H2 pin→Approved; T240 F2 silent Scope; clap 5; LLM cross-encoder.
- Live `retention apply --confirm` / `graph rebuild`.
- `.github/workflows/ci.yml` extra retrieval graph job (F37).
- Move `ROLE_PREFIXES` into `ai-brains-core` (OpenCode O3).

---

## 7. Verification plan (TDD)

**Phase 1 red (required before green):** AC1, AC2, AC3, AC4, AC6, AC12.

Then green: envelope + detector prefixes + F7/F8 lexical + F6 query bonus + F10 chrome-seed skip (helper + loop).

**Stay green:** AC7–AC11, AC16, T274, T260, T207/T261, T276 prefer-fill.

Targeted: `cargo nextest run -p ai-brains-retrieval --lib` + `--test recall_pin_rank` + `--test recall_rank_v2` (name on go) + `cargo nextest run -p ai-brains-cli --test recall_pin_rank --test recall_rank_v2` + `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`. Graph-on: `cargo nextest run -p ai-brains-cli --features graph -E "test(graph)"` (existing CI filter; AC17).

Full workspace gate only at implement closeout — **not** a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| TAGS GLOB pulls tagged dumps into SQL LIMIT 15 | In-memory retain; `len` is post-retain; recency retry when empty. |
| Recency retry buries old pins for generic `DECISION:` | Newest matching authority is still a **pin**, not onboarding chrome. Topical unique needles still MATCH the pin first-line (AC4). |
| Detector overfits | Closed additive prefixes from **live** first-lines only; rstest; `# Heading` without those prefixes stays unpenalized. |
| Query bonus on token `DECISION` promotes every authority pin equally | Then KIND/recency/BM25 break ties — still pins, not dumps. |
| Graph skip hides useful neighbors of review sessions | Those sessions stay recallable via MATCH; they must not **explode** the candidate set. Authority pins still expand. |
| Hotspot `session_chrome.rs` #10 | Additive helpers only; no move of T274 units. |
| `sync.rs` inherit envelope | `classify_pin_kind` change is enough; do not edit the file. |
| Semantic HTTP flake | AC14 unit/fallback; no live HTTP required. |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| recall/search/semantic/sync-vault still chrome Q=4 | **Absorb** F1–F12 / AC1–AC6 / AC12–AC14 |
| Placeholder Manual DoD canary + `--tag` | **Absorb** F2 envelope so `--tag` actually works; AC1/AC4/AC12 |
| T274 closeout “live dumps until install” | **Absorb / reopen** — PATH **is** 0.1.2 and still Q=4 |
| T274 closeout detector not rstest | **Absorb** F26 / AC2 for **new** prefixes |
| T274 closeout GLOB lowercase | **Partial F9** — envelope/retain SoT; no lowercase SQL GLOB |
| T274 closeout AC16 helper vs semantic caller | **Decline as DoD** — AC14 fallback; no HTTP |
| T274 AC6 buried CONSTRAINT Index | **Decline → T286** / T279 Safety |
| `memory list` just-now ingest | **Decline → T287** |
| preflight Index `## Objective` / summary 0 vs 3648 | **Decline → T286** |
| `graph neighbors` dump sessions | **Decline → T293** (F10 helps recall only) |
| leftover dest-missing / context skip upsert | **Decline → T294** |
| T276 F39 `--global` skip leftover MATCH | **Decline F23** |
| briefing granted-empty / H2 | **Decline → T288** / T263 H2 standing |
| T240 F2 / T211 F25 / T218 floors / 750 ms / clap 5 / rusqlite 0.40 | **Decline F20 / F24** |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `.changeguard` | **Decline** — not ranking |
| last-PR Cursor #200 | **N/A** empty — **no T301** |
| Open PR on HEAD | **N/A** — none (Dependabot remotes) |
| Identity mismatch leftover `7d97a456` | **Not this track** — T258 / T294 |

---

## 10. Implement order (on go)

1. Phase 0 re-verify (plan.md). FEATURE TX.
2. Red: AC1, AC2, AC3, AC4, AC12.
3. F2 envelope in `ranking.rs` + AC1.
4. F5 detector prefixes + AC2.
5. F6 `rerank_hits_with_query` + AC3.
6. F7/F8 lexical TAGS-or-authority + retain + recency retry; F34 binds.
7. F10/F36 helper + skip in `recall.rs` against `blended` content; pass query into rerank.
8. CLI hermetic `recall_rank_v2.rs` (AC12/AC13). CLI `test(graph)` AC17.
9. Docs F31. Targeted clippy/nextest + graph-on filter. Review loop. Full gate. Publish (implement-track Phase 6).

---

## 11. Soft residuals

| Residual | Notes |
|----------|--------|
| Pretty `[session]` badge | Like `[symbol]`; not DoD |
| Multiplicative BM25 / FTS title column | Elastic Labs / Engram #241; schema track |
| `--transcripts` mix flag | Prefer-fill + envelope is enough |
| More chrome prefixes as vault grows | Closed list; append in a later track if live first-lines show a new family |
| T276 F39 leftover hidden on `--global` | T294 |
| Index/summary still Objective until T286 | Honest split |
| JSON `{` chrome without `"decisions":` | Closed list; may still rank as Other + BM25 |
| PATH until `cargo install` | F21 |
| Retrieval `--features graph` in CI | OpenCode O1 declined; re-trigger if graph-on retrieval tests exist and the CLI job misses them |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/ranking.rs` | F2 envelope; F6 query bonus; units AC1/AC3 |
| `crates/ai-brains-retrieval/src/session_chrome.rs` | F5 prefixes; `tags_envelope_sql`; rstest AC2 |
| `crates/ai-brains-retrieval/src/lexical.rs` | F7/F8/F34 |
| `crates/ai-brains-retrieval/src/recall.rs` | F10 chrome-seed skip; query-aware rerank |
| `crates/ai-brains-retrieval/src/lib.rs` | export envelope / `tags_envelope_sql` / `LEADING_QUERY_BONUS` if tests need them |
| `crates/ai-brains-cli/tests/recall_rank_v2.rs` | **New** AC12/AC13 (tagged + body-MATCH dumps) |
| `crates/ai-brains-cli/tests/recall_rank_v2_graph.rs` (name on go) | **New** AC17 `#[cfg(feature = "graph")]` + `graph` in test name |
| `crates/ai-brains-retrieval/tests/recall_rank_v2.rs` | **New** AC4/AC5 (body-MATCH dumps); AC6 helper units |
| `Docs/CAPABILITIES.md` | pin-type + graph-seed sentence |
| `CHANGELOG.md` | T285 |
| `Docs/PROTOCOL-COMPAT.md` | no new required keys (one line) |
| `conductor/conductor.md` / `deferred.md` | Planned / absorb |

**Do not touch:** `project.rs`, CLI `preflight.rs`, `sync.rs`, `forget.rs` MATCH, `pin.rs` write, contracts DTO required keys, live `.env`.

---

## 13. AI fold-in

Inputs: `agy-review.md` + `opencode-review.md` (2026-08-22, HEAD `ee7dab2`). Product `src/` = 0.1.2 `#200`. **Do not edit those files.** **Agy B 0 / M 0.** **OpenCode B 0 / M 3.** last-PR #200 still empty. **No T301.**

### Pins locked by fold-in

1. **AC4/AC5 (OpenCode M1):** dumps **repeat the unique needle in the body** (FTS MATCH + high BM25) with chrome **first line**. A dump body without the needle is a sole-candidate pass — not red. First-line remains asymmetric vs T274 `## Objective`.
2. **F10/F36/M3:** skip reads `blended` hit `content` (or a 4-tuple). Live snapshot `(id, score, kind)` at `recall.rs:498–501` has no content.
3. **F37/AC17 (OpenCode M2 option a):** CLI `test(graph)` hermetic; reuse `ci.yml` `:109`/`:173`. No new GHA job. No retrieval `--features graph` CI line (O1 declined as DoD).
4. **F7 gate (OpenCode m1):** post-retain `len`, not SQL `pass1.len()` at live `:180`.
5. **F8 (OpenCode m2):** recency retry is pass-1 retry, not `substring_fallback`.
6. **F2 (Agy m2):** `strip_prefix(token)` + `trim_start`; empty → Other; no panic. Tokens duplicated in `ranking.rs`, not `ai-brains-core` (O3 declined).
7. **F34 (Agy m1):** recency-retry `NOT IN` is `?` + `params_from_iter` only.
8. **F31 (Agy O1):** CAPABILITIES hop-1 default + chrome-parent skip — already planned.
9. **sync.rs:532** (OpenCode O2): classify_pin_kind line; do not edit the file.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** recency-retry `NOT IN` bound via `params_from_iter` | **Folded** F34 / AC15 |
| Agy | **m2** role strip + whitespace/newlines; no panic on short/empty | **Folded** F2 / AC1 |
| Agy | **O1** CAPABILITIES graph-expansion sentence | **Already** F31 |
| Agy | **O2** dumps without needle in bodies | **Decline** — conflicts with OpenCode M1; body-without-needle is not red |
| OpenCode | **M1** AC4 redness needs needle in dump bodies | **Folded** AC4 / AC5 |
| OpenCode | **M2** hop-1 untestable graph-off; pick CLI `test(graph)` | **Folded** F37 / AC17 + F36 unit for graph-off |
| OpenCode | **M3** snapshot tuple omits content; skip from `blended` | **Folded** F10 |
| OpenCode | **m1** live `:180` gate is pre-retain | **Already** F7; **tightened** “move the gate” |
| OpenCode | **m2** F8 is not a third MATCH family | **Folded** F8 / §5.2 |
| OpenCode | **m3** envelope order role then TAGS | **Already** F2 |
| OpenCode | **O1** extra retrieval `--features graph` CI line | **Decline as DoD** — CLI graph job is enough. Re-trigger: graph-on retrieval tests exist and CI misses them |
| OpenCode | **O2** `sync.rs:529` → `:532` | **Folded** §2.3 |
| OpenCode | **O3** share `ROLE_PREFIXES` via `ai-brains-core` | **Decline** — retrieval duplicates three tokens; core share is another track. Re-trigger: owner wants one SOOT across CLI+retrieval |
| both | last-PR #200 Cursor | **Affirm N/A** — no T301 |
| both | deferred Index / memory-list / neighbors | **Affirm** T286 / T287 / T293 |

No Blockers. OpenCode M1–M3 folded (not declined). No new placeholder minted.
