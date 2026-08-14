# T248 Plan — Retention plan human summary

**Status:** ✅ **Completed** (2026-08-14 PR #161 `c633781`)  
**Spec:** [spec.md](./spec.md) F1–F18 / AC1–AC15 + §14 AI fold-in  
**Category:** UX / FEATURE  
**Ledger TX (on go):** `ledgerful ledger start T248-retention-plan-human --category FEATURE --message "TTY human for retention plan (auto); JSON keys frozen; apply stays JSON; no live apply"`

---

## AI fold-in (2026-08-14) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates planned work. AI2 M1 is a must-pin honesty fix (`memory_legacy` zero-row). AI1 `other` passthrough declined (T246 repeat).

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** | AI1 | **Agree dispatch / decline passthrough** | F1; no `other` arm |
| **AI1 M2–M4 / L1 / O1** | AI1 | **Agree** | Already F2/F6–F9 / F14 / AC1–AC7 |
| **AI1 remapped ACs** | AI1 | **Decline** | Keep AC1–AC15 |
| **AI2 M1** | AI2 | **Agree hard** | F7 `skip`; OPERATIONS rewrite |
| **AI2 L1 / L2** | AI2 | **Agree hard** | F2 order + Totals exact |
| **AI2 L3** | AI2 | **Agree hard** | Phase 2 `TempEnv` |
| **AI2 L4 / L5** | AI2 | **Agree** | F14 case-sensitivity; F10 as-is |
| **AI2 L6** | AI2 | **Agree hard** | HORIZON **36** |
| **AI2 L7** | AI2 | **Agree hard** | `format: String` ripple |

### Pins locked by fold-in

1. **F7:** `memory_legacy` → `skip` (not `soft_forget` / not `held`).
2. **F2/F11:** `next:` last after Errors; Totals exact string.
3. **F2:** HORIZON width 36.
4. **F1:** clap reject; `format: String`; apply `is_tty: false`.
5. **F14:** case-sensitive `--format`; human not a wire contract.
6. **Hermetics:** `TempEnv` clear `AI_BRAINS_RETENTION_*`.

---

## Preflight (plan time — 2026-08-14)

| Check | Result |
|-------|--------|
| `retention plan` | Pretty JSON; `classes: []`; `candidates=0`; 9 horizons; 4 honesty warnings |
| `retention plan --format human` | Title + zero totals + 4 raw `!` warnings; **no** horizons; **no** matrix; **no** “nothing to dispose” |
| `generated_at` | RFC3339 + Windows nanos |
| clap Plan/Apply | `default_value = "json"`; no `value_parser`; unknown → `OutputFormat::parse` → Json |
| `build_report` | Omits zero-count classes (`empty()` asserts `classes.is_empty()`) |
| Apply | `--confirm` refuse unchanged; **not** run live |
| Desktop | Honest-unavailable retention UI |
| Nightly | Totals `eprintln!` only — leave |
| clap / serde_json / is-terminal / chrono | lock 4.6.1 / 1.0.150 / 0.4.17 / 0.4.44 — **no bumps** (crates.io clap 4.6.6, serde_json 1.0.151, chrono 0.4.45) |
| rustc | 1.95.0 |
| Ledger | 0 pending, 0 unaudited drift |
| T243 / T245 / T246 / T247 | Completed — no rewrite |
| Live apply | **Not** run (F13) |
| Preflight | Scope `test-alias`; doctor degraded (backup_recent / recovery_kit / graph sparse) — unrelated |
| Recall | T166 engine locks + T246 presentation SOOT; no prior T248 pin |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Retention plan human / JSON-only | deferred.md / audit P2 E7/Q7 | **DoD** F1–F11 |
| Empty classes thin | README / placeholder F3 | **F3/F6** full matrix on pretty |
| T166 §6.2 json/human | T166 | Human half |
| Placeholder F1–F3 | spec draft | All absorbed |
| ISO/GDPR schedule visibility | research | Print horizons when empty |
| Desktop UI / doctor / nightly restyle / T166 engine leftovers | T172 / T249 / T166 | **Not absorbed** |

---

## Phase 0 — Ledger + impact (on go)

- [x] `ledgerful ledger status --compact` — expect 0 pending, 0 unaudited drift
- [x] `ledgerful ledger start T248-retention-plan-human --category FEATURE`
- [x] `ledgerful scan --impact`
- [x] Confirm no other agent is editing `retention.rs` / `contracts/retention.rs` / `class_based_retention.rs`

---

## Phase 1 — Red → Green: pure formatters (F1 / F2 / F3 / F6–F11 / AC1–AC7)

- [x] `resolve_retention_format(&str, is_tty)` — clap rejects unknown; `_` fail-closed json after parser; **no** `other` passthrough
- [x] Apply calls resolver with **`is_tty: false`** (`auto` → json even on TTY)
- [x] `format_retention_pretty` — F2 order (title → empty/Work → matrix → Totals → Honesty → cascade → Errors → `next:`)
- [x] Totals **exact** `Totals  candidates=N ce_wipe=N projection_delete=N skip=N held=N`
- [x] HORIZON width **36**; CLASS 18 / MECHANISM 18 / COUNT 5
- [x] Numeric horizon → `Nd`; policy labels unchanged
- [x] Zero-row `memory_legacy` MECHANISM **`skip`** (not `soft_forget` / not `held`)
- [x] Samples `", "` join; empty `—`; no `{:?}`
- [x] `next:` last; CE → `--scope`; projection → `--confirm`; zero → omit
- [x] Units AC1–AC7 with constructed `RetentionPlanReport` (no vault); AC2 asserts exact Totals + `skip`

---

## Phase 2 — Wire plan + clap (F1 / F5 / F14 / AC8–AC10)

- [x] Plan `--format: String` default `auto` + `value_parser` (not `Option<String>`)
- [x] Apply `--format: String` default `json` + same parser
- [x] `PlanOptions` / `ApplyOptions.format: String` + `main.rs` dispatch (AI2 L7)
- [x] `run_plan` / `emit_report` use local resolver (not `OutputFormat::parse`)
- [x] JSON path still `emit_json` (`to_string_pretty`); keys frozen
- [x] after_help: TTY example + `--format json`
- [x] Hermetic AC8–AC10: **`TempEnv` RAII clears `AI_BRAINS_RETENTION_*`** (AI2 L3); `#[serial(env)]` if the test binary also mutates those keys

---

## Phase 3 — Apply format opt-in only (F4 / AC11 / AC12 / AC15)

- [x] Apply default / `json` / `auto` = `emit_json` (no TTY switch)
- [x] `--format human` title `Retention apply`; gates unchanged
- [x] Hermetic: apply without `--confirm` still refuse
- [x] Existing CLI apply unit module + CP class_based_retention green

---

## Phase 4 — Docs (F14 / AC14)

- [x] `Docs/CAPABILITIES.md` OutputFormat row + Operator note
- [x] `Docs/PROTOCOL-COMPAT.md` §5 TTY/pipe + keys unchanged
- [x] `Docs/OPERATIONS.md` TTY vs json examples **and** `memory_legacy` mechanism → `skip` (v1 none auto)
- [x] Skill one-liner (`.agents/skills/ai-brains/SKILL.md`)
- [x] `CHANGELOG.md` T248 row only

---

## Phase 5 — Live dogfood + gate (on go; **no apply**)

- [x] TTY `retention plan` shows `Nothing to dispose.` + class matrix (live vault currently 0 candidates)
- [x] Piped `retention plan` parses as JSON (`horizons`, `classes`, `api_version`)
- [x] `--format xml` exit 2
- [x] Targeted nextest + clippy (full gate after review)
- [x] Review log + conductor Completed only after go+ship

---

## Isolation checklist

- [x] No live `retention apply --confirm`
- [x] No `class_based_retention.rs` / contracts DTO rewrite
- [x] No nightly `eprintln!` restyle
- [x] No desktop / HTTP / doctor retention check
- [x] No `OutputFormat::parse` change (T227 F34)
- [x] No new crates / lock bumps
- [x] No T243 / T245 / T246 / T247 rewrite
- [x] No `AI_BRAINS_KEY` print/commit
