# T216 Plan — Forget-list + memory inventory skim

Status: **In Progress** (implementing on `feat/T216-forget-list-inventory`). Spec: [spec.md](./spec.md).

## Goal

1. Stop `forget --list-forgotten` from being an unbounded, scope-opaque content dump.  
2. Ship read-only **`memory list`** inventory skim (default pinned) without requiring a recall query.  
3. Ship **`memory list --summary`** counts (scoped + global by-project).  
4. Shared parameterized store list/count API; hermetic proof; docs; series T205–T216 close.  
5. **Exit-2 honesty** via `fail_usage` / `GovernedCliError` (AI1 M1).  
6. **Tag filter correctness** two-stage anchored SQL + Rust tokens (AI1 M2).

## Absorbed deferred / audit / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| deferred.md T216 | forget list effect 5 | Hard DoD F1–F48, AC1–AC20 |
| Placeholder counts/skim | inventory + summary | F1, F8, F11 |
| T203 LIMIT+1 | pagination | F6 (clamp_list_limit reuse M3) |
| T207/T214 Scope | Scope line + `--global` | F3, F4, F32 |
| T214 parameterized SQL | no format! ids | F15, L3 `(sql, params)` |
| T198 empty | non-blank | F14 |
| T212 truncate / label | preview + project col | F8/F9/F31; `display_label` L8 |
| T204 help IA | Daily inventory | F17 **const + test** (M4) |
| AI1 **M1** | exit 2 plumbing | F3 `fail_usage` / GovernedCliError |
| AI1 **M2** | tag false-match | F12/F41/F43 AC10 |
| AI1 **M3** | clamp free | F6 control-plane reuse |
| AI1 **M4** | help_ia CI break | F17 AC13 |
| AI1 **M5** | summary flag interactions | F11/F46/F47 AC19 |
| AI1 **M6** | role strip always | F9 |
| AI1 **M7** | project col width | F8 PROJECT_COL_MAX=20 |
| AI1 L1–L6/L8 | preview dual, clap env, SOOT, turn-only, stderr tip, no freeze, display_label | F26/F4/F15/F38/F36/F22/F8 |
| AI2 | affirms core | F45 |

**Not absorbed:** tag schema; auto-forget; CE wipe; governed discovery; daemon HTTP; clap 5; MSI; rusqlite 0.40; T214 soft residuals; separate UsageError type; is-terminal whole-crate migrate (L7 soft).

## Live dogfood freeze (2026-08-05)

| Metric | Value |
|--------|-------|
| `forget --list-forgotten` (test-alias) | **29** rows, unbounded, no Scope/JSON/limit |
| Main project memories | **~8399** |
| Pin tags | Content prefix `TAGS: …` first line only |
| Exit path today | plain `Err(String)` → **exit 1** (M1 gap for claimed exit 2) |
| `clamp_list_limit` | Already in CLI via control-plane (source/evidence) |
| `help_ia` Daily | Exact string without `memory` |
| clap / rusqlite | 4.5 / 0.39 — no bump |

## Research freeze (2026-08-05)

| Topic | Note |
|-------|------|
| clig.dev | Human tables; JSON; stderr messages; suggest next (F36) |
| clap 4.6.5 / rusqlite 0.40.1 | no bump (L9) |
| T203 | DEFAULT 50 / MAX 200 via clamp_list_limit |
| handle_cli_result | already downcasts GovernedCliError |

## Phases

### Phase 0 — Plan freeze

- [x] Live dogfood list-forgotten + project scale
- [x] Code map (forget.rs unbounded; no pinned inventory)
- [x] Online / dep pin research
- [x] Spec F1–F40 + AC1–AC17
- [x] **AI fold-in** M1–M7 / L1–L9 / AI2 → F3–F48 + AC10–AC20 + **§14**
- [x] deferred.md + conductor → Planning
- [x] `ai-brains pin` plan-start + fold-in
- [x] User **go** before code / ledger TX

### Phase 1 — Ledger + red

- [x] `ledgerful doctor` / `ledgerful ledger status --compact`
- [x] `ledgerful ledger start T216-forget-list-inventory --category FEATURE --message "memory list inventory + forget list-forgotten honesty (limit/scope/json/summary; exit-2 fail_usage; tag two-stage)"`
- [x] Red pure: preview_line multibyte + role strip; tag token match (foo vs foobar)
- [x] Red store: list_memories limit+1; count; by_project; count_forgotten_memories
- [x] Red hermetic: exit **2** missing scope; list-forgotten truncation

### Phase 2 — Store API (F15/F16/F37/F42/L3)

- [x] `MemoryListRow` + filter types
- [x] `list_memories` with **`(sql, params): (String, Vec<String>)`** SOOT (mirror list_forgotten_memories)
- [x] SQL tag pre-filter only: `content LIKE 'TAGS:%'` when tag set (F12 stage 1)
- [x] `count_memories` / `count_pinned_memories` reuse / **`count_forgotten_memories`**
- [x] `count_memories_by_project` for summary global
- [x] Thin-wrap `list_forgotten_memories`
- [x] Unit tests AC16

### Phase 3 — Pure CLI formatters (F8/F9/F10/F11/F26/M6/M7)

- [x] `preview_line` 80 chars; **always** strip USER/ASSISTANT/SYSTEM (F9); **do not** share forget 100-char match preview (L1)
- [x] Tag token parse/match pure unit (F12 stage 2 / F41)
- [x] Human table + footer Showing N of T; global project col truncate 20 via `display_label` (pub(crate) project.rs)
- [x] JSON list + summary builders
- [x] Summary interactions: limit ignored; status ignored; tag filters counts (F11)

### Phase 4 — `memory list` command (F1/F3/F5/F6/M1/M3)

- [x] `commands/memory.rs` + clap in main.rs
- [x] Missing scope / invalid status / empty tag → **`fail_usage`** (exit 2)
- [x] `clamp_list_limit` from control-plane
- [x] clap-passed `project_id` env (F4) — no raw env::var on list path
- [x] `--status`, `--limit`, `--global`, `--format`, `--summary`, `--tag`
- [x] display_order ~18–22
- [x] **help_ia:** update Daily line **and** exact test assertion (M4/F17)

### Phase 5 — Wire forget --list-forgotten (F28/F39)

- [x] Pass through global/limit/format/tag + project_id
- [x] Share backend; remove unbounded dump
- [x] after_help cross-links; stderr next-step on human list (F36)
- [x] CHANGELOG BREAKING default limit 50

### Phase 6 — Hermetic + docs

- [x] `tests/memory_list_inventory.rs` AC1–AC20
- [x] CAPABILITIES / OPERATIONS / CHANGELOG / soft skill (F36/F23/F38 turn-only note)
- [x] Manual dogfood + `$LASTEXITCODE` for exit 2

### Phase 7 — Review + gate + close

- [x] Internal review → fix
- [x] Cross-model if needed
- [x] Full gate: fmt; clippy -D warnings; nextest; deny; audit
- [x] `ledgerful verify` + ledger commit
- [x] conductor **Completed**; deferred strike T216; pin closeout
- [x] PR

## Stop-before

- Destructive git / force-push / push main without approval  
- Schema migration for tags  
- New error type instead of GovernedCliError/fail_usage  
- Scope exceeds F25 list  

## Manual test checklist (on go)

```powershell
ai-brains memory list --limit 5
ai-brains memory list --status forgotten --limit 5
ai-brains forget --list-forgotten --limit 5
ai-brains memory list --summary
ai-brains memory list --summary --global
ai-brains memory list --format json --limit 3
ai-brains memory list --no-project-context   # expect exit 2 unless --global
ai-brains memory list --status bogus         # expect exit 2
```

## Notes

- Plan-only: no production code until **go**.  
- Prefer pure + hermetic over live network.  
- Event log remains append-only; list never writes.  
- AI fold-in disposition: spec **§14**.
