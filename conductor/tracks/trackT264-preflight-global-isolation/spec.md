# T264 — Preflight global isolation

- **Track ID:** T264-PreflightGlobalIsolation
- **Status:** **Planned** (Pending in registry; plan-only until go)
- **Category:** UX / FEATURE
- **Owner:** —
- **Source:** Audit 2026-08-16 — `preflight --global --pretty --compact` **5/4**; `--global --summary` **7/6**; opportunity “label or cap per project”
- **Depends on:** T214 global rollup ✅; T219/T250 pretty ✅; T220 summary JSON ✅; T230 `display_label` ✅; T259 leftover split ✅ (memories stay); T260 symbol demote ✅
- **Blocks / feeds:** `--global` stays a **rollup**, not this-repo law. Full JSON envelope stays **T265**. Format maze stays **T266**. List footer leftover-as-AI-Brains stays **T267**.
- **Absorbs:** Audit T264 row (Safety blender + summary mix); T214 residual “true multi-project rollup or honest label”; T219 F13 *global-only* exception (project-scoped selection stays); T260/T261/T262 closeout leftover-**project** `--global` pointer **as preflight isolation** (not recall drop); T214 `active_sessions` `format!` SQL on the file we must touch
- **Not absorbed:** Drop leftover `7d97a456` from `recall --global` (changes `--global` meaning); T265 structured `sections[]`; T266 format maze; T267 footer; ledgerful-on-global (T214 F9); grow `PreflightContextResponse` (T180); T240 F2; T255 declines; clap 5
- **Research date:** 2026-08-18 (plan dogfood HEAD `d8be361` T263 `#178`)
- **AI fold-in:** none yet (plan pass)
- **Ledger:** planning DOCS TX `a0500604-b8ff-47b9-b24d-9c0923b8855e`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** write live `.env`. Do **not** drop leftover from `--global` recall. Do **not** enable `AI_BRAINS_GOVERNED_BRIEFING`. Do **not** reopen T240 F2 / T255 declines. Do **not** grow T180 `{text, word_count}` keys.

---

## 1. Objective

`--global` remains a **vault rollup**, not a blender that presents other repos’ DECISIONs as *this* repo’s bearings.

1. **Label.** Under `--global`, every Safety item, Memory Index line, Recent memory, and Session **header** carries a project tag. JSON `text` uses a stable 8-char project id. Human/pretty upgrades that tag via T230 `display_label`.
2. **Cap per project.** Stop the single blended top-N. Round-robin recency with a per-project ceiling so leftover / hip-hierarchy / coordinator cannot occupy the whole Safety or Session window.
3. **Summary span honesty.** Keep T214 vault totals + T220 `In context` marker counts. Add how many **projects** those in-context lines span. Do not suppress the counts.
4. **Leave project-scoped pretty alone.** No `--global` → no tags, no new caps, T219 F13 selection stands.

That advances the north star: capture stays grant-independent; the append-only log stays SoT; agents starting with `preflight --global --pretty` can see *whose* constraint they are reading.

No models. No new crates. No clap 5. No T180 key growth. No leftover-project drop from `recall --global`.

---

## 2. Live baseline (2026-08-18)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `d8be361` T263 `#178` on `main`. Tree CLEAN. `origin/main` even (`00`). |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **0.1.1**. Compact / summary JSON exist (T250/T220). **Do not `cargo install`.** Preflight `--global` assembly is retrieval (unchanged by T263) — PATH dogfood matches source class. |
| `preflight --summary` (daily) | Scope path owner `3581317d` (`C:\dev\ai-brains`). Pinned **2995**. Grants **0 of 3** (T241; not this track). |
| `preflight --global --summary` | `Scope: global`. **Projects: 53**. Pinned **36632**. Active sessions **40**. **In context decisions: 22** / constraints: 4 / hotspots: 0. Word count **1500**. **No project span.** |
| `preflight --global --summary --format json` | T220 pretty object. `scope=global`, `projects=53`, `in_context_decisions=22`. No span key. |
| `preflight --global --pretty --compact -m 400` | Safety: unlabeled `DECISION/CONSTRAINT: T260` + `T257` (this repo). Session `d6fb6231`: unlabeled hip-hierarchy **DECISION 0098 / 0085**. **This is the live blender.** |
| `preflight --global --pretty -m 800` | Safety: T260 / T257 / Desktop CONSTRAINT (this-repo recency). Sessions: (1) hip-hierarchy 0098/0085, (2) AI-Brains T263 review `d10ff30d`, (3) conductor **0011-OwnerHitl** `f73229ca`, `+2 more sessions`. No `[label]`. |
| `preflight --pretty --compact -m 400` (no `--global`) | Scope `project=C:\dev\ai-brains (3581317d-…)`. This-repo session + index. **No foreign DECISION.** Keep this path frozen. |
| Audit 2026-08-16 | Safety then mixed coordinator **0022/0023** + hip-hierarchy review **0092**. Those pins aged out of LIMIT 10; the **class** is live in Sessions today. |
| Last GitHub PR | [#178](https://github.com/Ryan-AI-Studios/AI-Brains/pull/178) T263. `gh pr view --comments`, `/reviews`, `/comments` all **empty**. No open PR on `main`. **last-PR Cursor: N/A.** |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings / :8081 unreachable; :8083 ok). 0 pending 0 drift. Hotspot **#1** `project.rs` (3.874) — **do not edit.** `#9` `preflight.rs` (2.132) — new pretty helpers go in a **sibling file**. |
| ai-brains recall | Lexical/semantic: T260 leftover-`--global` pointer, `build_preflight`, `display_label`. No prior “label+cap, do not drop leftover from recall --global” pin. |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Unlabeled `--global` pretty | An agent treats hip-hierarchy 0098 as AI-Brains law. T214 fixed the **header** (`Scope: global`) and **vault SQL**. The **body** is still a recency blender. **DoD.** |
| Single top-N | Safety SQL `ORDER BY updated_at DESC LIMIT 10` + pretty first 8. Sessions `active_sessions(conn, None)` then pretty first 3. One busy leftover/foreign project wins. **DoD = per-project cap.** |
| Summary “In context decisions: 22” | Honest as a budget-window count (T214 F4) but hides *whose* 22. **DoD = span line + optional JSON key.** Do not drop the counts. |
| Drop leftover from `recall --global` | T260/T259 closeout pointed this here. `--global` means all projects. T259 F5: memories stay on leftover. Dropping `7d97a456` hides the only unscoped path to 18k historical pins. **Decline drop.** Isolation is label+cap on **preflight**. |
| T265 JSON envelope | Agents still get `{text, word_count}`. Labels *inside* `text` are enough for honesty. Structured `sections[]` stays T265. |
| T219 F13 | Project-scoped selection/ranking/LIKE stay. This track is the **documented global exception**. |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Legacy assembly | `ai-brains-retrieval/src/preflight.rs` `build_legacy_preflight` | Safety / index SQL unscoped when `global`. **No `project_id` in SELECT.** |
| Safety LIKE | same `:276–288` | `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` only — **not** `DECISION:`. Live Safety hits are `DECISION/CONSTRAINT:` pins that match CONSTRAINT. Do **not** widen the LIKE set. |
| Safety LIMIT | `LIMIT 10` | Then T219/T250 pretty `safety_max` 8 / compact 3. |
| Index / Recent | same `:379–475` | Global: all pinned recency. Recent = first 3 of collected. No project tag. |
| Sessions | `sessions.rs` `active_sessions` | `SessionContext { session_id, turns }` — **no project_id**. Global = all active. SQL uses `format!` for the scoped arm (T214 residual). |
| Pretty caps | `cli/.../preflight.rs` `PrettyCaps` / `format_preflight_pretty_body_with` | Display-only. Does not know projects. Chrome strip is **leading** `(ts) ROLE:` / `ROLE:` — a `[label]` prefix would break it unless pretty peels the tag first. |
| Summary | `format_preflight_summary_lines` / `PreflightSummaryJson` | T214 dual counts + T220 keys. No span. Additive `#[serde(skip_serializing_if = "Option::is_none")]` already used (`projects`, `grants_status`). |
| T180 DTO | `PreflightContextResponse` | `{text, word_count}` only. **Do not grow.** |
| Internal context | `retrieval::PreflightContext` | `{text, word_count}` — **3** construction sites, all in `preflight.rs`. Additive field OK (not a contract). |
| `display_label` | `project.rs:422` T230 | Hotspot #1 — **call** from CLI pretty, do not move / edit `project.rs`. |
| `get_project_by_id` | `query_store.rs:637` | `(name, alias)` JOIN `project_alias_projection`. Reuse from CLI. |
| Governed `--global` | `build_governed_preflight` | Already empty packet + warning. **Leave.** |
| Ledgerful | `query_ledgerful` | Skipped when `global` (T214 F9). **Leave.** |
| Hermetics | `tests/preflight_global_summary.rs`, `preflight_summary_json.rs`, `preflight_pretty_readability.rs` | Two-project vault pattern exists. Add isolation suite; keep T214/T220/T250 green. |

### 2.4 Dependency / standards research (2026-08-18)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (docs.rs; MSRV 1.85) | **No bump.** No new flags required (`--global` / `--pretty` / `--compact` / `--summary` exist). clap **5** not current. Snapshot — re-verify at execute. |
| `serde_json` | workspace **1.0** / lock **1.0.150** / crates.io **1.0.151** | **No bump.** Additive optional summary key only. |
| `rusqlite` | workspace SQLCipher **0.39** | **No bump.** New SELECTs use `params![]` only. |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.1** | **No bump.** |
| New crates | — | **Zero.** |
| [clig.dev](https://clig.dev/) (fetched 2026-08-18) | Human-readable first; “saying just enough”; changing human output is OK; JSON is the stable machine path; Heroku `--all` team scope lists apps **grouped**, not merged as one tenant | `--global` stays an explicit widen (T214). Items must say which tenant. Compact still “just enough” via per-project cap + T250 first-line. |
| T180 / PROTOCOL-COMPAT | compact 2-key freeze; summary pretty envelope; additive optional keys ignored by N−1 | Labels live **in** `text`. Optional `in_context_project_span` on **summary JSON only**. |
| T219 F13 / T250 | Marker selection / LIKE / `dedup_hotspots` frozen for **project-scoped** | Global selection is this track. Pretty chrome strip must peel `[tag]` first. |
| T214 F9 / T170 D21 | No ledgerful-on-global; summary ≠ governed authority | Affirm. |

Training data is not a pin. Re-verify clap/serde_json at execute.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Global only** | Tags + per-project caps apply iff `global == true`. Project-scoped / unresolved pretty, JSON, and summary stay T214/T219/T220/T250. |
| **F2 — Tag in retrieval text** | When global, retrieval prefixes Safety items, Index lines, Recent entries, and Session **headers** with `[` + first 8 hex chars of `COALESCE(m.project_id, s.project_id)` + `]`. Missing/unparseable id → `[unknown]`. Session **turns** are **not** tagged (header is the owner; keeps T250 leading chrome strip). |
| **F3 — Pretty upgrades the tag** | Human/pretty (not JSON, not summary) rewrites `\[[0-9a-fA-F]{8}\]` / `[unknown]` via `get_project_by_id` + T230 `display_label`. Fallback stays the 8-char (or `unknown`). Do **not** edit `project.rs` — **call** `display_label`. |
| **F4 — Peel before chrome** | Pretty must strip / remember the `[tag]` **before** `strip_pretty_chrome` / `strip_role_prefix`, then reattach the (possibly upgraded) tag. Units lock `(ts) ROLE:` and leading `ROLE:` still strip when a tag is present. |
| **F5 — Per-project caps (retrieval)** | After recency ORDER BY, apply **round-robin by first-seen project** then next slot per project. Ceilings (global only): Safety **2 / project**, vault **8**; Index **3 / project**, vault **15**; Recent **1 / project**, vault **3**; Sessions **1 / project**, then existing pretty session cap (3 / compact 1). Fetch window > vault cap (Safety SELECT LIMIT **40**, not 10) so later projects can fill. LIKE / `dedup_hotspots` / low-signal / privacy filters **unchanged**. |
| **F6 — Algorithm SOOT** | Pure `take_round_robin(items, per_project, max_total)` in **`ai-brains-retrieval/src/preflight_global.rs`**. Recency order inside each project bucket. Emit i-th item from each first-seen project, `i = 0..per_project`, stop at `max_total`. Not “first 8 of blended LIMIT 10”. |
| **F7 — Summary span** | Under `--global` only, after `In context constraints:` print `In context spans N projects` where `N` = distinct non-`unknown` project ids that contributed at least one **emitted** Safety/Index/Recent/Session item (post-cap, pre-pretty). `N == 0` still prints (`spans 0 projects`) so the line is stable. Project-scoped: **omit** the line. |
| **F8 — Summary JSON additive** | `PreflightSummaryJson.in_context_project_span: Option<u32>` with `skip_serializing_if = "Option::is_none"`. Present only when `scope == "global"`. `api_version` stays `"1"`. T220 required keys unchanged. No `PreflightContextResponse` growth. |
| **F9 — Internal span field** | Add `PreflightContext.in_context_project_span: Option<u32>` (`Some` iff global). Governed empty-global → `Some(0)`. Not serialized on the T180 path. |
| **F10 — Session project_id** | `SessionContext.project_id: Option<String>`. `active_sessions` SELECTs `project_id`. Any edit of `sessions.rs` converts both arms to **`params![]`** (absorb T214 format! residual on this file only). Do not load all vault turns differently. |
| **F11 — Decline leftover recall drop** | Do **not** exclude leftover `7d97a456` (or any project) from `recall` / `search` `--global`. T259 F5 + `--global` means all projects. |
| **F12 — Decline T265 envelope** | No `sections[]`. Labels in `text` are the honesty fix. |
| **F13 — Decline LIKE widen** | Do not add `DECISION:` to Safety SQL. Foreign DECISIONs that appear do so via Session/Index/Recent or CONSTRAINT-bearing pins — label those. |
| **F14 — Decline ledgerful-on-global** | T214 F9 stands. |
| **F15 — Decline GOVERNED_BRIEFING** | T170 D21. Governed `--global` stays empty packet. |
| **F16 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.1**. |
| **F17 — Capture independence** | SQL + string labels only. No models, embeddings, graph, or new events. |
| **F18 — Tests** | Naming `function_or_feature__condition__expected_result`. Pure round-robin + peel/upgrade units. Hermetic two-project vault for pretty labels + summary span + JSON key + project-scoped negative (no `[8hex]`). Existing T214/T220/T250 hermetics stay green. No `unwrap`/`expect`/`panic` in production. |
| **F19 — Cross-model** | FEATURE (operator `--global` body + additive summary key). After Phase-1 review clean, run read-only `codex-review`. |
| **F20 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F21 — PATH-behind** | Do not `cargo install` unless the user asks. Tests/manual AC use `cargo run` / hermetic bin. |
| **F22 — Stop-before** | Even after go: no live `.env`, no leftover rebind, no `policy bootstrap`, no T240 F2 silent Scope switch, no `nightly` mutate. |
| **F23 — Hotspots / files** | Do **not** edit `project.rs`. New retrieval helpers in `preflight_global.rs`. New CLI pretty peel/upgrade in **`preflight_pretty.rs`** (or equivalent sibling) — do not grow hotspot `#9` `preflight.rs` except dispatch + summary line/JSON field. `sessions.rs` + `build_legacy_preflight` are required. |
| **F24 — Tag grammar** | Tag is `[` + exactly 8 ASCII hex chars + `]` or literal `[unknown]`. No `]` inside labels after pretty upgrade: `display_label` output is truncated at **32** chars and any `]` replaced with `·` before wrap. |
| **F25 — Compact / line-cap** | T250 `PrettyCaps` numbers stay. Caps in **F5** run in retrieval *before* pretty. Compact may show fewer items but never unlabeled foreign lines under `--global`. |
| **F26 — Contracts** | PROTOCOL-COMPAT: one additive row for summary optional `in_context_project_span`. T180 compact 2-key test stays green. CHANGELOG + CAPABILITIES. |
| **F27 — T266 / T267 / T268+** | Format maze, list footer, scan-roots — **not** this track. |
| **F28 — T240 F2 / T255** | Stay closed unless owner reopens. |
| **F29 — Word-budget honesty** | Tags count as words in `trim_to_word_budget`. Do not change the helper. Default `-m` 1500 still applies. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Pure: `take_round_robin` on 5 leftover + 5 other (recency leftover-first), `per_project=2`, `max=8` emits **2 leftover then 2 other** (interleaved by round), never 5 leftover. |
| **AC2** | Pure: empty input → empty; all-unknown project ids still respect `max_total`; `per_project=1` emits at most one per id. |
| **AC3** | Unit: `peel_global_tag("[3581317d] (just now) ASSISTANT: DECISION: x")` → tag + remainder whose chrome strip is `DECISION: x`. Untagged `(just now) ASSISTANT:` still strips (T250 AC stands). |
| **AC4** | Unit: `upgrade_global_tag` with alias `acme` → `[acme]`; missing project → original 8-char; `]` in name sanitized (F24). |
| **AC5** | Hermetic two-project vault: pin `CONSTRAINT: alpha-only` in A and `CONSTRAINT: beta-only` in B. `preflight --global --pretty --no-hook-prompt` stdout contains both bodies **and** a `[` tag on each Safety item. No Safety/Index/Recent/Session-header line that has `CONSTRAINT:` / `DECISION:` / `HOTSPOT:` **without** a leading `[`. |
| **AC6** | Same fixture, **no** `--global`, `--pretty`: stdout has **neither** `[`+8 hex `]` tags **nor** the span line. Both constraints do **not** appear together unless they share the scoped project (project-scoped negative). |
| **AC7** | Hermetic: `--global --summary` contains `In context spans N projects` with `N >= 2` on the two-project fixture. Project-scoped `--summary` does **not** contain `spans`. |
| **AC8** | Hermetic: `--global --summary --format json` parses; `in_context_project_span` is a number `>= 2`; T220 required keys still present; no human banner. Project-scoped summary JSON **omits** the key. |
| **AC9** | Hermetic: `--global --format json` (no summary) still exactly 2 compact keys (`t180_c_preflight_json_keys` green). `text` contains `[`+8 hex `]` on Safety items (agent honesty). |
| **AC10** | Hermetic: 3 CONSTRAINT pins in A (newer) + 1 in B. `--global --pretty` Safety includes **B** (not 3×A only). At most **2** Safety items from A (F5). |
| **AC11** | Unit/hermetic: `--global --pretty --compact` Safety/Session lines that remain are still tagged. T250 compact numbers still apply (≤3 Safety items, ≤1 session). |
| **AC12** | Existing `preflight_global_summary` / `preflight_summary_json` / `preflight_pretty_readability` stay green (Scope: global, T220 keys, 140-cap, compact). |
| **AC13** | Docs: CAPABILITIES `--global` rollup bullet (label + per-project cap + span); PROTOCOL-COMPAT summary optional key; CHANGELOG T264. |
| **AC14** | Manual (source bin, classify-only): `preflight --global --pretty --compact -m 400` Session `d6fb6231` (or current foreign session) shows a **non-AI-Brains** label/id; T260 Safety line shows this-repo label/id. Exit **0**. Do **not** pin. Do **not** `cargo install`. |

---

## 5. Design notes

### 5.1 Why label in retrieval, upgrade in pretty

T180 freezes **keys**, not the `text` string. Agents on `--format json --global` must not see unlabeled foreign CONSTRAINT as local law — so the 8-char tag is in retrieval. Humans should see `C:\dev\ai-brains` / alias, not a raw uuid — T230 `display_label` already exists. Copying T230 into retrieval would fork SOOT and force a `project.rs` extract (hotspot #1). Call it from CLI pretty only.

### 5.2 Round-robin vs first-fit

First-fit (“take 2 leftover, then others”) still lets compact’s first 3 be leftover+leftover+one. Round-robin by first-seen project puts one line per tenant before the second leftover line. Compact `--global` then shows **multiple repos** in 3 Safety slots.

### 5.3 Session header vs turn tags

Turns emit `ROLE: …`. A turn-level `[tag]` breaks T250 leading chrome. The header `--- Session: {uuid} [3581317d] ---` is enough: every turn under it is that project. Pretty classifies the header as Session; F31 `+N more turns` unchanged.

### 5.4 Span vs marker counts

`In context decisions: 22` stays a **text marker scan** (T214 F6 / T220 F31). Span is **SQL identity** of emitted items, not a parse of `DECISION:` lines (one pin can contain both DECISION and CONSTRAINT). They can disagree; docs say so.

### 5.5 Fetch window

`LIMIT 10` then “cap 2 per project” cannot see project B if A filled 10 rows. Safety LIMIT **40** (5× vault cap). Index already has no LIMIT (budget break). Sessions already load all active then we cap.

---

## 6. Non-goals

- Dropping any project from `recall` / `search` / `sync query` `--global`
- T265 `sections[]` / json-v2
- T266 format policy / T267 list footer
- Adding `DECISION:` to Safety LIKE
- Ledgerful under `--global`
- `AI_BRAINS_GOVERNED_BRIEFING` / governed multi-project packet
- Growing `PreflightContextResponse`
- clap 5 / lock bumps / new crates
- Editing `project.rs` (except calling `display_label`)
- Live leftover `rebind-path` / `.env` / `policy bootstrap`
- T240 F2 silent Scope switch; T255 doctor-16th / product `.cmd`
- Changing T250 `PrettyCaps` numbers or `trim_to_word_budget`

---

## 7. Verification plan (TDD)

**Red first (names):**

1. `take_round_robin__leftover_then_other__interleaves_per_project` (AC1)
2. `take_round_robin__empty_and_unknown__respects_max` (AC2)
3. `peel_global_tag__tagged_timestamp_role__chrome_still_strips` (AC3)
4. `upgrade_global_tag__alias_missing_and_bracket` (AC4)
5. `preflight_global_isolation__two_projects__pretty_labels_and_no_unlabeled_safety` (AC5)
6. `preflight_global_isolation__project_scoped__no_tags_no_span` (AC6)
7. `preflight_global_isolation__summary_span_and_json_key` (AC7–AC8)
8. `preflight_global_isolation__three_a_one_b__b_appears_a_capped` (AC10)
9. `preflight_global_isolation__compact_still_tagged` (AC11)

Then green: `preflight_global.rs` + `sessions.rs` + `build_legacy_preflight` + `preflight_pretty.rs` + summary field. Re-run T214/T220/T250 hermetics + `t180_c_preflight_json_keys`.

Manual AC14 on source bin only.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Pretty chrome regression | F4 peel-first; AC3 unit; T250 hermetics AC12 |
| Hotspot `preflight.rs` growth | F23 sibling files |
| `display_label` fork | F3 call only; no `project.rs` edit |
| T180 break | AC9; no new required keys |
| T220 key surprise | Optional skip_serializing; AC8 omit on project scope |
| Word-budget shrink from tags | F29 accept; 8-char tags are cheap |
| `format!` SQL leftover | F10 params![] on touch |
| PATH-behind | F21; AC14 source bin |
| Over-scope leftover recall drop | F11 decline, written |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit `preflight --global` blender (5/4, summary 7/6) | **Absorb** F1–F8 / AC5–AC8 / AC14 |
| T214 “true multi-project rollup or honest label” | **Absorb** label + cap (body). Header already T214. |
| T214 `active_sessions` `format!` SQL | **Partial absorb** F10 — this file only |
| T214 F9 ledgerful-on-global | **Decline** F14 |
| T219 F13 marker selection | **Partial** — project-scoped stands; global selection **this** track |
| T250 F12 governed section caps / `--max-line` / pager | **Decline** — not blender |
| T220 required keys / T180 2-key | **Affirm** F8 / F12 / AC8–AC9 |
| T259/T260/T261 leftover-project `--global` | **Partial:** preflight label+cap **absorb**. Recall **drop decline** F11 |
| T227 F3 / T263 H2 pin inject | **Decline** — not preflight authority |
| T241 0-of-3 daily grants | **Decline** — already warns |
| T265 JSON envelope | **Decline** F12 |
| T266 / T267 / T268–T271 | **Decline** F27 |
| T240 F2 / T255 declines | **Decline** F28 |
| last-PR Cursor #178 | **N/A** — comments/reviews/inline empty; nothing to mint |
| MSI / clap 5 / ISSUES.md | **N/A** / forbidden / does not exist |

---

## 10. Implement order (on go)

1. Phase 0 re-verify (plan.md).
2. Red: pure round-robin + peel/upgrade units.
3. Green: `preflight_global.rs` + wire Safety/Index/Recent/Session tags + caps.
4. `sessions.rs` `project_id` + params![].
5. `PreflightContext` span + summary line + JSON optional key.
6. CLI `preflight_pretty.rs` peel + `display_label` upgrade.
7. Hermetics AC5–AC11; keep T214/T220/T250/T180 green.
8. Docs. Review → codex-review (FEATURE). Gate. PR.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| Recall leftover-first under `--global` | Ranking, not preflight. T260 demoted symbols. Do not drop the project (F11). New track only if owner wants a **filter flag**, not a silent exclude. |
| Safety LIKE still omits pure `DECISION:` pins | Intentional F13. They appear in Index/Recent/Session when recent. |
| Span vs marker-count disagreement | Documented §5.4 |
| Daemon/HTTP preflight | None today. CLI only. |
| PATH until `cargo install` | F21 |
| `display_label` extract out of `project.rs` | Soft — do not do it here |

---

## 12. Touch map

| Path | Why |
|------|-----|
| `crates/ai-brains-retrieval/src/preflight_global.rs` | **New.** `take_round_robin`, tag prefix, span count + units |
| `crates/ai-brains-retrieval/src/preflight.rs` | SELECT `project_id`; call helper; `PreflightContext` field; Safety LIMIT 40 when global |
| `crates/ai-brains-retrieval/src/sessions.rs` | `project_id`; params![] |
| `crates/ai-brains-retrieval/src/lib.rs` | `mod preflight_global` |
| `crates/ai-brains-cli/src/commands/preflight_pretty.rs` | **New.** peel / upgrade / chrome-safe emit |
| `crates/ai-brains-cli/src/commands/preflight.rs` | Dispatch to sibling; summary span line; JSON field |
| `crates/ai-brains-cli/src/commands/mod.rs` | `mod preflight_pretty` |
| `crates/ai-brains-cli/tests/preflight_global_isolation.rs` | **New** hermetic AC5–AC11 |
| Existing T214/T220/T250 tests | Must stay green (AC12) |
| `Docs/CAPABILITIES.md` / `Docs/PROTOCOL-COMPAT.md` / root `CHANGELOG` | F26 / AC13 |
| `conductor/conductor.md` / `deferred.md` / README-T256–T271 | Registry + absorb notes |
| **Do not touch** | `project.rs` (call only), contracts `PreflightContextResponse`, `word_budget.rs`, `query_ledgerful`, governed packet builder, clap pins, live `.env` |
