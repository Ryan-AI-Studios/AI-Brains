# T216 — Forget-list + memory inventory skim

- **Track ID:** T216-ForgetListInventory
- **Phase:** Post-T214 skill·CLI audit follow-ups (P3 residual — last of T205–T216 series)
- **Status:** ✅ **Completed** (2026-08-05) — PR #99 `1980d83`
- **Depends on:** T77 forget validation; T88 pin prints memory_id; T107 dry-run; T112 scope/`--global`; T198 empty success; T203 list LIMIT+1 / `more_available`; T204 help IA; T207 Scope line SOOT; T212 project labels (soft reuse for summary table); T214 Scope + count patterns
- **Blocks / feeds:** Operator/agent ability to **skim vault inventory** and **forgotten rows** without inventing a recall query; safer forget workflows (see-before-force); residual series T205–T216 **closes** after ship
- **Category:** FEATURE / UX / DOCS
- **Source:** Non-destructive skill/CLI audit 2026-08-04 — **forget list effect 5**
- **Deferred absorbed:** deferred.md T216 placeholder; series residual “forget-list”; optional counts by project/tag sketch from placeholder
- **Not absorbed:** Tag schema/migration; auto-forget / retention apply rewrite; CE wipe / hard delete; governed policy-gated memory discovery (legacy pin path stays open like today); daemon HTTP list routes; clap 5; MSI; rusqlite 0.40 bump; T214 soft residuals (ledgerful-on-global, summary JSON DTO)
- **Research date:** 2026-08-05 (expand + live re-scan + online clig / dep pins)
- **AI fold-in:** 2026-08-05 — AI1 **M1–M7** accepted; **L1–L6/L8** elevated or noted; **L7/L9** affirm. AI2 affirms F1/F3/F6/F10/F11/F15 core (no new criticals). Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. **Honest forget list:** `forget --list-forgotten` must not dump an unbounded wall of content with silent env-only project filter and no machine format.  
2. **Inventory skim without recall:** operators/agents can list **pinned** (and optionally forgotten) memories with stable scope, limit, and JSON — **no** query string required.  
3. **Counts orientation:** optional **summary** shows pinned/forgotten totals; under `--global`, a compact **by-project** breakdown. Soft **tag** histogram from content `TAGS:` prefix (no schema).  
4. **Capture independence:** list/count paths are SQL + pure formatters only (no models, embeddings, graph, ledgerful).  
5. **Dangerous surface honesty:** mutation paths (`forget` by id/match, restore) stay `[dangerous]`; **read list/summary** must not require force and must not append events.  
6. **Back-compat:** existing `forget --list-forgotten` remains; behavior becomes limited + scoped-honest (document BREAKING if default truncates long lists — accept with `more_available` + higher `--limit`).

## 2. Live baseline (re-scan 2026-08-05)

### 2.1 Audit signal — confirmed live

| Fact | Live |
|------|------|
| `forget --list-forgotten` | Prints `Forgotten memories:` + `  {uuid} — {80-char first line}` for **all** matching rows — **no limit** |
| Project filter | Silent: `AI_BRAINS_PROJECT_ID` if parseable → filter; else all. **No** Scope line, **no** `--global` flag |
| Dogfood (project = test-alias) | **29** forgotten rows; previews dominated by `ASSISTANT:` / `USER:` / `TAGS:` noise |
| Dogfood vault scale | Main project **~8399** memories; inventory skim via recall requires a query string (wrong tool) |
| `forget` with no args | Error envelope: “Specify a memory ID, use --match…, --list-forgotten…, or --restore…” |
| `--format` / JSON | **None** on forget list |
| `pin` | Writes `TAGS: a, b\n{content}` into content — **no** tags column |
| `project list` | Per-project **memory_count** only (all non-filtered projection rows) — not status split, not skim |
| Empty forgotten | `No forgotten memories.` exit 0 (keep T198 spirit) |
| Help | Forget is `[dangerous]`; list-forgotten is a read flag buried under mutation command |
| CAPABILITIES | Documents list-forgotten restore; no inventory skim |

### 2.2 Root cause (frozen)

```text
// forget.rs list_forgotten branch
let project_id = env::var("AI_BRAINS_PROJECT_ID").ok().and_then(parse);
let memories = conn.list_forgotten_memories(project_id)?; // unbounded Vec
// no Scope line, no limit, no JSON, no counts
// no path to list status='pinned' without lexical_search/recall query
```

Operators cannot answer “what’s in this project / what did we soft-delete?” without either recalling (needs query) or scrolling a raw forgotten dump.

### 2.3 Code / touch map

| Site | Role |
|------|------|
| `ai-brains-cli/src/commands/forget.rs` | List-forgotten → shared inventory list path; keep mutation branches |
| `ai-brains-cli/src/commands/memory.rs` (**new**) | `memory list` / `memory list --summary` human+JSON |
| `ai-brains-cli/src/commands/mod.rs` | `pub mod memory` |
| `ai-brains-cli/src/main.rs` | `Memory` subcommand + Forget flags: `--global`, `--limit`, `--format`; after_help |
| `ai-brains-cli/src/help_ia.rs` | Daily (or Operator) inventory includes `memory` |
| Soft pure helpers | `preview_line`, `parse_tags_prefix`, `format_memory_list` — unit-tested |
| `ai-brains-store` `QueryStore` + impl | Parameterized: `list_memories`, `count_memories`, `count_memories_by_project` (status filter) |
| Hermetic | `tests/memory_list_inventory.rs` (and/or forget_list) |
| Docs | CAPABILITIES forget + memory list; CHANGELOG; OPERATIONS one-liner; soft skill |

### 2.4 Schema facts (no migration DoD)

| Column / table | Use |
|----------------|-----|
| `memory_projection.status` | `'pinned'` \| `'forgotten'` |
| `memory_projection.project_id` | Scope filter (nullable; join session when needed like list_forgotten today) |
| `memory_projection.session_id` | Optional column on rows; not required for DoD list |
| `memory_projection.updated_at` | ORDER BY DESC |
| `memory_projection.content` | Preview + optional `TAGS:` prefix parse |
| `project_projection` / aliases | Summary by-project labels (soft reuse T212 `display_label` if free; else uuid prefix) |

**No new migration.** Tags remain content-prefix heuristic only.

### 2.5 Deps / pins (researched 2026-08-05)

| Item | Workspace / crates.io | Decision |
|------|----------------------|----------|
| clap | workspace **4.5**; latest **4.6.5** | **No bump** DoD |
| rusqlite | **0.39.0** SQLCipher; latest **0.40.1** | **No bump** (T213 L4 residual stays soft) |
| serde / serde_json | 1.0 | JSON list DTO local to CLI (or contracts only if shared — **prefer CLI-local** to avoid T180-style freezes on unrelated envelopes) |
| chrono | 0.4 | Parse/display `updated_at` relative optional; ISO ok |
| Zero new crates | Required — no comfy-table / regex crate (manual string ops for TAGS:) |
| Capture independence | SQL only |

### 2.6 Online / product research

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — humans first; stdout data; stderr messages; suggest next | Scope line stdout; more_available hint; footer suggests `forget --memory-id … -f` / `forget --restore` |
| clig — `--json` / machine format when tables break scripts | F9 `--format json` (T212 style: format only, no dual `--json` flag) |
| clig — saying just enough; avoid unbounded walls | **Hard limit** default + `more_available` (T203 LIMIT+1) |
| clig — noun verb consistency | Primary inventory: **`memory list`**; forget keeps mutation + list-forgotten alias |
| Thoughtworks CLI guidelines — common naming; expressive flags | `--limit`, `--global`, `--status`, `--summary`, `--tag` |
| T203 discovery lists | Reuse LIMIT+1 / `more_available` / empty non-blank |
| T207/T214 Scope vocabulary | Reuse `format_scope_line` / `Scope: global` |
| T214 M1 | **Never** `format!` SQL for ids; parameterized binds only |
| T170 / CE honesty | List does not claim purge; docs keep soft-forget ≠ CE wipe |
| AI1 M1 | Exit **2** via `governed_common::fail_usage` → `GovernedCliError` (handle_cli_result already downcasts) |
| AI1 M2 | Tag filter two-stage: SQL `LIKE 'TAGS:%'` + Rust token match |
| AI1 M3 | Reuse `clamp_list_limit` — CLI already depends on control-plane |

## 3. Frozen decisions (F1–F48)

| ID | Decision |
|----|----------|
| **F1 — Surfaces** | **Primary inventory:** `ai-brains memory list` (read-only, **not** `[dangerous]`). **Forgotten convenience:** `ai-brains forget --list-forgotten` remains and **shares** the same list backend with `status=forgotten`. Secondary: CAPABILITIES + after_help. |
| **F2 — No mutation on list/summary** | List and summary never append events, never require `--force`, never call restore/forget writers. |
| **F3 — Scope model + exit-2 plumbing (M1)** | Mirror recall/preflight: `--global` → no project filter + `Scope: global`. Else use effective project_id when parseable → project scope. **Without `--global` and without resolvable project_id → exit **2**.** **Mechanism (frozen):** call `crate::commands::governed_common::fail_usage(msg)` which returns `Err(Box::new(GovernedCliError::emitted(EXIT_USAGE, msg)))`. `GovernedResult` is already `Result<(), Box<dyn Error>>`, so `memory::run` / list path of `forget::run` keep that signature. `handle_cli_result` (`main.rs`) **already** `downcast_ref::<GovernedCliError>` and `exit(g.exit_code)`. **Do not** invent a third usage-error type. Hint text must mention `ai-brains context` / set `AI_BRAINS_PROJECT_ID` **or** `--global`. |
| **F4 — Effective project (L2)** | When not global, project_id from **clap dispatch** (`#[arg(long, env = "AI_BRAINS_PROJECT_ID")]` pattern used by recall/preflight) passed into `memory::run` / `forget::run` as `Option<ProjectId>`. **Do not** re-read `std::env::var` inside list path (today’s forget list does — migrate list path off raw env). Do **not** invent project from git. Mutation forget paths may keep existing env reads if not touched; prefer one SOOT when signature already changes (F28). |
| **F5 — Status filter** | `memory list --status pinned\|forgotten` default **`pinned`**. `forget --list-forgotten` ≡ `memory list --status forgotten` (+ same flags where wired). Invalid status → exit **2** via `fail_usage` (F3). |
| **F6 — Limit (M3)** | `--limit` / `-l` optional. **Reuse** `ai_brains_control_plane::{clamp_list_limit, DEFAULT_LIST_LIMIT, MAX_LIST_LIMIT}` (50 / 200) — CLI **already** depends on control-plane (`Cargo.toml`); **do not** copy constants. Query `LIMIT n+1`; set `more_available` if extra row. |
| **F7 — ORDER BY** | `updated_at DESC, memory_id ASC` (deterministic). |
| **F8 — Human table columns (M7/L8)** | `memory_id` (full 36) \| `updated` (relative when free; else `YYYY-MM-DD` / raw) \| `preview`. Under `--global` add **`project`** column: `display_label` from `commands/project.rs` (**already `pub(crate)`** — call directly; no extract). **Truncate label to 20 chars** with `…` (`PROJECT_COL_MAX = 20`) for table scan; full `project_id` always in JSON. No full content dump. |
| **F9 — Preview (M6)** | Pure `preview_line(content, max_chars=80)`: first non-empty line; **always** strip leading role prefix when line starts with `USER:` / `ASSISTANT:` / `SYSTEM:` (case-sensitive token + whitespace); then char-safe truncate. **No flag to disable.** Display only — stored content unchanged. |
| **F10 — JSON format** | `--format human\|json` only (default human). No separate `--json` flag. Shape: `{ "api_version": "1", "scope": "global"\|"project", "project_id": string\|null, "status": "pinned"\|"forgotten", "items": [ { "memory_id", "preview", "updated_at", "project_id" } ], "returned": N, "more_available": bool, "limit": N, "total": N }`. `total` = SQL COUNT for filter (not just page length). Pretty-print. |
| **F11 — Summary mode (M5)** | `memory list --summary` is a **mode switch** (not clap `conflicts_with` on other flags). Prints Scope + **always both** `Pinned: N` + `Forgotten: N` for scope. Under `--global` only: table `label | project_id | pinned | forgotten` (projects with either count > 0), ordered by `(pinned+forgotten) DESC, project_id ASC`. Exit 0. JSON summary: `{ api_version, scope, project_id, pinned, forgotten, by_project?: [...] }`. **Flag interactions:** `--limit` **ignored** (summary not paginated); `--tag` **applies** to counts (only tag-matching rows counted — same two-stage rules as F12 when status filter N/A for dual counts: apply tag filter within each status count); `--status` **ignored** (both pinned + forgotten always shown). Document in after_help. |
| **F12 — Tag filter two-stage (M2)** | `--tag <name>` (non-empty; empty → exit **2**): **(1) SQL pre-filter:** `content LIKE 'TAGS:%'` (**anchored at start** — pin writes `TAGS:` as first line; **never** `LIKE '%TAGS:%'` which false-matches body text). **(2) Post-fetch Rust:** parse first line after `TAGS:`, split on comma, trim, **case-insensitive exact token** match against `--tag` (so `TAGS: foo, bar` matches `foo`; `TAGS: foobar` does **not** match `foo`). May over-fetch before token filter when limit set — apply token filter then re-page, or fetch larger candidate cap then truncate to limit (document: prefer fetch `LIMIT max(limit*4, 50)+1` candidates with SQL prefix then filter in Rust, then take `limit+1` for more_available). **Honesty:** tags are not a first-class column. |
| **F13 — Tag counts (summary soft)** | Under `--summary`, optional **Top tags** section: scan **up to 2000** most-recent pinned rows in scope for `TAGS:` prefixes; emit top **10** by frequency. Cap work; document best-effort. **DoD:** F11 project counts hard; tag histogram **F24 soft** unless pure+hermetic free. Prefer ship F11 first. |
| **F14 — Empty states** | Zero rows: human non-blank (`No pinned memories.` / `No forgotten memories.` / summary zeros). Exit **0**. |
| **F15 — Share store API (L3)** | `QueryStore::list_memories(filter) -> Vec<MemoryListRow>`; `count_memories(filter) -> u64`; `count_memories_by_project() -> Vec<(project_id, pinned, forgotten)>` (global summary). Also add **`count_forgotten_memories(project_id: Option<&ProjectId>)`** mirroring existing `count_pinned_memories` (T214). **Parameterized SQL only** — follow existing `list_forgotten_memories` **`(sql, params): (String, Vec<String>)` + `param_refs`** SOOT for variable-length optional binds (project/tag/limit). Fixed-arity helpers may use `rusqlite::params![]`. **Never** `format!` id interpolation. Extend or replace `list_forgotten_memories` to call shared list (thin wrapper OK). |
| **F16 — Project filter SQL** | Same honesty as today’s forgotten list: match `mp.project_id = ? OR sp.project_id = ?` with LEFT JOIN session when project-scoped. Global: no project predicate. |
| **F17 — help_ia (M4)** | Add `memory` under **Daily** group inventory. **Must update both** `ROOT_AFTER_LONG_HELP` Daily line (`help_ia.rs` ~L11) **and** the exact-string assertion in `root_after_long_help__contains_setup_and_stop_session` (~L55–59). Suggested Daily line: `recall, preflight, doctor, project, pin, memory, context, stop-session, daemon`. Forget after_help cross-link `memory list` / `memory list --status forgotten`. |
| **F18 — Zero new crates** | — |
| **F19 — Exit codes** | Success 0; usage/unknown status/missing scope **2** via F3; store errors keep existing command-fail path (typically 1 / envelope). |
| **F20 — Series close** | After T216 ship, T205–T216 audit series residual for forget-list is **closed**. |
| **F21 — Capture independence** | List/summary never open embedding/graph/ledgerful. |
| **F22 — No daemon / contracts growth (L6)** | Prefer CLI-local JSON in `commands/memory.rs`. **No** `protocol_compat_cli.rs` freeze test (T180 only freezes contracts DTOs). If promoted to `ai-brains-contracts` later, add freeze then. |
| **F23 — Soft forget honesty** | Docs one-liner: list/restore are not CE wipe / not NIST Purge. |
| **F24 — Soft residuals** | Tag histogram if not free; relative `updated` helper extract share; `--offset` cursor; man pages; clap 5; governed memory list; HTTP routes; whole-crate `is-terminal` → `std::io::IsTerminal` (L7 — keep crate dep for consistency in new code). |
| **F25 — Not in track** | Auto-forget; retention apply; hard delete; tag migration; rewrite pin tags storage; T214 ledgerful-on-global. |
| **F26 — Preview max (L1)** | List/inventory `preview_line` = **80** chars. `forget.rs` `truncate_preview` (100) for `--match` dry-run is a **different** function — **do not unify**. |
| **F27 — Determinism** | Sort keys F7 / summary F11; tests normalize timestamps when needed. |
| **F28 — forget flag wiring** | `forget --list-forgotten` gains `--global`, `--limit`/`-l`, `--format`, optional `--tag` (same as memory list). `--summary` **not** required on forget (use `memory list --summary`). Signature accepts clap-passed `project_id` / flags (F4). |
| **F29 — display_order** | `memory` near `pin` / Daily (e.g. 18–22); keep forget dangerous order. |
| **F30 — Restore / match / dry-run** | Unchanged behavior except docs cross-links. |
| **F31 — Multibyte** | Char-safe truncate (T212 M1 lesson) — no byte-slice panic. |
| **F32 — Scope line SOOT** | Reuse `pub(crate) format_scope_line` from recall (T214). |
| **F33 — total count** | Always compute SQL `COUNT(*)` for filter in list JSON; human list footer: `Showing N of T` when T > N or more_available. |
| **F34 — Privacy** | List shows content previews as stored (operator vault access). No extra redaction layer in T216. |
| **F35 — Tests naming** | `function_or_feature__condition__expected_result`; hermetic tempfile vault; no bare `set_var`. |
| **F36 — Docs + next-step stderr (L5)** | CAPABILITIES “Memory inventory (T216)”; CHANGELOG; OPERATIONS; skill optional. Human list (non-JSON): after table, **stderr** one-liner (clig suggest-next): `Use ai-brains forget --memory-id <id> -f to forget, or ai-brains forget --restore <id> for forgotten rows.` (status-aware wording OK). **Not** on stdout (pipe-clean). Skip on empty list / JSON / `--summary`. |
| **F37 — list_forgotten_memories** | Implement via shared filter; deprecate unbounded call in production CLI (tests may still use limit large). |
| **F38 — Global without projects (L4)** | Summary by_project empty array / “No projects with memories.” **by_project includes only projects with pinned > 0 OR forgotten > 0**; **turn-only projects excluded** — use `project list` for those. Document in CAPABILITIES. |
| **F39 — BREAKING note** | Default limit 50 truncates previous unbounded list-forgotten — document in CHANGELOG as intentional honesty. |
| **F40 — Parallel work** | Orthogonal to T214 ship residuals; no shared ledger TX with other tracks. |
| **F41 — AC10 tag cases (M2)** | Hermetic must prove: `TAGS: foo, bar` matches `--tag foo`; `TAGS: foobar` does **not**; body text containing `TAGS:` mid-content without prefix **does not** match via SQL anchor. |
| **F42 — count_forgotten_memories** | Explicit QueryStore method parallel to `count_pinned_memories` (reuse in summary path). |
| **F43 — Candidate over-fetch for tag** | When `--tag` set, SQL `LIKE 'TAGS:%'` + project/status + elevated limit, then Rust token filter, then apply page limit + more_available on filtered set (F12). |
| **F44 — fail_usage messages** | Stable English strings for hermetic asserts (scope missing; invalid status; empty tag). |
| **F45 — AI2 affirm** | AI2 summary table (store methods, memory list, scope exit 2, forget delegate, limit 50, hermetic) matches F1–F15 — no delta. |
| **F46 — Summary + tag dual counts** | When `--summary --tag X`: `Pinned` = count pinned matching tag; `Forgotten` = count forgotten matching tag; by_project under global uses same tag filter per cell if free, else residual note. |
| **F47 — No clap conflicts_with on summary** | Avoid brittle conflicts_with matrix; document ignore semantics (F11). |
| **F48 — Fold-in freeze date** | 2026-08-05 AI review file. |

## 4. Acceptance criteria

| ID | Criterion | Proof |
|----|-----------|--------|
| **AC1** | `memory list` project-scoped default status pinned: Scope line + ≤limit rows + exit 0 | Hermetic |
| **AC2** | `memory list --status forgotten` matches forget list backend | Hermetic / unit |
| **AC3** | `forget --list-forgotten` with >50 forgotten: returns 50 + more_available / Showing N of T | Hermetic |
| **AC4** | `--global` lists across projects; Scope: global; no fail_usage | Hermetic multi-project |
| **AC5** | Missing project + not global → exit **2** with hint | Hermetic |
| **AC6** | `--format json` schema keys present; `total`/`more_available` correct | Hermetic |
| **AC7** | Empty vault / empty filter: non-blank message, exit 0 | Hermetic |
| **AC8** | `memory list --summary` pinned/forgotten counts match SQL seed | Hermetic |
| **AC9** | `--global --summary` by_project rows ordered; zeros omitted or shown honestly | Hermetic |
| **AC10** | Tag filter (F12/F41): `TAGS: foo, bar` matches `--tag foo`; `TAGS: foobar` does **not**; body-only mid-content `TAGS:` does **not** match; unknown tag → empty success exit 0 | Hermetic + unit |
| **AC11** | Multibyte preview truncate no panic; role prefix always stripped (F9) | Unit |
| **AC12** | List/summary append **0** events | Hermetic / unit |
| **AC13** | help_ia Daily includes `memory`; **const + exact test string** both updated (F17/M4); forget help mentions inventory | Unit / hermetic help |
| **AC14** | CAPABILITIES + CHANGELOG + OPERATIONS touch | Doc review |
| **AC15** | Full CI gate green | CI |
| **AC16** | Store SQL uses binds / no format! id interpolation; list follows `(sql, params)` SOOT when variable | Code review + unit |
| **AC17** | Invalid `--status` → exit **2** via `GovernedCliError` / `fail_usage` (not plain boxed exit 1) | Hermetic |
| **AC18** | Missing project + not global → exit **2** (process exit code, not only message) | Hermetic |
| **AC19** | `--summary --limit` ignored; `--summary` always prints both Pinned + Forgotten; `--summary --tag` filters counts | Hermetic / unit |
| **AC20** | Global human project column ≤20 chars + `…` when longer (F8) | Unit |

## 5. Non-goals

- Tag column / migration / pin rewrite  
- Auto-forget, retention apply UX rewrite  
- CE wipe / hard delete / NIST Purge claims  
- Governed policy gate on legacy memory list  
- Daemon/HTTP inventory routes  
- clap 5 multi-heading; man pages  
- MSI / packaging  
- rusqlite 0.40  
- Semantic/ANN inventory  
- T214 soft residuals  

## 6. Risk & verification

| Risk | Mitigation |
|------|------------|
| Unbounded COUNT on huge vaults | COUNT is indexed-friendly on status/project; still O(n) — acceptable; limit materialize page only |
| Silent whole-vault dump | F3 fail_usage without project/global |
| Exit 2 claimed but exit 1 shipped (M1) | **fail_usage + AC17/AC18 process exit assert** |
| Tag LIKE mid-body false match (M2) | Anchored `TAGS:%` + token match; AC10/F41 |
| Tag + limit under-fetch | F43 over-fetch then filter |
| BREAKING list truncation | F39 CHANGELOG; `--limit 200` max |
| help_ia exact-string CI break (M4) | F17 update const **and** test |
| format! SQL regression | F15/F16/AC16; `(sql, params)` SOOT |

**Implement order on go:** store list/count + count_forgotten (red) → pure preview/tags (F9/F12) → `memory list` CLI with fail_usage → wire forget list → help_ia const+test → hermetic AC1–AC20 → docs → gate.

**Manual dogfood:** live vault `memory list --limit 5`; `memory list --summary --global`; `forget --list-forgotten --limit 5`; confirm Scope honesty under test-alias env; `$LASTEXITCODE` on missing scope = 2.

## 7. Residual after ship

| Residual | Disposition |
|----------|-------------|
| Tag histogram (if not shipped) | Soft F24 |
| `--offset` / cursor pagination | Soft |
| Shared relative-time helper extract | Soft |
| Governed memory discovery | Future track |
| HTTP list routes | Future |
| Structured tags table | Product decision / future |
| T214 soft residuals | Stay on T214 closeout |

## 8. Series context

Post-T204 non-destructive audit **T205–T216**. Suggested order completed through T214/T215; **T216 is the last series residual**. Closing T216 closes “forget list effect 5” and the series placeholder row in deferred.md.

## 9. Implementation notes

### 9.1 Filter struct (sketch)

```rust
pub struct MemoryListFilter {
    pub status: MemoryStatus, // Pinned | Forgotten
    pub project_id: Option<ProjectId>, // None = global
    pub tag: Option<String>, // SQL LIKE 'TAGS:%' + Rust token match (F12)
    pub limit: usize, // clamp_list_limit; query limit+1 (or elevated candidate for tag)
}
```

### 9.2 Exit-2 usage path (M1)

```rust
use crate::commands::governed_common::fail_usage;
// missing scope / invalid status / empty tag:
return fail_usage("…hint…");
// handle_cli_result downcasts GovernedCliError → process::exit(2)
```

### 9.3 Example human output (list)

```text
Scope: project=test-alias (441837f6-…)
status=pinned  limit=5
memory_id                            updated  preview
aaaaaaaa-bbbb-…                      3h       DECISION: …
…
Showing 5 of 576  (more available; raise --limit)
```
(stderr) `Use ai-brains forget --memory-id <id> -f to forget, …`

### 9.4 Example summary global

```text
Scope: global
Pinned: 9600
Forgotten: 40
label       project_id                           pinned  forgotten
(no alias)  7d97a456-…                             8399         10
test-alias  441837f6-…                              576         29
…
```

## 10. Definition of Done

- [ ] F1–F48 decisions respected (soft F24 items documented if deferred)
- [ ] AC1–AC20 met or explicitly residual with justification ≤3 mediums
- [ ] Review log clean for critical/high; mediums fixed or deferred per AGENTS
- [ ] Full gate: fmt, clippy -D warnings, nextest workspace, deny, audit
- [ ] `ledgerful verify` clean after ledger commit
- [ ] conductor + deferred updated; pin closeout decision
- [ ] No production `unwrap`/`expect` in new code

## 14. AI fold-in disposition (2026-08-05)

| ID | Severity | Disposition | Spec/plan landing |
|----|----------|-------------|-------------------|
| **AI1 M1** | Critical | **Accept** | F3, F19, F44, AC17/AC18, §9.2 — `fail_usage` / `GovernedCliError`; no new error type |
| **AI1 M2** | Critical | **Accept** | F12, F41, F43, AC10 — anchored `LIKE 'TAGS:%'` + Rust exact token; not mid-body |
| **AI1 M3** | Medium | **Accept** | F6 — reuse `clamp_list_limit` + DEFAULT/MAX; drop stale “avoid control-plane” caution |
| **AI1 M4** | Medium | **Accept** | F17, AC13 — update `ROOT_AFTER_LONG_HELP` **and** exact test string |
| **AI1 M5** | Medium | **Accept** | F11, F46, F47, AC19 — summary mode: limit ignored; status ignored; tag filters counts |
| **AI1 M6** | Medium | **Accept** | F9, AC11 — **always** strip USER/ASSISTANT/SYSTEM prefix; no disable flag |
| **AI1 M7** | Medium | **Accept** | F8, AC20 — `display_label` pub(crate); `PROJECT_COL_MAX=20` |
| **AI1 L1** | Low | **Accept** | F26 — do **not** unify 80-char list vs 100-char match-preview |
| **AI1 L2** | Low | **Accept** | F4, F28 — clap env / dispatch `project_id`; no raw `env::var` on list path |
| **AI1 L3** | Low | **Accept** | F15 — `(sql, params) Vec` SOOT from `list_forgotten_memories` |
| **AI1 L4** | Low | **Accept** | F38 — turn-only projects excluded from by_project; doc `project list` |
| **AI1 L5** | Low | **Accept** | F36 — stderr next-step suggest; not stdout |
| **AI1 L6** | Low | **Accept** | F22 — CLI-local JSON; no protocol_compat freeze |
| **AI1 L7** | Low | **Note** | F24 soft — keep `is-terminal` crate for crate-wide consistency |
| **AI1 L8** | Low | **Accept** | F8 — call `display_label` directly |
| **AI1 L9** | Low | **Affirm** | clap 4.5 / rusqlite 0.39 no bump |
| **AI2** | — | **Affirm** | F45 — store methods, memory list, scope exit 2, forget delegate, limit, tests |

**Not folded:** inventing UsageError separate from GovernedCliError (M1 option b already is GovernedCliError — use fail_usage); clap `conflicts_with` matrix for summary (prefer ignore semantics F47); forcing tag histogram DoD (stays F13/F24 soft).
