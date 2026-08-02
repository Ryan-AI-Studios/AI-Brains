# Track Completion Audit — T188

## Verdict: PASS WITH DEFERRED P3

**Track:** T188 — Restore Safety + Recovery Operator Surface  
**Reviewer:** Internal R1 (read-only product audit)  
**Date:** 2026-08-01  
**Branch:** `agent/T188-restore-safety-recovery-export`  
**Scope:** Spec F1–F28 / AC1–AC14 vs implementation + tests + docs honesty  
**Product code:** Acceptable for track completion pending orchestrator closeout (full gate, deferred strike, conductor ✅)

No P0/P1 findings. No P2 that blocks product acceptance. Several P3 residuals and open orchestrator closeout items (AC9 partial, AC10).

---

## Scope Reviewed

| Area | Paths |
|------|--------|
| Spec / plan | `conductor/tracks/trackT188-restore-safety-recovery-export/{spec,plan}.md` |
| Restore hard-fail | `crates/ai-brains-cli/src/commands/backup.rs` (`probe_restore_daemon_busy`, `run_restore*`) |
| Recovery export | `crates/ai-brains-cli/src/commands/recovery.rs` |
| CLI wiring | `crates/ai-brains-cli/src/main.rs` (`RecoveryCommands`, pre-`AppContext` bypass) |
| Crypto | `crates/ai-brains-crypto/src/recovery_kit.rs` (`schema_version`) |
| DataKey hygiene | `crates/ai-brains-cli/src/commands/device.rs` (`data_key_from_sqlcipher`) |
| Deps | root `Cargo.toml` / `ai-brains-cli/Cargo.toml` / `Cargo.lock` (`rpassword` 7.5.4) |
| Tests | unit (`backup.rs`, `recovery.rs`, `device.rs`, `recovery_kit.rs`); integration (`tests/recovery_drills.rs`); crypto (`crypto_recovery.rs`) |
| Docs / claims | `Docs/{RECOVERY-DRILLS,OPERATIONS,CAPABILITIES,INSTALL,SECURITY-LIMITS,RELEASE-CLAIMS,README}.md`, `SECURITY.md`, `CHANGELOG.md` |
| Deferred register | `conductor/deferred.md` §59 residuals #1/#6 (not yet struck) |

**Not claimed as fully executed in this review:** full workspace gate (`fmt`/`clippy`/`nextest`/`deny`/`audit`), live manual daemon drills, D4 as separate SECURITY sign-off beyond this audit.

---

## Requirement and DoD Matrix

### Frozen decisions F1–F22 (+ F-related)

| ID | Status | Evidence |
|----|--------|----------|
| **F1** hard-fail restore | **Met** | `backup.rs:436-438` returns `restore_daemon_busy_message()` before overwrite; unit test vault bytes/mtime/marker unchanged (`backup_restore__daemon_running__fails_no_overwrite`) |
| **F1b** robust probe | **Met** | `probe_restore_daemon_busy`: 3 attempts × 1000ms + 50ms backoff (`backup.rs:351-365`) |
| **F2** `--force` never overrides probe | **Met** | Hard-fail before confirm; test uses `force=true` + `daemon_up=true` and still fails; message mentions force |
| **F3** dry-run + daemon notice | **Met (weak test)** | `RESTORE_DRY_RUN_DAEMON_NOTICE` + print when `daemon_up`; dry-run succeeds; vault unchanged. Notice text asserted via constant, not captured stdout (see P3-1) |
| **F4** probe honesty residual | **Met** | Docs OPERATIONS / RECOVERY-DRILLS: IPC-only residual documented |
| **F5** dual stop guidance | **Met** | Message includes `ai-brains daemon stop` + `sc stop AI-Brains-Daemon` / `ai-brainsd`; unit asserts classes |
| **F6** CLI name | **Met** | `ai-brains recovery export` (`main.rs` `Recovery` + `RecoveryCommands::Export`) |
| **F7** DataKey source + no hex in errors | **Met** | `data_key_from_sqlcipher` length/hex errors omit material; `data_key_from_sqlcipher__malformed__error_has_no_key_material` |
| **F8** passphrase acquire | **Met** | File or TTY; no `--passphrase` argv; min 8 bytes; rpassword zero-echo |
| **F8b** file safety | **Met** | `Zeroizing` read, 8 KiB max, dir refuse, Unix non-regular refuse, no truncate of source file, trailing newline trim |
| **F9** output path | **Met** | `--output` required; kit to file only; stdout path + status; exists refuse without force |
| **F9b** perms / public refuse | **Met** | Unix `mode(0o600)`; Windows `C:\Users\Public` refuse + unit test; best-effort `icacls` ACL |
| **F10** leakage tests | **Met (stdout/stderr)** | Integration asserts no DataKey / ciphertext / passphrase / kit dump; salt/nonce not asserted as forbidden. Tracing capture not automated (code-review pin OK per plan B5.2) |
| **F11** dpapi status | **Met** | Prints `dpapi: present|absent`; integration asserts |
| **F12** event best-effort | **Met (code; weak test)** | `RecoveryKitCreated` + `AggregateType::System` + UUID; dry-run skips; warn on append fail; kit still success. No positive event-append assertion (P3-2) |
| **F13** doctor OOS | **Met** | Docs consistently: doctor absent |
| **F14** export dry-run | **Met** | Validates file (read+zeroize) or TTY present without prompt; no file |
| **F15** NIST playbook | **Met** | Post-drill improvement one-liner in RECOVERY-DRILLS §11 |
| **F16** capture independence | **Met** | Export/restore use vault/key/IPC only |
| **F16b** no migrate while daemon up | **Met** | Export bypasses `AppContext::from_cli` (`main.rs:1817-1841`); event path uses `open_without_migrate` when `daemon_up` |
| **F17** rpassword only new prod dep | **Met** | Workspace pin `rpassword = "7.5"`; lock 7.5.4; Apache-2.0 in deny allowlist |
| **F18** vs T187 | **Met** | Independent; drills use live SQLCipher |
| **F19** schema_version=1 | **Met** | `recovery_kit.rs` field + serde default; generate pins 1; legacy JSON defaults; tests unit + integration + crypto |
| **F20** TTY zero-echo double-entry | **Met** | `rpassword::read_password` twice; match + min length |
| **F21** no secrets in tracing | **Met (review)** | Production `tracing::warn!` paths carry only status strings, not passphrase/key/kit |
| **F22** Argon2 honesty | **Met** | Docs cite m=19456,t=2,p=1; kit JSON still param-free |

### Acceptance criteria AC1–AC14

| AC | Status | Notes |
|----|--------|-------|
| **AC1** | **Met** | Unit: daemon_up + force → err, vault unchanged |
| **AC2** | **Met** | Unit + integration: daemon-down force restore succeeds; force only skips confirm |
| **AC3** | **Met*** | Success path + notice constant; *stdout capture gap → P3-1 |
| **AC4** | **Met** | Unlockable kit; min 8; schema_version=1 |
| **AC5** | **Met*** | stdout/stderr/kit-dump leakage tests; tracing via code review |
| **AC6** | **Met** | Unix 0600 code path; Windows public refuse tested |
| **AC7** | **Met*** | R-DOC-CLI honest across primary docs; one stale DECISION pin (P3-3) |
| **AC8** | **Met** | Sole new prod dep rpassword 7.5.x |
| **AC9** | **Open (orchestrator)** | Targeted tests exist; full gate + formal SECURITY closeout not evidenced here |
| **AC10** | **Open (orchestrator)** | `deferred.md` §59 still lists #1/#6 as T188 promotion, not struck |
| **AC11** | **Met** | T181-F-03 product language hard-fail in RECOVERY-DRILLS matrix |
| **AC12** | **Met** | Malformed DataKey error test |
| **AC13** | **Met** | No AppContext migrate on export; kit OK with daemon_up; event best-effort |
| **AC14** | **Met** | Output exists refuse unit + integration |

### Phase checklist (plan)

| Phase | Status |
|-------|--------|
| A restore hard-fail | Complete in code + tests |
| B recovery export | Complete in code + tests |
| C docs/claims | Substantially complete (P3-3 pin) |
| D closeout | D1 done; D2–D7 open (orchestrator) |

---

## Findings

### P0 — none

### P1 — none

### P2 — none

### P3 findings

#### P3-1 — AC3 dry-run notice not asserted on actual stdout

- **Where:** `crates/ai-brains-cli/src/commands/backup.rs:630-661` (`backup_restore__daemon_running_dry_run__ok_with_notice`)
- **Issue:** Test asserts `RESTORE_DRY_RUN_DAEMON_NOTICE` string content and that dry-run returns `Ok`, but does not capture process/stdout to prove `restore_dry_run_daemon_notice()` ran. Removing the call while leaving the constant would still pass.
- **Production:** Notice is printed (`backup.rs:424-425`, `383-385`); message classes match F3/AC3.
- **Disposition:** Deferred candidate (test hardening).

#### P3-2 — `RecoveryKitCreated` success path unasserted

- **Where:** `recovery.rs:100-109`, `345-376`
- **Issue:** Event append is implemented correctly (System aggregate, UUID payload only, best-effort warn). No test asserts an event row appears when daemon is down / vault writable, nor that dry-run leaves event log untouched (implied by early return).
- **Disposition:** Deferred candidate (test coverage).

#### P3-3 — Stale DECISION pin in RECOVERY-DRILLS

- **Where:** `Docs/RECOVERY-DRILLS.md:197`
- **Text:** `` `DECISION: T181 recovery drills … kit export remains operator residual` ``
- **Conflict:** Same doc §4/§11 and RELEASE-CLAIMS correctly state export **shipped (T188)**.
- **Risk:** Claims scanners / operators could quote the residual line out of context.
- **Disposition:** Fix in closeout doc polish (low risk).

#### P3-4 — F-03 “unit + integration” slightly overstated

- **Where:** `Docs/RECOVERY-DRILLS.md:40`
- **Issue:** Daemon-**up** hard-fail is unit-injected (`run_restore_with_daemon_state(..., daemon_up=true)`). Integration suite covers daemon-**down** restore and export paths, not a live daemon pipe for busy restore.
- **Disposition:** Acceptable residual; optional live-daemon drill later. Not a product safety hole (logic is production-wired to real probe).

#### P3-5 — Restore still opens `AppContext` (migrate) before probe

- **Where:** `main.rs` `run()` → `AppContext::from_cli` then `run_restore`
- **Issue:** Spec baseline noted migrate-while-daemon-up risk. T188 F16b fixed **export** only. Mutating restore hard-fail runs after AppContext open; if open/migrate fails under exclusive lock, operator may see vault-locked error instead of F5 “daemon is running” class. Overwrite still blocked.
- **Disposition:** Documented residual / future hardening; out of F16b freeze for restore.

#### P3-6 — Orchestrator closeout incomplete (AC9 partial / AC10)

- Plan D2 full gate, D3 manual evidence, D5 strike deferred #1/#6, D6 conductor ✅, D7 pin decision remain open.
- `conductor/deferred.md` §59 lines 642–647 still list recovery export + restore hard-fail as residuals (should be struck after merge).
- **Disposition:** Orchestrator closeout; not product rework.

---

## Completeness Sweep

| Check | Result |
|-------|--------|
| `TODO` / `FIXME` / `unimplemented!` / `todo!` in T188 surfaces | **None** in `recovery.rs` / restore path of `backup.rs` (only “placeholder” word in dry-run TTY comment) |
| Fake success / stub export | **None** — generate, write, unlock roundtrip proven |
| `unwrap`/`expect`/`panic!` in production paths | **None** in production portions of `recovery.rs` / restore helpers (only `#[cfg(test)]` modules and `unwrap_or*`/`unwrap_or_else`) |
| Forbidden `--passphrase` argv | **Absent** from CLI; only `--passphrase-file` |
| Kit JSON on stdout | **Refused** — path + `dpapi:` only; integration asserts |
| Doctor CLI | **Still absent** (honest) |

---

## Wiring and Regression Review

### Restore

1. `probe_restore_daemon_busy` → true/false  
2. Dry-run: optional notice → integrity plan → exit 0 (no overwrite)  
3. Mutating + busy: **error before** confirm / SQLite backup API  
4. Mutating + free: confirm unless `--force` → Online Backup API restore → drop `_aibrains_backup_meta`

**Force never overrides probe:** enforced by control-flow order (`daemon_up` check before `force` confirm skip).

### Export

1. **Before** `AppContext::from_cli` (`main.rs:1817-1841`) — no migrate for kit path  
2. Key → `data_key_from_sqlcipher` → passphrase (file/TTY) → exists/public checks → `RecoveryKit::generate` → write 0600 → stdout status → best-effort event  

**Daemon up:** kit file still written (`recovery_export__daemon_up__no_migrate_kit_ok`); event soft-fail acceptable.

### Security properties (spot-check)

| Property | Result |
|----------|--------|
| Passphrase zeroize | `Zeroizing<Vec<u8>>` / `Zeroizing<String>`; dropped after generate |
| Public path refuse | Windows `Users\Public` (+ related patterns); unit tested |
| Secrets in tracing | No passphrase/key/kit in macros |
| rpassword | Sole new prod dep; Apache-2.0 |

---

## Verification Evidence

### Code anchors (representative)

```351:365:crates/ai-brains-cli/src/commands/backup.rs
pub async fn probe_restore_daemon_busy(client: &DaemonClient) -> bool {
    const ATTEMPTS: u32 = 3;
    const PER_ATTEMPT: std::time::Duration = std::time::Duration::from_millis(1000);
    // ...
}
```

```436:438:crates/ai-brains-cli/src/commands/backup.rs
    if daemon_up {
        return Err(restore_daemon_busy_message().into());
    }
```

```1817:1841:crates/ai-brains-cli/src/main.rs
    // T188 F16b: recovery export must not call AppContext::from_cli (always migrate()).
    if let Commands::Recovery { command } = &cli.command {
        return match command { /* Export → run_export */ };
    }
```

```12:21:crates/ai-brains-crypto/src/recovery_kit.rs
pub struct RecoveryKit {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    // ...
}
```

### Tests mapping

| Test | Proves |
|------|--------|
| `backup_restore__daemon_running__fails_no_overwrite` | AC1, F1, F2, F5 |
| `backup_restore__daemon_running_dry_run__ok_with_notice` | AC3 (partial), F3 |
| `backup_restore__daemon_down_force__succeeds` (unit + integration) | AC2 |
| `recovery_export__passphrase_file__writes_unlockable_kit` | AC4, F11 |
| `recovery_export__stdout__no_kit_json_or_secrets` | AC5 |
| `recovery_export__output_exists__refuses_without_force` | AC14 |
| `recovery_export__dry_run__no_file` | F14 |
| `recovery_export__short_passphrase__fails` | F8 min length |
| `recovery_export__daemon_up__no_migrate_kit_ok` | AC13 kit path |
| `refuse_public_output_path__users_public__refuses` | AC6 Windows |
| `data_key_from_sqlcipher__malformed__error_has_no_key_material` | AC12 |
| `recovery_kit__generate__schema_version_is_1` / legacy default | F19 |

### Commands recommended (orchestrator / CI; not re-run as authoritative in this read-only pass)

```powershell
cargo nextest run -p ai-brains-cli --test recovery_drills
cargo nextest run -p ai-brains-cli --lib
cargo nextest run -p ai-brains-crypto --test crypto_recovery
cargo clippy -p ai-brains-cli -p ai-brains-crypto --all-targets -- -D warnings
# Full gate at closeout: fmt, clippy workspace, nextest workspace, deny, audit
```

---

## Deferred Candidates

Eligible for `deferred.md` / track residual only if not fixed in closeout polish:

| ID | Item | Why deferrable |
|----|------|----------------|
| P3-1 | Capture dry-run stdout for notice substring | Non-blocking; production prints correct constant |
| P3-2 | Assert `RecoveryKitCreated` when vault free; no event on dry-run | Best-effort event; kit file is DoD |
| P3-3 | Refresh RECOVERY-DRILLS DECISION pin | Docs polish; primary R-DOC-CLI language already correct |
| P3-4 | Optional live-daemon restore hard-fail integration | Unit injection covers logic; IPC probe is production path |
| P3-5 | Restore probe-before-AppContext ordering | Spec did not require; residual honesty |

**Must not defer (orchestrator process, not product bugs):** strike deferred §59 #1/#6 after ship; leave #2 doctor; full gate AC9; conductor status ✅.

---

## Completion Decision

**Product implementation of T188 meets core DoD (F1–F22, AC1–AC8, AC11–AC14) with only limited P3 residuals.**

| Gate | Decision |
|------|----------|
| Security-critical restore hard-fail | **Pass** — no overwrite when probe true; force cannot bypass |
| Recovery export operator surface | **Pass** — file-only kit, secrets discipline, F16b bypass |
| schema_version / crypto hygiene | **Pass** |
| Docs R-DOC-CLI (export yes / doctor no) | **Pass** with P3 pin cleanup |
| Open critical/high | **None** |
| Track full closeout | **Not yet** — AC9 full gate, AC10 deferred strike, plan D2–D7 |

**Verdict: PASS WITH DEFERRED P3**

Recommended orchestrator next steps:
1. Optionally fix P3-1/P3-3 in a small polish commit (preferred before merge).  
2. Run full CI gate; record evidence.  
3. Strike deferred.md §59 #1 and #6; leave #2 doctor.  
4. Set conductor T188 ✅ after external/SECURITY review sign-off as required by plan D4–D6.  
5. Leave TX `c32896d2-ee9f-4b9a-99bf-9a45e0195351` handling to ledger workflow.

---

*End of internal R1 audit.*
