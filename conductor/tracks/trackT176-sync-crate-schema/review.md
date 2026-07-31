# T176 Review Log — Sync Crate + Schema

## Rounds

| Round | Source | Verdict | Date |
|-------|--------|---------|------|
| Internal R1 | explore subagent | **NEEDS_FIX** | 2026-07-30 |
| Internal R2 | re-review after fixes | **NEEDS_FIX** (ID-12 only) | 2026-07-30 |
| Internal R3 | ID-12 + ID-14 fix | **CLEAN_WITH_DEFERRED_LOWS** | 2026-07-30 |
| Codex R1 | cross-model review.codex.md | **FAIL** | 2026-07-31 |
| Codex R1 remediations | implementer | **fixed_pending_verification** | 2026-07-30 |
| Codex R2 | membership dual-write atomicity | **FAIL** (HIGH) | 2026-07-30 |
| Codex R2 remediation | implementer | **fixed_pending_verification** | 2026-07-30 |

## Implementation

- Branch: `track/T176-sync-crate-schema`
- Commits: `f68f274` (impl), `f0b897b` (review fixes), + ID-12/14 follow-up, + SOV dual-write, + atomic membership projector
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
| ID-9 Upgrade/CHECK tests | low_info | **verified_fixed** | 0026→0027 + CHECK tests landed (Codex P2-B) |
| ID-10 Strict wrap sort | low_info | **verified_fixed** | R2 |
| ID-11 Process DoD closeout | low_info | open | Orchestrator (conductor Complete + G3–G7) |
| ID-12 R27 check outside TX | medium | **fixed_pending_verification** | IMMEDIATE TX + recheck inside |
| ID-13 Package schema_version gate | low_info | **deferred** | Allowlist residual |
| ID-14 Revoke missing verify | low_info | **fixed_pending_verification** | verify_envelope before persist |
| Codex-P1-A Event log SOV | high | **fixed_pending_verification** | DeviceEnrolled/DeviceRevoked payloads + append on bootstrap/enroll/revoke |
| Codex-P1-B content_key_id | high | **fixed_pending_verification** | `build_and_sign_control` takes ContentKeyId; erasure/ACK bind target key |
| Codex-P2-A Self-revoke | medium | **fixed_pending_verification** | `run_revoke` rejects device_id == signer_id (ADR-0018 L4) |
| Codex-P2-B Regression | medium | **fixed_pending_verification** | Upgrade path, CHECK, signed DeviceRevoked/erasure/ACK tests |
| Codex-P2-C Governance | low_info | open | This log updated; leave conductor Complete to orchestrator after re-review |
| Codex-R2-A Membership dual-write not atomic | high | **fixed_pending_verification** | See detail below |

---

## Codex R1 FAIL detail

### Codex-R2-A — Membership dual-write not atomic
- **severity:** high
- **status:** fixed_pending_verification
- **problem:** CLI appended DeviceEnrolled/DeviceRevoked then separately wrote side stores. Side-store failure after append left SOV saying membership changed without identity/control rows; no projector.
- **fix (CQRS-correct):**
  1. Extended `DeviceEnrolledPayload` / `DeviceRevokedPayload` with wire fields (`envelope_id`, `signature_hex`, `body_hex`, `content_type_code`) so a projector can rebuild public membership + signed control.
  2. `ReplicationProjection` in `projections/replication.rs` applies enroll → identity+signed_control+envelope_index; revoke → tombstone+control+index. Idempotent on replication event_id.
  3. Registered in `apply_all`.
  4. Bootstrap private key cannot go in the event log: `SqliteEventStore::append_device_enrolled_with_private_key` single IMMEDIATE TX (R27 check → insert_event_row/apply_all → put_device_private_key_wrap).
  5. CLI bootstrap uses atomic append+wrap; enroll/revoke only `append_event` (projector applies). No separate dual-write.
- **evidence:** store tests `append_event__device_enrolled__projects_identity_and_signed_control`, `append_event__device_revoked__projects_tombstone_and_control`, `append_device_enrolled_with_private_key__bad_wrap__rolls_back_event`; CLI still green for bootstrap/enroll/revoke.

### Codex-P1-A — Append to canonical event log
- **severity:** high
- **status:** fixed_pending_verification
- **fix:** `ai-brains-events` adds `DeviceEnrolled` / `DeviceRevoked` payloads + `EventKind`; CLI appends via EventStore; public side stores now projected (Codex-R2-A).
- **evidence:** `payload.rs`, `event_kind.rs`, `device.rs` `append_device_*_event`; CLI tests `bootstrap__appends_device_enrolled_event_log_sov`, `revoke__peer_after_enroll__ok_and_event_log`

### Codex-P1-B — content_key_id for erasure/ACK
- **severity:** high
- **status:** fixed_pending_verification
- **fix:** `build_and_sign_control(..., content_key_id)`; nil for enroll/revoke/gap-skip; target key for ContentErasureTombstone / ErasureAck.
- **evidence:** tests `build_and_sign_control__erasure_tombstone__content_key_bound`, `build_and_sign_control__device_revoked__verifiable`, erasure_ack + gap_skip

### Codex-P2-A — Forbid self-revoke
- **severity:** medium
- **status:** fixed_pending_verification
- **fix:** `run_revoke` returns structured error when `device_id == signer_id`.
- **evidence:** CLI test `revoke__self__fails_adr0018_l4`

### Codex-P2-B — Regression coverage
- **severity:** medium
- **status:** fixed_pending_verification
- **fix:** `migration_0027__after_0026__applies_forward`, `device_identity__bad_status__check_constraint_rejects`, signed control encode/sign/verify suite in `control.rs`.

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
| ID-13 | v1 packages only from our `package-export` with schema 1; fail-closed allowlist can land with T177 package versioning. |
| ID-11 / Codex-P2-C process | Conductor Complete + G3–G7 left for orchestrator after final re-review. |

---

## Disposition policy

- All high/medium fixed before cross-model clearance.
- Membership public side stores are event-projected (Codex-R2-A); private key wrap remains command-path secret.
- Deferred lows only: ID-13 (and process ID-11 until closeout).
