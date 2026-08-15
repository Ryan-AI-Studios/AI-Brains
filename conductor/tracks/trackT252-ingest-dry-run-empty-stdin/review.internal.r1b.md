# T252 Internal Review R1b — Correctness / tests

**Track:** T252-IngestDryRunEmptyStdin  
**Category:** UX / BUGFIX  
**Reviewer:** Grok (read-only correctness, r1b)  
**Date:** 2026-08-15  
**Spec:** `conductor/tracks/trackT252-ingest-dry-run-empty-stdin/spec.md`  
**Primary files:**  
- `crates/ai-brains-cli/src/commands/ingest.rs`  
- `crates/ai-brains-cli/src/main.rs` (`Commands::Ingest` + `handle_cli_result`)  
- `crates/ai-brains-cli/tests/ingest_reads_json_stdin.rs`  
- `crates/ai-brains-cli/src/commands/governed_common.rs` (`fail_usage` / `EXIT_USAGE`)

Static review only. Production files and Git were not modified. Nextest / clippy / AC14 TTY dogfood were **not** re-executed. Track status was **not** marked completed.

## Verdict: PASS

No P0–P3 product findings. Empty/TTY stdin is `fail_usage` → `GovernedCliError` → exit **2** before serde. TTY is refused **before** `read_to_string`. Mid-payload `{` stays a string `Invalid JSON` → `handle_cli_result` `COMMAND_FAILED` exit **1**. Hermetics use `write_stdin` (piped, not TTY) and would not hang. Isolation surfaces (`DryRunIngestRequest`, `parse_ingest_request`, T180 ingest tests, `cli_help_ia` group order) are untouched on inspection. Usage const quoting is `'{` … `}'`. after_help tests assert keys, not wrap.

## Findings (id, severity, evidence, required_fix, status)

None.

## Hunt checklist

| # | Hunt | Result | Evidence |
|---|------|--------|----------|
| 1 | TTY `is_terminal` → `fail_usage` **before** `read_to_string` | **Clean** | `ingest.rs` `run()`: `if io::stdin().is_terminal() { return fail_usage(INGEST_EMPTY_STDIN_USAGE); }` then `read_to_string`. Same `is_terminal::IsTerminal` crate as T86. |
| 2 | Empty / whitespace after read → `fail_usage` | **Clean** | After read: `if ingest_stdin_needs_usage(false, Some(&input)) { return fail_usage(...) }`. Helper is `is_tty \|\| raw.is_none_or(\|s\| s.trim().is_empty())`. Units lock `""` and `" \n"`. |
| 3 | Mid-payload `{` stays serde `Invalid JSON` → exit 1 `COMMAND_FAILED` | **Clean** | Gate is trim-empty only. `{` is non-empty → `serde_json::from_str` `.map_err(\|e\| format!("Invalid JSON: {}", e))?` → **string** `Err`, not `fail_usage`. `handle_cli_result` default arm → `("COMMAND_FAILED", s)` + `ApiResult` JSON on stderr + `exit(1)`. |
| 4 | `fail_usage` must be `GovernedCliError` (not string `Err`) | **Clean** | `fail_usage` (`governed_common.rs`) `eprintln!` then `Err(Box::new(GovernedCliError::emitted(EXIT_USAGE, message)))` with `EXIT_USAGE = 2`, `emitted: true`. `handle_cli_result` downcasts and `std::process::exit(g.exit_code)` without wrapping `COMMAND_FAILED`. A string `Err(usage.into())` would be exit 1; AC1–AC3 would fail. |
| 5 | AC1–AC8 / AC10 / AC11 assertions specific; would they fail on old code? | **Clean** | See § Tests vs pre-T252. New usage hermetics + helper/const/help units are fail-closed vs old EOF JSON. Regression tests stay specific (`[dry-run] Would ingest turn test`, not-exit-2 + not usage text, `UUID`). |
| 6 | Race / hang: hermetics `write_stdin` so not TTY | **Clean** | `run_ingest` always `.write_stdin(input)` including `""` / `"\n  \n"` / `"{"`. Piped stdin → `is_terminal() == false` → `read_to_string` gets EOF immediately. `--help` never reaches `run()`. No live-TTY child in CI (spec: unit `is_tty=true` + AC14 manual). |
| 7 | Isolation: `DryRunIngestRequest`, `parse_ingest_request`, T180, `cli_help_ia`, no new `unwrap` | **Clean** | Dry-run struct still `deny_unknown_fields` + `String` UUID fields + optional `thinking`/`tx_id`. Live path still `parse_ingest_request(&input)?` after the gate. `protocol_compat_cli.rs` T180 ingest tests unchanged (weak message `\|\|` is pre-existing; `!success` still locks deny). `cli_help_ia.rs` still only Commands-list `  ingest` position (`display_order = 50`). New production path is `?` + `fail_usage`; no `unwrap`/`expect`/`panic!` in `ingest.rs`. Pre-existing `outcome.events[0]` index is F12, live-path only. |
| 8 | after_help wrap: tests assert keys, not exact whitespace | **Clean** | `ingest__help__contains_example_keys` loops seven keys on combined help. No snapshot of indent / clap wrap. `after_help` is a pretty-indented object plus a single-quoted echo line. |
| 9 | Usage const quoting (`'{` / `}'`) PowerShell ScriptBlock trap | **Clean** | `INGEST_EMPTY_STDIN_USAGE` is `echo '{"session_id":…}' \| ai-brains ingest --dry-run`. Unit asserts `'{` and `}'` plus seven keys + `ai-brains ingest --dry-run`. Unquoted `{…}` would miss `'{`. |
| 10 | Tests that could false-pass | **Clean** for required ACs | Unique keys (`session_id`, `harness_id`, `turn_id`) make AC8 fail-closed without after_help. `role`/`content`/`privacy` are not enough alone but are AND-ed with the unique keys. AC5 `COMMAND_FAILED \|\| Invalid JSON` is the spec wording; new code emits both via the envelope. See notes below (not findings). |
| 11 | Ingest routed through `run_sync_path_free` / vault skipped | **Clean** | `is_vault_path_free` has no `Ingest` arm. Dispatch `Commands::Ingest { dry_run } => commands::ingest::run(&ctx, *dry_run)` is **after** `AppContext::from_cli` (F9). |
| 12 | Pin bumps / DTO growth | **Clean** | Workspace clap `4.5`, lock clap `4.6.1`, serde_json `1.0.150`, is-terminal `0.4.17`. `IngestRequest` / `IngestResponse` in `ai-brains-contracts` unchanged. |

## Wiring

```
stdin TTY?  --yes--> fail_usage(INGEST_EMPTY_STDIN_USAGE) --> GovernedCliError exit 2
    |
   no
    v
read_to_string
    |
trim empty? --yes--> fail_usage(...) --> exit 2
    |
   no
    v
dry_run? --yes--> DryRunIngestRequest::from_str
                    err --> "Invalid JSON: {e}" --> handle_cli_result COMMAND_FAILED exit 1
                    empty content/role --> string Err --> exit 1
                    ok --> stdout preview, exit 0
    |
   no
    v
parse_ingest_request (unchanged) --> live ingest
```

`ingest_stdin_needs_usage(is_tty, raw)` stays a pure helper with an `is_tty: bool` parameter (F6; AI1 flatten declined). `run()` must still branch on `stdin().is_terminal()` **before** read (F4); it then calls the helper with `(false, Some(&input))` so trim is not inlined twice.

## Tests vs pre-T252

Pre-T252: empty / whitespace / TTY-close → serde EOF → `Invalid JSON: EOF while parsing…` wrapped as `COMMAND_FAILED` exit **1**. TTY hung on `read_to_string`. `--help` named stdin but had no payload keys.

| Test | vs pre-T252 |
|------|-------------|
| `ingest_stdin_needs_usage__tty_or_blank__true` | **Would not compile** (helper missing) |
| `ingest_stdin_needs_usage__payload__false` | **Would not compile** |
| `ingest_empty_stdin_usage__contains_example_keys` | **Would not compile** |
| `ingest__dry_run__empty_stdin__usage_exit_2` (AC1) | **Would fail** (exit 1, `COMMAND_FAILED`, `EOF while parsing`, no usage text) |
| `ingest__dry_run__whitespace_stdin__usage_exit_2` (AC2) | **Would fail** (same) |
| `ingest__live__empty_stdin__usage_exit_2` (AC3) | **Would fail** (same via `parse_ingest_request` JSON err) |
| `ingest__dry_run__accepts_placeholder_uuids` (AC4) | **Would pass** (intentional F2 lock) |
| `ingest__dry_run__truncated_object__command_failed` (AC5) | **Would pass** (intentional F3 lock; already exit 1 JSON) |
| `ingest__help__contains_example_keys` (AC8) | **Would fail** (`session_id` / `harness_id` / `turn_id` absent) |
| `ingest__dry_run__errors_on_empty_content` (AC10) | **Would pass** on old field-error path; **would fail** if T252 mapped field errors to usage (new `assert_ne!(code, Some(2))` + rejects `stdin is empty or not piped`) |
| `ingest__non_dry_run__still_validates_uuids` (AC11) | **Would pass** (intentional live UUID lock; still `contains("UUID")`) |
| T180 `t180_c_stdin_dry_run_deny__unknown_field__rejected` | **Would pass** (isolation; file not edited) |
| T180 `t180_c_stdin_prod_open__unknown_field__accepted` | **Would pass** |
| `cli_help_ia` `long_help__daily_commands_before_harness_ingest` | **Would pass** (Ingest `display_order = 50` unchanged; after_help is subcommand-only) |

`assert_empty_stdin_usage` locks exit **2**, `stdin is empty or not piped`, `ingest --dry-run`, `session_id`, **not** `COMMAND_FAILED`, **not** `EOF while parsing`, stdout empty. Not `is_ok`/`is_err`.

## Requirement / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** dry-run empty → exit 2 usage | **MET** | `write_stdin("")` + `assert_empty_stdin_usage` |
| **AC2** dry-run whitespace | **MET** | `write_stdin("\n  \n")` same helper |
| **AC3** live empty | **MET** | `run_ingest(&[], "")` same helper (example still names `--dry-run` per F5) |
| **AC4** placeholder UUID preview | **MET** | existing test; dry-run path after gate unchanged |
| **AC5** `{` → exit 1 payload | **MET** | `assert_eq!(code, Some(1))`; combined has `COMMAND_FAILED` **or** `Invalid JSON`; not usage text |
| **AC6** helper units | **MET** | TTY/`""`/`" \n"` true; `"{"` + valid object false |
| **AC7** usage const keys + quote | **MET** | seven keys + `ai-brains ingest --dry-run` + `'{` + `}'` |
| **AC8** help keys, not wrap | **MET** | `ingest --help` exit 0 + key presence. `cli_help_ia` group-order file not edited |
| **AC9** T180 dual-path | **MET** (static) | protocol_compat ingest tests unmodified |
| **AC10** empty content ≠ usage | **MET** | `!success`, `code != 2`, stderr lacks empty-stdin phrase |
| **AC11** live UUID reject | **MET** | `!success` + `UUID` |
| **AC12** docs | **MET** (static) | CLI-EXIT-CODES fail_usage footnote + table row; CAPABILITIES §4 ingest bullet; OPERATIONS one-liner after `echo $json \| … ingest`; CHANGELOG Unreleased **BREAKING** |
| **AC13** no contracts / crates | **MET** | DTOs unchanged; no new crate |
| **AC14** live dogfood incl. TTY no-hang | **NOT RE-RUN** | Code path prints usage immediately on TTY; this pass did not attach a console |
| **AC15** targeted nextest / clippy | **NOT RE-RUN** | Out of scope for this static pass |
| **AC16** plan-only until go | **N/A** | Implementation present |

### Hard F pins (correctness)

| F | Status |
|---|--------|
| F1 empty/whitespace → `fail_usage` exit 2, both live and dry-run | **MET** |
| F2 dry-run preview / placeholder UUIDs frozen | **MET** |
| F3 mid-payload envelope frozen | **MET** |
| F4 TTY refuse before read; same usage const | **MET** |
| F5 const in `ingest.rs`; single-quoted JSON; after_help multiline object | **MET** |
| F6 helper keeps `is_tty: bool` | **MET** |
| F8 no DTO / daemon ingest | **MET** |
| F9 vault still required | **MET** |
| F10 no pin bumps | **MET** on lock/workspace |
| F11 isolation | **MET** on inspected surface |
| F13 empty field ≠ empty stdin | **MET** (AC10) |
| F14 high-risk anti-patterns (all-serde→usage, TTY hang, unquoted `{`, T180 flip, skip vault, print key) | **ABSENT** |
| F15 capture independence | **MET** (string gate only) |

## Notes (not findings)

- **AC10 tightness:** does not re-assert the exact `content field is empty` string or `code == 1`. An unrelated exit 1 (e.g. vault-open) could still satisfy `!success && code != 2`. Pre-T252 already only checked `!success`. The **new** asserts are exactly F13 (not usage). Empty-content JSON is non-empty, so the new gate cannot fire.
- **AC5 `||`:** specified by AC5. Production envelope contains both tokens.
- **Helper TTY+payload:** AC6 does not require `ingest_stdin_needs_usage(true, Some("{"))`. `run()` never feeds TTY through the helper after a read (correct: must not read).
- **T180 unknown-field message `\|\| !err.is_empty()`:** pre-existing weak arm; isolation forbids rewriting. `!success` still fails if deny_unknown_fields is dropped.
- **Live whitespace hermetic:** not an AC. Same pre-`dry_run` trim gate as AC2.
- **AC14 / AC15:** not executed in this pass.

## Isolation / non-goals

- `parse_ingest_request` (`ai-brains-capture` `malformed.rs`) unchanged: required fields, role enum, empty-string field errors, then `IngestRequest` deserialize.
- `DryRunIngestRequest` structurally unchanged (T114 / T180 F26).
- T86 `read_query_from_stdin` / `read_json_from_stdin` (including parse swallow → `Object`) not reused (F12).
- Ingest not added to `is_vault_path_free` / `run_sync_path_free`.
- `cli_help_ia` group order untouched.
- No `AI_BRAINS_KEY` print.

## Closeout (not done by this review)

Do **not** mark the track completed from this pass. Remaining items stay with the implementer: targeted nextest (AC15), AC14 TTY no-hang dogfood, workspace CI gate, `review.md` / cross-model if required, conductor Completed, ledger commit.
