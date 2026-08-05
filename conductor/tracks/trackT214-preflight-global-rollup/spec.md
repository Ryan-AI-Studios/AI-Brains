# T214 — Preflight global rollup honesty

- **Track ID:** T214-PreflightGlobalRollup
- **Phase:** Post-T213 skill·CLI audit follow-ups (P3 honesty residual)
- **Status:** 📋 **Proposed / Expanded** (plan-only until **go**)
- **Depends on:** T56 smart preflight (summary heuristics); T112 `--global` retrieval semantics; T170 D21 (never use `--summary` for governed authority); T180 compact JSON keys; T198 empty success; T207 Scope line SOOT (`format_scope_line` + `get_project_by_id`); T212 label display patterns (soft reuse); T213 closed (orthogonal graph honesty)
- **Blocks / feeds:** Operator/agent trust that `preflight --global --summary` is not a single-project lie; residual forget-list stays **T216**; full multi-project **governed** packet stays out
- **Category:** FEATURE / BUGFIX / DOCS
- **Source:** Non-destructive skill/CLI audit 2026-08-04 — **preflight --global summary 6/6**
- **Deferred absorbed:** deferred.md T214 placeholder; audit2 T56 gap (marker counts ≠ structured); series residual “true multi-project rollup or honest label”
- **Not absorbed:** Governed multi-project ProjectBriefingPacket; enabling ledgerful bridge under `--global` (intentionally off today); rewrite of full preflight budget assembly / ranking; auto `--global` on empty; clap 5; MSI; `is-terminal`→`std::io::IsTerminal` migrate; T216 forget-list; silent growth of `PreflightContextResponse` keys (T180 lock)
- **Research date:** 2026-08-05 (expand + live re-scan + online CLIG / dep pins)
- **AI fold-in:** 2026-08-05 — AI1 affirms F2–F11 core (Scope SOOT, Active sessions SQL, dual counts, JSON freeze). AI2 **M1–M5** accepted; **L2/L3/L5/L6** elevated; **L1/L4/L7/L8** notes. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. **Honest scope label:** `preflight --summary` must not print `Project: <env-uuid>` when `--global` is set (or when no project is resolved). Align with T207 vocabulary: **`Scope: global`** / **`Scope: project=…`**.  
2. **True global rollup stats (SQL):** when `--global`, summary includes vault-scale counts that are **not** budget-window heuristics — at least projects-with-pins, pinned memories, active sessions.  
3. **Honest context-window markers:** HOTSPOT / DECISION / CONSTRAINT counts remain useful orientation but are **labeled as in-context** (budget-truncated text), not presented as full-vault totals.  
4. **Fix dead Active Sessions counter:** stop matching missing `"Session ID:"` marker; preflight body uses `--- Session: {uuid} ---`. Prefer SQL/`active_sessions` for the rollup line.  
5. **Capture independence:** summary + SQL stats never require models, embeddings, or graph crate.  
6. **No contract surprise:** non-summary `preflight --format json` stays `{text, word_count}` only (T180). Summary remains human stdout unless an explicit optional additive path is approved (default: no).

## 2. Live baseline (re-scan 2026-08-05)

### 2.1 Audit signal — confirmed live

| Fact | Live |
|------|------|
| `preflight --summary` (project from local `.env`) | `Project: 441837f6-…`; Hotspots/Decisions/Constraints heuristic; **Active Sessions: 0** always; word_count from split text |
| `preflight --global --summary` | **Still** `Project: 441837f6-…` (or other env id) while body is multi-project — **false single-project identity** |
| `preflight --global --summary --no-project-context` | Still printed a **concrete Project uuid** (from remaining env / global dotenv) — confirms label ignores `global` flag |
| Help text | `--global` = “Aggregate context across ALL projects (ignores project_id filter)” — **content** mostly honors; **summary header does not** |
| JSON path | `{"text","word_count"}` compact; T180 locks key count = 2 |
| Content under `--global` | Legacy SQL unscoped; **ledgerful bridge skipped** when global (`if !global` in retrieval) |
| Governed flag | `global \|\| project_id.is_none()` → empty governed packet + warning (T170: do not use summary for governed) |
| Active session text marker | Body emits `--- Session: {id} ---`; summary counts `"Session ID:"` → **always 0** |
| Onboarding / skill | Session start recommends `preflight --summary` |

### 2.2 Code / touch map

| Site | Role |
|------|------|
| `ai-brains-cli/src/commands/preflight.rs` | `print_summary` rewrite; pure format helpers; pass `global`; SQL stats gather or call store |
| `ai-brains-cli/src/main.rs` | Preflight dispatch: when `global`, clear effective `project_id` for labeling parity with recall (defense in depth); keep passing `global: true` into `build_preflight` |
| `ai-brains-cli/src/commands/recall.rs` | **Source** of `format_scope_line` — extract shared SOOT or `pub(crate)` re-export for preflight |
| Soft: `commands/scope_display.rs` (new) | Soft residual extract — **v1 prefers `pub(crate)`** `format_scope_line` from recall (AI2 F13) |
| `ai-brains-store` / `QueryStore` | Reuse `get_project_by_id` (T207); **new parameterized** helpers: `count_projects_with_pinned`, `count_pinned_memories(project_id: Option)`, `count_active_sessions(project_id: Option)` — **never** `format!` SQL (M1) |
| `ai-brains-retrieval/src/preflight.rs` | **No content algorithm change required for DoD.** Document global bridge skip. Do not flip ledgerful-on-global without product decision |
| Hermetic | `tests/preflight_global_summary.rs` (new) — multi-project vault; assert Scope + SQL counts + no false Project uuid |
| Unit | pure `format_preflight_summary` / scope / session-marker / label “In context” |
| Docs | CAPABILITIES preflight section; CHANGELOG; soft skill one-liner; PROTOCOL-COMPAT only if JSON grows (default no) |
| Smoke | `smoke.rs` preflight local `.env` Project line → update to **Scope:** vocabulary |

### 2.3 Root cause (frozen)

```text
print_summary(ctx, options.project_id, &context.text)
// ignores options.global
// Project: {project_id} even when build_preflight(..., global=true) widened SQL
// Active Sessions = text.matches("Session ID:")  // marker never emitted
```

Recall already does:

```text
if global { (None, None) } else { (project_id, session_id) }
```

Preflight does **not** — label path diverges.

### 2.4 Deps / pins (researched 2026-08-05)

| Item | Workspace / note |
|------|------------------|
| clap | **4.5** workspace; crates.io latest **4.6.5** — **no bump** DoD |
| rusqlite | **0.39.0** SQLCipher — no bump |
| serde / serde_json | 1.0 — summary human-only; no required DTO growth |
| is-terminal | **0.4.x** (crates.io **0.4.17**); std `IsTerminal` preferred since Rust 1.70 — **soft residual migrate**, not DoD |
| Zero new crates | Required — no table crates, no CLI framework swap |
| Capture independence | Summary SQL + marker scan only |

### 2.5 Online / product research

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) human-first; suggest next step; scope flags should mean what they say | Header must match effective scope; footer already suggests `--pretty` / json |
| T207 Scope vocabulary | Reuse `Scope: global` / `Scope: project=…` — **do not invent** `Project: global` as half-fix |
| T170 D21 | Governed authority via `preflight --format json` / briefing — **never** `--summary` for governed truth |
| T180 JSON key freeze | Do **not** silently add summary fields to `PreflightContextResponse` |
| audit2 T56 | Marker counts are orientation-only — label them honestly under global scale |
| Heroku-style `--all` team scope | Global is an explicit widen flag; never auto-widen (T207 F9 family) |

## 3. Frozen decisions (F1–F36)

| ID | Decision |
|----|----------|
| **F1 — Surface** | Primary: `ai-brains preflight --summary` with/without `--global`. Secondary: CAPABILITIES + skill session-start line. Full text / JSON paths keep existing assembly. |
| **F2 — Scope vocabulary (T207 align)** | Summary header uses **`Scope:`** line, not `Project:`: |
| | • `--global` → `Scope: global` (**always**, even if env project_id still present) |
| | • else with project → `Scope: project=<alias-or-name> (<uuid>)` via `get_project_by_id` when available; else `Scope: project=<uuid>` |
| | • else → `Scope: project=(none)` |
| | Remove legacy `Project: …` summary line (smoke + any docs). |
| **F3 — Effective project when global** | In preflight dispatch (`main.rs`), **mirror recall** at ~2821: `let effective_project_id = if *global { None } else { *project_id };` then pass `effective_project_id` into `PreflightRunOptions` **and** into `build_preflight` / summary. Still pass `global: true`. Defense in depth with F2 (summary must not re-read env). |
| **F4 — Dual count model** | Summary prints two blocks: |
| | 1. **Vault (SQL):** under `--global` only: `Projects: N` (F7). Always: `Pinned memories: N`, `Active sessions: N`. **Omit `Projects:` line when project-scoped** (AI2 L5 — human-only surface; `Scope: project=…` already identifies the one project). |
| | 2. **In context (budget window):** lines labeled with literal **`In context`** prefix (or frozen: `In-context hotspots/decisions/constraints`) from marker scan of rendered `context.text`; `Total Word Count:` from F21. |
| | Marker counts must never read as vault totals. |
| **F5 — Active sessions SOOT** | Vault **Active sessions** from **`QueryStore::count_active_sessions(project_id: Option)`** (parameterized SQL on `session_projection` `status='active'`, optional project filter). **Do not** call `active_sessions` for the rollup line (loads turns; uses `format!` SQL — M1). **Not** `"Session ID:"` text match. Soft residual: refactor retrieval `active_sessions` to `params![]` (pre-existing debt, **not** T214 DoD). |
| **F6 — Marker scan keep** | Keep counting `HOTSPOT:`, `DECISION:`, `CONSTRAINT:` in **rendered** text after ANSI strip in body. Document LIMIT/truncation. Pathological ANSI-split markers may differ from SQL safety LIKE (AI2 L7) — no action beyond note. |
| **F7 — Projects count (global rollup) — frozen SOOT** | **`QueryStore::count_projects_with_pinned() -> Result<u64>`** = `SELECT COUNT(DISTINCT project_id) FROM memory_projection WHERE status = 'pinned' AND project_id IS NOT NULL` with **parameterized** prepare/query (no bind vars needed for this statement, but **no** `format!` interpolation pattern). **Do not** reuse `list_projects` (memory_count includes unpinned/turns — over-reports). Printed **only under `--global`**. |
| **F8 — Pinned memories count** | **`QueryStore::count_pinned_memories(project_id: Option<&ProjectId>)`** = `COUNT(*)` where `status='pinned'`; when project set, filter with **`rusqlite::params![]`** (same project join/filter spirit as preflight scope — prefer direct `m.project_id = ?` OR session join if product already scopes that way; pick one SOOT in implement tests). **Never** `format!("... '{}'", pid)`. |
| **F9 — No ledgerful under global (freeze)** | Do **not** enable `query_ledgerful` when global in this track. Multi-repo hotspots are ambiguous; document “ledgerful hotspots require project-scoped preflight”. Soft residual product track if needed. |
| **F10 — Governed + global** | No multi-project governed packet. If `AI_BRAINS_GOVERNED_BRIEFING` on + global, body may be empty warning — summary still honest Scope: global + SQL vault stats (SQL still valid). Do not use summary as governed authority (T170). |
| **F11 — JSON contract** | Non-summary path: **no** new required keys on `PreflightContextResponse`. Soft decline: `preflight --summary --format json` machine object (unless free pure + additive; default **human only** for summary). |
| **F12 — Capture independence** | Summary path must work without models/graph feature. |
| **F13 — Shared Scope SOOT (AI2)** | **v1: `pub(crate) fn format_scope_line`** in `recall.rs` (or thin re-export) — lower churn than moving 5 unit tests. Soft residual: extract to `commands/scope_display.rs`. Preflight: `ctx.conn.get_project_by_id(&pid)` then `format_scope_line(global, Some(&pid), name_alias.as_ref())` (L2). **One SOOT**, no third copy. |
| **F14 — Zero new crates** | — |
| **F15 — Exit codes** | Summary success remains **0** when preflight build succeeds. No new exit class. |
| **F16 — Determinism** | SQL counts + stable key order of printed lines; no timestamps in summary header; sort nothing new unless SQL needs ORDER BY for multi-row (counts only). |
| **F17 — High findings** | Printing env project as scope under `--global`; Active Sessions always 0 while sessions exist; presenting marker counts as vault totals without label; auto-global; growing JSON keys without track; enabling ledgerful global without decision; governed summary as authority; **new F7/F8 SQL using `format!` interpolation** (M1). |
| **F18 — Hermetic locks** | Multi-project temp vault: pin A+B; `--global --summary` → `Scope: global`, `Projects:` ≥ 2, Pinned ≥ sum, no `Project:` scope line; project-scoped → `Scope: project=` + filtered pinned, **no** `Projects:` line; Active sessions ≥ 1 with open session fixture; AC9 empty vault = **init only, no pin** then all vault counts 0; smoke AC7. |
| **F19 — Docs** | CAPABILITIES **preflight section** + **Scope row (~line 202)** must cover preflight summary (not recall-only wording) (L6). Dual model + global honesty + ledgerful project-scoped only + T170 summary ≠ governed. CHANGELOG Unreleased. Soft skill one-liner. |
| **F20 — Help / after_help** | Soft: one example `preflight --global --summary` in preflight help if free. Flag help text already accurate for content. |
| **F21 — Word count** | Print **`Total Word Count:`** from **`context.word_count`** (retrieval). SOOT: `word_budget::word_count` = `split_whitespace().count()` — same as current summary re-split; prefer single field, drop re-split in printer. Signature: pass `&PreflightContext` (M3/M5). |
| **F22 — Footer** | Keep “Use --pretty or --format json for full context.” |
| **F23 — Privacy** | Global SQL counts are aggregates only — do not dump other projects’ names in summary. Scope line under project mode may show alias/name of **active** project only. |
| **F24 — Soft residuals** | Non-summary pretty Scope header on full preflight body; summary JSON DTO; ledgerful multi-root; is-terminal→std migrate (L1); clap 4.6; two-tier “sessions in text” count; adaptive max_words for global; extract `scope_display.rs`; refactor `active_sessions` off `format!` SQL. |
| **F25 — Not T216** | Forget list inventory out of scope. |
| **F26 — Not density** | Graph density doctor is T213 closed; no coupling. |
| **F27 — Store helpers + SQL safety (M1)** | All **new** count SQL lives on **QueryStore** with `rusqlite::params![]` / bound args (see `get_project_by_id` style). **Forbidden:** copy `sessions.rs:23-31` `format!("... project_id = '{}'", pid)` into T214 code. Hermetic store tests for F7/F8/F5 counts. CLI adapter only. |
| **F28 — Review** | FEATURE + honesty bugfix; primary review required. Cross-model soft (P3). |
| **F29 — Series order** | After T213; parallel-safe with T216 planning. |
| **F30 — Implement order** | F13 `pub(crate)` Scope → pure summary formatter red tests → F27 count helpers (params!) → `print_summary` signature rewrite (M3) + F3 dispatch → hermetic → smoke (M4) → docs F19 → gate. |
| **F37 — `print_summary` signature (M3)** | Breaking internal rewrite: `print_summary(ctx: &AppContext, global: bool, project_id: Option<ProjectId>, context: &PreflightContext) -> Result<(), …>` (or map SQL errors). Drop unused `_ctx` underscore; use `ctx.conn` for counts + `get_project_by_id`. Existing unit tests only cover `normalize_scope_paths` — no expected breakage. |
| **F38 — Smoke env-precedence (M4)** | Update `smoke.rs` stdout assert `Project:` → `Scope: project=` (UUID-only `.env` → likely `Scope: project={local_project_id}` if no projection/alias). **Preserve** (2) `!stdout.contains(inherited_project_id)` and (3) stderr local `.env` override warnings. Test remains primarily about env precedence. |
| **F39 — AC6 coverage (L8)** | `protocol_compat_cli.rs` already locks 2 JSON keys without `--summary`; no new JSON test required — must stay green under full gate. |
| **F31 — Header title** | Keep `--- AI-Brains Preflight Summary ---`. |
| **F32 — Marker case** | Match existing body markers case-sensitive as today (`HOTSPOT:`, `DECISION:`, `CONSTRAINT:`). |
| **F33 — Empty vault** | Global empty: Scope global; zeros; exit 0; non-blank summary (T198 family). |
| **F34 — Stdin / scope flags** | Unchanged; summary still applies after build. |
| **F35 — Ledger TX** | On go: `ledgerful ledger start T214-preflight-global-rollup --category FEATURE`. |
| **F36 — Plan-only** | No production code until user **go**. |

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Hermetic: `--global --summary` stdout contains `Scope: global` and does **not** contain `Project: <any-uuid>` as the scope identity line. |
| **AC2** | Hermetic multi-project: vault Projects SQL ≥ 2 and Pinned ≥ pins in A+B under `--global --summary`. |
| **AC3** | Hermetic project-scoped `--summary`: `Scope: project=` includes the scoped project id (alias optional); pinned count reflects that project only; **stdout has no `Projects:` line** (F4/L5). |
| **AC4** | Fixture with one active session → Active sessions ≥ 1; implementation must not use `"Session ID:"` text match as SOOT. |
| **AC5** | In-context marker lines are labeled so they cannot be read as vault totals (string lock on **In context** / **In-context**). |
| **AC6** | `preflight --format json` still exactly keys `text` + `word_count` (existing `protocol_compat_cli` + gate). |
| **AC7** | Smoke: Scope vocabulary on stdout; env-override stderr + inherited-id absence **preserved** (F38). |
| **AC8** | Pure unit: `format_scope_line(true, …) == "Scope: global"` shared SOOT. |
| **AC9** | Empty vault (**init only, no pins**): `--global --summary` exit 0, Scope global, Projects/Pinned/Active sessions all 0, non-empty stdout. |
| **AC10** | Docs: CAPABILITIES preflight + Scope row + CHANGELOG dual model + global honesty. |
| **AC13** | Store unit: `count_projects_with_pinned` / `count_pinned_memories` / `count_active_sessions` use bound params (or static SQL without string-built identifiers); multi-project fixture matches F7/F8. |
| **AC11** | Full CI gate green; no new crates; capture-independent. |
| **AC12** | When global, env `AI_BRAINS_PROJECT_ID` still set does **not** win label (AC1 covers). |

## 5. Non-goals

- Packaging / MSI / notarization  
- clap 5 multi-heading  
- Relicensing  
- Governed multi-project briefing packet  
- Ledgerful under `--global`  
- Auto-widen empty project → global  
- Changing default max_words or full preflight ranking/budget algorithm  
- T216 forget-list  
- Promoting `GraphHealthOutput` / density into preflight  

## 6. Verification plan

| Phase | Proof |
|-------|-------|
| Red | Pure summary formatter + scope; hermetic AC1–AC4 fail on current binary |
| Green | CLI + store helpers |
| Targeted | `cargo nextest run -p ai-brains-cli -E 'test(preflight)'` + store get_project/count tests; clippy `-p ai-brains-cli` |
| Manual | Live dogfood: project summary vs `--global --summary` on main vault; confirm Scope + Projects > 1 + Active sessions plausible |
| Full gate | fmt, clippy workspace, nextest workspace, deny, audit, ledgerful verify |
| Review | `review.md`; cross-model soft |

## 7. Risks

| Risk | Mitigation |
|------|------------|
| Dual SOOT Scope format drift | F13 `pub(crate)` single SOOT |
| SQL injection pattern spread from `sessions.rs` | F5/F7/F8/F27 QueryStore + `params![]` only (M1) |
| F7 over-count via `list_projects` | Frozen pinned-only COUNT (M2) |
| Agents parse `Project:` | Smoke/docs; honesty wins |
| Marker counts drop under global | F4/F6 In context labels |
| Smoke env-precedence collateral | F38 preserve (2)/(3) |

## 8. Coordination

- **T207:** Scope line SOOT — `pub(crate)` reuse.  
- **T112:** recall dispatch template for F3.  
- **T212:** `get_project_by_id` only — no list rewrite.  
- **T180:** two-key JSON freeze.  
- **T170:** summary ≠ governed.  
- **T213 / T216:** orthogonal.  

## 9. Notes

Placeholder objective preserved: *true multi-project rollup **or** honest label* → **both** shipped (SQL rollup + Scope honesty). Plan-only until **go**.

## 14. AI fold-in disposition (2026-08-05)

| ID | Source | Disposition |
|----|--------|-------------|
| AI1 §1–§4 | Affirms Scope extract, global label, Active sessions SQL, dual counts, JSON/ledgerful/capture freezes | **Affirm** — already F2–F12; refined by AI2 |
| AI1 table 1–5 | Actionable implementation map | **Affirm** as implement checklist |
| **M1** | `active_sessions` `format!` SQL; risk of F7/F8 copy | **Accept** → F5/F7/F8/F27; soft residual refactor `active_sessions` |
| **M2** | F7 SOOT unresolved / list_projects over-count | **Accept** → freeze `count_projects_with_pinned` pinned-only |
| **M3** | `print_summary` signature must change | **Accept** → F37 |
| **M4** | Smoke env-precedence asserts to preserve | **Accept** → F38 / AC7 |
| **M5** | word_count source | **Accept** → F21 use `context.word_count` (same `split_whitespace` SOOT as `word_budget`) |
| **L1** | is-terminal redundant | **Soft residual** F24 — no DoD migrate |
| **L2** | get_project_by_id before format_scope_line | **Elevate** → F13 |
| **L3** | exact dispatch template | **Elevate** → F3 |
| **L4** | empty vault SQL returns 0 | **Note** → AC9 init-only hermetic |
| **L5** | Projects: 1 noise in project mode | **Accept** → F4 omit Projects when not global |
| **L6** | CAPABILITIES Scope row ~202 | **Elevate** → F19 |
| **L7** | ANSI marker edge case | **Note** → F6 |
| **L8** | protocol_compat covers AC6 | **Elevate** → F39 |
| F13 extract vs pub(crate) | AI2 prefers pub(crate) v1 | **Accept** — pub(crate) DoD; extract soft |

**Not folded:** governed multi-project packet; ledgerful-on-global; JSON summary DTO; clap/rusqlite bumps; density/T216 scope.
