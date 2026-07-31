# T175 — Sync Protocol Threat Model + ADR (P11.0)

- **Track ID:** T175-SyncThreatModelAdr
- **Phase:** P11 — Encrypted event replication (**post-MVP**) (Task 11.0)
- **Status:** ✅ **Completed** (2026-07-30) — design + threat model + **ADR-0018 Accepted** after internal R2 CLEAN + Codex R1 FAIL→fix + Codex R2 FAIL→fix + **Codex R3 PASS**; zero crates; unblocks T176–T178
- **Depends on:**
  - [ADR-0015](../../../Docs/DECISIONS/ADR-0015-event-ledger-erasure-and-encrypted-replication.md) (Accepted — direction)
  - [ADR-0016](../../../Docs/DECISIONS/ADR-0016-content-envelope-cryptography.md) (Accepted — content CE / DEK model)
  - P8 CE path shipped (T163–T165); local-only product usable without sync
- **Unblocks:** T176–T178 (schema, fake relay, security tests) — design gate cleared; implementation not started
- **Category:** SECURITY / ARCHITECTURE
- **Master plan:** Task 11.0 in `.hermes/plans/2026-07-23_204630-memory-control-plane-successor.md`
- **ADR to produce:** **ADR-0018** (next free; ADR-0017 is desktop stack)
- **Stop-before:** Network relay code, enrollment UX productization, production sync feature flags, or claiming “metadata-private” / formal certification / PQ resistance / remote wipe without review sign-off

## 1. Objective

Define the **protocol threat model** and **normative ADR-0018** for multi-device sync of **end-to-end encrypted event envelopes** through an **untrusted relay** — never syncing mutable SQLite files. Local-only mode remains the **default** and fully functional without sync, capture, models, or graph.

This track answers **how** ADR-0015’s optional replication works; it does **not** implement `ai-brains-sync`.

| Master-plan Task 11.0 topic | T175 interpretation |
|-----------------------------|---------------------|
| Device identity | Stable `DeviceId`; Ed25519 signing + X25519 ECDH static; public material only on relay |
| Enrollment | OOB verify of **dual-key identity fingerprint** (SHA-256 of `schema_version ‖ DeviceId ‖ Ed25519_pub ‖ X25519_pub`); `DeviceEnrolled` signed by already-enrolled device; no unbound bearer PIN |
| Revocation | Stop per-recipient wraps for revoked device; future exclusion only; `DeviceRevoked` signed by enrolled device; **DeviceId permanently retired** (re-enroll = always new DeviceId) |
| Ordering | Per-device `local_seq` + topological apply by event dependencies; HLC display-only optional |
| Duplicates | Idempotent `event_id`; at-least-once delivery → exactly-once apply |
| Divergent lineage | Explicit domain conflicts — **never last-write-wins** |
| Deletion / erasure | Key tombstone + DEK-destroy propagation + **signed** per-device ACK round-trip |
| Replay | Signed envelopes (metadata+ciphertext); sequence gap handling; idempotent apply |
| Relay adversary | Fully untrusted; cannot decrypt; cannot forge device signatures |
| Metadata leakage | Residual + optional size-bucket padding; **non-claims** include metadata-private + PQ |
| Disaster recovery | RecoveryKit → DataKey only; lost device = revoke; no remote-wipe claim |
| Capture independence | Sync **optional**; capture never requires relay or `ai-brains-sync` |

## 2. Live baseline (re-scan 2026-07-30 + AI2/AI3 verify)

| Item | State |
|------|--------|
| ADR-0015 | Accepted **stub** (~7 lines, 2026-07-23): direction only; **no protocol detail** |
| ADR-0016 | Accepted (2026-07-28): per-unit content DEK under vault `DataKey`; AES-256-GCM; CE = destroy DEK wrap |
| P8 CE | T163–T165 Complete — local only; **no multi-device ACK** |
| Workspace crypto | `aes-gcm` **0.10**, `zeroize` **1.8**, `subtle` **2.6**, `rand` **0.10**, `sha2` **0.11**, `argon2` **0.5** — **no** ed25519/x25519/hkdf/openmls/hpke yet |
| Migrations | Through **`0026_content_envelopes_erasure.sql`** — T176 uses **`0027+`** (not master-plan 0026) |
| CLI collision **#1** | `ai-brains sync *` — `Commands::Sync` — Ledgerful bridge (`query` / `pull` / `push`) |
| CLI collision **#2** | `ai-brains safety sync` — `SafetyCommands::Sync` — hotspot pin into vault (**also** spelled “sync”) |
| Migration `0017_sync_state` | Bridge helper KV — **not** multi-device replication |
| Crate `ai-brains-sync` | **Does not exist** |
| Capture independence | Unchanged |
| Deferred #34 | ACK design → T175; ACK implement → T176–T178; DataKey rotation **direction** here, **implementation residual** stays open |

## 3. Research summary (online + standards + AI reviews, 2026-07-30)

### 3.1 Product shape (what we are **not**)

| Pattern | Decision for AI-Brains v1 sync |
|---------|--------------------------------|
| SQLite / SQLCipher file replication | **Forbidden** |
| CRDT document store (Automerge / Yjs) | **Not default** — event union + domain conflicts |
| MLS / OpenMLS (RFC 9420) | **Deferred** — N≈2–5 single-owner devices; revisit only if multi-**principal** groups |
| Matrix / Synapse | Threat-list only — **no** AGPL stack |
| Multi-user / multi-tenant vault sharing | **Out of scope v1** (see L15) |
| Deniable authentication / group messaging | **Out of scope v1** |
| Sealed-sender / metadata-hiding | **Not claimed**; optional size-bucket padding only (L14) |
| Post-quantum hybrid KEM | **Not claimed** (non-claim + residual harvest-now-decrypt-later) |

**Normative:** encrypted **event envelope** replication + local projection rebuild — not DB sync, not CRDT merge.

### 3.2 Threat-modeling method

**STRIDE** on boundaries: device↔vault, device↔relay, peer-via-relay.

| STRIDE | Sync mapping |
|--------|----------------|
| **S**poofing | Fake device; enrollment without fingerprint OOB; unknown `device_id` injection |
| **T**ampering | Bit-flip; metadata swap under signature; cursor rewrite; forged ACK |
| **R**epudiation | Unsigned enroll/revoke/ACK |
| **I**nformation disclosure | Relay plaintext (must fail design); size/timing/graph; PQ harvest-now |
| **D**enial of service | Drop/reorder/flood; signature-verify DoS from unknown device_ids; gap starvation |
| **E**levation | Revoked device decrypts **future** content; multi-principal trust creep |

### 3.3 Multi-device E2EE lessons

- Device compromise in-scope; **future exclusion** is the v1 revocation bar (not retroactive wipe of past keys).
- Enrollment must bind **both** Ed25519 and X25519 under one fingerprint (canonical package hash), not unbound bearer PIN (MITM); enroll/revoke signed by already-enrolled device.
- Metadata almost always leaks on untrusted store — honesty > marketing.
- PQ posture stated explicitly even when not mitigated (Signal PQXDH / industry practice).

### 3.4 Ordering without LWW

| Approach | v1 |
|----------|-----|
| Server total order as SOV | **Reject** |
| `(device_id, local_seq)` | **Required** for per-device streams + gap detection (L13) |
| HLC | **Optional display / soft sort only** — not apply SOV |
| Apply order | **Topological** by declared event dependencies (parent/correlation ids already in domain events), tie-break `(device_id, local_seq, event_id)` |
| CRDT auto-merge / LWW | **Forbidden** for conclusions/decisions |

**Conflict rule:** Concurrent contradictory conclusions/decisions → explicit conflict/review (ADR-0014). Event **sets** converge; projectors must handle missing-parent via existing staleness machinery (T149/T150), not silent LWW.

### 3.5 Cryptographic primitives (named for ADR; **no deps in T175**)

Research date: **2026-07-30**. Pin by **major/minor** in ADR; exact patch via Cargo.lock at T176. Drop volatile publish-date parentheticals.

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
| `curve25519-dalek` | **5.x** (transitive) | BSD-3-Clause | Curve arithmetic — **must** be in inventory; verify features |
| `hkdf` | **0.13.x** | MIT OR Apache-2.0 | KDF for wrap keys |
| `openmls` | 0.8.x | MIT | **Not v1** |
| `hpke` | 0.14.x | MIT OR Apache-2.0 | **Considered; deferred v1** (see §3.5.2) |
| `merlin` | (if pulled) | MIT | Transcript — name only if used |

**Intended feature flags (T176):**

```toml
ed25519-dalek = { version = "3", features = ["serde", "zeroize", "rand_core"] }
x25519-dalek  = { version = "3", features = ["serde", "zeroize", "static_secrets"] } # confirm static_secrets need; prefer ephemeral-for-wrap
```

- **`zeroize` feature mandatory** (AGENTS.md key material / ZeroizeOnDrop).
- **`serde`** for durable public device records.
- **`rand_core`** for `SigningKey::generate` / ephemeral ECDH.
- Prefer **not** enabling long-lived raw static-secret patterns that fight zeroize discipline; lock at T176 after API check.
- API note: dalek 3.x uses `SigningKey` / `VerifyingKey` (not 2.x `Keypair`).

**Forbidden:** AGPL Matrix SDKs, GPL sync engines, hand-rolled ECC/AES, unknown-git crypto, using raw X25519 shared secret as AES key without KDF, proprietary BaaS required for local-only.

#### 3.5.1 Multi-device content-DEK wrap construction (**frozen — B2 HIGH**)

Local wrap (ADR-0016) is **DataKey → AES-GCM → content DEK**. There is **no shared DataKey** across devices.

**v1 freeze: per-recipient sealed wrap (option 1), not group epoch KEK.**  
**Normative byte encoding:** [ADR-0018 §17](../../../Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md) (R1 M3 freeze). Summary:

For each content unit and each **enrolled, non-revoked** peer device (X25519 must match enrolled package):

1. Generate ephemeral X25519 keypair; `shared = X25519(eph_priv, peer_static_x25519)`.
2. `wrap_key = HKDF-SHA256(salt=[], ikm=shared, info= schema_version(u16 BE) ‖ u16be(len(label)) ‖ "aib-sync-dek-wrap" ‖ content_key_id(16) ‖ recipient_device_id(16) ‖ sender_device_id(16))` → 32 bytes.
3. AES-256-GCM-wrap content DEK under `wrap_key` with AAD = `schema_version(u16 BE) ‖ content_key_id(16) ‖ recipient_device_id(16)` (mirror ADR-0016 AAD discipline).
4. Persist **N wrap rows** (one per recipient) with ephemeral public, nonce, ciphertext+tag — **no plaintext DEK**.
5. **Revocation** = stop creating wrap rows for that `device_id` (and reject post-revoke enroll). No group re-key ratchet required for v1.

T178 WRAP KATs must use the exact ADR §17 bytes (`T178-WRAP-kat-info-aad-bytes`).
**Rejected for v1 primary:**

| Option | Why not v1 |
|--------|------------|
| Group / epoch KEK (HKDF advance on revoke) | MLS-adjacent complexity; schema + rekey race; AI1 proposed — **defer** as future optimization if N grows |
| Raw X25519 output as AES key | Cryptographically wrong |
| Shared vault DataKey over relay | Breaks E2E / device isolation |

**Why per-recipient over epoch:** matches ADR-0016 per-unit DEK granularity; trivial revoke (omit recipient); O(N) wraps acceptable for N≈2–5; T176 schema = wrap table keyed by `(content_key_id, recipient_device_id)`.

#### 3.5.2 HPKE (RFC 9180) evaluation (**B6 / AI2**)

**Considered:** `hpke` ~0.14 (MIT/Apache) standardizes KEM+KDF+AEAD for “encrypt to recipient public key” and reduces hand-composition footguns.

**v1 decision: defer HPKE crate** — implement explicit X25519 + HKDF-SHA256 + AES-256-GCM with **frozen info/AAD labels** in ADR-0018 so construction is auditable line-by-line and reuses already-named workspace-adjacent crates. HPKE remains a **hygiene candidate** if security review or interop demands a single standard API (same suite: X25519 / HKDF-SHA256 / AES-256-GCM).

Silence was incorrect; this is the recorded rejection.

### 3.6 Erasure + multi-device (absorbs deferred #34 carefully)

Local CE (ADR-0016 / T165): destroy content DEK wrap + purge derived plaintext.

**Multi-device extension:**

1. Erasing device emits a **signed key-tombstone / erasure control envelope** (cleartext control payload; ADR-0018 §5.1.1).
2. Peers apply: destroy local DEK wrap for that `ContentKeyId` + purge derived rows + tombstone.
3. Peers send **signed ErasureAck control envelopes** back through the relay (cleartext signed control; `wrap_count = 0`).
4. **Erasing device** maintains **local** ACK projection: `(erasure_id, peer_device_id) → pending | acked | failed | unreachable` (timeout after N sync cycles). Relay **cannot** forge ACKs. A valid ACK is **peer attestation** of local apply/CE steps — not remote media wipe proof; compromised enrolled peer can emit false ACK until revoked.
5. Offline peer residual: decryptable until peer syncs — **best-effort propagation**, not remote media sanitization.
6. **NIST SP 800-88r2:** scope is media under the **operator’s control**. Peer/stolen device storage is **not** operator-controlled media. Do **not** market multi-device CE as Purge, Destroy, or “remote wipe.”
7. **DataKey rotation (local vault KEK):** direction in ADR-0018; **implementation remains deferred** (P11 hygiene) — do not strike #34 wholesale.
8. **Multi-device wrap nonce budget (B4):** per-recipient wrap keys are **per seal** (ephemeral ECDH → fresh wrap_key) or short-lived; message count per wrap_key is **O(1)**. This is **strictly better** than ADR-0016 vault-lifetime DataKey budget. No extra multi-device nonce ledger required for v1 beyond ADR-0016 DataKey residual.

## 4. Design locks (to freeze in ADR-0018)

Proposed normative locks for security review. **L1–L16** after AI fold-in.

### L1 — Architecture

```
Local vault (SQLCipher + event log + projections + CE side stores)
    │  seal/sign encrypted envelopes (client-side)
    ▼
Untrusted relay (opaque blob store + optional receive timestamps)
    │  push/pull by cursor; no decrypt capability
    ▼
Peer vault: enrolled-set check → verify sig → decrypt → append → project
```

- Local ledger authoritative per device; relay is **not** SOV.
- Capture/FTS/briefing work with sync **off**.

### L2 — Device identity

- `DeviceId` (stable UUID) + **Ed25519** long-term signing key + **X25519** static ECDH public for recipient wraps.
- Relay: public keys + metadata only — never private keys or vault DataKey.
- Device private key storage: T176 detail (DPAPI / wrap under DataKey) — open Q4.

### L3 — Enrollment (fingerprint-bound OOB)

- **Enrollment package** = `schema_version ‖ DeviceId ‖ Ed25519_pub ‖ X25519_pub` (canonical identity blob).
- OOB-verified artifact = **`fingerprint = SHA-256(enrollment_package)`** — **both** public keys bound (not Ed25519 alone); human-readable / QR.
- Owner confirms match on an **already-enrolled** device (or **RecoveryKit-bound first-device bootstrap**, local-only — no untrusted-relay self-enroll of unknown `device_id`).
- **`DeviceEnrolled` is signed by an already-enrolled device** after OOB confirm. Self-enroll / enroll signed only by the new unknown device → **reject at L8**.
- **Bearer pairing codes without key binding are insufficient** (MITM-vulnerable).
- Unverified or **mismatched X25519** must not receive DEK wraps (`T178-L3-enroll-binds-x25519`).
- Enrollment emits signed `DeviceEnrolled` as a special **cleartext control** envelope type on the **same stream** (ADR-0018 §22 Q1; §5.1.1).
- **Re-enrollment after revoke:** that `DeviceId` is **permanently retired (tombstoned)** — never reused. Same physical machine always gets a **new DeviceId** + **new key material** + **full dual-key OOB**.

### L4 — Revocation

- Owner issues `DeviceRevoked` **signed by an already-enrolled, non-revoked device**.
- **Future exclusion:** no new per-recipient wrap rows for revoked device; reject post-revoke envelopes from that id after revoke applied.
- **DeviceId retirement:** revoked id is permanently tombstoned; re-enroll of the machine uses a new DeviceId (L3).
- **Residual:** past DEKs already held remain openable (stolen laptop class).
- **Not v1:** mandatory group epoch KEK ratchet (optional future; see §3.5.1).
- T178: `T178-L4-deviceid-permanently-retired` among revoke suite.

### L5 — Envelope model + signature scope

- Unit = outer transport envelope: signed outer fields + detached Ed25519 + body field (wire name may stay `ciphertext`) + **N wrap records** (data only).
- **Normative complete `signed_bytes`:** [ADR-0018 §5.2](../../../Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md) — not a partial “at least” list.
- Encoding freeze (summary): all multi-byte integers **BE**; UUIDs **16 raw bytes**;  
  `schema_version ‖ envelope_id ‖ device_id ‖ local_seq ‖ content_type_code ‖ event_id ‖ content_key_id ‖ ciphertext_len ‖ ciphertext ‖ wrap_count ‖ wrap_records…`  
  with each wrap = `recipient_device_id ‖ eph_x25519_pub(32) ‖ wrap_nonce(12) ‖ wrap_ct_len ‖ wrap_ct`, sorted by `recipient_device_id` ascending.
- **Data envelopes:** body = **`nonce (12) ‖ ciphertext ‖ tag (16)`** as one opaque blob (`ciphertext_len` covers full blob); content DEK AEAD; `wrap_count ≥ 1` for intended peers; no separate unsigned nonce on wire (ADR-0018 §5.3). Local store may split nonce columns and convert at encode/decode.
- **Control envelopes** (`DeviceEnrolled`, `DeviceRevoked`, erasure tombstone, `ErasureAck`, …): body = **cleartext control payload** (prose: `payload`); **not** encrypted under a content DEK; **`wrap_count = 0` always**; `content_key_id` zero UUID except erasure/ACK target key. Peers verify sig + enrolled-set, then parse cleartext. Public control OK: enrollment keys must be public; revoke/erasure/ACK are membership/integrity signaling; authenticity via Ed25519 (ADR-0018 §5.1.1).
- **Rule:** any wire field outside AEAD used for routing/apply **must** be in `signed_bytes`. Wrap list is **outer signed metadata** (v1, data path). AEAD/sig/metadata-swap all **fail closed**.
- No plaintext **event** bodies on relay or in replication tables (control payloads are cleartext by design, not event content).
- Idempotent apply: outer `event_id` (+ `envelope_id` for transport).
- At-least-once delivery; exactly-once apply by id.
- T178: `T178-L5-sig-canonical-bytes`, `T178-L5-meta-swap-fails`, `T178-L5-wrap-list-tamper`, **`T178-L5-content-nonce-in-blob`**, **`T178-L5-control-cleartext-parse`**, `T178-L5-tamper-ct`, `T178-L5-replay-idempotent`.

### L6 — Ordering and conflicts

- Monotonic per-device **`local_seq`** (required).
- **Apply order:** topological by declared event dependencies; tie-break `(device_id, local_seq, event_id)`.
- Projectors handle missing parent via staleness (existing machinery) — not silent drop of child.
- HLC optional for display only.
- Concurrent epistemic contradictions → **explicit conflict** (never LWW).

### L7 — Erasure propagation + ACK

- Tombstone/erasure control envelopes are **cleartext signed control** (same as L5 control freeze); propagate on the stream.
- **ACKs are signed cleartext control envelopes** from each peer → erasing device via relay (`T178-L7-ack-cleartext-signed`).
- Erasing device holds **local** ACK projection (`pending | acked | failed | unreachable`).
- Relay cannot forge ACKs; forged ACK fails signature (T178).
- **ACK residual:** signed ACK proves an **enrolled device attested** local apply/CE steps — **not** remote media sanitization or malware-free peer. Compromised enrolled device can emit valid false ACK until revoked. UX **MUST NOT** treat a single ACK as “cryptographically proven wiped everywhere.”
- UX: honest partial multi-device erase; no “erased everywhere” without policy quorum / all-acked; states stay honest enums.

### L8 — Replay, tamper, enrolled-set gate

- AEAD **fail-closed**; signature **fail-closed**; metadata swap **fail-closed**.
- Replay of same envelope/event id = no-op success.
- **Before signature verification:** check `device_id` ∈ enrolled **and** not revoked; unknown id → reject **without** expensive crypto work (DoS mitigation).
- T178: inject unknown `device_id`; inject metadata-swapped envelope; inject forged ACK.

### L9 — Relay adversary model

| Capability | Allowed | Outcome |
|------------|---------|---------|
| Store, drop, reorder, duplicate | Yes | No plaintext |
| Observe sizes, times, device graph | Yes | Residual |
| Inject unknown device_id envelopes | Yes attempt | Reject pre-verify |
| Forge valid device signatures / ACKs | No | Reject |
| Modify ciphertext or signed metadata undetected | No | Reject |
| Read DataKey / content DEKs | No | Design fail if possible |

### L10 — Optionality and naming (two collision surfaces)

- Default: **local-only**; multi-device off.
- **Do not repurpose** either existing surface:
  1. `ai-brains sync *` (`Commands::Sync`) — Ledgerful bridge  
  2. `ai-brains safety sync` (`SafetyCommands::Sync`) — hotspot pin  
- **v1 multi-device CLI freeze:**
  - **`ai-brains device`** — enroll / list / revoke / show fingerprint  
  - **`ai-brains replicate`** — push / pull / status / cursors (engine operations)  
- Crate remains `ai-brains-sync` (library name ≠ CLI `sync`).

### L11 — Disaster recovery + CE honesty

- RecoveryKit restores DataKey only; not destroyed DEKs.
- Pre-erase backup residual unchanged.
- Multi-device CE = **best-effort key-destruction propagation**, not NIST remote sanitization / remote wipe (800-88r2 operator-controlled media scope).

### L12 — Capture independence & licenses

- No `ai-brains-capture` → sync dependency.
- PolyForm NC + Small-Entity Commercial Exception; optional sync so commercial local-only needs no relay.
- Named primitives deny-allowlisted only.

### L13 — Sequence gap detection (AI1)

- Per peer stream: track `expected_local_seq` / high-water.
- Gap → `sync_gap` state: buffer out-of-order envelopes for that device; request missing seq range from relay (or wait); **do not** corrupt projections by applying past-gap without policy.
- After fill or explicit skip policy (documented), resume ordered apply.

### L14 — Payload length padding (AI1, residual mitigation)

- Before envelope seal, pad plaintext to **fixed buckets** (e.g. 256 B / 4 KiB / 64 KiB) to reduce event-type inference from size.
- **Does not** eliminate metadata leakage (timing, counts, graph, bucket itself).
- Non-claim “metadata-private” still stands; padding is best-effort hardening.

### L15 — Single-owner / single-vault membership (B3 HIGH)

- **v1 sync = one human principal, one vault membership group, N personal devices.**
- Multi-user vault sharing, multi-tenant, deniable auth, multi-principal group messaging **out of scope**; require a **new ADR** (likely MLS-class) if product needs them.
- Makes MLS deferral coherent.

### L16 — Post-quantum non-claim (AI2)

- v1 uses **classical ECC only** (Ed25519 / X25519).
- **Not post-quantum resistant.** Ciphertext retained by a future quantum-capable adversary who also obtains relay-stored blobs is a **residual** (harvest-now-decrypt-later). Not mitigated in P11 v1.

## 5. Threat model document structure (deliverable)

Path: `conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md` (primary).

1. Assets  
2. Actors (owner, devices, revoked, relay, MITM, malware-on-one-device)  
3. Trust-boundary DFD  
4. STRIDE tables + mitigations (incl. enrolled-set DoS, metadata, PQ)  
5. Residual risks (metadata; offline CE lag; revoked past keys; pre-erase backups; classical-only PQ; no FIPS)  
6. **Non-claims:** metadata-private; perfect multi-device deletion; NIST Purge/remote wipe; compliance certs; **post-quantum resistance**; multi-user vault  
7. **Traceability matrix (authoritative):** each L1–L16 + residual + non-claim → T178 test id or explicit defer  

## 6. ADR-0018 sketch (deliverable)

**Title:** Encrypted Event Envelope Replication Protocol (Untrusted Relay, Single-Owner)

**Status:** **Accepted** — 2026-07-30 (after internal R2 CLEAN + Codex R1 FAIL→fix + Codex R2 FAIL→fix + **Codex R3 PASS**).

**Must include:** L1–L16; §3.5.1 wrap construction; HPKE deferral; dual CLI collision + `device`/`replicate` freeze; migration `0027+`; fake-relay-first; PQ + 800-88r2 honesty; DataKey rotation **direction only**.

## 7. Security review gate

| Gate | Requirement |
|------|-------------|
| Internal | Review threat model + ADR vs this spec — **done** (R2 CLEAN) |
| External | May wait for T184; internal Accept + **Codex R3 PASS** cleared T176 design block |
| Evidence | ADR **Accepted** 2026-07-30; residuals listed; conductor unblocks T176 |

## 8. Deferred absorption

| Deferred | Disposition |
|----------|-------------|
| **#34 (1) multi-device key tombstone / erasure ACK** | **Design absorbed** (L7); **implement T176–T178** |
| **#34 (2) DataKey rotation / wrap-nonce accounting** | **Direction** in ADR-0018; **implementation residual remains open** (P11 hygiene) — do **not** strike wholesale |
| **#34 (3) historical out-of-T162** | Historical note only |
| Connector cursor, ChangeGuard renames | Out of scope |

## 9. Non-goals (this track + protocol v1 fence)

**This track:** no `ai-brains-sync` code, no network relay, no desktop enrollment UX, no Cargo dep adds.

**Protocol v1 fence:**

- Multi-user / multi-principal vault sharing  
- Deniable authentication / group messaging  
- MLS / OpenMLS / libp2p as required core  
- HPKE crate (deferred; construction still X25519+HKDF+AES-GCM)  
- Group epoch KEK as primary wrap (deferred)  
- Post-quantum hybrid  
- Replacing `sync` or `safety sync` CLI meanings  

## 10. Dependency / license gate (ADR-only)

- **T175 adds zero crates.**  
- Named licenses only: MIT, Apache-2.0, BSD-3-Clause, ISC, (MPL if already policy).  
- T176: `cargo deny check` + `cargo audit` including **curve25519-dalek 5.x** transitive.

## 11. Definition of Done

- [x] Threat model complete (incl. non-claims PQ + remote-wipe honesty)
- [x] ADR-0018 drafted with **L1–L16** + wrap construction §3.5.1
- [x] Dual naming collision + `device` / `replicate` freeze recorded
- [x] Migration `0027+` note
- [x] Erasure ACK model frozen; DataKey rotation direction only (residual open)
- [x] Traceability matrix L-locks → T178
- [x] Security review → ADR **Accepted** (internal R2 CLEAN + Codex R1 FAIL→fix + Codex R2 FAIL→fix + **Codex R3 PASS**; Accepted 2026-07-30)
- [x] Conductor: T175 Complete; T176 unblocked
- [x] No production network/sync code
- [ ] ledgerful provenance on doc commit — **orchestrator** (ledger tx open; closeout agent does not commit)

## 12. Open questions (resolve in ADR before Accept)

> **Note (L4 / R1):** Safe defaults for items 1–6 below live in **[ADR-0018 §22](../../../Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md)**. They are **normative** as of ADR-0018 **Accepted** 2026-07-30; T176 must not diverge from §22 without an ADR amendment.

1. Control records as special event types vs separate signed control stream? → **ADR-0018 §22 Q1** (same stream, distinct `content_type_code`)  
2. Selective sync: whole vault only vs project filter (default whole-vault recommended)? → **§22 Q2**  
3. ACK timeout N (sync cycles) and UX copy for partial CE? → **§22 Q3** (default N = 3)  
4. Device private key storage: DPAPI-only vs wrap under DataKey? → **§22 Q4** (DataKey + DPAPI recommend)  
5. First-device bootstrap when no enrolled peer exists (recovery-kit path detail)? → **§22 Q5** (RecoveryKit-bound, local-only; no relay self-enroll)  
6. Gap skip policy if seq permanently lost (fail-closed vs operator skip)? → **§22 Q6**  

**Resolved by fold-in (no longer open):** wrap construction (per-recipient X25519+HKDF+AES-GCM with frozen info/AAD bytes); HPKE deferred; HLC display-only; dual-key fingerprint enrollment + enroll/revoke signer rules; single-owner; signature covers metadata+body; CLI `device`/`replicate`; PQ non-claim; dual sync collisions; **DeviceId permanently retired after revoke** (re-enroll always new DeviceId); control cleartext signed; data AEAD nonce packed in body blob.

## 13. Review fold-in disposition (AI1–AI3)

| Source | Item | Disposition |
|--------|------|-------------|
| AI1 | Untrusted relay / STRIDE / OOB / non-LWW | Confirmed (already in baseline) |
| AI1 | L13 sequence gap | **Accepted** → L13 |
| AI1 | L14 size-bucket padding | **Accepted** as residual mitigation → L14 |
| AI1 | Epoch HKDF group KEK as primary | **Rejected for v1 primary**; per-recipient wrap frozen; epoch = future opt |
| AI1 | CLI `device` + `replicate` | **Accepted** → L10 |
| AI2 | HPKE named evaluation | **Accepted** → §3.5.2 defer |
| AI2 | PQ non-claim | **Accepted** → L16 |
| AI2 | Dual CLI sync collision | **Accepted** → L10 / baseline |
| AI2 | Signature covers metadata+ciphertext | **Accepted** → L5 |
| AI3 B1 | curve25519-dalek 5.x transitive | **Accepted** → §3.5 |
| AI3 B2 | Freeze wrap construction | **Accepted** → §3.5.1 per-recipient |
| AI3 B3 | Single-owner fence | **Accepted** → L15 |
| AI3 B4 | Multi-device nonce budget | **Accepted** → §3.6 item 8 |
| AI3 B5 | dalek feature flags | **Accepted** → §3.5 |
| AI3 B6 | HPKE evaluate | **Accepted** (with AI2) |
| AI3 B7 | Topological apply order | **Accepted** → L6 |
| AI3 B8 | Enrolled-set before verify | **Accepted** → L8 |
| AI3 B9 | Fingerprint OOB | **Accepted** → L3 |
| AI3 B10 | Signed ACK round-trip | **Accepted** → L7 |
| AI3 B11 | 800-88r2 remote media honesty | **Accepted** → L11 / §3.6 item 6 |
| AI3 B12 | Traceability matrix C9 | **Accepted** → plan C9 / threat-model §7 |
| AI3 B13 | Drop publish-date pins | **Accepted** → §3.5 |
| AI3 B14 | Partial #34 strike only | **Accepted** → plan E2 / §8 |

## 14. References

- ADR-0015, ADR-0016, ADR-0011, ADR-0014  
- Vision §11–12 — `Docs/MEMORY-CONTROL-PLANE-VISION.md`  
- Master plan Phase 11 — Task 11.0–11.3  
- NIST SP 800-88 Rev. 2 (scope: operator-controlled media)  
- NIST SP 800-38D — GCM nonce  
- RFC 9180 HPKE — considered, deferred v1  
- RFC 9420 MLS — deferred multi-principal  
- STRIDE (Microsoft SDL)  
- crates.io research 2026-07-30: ed25519-dalek 3.x, x25519-dalek 3.x, curve25519-dalek 5.x, hkdf 0.13, hpke 0.14, openmls 0.8  
- Live CLI: `Commands::Sync`, `SafetyCommands::Sync`  
- Deferred #34 (T162) — partial promote  
