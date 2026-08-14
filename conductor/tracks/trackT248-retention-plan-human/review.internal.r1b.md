# Track Completion Audit — T248 (internal r1b)

**Track:** T248-RetentionPlanHuman  
**Category:** UX / FEATURE  
**Reviewer:** Grok (read-only)  
**Date:** 2026-08-14  
**Spec:** `conductor/tracks/trackT248-retention-plan-human/spec.md`  
**Implementation:** commit `920c78c` on `feature/T248-retention-plan-human`  
**Primary files:** `crates/ai-brains-cli/src/commands/retention.rs`, `crates/ai-brains-cli/src/main.rs`, `crates/ai-brains-cli/tests/retention_plan_human.rs`  

Static review only. Production code and Git were not modified. Nextest / clippy / live TTY (AC13) were **not** re-executed in this pass. F13 honored: no live `retention apply --confirm`.

## Verdict: PASS

CLEAN. No P0–P3 findings. Focus pins (resolver fail-closed, apply `is_tty: false`, F2 order, exact Totals, column widths, sample join, F10 shorts, `next:`, JSON freeze, apply refuse exit 6) match the spec. New-behavior tests would fail against the pre-T248 human dump / silent-JSON parser.

## Summary

T248 is presentation-only. `plan_retention` / `build_report` / contracts DTO / `OutputFormat::parse` are not on this surface. Plan default is clap `auto` + local `resolve_retention_format`; apply default stays `json` and always calls the helper with `is_tty: false`. Pretty fills the 9-class matrix locally (JSON `classes` stays sparse `[]`). Honesty shorts map known constants; `command_id=` / `ce_pending=` fall through as-is. `fail_api` refuse path is unchanged (`INVALID_PAYLOAD` → exit 6).

## Focus checklist

| # | Pin | Result |
|---|-----|--------|
| 1 | `resolve_retention_format` fail-closed; no other passthrough | **Clean.** Maps `pretty\|human\|text\|markdown\|md` → `"human"`, `json` → `"json"`, `auto` + TTY → `"human"`, `auto` else `"json"`, `_` → `"json"`. No `Some(other) => other`. Does not call `OutputFormat::parse`. `emit_report` / `retention_output_format` are also fail-closed (human vs json). |
| 2 | Apply always `is_tty: false` | **Clean.** `run_apply` line 81: `resolve_retention_format(&options.format, false)`. Comment documents F4. Plan uses `std::io::stdout().is_terminal()` via `is_terminal::IsTerminal`. |
| 3 | Pretty layout order F2 | **Clean.** Sequential pushes: title → empty/`Work` → `Class matrix` → Totals → Honesty → cascade (if `> 0`) → `Errors:` (if any) → `next:` last. |
| 4 | Totals exact | **Clean.** `Totals  candidates={} ce_wipe={} projection_delete={} skip={} held={}` (two spaces after `Totals`). AC2 asserts the zero line verbatim. |
| 5 | HORIZON 36; CLASS 18; MECHANISM 18; COUNT 5 | **Clean.** Matrix format is exactly `{:<18} {:<36} {:<18} {:>5}`. Work table is `{:<18} {:>5} {:<18} {}` (CLASS / COUNT / MECHANISM / SAMPLES). |
| 6 | Sample join `", "` never Debug | **Clean.** `ids.join(", ")`; empty → `—`. No `{:?}` on the pretty path. Unit asserts `sess:0, sess:1` and rejects `["sess:0"`. |
| 7 | Honesty shorts F10; `command_id=` / `ce_pending=` as-is | **Clean.** Exact `==` on the five `RETENTION_HONESTY_*` constants → spec shorts. Else `warning.to_string()` (covers apply prefixes and unknown tokens). JSON warnings untouched. |
| 8 | `next:` CE vs projection; never invent UUID; omit on apply and empty | **Clean.** Plan only; `would_ce_wipe > 0` → literal `Repository:<uuid>` placeholder; else projection `--confirm` only; omit when sum is 0 or `mode == "apply"`. No generated UUID. |
| 9 | JSON `emit_json` pretty; `classes` stay `[]` when empty | **Clean.** JSON path is still `emit_json` → `to_string_pretty`. Pretty fill is local to `format_retention_pretty`. Hermetic AC8 asserts `classes: []` and no additive keys. |
| 10 | `fail_api` apply refuse still exit 6 | **Clean.** Confirm/dry-run gates still `fail_api(..., ApiError::new("INVALID_PAYLOAD", ...))`. `exit_code_for_api_error` maps that code to `EXIT_INVALID_PAYLOAD` (6). Hermetic AC11 asserts exit 6 + `INVALID_PAYLOAD`. |
| 11 | Tests would fail against pre-T248 | **Clean.** See § Tests vs pre-T248. |
| 12 | No `unwrap`/`expect` in production | **Clean.** Production `retention.rs` uses `unwrap_or` / `unwrap_or_else` / `?` only. `expect` is test-only. |
| 13 | Clippy risks (`collapsible_match`, unused) | **Clean.** No nested match. Resolver / emit `_` arms are fail-closed, not unused. `zero_row_mechanism` `_ => skip` is defensive for `&str` (same as explicit legacy/unclassified; `match_same_arms` is pedantic, not `-D warnings`). Hermetic `let _env = isolate_retention_env()` retains RAII. |

## P0

None.

## P1

None.

## P2

None.

## P3

None.

## Requirement / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** resolver | **MET** | Units: auto+TTY human; auto+pipe json; pretty/human/text/markdown/md → human; json → json. Apply wiring passes `false` so auto stays json. |
| **AC2** empty pretty | **MET** | `format_retention_pretty__empty__…`: `Nothing to dispose.`, all 9 class ids, `90d`, exact Totals zeros, four shorts, `memory_legacy` `skip` (not `soft_forget`/`held`), no `next:`. |
| **AC3** Work + samples + projection next | **MET** | raw_turn fixture: `Work`, `sess:0, sess:1`, no Debug, `next: … --confirm`, no `--scope`. |
| **AC4** CE next | **MET** | Exact `next: ai-brains retention apply --confirm --scope Repository:<uuid>`; CE honesty short present. |
| **AC5** custom horizon | **MET** | `horizons["raw_turn"]="45"` → `45d`; raw_turn line must not keep `90d`. |
| **AC6** nanos timestamp | **MET** | `2026-08-14 03:12 UTC`; no `.076`. |
| **AC7** honesty map + unknown | **MET** | Known shorts in empty + CE fixtures; `brand_new_honesty_token_xyz` echoed. |
| **AC8** JSON freeze | **MET** | Hermetic `--format json`: keys `api_version`/`horizons`/`classes`/`totals`/`warnings`; `api_version=="1"`; `classes==[]`; no `pretty_matrix`/`human`/`next_step`/`format`. |
| **AC9** pretty hermetic | **MET** | `--format pretty` empty vault: `Nothing to dispose`, `raw_turn`, `Class matrix`; exit 0. `TempEnv` clears all `RetentionConfig::from_env` keys. |
| **AC10** clap reject | **MET** | Hermetic `--format xml` exit 2, stdout not `{…}`; clap unit `InvalidValue` in `main.rs`. |
| **AC11** apply refuse | **MET** | `retention apply` without `--confirm` → exit 6 + `INVALID_PAYLOAD`. |
| **AC12** apply format | **MET** | Empty fixture `--confirm --format json` parseable `api_version=1` / `mode=apply` / `candidates=0`; `--format human` contains `Retention apply`. No live CE. |
| **AC13** live TTY/pipe | **NOT RE-RUN** | Spec live-on-go. Code path is auto + `is_terminal()`. Not a defect. |
| **AC14** docs | **MET** | CAPABILITIES OutputFormat rows + operator note; PROTOCOL-COMPAT §5 TTY/pipe + keys + pretty JSON + human-not-wire + case-sensitive tokens; OPERATIONS TTY vs json + `memory_legacy` → `skip`; skill one-liner; CHANGELOG T248 row; Plan + `RetentionCommands` after_help. |
| **AC15** existing suites | **NOT RE-RUN** | Apply unit module in `retention.rs` still present (`production_apply_requires_*`, `resolve_retention_apply_scope__*`). Isolation: no T248 markers in `class_based_retention.rs` / `contracts/retention.rs`. |

### Hard F pins (high-risk)

| F | Status |
|---|--------|
| F1 resolver + clap `String` + `value_parser` + apply `is_tty: false` | **MET** |
| F2 order + Totals + widths | **MET** |
| F3 empty pretty + JSON `classes` empty | **MET** |
| F4 apply default json; auto does not TTY-switch | **MET** |
| F5 JSON keys / warning strings / `emit_json` pretty | **MET** (no DTO edit) |
| F6 9-class matrix; numeric `d`; policy labels as stored; missing `—` | **MET** |
| F7 zero-row defaults; `memory_legacy` → `skip`; extra classes after nine | **MET** |
| F8 `", "` samples; notes off pretty | **MET** |
| F9 chrono RFC3339 → `YYYY-MM-DD HH:MM UTC`; frac-strip fallback | **MET** |
| F10 shorts + unknown/as-is prefixes | **MET** |
| F11 `next:` last; CE vs projection; no invented UUID | **MET** |
| F12 no new crates / no DTO / no nightly restyle (static) | **MET** on touched surface |
| F13 no live apply | **HONORED** by this review |
| F14 docs + after_help | **MET** |

## Tests vs pre-T248

Pre-T248 human path (spec §2.1 / plan dogfood): title + zero totals + raw `!` warnings; **no** horizons; **no** class rows; **no** `Nothing to dispose.`; `samples={:?}` if any class existed. Unknown `--format` went through `OutputFormat::parse` → silent JSON.

| Test | vs pre-T248 |
|------|-------------|
| empty pretty (`Nothing to dispose.`, `Class matrix`, `90d`, exact Totals `ce_wipe=`, shorts, `memory_legacy` skip) | **Would fail** |
| raw_turn `sess:0, sess:1` / reject Debug / projection `next:` | **Would fail** |
| CE `next:` + `--scope Repository:<uuid>` | **Would fail** |
| `45d` / `2026-08-14 03:12 UTC` / no `.076` | **Would fail** |
| known shorts + unknown echo (unknown half) | unknown echo would pass; shorts **would fail** |
| hermetic pretty AC9 | **Would fail** |
| hermetic / clap `--format xml` exit 2 | **Would fail** (old silent JSON exit 0) |
| JSON `classes: []` + frozen keys | **Would pass** (intentional regression) |
| apply refuse exit 6 | **Would pass** (intentional regression) |
| apply `--format json` parseable | **Would pass** (intentional regression) |
| apply human title `Retention apply` | may have passed if old title already matched; layout is proven by plan pretty units |

Resolver / pretty units are new functions; they encode the T248 contract, not the old dump.

## Isolation / non-goals

- `OutputFormat::parse` still lowercases and maps unknown → Json (T227 F34 leftover). Retention does not call it.
- No T248 markers in `class_based_retention.rs`, `contracts/retention.rs`, nightly, or desktop.
- Apply confirm / dry-run XOR / daemon / `--scope` / `RetentionApplied` gates unchanged.
- Capture-independent: plan is still read-only `plan_retention`.

## Not findings

- Pretty section **order** and **column widths** are not position-asserted (tests use `contains`). The format strings are spec literals and the push order is F2-normative; ACs do not require index checks.
- `run_apply`’s `is_tty: false` is a one-line literal. AC1 is helper-level (`auto` + `false` → json). No hermetic `--format auto` on apply (default is `json`, not `auto`).
- `command_id=` / `ce_pending=` as-is share the unknown-warning fallback already unit-tested.
- Cascade / `Errors:` / extra non-canonical class / missing-horizon `—` have no dedicated fixtures. Code paths exist; ACs do not require them.
- AC13 live TTY/pipe and AC15 full nextest were not re-run in this read-only pass.
- Pre-existing apply error line `unexpected daemon response: {other:?}` is T166 daemon-error display, not sample Debug (F8).
- Spec/plan files still say **Planning**; conductor hygiene, not a CLI contract defect.
- Soft residuals F16–F18 (verbose notes, JSON zero buckets, doctor check, nightly restyle, shared `resolve_*_format`, apply-warning shorts, color/pager) remain out of scope.

## Live evidence

Not collected. AC13 remains an on-go operator check: TTY `retention plan` human matrix; piped `retention plan` JSON with `classes` / `horizons`. Do not apply.
