# T325 — FTS authority-OR fill must recency-retry (T312 F8 leftover)

- **Track ID:** T325-FtsOrFillRecency
- **Status:** ✅ **Completed** (PR [#247](https://github.com/Ryan-AI-Studios/AI-Brains/pull/247) squash `9119c74`)
- **Category:** BUGFIX / RETRIEVAL
- **Owner:** Grok
- **Source:** Last-PR Cursor Bugbot on [#230](https://github.com/Ryan-AI-Studios/AI-Brains/pull/230) (T312, `mergedAt` **2026-08-28T02:35:31Z**). Medium comment [`3877408710`](https://github.com/Ryan-AI-Studios/AI-Brains/pull/230#discussion_r3877408710): F8 authority-OR path runs only BM25 `Prefer` MATCH + in-memory retain; when that retain is empty it does **not** recency-retry, unlike the AND pass. Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T312 ✅ F8 authority-OR fill (`lexical.rs` `match_query`); T274 ✅ `AuthorityFilter::PreferRecency`; T217 ✅ `match_or` / `select_or_tokens`; T285 ✅ GLOB-or-TAGS + in-memory `is_authority_pin_content`
- **Blocks / feeds:** Two-token AND-miss pins whose TAGS/OR hits lose the Prefer `LIMIT` window to newer/higher-BM25 `TAGS:` rows. Daily `recall` / `search` / `--semantic` lexical fallback / `sync query` vault half (follow `recall_full` — **do not** edit `sync.rs`).
- **Absorbs:** `#230` Cursor Bugbot medium (still true on product `211c934` / fold-in `6c23288`). T312 F8 leftover only.
- **Not absorbed (DoD):** T326 pin-count; T217 ≥3 gate; T218 floors; `candidate_depth`; KIND bump; FTS title/body split; clap 5; H2; T324 (Completed)
- **Research date:** 2026-08-29 (plan-write product HEAD `211c934` T324 `#246`). Fold-in against `6c23288` (this plan’s own docs commit; ahead **1** of `origin/main` = `211c934`). Snapshot — **re-verify at execute**.
- **AI fold-in:** 2026-08-29 `agy-review.md` + `opencode-review.md` (HEAD `6c23288`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m2 HEAD snapshot; OpenCode m1 CAPABILITIES pass-2 OR (not AND-fill); OpenCode m2 AC6 Prefer `ORDER BY rank` / no `updated_at`; OpenCode m3 AC1 12× flood shape + Phase-1 green-today note; OpenCode O1 AC16 no behavioral PreferRecency hermetic; OpenCode O2 volatile preflight. **Already:** Agy m1 F7/AC14/§5.1; Agy m3 F35 TAGS prefix; Agy O1 F33; Agy O2 tracing optional in §5.1. **Decline as citation:** Agy `#246` `mergedAt` 22:15:07Z is list `createdAt`; live `mergedAt` **2026-08-29T22:33:35Z**. OpenCode O3 Bugbot line 251 — semantics SoT, no fold. Disposition **§13**.
- **Ledger:** planning DOCS TX `e8a70f94-0beb-4b98-bc5b-50da64bdd87a`. Fold-in DOCS TX `86f98ed2-6873-4cda-9e7e-84b86500af12`. Minted with T315 planning DOCS `ca5b1614-6849-416d-ad27-1d44a23198d7`. Implement starts a **BUGFIX** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** pin production DECISIONs to the live vault. Do **not** grow hotspot `project.rs` / `sync.rs` / `governed_common.rs` / `forget.rs` / `session_chrome.rs` (#6). Touch `lexical.rs` F8 arm + existing hermetic `recall_rank_v3.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Frozen: T312 F8/F40/F41/F42 grammar except this recency retry; T217 R0/≥3; T218 floors; `candidate_depth`.

---

## 1. Objective

1. **OR-fill has the same recency retry as AND.** When F8 Prefer-OR retain is empty, one `PreferRecency` MATCH on the **OR** expr (same GLOB-or-TAGS + `?` binds as T274 F35) before pass-2.
2. **Do not weaken F8.** Still fire only when AND + AND-recency retain is empty; ≥2 contentful tokens; authority-only retain; pass-2 uses the OR expr when that retain becomes nonempty (T312 T260 `--symbols` mix).
3. **North star.** Capture independence: FTS two-pass only. No new events. No models. No hidden CoT.

This unblocks: T312 F8 already Prefer-ORs a two-token AND-miss pin into `candidate_depth` when that pin is inside the BM25 `LIMIT` window. A vault whose `TAGS:` envelope rows (session dumps that GLOB `ASSISTANT: TAGS:*` but fail `is_authority_pin_content`) fill that window still drops the pin. AND already recency-retries that case. OR does not.

---

## 2. Live baseline (re-scan 2026-08-29)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Fold-in against plan-write `6c23288` `docs(conductor): plan T325 F8 OR-fill PreferRecency…`. Product `src/` = T324 `#246` `211c934`. Branch `track/T325-fts-or-fill-recency`. `origin/main` = `211c934` (ahead **1**). Plan-write snapshot was `211c934` / ahead **0** (Agy m2). |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. T311/T263/T285 **on PATH**. **T312–T324 not.** Ranking hole for T312 F8 is **source-only**; PATH still T285 dump-first. **Do not `cargo install`.** |
| `preflight --summary` (PATH) | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4656**. In-context **0/0/0**. `Total Word Count: 718` at plan-write (**volatile**; re-measure at execute — PATH-behind T315). Grants 3 of 3. **Not this DoD.** |
| PATH `recall "graph backend" --no-bridge --limit 3` | **#1** audit dump `431f6505-…` score **−4.162**. **#2** `## Objective` T309 plan-audit **−3.893**. **#3** `# Review of Track 253` **−1.374**. JSON `results.len()==3`. T312 not on PATH — expected dump-first. |
| Last GitHub PR | [#246](https://github.com/Ryan-AI-Studios/AI-Brains/pull/246) T324. `mergedAt` **2026-08-29T22:33:35Z**. Issue/review/inline comments **[]**. Open PRs: **none**. `#237` Bugbot already **T326**. `#230` Bugbot **this track**. **No T327 from Cursor.** |
| `#230` Bugbot | Still true: `lexical.rs` F8 `or_pass` uses `AuthorityFilter::Prefer` only (`:231–238`). AND recency is `:197–213`. PreferRecency SQL `:390–392`. |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX). Doctor hygiene warns (**volatile** count; plan-write 5 with impact-stale; fold-in 4: legacy `.changeguard`; sig-pin; sig-version; timings-0). Impact **LOW** (conductor-only at plan-write). |
| Hotspots | CLI `project.rs` **#1** (3.615). `sync.rs` **#2**. `governed_common.rs` **#3**. `forget.rs` **#5**. `session_chrome.rs` **#6** (2.550) — **do not grow**. `lexical.rs` not top 10. |
| `ISSUES.md` | **Does not exist.** |
| rustc | **1.95.0** / edition **2024** |
| Line counts (physical) | `lexical.rs` **653**; retrieval `recall_rank_v3.rs` **238**; CLI `recall_rank_v3.rs` **214**. |

### 2.2 Why this leftover still matters

| Residual | Why it is still a product hole / why decline extras |
|----------|-----------------------------------------------------|
| F8 Prefer-OR skips recency | AND: Prefer (`ORDER BY rank`) → in-memory `is_authority_pin_content` → if empty **PreferRecency** (`ORDER BY mp.updated_at DESC`). F8: Prefer-OR → retain → **stop**. `TAGS:` rows MATCH-OR either token, pass SQL GLOB-or-TAGS, fail in-memory authority, fill `LIMIT` (`candidate_depth(5)=15`), drop the OR-only pin. **DoD.** |
| T312 AC5 greens without recency | AC5 dumps are **prose** (`Here's the assessment.` — no `TAGS:` / no GLOB). Prefer-OR SQL window is the **one** tagged pin. That is F8 itself, not this hole. **Stay-green, not the red.** |
| Red fixture | **TAGS-not-authority flood** that AND-hits both tokens (high TF → better BM25 than a one-token pin) × 15, then a newest OR-only `DECISION:` pin. Prefer-OR retain empty; recency-OR admits the pin. **DoD.** |
| Raise `candidate_depth` | T312 F4 freeze. Recency retry is the T274 lever, not a bigger BM25 window. **Decline.** |
| T217 R2 | R0 nonempty (dumps AND-hit) still returns before R2. F8 lives **inside** R0 `match_query`. Do **not** change `:85–87` / `:90`. **Decline.** |
| Live vault `graph backend` | PATH dump-first is T312 install lag, not T325 proof. Source F8 without TAGS flood already fills a lone pin (AC5). T325 is the **TAGS-window** hole. **Do not live-pin.** |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| AND recency | `lexical.rs` `:197–213` | `if retain.is_empty()` → `match_query_filtered(..., PreferRecency)` → filter `is_authority_pin_content`. **Copy this onto the OR expr.** |
| F8 Prefer-OR | `lexical.rs` `:215–251` | `AuthorityFilter::Prefer` only (`:238`). `pass2_expr = or_expr` when retain nonempty (`:247–249`). **Insert recency before that assignment.** |
| PreferRecency SQL | `lexical.rs` `:389–392` | `ORDER BY mp.updated_at DESC, mp.memory_id ASC LIMIT ?`. GLOB-or-TAGS same as Prefer (`:365–376`). **Reuse. Do not new enum.** |
| Prefer SQL | `_` arm `:393–395` | `ORDER BY rank LIMIT ?`. **Do not change.** |
| `is_authority_pin_content` | `session_chrome.rs:79–84` | `Decision` \| `Constraint` (INVARIANT maps Constraint). TAGS-then-prose is Other. **Do not edit this file (hotspot #6).** |
| GLOB-or-TAGS | `authority_glob_sql` `:90` + `tags_envelope_sql` `:113` | Prefer/PreferRecency SQL window is **wider** than in-memory retain. That gap **is** the hole. |
| T217 ladder | `lexical.rs:74–146` | R0 nonempty return `:86–88`; rescue ≥3 `:91`. **Freeze.** |
| Depth | `hybrid.rs:20–22` | `limit*3` clamp **15..50**. `candidate_depth(5)==15`. **Freeze.** |
| Hermetic T312 | retrieval `tests/recall_rank_v3.rs` AC5/F40/AC14; CLI `tests/recall_rank_v3.rs` AC12 | Stay-green. **New red in retrieval file.** |
| `forget --match` | `prefer_authority: false` | No F8 / no recency. **Freeze.** |
| Contracts | `RecallResult` | **No** new keys. |
| `match_prefer_recency_sql_for_test` | `lexical.rs:414–424` | Already asserts `updated_at` + `?`. Stay-green AC. |

**Pick:** copy the AND recency block onto F8’s `or_expr`. Do **not** refactor AND+OR into a generic helper (F33). Do **not** recency-retry when Prefer-OR retain is already nonempty (F28, mirrors AND).

### 2.4 Dependency / standards research (2026-08-29) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **`4.5`** / lock **4.6.1** | crates.io **4.6.6**. **No clap 5** in `cargo search` top. | **No bump.** No new flags. |
| `rusqlite` | exact **0.40.2** / lock **0.40.2** | crates.io **0.40.2** | **No bump.** MATCH + GLOB only. |
| `serde_json` | lock **1.0.150** | — | **No bump.** JSON keys freeze. |
| `uuid` | workspace **1.13** / lock **1.23.1** | — | **No bump.** UUID only in stored bodies (F42). |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| workspace | **0.1.3** | — | **No bump** |
| New crates | — | — | **Zero.** No `regex`. |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| FTS5 `ORDER BY rank` is BM25; better = more negative; `rank` is faster than `bm25()` with LIMIT | [SQLite FTS5](https://www.sqlite.org/fts5.html) §5.1.1 / §5.2 (page last updated **2026-08-27**; fetched 2026-08-29) | Prefer stays `ORDER BY rank`. Recency is a **different** ORDER BY on the same MATCH, not a BM25 retune. |
| Implicit AND of whitespace tokens | Same FTS5 grammar | `"t325or backend"` requires both tokens — pin lacking `backend` is AND-miss / OR-hit (F42). |
| Filter-then-score; selective WHERE after BM25 top-K | [pg_textsearch 1.4](https://github.com/timescale/pg_textsearch/releases/tag/v1.4.0) (2026-08-18): highly selective filters used to force repeated BM25 passes over ever-larger candidate sets — they now size top-K from selectivity. ES `function_score` filter+weight (T285 class). | SQLite edition: MATCH + GLOB window is the top-K; in-memory authority is the selective filter; empty → **second query** with recency ORDER BY, same LIMIT. Do **not** raise `candidate_depth` (that is “ever-larger candidate set”). |
| Recency as a ranking axis when relevance window misses | SQLite docs (sqldocs / FTS tutorials): `ORDER BY date DESC` is a supported alternative to `ORDER BY rank` | T274 already chose recency-retry, not a blended `rank, updated_at` sort (would still fill with high-BM25 TAGS dumps). |
| Title-weighted BM25 | Engram / FTS5 `bm25(table, w…)` needs per-column weights | One `content` column. **Decline** schema split (T312 F4). |

**N/A:** SQLCipher page encrypt, schtasks, Windows service, clap 5 (not this bump), T180 new required keys, llama.cpp `/health`, T307 reqwest/tower-http, T326 pin-count.

**Could not verify:** COUNT of in-scope `TAGS:` rows MATCH-ORing `graph` without vault SQL (do not print `AI_BRAINS_KEY`). Hermetic TAGS flood is the proof, not live archaeology.

**ledgerful / ai-brains:** `preflight --summary` 0/0/0 vs **4656** pins; PATH recall still dump-first; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "match_query"` → `lexical.rs:167`; `PreferRecency` → `:151` / `:205` / `:390`; `is_authority_pin_content` → `session_chrome.rs:79` + `lexical.rs:192/:209/:242`; `scan --impact` LOW (dirty conductor); hotspots `session_chrome.rs` #6 — **do not grow**. Ledger search `"authority-OR fill"` → T312 plan DOCS `8b1b418b` + FEATURE `7f7e99bb`. Semantic recall of this leftover is T312 plan-audit chatter, not a DECISION pin.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is this DOCS TX. Implement starts a **BUGFIX** TX. |
| **F1 — OR recency retry** | Inside `match_query` F8 arm: after Prefer-OR + in-memory retain, **if retain is empty**, one `match_query_filtered(..., or_expr, AuthorityFilter::PreferRecency)` + the same `is_authority_pin_content` filter. Then the existing `if retain.len() >= limit { return }` / `if !retain.is_empty() { pass2_expr = or_expr }`. |
| **F2 — Reuse PreferRecency** | No new `AuthorityFilter` variant. No new SQL builder. Reuse `:390–392` and `match_prefer_recency_sql_for_test`. |
| **F3 — F8/F40 gate freeze** | F8 still runs only when AND + AND-recency retain is empty. Do **not** OR-fill when AND-retain already has ≥1 authority pin (T312 F40). |
| **F4 — T217 freeze** | R0 nonempty still returns. Rescue still requires tokens ≥ 3. `forget --match` stays `rescue: false` + `prefer_authority: false`. Do **not** delete R2. |
| **F5 — KIND / depth / floors freeze** | Do **not** change `KIND_*`, `SESSION_CHROME_PENALTY`, `DUMP_OTHER_*`, `LEADING_QUERY_BONUS`, `candidate_depth` 15..50, T218 0.55/0.60. |
| **F6 — F42 needle grammar freeze** | Hermetic queries that prove **F8 / this recency** are **exactly 2 contentful tokens**. Pin **lacks ≥1**. UUID **only in stored bodies**. |
| **F7 — pass-2 OR when F8 filled** | When Prefer-OR **or** recency-OR retain is nonempty, `pass2_expr = or_expr` (T312 T260 `--symbols` mix). When both empty, pass-2 stays AND. |
| **F8 — T312 AC5/AC12 stay-green** | Prose-dump AND-miss (no TAGS flood) still pin #1 via Prefer-OR alone. |
| **F9 — forget unfiltered** | `forget --match` does not take envelope GLOB / two-pass / OR fill / recency. |
| **F10 — No new CLI flag** | No `--recency` / `--pins-only`. |
| **F11 — No DTO keys** | No `is_session` / `pin_kind` / `retry` on `RecallResult`. |
| **F12 — File growth** | Production edit is **`lexical.rs` F8 arm only**. Tests in existing retrieval `recall_rank_v3.rs` (+ optional unit next to `authority_or_sql__…ac15`). **Do not** edit `session_chrome.rs`, `ranking.rs`, `hybrid.rs`, `recall.rs`, `pin.rs`, CLI `preflight.rs`, `sync.rs`, `project.rs`, `forget.rs` production, `.github/workflows/ci.yml`. CLI `recall_rank_v3.rs` **stay-green** (no required new CLI hermetic). |
| **F13 — Capture independence** | Ranking/retrieval only. No models on default FTS. No new events. |
| **F14 — No unwrap/expect/panic** | Production. Existing `?` on `match_query_filtered`. |
| **F15 — Test names** | `function_or_feature__condition__expected_result`. |
| **F16 — Implement TX is BUGFIX** | Planning is DOCS. |
| **F17 — Debt file** | `conductor/ISSUES.md` does **not** exist. |
| **F18 — PowerShell** | `;` not `&&`. |
| **F19 — last-PR Cursor** | `#246` empty → **N/A**. `#237` → **T326**. `#230` → **this**. **No T327.** |
| **F20 — Decline peers** | T326; T307 Blocked; clap 5; H2; T240 F2; depth raise; FTS schema split; KIND bump; pretty score = effective; Index SQL (T286). |
| **F21 — PATH** | Do not `cargo install`. Hermetic/`cargo run` SoT. PATH dump-first is T312 lag, not Complete-blocking. |
| **F22 — Live vault pin** | Do **not** pin production DECISIONs. Hermetic unique needle is SoT. |
| **F23 — 80-net** | Recency block + one retrieval hermetic + optional SQL unit. Do **not** grow `main.rs` test blocks. |
| **F24 — T312 F40 stay-green** | AND-retain nonempty still skips OR (and therefore skips OR-recency). |
| **F25 — Docs** | CAPABILITIES pin-type row: **correct** the stale “then pass-2 AND fill” (live `lexical.rs:247–249` sets `pass2_expr = or_expr` when OR-fill retain is nonempty; AND only when both fills are empty) **and** add “OR-fill recency-retries like AND”. CHANGELOG Unreleased. PROTOCOL-COMPAT: no new required keys. |
| **F26 — Do not raise LIMIT** | Recency uses the **same** `limit` (`candidate_depth`). 15 newer TAGS-not-authority after the pin still miss — T274 same limitation; residual §11. |
| **F27 — Prefer stays BM25** | Do **not** change Prefer `ORDER BY rank` to a blend. Recency is the retry, not a sort tweak. |
| **F28 — Recency only when empty** | Do **not** recency-retry when Prefer-OR retain already has ≥1 authority pin (mirrors AND: recency is empty-only, not partial-fill). |
| **F29 — T274 F35 binds** | Recency-OR Prefer has **no** `NOT IN`. Pass-2 still `bound_not_in_sql` + `?` only. **Forbidden** to `format!` UUID strings into SQL. |
| **F30 — No clap 5 / no rusqlite bump** | Standing. |
| **F31 — T217 R2 stays** | Idempotent with F8 on prefer path (T312 F9). Do not “fix” double-fire. |
| **F32 — Isolation** | No `.env` rewrite; no live `nightly` without `--status`; no daemon stop. |
| **F33 — Copy-not-share the retry** | Copy the AND recency block onto `or_expr`. Do **not** extract a generic `retry_until_retain` this track (scope creep on hotspot-adjacent retrieval). |
| **F34 — ≥2 contentful tokens** | T312 F41 freeze. `"t325or backend"` is the red query. |
| **F35 — Red fixture is TAGS flood** | Floods **must** GLOB `ASSISTANT: TAGS:*` (or `TAGS:*`) and **fail** `is_authority_pin_content` (prose after TAGS). Body shape is T312 AC4 analog: fixed **`12× `"t325or backend "`** repeats, similar length; pin body **short** (TAGS + `DECISION: t325or {uuid} sqlite graph` — no extra pad). Prose-only dumps (AC5) do **not** prove this hole. If AC1 is **green-today**, diagnose BM25 window (strengthen repeats) first — not “hole absent”. |
| **F36 — Pin is newest** | Append 15 TAGS floods **first**, authority pin **last** so `ORDER BY updated_at DESC` includes it. No `sleep`. |
| **F37 — Same-tick honesty** | PreferRecency secondary key is `memory_id ASC`. Do not rely on UUID order. Sequential `append_event` is the fixture. If go finds a same-tick flake, **Stop-Before** — no sleep; do not raise LIMIT; re-check projector `updated_at` granularity. Windows `SystemTime` 100 ns FILETIME (T322 analog). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Retrieval hermetic `match_query__or_fill_tags_flood__recency_retry_pin_first` (F6/F35/F36): query **`"t325or backend"`** (exactly 2 contentful tokens; **no** UUID in the query); 15 floods `ASSISTANT: TAGS: t325\nHere's the assessment. dump {i}\n` + **`format!("t325or backend ").repeat(12)`** (both tokens, **not** DECISION/CONSTRAINT; AC4 analog); then pin `ASSISTANT: TAGS: t325\nDECISION: t325or {uuid} sqlite graph` (**no** `backend`; uuid in the **body**; **short** — no pad). `recall_full` `--limit 5` hit **#1** is the pin. **Required red** (today Prefer-OR retain empty → pass-2 AND dumps). If green-today on the red commit, **tune the 12× flood first** — do not conclude the hole is absent. |
| **AC2** | T312 `match_query__and_retain_empty__authority_or_fills_pin__ac5` **stay-green** (prose dumps, no TAGS flood). |
| **AC3** | T312 `match_query__and_retain_nonempty__no_or_distractor__f40` **stay-green**. |
| **AC4** | CLI `tests/recall_rank_v3.rs` AC12/AC13 **stay-green**. No required new CLI hermetic. |
| **AC5** | Existing `match_sql__pass1_glob_limit__pass2_bound_not_in` recency-retry asserts `updated_at` + `?` + `TAGS:` **stay-green**. |
| **AC6** | Extend existing `authority_or_sql__or_glob_tags_limit_placeholders__ac15`: Prefer SQL (`match_pass_sql_for_test(true, 0)`) **`contains("ORDER BY rank")`** and **`!contains("updated_at")`**. Today that unit asserts GLOB/TAGS/LIMIT/`?` only — without these two lines it **false-passes** a Prefer-arm recency bleed (F27). Guard, not Phase-1 red. F12 already permits this unit. |
| **AC7** | T217 rescue ≥3 units **stay-green**. R0 nonempty still skips R2. |
| **AC8** | T312 `forget_match__still_finds_verbose_other_dump__ac10` **stay-green** (unfiltered). Not required red. Do **not** add a second forget hermetic. |
| **AC9** | Compact recall JSON: no new required keys; `content` still includes `ASSISTANT:` / `TAGS:` raw. |
| **AC10** | `cargo clippy -p ai-brains-retrieval --all-targets -- -D warnings`; nextest `-p ai-brains-retrieval` + CLI `recall_rank_v3` + T217 units. |
| **AC11** | Docs: CAPABILITIES pin-type row **replaces** “then pass-2 AND fill” with “pass-2 uses the OR expr once OR-fill (or its recency retry) retains; AND only when both fills are empty” **and** names OR-fill **recency-retry**. CHANGELOG Unreleased. PROTOCOL-COMPAT unchanged (no new keys). |
| **AC12** | T312 AC14 `--semantic` F42 pin in top-3 **stay-green**. |
| **AC13** | T312 AC16 freeze unit (`KIND_DECISION == 2.0`, `candidate_depth(5) == 15`, T218 floors) **stay-green**. |
| **AC14** | When recency-OR fills, pass-2 still uses OR (F7). Covered by AC1 remainder + existing T312 pass-2 OR assignment moving **after** recency. No separate SQL-log AC. |
| **AC15** | **Manual:** PATH `recall "graph backend"` may still dump-first (T312 not installed) — honesty, not fail. Source SoT is AC1 nextest. **No** live pin. **No** `cargo install`. |
| **AC16** | AND recency path (`:197–213`) **untouched**. **No behavioral PreferRecency hermetic exists** (grep: only `lexical.rs` production + SQL-string unit `:580–620`). Stay-green set is that SQL unit + AC2/AC3 family. Do not hunt for a missing AND-recency recall hermetic. |

---

## 5. Design notes

### 5.1 Insertion point (F8 arm)

Today (`lexical.rs:231–249`):

```rust
let or_pass = match_query_filtered(..., AuthorityFilter::Prefer)?;
retain = or_pass.into_iter().filter(|m| is_authority_pin_content(&m.content)).collect();
if retain.len() >= limit { return Ok(retain); }
if !retain.is_empty() { pass2_expr = or_expr; }
```

On go, between the `>= limit` return and the `pass2_expr` assignment:

```rust
if retain.is_empty() {
    tracing::debug!(stage = "prefer_or_recency", "FTS authority-OR recency retry after empty Prefer-OR retain");
    let retry = match_query_filtered(
        conn, &or_expr, project_id, session_id, limit,
        exclude_symbol_stubs, AuthorityFilter::PreferRecency,
    )?;
    retain = retry
        .into_iter()
        .filter(|m| crate::session_chrome::is_authority_pin_content(&m.content))
        .collect();
    if retain.len() >= limit {
        return Ok(retain);
    }
}
if !retain.is_empty() {
    pass2_expr = or_expr;
}
```

AND recency (`:197–213`) stays as-is. `or_expr` is already in scope inside the `if !or_expr.is_empty()` block.

### 5.2 Why TAGS floods beat Prefer-OR

SQL Prefer/PreferRecency window = authority GLOB **OR** `TAGS:` envelope. In-memory retain = `Decision`/`Constraint` only (`first_contentful_line` skips a leading `TAGS:` line — `ranking.rs:102–113`). Session rows `ASSISTANT: TAGS: t325\nHere's the assessment. …` enter the SQL window, fail retain, occupy `LIMIT 15`. A short one-token pin can lose `ORDER BY rank` to dual-token floods **if** those floods have enough TF; FTS5 `b=0.75` length-normalizes, so **long** floods can score *below* a short pin. AC1 therefore pins **`12× "t325or backend "`** (T312 AC4 analog) and a **short** pin body. If that still greens today, strengthen repeats — do not raise `candidate_depth`. `ORDER BY updated_at DESC` with the pin appended last puts it in the recency window.

### 5.3 Why not blend `ORDER BY rank, updated_at`

High-TF TAGS dumps still lead BM25; a blended sort does not evict them. T274 already rejected that for AND. Copy the retry.

### 5.4 Why AC5 is not the red

AC5 dumps lack `TAGS:` / authority GLOB, so Prefer-OR SQL returns **only** the pin. Recency never needs to run. A green AC5 on today’s tree **does not** prove T325.

---

## 6. Non-goals

- Raising `candidate_depth` / KIND / T218 floors / `DUMP_OTHER_CHAR_FLOOR`
- T217 R0/≥3 gate change / deleting R2
- FTS title/body schema split / `bm25()` column weights
- Blended `ORDER BY rank, updated_at` on Prefer
- New CLI flags / DTO keys / pretty composite score
- Editing `session_chrome.rs` / `ranking.rs` / `hybrid.rs` / `sync.rs` / `project.rs`
- T326 `PinnedCountFailed` / T307 tower-http / clap 5 / H2
- Live vault pin / `cargo install`
- Fixing every MATCH family (`forget`, unfiltered search)
- Recency-retry when Prefer-OR retain is already nonempty (partial fill)

---

## 7. Verification plan (TDD)

**Red first (must fail on missing OR-recency):**

- `match_query__or_fill_tags_flood__recency_retry_pin_first` — today: hit #1 is a TAGS dump / not the pin → **fail**. Fixture = F35 12× repeats. **If green-today:** tune TF (more repeats), do not treat as hole-absent.
- Stay-green T312 AC5 / F40 / CLI AC12 still **pass** on this red commit
- AC6 Prefer `ORDER BY rank` / `!updated_at` may be added on green (guard, not required red)

**Green:** F1 recency block + F7 `pass2_expr` after both Prefer-OR and recency-OR.

**Docs:** AC11.

**Manual:** AC15 honesty only.

Do **not** require full workspace nextest to finish the **plan**.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| AC5 false-pass as T325 proof | F35 / §5.4 — red is TAGS flood, not prose dumps |
| Same-tick `updated_at` | F36/F37 — pin last; no sleep; Stop-Before if flake |
| Recency on nonempty Prefer-OR displaces AND pin | F28 / AC3 F40 |
| pass-2 AND after recency-OR fill drops T260 mix | F7 — move `pass2_expr = or_expr` to after recency |
| Raising LIMIT “to be safe” | F26 freeze |
| Growing `session_chrome.rs` | F12 — filter already exists |
| PATH dump-first treated as fail | F21 / AC15 honesty |
| UUID in query greens T217 | F6 / `"t325or backend"` |
| Clippy `too_many_arguments` | Already `#[allow]` on `match_query` |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-29 (T324 implement residuals through T142). Overlapping **open** rows:

| Item | Disposition |
|------|-------------|
| `#230` Bugbot F8 Prefer-OR skips PreferRecency | **Absorb** F1–F7 / AC1 / AC14 |
| T312 F8/F40/F41/F42 grammar | **Affirm freeze** F3/F6/F8/F24/F34 |
| T312 implement R3 live dump-first until OR-matching pin | **Partial** — F8 still the fill; this track is the TAGS-window retry; live PATH dump-first is F21 |
| T316 F27 T287 R1-1 GLOB+retain empty → recency first page (memory **list**) | **Not stolen** — list ORDER, not FTS MATCH |
| T324 implement residuals (other positionals / `--as-of ""` / PATH) | **Not stolen** — T324 Completed `#246`; absorb dirty conductor Completed note into this DOCS commit |
| T326 `PinnedCountFailed` fake `pinned=0` (`#237`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| T217 ≥3 / T218 floors / KIND / depth | **Decline** F4/F5 |
| last-PR Cursor `#246` | **N/A empty** (no defect) |
| last-PR `#237` / `#230` | **T326** / **this** — **no T327** |
| `ISSUES.md` | **Does not exist** |
| T324 uncommitted conductor Completed note | **Plan-write DOCS commit** |

---

## 10. Implement order (on go)

1. Phase 0: re-read `match_query` AND recency `:197–213` vs F8 `:215–251`; PreferRecency SQL `:390–392`; T312 AC5/F40; lock rusqlite **0.40.2**; T326 still Pending; BUGFIX TX. **Do not install.** **Do not** live-pin.
2. Red AC1 (must fail; F35 12× repeats). If green-today, strengthen repeats before concluding the hole is absent. Confirm AC2/AC3/AC4 still pass.
3. Green F1 recency on `or_expr` + F7 `pass2_expr` after both fills. AC6 Prefer `ORDER BY rank` / `!updated_at`. AC11 CAPABILITIES pass-2 **OR** wording.
4. Stay-green AC5–AC9 / AC12–AC14 / AC16.
5. Docs AC11.
6. Targeted clippy/nextest AC10. Implement-track full gate before publish (never `git push origin main`).

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| 15 **newer** TAGS-not-authority after the pin still miss | F26 — same T274 LIMIT; do not raise `candidate_depth` |
| PATH until owner `cargo install` (T312–T324) | F21 — hermetic/`cargo run` SoT |
| Live `graph backend` dump-first on PATH | T312 not installed; T325 does not live-pin a canary |
| Same-tick FILETIME | F37 Stop-Before; no sleep |
| T287 list recency vs GLOB empty | T316 residual — not FTS MATCH |
| T326 pin-count | Placeholder until `/plan-track T326` |
| Pretty `score=` still raw BM25 | T312 F38 freeze |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/lexical.rs` | F8 arm: PreferRecency on `or_expr` when Prefer-OR retain empty; `pass2_expr` after both. Optional debug span `prefer_or_recency`. |
| `crates/ai-brains-retrieval/tests/recall_rank_v3.rs` | AC1 TAGS-flood hermetic |
| `Docs/CAPABILITIES.md` | Pin-type row: OR-fill recency-retries like AND |
| `CHANGELOG.md` | Unreleased |
| `conductor/conductor.md` | Planned → (on go) Completed |
| `conductor/deferred.md` | this plan section |

**Do not touch:** `session_chrome.rs`, `ranking.rs`, `hybrid.rs`, `recall.rs`, `pin.rs`, CLI `preflight.rs`, `sync.rs`, `project.rs`, `forget.rs` production, `governed_common.rs`, daemon, contracts, `.github/workflows/ci.yml`. CLI `tests/recall_rank_v3.rs` stay-green only.

---

## 13. AI fold-in disposition (2026-08-29)

Source: `agy-review.md` + `opencode-review.md` (HEAD `6c23288`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.**

### Agy

| ID | Verdict | Action |
|----|---------|--------|
| **m1** pass-2 assignment after recency | **Already** | F7 / AC14 / §5.1 snippet (assignment **after** both Prefer-OR and PreferRecency) |
| **m2** HEAD `211c934` vs `6c23288` | **Agree** | Snapshot `6c23288` / ahead **1** of `origin/main` `211c934` |
| **m3** AC1 floods need `ASSISTANT: TAGS:` | **Already** | F35 / AC1 prefix (OpenCode m3 tightens **repeats**, not the prefix) |
| **O1** copy AND recency | **Already** | F33 |
| **O2** `prefer_or_recency` tracing | **Already** | §5.1 optional `tracing::debug!(stage = "prefer_or_recency")` |
| `#246` `mergedAt` 22:15:07Z | **Decline** as citation | Live `mergedAt` **2026-08-29T22:33:35Z** (`gh pr view --json mergedAt`); 22:15:07Z is list `createdAt` |

### OpenCode

| ID | Verdict | Action |
|----|---------|--------|
| **m1** CAPABILITIES `:306` “pass-2 AND fill” | **Agree** | F25 / AC11 — correct to OR-when-retain, AND-when-both-empty, then add recency |
| **m2** AC6 unit never asserts `ORDER BY rank` / no `updated_at` | **Agree** | AC6 — extend `:624` unit (verified: asserts GLOB/TAGS/LIMIT/`?` only) |
| **m3** AC1 BM25 flood shape underspecified | **Agree** | F35 / AC1 / §5.2 / §7 — `12× "t325or backend "` (AC4 analog) + short pin; green-today → tune TF |
| **O1** no behavioral PreferRecency hermetic | **Agree** | AC16 — grep confirmed 5 hits, all `lexical.rs` production/SQL unit |
| **O2** volatile preflight/doctor | **Agree** | §2.1 **volatile** tokens |
| **O3** Bugbot line 251 vs `:231–249` | **Decline** as required | Phase 0 already checks Prefer-only semantics; comment body is SoT |

### Pins locked by fold-in

1. **AC11:** CAPABILITIES pass-2 clause is **OR expr once retain nonempty**; do not add recency onto a row that still says AND-fill.
2. **AC6:** Prefer SQL must assert `ORDER BY rank` and `!updated_at` (F27 regression vector).
3. **AC1 flood:** `ASSISTANT: TAGS: t325` + **`12× "t325or backend "`** + short pin last. Green-today = fixture TF, not hole-absent.
4. **AC16:** no AND-recency recall hermetic exists; SQL unit + AC2/AC3.
5. **last-PR:** `#246` N/A empty; `#237` → T326; `#230` → this; **no T327**.

Plan-write HEAD `211c934`. Fold-in against `6c23288` (ahead **1**). Still **plan-only until go**.
