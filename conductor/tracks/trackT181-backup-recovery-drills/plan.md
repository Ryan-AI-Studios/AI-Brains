# T181 Plan — Backup, Restore, Recovery-Kit Drills (P12.3)

Status: **Completed** (2026-08-01).  
Spec: [spec.md](./spec.md) (F1–F48, AC1–AC11).  
Policy doc (to ship): [Docs/RECOVERY-DRILLS.md](../../../Docs/RECOVERY-DRILLS.md) (Phase B).  
Related: [conductor/failure-drills.md](../../failure-drills.md) F-REC-*; ADR-0016 §12 / §4; OPERATIONS backup + erasure honesty.

## Preconditions

- [x] Read `BackupService` + `commands/backup.rs` restore path (incl. meta DROP)  
- [x] Read `RecoveryKit`, `passphrase.rs` (`Argon2::default`), `SqlCipherKey::from_data_key`  
- [x] Read store `destroy_content_key_wrap` + `wal_checkpoint_truncate`  
- [x] Re-scan smoke.rs backup tests (elevate + rename map)  
- [x] Confirm doctor / recovery CLI still absent (F5)  
- [x] `ledgerful doctor` + `scan --impact` before edits  
- [x] Prefer T186 hermetic helper if on main; else T179 `--no-project-context` + env pin  

## License gate

- [x] No external backup SaaS  
- [x] No AGPL agents  
- [x] Zero new production deps  
- [x] Prefer zero new dev-deps (`assert_cmd` / `tempfile` / `predicates` / `rstest` only)  
- [x] No secrets in logs / evidence files  

---

## Phase A — Inventory + elevate map

- [x] **A1** Freeze entry points (create/list/verify/prune/restore + BackupService + RecoveryKit + store wipe APIs)  
- [x] **A2** Map existing tests → `T181-*` ids  
- [x] **A3** Gap list: R-01 (+ meta assert), K-05/K-06/K-07, E-01/E-02, F-01/F-02/R-03  
- [x] **A4** Placement: `ai-brains-cli/tests/recovery_drills.rs` (+ crypto elevate; store E-drills)  
- [x] **A5** Empirically capture wrong-key error string once; pin for F-02  

## Phase B — Playbook docs (SP 800-184)

- [x] **B1** Write `Docs/RECOVERY-DRILLS.md`:  
  - When to run (release preflight, phase gates, ad-hoc)  
  - Operator-facing drill matrix  
  - Commands (backup create/verify/restore)  
  - **At vault initialization:** manual RecoveryKit export residual (no CLI today)  
  - CE honesty (pre-erase residual; not Purge/Destroy)  
  - Argon2 generation-time defaults residual (no KDF params in kit JSON)  
  - ADR-0016 §4 DataKey wrap-nonce residual cross-link  
  - macOS `source_vault_path` `/private/var` note  
  - 3-2-1 operator guidance (non-product)  
  - Example RTO/RPO language (non-SLA)  
  - Secrets handling  
- [x] **B2** Link from `Docs/OPERATIONS.md` backup section  
- [x] **B3** Cross-link `conductor/failure-drills.md` F-REC-01/02 → T181  
- [x] **B4** CAPABILITIES.md one-liner if needed (optional)  

## Phase C — Automated drills (TDD)

### C0 Helpers

- [x] **C0.1** Test helper `assert_no_secret_leakage(output, secret_bytes)` — hex + base64 + raw forms  
- [x] **C0.2** Helper also forbids kit JSON / obvious wrapped-ciphertext dumps in CLI output  

### C1 Restore roundtrip

- [x] **C1.1** RED `backup_restore__seeded_content__present_after_force_restore` (T181-R-01)  
  - Assert seeded content  
  - Assert `_aibrains_backup_meta` **present** in backup file  
  - Assert `_aibrains_backup_meta` **absent** on live vault post-restore  
- [x] **C1.2** Green if product already correct; fix only if bug  
- [x] **C1.3** Elevate dry-run / force tests:  
  - Rename `test_backup_restore_dry_run` → `backup_restore__dry_run__*`  
  - Rename `test_backup_restore_force_skips_prompt` → `backup_restore__force__*`  
  - **Either** strengthen force test with content smoke **or** supersede by R-01 (delete redundant prompt-only proof)  

### C2 Recovery kit

- [x] **C2.1** Elevate K-01..K-04 with T181 ids  
- [x] **C2.2** RED/GREEN **T181-K-05** (library only): unlock → `SqlCipherKey::from_data_key` → open vault/backup — **no** bare hex PRAGMA path  
- [x] **C2.3** RED/GREEN **T181-K-06**: correct unlock + wrong SqlCipherKey → open fails  
- [x] **C2.4** **T181-K-07**: kit JSON lacks KDF param field names  
- [x] **C2.5** Negative secrets: passphrase, raw key, kit JSON / wrapped ciphertext not in any CLI output under test  

### C3 Envelope-aware

- [x] **C3.1** RED/GREEN T181-E-01: seal → backup → `destroy_content_key_wrap` + `wal_checkpoint_truncate` → restore pre-wipe → open succeeds  
- [x] **C3.2** RED/GREEN T181-E-02: seal → wipe → backup → restore → open fails  
- [x] **C3.3** Comments cite ADR-0016 residual (not a “bug”)  
- [x] **C3.4** Measure wall time; if &gt;60s default tier → mark `__slow` with owner + reason  

### C4 Failure injection

- [x] **C4.1** Prefer single `rstest` parameterized suite for F-01 / F-02 / R-03  
- [x] **C4.2** T181-F-01 corrupt: cases offset 0 (header) and/or ≥100 (body); substring class F46  
- [x] **C4.3** T181-F-02 wrong key: pin empirical substring after A5  
- [x] **C4.4** T181-R-03 missing path: `not found` / `Backup file not found`  
- [x] **C4.5** Soft: document F-03 daemon warn (no behavior change)  

## Phase D — Closeout

- [x] **D1** Targeted:  
  `cargo nextest run -p ai-brains-cli --test recovery_drills --test smoke`  
  `cargo nextest run -p ai-brains-crypto --test crypto_recovery`  
  `cargo nextest run -p ai-brains-store --test content_envelope_crypto`  
- [x] **D2** Full gate: fmt, clippy -D warnings, nextest workspace, deny, audit  
- [x] **D3** Manual evidence table filled  
- [x] **D4** Internal review → fix → R2 clean  
- [x] **D5** Codex cross-model (SECURITY) clean  
- [x] **D6** deferred.md §59 Completed; residuals listed  
- [x] **D7** conductor.md T181 ✅ Completed  
- [x] **D8** Pin decision after merge/green  

---

## Manual evidence

| Check | Command / result |
|-------|------------------|
| Happy path restore | CLI T181-R-01 nextest green; force restore + recall content |
| Kit unlock (library) | crypto K-01..K-07 + store K-05/K-06 green |
| Docs honesty + kit residual | Docs/RECOVERY-DRILLS.md + OPERATIONS + failure-drills |
| No secrets in output | assert_no_secret_leakage on CLI create/restore/F-02 + crypto |
| Wrong-key substring pinned | F46 classes; dual-mode plain SQLite residual |

---

## Out of scope checklist

- [x] doctor CLI as DoD  
- [x] recovery export CLI as DoD  
- [x] Argon2 param schema bump  
- [x] Lightweight test Argon2 in prod API  
- [x] Offsite 3-2-1 automation  
- [x] #34.2 DataKey rotation  
- [x] F-REC-03/04 projection/graph rebuild  
- [x] Multi-device sync restore  
- [x] Hard-fail restore when daemon running  
- [x] NIST Purge/Destroy claims  
- [x] Runtime memory-scan zeroize asserts  

## Residual log (seed)

| Item | Severity | Owner |
|------|----------|-------|
| doctor missing recovery kit warning | residual | future / T183 |
| recovery export CLI | soft / security-relevant | future / T183 / T184 note |
| Argon2 KDF params not in kit JSON | residual (F37) | future schema hygiene |
| Intermediate hex in `SqlCipherKey::from_data_key` zeroize tighten | soft prod residual | future crypto |
| F-REC-03/04 rebuild drills | soft | store/graph |
| Restore hard-fail if daemon up | product residual | future |
| #34.2 DataKey rotation | open | P11 hygiene |
| Measured RTO metrics product | never (docs only) |

## Deferred absorbed (from `conductor/deferred.md`)

| Source | Item | How absorbed |
|--------|------|--------------|
| §34 / T162–T166 | Pre-erase backup residual | T181-E-01 + docs honesty (not “fixed”) |
| §55 T178 | pre-erase backups formal defer | Same; productize drills |
| §37 T165 | NIST non-claim + pre-erase warnings | Docs + E drills |
| failure-drills F-REC-01/02 | Manual recovery | Automate R-01 + K path |
| Implementation-Plan §10.2 | Recovery drills table | Spec matrix |
| §58 T186 | Hermetic helpers | Soft depend |
| T185 placeholder | Backup/recovery drills checkbox | Evidence handoff |
| AI1/AI2 2026-08-01 | See spec §15 | F33–F48 |

## Suggested commit sequence (implement)

1. Docs-only: RECOVERY-DRILLS + OPERATIONS + failure-drills links  
2. Helpers + rename elevate  
3. RED tests (R-01, K-05/06/07, E-01/02, F-matrix)  
4. Green / product fixes if any  
5. Review closeout  
