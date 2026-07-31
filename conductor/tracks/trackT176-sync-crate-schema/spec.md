# T176 — Sync Crate + Replication Schema (P11.1)

- **Track ID:** T176-SyncCrateSchema
- **Phase:** P11 Task 11.1
- **Status:** 📋 **Proposed / Unblocked** — T175 Complete; ADR-0018 **Accepted** 2026-07-30 (not implementing yet)
- **Depends on:** T175 (threat model + ADR-0018 Accepted, L1–L16 + per-recipient wrap) ✅; store migrations discipline; P8 envelope types (T163–T165)
- **Category:** ARCHITECTURE / SECURITY

## Handoff freezes from T175 / ADR-0018 (Accepted — design unblocked)

| Freeze | Value |
|--------|--------|
| **Design gate** | ADR-0018 **Accepted** 2026-07-30 (Codex R3 PASS) — T176 **unblocked**; scaffold only on implementation go-ahead |
| **CLI** | **`ai-brains device`** (enroll/list/revoke/fingerprint) + **`ai-brains replicate`** (push/pull/status/cursors) |
| **Do not repurpose** | `ai-brains sync *` (Ledgerful) **or** `ai-brains safety sync` (hotspots) |
| **Crate** | `crates/ai-brains-sync` (library name ≠ CLI `sync`) |
| **Migration** | **`0027+`** (e.g. `0027_replication_state.sql`) — **not** 0026 (`0026_content_envelopes_erasure.sql` shipped) |
| **Wrap table key** | `(content_key_id, recipient_device_id)` — per-recipient X25519 + HKDF-SHA256 + AES-256-GCM |
| **Deps (named; deny/audit at implement)** | `ed25519-dalek` **3.x**, `x25519-dalek` **3.x**, transitive `curve25519-dalek` **5.x**, `hkdf` **0.13**; features `zeroize`+`serde`+`rand_core` |
| **HPKE / OpenMLS / epoch KEK** | Deferred / not v1 primary |
| **Control records** | Special signed control envelope types on **same stream** (default) |
| **Control envelopes** | **Signed cleartext payload** (not DEK-encrypted); **`wrap_count = 0` always**; wire field may stay `ciphertext` offset (prose `payload`); `content_key_id` zero UUID except erasure/ACK target key. Peers verify sig + enrolled-set, then parse. ADR-0018 **§5.1.1** |
| **Data envelopes** | Content **DEK** AEAD + **wrap list** (per-recipient); body = **`nonce(12) ‖ ct ‖ tag(16)`** single opaque blob; `ciphertext_len` covers full blob; nonce under outer sig. ADR-0018 **§5.2–5.3** |
| **Selective sync** | Whole-vault v1 default |
| **Device key storage** | Direction: wrap under DataKey + OS DPAPI — exact T176 |
| **Authoritative design** | `Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md` + `trackT175-.../threat-model.md` §7 |

## Placeholder objective

Create **`crates/ai-brains-sync`** and schema for **replication state only** — device cursors, encrypted envelope IDs, ACKs, revocation — **no plaintext content** in sync tables.

## Master-plan protocol properties (normative when ADR Accepted)

- Client-side encryption/signing  
- Idempotent event IDs  
- Per-device cursor + `local_seq` / gap handling (L13)  
- Local projection rebuild  
- Explicit conflicts; **never last-write-wins**  
- Tombstone/erasure propagation + signed ACK (L7)  
- Revoked device cannot decrypt future events  
- Size-bucket padding (L14); single-owner (L15); PQ non-claim (L16)

## Schema sketch (illustrative — ADR freezes names)

Master plan candidate: `0026_replication_state.sql` — **invalid**: `0026_content_envelopes_erasure.sql` already shipped (P8). Use **next free** (**`0027+`**) at implement.

Likely concerns:

- `device_identity` / enrollment records (public keys only)  
- `replication_cursor` per remote device/stream  
- `encrypted_envelope_index` (ids, hashes, sizes — not plaintext)  
- `ack_state` (erasure ACK projection)  
- `revocation_record`  
- **Per-recipient DEK wrap** rows: `(content_key_id, recipient_device_id)`  
- **No** event body plaintext columns  

## Naming collision warning

Existing product surfaces:

| Name | Meaning today |
|------|----------------|
| CLI `sync` / `sync query` / `sync pull` | **Ledgerful bridge** + local recall — **not** multi-device E2E |
| CLI `safety sync` | Hotspot pin into vault |
| Migration `0017_sync_state` | Key/value for bridge/sync helper state |

Multi-device feature uses **`device`** + **`replicate`**; crate `ai-brains-sync` with docs that disambiguate.

## Expected deliverables (sketch)

| Item | Notes |
|------|--------|
| `crates/ai-brains-sync` | Workspace member; feature-gated if needed |
| Migration | Deterministic; empty-state tests; **0027+** |
| Types | Envelope wrapper, device id, cursor, conflict record, control types |
| Crypto glue | Per-recipient wrap + Ed25519 sign; calls `ai-brains-crypto` where shared |
| Unit tests | Serde round-trip; id idempotency; wrap KAT stubs |

## Recommended deps (commercial-safe; confirm at expand)

| Crate | Role | License class |
|-------|------|---------------|
| Existing `aes-gcm`, `zeroize`, `rand`, `sha2`, `subtle` | Seal/sign supporting material | Already deny-green |
| `ed25519-dalek` **3.x** | Signatures | BSD-3-Clause |
| `x25519-dalek` **3.x** / `hkdf` **0.13** | Key agreement / KDF | BSD-3 / MIT OR Apache-2.0 |
| `curve25519-dalek` **5.x** (transitive) | Curve arithmetic | BSD-3-Clause — inventory + deny |
| `serde` / `thiserror` | Types/errors | Workspace |

### Avoid

| Dep | Why |
|-----|-----|
| Full **libp2p** as required core | Heavy; optional later |
| **Automerge / yrs** as default | CRDT model may not match event-log SOV |
| **OpenMLS** without multi-principal ADR | Deferred L15 |
| **hpke** crate | Considered-deferred in ADR-0018; explicit construction first |
| AGPL Matrix SDK / Synapse | Copyleft; wrong architecture |
| unknown-git crypto | `deny.toml` forbids |
| Hand-rolled ECC/AES | Project rule |

## Non-goals

- Real network relay (T177 fake first; network after)  
- Security test suite completeness (T178)  
- Changing capture independence  
- DataKey rotation **implementation** (direction only in ADR-0018; residual open)

## License / commercial gate

```text
cargo deny check
cargo audit
```

New crates must be MIT/Apache/BSD/ISC/MPL-allowlist. Sync crate must not pull GPL.

## Expand before implement

- [x] ADR-0018 **Accepted** (hard gate cleared 2026-07-30; T175 Complete / Codex R3 PASS)
- [ ] ADR field list → columns  
- [ ] Free migration id **0027+** confirmed still free  
- [ ] Crate feature flags (`sync` optional)  
- [ ] Interaction with content-envelope keys (P8)  
- [ ] CLI `device` / `replicate` surface plan  

## Definition of Done (when fleshed out)

Crate + migration green; no plaintext in replication tables; crypto via reviewed primitives; deny clean; local-only builds without sync feature still work.
