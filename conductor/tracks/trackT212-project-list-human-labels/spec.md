# T212 — Project list human labels

- **Track ID:** T212-ProjectListHumanLabels
- **Phase:** Post-T211 skill·CLI audit follow-ups (P1)
- **Status:** 📋 **Proposed / Expanded + AI fold-in** (plan-only until go)
- **Depends on:** T76 list polish (30-col name); T89 `set-alias` shipped; UX-friendly `(no alias) — short` name; **T206** detect honesty (PR #89); T198 empty list; T210/T211 series closed or parallel
- **Blocks / feeds:** Operators/agents pick the right project without UUID walls; better `set-alias` discoverability; soft T206 detect `--json` residual stays T206 (not list)
- **Category:** FEATURE / DOCS / BUGFIX (char-safe truncate panic fix)
- **Source:** Non-destructive skill/CLI audit 2026-08-04 — **project list quality 7 / effect 8 borderline**; detect 4 (detect fixed T206; list labels remain)
- **Deferred absorbed:** deferred.md T212 placeholder; UUID-only names; set-alias prompts; list path/last-activity sketch from placeholder; live byte-slice truncate panic risk
- **Not absorbed:** Auto set-alias from git; interactive prompts; bulk rewrite of stored `Project <uuid>` names (event history); IdP; T213 graph density; clap 5; MSI; new table-drawing crates
- **Research date:** 2026-08-05 (expand + live re-scan + online)
- **AI fold-in:** 2026-08-05 — AI1 affirms F4–F11 core. AI2 **M1–M5** accepted; **L2–L3/L5** elevated; **L1/L4** soft/affirm; **L6–L7** notes. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. **Scannable list:** `project list` shows a **human label first** (alias preferred), not a wall of truncated UUID-like names.  
2. **Last activity:** show when the project last had memory activity (or project `updated_at` fallback) so dead test projects are obvious.  
3. **Path hint (when known):** show a registered repo path alias when `repository_path_alias_projection` has rows for that project — never invent a path.  
4. **Discourage UUID-only / no-alias rows:** mark them and print a **non-interactive footer** with copy-paste `project set-alias <id> <suggested>` (no TTY prompt).  
5. **Agent-friendly JSON:** `--format json` (or `--json`) for stable machine parse without scraping tables.  
6. **Display-only:** do **not** rewrite event-sourced project names; do **not** auto-issue aliases.

## 2. Live baseline (re-scan 2026-08-05)

### 2.1 Audit signal — confirmed live

| Fact | Live |
|------|------|
| Main vault project | `7d97a456-…` name `Project 7d97a456-f2f4-43ea-1f1` (truncated), **empty alias**, **8397** memories |
| Hijack residual | `.env` → `test-alias` / `441837f6-…` (T206 detects honesty; list still opaque) |
| Friendly names | Newer rows: `(no alias) — 93e74c21` (UX track) |
| Legacy names | Older: `Project <uuid-prefix>` still on disk |
| Columns today | `project_id` \| `name (alias\|UUID)` \| `alias` \| `memories` — **name and alias redundant** when alias set |
| Path / activity | **Not shown** |
| `set-alias` | Exists (T89); list does **not** nudge |
| Flags | `project list` has **no** format/json options |
| Empty vault | Header + `No projects registered. (0 projects)` exit 0 (T198) — keep |

### 2.2 Code / touch map (AI2-verified)

| Site | Role |
|------|------|
| `ai-brains-cli/src/commands/project.rs` | `list()` rewrite; pure `display_label` / relative helper / **char-safe truncate** (fix live `name[..30]` panic — M1); footer **stderr** |
| `ai-brains-cli/src/main.rs` | `ProjectCommands::List` gains **`--format human\|json`** only (no dual `--json` flag — L3) + after_help |
| `ai-brains-store/src/query_store.rs` | **`list_projects_detail`**; path via **correlated subquery** (M5); `ORDER BY memory_count DESC, project_id ASC`; also fix **`list_projects`** tie-break (L2) |
| `ai-brains-store/src/lib.rs` | `QueryStore` trait + **one** impl (`VaultConnection` only) |
| `repository_path_alias_projection` | Path hint (often empty until governed path alias) |
| `memory_projection` | `MAX(updated_at)` = last projection mutation (M4 honesty) |
| Hermetic | `tests/project_list_labels.rs`; multibyte truncate regression; update smoke friendly-name if needed |
| Docs | CAPABILITIES; CHANGELOG; last_activity semantic note |

### 2.3 Schema facts (no migration DoD)

| Table | Fields usable |
|-------|----------------|
| `project_projection` | `project_id`, `name`, `created_at`, `updated_at` |
| `project_alias_projection` | `alias`, `project_id` |
| `memory_projection` | `project_id`, `updated_at` → last activity |
| `repository_path_alias_projection` | `normalized_path`, `project_id` (may be empty for most rows) |

No new migration required for DoD (read-only joins). Path absence is honest `—` / null.

### 2.4 Deps

| Item | Pin / note |
|------|------------|
| clap | Workspace **4.5** — no bump |
| rusqlite | **0.39.0** — no bump |
| chrono | **0.4** — relative age / parse timestamps |
| Zero new crates | Required — no comfy-table / tabled; keep plain columnar println |
| Capture independence | List is store query only |

## 3. Research summary (2026-08-05)

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — human tables; `--plain` / machine format when table breaks scripts | Default human table; **JSON for agents** (F9) |
| clig — suggest next command | Footer `set-alias` with real project_id (F8) |
| Arcjet / agent CLI 2026 — humans tables, agents JSON; stable defaults | F9 default human; `--format json` |
| Heroku CLI style — humans first; tables scannable | Label-first column order (F4) |
| Existing product | T89 set-alias; T76 header; friendly `(no alias) — short`; T206 detect hints already mention set-alias |
| Event sourcing | No bulk `UPDATE name` — display layer only (F10) |
| Path aliases T151 | Signal only; never sole identity — optional column (F6) |

## 4. Frozen decisions (F1–F42)

| ID | Decision |
|----|----------|
| **F1 — Surface** | Primary: **`ai-brains project list`**. Secondary: help/after_help on `list` + `set-alias` cross-link. |
| **F2 — No auto-alias** | Never auto-run set-alias from git slug or env. Operator-explicit only. |
| **F3 — No interactive prompt** | No “type alias:” stdin. Footer + after_help only (agent-safe). |
| **F4 — Human label (M2)** | Pure `display_label(name, alias) -> String` **order**: (1) non-empty `alias` → alias; (2) name starts with `(no alias)` (case-sensitive product form) → **literal `(no alias)`** (strip baked ` — <short>` — project_id has its own column); (3) name matches non-human `Project <uuid-ish>` or equals full/short project_id → `(no alias)`; (4) else → name as-is (true human label). Manual string ops only — **no regex**. |
| **F5 — Table columns (human default)** | **`label`** \| **`project_id`** (full 36) \| **`memories`** \| **`last_activity`** \| **`path`**. Drop separate name/alias columns from default human table. Soft F24 `--verbose` raw name. |
| **F6 — Path column (M5)** | Scalar subquery, **not** plain multi-row JOIN: `(SELECT normalized_path FROM repository_path_alias_projection WHERE project_id = p.project_id ORDER BY normalized_path ASC LIMIT 1)` → path. Empty → `—` human / `null` JSON. No git. No invent. Prevents duplicate project rows / double-counted memories. |
| **F7 — Last activity (M4)** | SQL: `COALESCE(MAX(mp.updated_at), p.updated_at)`. **Semantic (document in CAPABILITIES):** last **memory-projection mutation** (pin/forget/ingest/turn upsert), **not** “last user chat message only.” Display: relative when parseable and age &lt; 365d (`just now` / `Nm` / `Nh` / `Nd` style); else `YYYY-MM-DD`; missing → `—` / null. CLI-local relative helper OK (duplicate preflight pattern — L1; no retrieval pub refactor DoD). |
| **F8 — No-alias footer (M3)** | If ≥1 empty alias: print footer to **stderr** (clig: messages stderr, data stdout). Content: unaliased count + one copy-paste using **highest-memory unaliased** project: `ai-brains project set-alias <uuid> <suggestion>`. Default suggestion `my-project`; soft F26 git slug when free. Exit **0**. Empty vault: T198 only; **no** footer. JSON: no footer (use `unaliased_count`). |
| **F9 — JSON format (L3)** | **`--format json` only** (values: `human` default, `json`). Do **not** add a separate `--json` flag (conflict surface). Shape: `{ "api_version": "1", "projects": [ { "project_id", "name", "alias", "label", "memory_count", "last_activity", "path" } ], "unaliased_count": N }`. Sort F13. Pretty-print. Path null when absent. |
| **F10 — Display only** | No rename events; no mutate `project_projection.name` for legacy rows. |
| **F11 — Keep list_projects tuple** | `list_projects() -> (id,name,alias,count)` unchanged for detect/resolve/init/set_alias. New **`list_projects_detail()`** for list UI/JSON. Only `VaultConnection` implements `QueryStore`. |
| **F12 — Typed detail row** | Struct in store or CLI; CLI-local JSON with `api_version`. Soft F27 contracts lift. |
| **F13 — Sort (L2)** | **`ORDER BY memory_count DESC, project_id ASC`** on **both** `list_projects_detail` **and** existing `list_projects` (1-token determinism fix). |
| **F14 — Truncation** | Label ~28–32 **chars**; path truncated with `…`; project_id never truncated. |
| **F15 — Empty vault** | T198: header + `No projects registered. (0 projects)`; exit 0; no footer. |
| **F16 — Active marker DoD (L5)** | Mark row matching process `AI_BRAINS_PROJECT_ID` with `*` prefix on **label** (human table only). Missing/invalid env → no star. JSON optional `"active": true` soft. |
| **F17 — set-alias after_help** | Examples: list then set-alias. |
| **F18 — Zero new crates / no clap bump** | clap 4.5; serde_json already present; plain formatting. |
| **F19 — Capture independence** | Store query only. |
| **F20 — Series** | After T211. Before T213. |
| **F21 — Hermetic (≥8)** | AC1–AC6 + AC11 multibyte no-panic + AC12 active `*` when env set (soft if env hard in hermetic). Soft path AC10. |
| **F22 — High pre-ship** | Auto-alias; interactive hang; break 4-tuple; production panic/unwrap; footer on empty; plain multi-path JOIN duplicates; footer on JSON stdout; mislabel last_activity as chat-only. |
| **F23 — Docs** | CAPABILITIES: columns, footer stderr, json, last_activity semantic (M4), path honesty; CHANGELOG minor. |
| **F24 — Soft verbose name** | `--verbose` raw registered name. |
| **F25** | (absorbed into F16 DoD) |
| **F26 — Soft git suggestion** | Footer suggestion via existing `get_git_repo_slug` when free — not blocking. |
| **F27 — Soft contracts DTO** | Lift later. |
| **F28 — Soft detect --json** | T206 residual — out of DoD. |
| **F29 — Soft rename command** | Out. |
| **F30 — Determinism** | F13 both methods; path subquery ORDER BY path ASC LIMIT 1; stable serde. |
| **F31 — Review category** | FEATURE (+ bugfix M1). Primary review. |
| **F32 — Privacy** | Paths already in vault; no privacy widen. |
| **F33 — Performance** | Single detail SQL; no N+1; correlated path subquery OK. |
| **F34 — Exit codes** | list always **0** (incl. all-unaliased). |
| **F35 — Residual map** | Auto-alias out; detect --json → deferred; name migrate out. Residuals → **deferred.md**. |
| **F36 — Char-safe truncate (M1 bugfix)** | **Required fix:** replace live `&name[..min(30,len)]` (byte slice — panic on mid-UTF-8). Use `.chars().take(n)` (preflight pattern). **AC11** multibyte at width boundary must not panic. Class: production panic fix under Rust Safety. |
| **F37 — AI1 affirm** | Label-first, last_activity, path honesty, footer, json, display-only, keep 4-tuple. |
| **F38 — JSON path null** | Prefer JSON `path: null` when absent (not `"—"`). Human uses `—`. |
| **F39 — Footer template** | Example lines (implementer may tighten): `N project(s) have no alias.` + `Example: ai-brains project set-alias <uuid> <suggestion>` on stderr. |
| **F40 — Smoke friendly name** | Keep `(no alias)` substring assert; optionally strengthen to label-first header. |
| **F41 — list_projects ORDER BY** | F13 applied to existing method (L2). |
| **F42 — No production regex** | display_label uses starts_with / equality only. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| T212 placeholder UUID-only / set-alias UX | **Absorb** F4–F9, F21 |
| Path / last activity / memory age sketch | **Absorb** F5–F7 (age as relative last_activity) |
| Live byte-slice panic risk | **Absorb** F36 + AC11 |
| T206 detect --json | **Soft F28** not DoD |
| Auto set-alias from git | **Decline** F2 |
| Migrate legacy `Project <uuid>` names | **Decline** F10 |
| Interactive set-alias wizard | **Decline** F3 |
| Dual `--json` flag | **Decline** F9 (format only) |
| T213 graph density | Out |
| ISSUES.md missing | Soft → deferred.md (L6) |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Hermetic: project with alias `acme` → human **label** contains `acme` | Hermetic |
| **AC2** | No-alias / baked `(no alias) — short` name → label is exactly **`(no alias)`** (not full baked string as sole cue) | Hermetic + unit F4 |
| **AC3** | ≥1 unaliased: **stderr** contains `project set-alias` and the unaliased project_id; table still on stdout | Hermetic |
| **AC4** | Empty vault: T198 empty line; **no** set-alias footer on stdout or stderr | Hermetic / regression |
| **AC5** | `--format json`: exit 0; `projects[]` with `project_id`, `label`, `memory_count`; `unaliased_count` correct; **no** footer text in stdout | Hermetic |
| **AC6** | Project with ≥1 memory: `last_activity` non-empty in JSON | Hermetic |
| **AC7** | `list_projects` 4-tuple callers (detect honesty etc.) still pass | Regression |
| **AC8** | CAPABILITIES + CHANGELOG (incl. last_activity semantic M4) | Grep / review |
| **AC9** | Full CI gate green; no production unwrap/expect/panic | Gate |
| **AC10** | Soft: seeded path_alias → path column / JSON `path` non-null | Soft hermetic |
| **AC11** | Multibyte truncate: label/path with CJK or em-dash near width boundary does **not** panic (M1) | Unit / hermetic |
| **AC12** | Soft: with `AI_BRAINS_PROJECT_ID` set to a listed project, human label has `*` prefix (F16) | Soft hermetic |

## 7. Non-goals

- Packaging / MSI / notarization  
- clap 5 multi-heading  
- Auto-alias or bulk name migration  
- Interactive prompts  
- Separate `--json` flag (use `--format json`)  
- Graph density (**T213**), preflight global (**T214**), semantic (**T215**), forget list (**T216**)  
- New table crates  

## 8. Risk & blast radius

| Risk | Mitigation |
|------|------------|
| Break detect/resolve tuple | F11 keep `list_projects`; new detail method |
| Multi-path JOIN duplicates | F6 correlated subquery only |
| Wide path noise | F14/F36 char-safe truncate |
| Footer breaks scripts | **stderr** F8; JSON no footer |
| last_activity misread | F7/M4 CAPABILITIES honesty |
| Friendly-name smoke | F40 still contains `(no alias)` |

## 9. Verification plan

1. Red: hermetic AC1–AC6 + unit F4/F36 fail.  
2. Green: detail SQL + helpers + list/json + list_projects ORDER BY.  
3. Regression: detect honesty, empty list, friendly name, vault key list.  
4. Manual: live list — main no-alias, footer on stderr, `*` if env matches.  
5. Full gate; review.

## 10. Manual test script (on implement)

```powershell
ai-brains project list
ai-brains project list --format json
ai-brains project set-alias --help
# optional: set alias on main project after intentional operator choice — not auto
```

Expect: label-first table on stdout; last_activity; set-alias footer on **stderr**; JSON clean; active `*` when env matches.

## 11. Out of band

- Main vault still has empty alias for AI-Brains — product does not auto-fix; footer is the nudge.  
- Path column often `—` until path aliases registered — honesty over fake cwd.  
- last_activity moves when memories are forgotten/re-pinned — documented.

## 12. Suggested order note

… → T211 closed → **T212** / T215 → T213–T214/T216.

## 14. AI fold-in disposition (2026-08-05)

| ID | Source | Action |
|----|--------|--------|
| **AI1 #1** | display_label + column order | **Affirm** F4–F5 |
| **AI1 #2** | last_activity + path detail query | **Affirm** F6–F7 (SQL shape M5; semantic M4) |
| **AI1 #3** | Non-interactive set-alias footer | **Affirm** F8 (stream → **stderr** M3) |
| **AI1 #4** | `--format json` schema | **Affirm** F9 (no dual `--json` L3) |
| **AI1 #5** | Display-only + keep 4-tuple | **Affirm** F10–F11 |
| **M1** | Live `name[..30]` panic risk | **Accept** F36 bugfix + **AC11** |
| **M2** | Baked `(no alias) — short` vs label | **Accept** F4 ordered: prefix → literal `(no alias)` |
| **M3** | Footer stdout vs stderr | **Accept** F8 **stderr** (clig) |
| **M4** | last_activity = projection mutation | **Accept** F7 documented semantic |
| **M5** | Multi-path JOIN duplicates | **Accept** F6 correlated subquery |
| **L1** | Reuse relative_timestamp | **Affirm** CLI-local duplicate OK |
| **L2** | list_projects tie-break | **Accept** F13/F41 both methods |
| **L3** | Drop `--json` dual flag | **Accept** F9 format only |
| **L4** | Git slug footer suggestion | **Soft** F26 |
| **L5** | Active `*` marker | **Accept** elevated **DoD** F16 + soft AC12 |
| **L6** | ISSUES.md missing | **Soft** deferred.md |
| **L7** | smoke friendly-name | **Affirm** F40 still passes on `(no alias)` |
