# T286 — Preflight Index/summary must not look like an empty brain

- **Track ID:** T286-PreflightIndexPins
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / UX / RETRIEVAL
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `--pretty` **7/5** Index `1. ## Objective -- just now`; `--summary` **8/7** in-context decisions **0** / 3647 pins. Placeholder minted with T285–T300 (`76c4db9`). T285 Completed `#201` `16ee1aa` (Index **not** DoD there — F13).
- **Depends on:** T274 ✅ Index two-pass (live still Objective); T285 ✅ envelope + TAGS-or-GLOB **on recall** (not Index); T279 ✅ Safety (do **not** reopen); T250 ✅ density; T214 ✅ dual counts; T220 ✅ summary JSON keys; T265 ✅ `sections[]`; T272 ✅ Safety skip-set
- **Blocks / feeds:** Agents starting a session with `preflight --pretty` / `--summary` see a pin, not a review-track dump. Briefing granted-empty remains **T288**. `memory list` ORDER **T287**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “preflight Index `## Objective`; summary decisions 0 vs 3647 pins”; T285 F13 / closeout “Index/summary still Objective”; T274 AC6/AC7 as **regression** (untagged stays); live TAGS envelope hole on Index SQL
- **Not absorbed (DoD):** T287 list ORDER; T288 briefing stanza; T293 neighbors CLI; T279 Safety SQL; T263 H2 pin→Approved; T240 F2; T214 9-arg formatters; T220 new keys; T180 required-key growth; clap 5 / rusqlite 0.40; Session-section reorder
- **Research date:** 2026-08-23 (plan dogfood HEAD `16ee1aa` T285 `#201`; product `src/` = rank v2; PATH **0.1.2** 2026-08-22 19:41 **without** T285)
- **AI fold-in:** none yet (plan pass). last-PR Cursor **#201** comments/reviews/inline **empty** → N/A. **No T301.**
- **Ledger:** planning DOCS TX `397f9c55-5953-402b-95fc-db431f5a037c`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement (hermetic needle is SoT; Manual DoD unique canary is allowed on go). Do **not** rewrite `.env`. Do **not** grow hotspot `project.rs` / `sync.rs` / CLI `preflight.rs` **production** (tests OK). Do **not** edit T279 Safety SQL. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Index item 1 is a pin.** `preflight --pretty` **Memory Index** line `1.` is a leading-line `DECISION:` / `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` pin (after the T285 envelope) when the scoped vault has at least one — **not** `## Objective`.
2. **Summary does not imply an empty brain.** `--summary` still shows vault `Pinned memories: N` (T214). `In context decisions` is ≥ **1** when that pin is in the budget window (existing substring scan of assembled text). Do **not** add a new JSON key for “vault authority.”
3. **Keep T274/T279/T220 contracts.** Untagged `DECISION:` Index still wins (T274 AC6). Safety skip-set stays T272/T279. Summary JSON keys stay T220. `sections[]` ids stay T265. Recent stays recency.
4. **North star.** Capture independence: Index selection + display titles only. No new events. No hidden CoT. Agents who run `preflight --summary` must not conclude this repo has no decisions because a chrome window counted `DECISION:` **0** next to 3k pins.

This unblocks the daily product: T274 two-pass is on and still loses because Index pass-1 GLOB misses `ASSISTANT: TAGS: …\nDECISION:`. T285 fixed that for `recall`; Index still uses `index_marker_glob_sql` only. The renderer uses `content.lines().next()`, so even a tagged pin that leaked into pass-2 would title as `TAGS:`.

---

## 2. Live baseline (re-scan 2026-08-23)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `16ee1aa` T285 squash `#201`. Tree **CLEAN**. `origin/main` = HEAD. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. **Has T274. Does not have T285.** Index hole is in **source + PATH**. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3716**. In-context hotspots **5** / decisions **0** / constraints **0**. Word count **255**. Grants omitted (3 of 3). Hole stands: 3k pins vs 0 decisions. |
| `preflight --pretty -m 1500` | Safety = live `HOTSPOT: crates/…` (T279 OK). Sessions = review-track / onboarding dumps (`## Objective` in body). **Memory Index line 1: `## Objective -- just now`.** Budget often cuts after item 1. |
| `preflight --pretty -m 400` | Safety + Sessions only — Index truncated by word budget. Manual DoD uses `-m 1500` (CAPABILITIES default) so Index is visible. |
| Last GitHub PR | [#201](https://github.com/Ryan-AI-Studios/AI-Brains/pull/201) T285 (2026-08-23). `gh pr view --comments`, `/reviews`, `/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081 unreachable). **0 pending / 0 drift** at scan. Hotspot **#1** `project.rs` (3.995) — **do not touch.** `sync.rs` #2. `session_chrome.rs` **#6** (2.397) — **extend here.** CLI `preflight.rs` **#8** — **do not grow production.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why Index still leads with `## Objective`

| Layer | Truth |
|-------|--------|
| T274 two-pass is on | `preflight.rs` `:458–492`: pass-1 `index_marker_glob_sql` then pass-2 recency `NOT IN`. Hermetic `preflight_index_pin_rank` **green** (untagged `DECISION:`). |
| Pass-1 GLOB is prefix-only | `index_marker_glob_sql` = DECISION/CONSTRAINT/INVARIANT/HOTSPOT and `ASSISTANT: <marker>` (`session_chrome.rs` `:90–101`). **No** `TAGS:` / `ASSISTANT: TAGS:`. Live `pin --tag` stores `ASSISTANT: TAGS: …\nDECISION:` (`pin.rs` `:53–57` + default assistant role). GLOB **miss**. Pass-1 empty → pass-2 newest ingest = `## Objective`. |
| `drain_index_pass` retain | `authority_only && classify_pin_kind == Other` skip (`:682`). After T285, `classify_pin_kind` **is** envelope-aware — but it never sees tagged pins because they never enter pass-1 SQL. |
| Index title | `:538` `content.lines().next()` — raw first line. Tagged pin would display `ASSISTANT: TAGS: …` (pretty then strips role → `TAGS:`). Summary `text.matches("DECISION:")` stays **0**. |
| T285 did not steal this | Spec F13 / F29: do not edit retrieval `preflight.rs` Index SQL. `tags_envelope_sql` exists and is used by **lexical** Prefer only (`lexical.rs` `:318–329`). |
| Summary dual-count | T214: vault `Pinned memories` is SQL COUNT (already **3716**). In-context is a **window substring**. Placeholder option (b) “show Pinned: N next to 0” is **already true** and still trains agents that there are no decisions. Fix is **get a pin into the window** (option a without a new key). |
| Session section | Recency of **active** sessions (T214/T250). Still chrome. **Decline as DoD** — Index is the decision list; Session is current work. |
| Safety | Live hotspots. **T279 freeze.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Index two-pass | `retrieval/src/preflight.rs` `:458–492` | `index_select_sql` + `drain_index_pass`. Pass-1 extra = `index_marker_glob_sql("m.content")`. **Change extra to `index_pass1_glob_sql`.** |
| Index title | `preflight.rs` `:536–545` | `content.lines().next()`. **Use `first_contentful_line`.** `truncate_index_summary` 60-char Unicode (`:1002`) stays. |
| Drain retain | `preflight.rs` `:682` | `classify_pin_kind != Other` includes **Hotspot** (T274 F11). **Do not** switch to `is_authority_pin_content` (that drops Hotspot). |
| GLOB helpers | `session_chrome.rs` | `index_marker_glob_sql` (marker+HOTSPOT); `tags_envelope_sql`; `authority_glob_sql`; `safety_marker_glob_sql` (T279). **Add** `index_pass1_glob_sql` = marker **OR** tags (lexical Prefer join shape). **Keep** `index_marker_glob_sql` as the marker-only helper. |
| Envelope | `ranking.rs` `first_contentful_line` `:102` | Public. Retrieval Index **reuses** — do not fork. |
| Summary counts | CLI `preflight.rs` `:884–888` | `text.matches("DECISION:")` on assembled window. **Do not change** if Index emits `DECISION:` on line 1. 9-arg formatters **frozen** (T214). |
| Pretty | CLI `preflight.rs` + `preflight_pretty.rs` | T219 role-strip; T250 Index **not** line-capped; T264 tag upgrade. Display-only. **Do not grow.** |
| T220 JSON | `PreflightSummaryJson` | Keys frozen. `in_context_decisions` stays. |
| T265 | `sections[]` | Split from `text`. Index **content** changes; `id` `index` frozen. |
| T272 skip | `safety_ids` into `drain_index_pass` | Emitted Safety ids. **Do not retune.** |
| T279 Safety | `preflight_safety.rs` + `safety_marker_glob_sql` | **Do not edit.** |
| Recent | `preflight.rs` `:494–511` | Recency-only. **Do not retitle as Index.** |
| Hermetic T274 | `retrieval/tests/preflight_index_pin_rank.rs` | Untagged pin vs Objective dump. **Stay green.** |
| CLI summary AC5 | `cli/tests/preflight_summary_json.rs` | Untagged `DECISION:` → `in_context_decisions >= 1`. **Stay green.** Add tagged twin. |
| `project.rs` | hotspot **#1** | **Do not touch.** |
| CLI `preflight.rs` | hotspot **#8** | Production **do not grow.** Tests may be added. |

### 2.4 Dependency / standards research (2026-08-23) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). GitHub clap-rs **v4.6.6 latest**. **No clap 5.** | **No bump.** No new flags. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** JSON keys frozen. |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot #62) | **No bump.** `relative_timestamp` stays. |
| `rusqlite` | lock **0.39.0** | crates.io **0.40.2** (Dependabot #61) | **No bump.** Projection GLOB only. |
| rustc / edition | **1.95.0** / **2024** | endoflife.date: 1.98 current (2026-08-20); 1.95 ended — **do not bump rustc this track** | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.2** | — | **No bump** |
| New crates | — | — | **Zero.** No `regex` in retrieval (T211 F18). |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Index is **not** FTS MATCH | Live `index_select_sql` on `memory_projection` `ORDER BY updated_at DESC` | SQLite FTS5 `bm25()` column weights / prefix indexes are **N/A** for this SQL. Do not add MATCH to Index. |
| Metadata filter before rank | Hybrid RAG + sqlite-vec (2026-05) pre-filter CTE; T285 lexical Prefer | Pass-1 GLOB-or-TAGS then in-memory `classify_pin_kind` is the projection edition of T285 F7. |
| GLOB is case-sensitive | [SQLite GLOB](https://www.sqlite.org/lang_expr.html); T274 F8 / T285 F9 | Envelope + classify remain SoT for lowercase `decision:`. SQL GLOB stays a subset. |
| Dual count honesty | [clig.dev](https://clig.dev/) machine vs human; T214 F4 | Vault COUNT and window scan stay two blocks. Do not relabel `In context decisions` as vault totals. Do not add a third count key. |
| T220 keys | CAPABILITIES summary JSON row (live) | `in_context_decisions` stays; semantics “substring of assembled window.” After Index emits `DECISION:`, the existing scan works. |
| Bind `NOT IN` | T274 F35; live `bound_not_in_sql` | Pass-2 ids stay `?` placeholders. |

**N/A:** SQLCipher page encrypt, schtasks, Windows service, llama.cpp `/health`, FTS5 `bm25(title, body)` (Index is not FTS), clap 5 (not released).

**Could not verify:** exact COUNT of post-envelope leading-marker rows in `3581317d` without vault SQL (do not print `AI_BRAINS_KEY`). Hermetic unique needle + live Manual canary are the proof.

**ledgerful / ai-brains:** `preflight --summary` 5/0/0 vs **3716** pins; `--pretty -m 1500` Index `1. ## Objective -- just now`; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "index_marker_glob_sql"` → `session_chrome.rs:90` + `preflight.rs:462`; `search "truncate_index_summary"` → `preflight.rs:1002`; `scan --impact` CLEAN at `16ee1aa`; hotspots `project.rs` #1 / `session_chrome.rs` #6 / CLI `preflight.rs` #8. Semantic recall of this topic still returns onboarding chrome (T285 PATH-behind) — evidence of ranking, not SoT for Index.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `397f9c55`. Implement starts a **FEATURE** TX. |
| **F1 — Index item 1 is authority** | When the scoped vault has ≥1 injectable leading-marker pin (Decision / Constraint / Hotspot after envelope), Memory Index line `1.` is that class of pin — not session chrome. Chrome dumps may still fill **later** Index slots (pass-2 recency). |
| **F2 — Pass-1 GLOB-or-TAGS** | Index pass-1 SQL extra is **`index_pass1_glob_sql(column)`** = existing `index_marker_glob_sql` **OR** `tags_envelope_sql` (same inner-join shape as lexical Prefer `:318–329`). `index_marker_glob_sql` stays marker+HOTSPOT only (T274 F11). Do **not** concatenate the two `AND (…)` clauses (that would require TAGS **and** marker). |
| **F3 — Retain is classify ≠ Other** | `drain_index_pass` `authority_only` keeps `classify_pin_kind != Other` (Hotspot stays). **Forbidden** to switch to `is_authority_pin_content` (drops Hotspot from Index). Envelope classify is already T285. |
| **F4 — Envelope titles** | Index numbered line uses `first_contentful_line(content)` then `truncate_index_summary`. Empty / role-only / TAGS-only → **`Untitled Memory`**. **No** `unwrap`/`expect`/`panic`. Stored `content` in JSON `text` for Recent/session stays raw; only the Index **title** line is envelope-stripped (so `matches("DECISION:")` sees the marker). |
| **F5 — T274 untagged stands** | `preflight__index_prefers_leading_decision_over_objective_dump` stays green. Do not weaken it into TAGS-only. |
| **F6 — T272 skip-set freeze** | Index/Recent skip = **emitted** Safety ids. Do not skip the whole fetch window. Do not retune `GLOBAL_*` caps. |
| **F7 — T279 Safety freeze** | Do not edit `safety_marker_glob_sql` / `preflight_safety.rs` / live hotspot fetch. Pretty Safety must still match `safety sync --dry-run` paths (hotspots), not `## Objective`. |
| **F8 — T250 / T219 caps freeze** | Index item cap **15** (compact **5**). Index **not** line-capped on default pretty. Role-strip on pretty stays. |
| **F9 — T264 round-robin freeze** | `--global` Index still `take_round_robin` per-project 3 / vault 15. Tags `[8hex]` stay on **item first lines** (after title strip). |
| **F10 — T220 / T214 counts freeze** | No new summary keys. No third “vault decisions” line. `Pinned memories` stays SQL. `In context decisions` stays case-sensitive `DECISION:` substring of assembled window. 9-arg `format_preflight_summary_lines` / `build_preflight_summary_json` **arity frozen**. |
| **F11 — T180 / T265 freeze** | Compact `{text, word_count}` required; `sections[]` additive. Index **item strings** change; section `id` `index` does not. No typed `decisions[]` array. |
| **F12 — Session section decline** | Active-session recency stays. Do **not** pin-rank Session turns. Soft residual: Session may still show `## Objective` as current work. |
| **F13 — Recent recency freeze** | Most Recent Memories stays `ORDER BY updated_at` (T274). Do not run two-pass on Recent. |
| **F14 — Pass-2 fill stands** | Recency fill of other injectable rows after pass-1. Dumps **may** appear as items 2+. Do **not** hard-exclude transcripts from Index (T260 analog: prefer-fill, not delete). |
| **F15 — Decline T287 / T288 / T293** | `memory list` ORDER, briefing stanza, graph neighbors CLI — other placeholders. |
| **F16 — Decline T263 H2 / T240 F2** | Standing. |
| **F17 — No new CLI flag** | No `--pins-only` / `--index-authority`. `--compact` / `--summary` / `--pretty` stay. |
| **F18 — No DTO keys** | No `in_context_authority` / `index_kind`. PROTOCOL-COMPAT: N−1 ignore. |
| **F19 — Capture independence** | Selection + titles. No models. No new events. No graph default-on Cargo. **Do not rewrite** `pin.rs` stored shape. |
| **F20 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no new crates, workspace **0.1.2**. |
| **F21 — PATH** | Do not `cargo install` unless the user asks. |
| **F22 — Live vault pin** | Do **not** pin production DECISIONs as implement. Hermetic unique needle is SoT. Manual DoD **unique canary** (uuid in the string) is allowed on go. |
| **F23 — last-PR Cursor** | #201 empty → **N/A**. Dependabot remotes not this track. **No T301.** |
| **F24 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. `tempfile::tempdir` per hermetic. rstest if adding GLOB cases. |
| **F25 — Cross-model** | Retrieval Index is FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F26 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F27 — File growth** | `index_pass1_glob_sql` in `session_chrome.rs`. Pass-1 call + title line in `retrieval/src/preflight.rs`. New retrieval hermetic cases + CLI pretty/summary tagged cases. **Do not** grow `project.rs`, `sync.rs`, CLI `preflight.rs` production, `pin.rs` write, `.github/workflows/ci.yml`, `lexical.rs` (T285 Prefer join stays; **do not** extract a shared OR helper as DoD — duplicate the 8-line inner join in `index_pass1_glob_sql`). |
| **F28 — Existing tests stay green** | T274 AC6/AC7 untagged Index; T220 AC5 untagged summary JSON; T219 no leading `ASSISTANT:` on Index; T250 Index not line-capped; T265 `sections[].id=="index"`; T272 skip-set; T279 Safety empty/hotspots; T214 dual labels. |
| **F29 — Docs** | CAPABILITIES preflight: Index pass-1 is marker-GLOB **or** TAGS envelope, titles after `first_contentful_line`; summary in-context still window substring (will be ≥1 when a pin titles `DECISION:`). CHANGELOG T286. PROTOCOL-COMPAT: no new required keys. |
| **F30 — PowerShell** | `;` not `&&`. |
| **F31 — Pass-2 `NOT IN` bound** | T274 F35 stands. `bound_not_in_sql` + `params_from_iter`. No UUID literals in SQL. Empty pass-1 → omit `NOT IN`. |
| **F32 — Low-signal / privacy** | `is_low_signal` + `is_injectable_privacy` on drain stay. NeverInject pins do not enter Index. |
| **F33 — Word budget** | Pass-1 still breaks when assembled Index words exceed `max_words`. Pins must enter **before** dumps so a small `-m` can still show item 1 as a pin (Manual uses `-m 1500`; hermetic may use a modest budget like T274’s 60). |
| **F34 — JSON Index text** | Non-summary `text` Index lines **do** use envelope titles (same assembly as pretty input). Pretty role-strip remains a second pass. Agents piping JSON see `1. DECISION:` not `1. TAGS:`. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Retrieval hermetic: older `ASSISTANT: TAGS: t286\nDECISION: {needle}` + newer `## Objective\n…` dump → `build_preflight` Index contains `DECISION:` and `{needle}`; the first numbered Index line does **not** start with `## Objective`. **Required red.** |
| **AC2** | Same fixture: the first numbered Index line contains `DECISION:` and does **not** contain `TAGS:` as the title (envelope skipped). |
| **AC3** | T274 `preflight__index_prefers_leading_decision_over_objective_dump` **stays green** (untagged pin). |
| **AC4** | Unit: `index_pass1_glob_sql("m.content")` contains `GLOB 'TAGS:*'`, `GLOB 'ASSISTANT: TAGS:*'`, `GLOB 'DECISION:*'`, `GLOB 'HOTSPOT:*'`, and a single `AND (` grouping with `OR` (not two stacked `AND (` requiring both). Column ident-checked. |
| **AC5** | CLI hermetic: `preflight --pretty -m 1500 --no-hook-prompt` stdout Memory Index line `1.` (after pretty strip) starts with `DECISION:` (or `1. DECISION:`) when a tagged pin exists vs a newer Objective dump. EXIT **0**. **Required red** (CLI). |
| **AC6** | CLI hermetic: `preflight --summary --format json` `in_context_decisions >= 1` for the tagged-pin fixture. Keys T220 still present (`pinned`, `in_context_decisions`, no extra required key). EXIT **0**. **Required red** (CLI). |
| **AC7** | T220 `preflight_summary_json__legacy_markers__in_context_counts_meaningful` **stays green**. |
| **AC8** | Pretty Safety is still live hotspot paths / T279 empty SOOT — **not** `## Objective`. Regression guard (existing T279 tests stay). |
| **AC9** | Non-summary `--format json`: `sections[]` still has `id=="index"` when Index is in `text`; required `text`/`word_count` stay. |
| **AC10** | Envelope empty / `"ASSISTANT: TAGS: x"` with no following line → Index title `Untitled Memory` (unit or drain fixture). No panic. |
| **AC11** | T219: displayed Index lines still do not begin with `ASSISTANT:`. |
| **AC12** | Pass-2: a newer Objective dump **may** appear as Index item 2+ when pass-1 has the pin; item 1 is still the pin (AC1). Do not assert dumps are absent. |
| **AC13** | `--global` Index still round-robins (existing T264 tests). Skip-set still emitted Safety ids (T272). |
| **AC14** | Compact recall/preflight JSON: no new required keys. Summary JSON serde of tagged-pin run has the T220 field set only (additive optionals like `grants_status` stay skip-if-none). |
| **AC15** | `memory list` recency order unit still `updated_at DESC` (T216 / T287 freeze). |
| **AC16** | Manual PATH-or-`cargo run`: `preflight --pretty -m 1500` Index `1.` is not `## Objective` when ≥1 in-scope pin exists; `preflight --summary` still prints `Pinned memories:` and `In context decisions` is **not** 0 next to thousands of pins **or** the hermetic proof already covers it. EXIT **0**. Safety block still hotspot paths. |

---

## 5. Design notes

### 5.1 Pass-1 SQL (F2)

```
marker = index_marker_glob_sql(col)   -- AND (DECISION:* OR … OR HOTSPOT:*)
tags   = tags_envelope_sql(col)       -- AND (TAGS:* OR ASSISTANT: TAGS:*)
pass1  = AND ( <marker inner> OR <tags inner> )
```

Copy the `strip_prefix(" AND (")` / `strip_suffix(')')` join from `lexical.rs` Prefer. Do not share a helper this track (F27). Identifier check via existing `debug_assert!(is_safe_sql_ident)`.

Unbounded `ORDER BY updated_at DESC` stays (T274 — Index is not FTS `LIMIT depth`). `drain_index_pass` still breaks on word budget / global fetch cap.

### 5.2 Titles (F4 / F34)

```
line = first_contentful_line(content)
title = if line.is_empty() { "Untitled Memory" } else { line }
summary = truncate_index_summary(title)  -- 60 / 57… Unicode
"{i}. {summary} -- {relative_ts}"
```

Pretty may still strip a leftover role token (T219). JSON `text` Index lines are already envelope-titled so summary `matches("DECISION:")` works without growing CLI `preflight.rs`.

### 5.3 Why not a new summary key

Placeholder offered (a) vault-authority count vs (b) show Pinned N. (b) shipped in T214 and the audit still scored Q=7. A new `in_context_authority` key is T220 growth. Putting the pin in the window makes the **existing** `in_context_decisions` honest.

### 5.4 Session vs Index

Pretty order is Safety → Session → Index → Recent. Session chrome is **honest current work**. Index is the decision list. Do not merge them.

---

## 6. Non-goals

- Hard-exclude session ingest from Index pass-2.
- Reorder Session turns / skip chrome turns in Session.
- `memory list` pin-first (**T287**).
- Briefing vault-pin stanza (**T288**).
- `graph neighbors` pin-first (**T293**).
- T279 Safety SQL / live `safety sync` pin.
- T263 H2 pin→Approved; T240 F2 silent Scope.
- New summary JSON keys / typed `decisions[]`.
- Grow CLI `preflight.rs` production / 9-arg formatters.
- FTS MATCH on Index / title-body split / rusqlite 0.40 / clap 5.
- Rewrite `pin` storage.
- `cargo install` as planning or implement unless the user asks.

---

## 7. Verification plan (TDD)

**Red first (required):**

1. `preflight__index_prefers_tags_envelope_decision_over_objective_dump` (AC1) in `crates/ai-brains-retrieval/tests/preflight_index_pin_rank.rs` (or sibling). Same `append_pinned` / `set_updated_at` helpers as T274.
2. Assert first numbered Index line is `DECISION:` not `TAGS:` (AC2) — same test or split.
3. Unit `index_pass1_glob_sql__tags_or_marker__single_and_group` (AC4) in `session_chrome.rs` tests.
4. CLI `preflight__pretty_index_item1_is_decision_when_tagged_pin_exists` (AC5).
5. CLI `preflight__summary_json_tagged_pin__in_context_decisions_nonzero` (AC6).

**Green:** F2 helper + pass-1 extra + F4 title. Do not “fix” T274 untagged into TAGS-only.

**Stay green:** AC3, AC7–AC9, AC11, AC13–AC15, T279 Safety.

**Manual (on go):** see plan.md. Unique canary pin allowed.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| TAGS GLOB pulls tagged dumps; classify skips them but scan is wider | Envelope Other skip already in drain. Accept scan cost vs 3k pins. |
| Word budget still cuts Index after Safety+Sessions | Hermetic modest budget; Manual `-m 1500`. F33: pins enter first so item 1 can survive a cut. |
| Summary still 0 if title is `CONSTRAINT:` only | AC is **decisions** ≥1 when a DECISION pin exists. Constraint-only vaults can stay 0 decisions (honest). |
| CLI `preflight.rs` hotspot growth | Production frozen; tests only. |
| PATH-behind T285+T286 | F21. Hermetic/`cargo run` is DoD. |
| Session still Objective | F12 documented residual. |

---

## 9. Deferred absorb/decline

### Open overlapping rows (entire `deferred.md` scan)

| Item | Disposition |
|------|-------------|
| preflight Index `## Objective`; summary decisions 0 vs 3647 pins | **Absorb** F1–F4 / AC1–AC6 / AC16 |
| T285 F13 / closeout Index/summary still Objective | **Absorb** (this track) |
| T274 AC6/AC7 Index pin + window `DECISION:` | **Absorb as regression** F5 / AC3 |
| T274 closeout live dumps until install | **Partial** — PATH 0.1.2 until owner install (F21); source hole is TAGS GLOB |
| T264 Index fetch-80 leftover-heavy | **Decline** — T264 soft; not leftover drop (T264 F11 / T276) |
| T272 Index skip vs Safety | **Affirm freeze** F6 / AC13 |
| T219 Index `ASSISTANT:` strip | **Affirm** F8 / AC11 — titles also skip TAGS |
| T250 Index not line-capped | **Affirm** F8 |
| T214 dual counts / 9-arg | **Affirm** F10 |
| T220 summary keys / T265 `sections[]` | **Affirm** F10 / F11 |
| T279 Safety Objective | **Decline** — Completed; F7 |
| `memory list` just-now | **Decline → T287** |
| briefing granted-empty | **Decline → T288** |
| graph neighbors dumps | **Decline → T293** |
| leftover dest-missing | **Decline → T294** |
| T240 F2 / T263 H2 / 750 ms / clap 5 / density floors | **Decline** F16 / F20 |
| last-PR Cursor #201 | **N/A** empty — **no T301** |
| Identity mismatch leftover `7d97a456` | **Not this track** — T258 / T294 |
| Session chrome in pretty | **Decline as DoD** F12 — residual |

### last-PR Cursor (#201)

`gh pr view 201 --comments`, `pulls/201/reviews`, `pulls/201/comments` → **[]**. Open HEAD PR: none (main). Dependabot remotes are not findings. **No mint.**

---

## 10. Implement order (on go)

1. Phase 0 re-verify pins + deferred + #201 still empty.
2. Red AC1/AC2/AC4/AC5/AC6.
3. Green `index_pass1_glob_sql` + pass-1 extra + envelope titles.
4. Confirm T274/T220/T219/T279/T272/T265 stay green.
5. Docs CAPABILITIES + CHANGELOG.
6. FEATURE TX commit; targeted nextest; clippy `-D warnings`; `codex-review`; full gate; publish Phase 6.

---

## 11. Soft residuals

- Session section still recency chrome (F12).
- Index items 2+ may still be `## Objective` (F14).
- PATH until `cargo install` (F21).
- `in_context_decisions` still a substring, not a vault-authority COUNT (F10).
- Duplicate OR-join with lexical Prefer (F27).
- T264 Index fetch-80 leftover-heavy (not this).
- Duplicate `classify_pin_kind` on every drain row (pre-existing).

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/session_chrome.rs` | Add `index_pass1_glob_sql`; unit AC4 |
| `crates/ai-brains-retrieval/src/lib.rs` | Re-export `index_pass1_glob_sql` if tests need it |
| `crates/ai-brains-retrieval/src/preflight.rs` | Pass-1 extra; Index title `first_contentful_line` |
| `crates/ai-brains-retrieval/tests/preflight_index_pin_rank.rs` | AC1/AC2 tagged; AC3 stays |
| `crates/ai-brains-cli/tests/` | AC5 pretty; AC6 summary JSON tagged |
| `Docs/CAPABILITIES.md` | Index envelope + TAGS-or-GLOB sentence |
| `CHANGELOG.md` | T286 |
| `conductor/conductor.md` | Planned → (on go) In Progress → Completed |
| `conductor/deferred.md` | This absorption table |

**Do not touch:** `project.rs`, `sync.rs`, CLI `preflight.rs` production, `preflight_pretty.rs`, `pin.rs` write, `lexical.rs`, `preflight_safety.rs`, `ci.yml`, clap/rusqlite pins.

---

## 13. AI fold-in

None this pass (plan). Review-track writes `*-review.md` only after `/review-track 286`.
