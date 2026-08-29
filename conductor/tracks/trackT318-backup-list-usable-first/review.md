# T318 Review Log — backup list usable-first

**Track:** T318-BackupListUsableFirst  
**Status:** Completed (gates green; publish pending)  
**FEATURE TX:** `93fbf235-8dc2-40d8-add1-9ac9bfc2643b`  
**Branch:** `track/T318-backup-list-usable-first`

## Scope implemented

- Default/Quiet `backup list` emits usable rows only (`is_usable_class`); residuals-only → `No usable backups.`
- Residual F6 summary moved from stderr → **stdout** (SOOT unchanged)
- Verbose still prints every class; no footer
- Mixed `backup verify` (`ok >= 1`): counts + `format_mixed_fail_trailer`; no first-5 `FAIL —`
- Zero-OK verify keeps T225 first-5 + create nudge
- List `after_help` one sentence; docs CAPABILITIES/OPERATIONS/CHANGELOG
- Isolation: no `doctor.rs` / brain `backup.rs` / `project.rs` / `forget.rs` production diffs

## Internal review vs DoD / ACs

| AC | Result | Evidence |
|----|--------|----------|
| AC1 mixed usable-only table | **met** | `backup_list_honesty__mixed_usable_and_residual__usable_first` PASS |
| AC2 footer stdout / not stderr | **met** | mixed + F31 census hermetics PASS |
| AC3 all-residual `No usable backups.` | **met** | `backup_list__all_residual__no_usable_and_footer` + F31 flips PASS |
| AC4 verbose tokens, no footer | **met** | `backup_list_honesty__verbose_plain__per_file_detail` PASS |
| AC5 quiet mixed usable, no footer | **met** | `backup_list_honesty__quiet_mixed__usable_row_no_footer` PASS |
| AC20 quiet all-residual | **met** | `__quiet__no_summary` + `__quiet_and_verbose__quiet_wins` PASS |
| AC6 empty `No backups found.` | **met** | `backup_list__empty__no_backups_found_exit_0` PASS |
| AC7 list_sort units | **met** | `list_sort_tests` PASS |
| AC8 mixed verify no `FAIL —` | **met** | smoke `backup_verify_all__mixed__reports_per_file` + recoverable verify + `format_mixed_trailer__contains_verbose_and_count` PASS |
| AC9 zero-OK 5-FAIL + nudge | **met** | `backup_verify__multi_fail__preview_cap_trailer_and_nudge` PASS |
| AC10 verbose mixed stream | **met** | `backup_verify_all__mixed__verbose_per_file_stream` PASS |
| AC13 create help | **met** | `backup_create_help__after_help__mentions_no_prune_default_dir` PASS |
| AC14 list after_help | **met** | `backup_list_help__after_help__names_usable_only_and_verbose` PASS |
| AC15 docs | **met** | CAPABILITIES/OPERATIONS/CHANGELOG T318 sentences |
| AC16 isolation | **met** | `git diff` empty for doctor/brain backup/project/forget |
| AC17 manual | **met** | `cargo run`: list exit 0 — 1 usable row + stdout `22 backup(s) not recoverable…`; stderr empty. verify exit 1 — `Verified 23 backups: 1 OK, 22 FAIL.` + `22 FAIL (use --verbose for per-file).`; no `FAIL —`; stderr empty. N=23 |
| AC11 JSON verify | **met** | stay-green T225 JSON (unchanged arm) |
| AC12 empty verify | **met** | stay-green T198 |
| AC18 exits | **met** | list exit 0 hermetics; verify FAIL→1 |
| AC19 T277 mixed doctor Ok | **met** | `backup_create__key_mismatch_residual__new_readable_and_doctor_ok` PASS |

## Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1-1 | low-info | PATH `ai-brains` until owner `cargo install` still pre-T318 emit | deferred | F27 — hermetic/`cargo run` SoT |
| R1-2 | low-info | Live residual fleet still 22 KeyMismatch/plain/Incomplete | deferred | F12 — expected; verify exit 1 |
| R1-3 | low-info | T209 L3 real wrong-key SQLCipher fixture still soft | deferred | Declined F13/soft; not this DoD |
| R1-4 | low-info | verify JSON `summary` / `--quiet` / class-aware prune declined | deferred | F13 |
| CX-P1-1 | process | Completion gates / uncommitted mid-review | verified_fixed | Full gate EXIT 0 (`NEXTEST_TEST_THREADS=2`); commits + publish |
| CX-P1-2 | process | Red→green commit evidence absent at CX time | verified_fixed | FEATURE product commits before PR |
| CX-P2-1 | low | Stale List clap about / verbose Incomplete | verified_fixed | `main.rs` List about + quiet/verbose docs updated |
| CX-P2-2 | low-info | PreT109 Default visibility lacks dedicated hermetic | deferred | `is_usable_class` already includes PreT109; readable mixed hermetic covers usable band; dedicated fixture not this DoD |
| CX-P3-1 | low | Mixed trailer unit substring-only | verified_fixed | `assert_eq!` exact SOOT |

No critical/high/medium open. Easy lows fixed in-scope.

## Cross-model (Codex)

- File: `review.codex.md` — product core **PASS**; process P1 mid-implement (expected); P2 help fixed; PreT109 deferred; P3 exact assert fixed.
- Internal explore review: AC1–AC20 met; no medium+ code blockers.

## Targeted gates (observed)

- Units + honesty + recoverable + smoke mixed/zero-OK/verbose: **PASS**
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`: **PASS**
- `cargo fmt --check`: **PASS**
- Manual AC17: **PASS** (N=23)
- Full gate (`fmt` + workspace clippy + workspace nextest threads=2 + deny + audit): **PASS** EXIT 0
- Unrelated: default-parallel recall timeouts → re-run threads=2 PASS (deferred low-info)
