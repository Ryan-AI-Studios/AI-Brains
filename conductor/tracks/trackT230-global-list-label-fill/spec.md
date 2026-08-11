# T230 — Global inventory label fill

- **Track ID:** T230-GlobalListLabelFill
- **Phase:** Post-audit CLI quality series (T217–T232) — P3 polish after T228
- **Status:** 🚧 **Implementing** — code + tests 2026-08-11
- **Depends on:** T212 `display_label` SOOT ✅; T216 memory inventory `--summary --global` by-project table ✅; T207/T214 Scope vocabulary (unchanged)
- **Blocks / feeds:** Operators can scan multi-project vaults without blank `label` cells; closes audit residual “Global summary blank labels”
- **Category:** UX / FEATURE (light) / DOCS
- **Source:** Audit 2026-08-05 + live dogfood 2026-08-11 — `memory list --summary --global` many empty `label` cells; series README row; deferred.md Placeholder
- **Deferred absorbed:** deferred.md “Global summary blank labels” → **DoD**; series README T230; T216 F8 soft “reuse display_label” incomplete for orphan `project_id`s → **harden SOOT**
- **Not absorbed:** Auto-create aliases from git slug (explicit non-goal); orphan project re-registration / data repair; tag histogram (T216 F24 soft); `--offset` cursor; clap 5; T231 unified search; T229 ops; short-UUID-in-label-column alternative (project_id already full); contract DTO key growth; **alias.trim()** collapse (pre-existing whitespace-alias residual — F5/F34)
- **Research date:** 2026-08-11 (live dogfood + code truth + clig.dev + dep pins)
- **AI fold-in:** 2026-08-11 — AI1 **M1** (name empty/ws only; **reject** alias.trim), **M2→F29**, **M3**, **L1–L2**, **O1 hard**. AI2 **M1–M3 hard**; **O1–O4 hard**; **L3/O5 soft residual**; **O6** no mandatory cross-model. Disposition **§15**.
- **Ledger:** plan-only — no TX until go

## 1. Objective

1. **Never-blank labels:** Under global (and any) inventory tables that use `display_label`, never emit an empty `label` cell — human **or** JSON (where a `label` field exists).
2. **SOOT harden `display_label`:** Empty / whitespace-only `name` with empty `alias` falls through to `(no alias)` (same token as T212 machine names). `project_id` stays its own column — do **not** invent short-uuid labels that duplicate it.
3. **Orphan project_ids:** Rows from `count_memories_by_project` whose `project_id` is missing from `project_projection` (`get_project_by_id` → `None`) still get `(no alias)` via the same helper.
4. **JSON frozen shape:** No new keys; existing `label` string becomes non-empty for every by-project / project-list row. Non-summary list JSON has **no** `label` field (unchanged). Exit codes unchanged.
5. **Capture independence:** Display-only; no models, embeddings, graph, or event appends. **No** `ProjectRegistered` from read paths (CQRS read purity).

## 2. Live baseline (re-scan 2026-08-11)

### 2.1 Operator dogfood (this machine)

| Command | Observed |
|---------|----------|
| `memory list --summary --global` | **15 blank `label` cells** (whitespace-only under `label` header); many `(no alias)` OK; aliases OK |
| `memory list --summary --global --format json` | Same 15 rows with `"label": ""` |
| `project list` | All rows non-blank (`(no alias)` or human path/alias) — **does not list orphan ids** |
| Blank sample | `eae2d22b-…` pinned **437**, **not** in `project list` JSON |

### 2.2 Root cause (frozen)

```text
// memory.rs — build_by_project / global list project col
let (name, alias) = match get_project_by_id(...) {
    Some((n, a)) => (n, a),
    None => (String::new(), String::new()),  // orphan project_id
};
let label = display_label(&name, &alias, &project_id);

// project.rs — display_label today
if !alias.is_empty() { return alias; }
if name.starts_with("(no alias)") { return "(no alias)"; }
if is_non_human_project_name(name, project_id) { return "(no alias)"; }
// is_non_human("", id) is false (empty ≠ uuid / short / "Project …")
name.to_string()  // "" when name empty → BLANK CELL
```

| Gap | Detail |
|-----|--------|
| Empty name arm missing | `display_label("", "", id)` → `""` (not non-human) |
| Orphan memories | `count_memories_by_project` groups `memory_projection.project_id` only — **no** JOIN to `project_projection` |
| Live scale | 41 by-project rows; **15** orphans; project list 30 registered |
| **CLI pin cannot create orphans** | `ensure_project_and_session_exists` (`context.rs:120-141`) auto-`ProjectRegistered` before pin — normal CLI pin always registers |
| Live orphan provenance | Likely **legacy import / projection lag / corruption** — not normal pin path |
| Hermetic gap | T216 AC9 asserts headers/ids, **not** non-empty label |
| Unit gap | No `display_label__empty_name__no_alias` |
| Shared forget path | `forget --list-forgotten` → `run_inventory` → same `emit_list_human` global project col |

### 2.3 Touch map

| Site | Role |
|------|------|
| `ai-brains-cli/src/commands/project.rs` | **F4 harden:** empty/whitespace **name** → `(no alias)` after baked-prefix check; unit tests (O1) |
| `ai-brains-cli/src/commands/memory.rs` | Call sites already use `display_label` — by-project summary, global list project col; **no** local empty bypass |
| `forget --list-forgotten` | **Shared backend** `run_inventory` (F30) — global human project column inherits fix; optional AC15 hermetic |
| `ai-brains-store/tests/memory_list_inventory.rs` | **F29 AC8:** store-level `pin_memory` without `register_project` surfaces orphan pid |
| `ai-brains-cli/tests/memory_list_inventory.rs` | AC6/AC7 labels non-empty; AC9 human non-summary; AC15 forget global soft/hard |
| `ai-brains-cli/tests/project_list_labels.rs` | Verify still green (T212 ACs) |
| Docs | CAPABILITIES Memory inventory labels; **new** CHANGELOG T230 row only |
| Contracts | **None** — no `MemoryListItemJson.label`; no contracts crate change |

### 2.4 Deps / research pins

| Pin | Evidence | Action |
|-----|----------|--------|
| `clap` workspace `4.5` → lock **4.6.1** (crates.io latest **4.6.6** 2026-08) | Cargo.lock | **No bump** — no new flags |
| `chrono` `0.4` → lock **0.4.44** (crates.io **0.4.45**) | Cargo.lock | **No bump** |
| `is-terminal` `0.4` → lock **0.4.17** | Cargo.lock | **No bump** |
| `serde` lock **1.0.228** (crates.io **1.0.229**) | Cargo.lock | **No bump** |
| **Zero new crates** | F15 | — |
| [clig.dev](https://clig.dev/) | Consistent human chrome; machine JSON shape stable | Intentional non-empty labels |
| T212 F4 | alias → baked `(no alias)` → machine Project/id → name | **Extend** empty/ws name → `(no alias)` before “name as-is” |
| T216 F8/F11 | `display_label` + `PROJECT_COL_MAX=20`; by-project table | Reuse; no new columns |
| Store pin without register | `store/tests/memory_list_inventory.rs:46-67` | **F29 orphan inject home** |
| CLI pin auto-register | `context.rs:120-141` + CLI `pin_memory` hermetic | **Cannot** produce orphans via CLI pin |
| Non-summary JSON | `MemoryListItemJson` — no `label` field | **F31 / AC9** human-only for list project col |
| Truncation | `(no alias)` = 10 chars < 20 / 30 | **F8 / AI1 M3** — never clipped |

## 3. Frozen decisions (F1–F35)

| ID | Decision |
|----|----------|
| **F1 — Scope** | Display-only label fill for inventory tables using `display_label`. No FTS, no ranking, no auto-alias, no projection repair. |
| **F2 — Never blank** | `display_label` **always** returns a non-empty string for any `project_id` input (including empty/orphan name + empty alias). |
| **F3 — Fallback token** | Empty / whitespace-only `name` + empty `alias` → literal **`(no alias)`** (T212 vocabulary). **Not** short uuid in the label column (full id already in `project_id`). |
| **F4 — Order (SOOT)** | (1) non-empty alias → alias **as-is** (no trim); (2) name starts with `(no alias)` → literal `(no alias)`; (3) `name.trim().is_empty()` → `(no alias)`; (4) non-human `Project <uuid-ish>` / full\|short id match → `(no alias)`; (5) else → name as-is. Manual string ops only — **no regex**. |
| **F5 — Whitespace** | **Name:** `name.trim().is_empty()` counts as empty. **Alias:** do **not** trim-collapse (AI1 `alias.trim()` **rejected**). Whitespace-only alias (`"   "`) remains pre-existing residual (**F34**). |
| **F6 — Orphans + CQRS** | Missing `project_projection` row is a **presentation** fallback only: label `(no alias)`. Do **not** auto-`ProjectRegistered` from list/summary/forget-list (read purity). |
| **F7 — JSON scope** | `by_project[].label` and `projects[].label` never `""`. Shape/keys frozen. **Non-summary** `MemoryListItemJson` has **no** `label` — do **not** add one (F14/F31). |
| **F8 — Human truncate** | Keep `PROJECT_COL_MAX=20` / T212 30-char label col; truncate after fill. `(no alias)` length **10** fits both — verify only, no special case. |
| **F9 — Call sites** | Inventory paths that resolve name/alias for display use `display_label`: memory summary by-project, memory list global project col, project list human+JSON, **forget `--list-forgotten --global`** via shared `run_inventory`. No parallel empty-name branch. |
| **F10 — Exit codes** | Unchanged (summary/list exit 0 on success). |
| **F11 — Footer (soft)** | Optional stderr unaliased nudge on `memory list --summary --global` human (T212 pattern). **Soft residual** if noisy or set-alias fails on true orphans. DoD does **not** require summary footer. |
| **F12 — set-alias auto** | Non-goal (audit + placeholder). |
| **F13 — Capture / privacy** | Display only; no new events; no CoT. |
| **F14 — Contracts** | No `ai-brains-contracts` change. No CLI-local JSON shape growth for non-summary list. |
| **F15 — Zero new crates** | No rusqlite-in-CLI-tests for orphan inject (decline high-friction option B). |
| **F16 — Dep bumps** | No clap/chrono/serde/is-terminal bump. |
| **F17 — Tests** | Unit + hermetic ACs; TDD red→green. |
| **F18 — Docs** | CAPABILITIES: non-empty labels incl. orphan project_ids; **new** CHANGELOG T230 only (do not rewrite T212/T216 history). |
| **F19 — Scope line** | Unchanged (`Scope: global` already correct). |
| **F20 — Ordering** | by-project order stays `(pinned+forgotten) DESC, project_id ASC`. |
| **F21 — Tag filter** | Summary `--tag` still applies; labels still non-empty. |
| **F22 — Project list** | Must remain green; empty-name harden is defensive (registered rows already non-blank). Project list **never lists orphans** (FROM `project_projection`). |
| **F23 — unwrap/expect** | Forbidden in production paths. |
| **F24 — Soft residuals** | Summary unaliased footer; orphan re-registration tooling; short-uuid label alt; tag histogram; auto-alias from git; whitespace-only alias. |
| **F25 — Out of scope** | T231, T229, clap 5, CE wipe, daemon HTTP list; CLI-level orphan inject via pin/migrate. |
| **F26 — Determinism** | Same vault → same labels; sort already deterministic. |
| **F27 — Privacy inheritance** | N/A (no derived memories). |
| **F28 — Ledger category** | FEATURE (light UX) on implement. |
| **F29 — Orphan inject strategy (AI2 M1 hard)** | **Pinned:** (1) **Unit AC1–AC3** prove `display_label` for empty/ws; (2) **Store-level AC8:** `pin_memory(store, orphan_pid, …)` **without** `register_project` → `count_memories_by_project` returns orphan pid (store surfaces orphans); (3) **Live dogfood** blank **15 → 0** closes CLI E2E for real orphans. **Decline** CLI pin inject (auto-registers). **Decline** SQLCipher open-in-CLI-test (friction). **Decline** migrate-based orphan invent. |
| **F30 — forget shared path (AI2 M2 hard)** | `forget --list-forgotten` shares `run_inventory` / `emit_list_human`. Fix applies automatically. Touch map names it. **AC15:** soft or cheap hermetic — project column non-empty under `--global` (registered project sufficient for shared-path lock). |
| **F31 — AC9 human-only (AI2 M3 hard)** | Global non-summary **human** project column only. JSON non-summary items keep `project_id` only — **no** new `label` field. |
| **F32 — Green patch site** | Insert empty/ws check **after** baked `(no alias)` prefix, **before** `is_non_human_project_name`. Do **not** change alias branch to `trim()`. |
| **F33 — AC6/AC7 named hermetic (AI2 O4 hard)** | New or extended `memory_list__global_summary__labels_non_empty` — every human data-row label non-empty; every JSON `by_project[].label` non-empty (registered multi-project vault sufficient; orphan covered by F29 unit+store+live). |
| **F34 — Whitespace alias residual** | `display_label("name", "   ", id)` still returns `"   "` — pre-existing; not T230 DoD. |
| **F35 — Review depth** | FEATURE light, one-line SOOT + additive tests, no contract change → **internal CLEAN** sufficient; Codex **optional**; **no** mandatory cross-model (AI2 O6). |

## 4. Acceptance criteria

| ID | Criterion | Proof |
|----|-----------|-------|
| **AC1** | `display_label("", "", id)` → `"(no alias)"` | Unit |
| **AC2** | `display_label("   ", "", id)` → `"(no alias)"` | Unit |
| **AC3** | Alias wins over empty name: `display_label("", "acme", id)` → `"acme"` | Unit (explicit empty-name+alias case) |
| **AC4** | True human name / path unchanged | Unit (existing + keep) |
| **AC5** | Baked `(no alias) — short` still → `(no alias)` | Unit (existing) |
| **AC6** | `memory list --summary --global` human: every data row label non-empty (after truncate) | Named hermetic + live dogfood |
| **AC7** | `memory list --summary --global --format json`: every `by_project[].label` non-empty | Named hermetic |
| **AC8** | Orphan surface + display: store `pin_memory` without register → orphan in `count_memories_by_project`; unit AC1 covers `display_label("","",orphan)`; live blank **0** | Store hermetic + unit + live |
| **AC9** | Global memory **list** (non-summary) **human** project column non-empty for orphan-style empty name path (unit SOOT + human list uses `display_label`). JSON non-summary **unchanged** (no `label` field). | Unit + hermetic human list (registered OK); orphan live optional |
| **AC10** | T212 `project_list_labels` suite still green | Verify |
| **AC11** | T216 existing inventory tests still green | Verify |
| **AC12** | Exit 0; Scope line unchanged | Hermetic |
| **AC13** | CAPABILITIES + new CHANGELOG only | Doc review |
| **AC14** | Full gate green | fmt/clippy/nextest/deny/audit |
| **AC15** | `forget --list-forgotten --global` human project column non-empty (shared `run_inventory`) | Cheap hermetic preferred; else soft residual with shared-path note |
| **AC16** | `(no alias)` fits truncation (10 ≤ 20 and ≤ 30) | Unit comment or assert `chars().count() <= PROJECT_COL_MAX` |

## 5. Implementation sketch (on go)

### 5.1 Red

1. Unit: `display_label__empty_name__returns_no_alias`
2. Unit: `display_label__whitespace_name__returns_no_alias`
3. Unit: `display_label__empty_name_with_alias__alias_wins` (AC3)
4. Hermetic CLI: `memory_list__global_summary__labels_non_empty` (AC6 human + AC7 JSON)
5. Store: orphan `pin_memory` without register → pid present in `count_memories_by_project` (AC8 store half)
6. Optional cheap: `forget_list_forgotten__global__project_col_non_empty` (AC15)
7. Confirm red fails on current main (units fail; hermetics fail if they assert non-empty on a path that still blanks — registered-only hermetics pass label non-empty only after green if they only use registered projects; **prefer orphan-style unit for red**)

### 5.2 Green (pinned patch)

```rust
// project.rs display_label — after baked prefix, before is_non_human:
if name.trim().is_empty() {
    return "(no alias)".to_string();
}
// Do NOT: let trimmed_alias = alias.trim();  // F5 / F34 residual
```

No call-site change required if SOOT is complete — `None => (String::new(), String::new())` still hits the helper.

### 5.3 Docs / closeout

- CAPABILITIES § Memory inventory: labels never blank; orphans show `(no alias)`; non-summary JSON still id-only
- CHANGELOG new T230 bullet
- deferred.md + series README + conductor → Completed after PR
- Pin: `DECISION: T230 — display_label never blank; empty/ws name → (no alias); orphan inject = store+unit+live; no alias.trim; forget global shared path; AC9 human-only`

## 6. Manual test (required)

```powershell
ai-brains memory list --summary --global
# Expect: no blank label cells; Scope: global; exit 0
ai-brains memory list --summary --global --format json
# Expect: every by_project.label non-empty string
ai-brains memory list --global --limit 5
# Expect: project column non-empty (human)
ai-brains forget --list-forgotten --global --limit 5
# Expect: project column non-empty when rows exist (shared backend)
ai-brains project list
# Expect: still non-blank (regression)
```

Record before/after blank counts on live vault (baseline blank=**15**).

## 7. Deferred fold-in disposition

| Deferred / residual | Overlap? | Disposition |
|---------------------|----------|-------------|
| deferred.md Global summary blank labels | Yes | **Absorb → DoD** |
| Series README T230 | Yes | **Absorb → DoD** |
| T216 F8 display_label reuse incomplete for orphans | Yes | **Absorb → F4/F6/F29** |
| T216 F24 tag histogram / --offset / relative-time extract | Soft unrelated | **Decline** (stay soft) |
| T212 unaliased footer | Soft pattern reuse | **Soft F11** for memory summary (not DoD) |
| Auto-alias from git slug | Explicit non-goal | **Decline** |
| Orphan project re-register / data repair | Related but product-heavy | **Soft residual** |
| Whitespace-only alias | Pre-existing | **Soft F34** |
| T228 F32/F34 sync residuals | No | **Decline** |
| T231 unified search | No | **Decline** |
| T229 nightly ops | No | **Decline** |
| T227 F34 OutputFormat | No | **Decline** |

## 8. Risk & review

| Risk | Mitigation |
|------|------------|
| Truncate mid-`(no alias)` | Token length 10 < 20/30 — AC16 |
| Whitespace-only alias | F34 residual; do not alias.trim |
| set-alias on orphan fails | Footer soft; do not auto-register |
| CLI orphan hermetic impossible | F29 store+unit+live |
| Accidental `MemoryListItemJson.label` | F31 / F14 forbid |
| Regression T212 exact-match tests | Empty arm additive; run full project_list_labels |

**Review category:** FEATURE (light) — **internal CLEAN**; Codex optional; **no** mandatory cross-model (F35).

## 9. Definition of done

- [ ] F1–F35 respected
- [ ] AC1–AC16 green (AC15 soft only if hermetic deferred with justification)
- [ ] Live dogfood blank count → **0**
- [ ] CAPABILITIES + new CHANGELOG
- [ ] deferred.md + series README + conductor Completed
- [ ] Full gate green; ledger commit; pin decision
- [ ] No production `unwrap`/`expect`
- [ ] No `ProjectRegistered` from list/summary/forget-list

## 10. Non-goals (restate)

- Auto-create aliases from git slug or path
- Repairing orphan `memory_projection` rows / forcing `ProjectRegistered`
- Changing by-project SQL join to hide orphans
- Producing orphans via CLI pin in hermetics
- Adding `label` to non-summary list JSON
- `alias.trim()` semantics change
- New CLI flags / dependency bumps
- T231 product merge

## 11. Soft residuals (post-ship)

| Residual | Notes |
|----------|-------|
| F11 summary unaliased footer | Optional T212-style stderr |
| Orphan re-registration tooling | Product/data repair track |
| F34 whitespace-only alias | `display_label(name, "   ", id)` |
| CLI-level orphan E2E hermetic | Soft — F29 covers via store+unit+live |
| T216 F24 tag histogram / offset | Unchanged soft |
| Short-uuid-in-label | Declined; id column already full |

## 15. AI fold-in disposition (2026-08-11)

### AI1

| ID | Severity | Disposition | Notes |
|----|----------|-------------|-------|
| **M1** | Medium | **Accept hard (name only)** | Empty/ws `name` guard after baked prefix. **Reject** `alias.trim()` / `trimmed_alias` — conflicts with F5/F34. |
| **M2** | Medium | **Accept → F29** | Orphan hermetic required in spirit; **strategy pinned by AI2 M1** (store+unit+live, not CLI pin). |
| **M3** | Medium | **Accept hard** | Truncation verify AC16; no code special-case. |
| **L1** | Low | **Accept hard** | F6 CQRS read purity. |
| **L2** | Low | **Accept hard** | F18 docs. |
| **O1** | Opp | **Elevate hard** | Unit suite AC1–AC3 named tests. |

### AI2

| ID | Severity | Disposition | Notes |
|----|----------|-------------|-------|
| **M1** | Medium | **Accept hard → F29** | CLI pin auto-registers; pin store-level AC8 + unit + live dogfood; decline SQL-in-CLI and migrate invent. |
| **M2** | Medium | **Accept hard → F30/AC15** | Touch map + forget global shared path; cheap hermetic preferred. |
| **M3** | Medium | **Accept hard → F31/AC9** | Human-only non-summary project column; no JSON `label` add. |
| **L1–L2** | Low | Note | Visibility + ordering OK. |
| **L3** | Low | **Accept soft → F34** | Whitespace alias residual. |
| **L4–L7** | Low | Note | Safe / no action. |
| **O1** | Opp | **Elevate hard** | Store-level orphan inject = F29. |
| **O2** | Opp | **Elevate hard** | AC3 explicit empty-name+alias unit. |
| **O3** | Opp | **Elevate hard (preferred)** | AC15 forget global hermetic. |
| **O4** | Opp | **Elevate hard** | F33 AC6/AC7 named hermetic. |
| **O5** | Opp | **Accept soft** | §11 residual. |
| **O6** | Opp | **Accept hard** | F35 no mandatory cross-model. |

**Verdict after fold-in:** Ready to implement on **go**. Core one-line SOOT + additive tests; orphan strategy and AC9 scope no longer ambiguous.
