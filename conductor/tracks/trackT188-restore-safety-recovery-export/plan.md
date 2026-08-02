# T188 Plan — Restore Safety + Recovery Operator Surface

Status: **🚧 In Progress / implemented pending review** (2026-08-01).  
Spec: [spec.md](./spec.md) (F1–F28, AC1–AC14).

## Preconditions

- [x] Expand freezes + AI fold-in disposition  
- [x] Pick F17: **rpassword 7.5.x** (Apache-2.0; sole new prod dep; workspace pin)  
- [x] If rpassword: confirm Apache-2.0 in deny allowlist  
- [x] Soft: T187 not required  
- [x] `ledgerful doctor` + `scan --impact` at implement  
- [x] `ledgerful ledger start T188-RestoreSafetyRecoveryExport --category SECURITY` (TX `c32896d2-ee9f-4b9a-99bf-9a45e0195351` — leave pending for orchestrator)

## License / secrets gate

- [x] At most one new prod dep (rpassword)  
- [x] Passphrase/`Zeroizing` discipline  
- [x] deny + audit green  

---

## Phase A — Restore hard-fail (TDD)

- [x] **A1** Implement `probe_restore_daemon_busy()`: timeout ≥1000ms, ≥2 retries (F1b)  
- [x] **A2** RED/GREEN: mutating restore + probe true → non-zero; vault unchanged  
- [x] **A3** Message: `daemon is running` + **both** `daemon stop` and service-stop guidance (F5)  
- [x] **A4** `--force` does not override probe  
- [x] **A5** Dry-run + daemon up → success **and** stdout contains live-restore-will-fail notice (F3; assert)  
- [x] **A6** Daemon down restore success regression  
- [x] **A7** Update T181-F-03 soft language → hard-fail product  

**Tests:**  
`backup_restore__daemon_running__fails_no_overwrite`  
`backup_restore__daemon_running_dry_run__ok_with_notice`  
`backup_restore__daemon_down_force__succeeds`

---

## Phase B — `recovery export` (TDD)

### B0 Dep + crypto hygiene

- [x] **B0.1** Add rpassword **or** hand-roll zero-echo helper → **rpassword 7.5.x**  
- [x] **B0.2** `RecoveryKit.schema_version` default 1 (F19)  
- [x] **B0.3** Test `data_key_from_sqlcipher__malformed__error_has_no_key_material` (F7/AC12)  

### B1 CLI surface

- [x] **B1.1** `recovery export --output <path> [--passphrase-file] [--dry-run] [--force|--overwrite]`  
- [x] **B1.2** Output exists → refuse unless force (AC14)  
- [x] **B1.3** Forbid `--passphrase` argv  

### B2 Passphrase paths

- [x] **B2.1** File: Zeroizing read, 8 KiB cap, regular-file only, min 8 bytes (F8/F8b)  
- [x] **B2.2** TTY: zero-echo double-entry; min 8; Zeroizing after generate  
- [x] **B2.3** Dry-run: file path validated (read+zeroize); TTY path = TTY present, **no** prompt (F14)  

### B3 Generate + write

- [x] **B3.1** DataKey from sqlcipher key; `RecoveryKit::generate`  
- [x] **B3.2** Write JSON; Unix **0600**; Windows public-path refuse + best-effort ACL (F9b)  
- [x] **B3.3** Stdout: path + `dpapi: present|absent` only (F11)  
- [x] **B3.4** **No migrate()** while daemon up (F16b); kit export does not require writer  

### B4 Event best-effort

- [x] **B4.1** On success: try append `RecoveryKitCreated` via System aggregate + kit UUID  
- [x] **B4.2** If append fails (busy daemon): warn; **export still exit 0** if file written  
- [x] **B4.3** Dry-run: no event  
- [x] **B4.4** Unit assert: daemon-down export writes kit **and** ≥1 `RecoveryKitCreated` in vault (R1 P3-2)  

### B5 Leakage tests

- [x] **B5.1** No DataKey / ciphertext / passphrase in stdout+stderr  
- [x] **B5.2** Capture tracing if harness allows; else code-review pin no `tracing!(…passphrase…)`  
- [x] **B5.3** `recovery_export__stdout__no_ciphertext_field` (via `recovery_export__stdout__no_kit_json_or_secrets`)  
- [x] **B5.4** Unlock roundtrip from written file  

**Tests:**  
`recovery_export__passphrase_file__writes_unlockable_kit`  
`recovery_export__daemon_down__appends_recovery_kit_created_event`  
`recovery_export__stdout__no_kit_json_or_secrets`  
`recovery_export__output_exists__refuses_without_force`  
`recovery_export__dry_run__no_file`  
`recovery_export__daemon_up__no_migrate_kit_ok`

---

## Phase C — Docs / claims

- [x] **C1** RECOVERY-DRILLS §4 CLI checklist; post-drill improvement one-liner (L1)  
- [x] **C2** OPERATIONS: restore hard-fail + robust probe; export path  
- [x] **C3** CAPABILITIES / INSTALL / SECURITY.md  
- [x] **C4** R-DOC-CLI: export present; doctor absent  
- [x] **C5** Argon2 default tuple + “backups inherit SQLCipher page encryption when T187 live” note  
- [x] **C6** CHANGELOG  
- [x] **C7** R1 P3-3: refresh DECISION pin (export shipped; doctor residual); P3-4 F-03 automation honesty 

---

## Phase D — Closeout

- [x] **D1** Targeted nextest + clippy  
- [x] **D2** Full gate  
- [x] **D3** Manual evidence  
- [x] **D4** SECURITY review (passphrase/export/restore)  
- [x] **D5** Strike deferred #1/#6; leave #2 doctor  
- [x] **D6** conductor ✅  
- [x] **D7** Pin decision  

---

## F17 choice (frozen)

**`rpassword` 7.5.x** (Apache-2.0) — sole new production dependency. Workspace pin in root `Cargo.toml`; depended from `ai-brains-cli` only. Provides zero-echo TTY passphrase for interactive `recovery export`. Alternative hand-roll declined to keep Windows/Unix surface small.

---

## AI fold-in plan deltas

| # | Delta |
|---|--------|
| 1 | A1 robust probe ≥1s + 2 retries |
| 2 | A5 dry-run notice **asserted** |
| 3 | B3 no migrate while daemon up |
| 4 | F12 event best-effort System aggregate (not soft-orphan) |
| 5 | F17 rpassword allowed **or** hand-roll |
| 6 | F8b file safety + min passphrase 8 |
| 7 | F19 schema_version=1 |
| 8 | Output exists refuse; 0600; public path refuse |
| 9 | dpapi present/absent on stdout |
| 10 | Leakage includes ciphertext + tracing discipline |

---

## Out of scope checklist

- [ ] Full doctor  
- [ ] sc query disambiguation  
- [ ] RecoveryKitCreated payload expansion (vault_id)  
- [ ] Argon2 params in kit JSON  
- [ ] T187 / T189 product work  
