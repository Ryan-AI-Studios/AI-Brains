# T192 Internal Review R1

**Track:** T192 Doctor CLI  
**Reviewer:** Grok Build (read-only internal)  
**Date:** 2026-08-02  
**Scope:** contracts doctor DTO, F17b brain backup read paths, `commands/doctor.rs` + early `main` handler, `doctor_cli` hermetic tests, docs/claims flip, F16 live event_type storage  

## Verdict: PASS

Implementation matches the frozen check matrix, exit policy, read-only open path, F17b non-mutating backup reads, kit reparse refuse, and docs/claims honesty for R-DOC-CLI. No P0/P1 product defects found. Residual gaps are test/process polish (AC8 automation, daemon-up half, closeout strike of deferred #2) and documented honesty residuals (offline kit without `--kit-path`).

---

## Requirement matrix (AC1–16, F highlights)

### Acceptance criteria

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** | **Met** | `Commands::Doctor` in `crates/ai-brains-cli/src/main.rs:204–226`; test `doctor__help__lists_command` (`doctor_cli.rs:589–599`) |
| **AC2** | **Met** | `doctor__happy_temp_vault__ok_or_degraded_exit_0` — exit 0, status ok\|degraded, kit_file Skip (`doctor_cli.rs:65–98`) |
| **AC3** | **Met** | `doctor__wrong_key__fail_exit_1` — exit 1, `vault_open` Fail, no key leakage (`doctor_cli.rs:100–131`) |
| **AC4** | **Met** | `doctor__missing_vault__fail_no_create` — exit 1, vault not created (`doctor_cli.rs:133–163`) |
| **AC5** | **Partial** | Daemon-down path: hermetic `doctor__daemon_up_or_down__not_fail` asserts severity Ok + message up\|down without overall Fail (`doctor_cli.rs:165–199`). Daemon-**up** only injectable via `run_with_daemon_state(..., true)` — no hermetic/unit assertion that message is `"up"` while still non-fail. Probe never hard-fails alone (`doctor.rs:137–142`) |
| **AC6** | **Met** (Windows reparse soft-skip residual) | Good kit Ok (`doctor__kit_path_good__…`); bad pass Fail exit 1 (`doctor__kit_path_bad_pass__…`); reparse Fail on Unix always, Windows only if `mklink` succeeds else eprintln skip (`doctor_cli.rs:282–373`); F15b refuse in `check_recovery_kit_file` (`doctor.rs:343–357`) |
| **AC7** | **Met** | `doctor__stdout__no_secrets` + wrong-key leakage asserts (`doctor_cli.rs:489–520`, `129–130`); report messages avoid key/kit dump |
| **AC8** | **Partial** | **Functional Met by structure:** early handler before `AppContext::from_cli` (`main.rs:1942–1968`); only `VaultConnection::open_read_intent` (`doctor.rs:71`); match arm `unreachable!("doctor handled before AppContext")` (`main.rs:2022`). **Automated Met gap:** plan test `doctor__no_migrate_while_daemon_up` **absent** from `doctor_cli.rs` |
| **AC9** | **Met** | `doctor__json__schema_v1_deserializes` + `doctor__default_format_human__without_json_flag` (`doctor_cli.rs:405–451`); contracts `schema_version=1`, clap `default_value = "human"` (`main.rs:207–208`); no `OutputFormat::parse` |
| **AC10** | **Met** | `doctor__fail_on_degraded__exit_1` (`doctor_cli.rs:453–487`); `exit_code_for` (`doctor.rs:529–537`) |
| **AC11** | **Met** | Docs flipped: CAPABILITIES / INSTALL / OPERATIONS / RECOVERY-DRILLS / SECURITY-LIMITS / RELEASE-CLAIMS / PROTOCOL-COMPAT / Docs/README / CHANGELOG / RELEASE-CHECKLIST. Claims script: rule #54 removed with comment (`scripts/check-release-claims.ps1:54`) |
| **AC12** | **Met** | No `humantime` in workspace `Cargo.toml` / cli deps; reuses `parse_duration` + existing `rpassword`. (deny/audit not re-run in this read-only review — code path has zero new deps) |
| **AC13** | **Partial** (process) | Implementation ready for gate; **deferred #2 not struck** (`conductor/deferred.md:16` still open T192); conductor still In Progress. Expected until closeout E5 |
| **AC14** | **Met** | Doctor module deps: store open_read_intent, brain backup list, crypto kit, daemon probe — no models/graph/embed load path in `commands/doctor.rs` |
| **AC15** | **Met** | F17b `backup_dir_read_only` (`backup.rs:62–72`); `list_backups` / `find_backup_files` / `preview_backup_path` non-creating; tests `list_backups__missing_dir__no_create`, `preview_backup_path__missing_dir__no_create`, `run_backup__creates_dir` (`backup.rs:732–804`); CLI `doctor__no_backups_dir__does_not_create` (`doctor_cli.rs:522–555`) |
| **AC16** | **Met** | Live storage: `event_store` `trim_matches('"')` so column is bare `RecoveryKitCreated` (`event_store.rs:121–124`); doctor queries unquoted kind (`doctor.rs:301–332`); `doctor__recovery_kit_created_event__ok_not_false_warn` (`doctor_cli.rs:557–586`) |

### Freeze highlights (F1–F34)

| ID | Status | Notes |
|----|--------|-------|
| **F1** CLI name | Satisfied | Top-level `Doctor` subcommand |
| **F2** Read-only | Satisfied | No migrate/create vault/append; F17b no `backups/` create on list |
| **F3** open_read_intent | Satisfied | `doctor.rs:71` only open path |
| **F4** Key resolution | Satisfied | Same zero-key default + `ALLOW_ZERO_KEY` gate via store policy; never prints key |
| **F5** No AppContext | Satisfied | Early async handler before `AppContext::from_cli` |
| **F6** Daemon info | Satisfied (honesty residual) | up/down both Ok severity; probe API is bool only — cannot emit Warn on “probe error” separately from down (same as T188 probe surface) |
| **F7–F8** Inventory + roll-up | Satisfied | Fixed 10-check order; `DoctorReport::roll_up` fail≻degraded≻ok; skip does not degrade |
| **F9** Exit codes | Satisfied | 0 ok\|degraded; 1 fail; `--fail-on-degraded`; no 3–7 |
| **F10** Format | Satisfied | human default + `--json` override; not `OutputFormat::parse` |
| **F11** Contracts | Satisfied | `DoctorStatus` / `CheckSeverity` / `DoctorReport` schema_version=1; skip empty optionals; contracts unit tests |
| **F12–F14** hard/soft/info | Satisfied | Matrix implemented in `build_report` |
| **F15 / F15b** kit path + reparse | Satisfied | Skip without path; unlock+key match; reparse refuse before read |
| **F16** kit event query | Satisfied (live truth) | Unquoted `RecoveryKitCreated` (not draft `'"…"'`); AC16 proves; code comment documents draft-vs-live (`doctor.rs:147–148`, `301–306`) |
| **F17 / F17b** backup age + read-only dir | Satisfied | `parse_duration` pub; `and_utc()` age; read paths non-mutating; write paths still `backup_dir()` create |
| **F18–F21** cipher / schema / zero-key / full | Satisfied | `cipher_version`, `has_core_tables` (both tables), zero-key/env warn, integrity on `--full` only |
| **F22** secrets | Satisfied | Leakage tests + no kit dump |
| **F23** zero new deps | Satisfied | No humantime |
| **F24** capture independence | Satisfied | No model/graph requirement |
| **F25** hermetic tests | Satisfied | `common::hermetic_bin` + tempdir |
| **F26** docs/claims | Satisfied | Rule #54 removed; elevated docs flipped |
| **F27** deferred strike | Gap (closeout) | Still open until E5 |
| **F28** domain boundary | Satisfied | CLI adapter + brain F17b helper |
| **F29** after_help | Satisfied | `main.rs:201–203` |
| **F30** determinism | Satisfied | Fixed check order; generated_at wall-clock (allowed) |
| **F31** error before report | Partial | Vault-path missing → Err before report (OK). Invalid key format → Err without `DoctorReport` (rare; prefer report when path resolved) |
| **F32** non-claims | Satisfied | Docs residual language for offline kit |
| **F33** no new dirs | Satisfied | AC15 hermetic assert |
| **F34** no `--passphrase` argv | Satisfied | Only `--passphrase-file` on Doctor |

---

## Findings

### [P2] Missing automated AC8 no-migrate / no-AppContext regression test
Confidence: **High**  
Location: `C:\dev\AI-Brains\crates\ai-brains-cli\tests\doctor_cli.rs` (absent); plan Phase C test list; structural wire `main.rs:1942–1968`, `doctor.rs:71`  
Problem: AC8 requires spy/assert that doctor does not migrate. Wiring is correct today, but the planned hermetic/unit guard (`doctor__no_migrate_while_daemon_up`) was not landed. A future refactor that routes `Doctor` through `AppContext::from_cli` would not be caught by the current 15 tests.  
Evidence: Plan.md C tests include `doctor__no_migrate_while_daemon_up`; grepping `doctor_cli.rs` shows no migrate/AppContext assertion. Fifteen tests cover happy/fail/kit/format/backups/event — not migrate.  
Correction: Add a cheap regression: (a) unit test that `build_report` / runner never calls migrate (already true via open_read_intent-only module), and/or (b) hermetic: init vault, record mtime/schema version, run doctor with injectable `daemon_up=true`, assert no schema bump / no writer side effects; or static test that `Commands::Doctor` is handled only in the pre-AppContext branch (compile-time/unreachable already helps).  
Deferrable: **No** (easy, AC-linked; fix before closeout preferred)

### [P3] AC5 daemon-up message path untested
Confidence: **High**  
Location: `doctor.rs:41–48` (`run_with_daemon_state`), `doctor_cli.rs:165–199`  
Problem: Integration test only observes hermetic daemon-down (`message` down). Injectable `daemon_up: bool` exists but no unit test asserts `daemon_reachable` message `"up"` with severity Ok and non-fail roll-up.  
Evidence: Only process spawn tests; no `build_report`/`run_with_daemon_state` call with `true`.  
Correction: One unit test: `build_report(..., daemon_up: true)` → check name `daemon_reachable`, severity Ok, message `"up"`.  
Deferrable: **Yes** (behavior trivial; structure already correct)

### [P3] `process::exit` inside public `run_with_daemon_state`
Confidence: **High**  
Location: `crates/ai-brains-cli/src/commands/doctor.rs:42–52`  
Problem: Public runner maps fail/degraded-with-flag to `process::exit(code)` after emit. Fine for the binary entrypoint; unsafe for any in-process library/test caller of `run_with_daemon_state` (skips Drop unwind, kills test process). Unit tests correctly use `exit_code_for` only — so current suite is safe.  
Evidence: `if code != 0 { process::exit(code); }`; `run` is `pub async` and `run_with_daemon_state` is `pub`.  
Correction (optional polish): Return `Result<i32, _>` or map exit only in `main`/`run` after `build_report`+`emit`; keep pure `exit_code_for` for injectors. Align with recovery if desired (recovery returns Err rather than exit for most failures).  
Deferrable: **Yes** (CLI-only adapter; matches other command exit patterns)

### [P3] Invalid `--format` values silently treated as human
Confidence: **Medium**  
Location: `doctor.rs:539–565`; clap `format: String` with `default_value = "human"` (`main.rs:207–208`)  
Problem: Spec surface is `human|json`. Values like `--format pretty` fall through to human (not clap usage exit 2), unlike a `ValueEnum`/`value_parser`.  
Evidence: `use_json = force_json || format.eq_ignore_ascii_case("json")` — anything else is human.  
Correction: Restrict with clap `value_parser = ["human", "json"]` or enum.  
Deferrable: **Yes**

### [P3] Spec F16 draft still describes JSON-quoted `event_type` (docs/spec drift)
Confidence: **High**  
Location: `spec.md` F16 / live baseline row; implementer truth `event_store.rs:121–124` + `doctor.rs:301–306`  
Problem: Freeze text says column stores `'"RecoveryKitCreated"'` with embedded quotes. Live code trims quotes on insert; doctor correctly queries bare `RecoveryKitCreated`. AC16 green. Spec text is stale relative to implementer note.  
Evidence: Insert path `trim_matches('"')`; doctor `const KIND: &str = "RecoveryKitCreated"`; test after `recovery export` finds event.  
Correction: Amend F16/live baseline in spec (or errata in review log) so future readers do not “fix” to wrong SQL.  
Deferrable: **Yes** (code correct; documentation honesty)

---

## Completeness sweep

| Check | Result |
|-------|--------|
| TODO/FIXME/stub/placeholder in doctor | **None** in `commands/doctor.rs` |
| No-op paths | Integrity skip when not `--full`; kit_file skip without path — intentional |
| Silent `create_dir` | F17b fixed for list/find/preview; write paths still create (required) |
| AppContext misuse | **None** — early handler only |
| Secrets leakage | Tests + no kit/key in messages |
| humantime dep | **Absent** |
| `has_core_tables` both tables | Requires `events` **and** `memory_projection` (`backup.rs:436–451`). Callers: doctor `schema_readable` + backup metadata probe only. Pre-existing AND semantics; pub export only — **low regression risk** |
| Zero-key / `ALLOW_ZERO_KEY` | Store `enforce_key_policy` on open; doctor `check_zero_key_escape` warns on zero key **or** env set (`doctor.rs:480–500`); hermetic tests set env via `hermetic_bin` |
| Docs residual offline kit | Honest across CAPABILITIES, RECOVERY-DRILLS, RELEASE-CLAIMS R-DOC-CLI, SECURITY-LIMITS §6 |
| Wiring CLI → checks → report → exit | `main` → `doctor::run` → probe → `build_report` → `emit_report` → `exit_code_for` / `process::exit` |

---

## Tests quality

| Question | Assessment |
|----------|------------|
| Would tests fail on old behavior? | **Yes** for core: no `Commands::Doctor` → help/spawn fail; wrong-key must exit 1 with report; AC15 fails if list creates `backups/`; AC16 fails if SQL quoting wrong; human default fails if Json default reused |
| Missing critical cases? | No-migrate (AC8); daemon-up inject; optional `--full` integrity; non-zero production key path (happy always zero-key hermetic → always degraded via zero_key + no backups) |
| Count | 15 integration tests in `doctor_cli.rs` + contracts roll-up/serde + doctor unit exit_code/order + brain F17b tests |

---

## Gaps / residual notes

1. **Closeout (AC13 / F27):** Strike deferred **#2**, mark conductor Completed, full gate (fmt/clippy/nextest/deny/audit), ledger commit — not done at R1 (expected mid-track).  
2. **Offline kit without `--kit-path`:** Documented residual; event ≠ file existence; matches F15/F32.  
3. **Daemon probe = our IPC only:** after_help + CAPABILITIES honesty; F6 residual.  
4. **TTY-smart format default:** Soft residual (F10 fixed human).  
5. **Windows reparse kit test:** Soft-skips without Developer Mode/admin `mklink`.  
6. **When backups already exist:** `list_backups` still uses RW `Connection::open` for metadata (pre-existing backup list behavior). AC15/F17b only required no `backups/` directory creation; not elevated to a finding for empty-dir doctor purity.  
7. **SECURITY.md:** Spec F26 lists `SECURITY.md`; repo surface is `SECURITY-LIMITS.md` (flipped). No separate SECURITY.md product file found.  
8. **F16 live truth** correctly preferred over early draft; keep AC16 as permanent regression guard.

---

## Summary judgment

Ship-quality operator surface for T192 freezes: contracts v1, early read-only handler, full check matrix, F17b, hermetic coverage for AC1–4,6–7,9–12,14–16, and docs/claims flip including rule #54 removal. Address **P2 AC8 regression test** before track closeout; remaining P3s may defer to ISSUES with one-line notes. **Verdict PASS.**
