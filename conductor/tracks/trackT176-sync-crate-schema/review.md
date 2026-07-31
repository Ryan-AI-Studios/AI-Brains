# T176 Review Log — Sync Crate + Schema

## Rounds

| Round | Source | Verdict | Date |
|-------|--------|---------|------|
| Internal R1 | explore subagent | **NEEDS_FIX** | 2026-07-30 |
| Internal R2 | fix subagent | **fixed_pending_verification** | 2026-07-30 |
| Codex cross-model | TBD | | |

## Implementation

- Branch: `track/T176-sync-crate-schema`
- Commit (initial): `f68f274`
- Ledger tx: `5dae83e3-193e-467b-a817-c7f92e172af4`

---

## Internal R1 findings

### ID-1 — Signed DeviceEnrolled not produced (bootstrap/enroll)
- **severity:** high
- **status:** fixed_pending_verification
- **source:** internal-review
- **files:** `crates/ai-brains-cli/src/commands/device.rs`, sync control/envelope, store `signed_replication_control`
- **required_fix:** Self-sign (bootstrap) / enrolled-signer sign (enroll) `DeviceEnrolled` control; persist signed control; tests verify signature
- **evidence:** `build_and_sign_control` + `bootstrap_local_device` / `enroll_peer_device`; tests: `build_and_sign_control__device_enrolled__verifiable`, `bootstrap_local_device__second_call__err` (stores verifiable control), `cli_enroll__after_bootstrap_and_package__ok`, `signed_control_rows__maps_envelope`

### ID-2 — package-export raw seed sidecar
- **severity:** high
- **status:** fixed_pending_verification
- **source:** internal-review
- **files:** `device.rs` package-export
- **required_fix:** No raw seed files; DPAPI and/or passphrase wrap; zeroize buffers; document
- **evidence:** Default public package only; optional `--write-private-key` DPAPI on Windows (`Zeroizing`); no `.seeds`. Tests: `cli_package_export__public_only__no_raw_seeds`, `cli_package_export__write_private_key_dpapi__ok` (Windows)

### ID-3 — Bootstrap non-atomic
- **severity:** medium
- **status:** fixed_pending_verification
- **source:** internal-review
- **required_fix:** Single SQLite transaction for identity + private key wrap
- **evidence:** `bootstrap_local_device` one TX (identity + private_key + signed_control + envelope_index); used by CLI bootstrap

### ID-4 — Data envelopes allow wrap_count=0
- **severity:** medium
- **status:** fixed_pending_verification
- **source:** internal-review
- **required_fix:** Fail closed on DataEvent with empty wraps
- **evidence:** `check_wrap_count_rules` in `sign_envelope` / `verify_envelope`; test `verify_envelope__data_empty_wraps__err`

### ID-5 — Fake store bootstrap second-call test
- **severity:** medium
- **status:** fixed_pending_verification
- **source:** internal-review
- **required_fix:** Real API second-call rejection or remove fake test
- **evidence:** Real `bootstrap_local_device` → `BootstrapAlreadyEnrolled`; test `bootstrap_local_device__second_call__err`

### ID-6 — Missing CLI tests for enroll/revoke/package-export
- **severity:** medium
- **status:** fixed_pending_verification
- **source:** internal-review
- **required_fix:** assert_cmd coverage
- **evidence:** `cli_package_export__public_only__no_raw_seeds`, `cli_enroll__after_bootstrap_and_package__ok`, `cli_revoke__after_enroll__ok`, `bootstrap__second_call__err` (existing)

### ID-7 — Revoke without signed DeviceRevoked control
- **severity:** medium
- **status:** fixed_pending_verification
- **source:** internal-review
- **required_fix:** Sign + persist DeviceRevoked control; keep R23 wrap delete
- **evidence:** `run_revoke` → `build_and_sign_control(DeviceRevoked)` + `revoke_device_with_control` (control + index + tombstone + R23 one TX); test `cli_revoke__after_enroll__ok`

### ID-8 — HKDF OKM no fixed hex (T178 residual)
- **severity:** low_info
- **status:** fixed_pending_verification
- **deferrable:** yes (T178 owns full WRAP KATs); optional pin in T176
- **evidence:** Pinned OKM hex in `wrap_dek__hkdf_okm__kat` (`ac8bacd1a06000523db2170a84db49ccecb1ed2fbc5f6642c975840d21aadde3`)

### ID-9 — No 0026→0027 upgrade / CHECK tests
- **severity:** low_info
- **status:** open
- **deferrable:** yes if documented

### ID-10 — Duplicate wrap recipients accepted (≤ sort)
- **severity:** low_info
- **status:** fixed_pending_verification
- **required_fix:** Prefer fix now — strictly increasing recipients
- **evidence:** `wraps_are_sorted` uses strict `<`; test `signed_bytes__duplicate_recipient__err`

### ID-11 — Process DoD (manual, ledger, conductor, deferred)
- **severity:** low_info
- **status:** open
- **owner:** orchestrator closeout

---

## Disposition policy

- Fix all **high** and **medium** before cross-model.
- Fix easy lows (ID-4 related, ID-10) now.
- Defer only difficult non-blocking lows with `deferred.md` / ISSUES notes.
