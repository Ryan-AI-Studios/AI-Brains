Now I have a complete picture. Let me produce the audit report.

---

# Track Completion Audit ΓÇö T209

## Verdict: PASS WITH DEFERRED P3

---

## Scope Reviewed

| Item | Path |
|------|------|
| Spec | `conductor/tracks/trackT209-backup-list-sqlcipher-honesty/spec.md` |
| Plan | `conductor/tracks/trackT209-backup-list-sqlcipher-honesty/plan.md` |
| Internal review | `conductor/tracks/trackT209-backup-list-sqlcipher-honesty/review.md` |
| Brain lib | `crates/ai-brains-brain/src/backup.rs` |
| Brain re-exports | `crates/ai-brains-brain/src/lib.rs` |
| CLI backup command | `crates/ai-brains-cli/src/commands/backup.rs` |
| CLI doctor command | `crates/ai-brains-cli/src/commands/doctor.rs` |
| CLI clap wiring | `crates/ai-brains-cli/src/main.rs` |
| Hermetic tests | `crates/ai-brains-cli/tests/backup_list_honesty.rs` |
| Smoke tests | `crates/ai-brains-cli/tests/smoke.rs` |
| Common test helpers | `crates/ai-brains-cli/tests/common/mod.rs` |
| Docs | `Docs/CAPABILITIES.md`, `CHANGELOG.md`, `Docs/OPERATIONS.md` |
| Conductor/deferred | `conductor/conductor.md`, `conductor/deferred.md` |

Branch: `track/T209-backup-list-sqlcipher-honesty` vs `origin/main`. Implementation uncommitted (working tree), no PR yet (D4 pending).

---

## Requirement and DoD Matrix

| Req / DoD | Status | Evidence |
|-----------|--------|----------|
| **F2/F3** Header-first SOOT; no duplicate magic | Γ£à | `classify_backup_read` calls `is_plain_sqlite_header(path)` first (backup.rs:433); plain ΓåÆ immediate `LegacyPlain`; no key probe |
| **F4** PreT109 preserved (`debug!` only) | Γ£à | `emit_list_noise` PreT109 always `debug!` (backup.rs:506-510); AC6 smoke anchors regression |
| **F5** Default noise (Corrupt `warn!`; LegacyPlain/KeyMismatch/PreT109 `debug!`) | Γ£à | `emit_list_noise` switch correct for all 5 classes |
| **F6** Summary `eprintln!` (template; Default only; zero for Readable/PreT109-only) | Γ£à | `run_list` backup.rs:211-214; counts only LegacyPlain+KeyMismatch; template `"not fully readable (legacy plain or current key): use --verbose or ai-brains backup verify"` |
| **F7** `--verbose` per-file detail; no summary | Γ£à | Verbose ΓåÆ `warn!` for LegacyPlain/KeyMismatch/Corrupt; summary gate `mode == ListMode::Default` |
| **F8** Quiet wins; no `conflicts_with` | Γ£à | `from_flags(quiet, verbose)`: quiet wins; `BackupCommands::List` has no `conflicts_with` |
| **F9** Table tokens `(legacy plain)` / `(unreadable key)` / `(corrupt)` / `(no metadata)` | Γ£à | `empty_meta_token()` at backup.rs CLI:142-149 |
| **F10** Exit 0 on any class mix | Γ£à | All hermetic ACs assert `status.code() == Some(0)` |
| **F12/M3** Doctor `ListMode::Quiet` | Γ£à | doctor.rs:305 `service.list_backups(ListMode::Quiet)` |
| **F13** Verify still refuses plain | Γ£à | `verify_single_backup` checks `is_plain_sqlite_header` first (CLI backup.rs:368) |
| **F14** `ListMode` + `BackupReadClass` + `from_flags` + `BackupInfo.class` | Γ£à | `BackupReadClass` (5 variants, Default), `ListMode` (3 variants), `from_flags`, `BackupInfo.class`; all re-exported from `ai_brains_brain` lib.rs:17-19 |
| **F15** Zero new crates | Γ£à | No new dependencies added |
| **F16/M5** AC1 `env_remove("RUST_LOG")`; AC2 `.env("RUST_LOG", "warn")` | Γ£à | `AMBIENT_DENYLIST` includes `"RUST_LOG"` (common/mod.rs:61); `hermetic_bin()` strips it; `list_output` with `None` = no re-set; AC2 uses `Some("warn")` |
| **F18** No key logging | Γ£à | All tracing calls log `path = %path.display()` only; key passed only to `apply_key_pragmas` |
| **F21** Deterministic summary template | Γ£à | Fixed `eprintln!` template with `{residual_count}`; path sort newest-first unchanged |
| **F22** Unit B1ΓÇôB4 + hermetic AC1ΓÇôAC5/AC7/AC9 | Γ£à | See test matrix below |
| **F23/AC8** CAPABILITIES ┬º11 + CHANGELOG + OPERATIONS | Γ£à | CAPABILITIES ┬º11 table with 5 classes; CHANGELOG Unreleased T209 entry; OPERATIONS one-liner |
| **F24 (soft)** `#[derive(Serialize)]` on `BackupReadClass` | Γ£à | backup.rs:15 `serde::Serialize` |
| **F29** `sqlcipher_log_policy::install()` unchanged | Γ£à | main.rs:1752 untouched |
| **F31** `MIN_PLAUSIBLE_BACKUP_BYTES = 512`; size gate | Γ£à | backup.rs:12; classify: metadata Err ΓåÆ Corrupt; open Err ΓåÆ Corrupt; key/schema fail + len<512 ΓåÆ Corrupt; + lenΓëÑ512 ΓåÆ KeyMismatch |
| **F33** Plain fixture uses valid SQLite magic | Γ£à | `write_plain_bak` writes `b"SQLite format 3\0"` + padding |
| **F34** AC2/AC9 non-conflict (separate tests) | Γ£à | `backup_list_honesty__short_garbage_rust_log_warn__corrupt_warn` vs `backup_list_honesty__large_key_mismatch__summary_not_warn_flood` |
| **No `list_backups(bool)` leftovers** | Γ£à | All call sites use `ListMode`; old signature gone |
| **AC1** Plain + unset RUST_LOG ΓåÆ `(legacy plain)` + no per-file WARN + summary | Γ£à | Hermetic test; denylist strips RUST_LOG; `debug!` suppressed by `ai_brains_brain=info` default filter |
| **AC2** Short garbage + RUST_LOG=warn ΓåÆ Corrupt WARN | Γ£à | Hermetic test; 27-byte file < 512; `(corrupt)` token |
| **AC3** ΓëÑ2 plain ΓåÆ Γëñ1 `eprintln` summary | Γ£à | Hermetic test; single `eprintln!` path; 2 counted in one call |
| **AC4** `--verbose` per-file detail | Γ£à | Hermetic test; `combined.contains("legacy plaintext")` (WARN message only, not filename column ΓÇö M1 fix verified) |
| **AC5** `--quiet` no summary; dual `--quiet --verbose` quiet wins | Γ£à | Two hermetic tests; `from_flags(true, true) = Quiet`; `debug!` suppressed |
| **AC6** Pre-T109 no WARN | Γ£à | smoke.rs `backup_list__pre_t109_backup__no_warn_on_stderr`; real keyed bak stripped of meta table; `RUST_LOG=warn` |
| **AC7** Readable path-end + no residual tokens | Γ£à | Hermetic test; `truncate_right` preserves path tail; real backup metadata |
| **AC9** Large wrong-key ΓåÆ `(unreadable key)` summary; no per-file WARN flood | Γ£à | Hermetic test; 600-byte 0xAB file ΓëÑ 512; KeyMismatch ΓåÆ `debug!` under Default mode |
| **AC10 (soft)** Doctor healthy with plain residual | Γ¼£ soft | Code uses `ListMode::Quiet`; no hermetic test (allowed soft) |

---

## Findings

### P3 ΓÇö Deferred low-priority items (from internal review R1, disposition confirmed)

**P3-L2 ΓÇö AC6 smoke assertion is OR-weak**

`backup_list__pre_t109_backup__no_warn_on_stderr` (smoke.rs:833-836) uses:
```rust
assert!(
    !stderr.contains("WARN") || !stderr.contains("backup metadata"),
    ΓÇª
);
```
This is `NOT(WARN AND "backup metadata")`. A future regression that re-introduces a WARN with different wording (no "backup metadata") would pass silently. The implementation is correct ΓÇö `emit_list_noise` for PreT109 always emits `debug!` ΓÇö so the test anchors the regression case adequately, but its precision is low.

**Risk:** Low. The production behavior is correct; only the test's discriminating power is weak.  
**Deferred candidate:** Add a strict `!stderr.contains("WARN")` assert, or add a unit test that `classify_backup_read` on a PreT109 bak returns `BackupReadClass::PreT109` (which already flows through `debug!` in `emit_list_noise`).

---

**P3-L3 ΓÇö AC9 fixture uses non-SQLCipher garbage, not a real rotate-wrong-key encrypted bak**

The B3 unit and hermetic AC9 test use `vec![0xABu8; 600]` ΓÇö opaque bytes that are ΓëÑ 512 and not plain-header. This correctly exercises the F31 size discriminator (error strings cannot split garbage vs wrong-key). It does not exercise the real vault-rotate-then-list path end-to-end.

**Risk:** Very low. F31 explicitly states the discriminator is size-based because error strings cannot distinguish; F34 elevated AC9 to required for exactly this reason. The test proves the code path.  
**Deferred candidate:** Soft ΓÇö build a real SQLCipher backup, list with a wrong key, verify `(unreadable key)`. Not required for F31/F34.

---

**P3-L4 ΓÇö No dedicated unit for `BackupReadClass::PreT109`**

Units B1ΓÇôB4 cover LegacyPlain / Corrupt / KeyMismatch / Readable. PreT109 (key opens, meta table absent) is covered only by the AC6 smoke test. Spec F22 wording covers "plain / garbage<512 / large key-fail / readable" ΓÇö PreT109 is not listed as a required unit. Smoke test covers the regression.

**Risk:** Negligible.  
**Deferred candidate:** Add a unit that creates a real bak, drops `_aibrains_backup_meta`, and asserts `classify_backup_read ΓåÆ (PreT109, {})`.

---

## Completeness Sweep

| Area | Verdict |
|------|---------|
| Placeholders / stubs in classify path | None ΓÇö `classify_backup_read` has no `todo!`, `unimplemented!`, `unreachable!`, or no-op arms |
| `list_backups(bool)` leftovers | None found; all call sites use `ListMode` |
| Old `read_backup_metadata` call sites in list path | Removed; list now uses `classify_backup_read` which reads meta internally |
| All 5 `BackupReadClass` variants handled in `emit_list_noise` | Exhaustive `match class { Readable | PreT109 | LegacyPlain | KeyMismatch | Corrupt }` |
| All 3 `ListMode` variants handled where mode matters | Correct for LegacyPlain, KeyMismatch, Corrupt arms |
| Summary gating | `mode == ListMode::Default && residual_count >= 1` ΓÇö Quiet and Verbose suppress |
| Doctor wiring | `list_backups(ListMode::Quiet)` at doctor.rs:305 |
| Clap wiring | `BackupCommands::List { quiet, verbose }` dispatched via `ListMode::from_flags(*quiet, *verbose)` at main.rs:2886-2889 |
| Re-exports | `BackupReadClass`, `BackupInfo`, `BackupService`, `ListMode` re-exported from `ai_brains_brain` lib.rs |
| Key logging | Zero ΓÇö only `path = %path.display()` in tracing fields |
| F24 soft Serialize | `#[derive(serde::Serialize)]` on `BackupReadClass` |

---

## Wiring and Regression Review

**End-to-end production path:**
1. User calls `ai-brains backup list [--quiet] [--verbose]`  
2. `main.rs` matches `BackupCommands::List { quiet, verbose }` ΓåÆ `ListMode::from_flags(*quiet, *verbose)`  
3. `commands::backup::run_list(&ctx, mode)` ΓåÆ `BackupService::list_backups(mode)`  
4. Per file: `classify_backup_read(&path, &self.key)` ΓåÆ `(BackupReadClass, HashMap)`  
5. `emit_list_noise(&path, class, mode)` ΓÇö per-class, per-mode log level  
6. `run_list` prints table with `empty_meta_token(class)` as fallback  
7. `run_list` emits `eprintln!` summary when `Default` + residual_count ΓëÑ 1  

All steps reachable in production, no dead branches.

**Regression checks:**
- Verify plain refuse (`verify_single_backup`, CLI backup.rs:368): unchanged; plain ΓåÆ error immediately Γ£à  
- Doctor `backup_recent` (doctor.rs:292-362): `list_backups(ListMode::Quiet)` ΓÇö no spurious WARN to doctor's health surface Γ£à  
- Corrupt WARN under Default/Verbose not suppressed: `emit_list_noise` Corrupt arm is `Default | Verbose ΓåÆ warn!` Γ£à  
- Quiet suppresses Corrupt WARN: `Quiet ΓåÆ debug!` for Corrupt Γ£à  

---

## Verification Evidence

**Internal review R1ΓåÆR2 disposition:**
- **T209-R1-M1** (AC4 verbose assert under-constrained): **verified_fixed** ΓÇö assert now requires `"legacy plaintext"` substring from the WARN message, not from the filename column
- **T209-R1-L1** (identical PreT109 arms): **verified_fixed** ΓÇö collapsed; current code has two equivalent `(BackupReadClass::PreT109, HashMap::new())` returns that are distinct error paths (rows `Err` vs prepare `Err`) and are correct

**Confirmed test results (review.md R2):**
```
cargo nextest run -p ai-brains-brain --lib           ΓåÆ 31 passed
cargo nextest run -p ai-brains-cli -E 'test(backup_list)'  ΓåÆ 14 passed (8 honesty + 6 smoke)
cargo clippy -p ai-brains-brain -p ai-brains-cli --all-targets -- -D warnings  ΓåÆ clean
```

**Spec cross-checks confirmed by code inspection:**
- `MIN_PLAUSIBLE_BACKUP_BYTES = 512` declared `pub const` at backup.rs:12
- F31 logic verified: metadata-Err ΓåÆ Corrupt; open-Err ΓåÆ Corrupt; `len < 512` after key-fail ΓåÆ Corrupt; `len >= 512` after key-fail ΓåÆ KeyMismatch
- `AMBIENT_DENYLIST` includes `"RUST_LOG"` (common/mod.rs:61) ΓÇö F16/AC1 env_remove fulfilled
- No `conflicts_with` in `BackupCommands::List` clap definition (main.rs:1516-1523)
- Summary template contains required substrings: `"not fully readable"`, `"--verbose"`, `"verify"` (CLI backup.rs:213)

---

## Deferred Candidates

| ID | Priority | Candidate | Spec anchor |
|----|----------|-----------|-------------|
| T209-R1-L2 | P3 | Strengthen AC6 smoke assert: `!stderr.contains("WARN")` or add PreT109 unit | F4, AC6 |
| T209-R1-L3 | P3 | Soft: build real rotate-wrong-key bak for AC9 end-to-end | F31/F34 soft |
| T209-R1-L4 | P3 low-info | Add unit for `BackupReadClass::PreT109` classify path | F22 soft |

None of these are DoD-blocking. All are low-information or low-risk. Recommend appending to `conductor/deferred.md` under T209 residuals when D4 PR is submitted.

---

## Completion Decision

**Verdict: PASS WITH DEFERRED P3**

All required Frozen Decisions (F1ΓÇôF34 excluding soft F24/F26/F35), all required Acceptance Criteria (AC1ΓÇôAC9; AC10 soft-allowed), and all DoD items are satisfied:

- Γ£à `BackupReadClass` + `ListMode` + `from_flags` + `BackupInfo.class` ΓÇö API shape complete  
- Γ£à Header-first classify (F2/F3) with F31 size gate ΓÇö no false-quiet garbage, no false-plain encrypted  
- Γ£à Default quiet: no per-file WARN flood for LegacyPlain/KeyMismatch; Corrupt still warns  
- Γ£à One `eprintln!` summary under Default; omitted under Quiet and Verbose  
- Γ£à `--verbose` per-file detail; `--quiet` wins at runtime; no `conflicts_with`  
- Γ£à Table tokens honest for all 5 classes  
- Γ£à Doctor uses `ListMode::Quiet`; verify plain regression unchanged  
- Γ£à No key logging; no new crates  
- Γ£à Hermetic suite (AC1ΓÇôAC5/AC7/AC9) + smoke (AC2/AC6) + brain units (B1ΓÇôB4) ΓÇö 14 passing  
- Γ£à CAPABILITIES ┬º11, CHANGELOG Unreleased, OPERATIONS one-liner  
- Γ£à clippy clean on brain + CLI  

Only three low/low-info P3 items remain (L2 assertion quality, L3 fixture realism, L4 PreT109 unit coverage), all pre-existing open items from internal review R1 and appropriately deferred. The implementation correctly solves the live `backup list 4/3` audit signal and closes the T208 residual WARN flood. Ready for D4 (PR ΓåÆ conductor Completed ΓåÆ deferred strike).
