# T251 Device discoverability — internal r1b

**Track:** T251-DeviceDiscoverability  
**Category:** UX / FEATURE  
**Reviewer:** Grok (read-only except this file)  
**Date:** 2026-08-14  
**Spec:** `conductor/tracks/trackT251-device-discoverability/spec.md`  
**Plan:** `conductor/tracks/trackT251-device-discoverability/plan.md`  
**Primary files:** `crates/ai-brains-cli/src/commands/device.rs`, `crates/ai-brains-cli/src/main.rs`, `crates/ai-brains-cli/tests/device_status_discoverability.rs`

Static review only. Production code and Git were not modified. Nextest / clippy / live vault dogfood (AC14) were **not** re-executed. Track status was **not** marked completed.

## Verdict: CLEAN

No critical/high/medium/low findings. First-class `DeviceCommands::Status` (unit, no flags) prints the shared roster then **always** `next: ai-brains replicate status`. List / fingerprint / replicate isolation holds. T198 plural SOOT and singular error copies are intact. New-behavior hermetics would fail against pre-T251 (`unrecognized subcommand`, exit 2).

## Summary

T251 is presentation / discoverability only. `run_list` and `run_status` share `emit_device_roster`; fingerprint uses `EMPTY_ENROLL_HINT` only (not the emitter). Status is not a `visible_alias` of List. No contracts DTO, no `--format` on Status, no `replicate.rs` rewrite, no `OutputFormat::parse` change, no doctor 16th check, clap workspace pin stays `4.5`.

## Hunt checklist

| # | Hunt | Result |
|---|------|--------|
| 1 | Status as `visible_alias` (would drop `next:`) | **Clean.** `DeviceCommands::Status` is a first-class unit variant immediately after `List`. Only `visible_alias` in `main.rs` are pre-existing (`search`, `statement`, `overwrite`). Dispatch: `DeviceCommands::Status => commands::device::run_status(&ctx)`. Clap unit `device_status__parses` matches the unit variant. |
| 2 | `next:` missing on enrolled path | **Clean.** `run_status` always `emit_device_roster` then `println!("{DEVICE_STATUS_NEXT}")`. No daemon-style conditional. Hermetic `device_status__enrolled_vault__outputs_table_and_next_replicate_status` requires `DEVICE_ID`/`local` **and** the exact next line. |
| 3 | `next:` leaked onto list / fingerprint | **Clean.** `run_list` is roster-only. Fingerprint empty prints the const and returns. Hermetics `device_list__*__does_not_contain_next` and `device_fingerprint__empty_vault__does_not_contain_next` reject `next:`. |
| 4 | T198 copy changed | **Clean.** `EMPTY_ENROLL_HINT` is the exact plural sentence `No enrolled devices. Run \`ai-brains device bootstrap\` first.` Used by list (via emitter), status (via emitter), and fingerprint. Existing `device_fingerprint__no_enroll__bootstrap_message_exit_0` still asserts that sentence. |
| 5 | Singular error copy unified | **Clean.** `device.rs` `load_local_signing_key` (~139) and `replicate.rs` `load_local_device` (~206) still read `No enrolled device on this vault. Run \`ai-brains device bootstrap\` first.` |
| 6 | Silent JSON / `--format` on Status | **Clean.** `Status` is a unit variant (no fields). Hermetic `--format json` is clap exit **2**, `unexpected argument`, not `unrecognized subcommand`, stdout does not start with `{`. |
| 7 | Tests that would not fail against old behavior | **Clean** for new-behavior locks. See § Tests vs pre-T251. Regression hermetics (list / fingerprint / replicate) are *supposed* to pass on old output. |
| 8 | `for`-loop inside a single `#[test]` | **Clean.** T251 hermetics are one-command / one-vault. `last_nonempty_line` is an iterator helper, not a parameterized for-loop. Pre-existing `cli_help_ia` for-loop was not touched. |
| 9 | `unwrap` / `expect` / `panic` in production | **Clean.** New production path is `?` + `println!`. `expect` in `device.rs` is test-only (`#[cfg(test)]`). |
| 10 | Clippy `dead_code` unused wrapper | **Clean.** `emit_device_roster` is used by `run_list` and `run_status`. `EMPTY_ENROLL_HINT` / `DEVICE_STATUS_NEXT` / `run_status` are referenced. No unused adapter. |
| 11 | Help `after_help` missing `ai-brains device status` | **Clean.** Device `after_help` includes that example. Hermetic AC6 asserts combined help contains the string and stdout lists `status`. |
| 12 | Dispatch missing Status arm | **Clean.** `main.rs` ~3734. |
| 13 | Isolation breaches | **Clean** on static inspection. `replicate.rs` still owns the dashboard + `--format json` / `--quiet`; singular error at ~206 unchanged; no T251 markers. `OutputFormat::parse` still T227 lowercase + unknown→Json and is unused by device status. No `DeviceStatusResponse` in `ai-brains-contracts`. Doctor unit still `assert_eq!(report.checks.len(), 15)`. Workspace clap `4.5`. Preflight / T243–T250 product files have no T251 markers. |

## Findings

None.

## Requirement / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** recognized; empty exit 0 | **MET** (static) | First-class `Status`. Hermetic empty: exit 0, not `unrecognized subcommand`. Revoked-only ≡ empty via existing `list_enrolled_devices` (`status IN ('active','local')`); no revoke hermetic (F3 / AI2 L4). |
| **AC2** empty T198 + last `next:` | **MET** | Exact `T198_EMPTY` + `last_nonempty_line` == `next: ai-brains replicate status`. |
| **AC3** enrolled table + `next:` | **MET** | Bootstrap then status: `DEVICE_ID` or `local`, contains next pointer, exit 0. Implementation appends next after the table. |
| **AC4** list no `next:` | **MET** | Empty + enrolled list hermetics. |
| **AC5** fingerprint T198, no `next:` | **MET** | Existing `empty_states_exit_hygiene` test unchanged; companion T251 hermetic rejects `next:`. |
| **AC6** help lists status + example | **MET** | `after_help` + hermetic. |
| **AC7** replicate status unchanged | **MET** (static + hermetic) | Empty replicate still `enrolled_count` / honesty / bootstrap hint. `replicate.rs` `run_status` still JSON keys `relay` / `enrolled_count` / `cursors` / `gap_or_blocked` / `devices` / `honesty`. |
| **AC8** `--format json` clap 2 | **MET** | Hermetic exit 2 + unexpected argument + no silent `{`. |
| **AC9** docs | **MET** | CAPABILITIES OutputFormat rows + operator note; PROTOCOL-COMPAT §5 additive human-only rows; OPERATIONS one-liner; INSTALL §7 tip; CHANGELOG Unreleased **always** appends `next:`; CLI-EXIT-CODES table row + Device status footnote. |
| **AC10** shared const + grep plural | **MET** | Single `EMPTY_ENROLL_HINT`. Singular errors remain at the two loaders. |
| **AC11** no DTO / no Status flags | **MET** | Unit variant; no contracts type. |
| **AC12** `cli_help_ia` | **NOT RE-RUN** | Device after_help is additive; that suite does not snapshot Device examples. No T251 edits in `cli_help_ia.rs`. |
| **AC13** full CI gate | **NOT RE-RUN** | Out of scope for this static pass. |
| **AC14** live empty vault dogfood | **NOT RE-RUN** | Spec: do not bootstrap live vault. Code path matches expected empty + `next:`. |
| **AC15** live list / fingerprint / replicate | **NOT RE-RUN** | Isolation as AC7 / AC4 / AC5. |
| **AC16** plan-only until go | **N/A** | Implementation exists; lock flipped after go. |

### Hard F pins

| F | Status |
|---|--------|
| F1 first-class Status after List, no flags | **MET** |
| F2 body = list + always `next:`; no inline replicate | **MET** |
| F3 no multi-device product fill / no top-level status / no default-subcommand | **MET** |
| F4 shared emitter; fingerprint uses const only; singular errors untouched | **MET** |
| F5 list / fingerprint frozen | **MET** |
| F6 human-only; no JSON DTO | **MET** |
| F7 docs + after_help + exit-code footnote | **MET** |
| F8 no new crates / no clap 5 | **MET** (workspace clap `4.5`) |
| F9 exit 0 empty+enrolled; extra args clap 2 | **MET** |
| F10 no pin bumps | **MET** on static `Cargo.toml` |
| F11 isolation | **MET** on inspected surface |
| F13 T198 exact sentence | **MET** |
| F14 honesty owner stays replicate / Device after_help | **MET** (status does not reprint PQ paragraph) |
| F15 high-risk anti-patterns | **ABSENT** |

## Tests vs pre-T251

Pre-T251: `device status` → clap `unrecognized subcommand 'status'`, exit **2**. List/fingerprint empty = T198 one-liner, no `next:`. Replicate status already a dashboard.

| Test | vs pre-T251 |
|------|-------------|
| `device_status__empty_vault__outputs_hint_and_next_replicate_status` | **Would fail** (exit 2 / unrecognized) |
| `device_status__enrolled_vault__outputs_table_and_next_replicate_status` | **Would fail** |
| `device_status__with_format_json_flag__fails_exit_2` | **Would fail** (`unrecognized subcommand` instead of unexpected `--format`) |
| `device_status__help__lists_status` (`ai-brains device status`) | **Would fail** |
| `device_status__parses` (clap unit) | **Would fail** |
| `device_list__*__does_not_contain_next` | **Would pass** (intentional regression) |
| `device_fingerprint__empty_vault__does_not_contain_next` | **Would pass** (intentional regression) |
| `replicate_status__empty_vault__still_prints_enrolled_count_honesty_hint` | **Would pass** (intentional regression) |
| `device_fingerprint__no_enroll__bootstrap_message_exit_0` | **Would pass** (intentional AC5 lock) |

Note (not a finding): help’s first assert `stdout.contains("status")` alone would match Bootstrap’s `status=local` about-text on the old binary. The same test’s `ai-brains device status` assert, plus the parse unit and AC1 hermetic, still fail closed.

Note (not a finding): enrolled hermetic uses `contains(DEVICE_STATUS_NEXT)` rather than `last_nonempty_line`. AC3 only requires contains; F2 always-append is implemented as a trailing `println!` after the roster.

## Isolation / non-goals

- `replicate.rs` `load_local_device` singular sentence at ~206 unchanged; `run_status` still the T177 dashboard (human + `--format json` + `--quiet`). No T251 comments in that file.
- No `DeviceStatusResponse` / new `ai-brains-contracts` type.
- `OutputFormat::parse` untouched (still unknown → Json). Device status does not call it.
- Doctor 15-check matrix unit still requires `checks.len() == 15`.
- Capture-independent: string emit + existing `list_enrolled_devices`. No new events, crates, or models.
- Live vault bootstrap / daemon start / `AI_BRAINS_KEY` print: not in this change set.

## Closeout (not done by this review)

Do **not** mark the track completed from this pass. Remaining operator/closeout items stay with the implementer: targeted nextest, workspace CI gate, live empty-vault dogfood (no bootstrap), `review.md` / cross-model if required, conductor Completed, pins, ledger commit.
