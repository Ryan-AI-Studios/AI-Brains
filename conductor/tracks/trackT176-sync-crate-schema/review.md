# T176 Review Log — Sync Crate + Schema

## Rounds

| Round | Source | Verdict | Date |
|-------|--------|---------|------|
| Internal R1 | explore subagent | **NEEDS_FIX** | 2026-07-30 |
| Internal R2 | re-review after fixes | **NEEDS_FIX** (ID-12 only) | 2026-07-30 |
| Internal R3 | ID-12 + ID-14 fix | **CLEAN_WITH_DEFERRED_LOWS** | 2026-07-30 |
| Codex cross-model | TBD | | |

## Implementation

- Branch: `track/T176-sync-crate-schema`
- Commits: `f68f274` (impl), `f0b897b` (review fixes), + ID-12/14 follow-up
- Ledger tx: `5dae83e3-193e-467b-a817-c7f92e172af4`

---

## Findings disposition

| ID | Severity | Status | Notes |
|----|----------|--------|-------|
| ID-1 Signed DeviceEnrolled | high | **verified_fixed** | R2 re-review |
| ID-2 Raw seed sidecar | high | **verified_fixed** | R2 re-review |
| ID-3 Bootstrap non-atomic | medium | **verified_fixed** | Single TX; R2 |
| ID-4 Data wrap_count=0 | medium | **verified_fixed** | R2 |
| ID-5 Fake bootstrap test | medium | **verified_fixed** | R2 |
| ID-6 CLI enroll/revoke tests | medium | **verified_fixed** | R2 |
| ID-7 Signed DeviceRevoked | medium | **verified_fixed** | R2 |
| ID-8 HKDF OKM hex pin | low_info | **verified_fixed** | Optional pin landed |
| ID-9 Upgrade/CHECK tests | low_info | **deferred** | Non-blocking; fresh-vault covered |
| ID-10 Strict wrap sort | low_info | **verified_fixed** | R2 |
| ID-11 Process DoD closeout | low_info | open | Orchestrator |
| ID-12 R27 check outside TX | medium | **fixed_pending_verification** | IMMEDIATE TX + recheck inside |
| ID-13 Package schema_version gate | low_info | **deferred** | Allowlist residual |
| ID-14 Revoke missing verify | low_info | **fixed_pending_verification** | verify_envelope before persist |

---

## ID-12 detail
- **severity:** medium
- **status:** fixed_pending_verification
- **fix:** `bootstrap_local_device` uses `TransactionBehavior::Immediate` and re-runs `has_active_or_local_device` inside the transaction before inserts.
- **evidence:** `replication.rs` `bootstrap_local_device`; test `bootstrap_local_device__second_call__err`

## ID-14 detail
- **severity:** low_info
- **status:** fixed_pending_verification
- **fix:** `run_revoke` verifies signature against signer public before persist (match bootstrap/enroll).

## Deferred (low only)

| ID | Justification |
|----|---------------|
| ID-9 | Fresh vault migration tested; 0026→0027 upgrade is mechanical register; CHECK exercised indirectly by length rejects. Cap deferred lows. |
| ID-13 | v1 packages only from our `package-export` with schema 1; fail-closed allowlist can land with T177 package versioning. |

---

## Disposition policy

- All high/medium fixed before cross-model.
- Deferred lows only: ID-9, ID-13 (and process ID-11 until closeout).
