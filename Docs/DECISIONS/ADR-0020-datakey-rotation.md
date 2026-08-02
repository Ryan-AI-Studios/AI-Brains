# ADR-0020: DataKey Rotation (Vault KEK Ceremony)

## Status

**Accepted** — 2026-08-02.

Normative for **T189** (`vault rotate-datakey`). Closes the implementation direction left open by [ADR-0016 §4](ADR-0016-content-envelope-cryptography.md) and [ADR-0018 §19](ADR-0018-encrypted-event-replication-protocol.md) (deferred **#34.2**).

Freezes cited: **F1–F34**, acceptance **AC1–AC19** — see
[`conductor/tracks/trackT189-datakey-rotation/spec.md`](../../conductor/tracks/trackT189-datakey-rotation/spec.md).

## Context

ADR-0016 established envelope encryption under a vault-lifetime **DataKey** (KEK)
with AES-256-GCM + random 96-bit nonces for content-DEK wraps. NIST SP 800-38D
implies a practical ~2³² message budget **per key**. No product rotation path
existed; wrap-nonce budget was an accepted residual.

ADR-0018 multi-device peer wraps (X25519 + HKDF + AES-GCM) improve the
*per-recipient* nonce story but **do not** rotate the local vault DataKey or
SQLCipher page key (`SqlCipherKey::from_data_key` binding from T187).

T187 ships live SQLCipher page encryption; T188 ships RecoveryKit export
(`schema_version=1`). Operators need a **ceremony** to rotate DataKey + page key
together without claiming silent auto-rotation, dual active KEKs, or NIST Purge
of offline backups.

## Decision

### 1. Meaning of rotation (F1–F4, F9–F11)

1. Generate a new random `DataKey` (production: `DataKey::generate()`).
2. Re-wrap all **active** content DEKs (`content_key_store.status = 'active'` only).
3. Re-seal the **local** device private-key blob if present (**at most one** row
   in `device_private_key_store`).
4. Change the SQLCipher page key via `SqlCipherKey::from_data_key(new)` — **do not**
   split page key from DataKey in v1 (F2).
5. Emit audit event `DataKeyRotated` (no secrets).
6. **Mandatory** RecoveryKit re-export for the new key.

**Out of scope for v1:** re-seal of `encrypted_content_blob`; mutation of
`peer_content_key_wrap`; cross-device content re-wrap over relay; dual active KEKs;
silent/scheduled rotation; shared DataKey across devices.

**Multi-device honesty (F3/F4):** each enrolled device runs its own local ceremony.
Per-recipient wire wraps need no change when only the local vault DataKey rotates.

### 2. Primary path = crash-safe `sqlcipher_export` (F5, F7, AC16)

**Default (primary):** encrypted→encrypted export under the **new** key:

1. Open source vault with the **old** key.
2. `ATTACH` destination under the **new** KEY.
3. `SELECT sqlcipher_export(…)` (schema/data page copy under new codec).
4. Apply active DEK re-wraps + device private re-seal **on the new DB only** so the
   **old file remains openable with the old key until atomic replace**.
5. Verify (sqlite_master + unwrap sample active DEKs under new DataKey).
6. Hold source exclusive (`PRAGMA locking_mode=EXCLUSIVE`) through export + dest
   rewrap + verify; **drop source handle only immediately before replace**
   (Windows cannot replace a path while a process holds the DB open).
7. **Atomic replace** via Windows `MoveFileEx` REPLACE_EXISTING only (no two-step
   rename-aside fallback — that would leave the canonical path missing on crash).
8. On any failure **before** replace: abandon/delete the new file; old vault
   untouched (fail closed).

**Residual (P3 honesty):** between exclusive `drop(source)` and `MoveFileEx`
there is a tiny OS-required window where another process could open the old
vault. Daemon hard-fail + exclusive hold through rewrap minimize this; full
zero-window replace under SQLite exclusive is not available on Windows.

### 3. Opt-in in-place rekey (F5b, F7b)

`PRAGMA rekey` is **not** the default. It is available only with
**`--accept-rekey-risk`**:

1. Auto-create `*.pre-rotate.bak` snapshot first.
2. Commit wrap/device updates under the old page key.
3. `PRAGMA wal_checkpoint(TRUNCATE)` → `journal_mode=DELETE` →
   `PRAGMA rekey = "x'…'"` via **`execute_batch` only** (rusqlite has no native
   rekey) → restore `journal_mode=WAL` + product sync pragmas.
4. Verify; on wrap/rekey/verify failure → **auto-restore** snapshot and fail closed.
5. Residual: mid-rekey crash can still corrupt the in-place file; recovery =
   snapshot/backup (document honestly).

### 4. Ceremony, gates, and CLI (F6, F8, F12–F14, F25, F31–F34)

Operator sequence (summary):

```text
backup create → daemon stop → dry-run → rotate-datakey --confirm --kit-output …
→ verify new kit unlock → update AI_BRAINS_KEY → daemon start → retire old kits
```

| Gate | Rule |
|------|------|
| Daemon | Mutating rotate: robust IPC probe hard-fail if daemon up (T188). No force-with-daemon. |
| Backup | `--require-backup` default ON: recent non-empty backup opens with **current** key and mtime ≤24h, **or** `--i-have-backup "I have a backup"` (audited `backup_bypassed`). |
| Pre-mutation (F31) | Validate passphrase source + kit path (writable, overwrite, reparse refuse) **before** vault mutation. |
| Confirm | Non-dry-run requires `--confirm` (or interactive yes). |
| Kit | Success requires kit write for **new** key; optional `--print-key` off by default. |
| Stale key | Mandatory WARNING that old `AI_BRAINS_KEY` cannot open the rotated vault. |
| Kit verify before retire (F32) | Do not destroy old kit copies until new kit unlock is verified. |

CLI surface: `ai-brains vault rotate-datakey` (flags per F13). Capture-independent
(F22): no models/graph on the rotate path.

### 5. Crypto / store APIs (F18–F20)

| API | Role |
|-----|------|
| `SqlCipherKey::to_data_key` | Typed parse of `x'<64 hex>'` → `DataKey`; **never** put key hex in error messages |
| `rotate_content_dek_wrap` | Unwrap under old DataKey; wrap under new (fresh CSPRNG 96-bit nonce) |
| `list_active_content_key_wraps` | `WHERE status = 'active'` only |
| `update_content_key_wrap` | Active-only UPDATE; never peer wraps |

Tests use `DataKey::from_bytes` for determinism (F33). Hold **aes-gcm 0.10**; zero
new production deps (F21).

### 6. Audit event (F15, F30)

```text
EventKind::DataKeyRotated
Payload: {
  rotation_id: Uuid,
  living_wraps_rewrapped: u64,
  device_private_resealed: u32,   // 0 or 1
  backup_bypassed: bool,
  completed_at: String            // RFC3339; KAT-pinnable
}
```

- **No secrets** in payload or logs.
- **`AggregateType::System` + `aggregate_id = Uuid::nil()`**; `rotation_id` lives
  only in the payload.
- Append best-effort after success: if event fails → **warn**, rotation still
  success if vault opens with the new key.

Prefer **event-only** audit (F16); no new table default. `WRAP_SCHEMA_VERSION`
stays 1.

### 7. Claims honesty (F23)

On ship: RELEASE-CLAIMS **R-34.2** → **implemented-with-residuals**. Residuals
include: multi-device requires per-device ceremony; offline backups/old kits open
only under the old key; rekey path crash residual; no NIST Purge of offline media.
~~Argon2 params not in kit JSON (F24 / F37)~~ **closed by T194** (`passphrase.kdf`).

## Consequences

### Positive

- Operators can reset the DataKey wrap-nonce budget under ceremony controls.
- Crash-safe default path; old vault recoverable until atomic replace succeeds.
- Page key and DataKey stay bound (`from_data_key`).
- Contracts updated for `DataKeyRotated` without secret leakage.

### Negative / residual

- Export path uses ~2× disk temporarily and is slower than in-place rekey.
- Each multi-device peer must rotate **its own** vault DataKey independently.
- Old RecoveryKits and pre-rotation backups remain decryptable under the **old**
  key until operators retire them (not product Purge).
- In-place rekey remains opt-in with snapshot restore, not crash-atomic.

### Compatibility

- Existing vaults: rotate is opt-in CLI; no automatic migration.
- Peer wraps and content blob ciphertext bytes unchanged by rotation.
- RecoveryKit `schema_version=1` unchanged; new kits bind the **new** DataKey only.

## Alternatives considered

| Option | Decision | Why |
|--------|----------|-----|
| Default `PRAGMA rekey` | **Reject as default** | No crash-atomicity; mid-rekey power loss can corrupt in place |
| Dual active KEK / gradual re-wrap under daemon | **Reject for v1** | Complexity; exclusive lock + dual versioning out of scope |
| Split page key from DataKey | **Reject for v1** | New ADR territory; product binding is SOOT |
| Silent automatic rotation | **Forbid** | Ceremony + backup gate required |
| Rotate peer wraps with vault DataKey | **Reject** | Different key layer (ADR-0018 §17) |
| AWS-style multi-version CMK | **Reject for v1** | Single active DataKey after success (F9) |

## Related

- Spec: `conductor/tracks/trackT189-datakey-rotation/spec.md` (F1–F34, AC1–AC19)
- Spike: `conductor/tracks/trackT189-datakey-rotation/spike-a2.md`
- ADR-0016 §4 (pointer), ADR-0018 §19 (pointer)
- T187 page encrypt / T188 recovery export patterns reused
- OPERATIONS ceremony; RECOVERY-DRILLS; RELEASE-CLAIMS R-34.2
