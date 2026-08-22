# T277 review log — Recoverable encrypted backup under the current key

**Track:** T277-RecoverableBackup  
**Status:** Completed (publish pending Phase 6)  
**FEATURE TX:** `e877fd0d-7573-49c1-b76e-758b99116e41`  
**HEAD (implement):** `track/T277-recoverable-backup`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC15 / DoD | **PASS** — AC1 red then green (F42 `drop(dst)` + F43 three asserts); AC2–AC4/AC13 mixed hermetic green; AC7 skipped (owner did not confirm live create); F1–F44 held |
| CX1 | Codex gpt-5.6-sol (FEATURE, F2 data-safety) | **PASS** — no P0–P3; AC7 skip allowed |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

None at R1.

## DoD matrix (AC1–AC15)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `run_backup_from_conn__missing_cores__fails_and_deletes` — red (`Ok` dest present) then green (`is_err` + dest absent + `Incomplete` + `core tables`) |
| AC2 | Met | `backup_create__key_mismatch_residual__new_readable_and_doctor_ok` — new file first, no residual tokens; leftover `(unreadable key)` |
| AC3 | Met | same test — `doctor --json --backup-max-age 7d` `backup_recent` `CheckSeverity::Ok` |
| AC4 | Met | `backup_verify__mixed_ok_and_key_mismatch__one_ok_exit_1_no_nudge` — `1 OK` `1 FAIL`, exit 1, no nudge |
| AC5 | Met | `classify_backup_read__real_backup__readable_with_meta` still green |
| AC6 | Met | `backup_list_honesty` 13 + `doctor__backup_recent__all_incomplete__warn_no_usable` green |
| AC7 | Skipped | `/implement-track 277` did not confirm live `backup create --no-prune`; hermetic is DoD (plan Phase 4) |
| AC8 | Met | CAPABILITIES §11 KEY-change; OPERATIONS Backup; CHANGELOG T277; RECOVERY-DRILLS one-liner |
| AC9 | Met | no production `unwrap`/`expect`/`panic` on F2 path; clap 4.6.1; rusqlite 0.39.0; chrono 0.4.44; serde_json 1.0.150; no DTO keys |
| AC10 | Met | `probe_restore_daemon_busy` still restore/export/doctor/vault only; create does not call it; recovery_drills + restore daemon units green |
| AC11 | Met | `backup.rs` create path has no models/graph |
| AC12 | Met | `main.rs` `:4443` `keep.or(Some(10))` unchanged |
| AC13 | Met | mixed default list stderr `not recoverable under current key` |
| AC14 | Met | `backup_verify__valid_backup__reports_ok` + mixed create substring `Backup created and verified:` |
| AC15 | Met | `dev-check.ps1` SUCCESS nextest **3270** passed / 1 skipped; `ledgerful verify --scope full` exit 0 |

## Targeted gates (R1)

```text
cargo nextest run -p ai-brains-brain backup
  28 passed (incl. AC1)

cargo nextest run -p ai-brains-cli --test backup_recoverable --test backup_list_honesty
  16 passed

cargo nextest run -p ai-brains-cli --test doctor_cli --test smoke --test recovery_drills
  118 passed

cargo clippy -p ai-brains-brain -p ai-brains-cli --all-targets -- -D warnings
  exit 0

.\scripts\dev-check.ps1
  SUCCESS; nextest 3270 passed / 1 skipped; deny + audit 19 allowed

ledgerful verify --scope full
  exit 0
```

## Manual (AC7)

```text
PRE list: 22 residual; T244 vault-2026-08-12T15-50-06.db.bak (unreadable key)
PRE doctor backup_recent: warn, no usable encrypted backup under current key, rem=ai-brains backup create
PRE dry-run --no-prune: C:\dev\ai-brains\backups\vault-<now>.db.bak, size 123043840 (F36 volatile)
Daemon: Stopped
CREATE: skipped — owner did not confirm live backup create --no-prune
Did not cargo install; did not prune residuals; did not restore
```

## Notes

- F2 sequence: meta inserts → `drop(dst)` → `classify_backup_read` → `remove_file` + `Err("{class:?}: missing core tables")` if `!is_usable_class`.
- Existing brain create fixtures that lacked `events`/`memory_projection` now include those tables so they stay proofs of encryption/meta/dir (not Incomplete).
- Integrity-check fail path is pre-T277 and still leaves the dest file; out of scope (F2 is post-meta).
