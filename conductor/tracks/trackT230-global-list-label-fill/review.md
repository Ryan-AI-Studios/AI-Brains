# T230 Internal Review + Codex product disposition

## Verdict: CLEAN (product) — closeout pending for governance Completed

One-line SOOT harden of `display_label` (empty/whitespace **name** → `(no alias)`), additive unit + store + CLI hermetics, CAPABILITIES + new CHANGELOG only. F1–F35 and AC1–AC16 met. No production unwrap/expect, no contract growth, no dep bumps, no `alias.trim()`, no `ProjectRegistered` from read paths, no `MemoryListItemJson.label`.

## AC matrix (AC1–AC16)

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** | Met | `display_label__empty_name__returns_no_alias` → `"(no alias)"` |
| **AC2** | Met | `display_label__whitespace_name__returns_no_alias` (`"   "`, `"\t\n  "`) |
| **AC3** | Met | `display_label__empty_name_with_alias__alias_wins` (`""`/`"   "` + `"acme"`) |
| **AC4** | Met | Existing `display_label__true_human_name__as_is` kept |
| **AC5** | Met | Existing `display_label__baked_no_alias_prefix__literal_no_alias` kept |
| **AC6** | Met | Hermetic + live dogfood blank **15→0** |
| **AC7** | Met | JSON `by_project[].label` non-empty (hermetic + live 41/41) |
| **AC8** | Met (unit+store+live) | Store orphan inject; unit AC1; live orphan `eae2d22b` → `(no alias)` |
| **AC9** | Met | `memory_list__global_human__project_col_non_empty`; no `MemoryListItemJson.label` |
| **AC10** | Met | `project_list_labels` in nextest bundle green |
| **AC11** | Met | store inventory + CLI inventory green |
| **AC12** | Met | exit 0 + `Scope: global` |
| **AC13** | Met | CAPABILITIES T230 row; new CHANGELOG T230 only |
| **AC14** | Met | Full gate: nextest **2558** pass (1 skip); fmt/clippy/deny/audit green |
| **AC15** | Met | `forget_list_forgotten__global__project_col_non_empty` |
| **AC16** | Met | `(no alias)` fits ≤20 and ≤30 |

## Findings

### Internal
_No P0–P3 product defects._

### Codex R1 product (2026-08-11) — `review.codex.md`

| ID | Sev | Finding | Disposition |
|----|-----|---------|-------------|
| CX1 | P2 | Live orphan E2E blank 15→0 not recorded | **Fixed** — install + dogfood: `blank_labels=0` / 41 by_project; orphan `eae2d22b` → `(no alias)`; plan manual evidence |
| CX2 | P2 | Governance deferred/README still Planning while conductor Implementing | **Closeout process** — product PR ships code; Completed + deferred strike in closeout PR (T225–T228 pattern). Not a product defect |

## Completeness

### Production SOOT (`project.rs::display_label`)

Order matches **F4 / F32**:

1. `!alias.is_empty()` → alias **as-is** (**F5**: no `alias.trim()`)
2. `name.starts_with("(no alias)")` → literal `(no alias)`
3. `name.trim().is_empty()` → `(no alias)` ← T230 green patch
4. `is_non_human_project_name` → `(no alias)`
5. else → `name.to_string()`

Call sites: `build_by_project_rows`, global `emit_list_human`, project list, forget via `run_inventory` (**F30**).

### Gates observed

- nextest workspace: **2558 passed**, 1 skipped
- clippy workspace `-D warnings`: clean
- fmt check / deny / audit: green (audit allowed warnings only)
- Live: blank 15 → **0** after release install

## Residual / soft (post-ship)

| Item | Notes |
|------|-------|
| **F11** summary unaliased footer | Soft; not DoD |
| **F34** whitespace-only alias | Pre-existing; not DoD |
| **CLI-level orphan E2E hermetic** | Soft; F29 store+unit+live covers |
| **Orphan re-registration tooling** | Soft product residual |
| **Governance Completed** | Closeout PR after product CI green |

## Reviewer notes

- FEATURE light; F35 optional cross-model — user requested Codex anyway.
- Product complete for PR; track **Completed** only after closeout + final clean Codex.
