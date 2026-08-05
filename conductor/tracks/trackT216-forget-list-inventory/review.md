# T216 Review Log
## Round 1 — Internal
Date: 2026-08-05
Reviewer: subagent internal

### Scope reviewed
- Spec: `conductor/tracks/trackT216-forget-list-inventory/spec.md` (F1–F48, AC1–AC20, DoD)
- Plan: `conductor/tracks/trackT216-forget-list-inventory/plan.md`
- Store: `crates/ai-brains-store/src/lib.rs`, `query_store.rs`, `tests/memory_list_inventory.rs`
- CLI: `commands/memory.rs`, `commands/forget.rs`, `main.rs` (Memory + Forget clap), `help_ia.rs`
- Hermetic: `crates/ai-brains-cli/tests/memory_list_inventory.rs`
- Docs: `Docs/CAPABILITIES.md`, `CHANGELOG.md`, `Docs/OPERATIONS.md` (+ spot-check `Docs/WORKFLOWS.md`)

### Findings
| ID | Severity | Description | Files | Required fix | Status |
|----|----------|-------------|-------|--------------|--------|
| T216-R1-01 | medium | **F46 / AC19 incomplete for global summary + tag:** `run_summary` tag-filters top-line `Pinned`/`Forgotten` via `count_memories`, but `build_by_project_rows` always calls unfiltered `count_memories_by_project()`. Under `--summary --global --tag X`, totals and by-project cells disagree (table shows full vault counts). Spec F46: “by_project under global uses same tag filter per cell **if free, else residual note**.” Implementation does neither filter nor residual note; CAPABILITIES Summary row also omits this interaction. | `crates/ai-brains-cli/src/commands/memory.rs` (`run_summary` / `build_by_project_rows`); `Docs/CAPABILITIES.md` Summary row | Either (a) apply the same two-stage tag filter per project cell in by_project when `--tag` is set, **or** (b) omit/null by_project when tag is set **or** print an explicit residual line + doc note that by_project is unfiltered under `--tag`. Add hermetic assert for chosen behavior. | open |
| T216-R1-02 | low_info | **WORKFLOWS.md still documents pre-T216 list-forgotten shape** (`memory_id`, `forgotten_at`, one-line excerpt; match status `active`/`forgotten`). Out of AC14 DoD (CAPABILITIES/CHANGELOG/OPERATIONS only) but operator-facing drift. | `Docs/WORKFLOWS.md` § soft-delete | Align list-forgotten description with Scope + bounded table + restore ≠ CE wipe; optional pointer to `memory list`. | open |
| T216-R1-03 | low_info | **AC3 proof uses `--limit 3` with 6 forgotten**, not default limit 50 with >50 rows. Mechanism (LIMIT+1 / Showing N of T / more_available) is covered; default-50 truncation is only documented + unit/store path, not hermetically at 50. | `crates/ai-brains-cli/tests/memory_list_inventory.rs` | Optional: hermetic with 51 forgotten and default limit (or assert `limit: 50` in JSON without explicit `--limit`). Not blocking if gate cost is a concern. | open |
| T216-R1-04 | low_info | **AC12 “0 events” is only stability of successive JSON list totals**, not an event-log / sequence counter assert. Code review: `memory.rs` never opens `EventStore` / append path — AC12 holds by construction. | `crates/ai-brains-cli/tests/memory_list_inventory.rs` | Optional strengthen with event-count probe if a cheap store API exists; otherwise leave (code-review sufficient). | open |
| T216-R1-05 | low_info | **Tag page under-fetch (F43 design):** SQL candidate cap `max(limit*4, 50)+1` then Rust token filter can return fewer than `limit` rows while `count_memories` total is higher (sparse token density among `TAGS:` rows). Footer can claim “more available” without a complete next page under raised `--limit`. Spec accepts over-fetch; user docs do not mention the residual. | `crates/ai-brains-cli/src/commands/memory.rs`; CAPABILITIES Tags row | Document residual in CAPABILITIES Tags row (or ship denser scan later). No spec violation. | open |
| T216-R1-06 | low_info | **Duplicated tag token matcher** (`content_has_tag` in CLI vs `content_has_tag_token` in store). Currently equivalent (incl. role-prefix strip); future drift risks list vs count mismatch. | `memory.rs`, `query_store.rs` | Soft extract shared pure helper (e.g. core or store pub fn) when convenient. | open |
| T216-R1-07 | low_info | **F36 stderr next-step is fixed**, not status-aware (always mentions both forget and restore). Spec allows status-aware wording; fixed string is OK. | `memory.rs` `emit_list_human` | Optional: forgotten list tip leads with restore. | open |

### AC / F checklist (reviewer notes)

| Item | Result | Notes |
|------|--------|-------|
| F1 surfaces (`memory list` + forget alias) | **met** | Shared `run_inventory`; Memory not `[dangerous]` |
| F2 no mutation on list/summary | **met** | No `EventStore` / force on inventory path |
| F3 exit-2 via `fail_usage` / `GovernedCliError` | **met** | Scope missing, invalid status, empty tag; hermetic AC5/AC17/AC18 |
| F4 clap-passed project_id; no `env::var` on list | **met** | `#[arg(env = "AI_BRAINS_PROJECT_ID")]`; mutation forget may still env-read (allowed) |
| F5 status default pinned; forget ≡ forgotten | **met** | |
| F6 `clamp_list_limit` 50/200 + limit+1 | **met** | control-plane reuse |
| F7 ORDER BY updated_at DESC, memory_id ASC | **met** | store SQL + unit order assert |
| F8 PROJECT_COL_MAX=20 + `display_label` | **met** | unit AC20 |
| F9 always strip USER/ASSISTANT/SYSTEM; 80 chars | **met** | units + hermetic pin preview |
| F10 JSON shape | **met** | hermetic AC6 (list); summary JSON untested hermetically |
| F11 summary mode / ignore limit+status | **met** | code + AC19 limit; status ignore by early branch |
| F12/F41/F43 two-stage tag | **met** | SQL anchored (+ role variants); Rust exact token; hermetic AC10 |
| F14 empty non-blank exit 0 | **met** | |
| F15 store API + `(sql, params)` SOOT | **met** | no id `format!` interpolation |
| F16 project SQL join honesty | **met** | |
| F17 help_ia const **and** exact test | **met** | Daily includes `memory` |
| F19/F44 exit codes + stable msgs | **met** | |
| F21 capture independence | **met** | SQL + formatters only on list path |
| F22 CLI-local JSON | **met** | |
| F26 preview 80 vs match 100 separate | **met** | |
| F28 forget flag wiring | **met** | global/limit/format/tag/project_id |
| F32 `format_scope_line` SOOT | **met** | |
| F33 total COUNT + Showing footer | **met** | |
| F36 CAPABILITIES + CHANGELOG + OPERATIONS | **met** | skill soft skipped; WORKFLOWS stale (R1-02) |
| F37 unbounded CLI dump removed | **met** | CLI no longer calls `list_forgotten_memories` |
| F38 by_project zeros / turn-only excluded | **met** | store test |
| F39 BREAKING default 50 | **met** | CHANGELOG Changed |
| F42 `count_forgotten_memories` | **met** | |
| F46 summary+tag dual counts | **partial** | top-line yes; by_project no → R1-01 |
| F47 no conflicts_with on summary | **met** | |
| AC1–AC20 | **mostly met** | gaps: R1-01 (AC19 global+tag), soft test gaps R1-03/R1-04 |
| Production unwrap/expect/TODO in new list code | **clean** | |
| Forget mutation regression | **no evidence** | match/restore/id-forget branches intact; list early-return |
| Spec drift (exit 1 instead of 2; unbounded list; env on list) | **no** | |

### Soft residuals (spec F24 — not findings against DoD)
- Tag histogram (F13/F24)
- `--offset` / cursor pagination
- Shared relative-time helper extract
- Manual dogfood + `$LASTEXITCODE` (plan Phase 6 unchecked)
- Full CI gate / ledger commit (plan Phase 7 unchecked)

### Verdict
**NEEDS_FIX**

One **medium** honesty gap (global summary by_project vs `--tag`, F46) must be fixed or explicitly residual-documented with CAPABILITIES note before clearance. No critical/high findings. Core inventory surface (store API, `memory list`, bounded forget list, exit-2, tag two-stage, help_ia, primary docs) is substantially complete.

## Round 2 — Internal (post R1 fix)
Date: 2026-08-05
Reviewer: subagent internal (re-review)

### Scope reviewed
- R1 fix for **T216-R1-01**: `build_by_project_rows` tag path, CAPABILITIES Summary/Tags, hermetic `memory_list__global_summary_tag__by_project_matches_totals`
- Spot re-check: `run_summary` / non-tag by_project path, store `count_memories` two-stage tag, WORKFLOWS soft-delete (R1-02), CAPABILITIES Tags sparse residual (R1-05)
- Regression scan: list path early-return, fail_usage exit-2, forget list share, no new mutation hooks

### T216-R1-01 verification (medium → fixed)

| Check | Result | Evidence |
|-------|--------|----------|
| Optional tag on `build_by_project_rows` | **met** | `memory.rs` `build_by_project_rows(ctx, tag: Option<&str>)`; `run_summary` passes `tag.as_deref()` only under `--global` |
| Same two-stage filter as totals | **met** | Tag branch re-counts each project via `count_memories(&MemoryListFilter { status, project_id: Some(pid), tag: Some(...), limit: 0 })` — same store path as top-line Pinned/Forgotten (SQL `TAGS:%` / role-prefix + Rust exact token in store) |
| Zero rows omitted after filter | **met** | `if pinned > 0 \|\| forgotten > 0` before push |
| Order `(p+f) DESC, project_id ASC` | **met** | Explicit `sort_by` on filtered vec; non-tag path still uses store SQL `ORDER BY (pinned + forgotten) DESC, project_id ASC` |
| CAPABILITIES F46 honesty | **met** | Summary row: “With `--tag`, top-line **and** by-project cells use the same two-stage tag filter (F46).” |
| Hermetic proof | **met** | `memory_list__global_summary_tag__by_project_matches_totals`: two projects (A=1 arch, B=1 arch + 1 untagged); JSON `pinned==2`; B cell `pinned==1` (not vault 2); sum(by_project.pinned)==top-line; zero cells forbidden |
| Non-tag path unchanged | **met** | `else { count_memories_by_project()? }` when `tag` is `None` |

**Status: verified_fixed**

### Other R1 findings (re-check)

| ID | Severity | Round 2 status | Notes |
|----|----------|----------------|-------|
| T216-R1-02 | low_info | **fixed** | WORKFLOWS §3 soft-delete: Scope-honest bounded list, `memory list` pointer, soft-forget ≠ CE wipe / NIST Purge |
| T216-R1-03 | low_info | **open / deferred** | AC3 still uses limit 3; mechanism covered |
| T216-R1-04 | low_info | **open / deferred** | AC12 by construction (no EventStore on inventory path) |
| T216-R1-05 | low_info | **fixed (docs)** | CAPABILITIES Tags row notes sparse under-fill / raise `--limit` |
| T216-R1-06 | low_info | **open / deferred** | Dual `content_has_tag` / `content_has_tag_token` still present; still equivalent |
| T216-R1-07 | low_info | **open / deferred** | Fixed stderr tip still OK per F36 |

### Findings (Round 2)
| ID | Severity | Description | Files | Required fix | Status |
|----|----------|-------------|-------|--------------|--------|
| — | — | No new critical / high / medium findings. | — | — | — |

### Soft residual noted (not medium+)
| Note | Severity | Disposition |
|------|----------|-------------|
| Tag-filtered global summary walks every project from `count_memories_by_project` and runs 2× `count_memories` (full candidate content scan per status). Correct and honest; can be O(projects × tagged-candidates) on huge multi-project vaults. | low_info | Accept for inventory skim; optional later SQL/group-by tag residual (out of F46 fix scope). |

### F46 / AC19 checklist
| Item | Result |
|------|--------|
| F46 top-line dual counts with `--tag` | **met** (unchanged from R1) |
| F46 by_project same tag filter per cell under global | **met** (R1-01 fixed) |
| AC19 `--summary --tag` filters counts (incl. global by_project) | **met** |
| CAPABILITIES documents interaction | **met** |

### Regression scan (fix path)
| Surface | Result |
|---------|--------|
| List pagination / LIMIT+1 / fail_usage | **no regression** observed in code path |
| Summary without `--tag` | still `count_memories_by_project` only |
| Forget mutation branches | inventory still early-return; no EventStore on list/summary |
| Production unwrap/expect in fix | **none** (`ProjectId::from_str` → `continue`; store `?`) |

### Verdict
**CLEAN**

Medium T216-R1-01 is **verified_fixed** (code + hermetic + CAPABILITIES). R1 soft lows R1-03/R1-04/R1-06/R1-07 remain open/deferred (allowed). No new medium+ regressions from the fix. Track may proceed to final verification / closeout when implementer gates and ledger work are done (out of this re-review scope; not marking Completed).

## Round 3 — Cross-model (Claude; Codex rate-limited)
Date: 2026-08-05
Reviewer: Claude Sonnet 4.5 (codex-review skill; Codex usage limited)

### Verdict
**PASS** — 0 P0/P1/P2; no new deferrals beyond spec F24 soft residuals.
Raw: `review.claude.md`

### Gates observed by orchestrator
- cargo fmt --check: ok
- cargo clippy --workspace --all-targets -- -D warnings: ok
- cargo nextest run --workspace: **2217 passed** (1 skipped)
- cargo deny check: ok
- cargo audit: 19 allowed warnings only (pre-existing)
- Manual dogfood: `memory list --limit 5` Scope project + Showing 5 of 553; `memory list --summary` Pinned 553 Forgotten 29; `forget --list-forgotten --limit 5` Showing 5 of 29; `--summary --global` Pinned 15775; missing scope exit 2

### Soft residuals (F24 / R1 lows — not blocking)
- Tag histogram under summary
- --offset cursor
- Shared relative-time helper extract
- Tag matcher CLI/store duplicate (R1-06)
- AC3 default-50 hermetic at scale (mechanism proven at limit 3)
- Governed / HTTP inventory (out of scope)


## Closeout
Date: 2026-08-05
PR: #99 squash-merged `1980d83`
CI: gate-windows / gate-linux / gate-macos SUCCESS
Full local gate: 2217 nextest; clippy -D warnings; deny ok; audit allowed warnings only
Cross-model: Claude **PASS** (Codex rate-limited)
Internal: Round2 CLEAN (R1-01 F46 verified_fixed)
conductor/deferred/coordinated updated; series T205–T216 closed
Soft residuals: F24 tag histogram, --offset, relative-time helper, tag matcher dual
