# T176 — Sync Crate + Replication Schema (P11.1)

- **Track ID:** T176-SyncCrateSchema
- **Phase:** P11 Task 11.1
- **Status:** 📋 **Proposed / Expanded** — ready to implement after human go-ahead
- **Depends on:** T175 Complete; ADR-0018 **Accepted** (2026-07-30, Codex R3 PASS); P8 CE types (T163–T165); store migrations discipline
- **Blocks:** T177 (fake relay needs types/schema); T178 (security suite needs harness primitives)
- **Category:** ARCHITECTURE / SECURITY
- **Normative design:** [ADR-0018](../../../Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md) + [threat-model §7](../trackT175-sync-threat-model-adr/threat-model.md)
- **Deferred absorbed:** #50 freezes (implement); #34.1 multi-device ACK **implementation** (schema + types). **Not** #34.2 DataKey rotation impl.

## 1. Objective

Ship the first **implementable** multi-device replication layer under ADR-0018:

1. New workspace crate **`crates/ai-brains-sync`** — envelope codec, device identity, sign/verify, per-recipient DEK wrap, fingerprint OOB helpers (library; no sockets).
2. Store migration **`0027_replication_state.sql`** — replication tables only; **no plaintext event bodies**.
3. Crypto + store glue — wrap/sign primitives using named crates; private device keys **dual-wrapped** on Windows: vault `DataKey` (AES-256-GCM) then OS **DPAPI** (`ai_brains_crypto::dpapi`).
4. CLI surfaces **`ai-brains device *`** and **`ai-brains replicate *`** — local vault ops / status; **no real network relay** (T177 owns fake-relay convergence).

After T176:

| Capability | Present |
|------------|---------|
| Enroll first device (local / RecoveryKit-bound) | Yes |
| List enrolled devices; show dual-key fingerprint | Yes |
| Seal/sign data + control envelopes (in-process) | Yes |
| Per-recipient X25519+HKDF-SHA256+AES-256-GCM DEK wrap | Yes |
| Persist wrap rows, cursors, ACK projection, gap state | Yes |
| Push/pull over network or fake relay | **No** — T177 |
| Full T178 claim matrix | **No** — stubs + unit KATs only; suite in T178 |

**Does not** implement fake relay, multi-device convergence tests, production network, MLS, HPKE crate, DataKey rotation, selective project sync, or multi-user vaults.

## 2. Live baseline (re-scan 2026-07-30)

| Area | Live state |
|------|------------|
| ADR-0018 | **Accepted** — L1–L16, §5 control cleartext / data `nonce‖ct‖tag`, §17 wrap bytes, CLI `device`/`replicate` |
| Migrations | Through **`0026_content_envelopes_erasure`**. Next free: **`0027`**. Master-plan sketch `0026_replication_*` is **invalid**. |
| Existing “sync” surfaces | (1) `Commands::Sync` Ledgerful bridge; (2) `SafetyCommands::Sync` hotspot pin; (3) table `sync_state` (0017 key/value helper) — **do not repurpose any of these** |
| Domain IDs | `DeviceId`, `ReplicationEventId`, `ContentKeyId` already in `ai-brains-core` (serde round-trips) |
| Core `Device` | Name + id only — **no** crypto fields yet |
| Workspace crypto | `aes-gcm` 0.10, `zeroize` 1.8, `subtle` 2.6, `rand` 0.10, `sha2` 0.11, `argon2` 0.5 — **no** ed25519/x25519/hkdf |
| `ai-brains-crypto` | `DataKey`, content DEK wrap/seal (ADR-0016), DPAPI/Windows path present — **no** device keys |
| Capture independence | Capture must not depend on `ai-brains-sync` |
| Workspace members | No `ai-brains-sync` yet |
| Edition / toolchain | edition **2024**; rustc **1.95** (`rust-toolchain.toml`); dalek MSRV **1.85** ✓ |

## 3. Research summary (online + standards, 2026-07-30)

### 3.1 Dependency inventory (crates.io API live)

Research date header: **2026-07-30**. Pin by **major.minor**; exact patch via `Cargo.lock` at implement. Drop publish-date parentheticals from normative tables.

| Crate | max_stable | License | MSRV | Role in T176 | Action |
|-------|------------|---------|------|--------------|--------|
| **ed25519-dalek** | **3.0.0** | BSD-3-Clause | 1.85 | Device signing (`SigningKey` / `VerifyingKey`) | **Add** (workspace dep) |
| **x25519-dalek** | **3.0.0** | BSD-3-Clause | 1.85 | Static + ephemeral ECDH | **Add** |
| **curve25519-dalek** | **5.0.0** | BSD-3-Clause | 1.85.0 | **Transitive** of both dalek 3.x | Inventory + deny; do not depend directly unless needed |
| **hkdf** | **0.13.0** | MIT OR Apache-2.0 | 1.85 | Wrap-key KDF (SHA-256) | **Add** |
| **hmac** | (via hkdf 0.13 → hmac ^0.13) | MIT OR Apache-2.0 | — | Transitive of hkdf | Inventory only |
| **aes-gcm** | workspace **0.10.x** | Apache-2.0 OR MIT | — | Content AEAD + DEK wrap AEAD | **Reuse** (do not bump to 0.11 in T176) |
| **zeroize** | workspace **1.8** | Apache-2.0 OR MIT | — | Key material wipe | **Reuse** |
| **rand** / **rand_core** | workspace rand **0.10** | MIT OR Apache-2.0 | — | Entropy; dalek 3.x uses **rand_core 0.10** | Align; expect multi-version warn already present |
| **sha2** | workspace **0.11** | MIT OR Apache-2.0 | — | Fingerprint SHA-256; HKDF hash | **Reuse** |
| **subtle** | workspace **2.6** | BSD-3-Clause | — | Constant-time | **Reuse** |
| **hpke** | **0.14.0** | MIT/Apache-2.0 | 1.85 | RFC 9180 standard wrap | **Do not add** (ADR-0018 §18 deferred) |
| **openmls** | **0.8.1** | MIT | — | MLS groups | **Do not add** (L15) |
| **merlin** | 3.0.0 | MIT | — | Not a normal dep of dalek 3.0.0 | **Not required** |

**Confirmed normal deps of ed25519-dalek 3.0.0:** `curve25519-dalek ^5`, `ed25519 ^3`, `sha2 ^0.11`, `subtle ^2.3`; optional `rand_core ^0.10`, `serde`, `zeroize`, `signature`.

**Confirmed normal deps of x25519-dalek 3.0.0:** `curve25519-dalek ^5`, `rand_core ^0.10`; optional `serde`, `zeroize`, `getrandom`.

**deny.toml:** BSD-3-Clause, MIT, Apache-2.0 already allowlisted. Gate: `cargo deny check` + `cargo audit` after first Cargo.toml change, including **curve25519-dalek 5.x** transitive.

### 3.2 Intended Cargo pins (T176)

```toml
# workspace.dependencies (illustrative)
ed25519-dalek = { version = "3", default-features = true, features = ["serde", "zeroize", "rand_core"] }
# default already includes zeroize+fast; restate zeroize for audit clarity
x25519-dalek = { version = "3", default-features = true, features = ["serde", "zeroize", "static_secrets"] }
# NO getrandom feature — its random() constructors panic; see R25 panic-free keygen
hkdf = "0.13"
```

| Flag | Why |
|------|-----|
| ed25519 `serde` + `zeroize` + `rand_core` | Durable publics; ZeroizeOnDrop secrets; byte-oriented construct APIs |
| x25519 `static_secrets` | Long-term recipient `StaticSecret` for peer wraps (device identity) |
| x25519 `zeroize` (default) | Wipe on drop; dalek 3.0 removed Zeroize *impl* misuse surface — still zeroizes on drop |
| **Do not** enable x25519 `getrandom` | Enables `EphemeralSecret::random()` / `StaticSecret::random()` that panic on entropy failure — violates AGENTS.md |
| **Do not** enable `reusable_secrets` unless a protocol step needs multi-use ephemeral without static | Prefer one-shot eph seed per wrap |

**API notes (dalek 3.x / edition 2024) — panic-free keygen (R25):**

- Ed25519: `SigningKey` / `VerifyingKey` (not 2.x `Keypair`).
- **Forbidden in production:** `SigningKey::generate` via `UnwrapErr(SysRng)`, `EphemeralSecret::random()`, any path that panics on entropy failure.
- **Required pattern:** fallible OS CSPRNG fill, then construct from bytes:

```text
SysRng.try_fill_bytes(&mut seed32)?   // map entropy error → thiserror
SigningKey::from_bytes(&seed32)
StaticSecret::from(seed32)            // long-term X25519
# ephemeral per wrap: fresh 32-byte seed → StaticSecret / bare x25519 once; never reuse seed
```

- Prefer `try_generate_from_rng` only if it returns `Result` and takes `TryCryptoRng` without panic adapters — else always use fill-then-`from_bytes`.
- Independent Ed25519 and X25519 keypairs (never convert signing key → DH key).
- Shared secret bytes → **HKDF only** (never raw as AES key).

**Audit note (pre-clear):** OSV/RUSTSEC historical dalek advisories fixed in 2.x/4.x are irrelevant at pins 3.0.0 / 5.0.0; still run `cargo audit` at implement.

### 3.3 HKDF salt call style (normative implement note)

ADR-0018 §17.2: **salt = empty (zero-length byte string)** — wording retained.

RustCrypto `hkdf` 0.13:

| Call | Salt fed to HMAC-Extract |
|------|--------------------------|
| `Hkdf::<Sha256>::new(None, ikm)` | Library substitutes **HashLen zeros** (32×0x00) |
| `Hkdf::<Sha256>::new(Some(&[]), ikm)` | **Zero-length** slice |

**Cryptographic equivalence (SHA-256):** HMAC (RFC 2104) zero-pads any key shorter than the block size (64 B) to 64 B. Both a 32-byte all-zero salt and a 0-byte salt pad to the **same** 64-byte all-zero HMAC key, so PRK/OKM are **byte-identical** for SHA-256. This is **not** a divergent-output footgun.

**T176 freeze (clarity, not crypto distinction):** use **`Hkdf::<Sha256>::new(Some(&[]), shared.as_bytes())`** so the call site matches ADR “empty salt” wording and code review does not debate `None`. KATs pin expected OKM for fixed IKM+info; they must **not** claim `None` vs `Some(&[])` produce different outputs under SHA-256.

Label / info / AAD bytes: ADR-0018 §17.1–17.3 exactly (`aib-sync-dek-wrap`, length-prefixed label, fixed UUID fields).

### 3.4 Standards & practice alignment

| Source | Takeaway for T176 |
|--------|-------------------|
| **ADR-0018** (normative) | Envelope layout, wrap construction, control cleartext, CLI names, migration 0027+ |
| **ADR-0016** | Local content DEK under DataKey; multi-device does **not** share DataKey |
| **NIST SP 800-38D** | 96-bit random nonce per GCM invocation; O(1) seals per wrap_key (ephemeral ECDH) |
| **NIST SP 800-88r2** | No remote Purge claim; multi-device CE = best-effort propagation |
| **RFC 5869 HKDF** + **RFC 2104 HMAC** | Extract-Expand; empty vs omitted salt equivalent under SHA-256 key padding (see §3.3) |
| **RFC 9180 HPKE** | Evaluated; **deferred** — explicit X25519+HKDF+AES-GCM for auditability |
| **RFC 9420 MLS** | Deferred under single-owner L15 |
| **Industry E2EE** | Fingerprint-bound enroll (Signal safety numbers / dual-key package); future exclusion on revoke; no silent auto-trust; metadata honesty |

### 3.5 Architecture placement

```
ai-brains-core          DeviceId, ContentKeyId, ReplicationEventId
ai-brains-crypto        DataKey, content DEK seal/wrap (local); DPAPI helpers
ai-brains-store         migration 0027 + SQL APIs for replication tables
ai-brains-sync  (NEW)   envelope wire codec, sign/verify, peer wrap, fingerprint,
                        enrollment package, control payload types
ai-brains-cli           device / replicate subcommands (optional dep on sync)
ai-brains-capture       MUST NOT depend on ai-brains-sync
```

**Crypto ownership split:**

| Concern | Crate |
|---------|-------|
| Local content DEK under DataKey | `ai-brains-crypto` (existing) |
| Device Ed25519/X25519 keygen, sign, peer DEK wrap | **`ai-brains-sync`** (new; may call `aes-gcm` / reuse small helpers from crypto) |
| SQL rows | `ai-brains-store` |
| Prefer **not** dumping all of ADR-0018 into `ai-brains-crypto` | Keeps local CE path independent of multi-device |

Optional: thin re-export of AES-GCM seal helpers from crypto if duplication is painful — do **not** create a circular store↔sync edge that capture can hit.

### 3.6 Feature / optionality strategy

| Choice | Decision |
|--------|----------|
| Default product | **Local-only**; multi-device off |
| Crate always in workspace | Yes — library available for tests |
| CLI dependency | `ai-brains-cli` depends on `ai-brains-sync` for `device`/`replicate` only |
| Capture / models / graph | **No** sync dependency |
| Feature flag on workspace | Prefer **always-compile** `ai-brains-sync` (simpler CI) unless binary size forces `sync` feature — if feature-gated, gate **only** CLI subcommands + daemon hooks, not migration registration (schema may still ship empty tables) |

**Recommended v1:** always compile crate + migration (empty tables harmless); subcommands no-op / error clearly when no local device enrolled. Avoid dual schema paths.

## 4. Design locks (normative for implement)

| ID | Lock |
|----|------|
| **R1** | Migration file: **`0027_replication_state.sql`**; register in `MIGRATIONS` after 0026. **Never** reuse 0026. |
| **R2** | Crate: **`crates/ai-brains-sync`** workspace member; library name ≠ CLI `sync`. |
| **R3** | CLI: **`ai-brains device`** (enroll / list / revoke / fingerprint) + **`ai-brains replicate`** (push / pull / status / cursors). **Forbidden** to extend `Commands::Sync` or `SafetyCommands::Sync` for multi-device. |
| **R4** | No plaintext event body columns in any 0027 table. Control payloads are cleartext **membership/integrity** bytes under outer signature (not content secrecy) — store as opaque BLOB only if needed for local ACK/debug; prefer re-derive from applied control events where possible. |
| **R5** | Wrap table primary key: **`(content_key_id, recipient_device_id)`**. Columns hold eph X25519 pub, wrap_nonce, wrap_ct (AES-GCM ct‖tag of content DEK). No plaintext DEK. **Upsert:** new wrap for same PK **replaces** prior row; `sender_device_id` is audit (who sealed current wrap). No multi-sender simultaneous history in v1. |
| **R6** | Device private keys **dual-layer on Windows:** (1) AES-256-GCM under vault **`DataKey`** with frozen inner blob + AAD (§5.1.1); (2) outer **OS DPAPI** via `ai_brains_crypto::dpapi` on the sealed blob at rest. Non-Windows: DataKey layer only (DPAPI N/A). Never store raw Ed25519/X25519 secrets in SQL. |
| **R7** | Enrollment package + fingerprint per ADR-0018 L3 (schema_version ‖ DeviceId ‖ Ed25519_pub ‖ X25519_pub; `SHA-256` of full package). CLI displays fingerprint as **4-char hyphen-separated hex** (R24). |
| **R8** | Control type codes (u16) frozen in this track’s constants module (see §6). Same stream as data envelopes. Unknown codes → **reject** (fail closed). |
| **R9** | Data body wire packing: `nonce(12) ‖ ct ‖ tag(16)`; local store may keep separate nonce columns and convert at wire boundary. |
| **R10** | Outer `signed_bytes` exact concat ADR-0018 §5.2; wrap list sorted by `recipient_device_id` ascending; unsorted → reject. |
| **R11** | Per-recipient wrap: X25519 eph → HKDF-SHA256 with **`Some(&[])`** salt (clarity; equivalent to `None` under SHA-256 HMAC padding) → AES-256-GCM; info/AAD per §17. |
| **R12** | Enrolled-set gate **before** signature verify for **relay-received** envelopes (unknown `device_id` reject without verify). Status **`active` or `local`** passes; **`revoked`** fails. Local first-device bootstrap is **not** relay-received (R26). |
| **R13** | Gap policy: fail-closed default; operator skip only with **signed `GapSkipAudit` control envelope** appended to the **local event log**; `replication_gap_skip_audit` is an index on `signed_event_id`, not SOV of the signature. |
| **R14** | ACK projection: `(erasure_id, peer_device_id) → pending \| acked \| failed \| unreachable`; default timeout **N = 3** sync cycles (`ACK_TIMEOUT_SYNC_CYCLES = 3`). ACK = peer attestation, not wipe proof. `tick_ack_cycle` is a **no-op in production T176** (no relay cycles); unit-testable; wired in T177. |
| **R15** | Single-owner / single-vault only (L15). No multi-user enrollment path. |
| **R16** | Revoked `DeviceId` **permanently retired** (tombstone table/status); re-enroll = new DeviceId + new keys + full OOB. |
| **R17** | Zero `unwrap`/`expect`/`panic` in production paths; `thiserror` (+ miette at CLI boundary as existing pattern). |
| **R18** | No sockets / no HTTP relay client in T176. `replicate push/pull` may validate local readiness and return **“relay not configured / T177”** structured error. |
| **R19** | Size-bucket padding helpers (256 / 4096 / 65536) in sync crate for seal path; padding is best-effort (L14), not metadata privacy claim. |
| **R20** | HPKE / OpenMLS / epoch-group KEK **not** implemented. |
| **R21** | DataKey **rotation implementation** out of scope (deferred #34.2 residual remains open). |
| **R22** | Capture independence: no `ai-brains-capture` → `ai-brains-sync` edge. |
| **R23** | **Revocation wrap cleanup:** on `device revoke` or apply of `DeviceRevoked`, run `DELETE FROM peer_content_key_wrap WHERE recipient_device_id = ?` (and mark identity revoked + tombstone). Does **not** erase past DEKs already unwrapped on the stolen device (L4 residual). |
| **R24** | **Human-readable fingerprint:** CLI prints SHA-256 as uppercase or lowercase hex in **4-character groups separated by hyphens** (e.g. `5f3a-9b1c-4e8f-…` for 32 bytes → 16 groups). Stable for OOB voice/visual compare; machine APIs may also emit raw hex. |
| **R25** | **Panic-free keygen:** fill 32 bytes via `SysRng.try_fill_bytes` (or equivalent fallible CSPRNG) → `SigningKey::from_bytes` / `StaticSecret::from`; **no** `UnwrapErr`, **no** x25519 `getrandom` feature / panic `random()` constructors. |
| **R26** | **First-device bootstrap exception:** first device status=`local`, self-signs local `DeviceEnrolled`, `enrolled_by_device_id = self`; never arrives as untrusted-relay self-enroll. L8 enrolled-set gate applies to relay-received envelopes (T177+). |
| **R27** | **`device bootstrap` idempotent-reject:** fails structured error if any row with status `active` or `local` already exists. Further devices use `device enroll` ceremony (§9.1). |
| **R28** | **Private-key blob layout v1** frozen in §5.1.1; bump `wrap_schema_version` on any change. |
| **R29** | **No `content_hash_sha256` column** on envelope index (outer signature is authoritative integrity). |
| **R30** | **`apply_order` in sync crate** = pre-decryption deterministic order by `(device_id, local_seq, event_id)` only. Domain topo (parent/correlation) is store/projector **after** DEK open — not in `ai-brains-sync`. |

## 5. Schema (migration `0027_replication_state.sql`)

Illustrative DDL — column names normative intent; adjust only for SQLCipher/SQLite idioms already used in 0026.

### 5.1 Side stores (not truncated on full projection rebuild)

Like CE side stores: durable crypto/replication material that is **not** pure event projection.

#### Status enum (`device_identity.status`) — freeze (B2)

| Status | Meaning | Enrolled-set (L8) | Receives peer wraps | Notes |
|--------|---------|-------------------|---------------------|--------|
| **`local`** | First device; RecoveryKit-bound bootstrap | **Yes** | **Yes** | Enrollment **class**, not a temporary lifecycle; **does not** transition to `active` when peers join |
| **`active`** | Peer enrolled via OOB + signed `DeviceEnrolled` | **Yes** | **Yes** | |
| **`revoked`** | Future exclusion applied | **No** | **No** (R23 deletes wrap rows) | DeviceId also in `device_id_tombstone` |

#### First-device bootstrap (B3 / R26)

- First device creates a **local** `DeviceEnrolled` control record (never relay-received in T176).
- **Self-signs** with its own Ed25519 key; `enrolled_by_device_id = device_id` (self), **not NULL**.
- L8 gate applies to **relay-received** envelopes only (T177+).

```sql
-- Local + peer public identity (public keys only for peers).
CREATE TABLE device_identity (
    device_id            TEXT PRIMARY KEY,
    schema_version       INTEGER NOT NULL DEFAULT 1,
    ed25519_public       BLOB NOT NULL,   -- 32 bytes
    x25519_public        BLOB NOT NULL,   -- 32 bytes
    display_name         TEXT,
    status               TEXT NOT NULL,   -- 'active' | 'revoked' | 'local' (see table above)
    enrolled_at          TEXT NOT NULL,
    revoked_at           TEXT,
    enrolled_by_device_id TEXT NOT NULL,  -- signer of DeviceEnrolled; self for first local
    fingerprint_sha256   BLOB NOT NULL,   -- 32 bytes of enrollment_package hash
    CHECK (status IN ('active', 'revoked', 'local')),
    CHECK (length(ed25519_public) = 32),
    CHECK (length(x25519_public) = 32),
    CHECK (length(fingerprint_sha256) = 32)
);

-- Permanently retired DeviceIds (L3/L4). Insert on revoke; never delete.
CREATE TABLE device_id_tombstone (
    device_id     TEXT PRIMARY KEY,
    revoked_at    TEXT NOT NULL,
    reason_code   TEXT NOT NULL DEFAULT ''
);

-- Local device private key material (this vault's device only).
-- Inner AES-GCM under DataKey (§5.1.1); Windows: outer DPAPI on stored blob (R6).
CREATE TABLE device_private_key_store (
    device_id            TEXT PRIMARY KEY,
    wrap_schema_version  INTEGER NOT NULL DEFAULT 1,
    algorithm            TEXT NOT NULL DEFAULT 'AES-256-GCM',
    protection           TEXT NOT NULL DEFAULT 'datakey',
    -- 'datakey' | 'datakey_dpapi' (Windows dual-layer)
    wrap_nonce           BLOB NOT NULL,  -- 12-byte GCM nonce for DataKey layer
    wrap_ciphertext      BLOB NOT NULL,  -- ct‖tag of inner seeds; or DPAPI(ct‖tag) when protection=datakey_dpapi
    created_at           TEXT NOT NULL,
    CHECK (protection IN ('datakey', 'datakey_dpapi')),
    FOREIGN KEY (device_id) REFERENCES device_identity(device_id)
);

-- Per-recipient multi-device content DEK wraps (projected from verified data envelopes
-- and/or produced locally when sealing for peers).
-- PK upsert: latest verified wrap wins (R5); sender_device_id is audit only.
CREATE TABLE peer_content_key_wrap (
    content_key_id       TEXT NOT NULL,
    recipient_device_id  TEXT NOT NULL,
    sender_device_id     TEXT NOT NULL,
    schema_version       INTEGER NOT NULL DEFAULT 1,
    eph_x25519_public    BLOB NOT NULL,  -- 32 bytes
    wrap_nonce           BLOB NOT NULL,  -- 12 bytes
    wrap_ciphertext      BLOB NOT NULL,  -- ct‖tag of content DEK
    created_at           TEXT NOT NULL,
    PRIMARY KEY (content_key_id, recipient_device_id),
    CHECK (length(eph_x25519_public) = 32),
    CHECK (length(wrap_nonce) = 12)
);

-- Opaque envelope index for replication (no plaintext bodies).
-- Integrity of body: outer Ed25519 over signed_bytes only (R29 — no content_hash column).
CREATE TABLE encrypted_envelope_index (
    envelope_id          TEXT PRIMARY KEY,
    event_id             TEXT NOT NULL UNIQUE,
    sender_device_id     TEXT NOT NULL,
    local_seq            INTEGER NOT NULL,
    content_type_code    INTEGER NOT NULL,
    content_key_id       TEXT,            -- zero UUID for most control
    body_len             INTEGER NOT NULL,
    padding_bucket       INTEGER,         -- 256 | 4096 | 65536 when applied
    applied_at           TEXT,
    UNIQUE (sender_device_id, local_seq)
);

CREATE INDEX idx_envelope_sender_seq
    ON encrypted_envelope_index (sender_device_id, local_seq);
```

#### 5.1.1 Device private-key inner blob (v1 — R28)

Single AES-256-GCM seal under vault `DataKey` of **64-byte** plaintext:

```text
plaintext = ed25519_seed[32] ‖ x25519_seed[32]
```

| Field | Value |
|-------|--------|
| Nonce | 12-byte OS CSPRNG, fresh per seal |
| Ciphertext storage | `ct ‖ tag` (same as content DEK wrap) |
| AAD | `AIBC` ‖ `kind=0x03` ‖ `wrap_schema_version` u32 BE ‖ `device_id` 16 raw UUID bytes |

```text
aad =
    magic[4] = b"AIBC"
  ‖ kind[1]  = 0x03          # AAD_KIND_DEVICE_PRIVATE_KEY (distinct from content 0x01 / DEK wrap 0x02)
  ‖ version  = u32 BE (wrap_schema_version)
  ‖ device_id[16]
```

- Independent seeds (signing ≠ DH).
- Windows at rest: `protection = 'datakey_dpapi'`; `wrap_ciphertext` column holds **DPAPI-protect(ct‖tag)** (or DPAPI of a small framing blob that includes nonce if implementer prefers — then document in code; **recommend** keep `wrap_nonce` column clear + DPAPI only the ct‖tag so open path is: DPAPI unwrap → AES-GCM open with nonce + AAD).
- Bump `wrap_schema_version` on any layout change.

### 5.2 Event-derived / operational projections (rebuild policy explicit)

```sql
-- Per peer stream cursor + gap state (L13).
CREATE TABLE replication_cursor (
    peer_device_id       TEXT PRIMARY KEY,
    high_water_seq       INTEGER NOT NULL DEFAULT 0,
    expected_local_seq   INTEGER NOT NULL DEFAULT 1,
    state                TEXT NOT NULL DEFAULT 'in_sync',
    -- 'in_sync' | 'sync_gap' | 'blocked'
    updated_at           TEXT NOT NULL,
    CHECK (state IN ('in_sync', 'sync_gap', 'blocked'))
);

-- Gap buffer: seq/envelope_id metadata only (B8 / R30 adjacent).
-- Bodies are NOT stored here; T177 re-fetches by seq range from the relay until gap fills.
-- T176 creates the table empty; no production writer without a relay.
CREATE TABLE replication_gap_buffer (
    peer_device_id       TEXT NOT NULL,
    local_seq            INTEGER NOT NULL,
    envelope_id          TEXT NOT NULL,
    buffered_at          TEXT NOT NULL,
    PRIMARY KEY (peer_device_id, local_seq)
);

-- Erasure ACK projection (L7 / deferred #34.1 implement).
CREATE TABLE erasure_ack_projection (
    erasure_id           TEXT NOT NULL,
    peer_device_id        TEXT NOT NULL,
    content_key_id       TEXT NOT NULL,
    status               TEXT NOT NULL,
    -- 'pending' | 'acked' | 'failed' | 'unreachable'
    sync_cycles_waiting  INTEGER NOT NULL DEFAULT 0,
    updated_at           TEXT NOT NULL,
    PRIMARY KEY (erasure_id, peer_device_id),
    CHECK (status IN ('pending', 'acked', 'failed', 'unreachable'))
);

CREATE INDEX idx_erasure_ack_status
    ON erasure_ack_projection (status);

-- Operator gap-skip audit index (R13 / B11): authoritative signed envelope lives in the event log.
-- signed_event_id references the GapSkipAudit control event_id (idempotent apply).
CREATE TABLE replication_gap_skip_audit (
    audit_id             TEXT PRIMARY KEY,
    peer_device_id        TEXT NOT NULL,
    skipped_seq          INTEGER NOT NULL,
    signed_event_id      TEXT NOT NULL,
    created_at           TEXT NOT NULL
);
```

### 5.3 Rebuild policy

| Table class | On `rebuild_projections` |
|-------------|--------------------------|
| `device_identity`, `device_id_tombstone`, `device_private_key_store`, `peer_content_key_wrap`, `encrypted_envelope_index` | **Retain** (side stores / durable crypto index) — document like CE |
| `replication_cursor`, `replication_gap_buffer`, `erasure_ack_projection`, `replication_gap_skip_audit` | **Policy:** retain by default (operational state not pure event log); if a later track event-sources control fully, may truncate+replay — **v1 retain** to avoid wiping gap/ACK mid-flight |

Document explicitly in `replay.rs` comments (mirror T163 CE comments).

### 5.4 Naming collisions

| Name | Meaning | Multi-device? |
|------|---------|--------------|
| `sync_state` (0017) | Ledgerful/helper KV | **No** |
| CLI `sync` | Ledgerful bridge | **No** |
| CLI `safety sync` | Hotspot pin | **No** |
| `ai-brains-sync` crate | Multi-device E2EE library | **Yes** |
| CLI `device` / `replicate` | Multi-device ops | **Yes** |
| `peer_content_key_wrap` | Multi-device DEK wraps | **Yes** (distinct from local `content_key_store`) |

## 6. Wire / type constants

### 6.1 Schema version

| Constant | Value |
|----------|-------|
| `REPLICATION_SCHEMA_VERSION` (envelope / enrollment) | **`1` u16** |

### 6.2 Content type codes (`u16`)

| Code | Name | Kind |
|------|------|------|
| `0x0001` | `DataEvent` | Data (DEK body + wraps) |
| `0x0010` | `DeviceEnrolled` | Control |
| `0x0011` | `DeviceRevoked` | Control |
| `0x0012` | `ContentErasureTombstone` | Control — **sole** erasure/tombstone signaling code (ADR “KeyTombstone” alias collapsed; payload may carry reason discriminator later) |
| `0x0013` | `ErasureAck` | Control |
| `0x0014` | `GapSkipAudit` | Control (signed operator skip) |

**Reservations (B14):**

| Range | Policy |
|-------|--------|
| `0x0000` | Permanently unused (sentinel) |
| `0x0002`–`0x000F` | Reserved future **data** subtypes |
| `0x0015`+ | Reserved future **control** types |
| `0x0013` former KeyTombstone | **Reassigned** to `ErasureAck` (no separate KeyTombstone code) |

Unknown `content_type_code` at parse → **fail closed** (reject). Domain event kind discrimination **inside** DEK plaintext is separate from outer `content_type_code`.

### 6.3 Padding buckets (L14)

`256`, `4096`, `65536` bytes (plaintext pad-before-seal). Helper: `fn pad_to_bucket(len) -> bucket`.

### 6.4 ACK timeout

`ACK_TIMEOUT_SYNC_CYCLES: u32 = 3`.

## 7. Crate API sketch (`ai-brains-sync`)

Module layout (illustrative):

```
ai-brains-sync/
  src/
    lib.rs
    error.rs
    ids.rs              // re-export core DeviceId etc.
    enrollment.rs       // package bytes, fingerprint, OOB helpers
    device_keys.rs      // generate dual keypair; zeroize wrappers
    signed_bytes.rs     // canonical concat + verify
    envelope.rs         // outer envelope encode/decode
    control.rs          // control payload structs
    wrap.rs             // per-recipient DEK wrap/unwrap (ADR §17)
    padding.rs          // L14 buckets
    apply_order.rs      // pre-decrypt tie-break only (R30); not domain topo
    fingerprint_fmt.rs  // R24 hyphen groups
```

Public operations (names flexible; behaviors frozen):

| API | Behavior |
|-----|----------|
| `generate_device_keys()` | Ed25519 + X25519 via **R25** fallible fill; secrets zeroizing |
| `enrollment_package(...)` / `fingerprint_sha256(...)` / `format_fingerprint_hyphen(...)` | L3 + R24 |
| `sign_envelope` / `verify_envelope` | L5/L8; enrolled-set check is **caller's** pre-gate or combined API |
| `wrap_content_dek_for_recipient` / `unwrap_content_dek` | §17 |
| `encode_data_body` / `decode_data_body` | nonce‖ct‖tag packing |
| `pad_plaintext` | L14 |
| Control encode/decode for enroll/revoke/tombstone/ack | Cleartext payloads |
| `envelope_stream_order(...)` | Sort by `(device_id, local_seq, event_id)` only (R30) |

**Errors:** structured (`UnknownDevice`, `SignatureInvalid`, `WrapOpenFailed`, `TombstonedDeviceId`, `NotEnrolled`, `UnsortedWrapList`, `BootstrapAlreadyEnrolled`, `EntropyFailed`, …) — no plaintext leak on auth failure.

## 8. Store APIs sketch (`ai-brains-store`)

Mirror `content_envelope` module style:

- `insert_device_identity` / `list_enrolled_devices` (`active`+`local`) / `get_device` / `tombstone_device`
- `put_device_private_key_wrap` / `get_device_private_key_wrap` (dual-layer aware)
- `upsert_peer_content_key_wrap` / `get_peer_wrap` / `delete_peer_wraps_for_key` (CE) / **`delete_peer_wraps_for_recipient`** (R23 revoke)
- `insert_envelope_index` / `envelope_exists(event_id)` (idempotent apply)
- `get_cursor` / `set_cursor` / `set_gap_state` / `buffer_gap_seq` (metadata only)
- `upsert_erasure_ack` / `list_pending_acks` / `tick_ack_cycle`
  - **`tick_ack_cycle`:** production T176 path does not invoke it (no relay). Unit tests may drive increment → `unreachable` at N=3. T177 wires real cycles.

No plaintext content parameters.

## 9. CLI surface

### 9.1 `ai-brains device`

| Subcommand | Behavior in T176 |
|------------|------------------|
| `device bootstrap` | First-device local enroll (RecoveryKit-bound); status=`local`; self-signed `DeviceEnrolled`; dual-wrap private keys. **Fails** if any `active`/`local` device exists (R27). |
| `device fingerprint` | Print dual-key fingerprint in **R24** hyphen form (and raw hex if `--raw`) |
| `device list` | Table/JSON of enrolled devices + status |
| `device package-export` (or bootstrap on new machine) | On the **new** machine: generate keys + write enrollment package file / print package; **does not** enroll into a peer vault |
| `device enroll --package <path>` | On **already-enrolled** vault: load package, show fingerprint, confirm OOB, sign `DeviceEnrolled`, insert `device_identity` (status=`active`) |
| `device revoke <device_id>` | Local revoke + tombstone + **R23** delete peer wraps for recipient |

#### Enrollment ceremony in T176 (no relay) — B9

```text
1. New machine: device package-export (or bootstrap --package-only)
     → keys + enrollment_package bytes + fingerprint (R24)
2. Operator transfers package to enrolled machine (file / QR / clipboard)
3. Enrolled machine: device enroll --package <path>
     → display fingerprint; human confirms OOB match
4. Enrolled machine: signs DeviceEnrolled (signer must be active|local),
     appends control record to local event log, inserts device_identity
5. New machine does NOT yet receive peer state in T176
     → T177 relay delivers DeviceEnrolled + history so the new vault converges
```

T176 only mutates the **signing (already enrolled)** vault for peer enroll. First-device uses `bootstrap` on that vault alone.

### 9.2 `ai-brains replicate`

| Subcommand | Behavior in T176 |
|------------|------------------|
| `replicate status` | Local cursors, gap state, enrolled count, “relay: not configured” |
| `replicate cursors` | Dump `replication_cursor` |
| `replicate push` / `pull` | **Fail closed** with clear error: network/fake relay deferred to T177 (or no-op dry-run of local seal readiness) |

Docs/help text must state multi-device is optional and **not** metadata-private / **not** PQ / **not** remote wipe.

## 10. Interaction with P8 content envelopes

```
Local seal (ADR-0016):
  ContentDek → AES-GCM content
  DataKey    → wrap ContentDek → content_key_store

Multi-device (ADR-0018):
  For each peer: ephemeral X25519 + HKDF + AES-GCM → peer_content_key_wrap
  Outer sign envelope including wrap list
```

- Destroying local wrap (CE) should also remove or mark peer wraps for that `content_key_id` when CE runs with multi-device enabled (hook stub OK in T176; full multi-device CE orchestration may complete in T177–T178).
- **Minimum T176:** SQL + APIs so T165 wipe **can** call `delete_peer_wraps_for_key`; wire the call if low-risk, else document follow-up without blocking.

## 11. Deferred.md absorption

| Item | Disposition in T176 |
|------|---------------------|
| **#50** T175 freezes / ADR-0018 | **Implement under locks** R1–R30 |
| **#34.1** multi-device key tombstone / erasure ACK | **Schema + types + local projection APIs**; signed ACK encode/verify unit tests. Full multi-device CE orchestration proof → T177/T178 |
| **#34.2** DataKey rotation implementation | **Out of scope** — residual remains open (P11 hygiene) |
| **#34.3** historical T162–T165 | Already done |
| HPKE / MLS / epoch KEK | Deferred per ADR — no code |
| Workspace dep hygiene #40 | May add dalek/hkdf; **do not** broad-bump unrelated crates |

## 12. Testing strategy (T176 scope)

### 12.1 Unit / crate tests (`ai-brains-sync`)

| Test id (name pattern) | Intent |
|------------------------|--------|
| `enrollment_package__fingerprint__stable_hex` | Dual-key package hash stable |
| `fingerprint_format__hyphen_groups__16_groups` | R24 |
| `signed_bytes__canonical_concat__matches_fixture` | §5.2 byte KAT |
| `verify__meta_swap__fails_closed` | L5 |
| `wrap__hkdf_okm__matches_kat` | §17 OKM for fixed IKM+info (`Some(&[])` style) |
| `wrap__roundtrip_recipient__ok` | Happy path |
| `wrap__wrong_recipient__fails` | Isolation |
| `wrap__unsorted_list_sign__reject_or_sort_policy` | v1 reject unsorted |
| `data_body__nonce_ct_tag_packing__roundtrip` | §5.3 |
| `control__wrap_count_zero__parse` | Control path |
| `padding__bucket_selection__deterministic` | L14 |
| `apply_order__tie_break__device_seq_event` | Pre-decrypt order only (R30) |
| `device_keys__generate__no_panic_on_entropy_path` | R25 uses Result, not UnwrapErr |
| `device_private_blob__aad_layout__stable` | §5.1.1 AAD bytes |

Naming: `function_or_feature__condition__expected_result` (AGENTS.md).

### 12.2 Store / migration tests

| Test | Intent |
|------|--------|
| Fresh vault applies 0027 | Tables exist |
| Existing vault upgrades 0026→0027 | Forward-only |
| CHECK constraints | Bad status / wrong key lengths fail |
| No plaintext columns | Schema inspection or insert API types |
| Idempotent envelope index | Same event_id |
| Tombstone blocks re-enroll | Insert active after tombstone fails or API rejects |
| Peer wrap PK | Upsert replaces same (content_key_id, recipient) |
| Revoke deletes recipient wraps | R23 |
| Bootstrap second time | R27 structured err |
| First device enrolled_by self | R26 |

### 12.3 CLI smoke (manual + assert_cmd if pattern exists)

- `device bootstrap` / `list` / `fingerprint` (hyphen form) on temp vault  
- Second `device bootstrap` fails  
- `replicate status` shows not-configured relay  
- `sync query` still Ledgerful; unchanged help for `safety sync`

### 12.4 License / security gate

```powershell
cargo deny check
cargo audit
cargo tree -i curve25519-dalek
cargo nextest run -p ai-brains-sync -p ai-brains-store
```

### 12.5 Explicitly deferred to T178

Threat-model §7 matrix items that need multi-device adversary harness (unknown device injection E2E, forged ACK over relay, full WRAP KATs with golden vectors file, PQ/non-claim doc tests). T176 may land **preliminary** KATs that T178 promotes to authoritative fixtures.

## 13. Non-goals

- Real or fake network relay (T177)
- Complete security claim matrix (T178)
- Changing capture pipeline
- SQLCipher file sync / CRDT default
- Multi-user vault / MLS
- HPKE crate adoption
- DataKey rotation implementation
- Sealed-sender / metadata-hiding
- Post-quantum hybrid KEM
- Repurposing CLI `sync` or `safety sync`

## 14. License / commercial gate

New crates: **BSD-3-Clause** (dalek family) + **MIT OR Apache-2.0** (hkdf) — allowlisted. No AGPL/GPL. No unknown-git crypto. Hand-rolled ECC/AES forbidden.

Project remains PolyForm NC + Small-Entity Commercial Exception; optional multi-device so local-only commercial use needs no relay.

## 15. Risks & residuals (honesty)

| Residual | Handling |
|----------|----------|
| Metadata leakage (sizes, graph, timing) | L14 padding only; docs non-claim |
| PQ harvest-now-decrypt-later | L16 non-claim |
| ACK ≠ wipe proof | L7 residual; UX language |
| Offline peer lag | Best-effort CE |
| DataKey vault-lifetime nonce budget | #34.2 open |
| Revoked device past DEKs | Future exclusion only; R23 only purges **local** wrap cache for that recipient |
| HKDF salt call style | R11 clarity (`Some(&[])`); SHA-256 equivalent to `None` |

## 16. Definition of Done

- [ ] `crates/ai-brains-sync` workspace member; deps deny+audit green incl. curve25519-dalek 5.x (no x25519 `getrandom` feature)
- [ ] Migration `0027_replication_state` registered and tested empty-state
- [ ] Schema matches §5 intent (no plaintext bodies; no content_hash column)
- [ ] Sign/verify + per-recipient wrap unit tests green (HKDF OKM KAT)
- [ ] Device private key dual-wrap (DataKey + DPAPI on Windows) per §5.1.1; no raw secrets in SQL
- [ ] R23 revoke wrap cleanup; R24 fingerprint format; R25 panic-free keygen; R26/R27 bootstrap rules
- [ ] CLI `device` + `replicate` present; Ledgerful `sync` / `safety sync` untouched
- [ ] Capture independence preserved (no capture→sync dep)
- [ ] `replicate push/pull` do not open sockets
- [ ] conductor + deferred updated; review log started; full CI gate green
- [ ] Manual evidence recorded (bootstrap/list/fingerprint/status)

## 17. Review fold-in (2026-07-30 AI1–AI3)

| Source | Disposition |
|--------|-------------|
| AI1 dual-layer DPAPI | **Agree** → R6 mandatory dual-layer on Windows |
| AI1 R23 revoke wrap DELETE | **Agree** → R23 |
| AI1 R24 hyphen fingerprint | **Agree** → R24 |
| AI2 HKDF None vs Some(&[]) | **Agree** — correct rationale: SHA-256 equivalent; freeze `Some(&[])` for clarity only (§3.3) |
| AI2 OSV pre-clear / no drift | **Agree** — confirmatory |
| AI3 B1 panic-free keygen | **Agree** → R25; drop getrandom feature |
| AI3 B2 `local` status | **Agree** → §5.1 status table (enrolled for L8; permanent class) |
| AI3 B3 first-device self-sign | **Agree** → R26; `enrolled_by = self` |
| AI3 B4 private blob layout | **Agree** → §5.1.1 / R28 |
| AI3 B5 peer wrap upsert | **Agree** → R5 |
| AI3 B6 drop content_hash | **Agree** → R29 |
| AI3 B7 apply_order scope | **Agree** → R30 |
| AI3 B8 gap metadata-only | **Agree** → §5.2 comment |
| AI3 B9 enroll ceremony | **Agree** → §9.1 |
| AI3 B10 bootstrap reject | **Agree** → R27 |
| AI3 B11–B14 | **Agree** — event-log GapSkipAudit; tick stub; single 0x0012 tombstone; reserved ranges |

## 18. References

- `Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md`
- `Docs/DECISIONS/ADR-0016-content-envelope-cryptography.md`
- `Docs/DECISIONS/ADR-0015-event-ledger-erasure-and-encrypted-replication.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md`
- `conductor/deferred.md` #50, #34
- crates.io / docs.rs: ed25519-dalek 3.0.0, x25519-dalek 3.0.0, curve25519-dalek 5.0.0, hkdf 0.13.0, hpke 0.14.0 (deferred)
- NIST SP 800-38D; NIST SP 800-88 Rev. 2; RFC 5869; RFC 9180 (considered)
