# Track Completion Audit — T248

## Verdict: PASS

## Scope Reviewed

Read-only audit of working-tree implementation against `spec.md` (AI fold-in) F1–F15 / AC1–AC15 and the plan isolation checklist. Commit `920c78c` was not re-diffed via `git show` (no shell in this reviewer); isolation was proven by searching forbidden crates/files for T248 / formatter symbols and by reading the live surfaces below.

- `crates/ai-brains-cli/src/commands/retention.rs` — resolver, pretty formatter, plan/apply emit, apply gates, units AC1–AC7
- `crates/ai-brains-cli/src/main.rs` — `RetentionCommands` clap (`format: String`, `value_parser`, defaults, after_help) + dispatch
- `crates/ai-brains-cli/tests/retention_plan_human.rs` — hermetic AC8–AC12
- `crates/ai-brains-cli/src/commands/governed_common.rs` — `OutputFormat::parse` (untouched), `emit_json` (`to_string_pretty`), `fail_api` / exit 6
- `crates/ai-brains-contracts/src/retention.rs` — DTO / `CANONICAL_CLASSES` / honesty constants / `empty()` (no T248 rewrite)
- `crates/ai-brains-control-plane/src/class_based_retention.rs` — `collect_candidates` / `build_report` (no T248 rewrite; stream A still does not scan `memory_legacy`)
- Docs: `Docs/CAPABILITIES.md`, `Docs/PROTOCOL-COMPAT.md` §5, `Docs/OPERATIONS.md`, `CHANGELOG.md`, `.agents/skills/ai-brains/SKILL.md`
- Pins: workspace `Cargo.toml` + `Cargo.lock` (`clap` 4.6.1, `serde_json` 1.0.150, `chrono` 0.4.44, `is-terminal` 0.4.17)
- Isolation probes: `apps/` (no T248), `brain` nightly `eprintln!` (unchanged), no `comfy-table`/`tabled`, no `OutputFormat::parse` call from `retention.rs`

Orchestrator-observed gates were treated as verification evidence (clippy CLI PASS; retention units PASS; hermetic 5/5; CP `class_based_retention` 30/30; live TTY/pipe/`xml`; no live `retention apply --confirm`).

## Requirement and DoD Matrix

| ID | Verdict | Evidence |
|----|---------|----------|
| **F1** | PASS | Plan `--format: String` default `auto` + `value_parser = ["auto","pretty","human","text","json","markdown","md"]` (`main.rs` 1433–1438). Apply same parser, default `json` (1446–1451). `resolve_retention_format(&str, bool) -> &'static str` (retention.rs 302–309): pretty/human/text/markdown/md → human; json → json; auto+TTY → human; auto+pipe → json; `_` → json (no `other` passthrough). Plan probes `std::io::stdout().is_terminal()` via `is_terminal::IsTerminal`. Apply calls the same helper with **`is_tty: false`** (line 81). Does not call `OutputFormat::parse`. `PlanOptions`/`ApplyOptions.format: String` + `main.rs` dispatch. No `ignore_case` on the parser (case-sensitive; `JSON`/`Pretty` clap-reject). |
| **F2** | PASS | `format_retention_pretty` order: title (`Retention plan (dry-run)` / `Retention apply` + `generated YYYY-MM-DD HH:MM UTC`) → empty `Nothing to dispose.` / Work table (non-zero only) → Class matrix (all `CANONICAL_CLASSES` then extras) → exact `Totals  candidates=…` → Honesty → cascade if `parents_marked_for_resynthesis > 0` → `Errors:` → `next:` last (plan only). Columns `{:<18} {:<36} {:<18} {:>5}` (HORIZON **36**). Work `{:<18} {:>5} {:<18} {}`. No `comfy-table`. |
| **F3** | PASS | Empty when `classes.is_empty() \|\| totals.candidates == 0`: `Nothing to dispose.` + full matrix; no plan `next:`. JSON path is still `emit_json(report)` — planner still omits zero buckets (`RetentionPlanReport::empty` + hermetic `classes: []`). |
| **F4** | PASS | Apply default `json`; `auto` forced json via `is_tty: false`. `--format human`/`pretty`/`text`/`md` reuse the same formatter with apply title. Confirm / dry-run XOR / daemon / `--scope` / `RetentionApplied` sequence unchanged after format resolve. |
| **F5** | PASS | JSON is `emit_json(report)` → `to_string_pretty` on the T166 DTO. No contracts field/`api_version` change (`API_VERSION` still `"1"`). No additive wrapper keys. Honesty warning **strings** unchanged (pretty maps locally). Hermetic forbids `pretty_matrix`/`human`/`next_step`/`format`. |
| **F6** | PASS | Matrix always walks `CANONICAL_CLASSES`. Horizon text from `report.horizons`; numeric-only labels get `d`; policy labels (`revoked_superseded+30d_cooldown`, `none_auto`, `skip_apply`) passed through; missing key → `—`. AC5 locks `45` → `45d` without hardcoded `90d` on that row. |
| **F7** | PASS | Bucket present → bucket `mechanism`. Zero-row defaults: raw_turn/query_trace/review_trace/decision_approved → `projection_delete`; evidence/secret/orphaned_envelope → `ce_wipe`; **`memory_legacy` + `unclassified` → `skip`**. Extra non-canonical classes print after the nine. OPERATIONS `memory_legacy` row rewritten to `skip` (F14). Planner still does not scan stream-A `memory_legacy` (`collect_candidates` only turns/traces/reviews/decisions + envelopes). |
| **F8** | PASS | Samples `ids.join(", ")`; empty → `—`; no `{:?}` on the pretty path. Notes never printed. DTO already truncates ids. |
| **F9** | PASS | `DateTime::parse_from_rfc3339` → `%Y-%m-%d %H:%M UTC`. Fallback strips fractional seconds before tz. JSON `generated_at` untouched. AC6 locks nanos fixture → `2026-08-14 03:12 UTC` and no `.076`. |
| **F10** | PASS | Exact `==` map of the five honesty constants to F10 shorts; unknown echoed; `command_id=` / `ce_pending=` fall through as-is. JSON `warnings` verbatim via `emit_json`. |
| **F11** | PASS | `next:` only when `!is_apply && would_ce_wipe + would_projection_delete > 0`, after Errors. CE → `next: ai-brains retention apply --confirm --scope Repository:<uuid>`; else projection → `--confirm` only. Placeholder, not an invented UUID. Apply omits `next:` (unit). |
| **F12** | PASS | No new crates (`Cargo.toml` unchanged surface; no comfy-table). Lock pins unchanged. No CLI reqwest. No contracts DTO / HTTP / desktop / doctor / nightly restyle. Capture-independent presentation + existing `plan_retention`. |
| **F13** | PASS | No live `retention apply --confirm` in this audit or orchestrator go. Hermetic AC12 apply is tempdir vault only. No horizon retune, nightly CE, or `class_based_retention.rs` candidate rewrite. |
| **F14** | PASS | CAPABILITIES OutputFormat rows + operator note. PROTOCOL-COMPAT §5 plan+apply rows: TTY/pipe, keys frozen, `to_string_pretty`, human not a wire contract, **case-sensitive** tokens. OPERATIONS TTY vs `--format json` examples **and** `memory_legacy` → `skip`. Skill one-liner (`.agents/skills/ai-brains/SKILL.md` 111–116). CHANGELOG Unreleased T248 row only (other rows untouched). after_help on `RetentionCommands` + Plan: TTY example + `--format json`. |
| **F15** | PASS | Orchestrator: CP `class_based_retention` 30/30; CLI apply-gate units still in `retention.rs` (`production_apply_requires_*`, `resolve_retention_apply_scope__*`). Contracts `API_VERSION == "1"`. |
| **F16–F18** | N/A (soft residual) | Correctly not implemented (`--verbose` notes, JSON zero buckets, doctor check, nightly restyle, desktop/HTTP, `OutputFormat::parse` surface-wide, std `IsTerminal`, shared `resolve_*_format`, apply-warning shorts, T166 engine leftovers, color/pager/comfy-table). |
| **AC1** | PASS | Units: `auto`+TTY human; `auto`+pipe json; pretty/human/text/markdown/md → human either TTY; json → json. Apply wiring is `is_tty: false` (source). Resolver unit documents `auto`+false → json. |
| **AC2** | PASS | `format_retention_pretty__empty__…` asserts `Nothing to dispose.`, all 9 class ids, `90d`, exact `Totals  candidates=0 ce_wipe=0 projection_delete=0 skip=0 held=0`, F10 shorts, `memory_legacy` line `skip` not `soft_forget`/`held`, no `next: ai-brains retention apply`. |
| **AC3** | PASS | Raw-turn fixture: `Work`, `sess:0, sess:1`, no Debug `[\"sess:0\"`, `next: … --confirm`, no `--scope`. |
| **AC4** | PASS | CE fixture: exact `next: … --confirm --scope Repository:<uuid>` + CE honesty short. |
| **AC5** | PASS | Custom `horizons["raw_turn"]="45"` → `45d`; that row must not still show `90d`. |
| **AC6** | PASS | Nanos timestamp → `2026-08-14 03:12 UTC`; no `.076`. |
| **AC7** | PASS | Known shorts covered across empty + CE fixtures; unknown `brand_new_honesty_token_xyz` echoed. |
| **AC8** | PASS | Hermetic `--format json`: exit 0, keys `api_version`/`horizons`/`classes`/`totals`/`warnings`, `api_version=="1"`, `classes==[]`, no listed additive keys. TempEnv clears `AI_BRAINS_RETENTION_*`. |
| **AC9** | PASS | Hermetic `--format pretty`: `Nothing to dispose`, `raw_turn`, `Class matrix`. |
| **AC10** | PASS | Hermetic `--format xml` exit **2**; stdout does not start with `{`. |
| **AC11** | PASS | Hermetic `retention apply` without `--confirm`: exit **6**, `INVALID_PAYLOAD` present. Format resolve happens before the refuse and does not swallow it. |
| **AC12** | PASS | Hermetic empty-fixture `apply --confirm --format json` parses (`api_version`/`mode==apply`/`candidates==0`); `--format human` contains `Retention apply`. Tempdir vault only — **not** live CE. |
| **AC13** | PASS | Orchestrator live: TTY `retention plan` human (Nothing to dispose + 9-class matrix + skip + exact Totals); piped JSON parses; `--format xml` exit 2. No apply. |
| **AC14** | PASS | See F14. |
| **AC15** | PASS | Orchestrator: CP suite 30/30; CLI retention unit module (apply-gates + new pretty units) green. |
| **Isolation: no live apply** | PASS | AC12 is hermetic empty fixture. Orchestrator did not run live `--confirm`. This review did not either. |
| **Isolation: no planner / DTO rewrite** | PASS | No T248 symbols in `class_based_retention.rs` or contracts; DTO comments still T166. |
| **Isolation: no nightly restyle** | PASS | Nightly still `eprintln!` totals / intent-only CE notes. |
| **Isolation: no desktop / HTTP / doctor** | PASS | No T248 under `apps/`; OPERATIONS still honest-unavailable retention UI. |
| **Isolation: no `OutputFormat::parse` change** | PASS | Still lowercase + silent Json. Retention uses the local resolver only. Other governed commands still call `parse`. |
| **Isolation: no new crates / lock bumps** | PASS | clap 4.6.1 / serde_json 1.0.150 / chrono 0.4.44 / is-terminal 0.4.17. No comfy-table. |
| **Isolation: no T243/T245/T246/T247 rewrite** | PASS | Graph `resolve_graph_format` / update default json unchanged. T248 only added a sibling local resolver. |
| **Isolation: no `AI_BRAINS_KEY` print/commit** | PASS | Pretty path prints title/matrix/totals/honesty only. Hermetic uses the standard zero-key test helper, not a live key. |
| **§14 pin 1 F7 skip** | PASS | Zero-row `memory_legacy` → `MECHANISM_SKIP`; OPERATIONS mechanism column `skip`. |
| **§14 pin 2 order + Totals** | PASS | `next:` after Errors; AC2 exact Totals string. |
| **§14 pin 3 HORIZON 36** | PASS | Format `{:<36}` on header and rows. |
| **§14 pin 4 clap / String / apply false** | PASS | See F1. |
| **§14 pin 5 case-sensitive; human not wire** | PASS | PROTOCOL-COMPAT §5. No `ignore_case`. |
| **§14 pin 6 TempEnv** | PASS | `isolate_retention_env()` `TempEnv::remove` on all `RetentionConfig::from_env` keys. Standalone test binary; nextest process isolation; `#[serial(env)]` not required. |

## Findings

CLEAN. No P0–P3 product defects, missing wiring, JSON key drift, apply-gate regressions, or placeholders (`TODO` / `unimplemented!` / stub emit path) on the T248 surface.

## Completeness Sweep

- End-to-end wiring: `RetentionCommands::{Plan,Apply}` → `PlanOptions`/`ApplyOptions.format: String` → `run_plan`/`run_apply` → local `resolve_retention_format` → `emit_report` → `format_retention_pretty` / `emit_json`. No leftover `OutputFormat::parse` on this command. Old human Debug dump (`samples={:?}`, raw warning constants, no matrix) is gone.
- Placeholders / stubs: none in `retention.rs` production or the hermetic test.
- JSON key freeze: contracts DTO untouched; emit path is the DTO. Hermetic AC8 asserts the AC-required keys and empty `classes`. Nested class-bucket keys are unchanged because the type is unchanged.
- Tests vs old behavior (as requested):
  - **Would fail on pre-T248:** AC2–AC7 (new formatter); AC9 (`Nothing to dispose` / `Class matrix` absent on old human); AC10 (old `OutputFormat::parse` mapped `xml` → Json, exit 0).
  - **Pass on old by design (freeze / non-regression):** AC8 JSON keys; AC11 apply refuse; existing `production_apply_requires_*` / `resolve_retention_apply_scope__*` units. That is correct, not a hollow gate.
  - AC1’s apply-auto protection is the `false` argument at `run_apply` line 81. The named unit tests the shared resolver (`auto`+false → json), not a compiled assertion that `run_apply` passes `false`. Source inspection confirms the wiring; a later edit to `stdout().is_terminal()` on apply would not fail that unit. Not filed: implementation matches F1/F4, and clap default `json` is a second belt.
- HORIZON 36, `next:` after Errors, and `command_id=` as-is are implemented exactly; they are not separately hermetic-tested beyond the units that lock Totals / next copy / unknown-warning echo.
- Conductor registry still says Planning (plan Phase 5: Completed only after go+ship). Not a DoD miss.

## Wiring and Regression Review

- **Apply gates:** `run_apply` still refuses `--dry-run`+`--confirm` and missing `--confirm` with `INVALID_PAYLOAD` **before** store/plan/mutation. Scope + daemon gates still keyed off `would_ce_wipe`. Format is resolved first only to choose `fail_api`/`emit_report` presentation — same pattern as other governed commands. AC11 hermetic proves the refuse is not swallowed.
- **Apply is not quieter by default:** clap default `json` + `is_tty: false` so `auto` cannot TTY-switch. Opt-in `--format human` only changes the title/formatter after a successful confirm path.
- **JSON path:** `_ => emit_json(report)` still `to_string_pretty`. No compact flip. No injected zero class buckets.
- **Shared parser:** `OutputFormat::parse` still lowercases and silently returns Json. T227 F34 leftover untouched. T248 clap reject is local.
- **Planner / contracts:** `collect_candidates` still omits stream-A `memory_legacy`; `build_report` still emits only non-empty buckets. Pretty fills zeros locally.
- **Nightly / desktop / HTTP / doctor:** not restyled or newly wired.

## Verification Evidence

Cited from orchestrator (this reviewer did not re-run cargo or live CLI):

- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` PASS
- Units in `retention.rs` (AC1–AC7 + apply-gate units) PASS
- `cargo nextest run -p ai-brains-cli --test retention_plan_human` 5/5 PASS
- `cargo nextest run -p ai-brains-control-plane --test class_based_retention` 30/30 PASS
- Live: `retention plan --format pretty` shows `Nothing to dispose.` + 9-class matrix + `skip` + exact Totals; piped JSON parses; `--format xml` exit 2
- No live `retention apply --confirm` on the real vault (hermetic empty apply only)

Source-level confirmation of the focus pins: JSON emit = DTO + `to_string_pretty`; apply `is_tty: false`; `memory_legacy` zero-row `skip`; Totals exact two-space string; HORIZON `{:<36}`; `next:` last after Errors; clap `value_parser` without `ignore_case`; `OutputFormat::parse` unchanged; no planner/contracts rewrite; TempEnv on hermetics; AC12 tempdir only.

## Deferred Candidates

None. Soft F16–F18 remain spec residuals, not review deferrals.

## Completion Decision

**PASS.** T248 DoD (F1–F15 / AC1–AC15 / isolation / §14 pins) is implemented and wired. No blocking findings. Safe to treat the track as complete from an internal R1 audit once go+ship flips conductor status.
