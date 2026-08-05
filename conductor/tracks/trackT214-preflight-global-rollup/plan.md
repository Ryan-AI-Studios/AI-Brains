# T214 Plan — Preflight global rollup honesty

Status: **Planning** (expanded + **AI fold-in 2026-08-05**; plan-only until **go**). Spec: [spec.md](./spec.md).

## Goal

1. Stop `preflight --global --summary` from labeling a single env `Project: <uuid>` while content is multi-project.  
2. Align summary scope vocabulary with T207 (`Scope: global` / `Scope: project=…`).  
3. Add **SQL vault rollup** counts (global Projects, Pinned, Active sessions) + label marker counts as **In context**.  
4. Fix dead **Active Sessions** counter (`Session ID:` never appears in body).  
5. Hermetic + docs; zero new crates; no T180 JSON key growth; no ledgerful-on-global; **parameterized SQL only (M1)**.

## Absorbed deferred / audit / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| deferred.md T214 | preflight --global summary 6/6 | Hard DoD F1–F8, AC1–AC13 |
| Placeholder “true rollup or honest label” | Both | F2 + F4/F7/F8 |
| audit2 T56 | heuristic ≠ structured | F4 dual model |
| T207 Scope SOOT | format_scope_line / get_project_by_id | F2/F13 **pub(crate)** |
| T180 JSON freeze | text + word_count only | F11/AC6/F39 |
| T170 D21 | summary ≠ governed | F10 + docs |
| AI1 | Affirms Scope, dual counts, sessions, freezes | Affirm |
| AI2 **M1** | format! SQL risk | F5/F7/F8/F27 — QueryStore + params![] |
| AI2 **M2** | F7 SOOT freeze | `count_projects_with_pinned` pinned-only |
| AI2 **M3** | print_summary signature | F37 |
| AI2 **M4** | smoke preserve env asserts | F38 |
| AI2 **M5** | word_count field | F21 `context.word_count` |
| AI2 L2/L3/L5/L6 | lookup, dispatch, omit Projects, CAPABILITIES Scope row | F13/F3/F4/F19 |
| AI2 L1/L4/L7/L8 | is-terminal, empty vault, ANSI, protocol_compat | F24 / AC9 / F6 / F39 |

**Not absorbed:** governed multi-project packet; ledgerful under global (F9); auto-global; clap 5; MSI; is-terminal std migrate DoD; T216; silent PreflightContextResponse growth; full `active_sessions` format! refactor (soft residual).

## Live dogfood freeze (2026-08-05)

| Metric | Value |
|--------|-------|
| Project-scoped `--summary` | `Project: 441837f6-…` |
| `--global --summary` | **Same Project uuid label** — dishonest |
| Active Sessions | **0** (marker bug) |
| `sessions.rs` active_sessions | `format!("... project_id = '{}'", pid)` — pre-existing; **do not copy** |
| JSON keys | `text`, `word_count` only (T180) |

## Research freeze (2026-08-05)

| Topic | Note |
|-------|------|
| clig.dev | Scope flags must match output |
| clap / rusqlite | 4.5 / 0.39 — no bump |
| is-terminal | soft residual → std IsTerminal (L1) |
| word_budget | `word_count` = `split_whitespace().count()` — F21 SOOT |

## Phases

### Phase 0 — Plan freeze

- [x] Live dogfood project vs global summary
- [x] Code map (`print_summary` ignores `global`; Session ID marker dead)
- [x] Online / dep pin research
- [x] Spec F1–F39 + AC1–AC13
- [x] **AI fold-in** M1–M5 / L1–L8 disposition **§14**
- [x] deferred.md + conductor registry → Planning
- [x] `ai-brains pin` plan-start + freeze + fold-in
- [x] User **go** before code / ledger TX

### Phase 1 — Ledger + red

- [x] `ledgerful doctor` / `ledgerful ledger status --compact`
- [x] `ledgerful ledger start T214-preflight-global-rollup --category FEATURE --message "preflight global summary honesty: Scope + SQL rollup + in-context markers"`
- [x] Red pure: format_scope_line global; summary dual labels; no Project: line
- [x] Red hermetic skeleton: multi-project global summary fails today

### Phase 2 — Scope SOOT share (F13 / L2)

- [x] `pub(crate)` `format_scope_line` in recall (prefer over extract-to-module)
- [x] Preflight: `ctx.conn.get_project_by_id(project_id)` **before** `format_scope_line`
- [x] Recall units still green

### Phase 3 — SQL rollup helpers (M1 / M2 / F5 / F7 / F8 / F27)

- [x] **Must:** `rusqlite::params![]` / bound args — **do not** adopt `sessions.rs:23` `format!` pattern
- [x] `QueryStore::count_projects_with_pinned()` — frozen SOOT:  
  `SELECT COUNT(DISTINCT project_id) FROM memory_projection WHERE status='pinned' AND project_id IS NOT NULL`  
  **Not** `list_projects` (unpinned over-count)
- [x] `QueryStore::count_pinned_memories(Option<&ProjectId>)` — params when scoped
- [x] `QueryStore::count_active_sessions(Option<&ProjectId>)` — params when scoped; **not** load turns via `active_sessions`
- [x] Store hermetic tests (AC13)
- [x] Capture-independent (no models)

### Phase 4 — Green summary wire (M3 / F3 / L3 / F21)

- [x] Signature: `print_summary(ctx, global, project_id, context: &PreflightContext)` (F37)
- [x] Dispatch mirror recall:  
  `let effective_project_id = if *global { None } else { *project_id };`  
  pass into options + summary (F3)
- [x] Dual blocks: Vault SQL (Projects **only if global**) + In context markers + Total Word Count from `context.word_count`
- [x] Footer retained (F22)

### Phase 5 — Hermetic + smoke (M4 / F38 / L4 / L8)

- [x] `tests/preflight_global_summary.rs` AC1–AC5, AC9 (init-only empty), AC12
- [x] Smoke: stdout `Project:` → `Scope: project=`; **preserve** inherited-id absence + stderr override warnings
- [x] Confirm `protocol_compat_cli` preflight JSON still 2 keys (F39 — no new JSON test required)

### Phase 6 — Docs + gate (L6)

- [x] CAPABILITIES preflight section **and** Scope row (~line 202) cover preflight summary
- [x] CHANGELOG Unreleased
- [x] Soft skill one-liner
- [x] `cargo fmt` / clippy / nextest targeted → full gate
- [x] `ledgerful verify`; review.md; pin closeout decisions
- [x] conductor → Completed; deferred strike T214

## Implement notes

- **PowerShell:** use `;` not `&&`.  
- **No** domain logic sprawl in CLI beyond adapter + pure formatters. Prefer pure `format_preflight_summary_lines(...)` unit-tested.  
- **Do not** edit `.ledgerful/` by hand.  
- **Do not** enable ledgerful when global (F9).  
- **Do not** add JSON keys to non-summary preflight.  
- **Do not** call `list_projects` for F7 or `active_sessions` for F5 rollup.  
- Prefer T207 `get_project_by_id` for project Scope label.

## Manual test checklist (on go)

```powershell
ai-brains preflight --summary
ai-brains preflight --global --summary
ai-brains preflight --format json   # still 2 keys
```

Record: Scope lines, Projects (global only)/Pinned/Active sessions, In context markers, exit 0.

## Stop-before

- Force-push / push main  
- Governed multi-project packet design expansion  
- Ledgerful-on-global product flip without user OK  
- Spreading `format!` SQL “because sessions does it”  
- Scope exceeds T214 (T216, density, ANN)
