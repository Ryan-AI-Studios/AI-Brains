# T230 — Global inventory label fill — Plan

**Status:** 🚧 **Implementing** — code + tests 2026-08-11  
**Category:** UX / FEATURE (light)  
**Depends:** T212 `display_label` · T216 memory inventory global summary

## Goal

Never show empty `label` cells on global inventory tables. Harden `display_label` so empty/whitespace **name** → `(no alias)`; orphan `project_id`s (memories without `project_projection`) get the same token. JSON `by_project`/`projects` labels never `""`. Shared forget-list path inherits fix. Close audit residual “Global summary blank labels.”

## Absorbed deferred

| Item | Disposition |
|------|-------------|
| deferred.md Global summary blank labels | **DoD** |
| Series README T230 | **DoD** |
| T216 F8 incomplete for orphan project_ids | **Harden SOOT F4/F6/F29** |

**Not absorbed:** auto-alias from git; orphan re-registration; T216 tag histogram / `--offset`; T231; T229; T228 sync residuals; summary unaliased footer (soft F11 only); **alias.trim()**; non-summary JSON `label` field.

## Research pins (2026-08-11)

| Pin | Evidence |
|-----|----------|
| Live blank count | **15** empty labels on `memory list --summary --global` |
| Orphans | Blank ids **not** in `project list` (e.g. `eae2d22b-…` pinned 437) |
| Root cause | `get_project_by_id` None → `display_label("","",id)` → `""` |
| `count_memories_by_project` | Groups `memory_projection` only — no project JOIN |
| **CLI pin auto-registers** | `context.rs:120-141` — **cannot** produce orphans via CLI pin |
| Live orphan provenance | Legacy import / projection lag — not normal pin |
| Store orphan inject | `store/tests/memory_list_inventory.rs` `pin_memory` without register |
| forget shared path | `run_inventory` / `emit_list_human` global project col |
| Non-summary JSON | `MemoryListItemJson` — **no** `label` field |
| Truncation | `(no alias)` = 10 < 20 / 30 |
| T212 | Registered projects already non-blank |
| clig.dev | Human tables honest; JSON shape stable |
| Deps | clap **4.6.1** / chrono **0.4.44** / is-terminal **0.4.17** / serde **1.0.228** — **no bump** |
| Zero new crates | — |

## AI fold-in pins (hard)

| ID | Pin |
|----|-----|
| **AI1 M1 / F4–F5 / F32** | Empty/ws **name** guard after baked prefix; **reject** `alias.trim()` |
| **AI1 M2 → F29** | Orphan coverage required — strategy from AI2 M1 |
| **AI1 M3 / AC16** | Truncation fit verify only |
| **AI1 L1 / F6** | Read-only — no `ProjectRegistered` on list |
| **AI1 L2 / F18** | CAPABILITIES + new CHANGELOG |
| **AI1 O1** | Units AC1–AC3 named |
| **AI2 M1 / F29 / AC8** | Store orphan inject + unit AC1 + live blank 0; decline CLI pin / SQL-in-CLI |
| **AI2 M2 / F30 / AC15** | Touch map forget; cheap hermetic preferred |
| **AI2 M3 / F31 / AC9** | Human-only non-summary project col; no JSON label add |
| **AI2 O2 / AC3** | Explicit empty-name+alias unit |
| **AI2 O3** | forget `--list-forgotten --global` hermetic preferred |
| **AI2 O4 / F33** | Named AC6/AC7 labels_non_empty hermetic |
| **AI2 O6 / F35** | No mandatory cross-model |

**Soft:** F11 summary footer; F34 whitespace alias; CLI orphan E2E hermetic residual.

See `spec.md` §15 full disposition.

## Frozen decision index

See `spec.md` §3 **F1–F35**. Hard summary:

1. Never-blank `display_label` (F2).  
2. Empty/ws **name** → `(no alias)`; **no** alias.trim (F4/F5/F32).  
3. Orphans display-only — no auto-register (F6).  
4. Orphan proof = store + unit + live (F29).  
5. JSON labels non-empty only where `label` exists; no list-item label (F7/F31).  
6. forget global shared path (F30).  
7. Call sites use SOOT only (F9).  
8. New CHANGELOG only (F18).  

## Task checklist

### 0. Preflight (on go)

- [x] Ledger TX already open (860678cc-8b4c-401d-a547-3bc17d875e8e) — implementer does not commit ledger
- [x] Live baseline recorded in plan (**15** blank on planner machine)

### 1. Red — tests first

- [x] Unit `display_label__empty_name__returns_no_alias` (AC1)
- [x] Unit `display_label__whitespace_name__returns_no_alias` (AC2)
- [x] Unit `display_label__empty_name_with_alias__alias_wins` (AC3)
- [x] Hermetic CLI `memory_list__global_summary__labels_non_empty` (AC6 human + AC7 JSON)
- [x] Store AC8: `pin_memory` without `register_project` → orphan in `count_memories_by_project`
- [x] forget `--list-forgotten --global` project col non-empty (AC15) + AC9 human list
- [x] Confirm red fails on current main (empty-name + whitespace units failed before green)

### 2. Green — production

- [x] Harden `display_label` empty/ws **name** arm after baked prefix (F32)
- [x] Do **not** trim alias (F5/F34)
- [x] Confirm memory by-project + global list + forget path already call SOOT
- [x] No production unwrap/expect; no new crates; no `MemoryListItemJson.label`

### 3. Verify regression

- [x] CLI unit filter `display_label` (8/8 pass; binary crate — no `--lib`)
- [x] `cargo nextest run -p ai-brains-store --test memory_list_inventory` (7/7)
- [x] `cargo nextest run -p ai-brains-cli --test memory_list_inventory --test project_list_labels` (26/26)
- [x] `cargo clippy -p ai-brains-cli -p ai-brains-store --all-targets -- -D warnings`
- [x] `cargo fmt`
- [x] Live dogfood: blank count **15 → 0** (2026-08-11 after `cargo install --path crates/ai-brains-cli --locked --force`; JSON `by_project` 41 rows all non-empty labels; orphan `eae2d22b-…` shows `(no alias)`; forget global project col non-empty)

### 4. Docs

- [x] CAPABILITIES Memory inventory: never-blank labels; orphans `(no alias)`; non-summary JSON id-only
- [x] **New** CHANGELOG T230 entry only (do not edit T212/T216 historical bullets)
- [ ] deferred.md strike → closed by T230 (closeout PR after product merge)
- [ ] series README T230 closed (closeout PR after product merge)
- [x] conductor.md → **Implementing** (Completed in closeout PR after CI green)

### 5. Review + gate

- [x] Internal review vs `spec.md` ACs → **CLEAN** (`review.md`)
- [x] Full gate: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace` **2558 passed** (1 skipped) `; cargo deny check ; cargo audit` (allowed warnings only)
- [x] Live dogfood F29 third leg recorded
- [x] Codex product R1: P2 live dogfood → **fixed** (recorded); P2 governance lag → **closeout process** (not product defect)
- [ ] `ledgerful ledger commit` + pin decision (on product merge)
- [ ] PR + CI Win/Linux/macOS

### 6. Soft residuals (document, not block)

- [ ] F11 summary unaliased footer (optional)
- [x] F34 whitespace-only alias (residual noted; not DoD)
- [ ] Orphan re-registration tooling
- [x] CLI orphan E2E hermetic residual (F29 store+unit+live)
- [ ] T216 F24 tag histogram / offset (unchanged soft)

## Implement order (TDD)

```
red units (empty/ws/alias-wins) → red store orphan AC8 → red CLI AC6/AC7
  → green display_label (name empty/ws only)
  → AC15 forget global (cheap)
  → verify T212/T216 green + live blank=0
  → docs → gate → finalize
```

## Manual evidence (2026-08-11)

```
Before: memory list --summary --global → blank labels = 15
After (installed release CLI): blank_labels=0 total_by_project=41
  - orphan eae2d22b-… label=(no alias) pinned=437
  - forget --list-forgotten --global project col = test-alias (non-empty)
  - project list still non-blank (regression OK)
  - no MemoryListItemJson.label added
```

## Stop-before

- Do not auto-register orphan projects from list/summary/forget-list
- Do not hide orphan rows by joining only to `project_projection`
- Do not invent orphans via CLI pin (auto-registers) or high-friction SQLCipher open in CLI tests
- Do not add `label` to non-summary list JSON items
- Do not `alias.trim()` (F5/F34 residual)
- Do not bump clap to 5 / force lock churn without need
- Do not rewrite historical CHANGELOG T212/T216 rows
