# T247 Internal Completeness R1
## Verdict: CLEAN

Code and docs implement F1–F10, F17–F19 and AC1–AC7 / AC11–AC14. No evidence of `ProbeStatus::Skipped`, `try_join!`, CSV col-5 Last Result, PS-first fetch, models-crate edits, or lock pin bumps. AC8–AC10 remain live dogfood (orchestrator has not run them); that is verification residual, not an implementation gap.

## Requirement matrix (one row per F/AC)

| ID | Verdict | Evidence |
|----|---------|----------|
| **F1** | MET | `Nightly.quick` is `requires = "status"`, `conflicts_with_all = ["schedule", "unschedule"]` in `crates/ai-brains-cli/src/main.rs`. Status branch always opens vault + `resolve_nightly_model_endpoints()`, then passes string `"skipped"` into `format_endpoint_line`. No `ProbeStatus::Skipped`. No `if !status && quick` in `nightly::run`. |
| **F2** | MET | `NIGHTLY_STATUS_PROBE_TIMEOUT = 750ms` sits next to `NIGHTLY_PROBE_TIMEOUT = 2s`. Default status uses `tokio::join!` (not `try_join!`) and `probe_health(NIGHTLY_STATUS_PROBE_TIMEOUT)`. Run pre-summarize still sequential 2s. Models AC6 fixtures still `Duration::from_secs(2)`. |
| **F3** | MET | `SchtasksListV` + `parse_schtasks_list_v`. `fetch_schedule_snapshot`: LIST `/fo LIST /v` first; non-success → `Default` (all `None`), no PS; PS only if LIST succeeded and `last_result` missed; CSV `fetch_schedule_next_run` only if `next_run` missed. `parse_last_result_list_v` kept (`#[allow(dead_code)]`) + unit still present. `next_run_from_schtasks_csv_line` uses col 1 only. |
| **F4** | MET | `explain_last_task_result`: trim; `0x`/`0X` → `from_str_radix(16)` else `parse::<u32>()`; fail → `None`. `0` none; `1` fail/missing; `101` panic; `267009` / `267014` SCHED_S strings (not errors). Hint is a **following** line in `format_schedule_status_lines`. AC4 substring `Last task result: 101` asserted. |
| **F5** | MET | Block order: Scheduled → Last task result → hint? → Last scheduled run → missing-action → vault `Last nightly run:` → unsummarized → endpoints. Both timestamps printed independently; status does not write vault. |
| **F6** | MET | `first_quoted_action_target` takes first `"`/`'` token ending `.cmd`/`.bat`/`.exe`. Missing → `Action target missing: <path>` + `next: ai-brains nightly --schedule --dry-run`. Existing product exe + `nightly` → no missing line. Unquoted / non-script / no token → no line. Status does not schedule or write `.cmd`. |
| **F7** | MET (code) / UNVERIFIED (live ms) | Exit `Ok(())` for status regardless of probe/result/missing action. Latency design is parallel 750ms / `--quick` no HTTP. Plan.md timing template still empty; live &lt;1s / &lt;1.5s not recorded this review. |
| **F8** | MET | `Docs/OPERATIONS.md` 1 vs 101 vs Event ID 101, missing-wrapper next step, `--quick`, LIST /V primary. `Docs/CAPABILITIES.md` T247 honesty bullets. Root `CHANGELOG.md` Unreleased T247 row only. |
| **F9** | MET | CLI `Cargo.toml` has no `reqwest`. `ProbeStatus` remains Ok/Down/Timeout/Error. Lock: tokio **1.52.3**, clap **4.6.1**, reqwest **0.13.4**. No nightly types in `ai-brains-contracts`. Status path is query-only (no events). |
| **F10** | MET (status path) | Status does not call `schtasks /create`/`/delete`, does not write `nightly-run.cmd` or ProgramData wrappers. Remediating action is print-only `--dry-run` hint. |
| **F17** | MET | clap `requires = "status"`; unit `nightly_quick__without_status__clap_requires_status`. No manual `if !status && quick`. |
| **F18** | MET | `truncate_for_embed` still in `crates/ai-brains-brain/src/embeddings.rs` with T229 UTF-8 units; T247 did not reopen it. |
| **F19** | MET | `--quick` branch never constructs `LlamaCppProvider` / never calls `probe_health`. Still resolves endpoints; `probe_label = "skipped"`. |
| **AC1** | MET | `parse_schtasks_list_v__english_fixture__extracts_next_last_result_and_action` (next, last-run, result `"1"`, Task To Run). |
| **AC2** | MET | `parse_schtasks_list_v__missing_english_labels__fields_none` (locale + `N/A`). |
| **AC3** | MET | `explain_last_task_result__*` covers `0`, `1`, `101`, `0x65`/`0X65`, `267009`/`0x41301`, `267014`/`0x41306`, `"99"` / garbage. |
| **AC4** | MET | `format_schedule_status_lines__last_result_101__contains_101` still asserts `Last task result: 101`. Order unit asserts hint is the next line. |
| **AC5** | MET | `format_status_action_missing__absent_cmd__next_step_dry_run` + `…__product_exe_nightly__no_missing_line`. |
| **AC6** | MET | `format_endpoint_line__quick__probe_skipped` (`probe=skipped` string). |
| **AC7** | MET | clap units: requires status; conflicts `--schedule`; `--status --quick` parses. `conflicts_with_all` also lists `unschedule` (compile-visible). |
| **AC8** | UNVERIFIED (manual) | Code prints `probe=skipped`, still vault + schedule, exit 0. Live &lt;1s not run this review. |
| **AC9** | UNVERIFIED (manual) | Default still probes via `as_label()` (ok/down/timeout/error) at 750ms parallel. Live &lt;1.5s not run this review. |
| **AC10** | UNVERIFIED (manual) | Format prints `Action target missing:` + `Last scheduled run:` + vault `Last nightly run:`. Live 8/13 vs 2026-08-02 not run this review. |
| **AC11** | MET | OPERATIONS + CAPABILITIES + root CHANGELOG T247 as required. |
| **AC12** | MET | No contracts nightly DTO; pins unchanged on inspected lock; no CLI reqwest; models probe AC6 still 2s; focused nextest 48 + clippy/fmt reported green by orchestrator. |
| **AC13** | MET | clap rejects `--quick` alone (`MissingRequiredArgument`, clap usage exit 2). `Cli::parse()` is before `AppContext` / vault open. |
| **AC14** | MET | `nightly_status__schedule_state_parse__extracts_next_run_from_csv` (3-col + quote-aware split). |

## Findings (id, severity critical/high/medium/low, files, required_fix, status=open)

None.

## Completeness sweep

- **Placeholders / stubs:** none in the T247 status/parse/format/clap surface. `todo!` / `unimplemented!` / `ProbeStatus::Skipped` absent from `nightly.rs`.
- **`try_join!`:** absent. Status probes use `tokio::join!`.
- **CSV col-5 Last Result:** `next_run_from_schtasks_csv_line` reads index 1 only; comments forbid Last Result from CSV.
- **PS-first fetch:** inverted. `fetch_schedule_snapshot` is LIST /V first; `fetch_last_task_result_ps` is locale-miss fallback after successful LIST /V only.
- **Models crate:** `ProbeStatus` still `{Ok, Down, Timeout, Error}` + `as_label()`. No `Skipped` variant. `llama_cpp_probe_health.rs` AC6 still 2s.
- **Pin bumps:** tokio 1.52.3, clap 4.6.1, reqwest 0.13.4 — matches F9 (no 1.53 / clap 5 / reqwest train bump).
- **F20:** optional; `help_ia.rs` has no `--quick` one-liner (allowed).
- **Manual / plan evidence:** `plan.md` Phase 7 timings template unfilled. Not a code defect.

## Wiring

```
clap Nightly { status, quick }  --quick requires status, conflicts schedule/unschedule
  → main.rs dispatches nightly::run(..., quick)
  → if status:
       vault QueryStore (last_run / unsummarized / last_count / last_errors)
       resolve_nightly_model_endpoints()
       if quick: labels = ("skipped","skipped")   // no LlamaCppProvider, no probe_health
       else: tokio::join!(probe 750ms, probe 750ms) → as_label()
       print header
       Windows: schtasks LIST /V
         success → parse_schtasks_list_v
           last_result miss → PS Get-ScheduledTaskInfo
           next_run miss → schtasks CSV (col 1)
         non-success → all None, no PS
       format_status_schedule_block:
         Scheduled → Last task result → hint? → Last scheduled run → Action target missing?
       Last nightly run: (vault)
       unsummarized / last_count / last_errors
       format_endpoint_line(..., probe_label)   // probe=ok|down|timeout|error|skipped
       multi-import (T239)
       return Ok(())   // exit 0
  --quick without --status: clap parse fail (exit 2) before AppContext
```
