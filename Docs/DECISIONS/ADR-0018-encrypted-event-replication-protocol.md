# ADR-0018: Encrypted Event Envelope Replication Protocol (Untrusted Relay, Single-Owner)

## Status

**Accepted** — 2026-07-30.

Accepted after internal R2 CLEAN + Codex R1 FAIL→fix + Codex R2 FAIL→fix + **Codex R3 PASS** (security design review). Normative for P11 implementation tracks **T176–T178**. Complements [ADR-0015](ADR-0015-event-ledger-erasure-and-encrypted-replication.md) (replication **direction**) and [ADR-0016](ADR-0016-content-envelope-cryptography.md) (local content CE / DEK model) without changing capture independence or local-only default.

Companion threat model: [`conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md`](../../conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md).

## Context

ADR-0015 accepted that optional multi-device sync replicates **end-to-end encrypted event envelopes** through an **untrusted relay** and rebuilds projections locally — it does **not** synchronize mutable SQLite / SQLCipher files.

ADR-0016 froze local content-envelope cryptography: per-unit content DEK under vault `DataKey`, AES-256-GCM, CE = destroy DEK wrap + purge derived plaintext. Local CE is complete for a single vault; there is **no** multi-device ACK, no device identity, and **no shared DataKey** across devices.

P8 (T163–T165) shipped local CE only. Workspace crypto today includes `aes-gcm`, `zeroize`, `subtle`, `rand`, `sha2`, `argon2` — **not** ed25519 / x25519 / hkdf. Migrations end at `0026_content_envelopes_erasure.sql`. Two existing CLI surfaces already use the word “sync”:

1. `ai-brains sync *` — `Commands::Sync` — Ledgerful bridge  
2. `ai-brains safety sync` — `SafetyCommands::Sync` — hotspot pin into vault  

Before schema (T176), fake relay (T177), or security tests (T178), Task 11.0 requires freezing the **protocol threat model**, **normative locks L1–L16**, **per-recipient wrap construction**, **naming**, and **honest non-claims** (metadata, remote wipe, post-quantum).

### What we are not (v1)

| Pattern | Decision |
|---------|----------|
| SQLite / SQLCipher file replication | **Forbidden** |
| CRDT document store as default merge | **Not default** — event union + domain conflicts |
| MLS / OpenMLS (RFC 9420) | **Deferred** — revisit only for multi-**principal** groups |
| Matrix / Synapse / AGPL stacks | **Forbidden** as product deps |
| Multi-user vault sharing | **Out of scope** (L15) |
| Metadata-hiding / sealed-sender | **Not claimed** |
| Post-quantum hybrid KEM | **Not claimed** (L16) |
| Group / epoch KEK as primary wrap | **Rejected for v1 primary** |
| HPKE crate as v1 API | **Considered and deferred** (§18) |

### Standards and methods consulted (2026-07-30)

| Source | Takeaway |
|--------|----------|
| STRIDE (Microsoft SDL) | Boundaries: device↔vault, device↔relay, peer-via-relay |
| NIST SP 800-88 Rev. 2 | CE / key destroy; peer media not operator-controlled → no remote Purge/wipe claim |
| NIST SP 800-38D | GCM nonce uniqueness; multi-device wraps use fresh keys (O(1) seals) |
| RFC 9180 HPKE | Standard KEM+KDF+AEAD; evaluated, deferred for v1 explicit composition |
| RFC 9420 MLS | Multi-principal groups; deferred under L15 fence |
| Industry E2EE lessons | Fingerprint-bound enrollment; future exclusion on revoke; metadata honesty; PQ posture explicit |

Companion document: [`conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md`](../../conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md) (assets, actors, DFD, STRIDE, residuals, non-claims, **authoritative traceability matrix**).

## Decision

### 1. Architecture (L1)

```
Local vault (SQLCipher + event log + projections + CE side stores)
    │  seal/sign encrypted envelopes (client-side)
    ▼
Untrusted relay (opaque blob store + optional receive timestamps)
    │  push/pull by cursor; no decrypt capability
    ▼
Peer vault: enrolled-set check → verify sig → decrypt → append → project
```

- Local ledger is authoritative per device; the relay is **not** system of record.
- Capture, FTS, briefings, and local CE work with multi-device replication **off**.
- Unit of replication = **encrypted event envelope** + control envelopes — not DB pages, not CRDT docs.

### 2. Device identity (L2)

- Stable **`DeviceId`** (UUID) + long-term **Ed25519** signing key + **X25519** static public for recipient wraps.
- Relay stores **public** material and opaque envelopes only — never private keys, vault `DataKey`, or content DEKs.
- Device private key storage: **recommend** wrap under vault `DataKey` + OS DPAPI for the vault unlock path; exact APIs and schema in **T176** (open Q4 resolved as direction).

### 3. Enrollment (L3)

**Enrollment package (canonical identity blob, normative):**

```
enrollment_package =
    schema_version (u16 BE)
  ‖ DeviceId (16-byte UUID)
  ‖ Ed25519_pub (32 bytes)
  ‖ X25519_pub (32 bytes)
```

Any additional frozen identity fields, if added later, require an ADR amendment and a new `schema_version`. Field order is fixed; implementers MUST NOT reorder or omit X25519.

**OOB fingerprint (prefer single approach):**

- `fingerprint = SHA-256(enrollment_package)` — hash of the **full dual-key identity blob**, not Ed25519 alone.
- Human-readable / QR encoding of the fingerprint (or of the package with fingerprint display) is a UX detail; the **binding** is the hash of both public keys + DeviceId + schema_version.
- Owner confirms fingerprint match on an **already-enrolled** device, or via **recovery-kit-bound first-device bootstrap** (§22 Q5).

**Rules:**

- **Bearer pairing codes without key binding are insufficient** (MITM-vulnerable).
- Unverified publics **must not** receive content-DEK wraps.
- **X25519 bind:** if the X25519 public presented at wrap time does not match the enrolled package (or OOB fingerprint was computed without both keys), treat as **mismatch → reject / no wrap**. A MITM that swaps only X25519 after Ed25519-only OOB must not receive DEK wraps (`T178-L3-enroll-binds-x25519`).
- **Signer of `DeviceEnrolled`:** after OOB confirm, **`DeviceEnrolled` is signed by an already-enrolled device** (not by the new device alone). Self-enroll or enroll signed only by a new / unknown `device_id` → **reject at L8 gate** (`T178-L3-enroll-signer-must-be-enrolled`).
- **First-device bootstrap:** local-only, **RecoveryKit-bound**; **no** untrusted-relay self-enroll of an unknown `device_id`. The first device is enrolled offline into the local vault; subsequent devices use fingerprint OOB against an enrolled peer.
- Enrollment emits a **signed** `DeviceEnrolled` (or equivalent) control record as a **special signed control envelope type on the same stream** (§22 Q1).

**Re-enrollment after revoke (L3 + L4) — permanent DeviceId retirement:**

- After `DeviceRevoked` is applied, that **`DeviceId` is permanently retired (tombstoned)**. It MUST NOT be re-enrolled, re-issued, or reused under any policy (including full OOB).
- Re-enrolling the **same physical machine** after revoke always requires a **new `DeviceId`**, **new key material** (new Ed25519 + X25519 pair), and a **full dual-key OOB fingerprint confirm** against an already-enrolled device (no residual trust from the retired id).
- Peers MUST reject any `DeviceEnrolled` (or other control) that attempts to resurrect a tombstoned `DeviceId`.
- Rationale: avoids tombstone/past-key confusion, stale-envelope races against a “revoked-then-reused” id, and generation/epoch bookkeeping for the same UUID.

### 4. Revocation (L4)

- Owner issues signed `DeviceRevoked` via an **already-enrolled, non-revoked device** (same signer class as `DeviceEnrolled`; not self-revoke by the revoked id as the sole authority for removing itself from peers’ enrolled sets without an enrolled peer’s signature).
- **Future exclusion:** stop creating per-recipient wrap rows for that `device_id`; reject post-revoke envelopes from that id after revoke is applied.
- **DeviceId retirement:** the revoked `DeviceId` is **permanently tombstoned** — never reused (see L3 re-enrollment). Same physical machine → new DeviceId + new keys + full OOB.
- **Residual:** past DEKs already held on a stolen device remain openable.
- **Not v1:** mandatory group epoch KEK ratchet (optional future optimization if N grows).

### 5. Envelope model and signature scope (L5)

- Unit = outer transport envelope: **signed outer fields** + **detached Ed25519 signature** + opaque body field (historically named `ciphertext` on the wire) + **N per-recipient wrap records** (data only; control has `N = 0`).
- **No plaintext event bodies** on the relay or in replication tables. **Control payloads** are cleartext **membership/integrity signaling** under outer signature (not content secrecy) — see §5.1.
- Idempotent apply by outer `event_id` (and `envelope_id` for transport).
- At-least-once delivery → exactly-once apply by id.

#### 5.1 Outer vs inner (normative freeze)

| Layer | Contents | Authenticated how |
|-------|----------|-------------------|
| **Outer signed metadata** | Routing / apply fields that travel **outside** the body blob: ids, seq, type, `content_key_id`, wrap list, body bytes | Ed25519 over `signed_bytes` (§5.2) |
| **Body field (`ciphertext` wire offset)** | **Data:** AEAD blob under content DEK (§5.3). **Control:** cleartext control payload (not DEK-encrypted). | Covered by outer sig as opaque bytes; data also has AEAD integrity under DEK |
| **Wrap records (N)** | Per enrolled peer (data only): `recipient_device_id`, `eph_x25519_pub`, `wrap_nonce`, `wrap_ct` (§17) | **Outer signed metadata** (v1 wire freeze) — not separate control envelopes |
| **Detached signature** | Ed25519 signature field | **Not** included in `signed_bytes` |

**Rule:** Any field that travels on the wire **outside** AEAD ciphertext (data path) and is used for routing/apply **MUST** be in `signed_bytes`. Metadata swap under a signature that covered only the body blob is **forbidden** (mirror ADR-0016 AAD discipline) — verifiers **fail closed**.

**v1 wrap placement (single-owner, untrusted relay):** wrap material is **outer signed metadata** on the **data** content envelope (not embedded-only inside content AEAD, and not separate wrap-control envelopes). Peers that cannot open their wrap still verify the outer signature and reject tampered wrap lists. Local vault may project wraps into a table keyed by `(content_key_id, recipient_device_id)` after verify (T176 schema intent §17.4).

##### 5.1.1 Control vs data envelopes (normative freeze — public control)

**Control set** (`content_type_code` values for pure control; exact numeric codes → T176 schema, labels normative here):

| Control type | Role |
|--------------|------|
| `DeviceEnrolled` | Membership add (publics + DeviceId) |
| `DeviceRevoked` | Membership remove / permanent DeviceId retirement |
| `ContentErasureTombstone` / `KeyTombstone` | Erasure signaling for a target `content_key_id` |
| `ErasureAck` | Peer attestation of local apply/CE steps for an erasure |
| *(any later pure-control code)* | Same rules unless ADR amendment |

**Control envelope rules:**

1. Outer envelope still uses full `signed_bytes` authenticity (**L5**): enrolled-set gate → verify Ed25519 over §5.2 → then parse body.
2. The wire field historically called `ciphertext` is a **cleartext control payload** when `content_type_code` is in the control set. **Prose name:** `payload` (control) vs `ciphertext` (data). **Wire offset/field name may stay** `ciphertext` / `ciphertext_len` for a single layout; decode path branches on `content_type_code`.
3. **Control payloads are NOT encrypted under a content DEK.** There is no shared `DataKey`, no control-seal DEK, and no per-recipient wrap for pure control.
4. `wrap_count = 0` **always** for pure control envelopes (no wrap records).
5. `content_key_id` is the **all-zero UUID** except erasure/ACK types, where it identifies the **target content key** being erased/acked (public among enrolled devices — not a secret).
6. Peers: verify signature + enrolled-set (L8) → parse cleartext control body → apply control semantics. No AEAD open / DEK unwrap on the control path.

**Why public control is OK (honest threat rationale):**

- Enrollment **public keys** and `DeviceId` must be public among peers for fingerprint OOB and per-recipient wraps.
- Revoke / erasure-tombstone / ACK **metadata** is membership and integrity **signaling**, not content-body secrecy. Secrecy of past event bodies remains on the data path (per-unit DEK + wraps).
- **Authenticity** is via Ed25519 under the enrolled signing key + enrolled-set gate — not via confidentiality of the control body.
- Encrypting control with `N = 0` and no shared control key is **not implementable** (no recipient can open it). Public signed control is the intentional freeze.

**Data event envelopes:**

- Body field = **AEAD ciphertext blob** under the content DEK (§5.3 packing).
- `wrap_count ≥ 1` for each intended peer (typically **N wraps** for all enrolled non-revoked peers excluding self, per product policy).
- `content_key_id` = non-zero id of the content DEK unit.

#### 5.2 Canonical `signed_bytes` (complete; exhaustive)

Encoding: length-prefixed / fixed-width concat; all multi-byte integers **big-endian**; UUIDs as **16 raw bytes** (RFC 4122 layout). **No** JSON, **no** UUID string forms, **no** little-endian integers, **no** reordering.

```
signed_bytes =
    schema_version      : u16 BE
  ‖ envelope_id         : 16 bytes UUID
  ‖ device_id           : 16 bytes UUID   # sender / signer device
  ‖ local_seq           : u64 BE
  ‖ content_type_code   : u16 BE
  ‖ event_id            : 16 bytes UUID   # outer signed apply id (idempotent apply)
  ‖ content_key_id      : 16 bytes UUID   # zero UUID for most control; target key for erasure/ACK
  ‖ ciphertext_len      : u32 BE          # length of body field below
  ‖ ciphertext          : opaque bytes    # data: nonce‖ct‖tag (§5.3); control: cleartext payload
  ‖ wrap_count          : u32 BE          # N; MUST be 0 for pure control
  ‖ wrap_records[0..N)  : each as below, in canonical order
```

**Body field interpretation by `content_type_code`:**

| Kind | `ciphertext` bytes mean | `wrap_count` | `content_key_id` |
|------|-------------------------|--------------|------------------|
| **Data** | `nonce (12) ‖ ciphertext ‖ tag (16)` under content DEK (§5.3) | ≥ 1 (intended peers) | non-zero content key |
| **Control** | cleartext control payload (prose: `payload`) | **0** | zero UUID, except erasure/ACK → target key |

**Each wrap record** (fixed order inside the record; data only):

```
wrap_record_i =
    recipient_device_id : 16 bytes UUID
  ‖ eph_x25519_pub      : 32 bytes
  ‖ wrap_nonce          : 12 bytes
  ‖ wrap_ct_len         : u32 BE
  ‖ wrap_ct             : opaque bytes    # AES-GCM wrap of content DEK (§17); NO plaintext DEK
```

**Canonical wrap list order:** sort wrap records by `recipient_device_id` ascending (unsigned 16-byte lexicographic / big-endian UUID byte order). Implementers MUST NOT sign an unsorted list; verifiers MUST reject if presented order ≠ sorted order **or** re-canonicalize only if the ADR is amended — **v1: reject unsorted** (fail closed; simpler KATs).

**Signature:**

```
signature = Ed25519.sign(device_signing_key, signed_bytes)
```

Detached; the signature field is **not** part of `signed_bytes`. Verify with the enrolled Ed25519 public for `device_id` after the L8 enrolled-set gate.

**Not in `signed_bytes`:** detached signature itself; local-only projection state; relay receive timestamps (if any); transport framing outside this envelope.

**Inner data payload plaintext** (event body fields beyond the outer `event_id`, domain dependencies, etc.) is authenticated by AEAD under the content DEK after unwrap — not by duplicating every inner field into the outer signature. Outer `event_id` remains the **idempotent apply id** on the wire. **Control body** is authenticated only by the outer Ed25519 (no inner AEAD).

#### 5.3 Content AEAD nonce packing (data envelopes only — normative freeze)

For **data** envelopes, the outer body field (`ciphertext` wire offset) is a **single opaque blob**:

```
content_aead_blob =
    nonce       : 12 bytes   # AES-256-GCM nonce, OS CSPRNG, fresh per seal
  ‖ ciphertext  : variable   # AES-256-GCM ciphertext of (padded) event plaintext
  ‖ tag         : 16 bytes   # AES-256-GCM authentication tag
```

- `ciphertext_len` covers the **full blob** including nonce + ciphertext + tag.
- The nonce is thus **covered by the outer signature** (inside the opaque field) and is used for AEAD open **after** DEK unwrap.
- Implementations **MUST NOT** put a separate **unsigned** nonce beside the blob on the wire (no parallel unsigned nonce field).
- **Local store may separate nonce columns** (matches ADR-0016 / `ai-brains-crypto` `SealedContent` where nonce is a distinct field from ct‖tag). Encode/decode **converts** at the wire boundary: local `(nonce, ct‖tag)` ↔ wire `nonce ‖ ct ‖ tag`.
- Packing intent mirrors common envelope packing and ADR-0016 content seal (nonce + ciphertext+tag); only the **wire concatenation** is frozen here for multi-device interop.

**Control envelopes:** no content AEAD; body is cleartext control payload; this section does not apply.

#### 5.4 T178 outer-signature / control / nonce tests (required)

| Id | Intent |
|----|--------|
| **`T178-L5-sig-canonical-bytes`** | KAT: fixed fixture keys/ids → exact `signed_bytes` hex + valid Ed25519 over that string |
| **`T178-L5-meta-swap-fails`** | Swap any single outer field in `signed_bytes` (id, seq, type, event_id, content_key_id, body bytes) under same signature → verify fail (**fail closed**) |
| **`T178-L5-wrap-list-tamper`** | Modify wrap list (drop/reorder/alter eph_pub/nonce/wrap_ct/recipient) under same signature → verify fail |
| **`T178-L5-content-nonce-in-blob`** | KAT: data envelope body = `nonce(12)‖ct‖tag(16)`; open succeeds only with that packing; separate unsigned nonce path forbidden |
| **`T178-L5-control-cleartext-parse`** | Control envelope: `wrap_count=0`, body is cleartext control payload, no DEK unwrap; peer parses after sig+enrolled-set verify |
| `T178-L5-tamper-ct` | Bit-flip body under same sig → verify fail (may share fixture with meta-swap) |
| `T178-L5-replay-idempotent` | Replay same envelope/`event_id` → no-op success |
| **`T178-L7-ack-cleartext-signed`** | ErasureAck is cleartext signed control (`N=0`); verifies as L7 ACK authenticity (sender attestation only — see residual) |

Legacy label `T178-L5-sig-meta` maps to **`T178-L5-meta-swap-fails`** (prefer the new id in T178).

### 6. Ordering and conflicts (L6)

- Monotonic per-device **`local_seq`** is required.
- **Apply order:** topological by declared event dependencies (parent / correlation ids already in domain events); tie-break `(device_id, local_seq, event_id)`.
- Projectors handle missing parent via existing staleness machinery — not silent drop of child, not silent LWW.
- HLC is **optional display / soft sort only** — never apply SOV.
- Concurrent epistemic contradictions → **explicit conflict** (ADR-0014). **Never last-write-wins.**

### 7. Erasure propagation and ACK (L7)

Absorbs deferred **#34 (1)** design:

1. Erasing device emits a **signed** key-tombstone / erasure control envelope (cleartext control payload; §5.1.1).
2. Peers apply: destroy local DEK wrap for that `ContentKeyId` + purge derived rows + tombstone (ADR-0016 local CE).
3. Peers send **signed ErasureAck** control envelopes back through the relay (cleartext signed control; `wrap_count = 0`; `T178-L7-ack-cleartext-signed`).
4. Erasing device holds a **local** ACK projection: `(erasure_id, peer_device_id) → pending | acked | failed | unreachable` (timeout after N sync cycles; default **N = 3**, tunable at T176).
5. Relay **cannot** forge ACKs; forged ACK fails signature (T178). A valid signature proves **sender authenticity**, not remote wipe completeness (see residual below).
6. UX: honest **partial** multi-device erase — no “erased everywhere” without policy quorum / all-acked. **MUST NOT** treat a single ACK as “cryptographically proven wiped everywhere.”
7. Offline peer residual: decryptable until peer syncs — **best-effort propagation**, not remote media sanitization.

**ACK authenticity residual (normative honesty):**

- A signed `ErasureAck` proves that **an enrolled device attested** that it performed the local apply / CE steps named by the protocol — it does **not** prove remote media sanitization, disk destruction, or that the peer is malware-free.
- A **compromised enrolled device** can emit a **valid false ACK** (correct signature, lying body) until that device is revoked.
- Projection states remain honest enums: `pending | acked | failed | unreachable`. Product UX and claims language must treat `acked` as **peer attestation received**, not as cryptographic proof of wipe on all media.
- Non-claims reinforce: not perfect multi-device deletion; not NIST Purge / remote wipe (L11 + §24).

### 8. Replay, tamper, enrolled-set gate (L8)

- AEAD **fail-closed**; signature **fail-closed**; outer metadata swap under a valid body **fail-closed** (verify rejects).
- Replay of same envelope/event id = no-op success.
- **Before signature verification:** check `device_id` ∈ enrolled **and** not revoked; unknown id → reject **without** expensive crypto work (DoS mitigation).
- **Enrollment/revocation control apply:** `DeviceEnrolled` / `DeviceRevoked` whose **signer** is not already enrolled (or is revoked) → reject at this gate (L3/L4). Exception: first-device bootstrap is **local-only** / RecoveryKit-bound and never arrives as untrusted-relay self-enroll of an unknown id.
- T178 must inject unknown `device_id`, metadata-swapped envelope, forged ACK, X25519-swapped enroll package, and self-enroll-from-unknown.

### 9. Relay adversary model (L9)

| Capability | Allowed | Outcome |
|------------|---------|---------|
| Store, drop, reorder, duplicate | Yes | No plaintext |
| Observe sizes, times, device graph | Yes | Residual metadata |
| Inject unknown `device_id` envelopes | Attempt | Reject pre-verify |
| Forge valid device signatures / ACKs | No | Reject |
| Modify ciphertext or signed metadata undetected | No | Reject |
| Read DataKey / content DEKs | No | Design fail if possible |

### 10. Optionality and naming (L10)

- Default: **local-only**; multi-device replication off.
- **Do not repurpose** either existing surface:
  1. `ai-brains sync *` (`Commands::Sync`) — Ledgerful bridge  
  2. `ai-brains safety sync` (`SafetyCommands::Sync`) — hotspot pin  
- **v1 multi-device CLI freeze:**
  - **`ai-brains device`** — enroll / list / revoke / show fingerprint  
  - **`ai-brains replicate`** — push / pull / status / cursors  
- Crate name **`ai-brains-sync`** is OK (library name ≠ CLI `sync`).

### 11. Disaster recovery and CE honesty (L11)

- RecoveryKit restores **DataKey** only; it must not resurrect destroyed content DEKs.
- Pre-erase backup residual unchanged from ADR-0016.
- Multi-device CE = **best-effort key-destruction propagation**, not NIST remote sanitization / remote wipe.
- **NIST SP 800-88r2:** scope is media under the **operator’s control**. Peer/stolen device storage is not operator-controlled media. Do **not** market multi-device CE as Purge, Destroy, or “remote wipe.”

### 12. Capture independence and licenses (L12)

- No `ai-brains-capture` → sync dependency. Capture path remains functional without models, embeddings, graph, or sync.
- Project license: PolyForm Noncommercial 1.0.0 + Small-Entity Commercial Exception; optional sync so commercial local-only needs no relay.
- Named primitives only from deny-allowlisted licenses (MIT, Apache-2.0, BSD-3-Clause, ISC; MPL if already policy). **Forbidden:** AGPL/GPL sync engines, hand-rolled ECC/AES, unknown-git crypto, raw X25519 shared secret as AES key without KDF.

### 13. Sequence gap detection (L13)

- Per peer stream: track `expected_local_seq` / high-water.
- Gap → `sync_gap` state: buffer out-of-order envelopes for that device; request missing seq range from relay (or wait); **do not** corrupt projections by applying past-gap without policy.
- **Gap skip policy (open Q6 resolved):** **fail-closed default**; operator explicit skip only with a **signed audit event**. Exact UX/schema detail → T176.

### 14. Payload length padding (L14)

- Before envelope seal, pad plaintext to **fixed buckets** (e.g. 256 B / 4 KiB / 64 KiB) to reduce event-type inference from size.
- Does **not** eliminate metadata leakage (timing, counts, graph, bucket itself).
- Non-claim “metadata-private” still stands; padding is best-effort hardening.

### 15. Single-owner / single-vault membership (L15)

- **v1 sync = one human principal, one vault membership group, N personal devices.**
- Multi-user vault sharing, multi-tenant, deniable auth, multi-principal group messaging are **out of scope** and require a **new ADR** (likely MLS-class) if product needs them.
- Makes MLS deferral coherent.

### 16. Post-quantum non-claim (L16)

- v1 uses **classical ECC only** (Ed25519 / X25519).
- **Not post-quantum resistant.** Ciphertext retained by a future quantum-capable adversary who also obtains relay-stored blobs is a **residual** (harvest-now-decrypt-later). Not mitigated in P11 v1.

---

### 17. Per-recipient content-DEK wrap construction (normative — frozen)

Local wrap (ADR-0016) remains **DataKey → AES-GCM → content DEK** on each device. There is **no shared DataKey** across devices.

**v1 freeze: per-recipient sealed wrap (not group epoch KEK):**

For each content unit and each **enrolled, non-revoked** peer device (recipient X25519 **must** match the enrolled package — L3):

1. Generate ephemeral X25519 keypair; `shared = X25519(eph_priv, peer_static_x25519)`.
2. Derive wrap key via **HKDF-SHA256** with the **byte-exact** encoding below.
3. AES-256-GCM-wrap the content DEK under `wrap_key` with the **byte-exact AAD** below.
4. Persist **N wrap rows** (one per recipient) with ephemeral public, nonce, ciphertext+tag — **no plaintext DEK**.
5. **Revocation** = stop creating wrap rows for that `device_id` (and reject post-revoke enroll). No group re-key ratchet required for v1.

#### 17.1 Length-prefix encoding (shared primitive)

Unless a field has a **fixed** width stated below, multi-byte fields use:

```
len_u16_be(field) ‖ field_bytes
```

where `len_u16_be` is the field byte length as **u16 big-endian** (2 bytes). Fixed-width fields are concatenated **without** a length prefix.

| Symbol | Encoding |
|--------|----------|
| `schema_version` | **u16 BE** (2 bytes), fixed |
| `label` UTF-8 | `u16 BE length` ‖ UTF-8 bytes (not null-terminated) |
| `content_key_id` | **16 bytes** UUID, fixed, no length prefix |
| `recipient_device_id` | **16 bytes** UUID, fixed, no length prefix |
| `sender_device_id` | **16 bytes** UUID, fixed, no length prefix |

Implementers MUST NOT invent alternative orderings, little-endian integers, UUID string forms, or omitted length prefixes. Wrong encoding → wrong keys / AAD → **interop failure by design** (detectable via T178 WRAP KATs).

#### 17.2 HKDF-SHA256 (normative bytes)

| Parameter | Value |
|-----------|--------|
| Hash | SHA-256 |
| **salt** | **empty** (zero-length byte string) |
| **IKM** | `shared` (32-byte X25519 shared secret) |
| **info** | length-prefixed concatenation in **exact order** below |
| Output | 32 bytes → AES-256-GCM key |

**`info` construction (exact order):**

```
info =
    schema_version          (u16 BE)
  ‖ len_u16_be(label) ‖ label
  ‖ content_key_id          (16 bytes)
  ‖ recipient_device_id     (16 bytes)
  ‖ sender_device_id        (16 bytes)
```

where `label` = UTF-8 bytes of the exact ASCII string **`aib-sync-dek-wrap`** (no trailing NUL).

```
label = "aib-sync-dek-wrap"   # 17 bytes; len_u16_be = 0x00 0x11
```

**Pseudocode:**

```text
wrap_key = HKDF-SHA256(
  salt = [],
  ikm  = shared,
  info = schema_version ‖ u16be(len(label)) ‖ label
         ‖ content_key_id ‖ recipient_device_id ‖ sender_device_id,
  L    = 32
)
```

#### 17.3 AES-GCM wrap AAD (normative bytes)

AAD for the per-recipient DEK wrap (mirror ADR-0016 AAD discipline: bind identity metadata into AEAD so metadata swap **fails closed** — open/verify rejects on AAD mismatch):

```
aad =
    schema_version          (u16 BE)
  ‖ content_key_id          (16 bytes)
  ‖ recipient_device_id     (16 bytes)
```

Fixed-width fields only; **no** length prefixes on the UUIDs; **no** `sender_device_id` in AAD (sender is bound via HKDF `info` and the outer envelope signature). Secrets never appear in AAD.

Nonce: 96-bit (12-byte) random from OS CSPRNG, fresh per wrap AEAD invocation (same discipline as ADR-0016 §4).

#### 17.4 T178 WRAP KATs

T178 **must** ship known-answer tests that fix `schema_version`, ids, label bytes, empty salt, and expected `info` / `aad` byte strings and wrap ciphertext for a fixed ephemeral/static key fixture. Any implementer divergence from §17.1–17.3 fails those KATs (`T178-WRAP-kat-info-aad-bytes` and related).

**T176 schema intent:** wrap table keyed by `(content_key_id, recipient_device_id)`.

| Option | Why not v1 primary |
|--------|--------------------|
| Group / epoch KEK (HKDF advance on revoke) | MLS-adjacent complexity; schema + rekey race — **future opt** if N grows |
| Raw X25519 output as AES key | Cryptographically wrong |
| Shared vault DataKey over relay | Breaks E2E / device isolation |

**Multi-device wrap nonce budget:** per-recipient wrap keys are **per seal** (ephemeral ECDH → fresh wrap_key) or short-lived; message count per wrap_key is **O(1)**. Strictly better than ADR-0016 vault-lifetime DataKey budget. No extra multi-device nonce ledger required for v1 beyond the ADR-0016 DataKey residual.

### 18. HPKE (RFC 9180) — considered and deferred

**Considered:** `hpke` ~0.14 (MIT OR Apache-2.0) standardizes KEM+KDF+AEAD for “encrypt to recipient public key” and reduces hand-composition footguns.

**v1 decision: defer HPKE crate.** Implement explicit **X25519 + HKDF-SHA256 + AES-256-GCM** with **frozen info/AAD labels** so the construction is auditable line-by-line and reuses already-named workspace-adjacent crates. HPKE remains a **hygiene candidate** if security review or interop demands a single standard API (same suite: X25519 / HKDF-SHA256 / AES-256-GCM).

### 19. DataKey rotation (direction only)

- **Direction:** operational DataKey rotation (re-wrap living content DEKs under a new DataKey, and/or counter/nonce ledger for the vault KEK layer) remains a documented hygiene need from ADR-0016 §4.
- Multi-device replication does **not** close that gap; per-recipient wraps improve the *multi-device* nonce story only.
- **Implementation residual remains open** (P11 hygiene / deferred #34.2). This ADR freezes **direction**, not code or schema for rotation.

### 20. Migration and implementation order

| Item | Freeze |
|------|--------|
| Next free migration for replication | **`0027+`** (e.g. `0027_replication_state.sql`) — **not** `0026` (already `0026_content_envelopes_erasure.sql`) |
| Crate | `crates/ai-brains-sync` created in **T176** — **not** in T175 |
| Fake relay first | **T177** proves convergence on in-memory/file relay **before** any real network relay |
| Security tests | **T178** implements the traceability matrix |
| Network production relay | After T177; not part of T175–T176 |

### 21. Candidate dependencies (named only — zero Cargo changes in T175)

**Existing (reuse):**

| Crate | Workspace | License | Role |
|-------|-----------|---------|------|
| `aes-gcm` | 0.10.x | Apache-2.0 OR MIT | AEAD (content + wrap) |
| `zeroize` | 1.8.x | Apache-2.0 OR MIT | Key wipe |
| `subtle` | 2.6 | BSD-3-Clause | Constant-time |
| `rand` | 0.10.x | MIT OR Apache-2.0 | Entropy |
| `sha2` | 0.11.x | MIT OR Apache-2.0 | Fingerprints / HKDF hash |

**Candidates for T176+** (deny/audit at implement; include **transitives**):

| Crate | Pin | License | Role |
|-------|-----|---------|------|
| `ed25519-dalek` | **3.x** | BSD-3-Clause | Device signing |
| `x25519-dalek` | **3.x** | BSD-3-Clause | ECDH (static + ephemeral) |
| `curve25519-dalek` | **5.x** (transitive) | BSD-3-Clause | Curve arithmetic — inventory required |
| `hkdf` | **0.13.x** | MIT OR Apache-2.0 | KDF for wrap keys |
| `openmls` | 0.8.x | MIT | **Not v1** |
| `hpke` | 0.14.x | MIT OR Apache-2.0 | **Considered; deferred v1** |

**Intended feature flags (T176):**

```toml
ed25519-dalek = { version = "3", features = ["serde", "zeroize", "rand_core"] }
x25519-dalek  = { version = "3", features = ["serde", "zeroize", "static_secrets"] }
# confirm static_secrets need at implement; prefer ephemeral-for-wrap
```

- **`zeroize` feature mandatory** (AGENTS.md key material / ZeroizeOnDrop).
- **`serde`** for durable public device records.
- **`rand_core`** for `SigningKey::generate` / ephemeral ECDH.
- API note: dalek 3.x uses `SigningKey` / `VerifyingKey` (not 2.x `Keypair`).
- Exact patch pins via Cargo.lock at T176; `cargo deny check` + `cargo audit` including curve25519-dalek 5.x transitive.

### 22. Open questions — resolved defaults

| # | Question | Resolution |
|---|----------|------------|
| 1 | Control records: special event types vs separate stream? | **Recommend special signed control envelope types on the same stream** (`DeviceEnrolled`, `DeviceRevoked`, erasure tombstone, `ErasureAck`) with distinct `content_type_code` values. Same signature/gap/idempotency machinery. Separate control stream only if T176 proves filter/perf need — then ADR amendment. |
| 2 | Selective sync? | **Default whole-vault v1.** Project/scope filters deferred; not a silent partial default. |
| 3 | ACK timeout N? | **Default N = 3 sync cycles** then `unreachable` (or `failed` per policy); honest partial CE UX. Exact N tunable at T176 without ADR supersession if still “small constant.” |
| 4 | Device private key storage? | **Recommend wrap under DataKey + OS DPAPI** on vault unlock path. Exact T176. |
| 5 | First-device bootstrap? | **Recovery-kit-bound first device**, **local-only** (no untrusted-relay self-enroll of unknown `device_id`). Subsequent devices: dual-key fingerprint OOB against an already-enrolled device; `DeviceEnrolled` signed by that enrolled device (L3). |
| 6 | Gap skip if seq permanently lost? | **Fail-closed default**; operator explicit skip only with **signed audit event**. Detail T176. |

Resolved earlier by fold-in (no longer open): per-recipient wrap; HPKE deferred; HLC display-only; fingerprint enrollment; single-owner; signature covers metadata+ciphertext; CLI `device`/`replicate`; PQ non-claim; dual sync CLI collisions.

### 23. Traceability matrix

**Authoritative matrix:** threat-model §7 — each L1–L16 + residual + non-claim → proposed T178 test id or explicit `defer: …`.

Summary (non-authoritative quick index):

| Class | Examples |
|-------|----------|
| Crypto integrity | `T178-L5-sig-canonical-bytes`, `T178-L5-meta-swap-fails`, `T178-L5-wrap-list-tamper`, `T178-L5-content-nonce-in-blob`, `T178-L5-control-cleartext-parse`, `T178-L5-tamper-ct`, `T178-L8-unknown-device-preverify` |
| Enrollment | `T178-L3-enroll-fingerprint`, `T178-L3-enroll-binds-x25519`, `T178-L3-enroll-signer-must-be-enrolled` |
| Wrap | `T178-WRAP-per-recipient-roundtrip`, `T178-WRAP-wrong-recipient-fails`, `T178-WRAP-kat-info-aad-bytes` |
| Erasure | `T178-L7-ack-signed`, `T178-L7-ack-cleartext-signed`, `T178-L7-forged-ack-reject` |
| Ordering | `T178-L6-no-lww-conflict`, `T178-L13-gap-buffer` |
| Docs / claims | `T178-NC-no-purge-claim`, `T178-NC-no-pq-claim`; L10/L15/L16 mostly defer to T176/T185 |

### 24. Non-claims (normative product language)

Must **not** claim: metadata-private; perfect multi-device deletion; NIST Purge/Destroy/remote wipe; compliance certifications; post-quantum resistance; multi-user vault; relay as SOV; LWW auto-merge; capture dependency on sync; **single signed ACK = cryptographically proven wiped everywhere** (ACK is peer attestation only — L7 residual).

## Consequences

### Positive

- Clear protocol fence before any network or crate work (reduces rewrite risk).
- Per-recipient wrap matches ADR-0016 per-unit DEK granularity and makes revoke trivial (omit recipient).
- Dual CLI collision recorded so implementers do not overwrite Ledgerful or safety surfaces.
- Honest residuals (metadata, offline CE lag, PQ, past keys) prevent false marketing.
- Fake-relay-first (T177) keeps test hygiene and avoids premature network surface.

### Negative / residual risks

- Metadata still leaks to the relay (sizes, graph, timing) even with L14 padding.
- Offline / stolen devices retain past decrypt capability until sync or forever for past keys.
- Hand-composed X25519+HKDF+AES-GCM requires careful label/AAD discipline (mitigated by freeze + T178 KATs; HPKE remains hygiene path).
- DataKey rotation **implementation** still open (#34.2).
- Classical-only crypto → harvest-now-decrypt-later residual (L16).
- Single-owner fence may block future multi-user product until a new ADR.
- Signed ErasureAck is **self-attestation** only; compromised enrolled peer can false-ACK until revoke (L7 residual — not a wipe proof).

### Follow-on tracks

| Track | Work | Blocked until |
|-------|------|---------------|
| **T176** | `ai-brains-sync` crate; migration `0027+`; device/replicate CLI; wrap table; key storage | **Unblocked** (ADR-0018 Accepted) — not yet implemented |
| **T177** | Fake relay; two-client converge/reorder/retry | T176 types (design unblocked by T175 Complete) |
| **T178** | Security tests per threat-model §7 matrix | T176–T177 (design unblocked by T175 Complete) |
| P11 hygiene | DataKey rotation implementation | Explicit residual; not closed by this ADR |
| Future ADR | Multi-principal / MLS if product needs multi-user | L15 fence |

## Test plan outline (downstream T178)

| Class | Example ids |
|-------|-------------|
| Opacity / no relay decrypt | `T178-L9-relay-no-decrypt` |
| Outer sig KAT / meta / wrap / nonce / control | `T178-L5-sig-canonical-bytes`, `T178-L5-meta-swap-fails`, `T178-L5-wrap-list-tamper`, `T178-L5-content-nonce-in-blob`, `T178-L5-control-cleartext-parse`, `T178-L5-tamper-ct` |
| Replay idempotent | `T178-L5-replay-idempotent` |
| Unknown device pre-verify | `T178-L8-unknown-device-preverify` |
| Enroll X25519 bind / signer | `T178-L3-enroll-binds-x25519`; `T178-L3-enroll-signer-must-be-enrolled` |
| Revocation future exclusion + DeviceId retirement | `T178-L4-revoke-no-future-wrap`; `T178-L4-revoke-signer-must-be-enrolled`; `T178-L4-deviceid-permanently-retired` |
| Wrap info/AAD KATs | `T178-WRAP-kat-info-aad-bytes` |
| Forged ACK / cleartext ACK | `T178-L7-forged-ack-reject`; `T178-L7-ack-cleartext-signed` |
| Gap handling | `T178-L13-gap-buffer` |
| Capture without sync | `T178-L12-capture-without-sync` |
| Non-claims documented | `T178-NC-*` |

Full authoritative map: threat-model §7.

## License and dependency gate

- **T175 adds zero crates** and does not modify `Cargo.toml` / `Cargo.lock`.
- T176 must run `cargo deny check` and `cargo audit` including **curve25519-dalek 5.x** transitive.
- Allowed license classes only: MIT, Apache-2.0, BSD-3-Clause, ISC, (MPL if already policy).
- **Forbidden:** AGPL/GPL, unknown-git crypto, hand-rolled ECC/AES.

## References

- [ADR-0015](ADR-0015-event-ledger-erasure-and-encrypted-replication.md) — ledger + CE direction + encrypted replication vision  
- [ADR-0016](ADR-0016-content-envelope-cryptography.md) — content DEK / local CE  
- [ADR-0014](ADR-0014-source-aligned-freshness-and-explicit-conflict.md) — explicit conflict (no LWW)  
- [ADR-0011](ADR-0011-separate-evidence-conclusions-decisions.md) — epistemic separation  
- Threat model: `conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md`  
- Spec: `conductor/tracks/trackT175-sync-threat-model-adr/spec.md`  
- Vision §11–12 — `Docs/MEMORY-CONTROL-PLANE-VISION.md`  
- Master plan Phase 11 — Task 11.0–11.3  
- NIST SP 800-88 Rev. 2; NIST SP 800-38D  
- RFC 9180 HPKE (deferred); RFC 9420 MLS (deferred)  
- Live CLI: `Commands::Sync` (`main.rs` ~264), `SafetyCommands::Sync` (`main.rs` ~1217)  
- Deferred #34 — partial promote (ACK design vs DataKey rotation residual)

## Review checklist

- [x] Internal security review of threat model + this ADR vs T175 spec (internal R2 CLEAN; Codex R1 FAIL→fix; Codex R2 FAIL→fix; **Codex R3 PASS**)  
- [x] Open questions §22 accepted or amended (safe defaults normative on Accept)  
- [x] On accept: set Status → **Accepted** with date; unblock T176  
- [ ] Optional pin: `ai-brains pin "DECISION: ADR-0018 …"` (orchestrator / session optional)  
- [x] No production network/sync code in T175  
- [x] Traceability matrix present (threat-model §7)  
