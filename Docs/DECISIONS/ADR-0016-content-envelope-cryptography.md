# ADR-0016: Content-Envelope Cryptography (Implementation Choices for Cryptographic Erasure)

## Status

**Accepted** — 2026-07-28 (human accept after T162 spike + design review fold-in).

Normative for T163–T166. Supersedes informal implementation notes; **implements** the direction of [ADR-0015](ADR-0015-event-ledger-erasure-and-encrypted-replication.md) without changing its product vision.

## Context

ADR-0015 accepted that:

- The internal history remains an **append-only** event ledger.
- Sensitive payloads use **separable encryption keys** so deletion can make content unrecoverable while retaining minimal safe tombstones.
- Optional multi-device sync will replicate **end-to-end encrypted envelopes** (P11), not live SQLite files.

P7 shipped durable **erasure tickets** (`ErasureTicketAccepted`) and daemon-required erasure UX, but **explicitly does not** perform content-envelope wipe. Soft `MemoryForgotten` still leaves plaintext in the event log.

Before schema (T163) or crypto service code (T164), Task 8.0 requires freezing algorithms, key hierarchy, nonce policy, recovery, FTS/embedding behavior, crash safety, tombstones, and **honest limits** for legacy plaintext.

### Standards consulted (2026-07)

| Source | Takeaway for AI-Brains |
|--------|------------------------|
| **NIST SP 800-88 Rev. 2** (final Sep 2025) | Cryptographic erase (CE) = destroy key material so recovery of target data is infeasible; expanded CE/key-sanitization guidance; do not claim physical media Purge/Destroy. CE *may* support Purge-level sanitization only under conditions that include a **validated** crypto module — see §12 |
| **NIST SP 800-38D** (AES-GCM) | Nonce uniqueness under a key is critical; random 96-bit IVs → keep ≤ ~2³² messages/**per key** (applies to **every** AEAD key, including vault `DataKey` wraps) |
| **Envelope encryption** (industry / KMS patterns) | DEK encrypts data; KEK wraps DEK; store wrap with ciphertext; zeroize DEK after use |
| **ISO/IEC 19790** (via 800-88r2 CE notes) | Prefer proper zeroization of key material in memory when destroying keys |

### Existing stack (reuse)

| Piece | Version / location | License |
|-------|-------------------|---------|
| `aes-gcm` (`Aes256Gcm`) | workspace **0.10.3** | Apache-2.0 OR MIT |
| `zeroize` | 1.8.x | Apache-2.0 OR MIT |
| `argon2` | 0.5.3 (passphrase KDF only) | MIT OR Apache-2.0 |
| `DataKey`, DPAPI/passphrase `RecoveryKit` | `ai-brains-crypto` | Project |
| SQLCipher via store | vault-level at-rest | Existing deny-green |

**No new crypto crate is required for v1.** Prefer staying on `aes-gcm` 0.10.x; 0.11.x is optional later hygiene (MSRV 1.85+).

## Decision

### 1. Key hierarchy

```
Passphrase and/or DPAPI  →  unlock  →  DataKey (vault KEK)
DataKey                  →  AEAD-wrap →  Content DEK (32 bytes)
Content DEK              →  AEAD-seal →  ciphertext (nonce + ciphertext+tag)
```

- **CE primitive:** destroy all durable wraps of the **content DEK** (and purge derived plaintext).  
- **Not CE:** rotating or destroying the vault `DataKey` / SQLCipher key (that is vault destruction, not selective erase).

### 2. Key granularity

**Default: one content DEK per independently erasable content unit**, identified by `ContentKeyId`.  
**Content DEK material:** **32-byte OS CSPRNG random** (fresh per unit). **Not** HKDF-from-DataKey (or any KDF-from-DataKey) in v1.

| Rejected | Why |
|----------|-----|
| Single vault-wide content key | Selective CE impossible |
| Per-scope batch DEK as default | Erasing one key blinds many items; coarse UX |
| hkdf-from-DataKey as content DEK | Prefer independent random DEKs; optional later if product needs derived keys |

Sharing a DEK across units is allowed only when product rules require **atomic co-erasure**.

### 3. AEAD algorithm

**AES-256-GCM** via the existing RustCrypto `aes-gcm` crate (`Aes256Gcm`).

| Deferred | Why not v1 primary |
|----------|--------------------|
| AES-GCM-SIV | Extra dependency; per-unit DEKs already avoid nonce-misuse pressure |
| XChaCha20-Poly1305 | Useful for high-volume single-key seals; not needed for O(1) seals/DEK |

**Forbidden:** hand-rolled block modes, custom GHASH, unaudited “crypto helpers.”

### 4. Nonce and AAD

| Item | Choice |
|------|--------|
| Nonce | 96-bit (12-byte) **random** from OS CSPRNG, fresh per **AEAD invocation** (content seal **and** DEK wrap under DataKey) |
| Message budget (content DEK) | Design for few seals per DEK (typically one); rotate DEK rather than approach 2³² |
| Message budget (DataKey as wrap KEK) | **Documented residual** — see below |
| AAD | At least **envelope schema version** + **content_key_id** bytes; include stable blob/content id when known |
| Secrets in AAD | Never |

**DataKey wrap-nonce budget (honesty extension):**  
Content-DEK seals stay low-count by construction (per-unit DEK). The **DataKey → wrap → Content DEK** layer uses the **same AES-256-GCM + random 96-bit nonce pattern as passphrase key-wrap**. **DataKey is the direct AES-GCM key** for content-DEK wrap (**no Argon2 / no passphrase KDF on this path**). That invocation runs **once per content unit created**, under a single key the hierarchy classifies as **vault-lifetime** and **not destroyed on CE** (§1).

- **Near-term risk:** Accepted as **out of scope** for the original v1 envelope design — NIST’s ~2³² random-nonce threshold is far beyond plausible personal-vault wrap rates.  
- **Rotation (shipped direction):** Operational **DataKey rotation** is specified and implemented under **[ADR-0020](ADR-0020-datakey-rotation.md)** / **T189** (`vault rotate-datakey`): re-wrap active DEKs, re-seal local device private, change page key via crash-safe `sqlcipher_export` (primary) or opt-in `PRAGMA rekey`. Do not silently treat content-DEK budget analysis as covering the KEK layer without rotation.

### 5. Wrapping and recovery

- Wrap each content DEK under the vault **DataKey** with AES-256-GCM — **same AEAD family / AEAD+nonce pattern as passphrase wrap of DataKey**; **DataKey bytes are used directly as `Aes256Gcm` key material** (not Argon2-derived).  
- Persist **only** wrapped DEK material at rest.  
- `RecoveryKit` restores **DataKey** only; it must **not** resurrect destroyed content DEKs.  
- Zeroize plaintext DEKs after seal/open (`Zeroize` / `ZeroizeOnDrop`).  
- No keys or raw ciphertext in `Debug` / default tracing.

### 6. Storage shape (normative intent for T163)

- Keep event envelope **schema version 1** (no event envelope v2 required for v1).  
- Store ciphertext and key wraps in **dedicated projections / blob tables**, with events carrying `content_key_id` (and related ids) references.  
- Migration id: **next free** after `0025_briefings_query_traces.sql` (master-plan name `0025_content_envelopes_*` is **invalid** as-is).

### 7. FTS, embeddings, caches

- Envelope-class content must not leave durable **plaintext** FTS/embedding rows that survive CE.  
- Respect T157 privacy: **Sealed never embed**; cloud gates unchanged.  
- On erase: purge FTS, embeddings, and plaintext projection rows for that content; ciphertext may remain undecryptable.

### 8. Crash-safe order

**Seal:** generate+persist DEK wrap → write ciphertext → append referencing event → zeroize plaintext.  
Orphans: wrap-without-blob (safe), blob-without-wrap (garbage), event-without-blob (unavailable).

**Erase (T165):** policy → `ContentErasureRequested` → **destroy DEK wrap** → purge derived plaintext → mark dependents → `ContentErased` + tombstone → verify undecryptable.  
Never emit success `ContentErased` without successful key destroy.

### 9. Tombstones

Retain: `content_key_id`, `tombstone_id`, timestamps, non-sensitive reason codes, linkage needed for “broken support.”  
Forbid: sensitive plaintext, embeddings, DEK material.

### 10. Ticket vs cryptographic erasure

| Mechanism | Event / path | Claim |
|-----------|--------------|-------|
| Ticket | `ErasureTicketAccepted` (P7) | Intent accepted; **not** CE |
| Soft forget | `MemoryForgotten` | Hide/filter; **not** CE |
| CE | `ContentErasureRequested` → key destroy → `ContentErased` | CE under this ADR’s assumptions |

### 11. Legacy plaintext impossibility

Events and projections that already store **plaintext** in the append-only log **cannot** be cryptographically erased without history rewrite (forbidden by event-sourcing invariants).

- Soft forget remains the only mechanism for that class.  
- Forward migration may seal **new** copies under envelopes; it does **not** remove historical plaintext copies already logged.  
- Product and CLI must **never** claim CE for pre-envelope content.  
- **Operations surface:** the same impossibility and ticket≠CE honesty must appear in [OPERATIONS.md](../OPERATIONS.md) (erasure / governed ops) — not only in this ADR (spec L10).

### 12. Honest CE claim language

**May claim (after T165):**  
For envelope-backed content, after successful governed erase, plaintext is not recoverable from the live vault given destruction of the content DEK wraps and purge of derived indexes, assuming AES-256 remains unbroken and no offline pre-erase copy exists.

**Must not claim:**

- NIST media **Purge** / **Destroy** or physical sanitization — including “CE = Purge equivalence.” Under SP 800-88r2, CE can support Purge-level outcomes only under conditions that include a **NIST-validated crypto module** (e.g. FIPS 140). AI-Brains uses **plain RustCrypto** (`aes-gcm` 0.10.x pure Rust), which is **not** FIPS-/NIST-module-validated; key destruction alone does **not** earn Purge language  
- Erasure of user-held offline copies, exports, or **pre-erase backups**  
- That `ErasureTicketAccepted` or soft forget alone is CE  
- That vault SQLCipher encryption alone provides per-item CE  

### 13. Threat model (v1 design freeze)

| Asset | Threat if leaked / retained | CE relevance |
|-------|----------------------------|--------------|
| Content DEK (plaintext or wrap) | Decrypt all blobs under that `ContentKeyId` | **Destroy wraps on erase** — primary CE primitive |
| DataKey | Unwrap all living content DEKs | **Not** destroyed on single-item CE (vault KEK) |
| Ciphertext blob | Future crypto break / offline brute force | Residual; CE does not overwrite media |
| Pre-erase backup / export | Full restore of “erased” content | Residual — out of product CE claim |
| FTS / embedding / projection plaintext | Bypass CE if left after key destroy | **Must purge** on erase path (T165) |
| Legacy event plaintext in append-only log | Permanent disclosure class | **Impossibility boundary** (§11) — soft forget only |
| Offline user copies (screenshots, exports) | Independent of vault state | Out of product CE claim |

**Assumptions (v1):** OS CSPRNG is trustworthy; AES-256-GCM remains unbroken; vault process memory is not an adversary target beyond best-effort zeroize; operators do not equate SQLCipher vault lock with per-item CE.

## Consequences

### Positive

- Selective CE without abandoning append-only provenance (tombstones + dependency staleness).  
- Reuses deny-green crypto already in production for key wrap.  
- Clear split between P7 tickets and P8 wipe — reduces false security claims.  
- Per-unit DEKs make GCM random-nonce limits a non-issue **for content seals**.

### Negative / residual risks

- Pre-erase backups and offline copies remain decryptable — must document.  
- Legacy vault history remains non-CE.  
- Operators must not confuse SQLCipher vault lock with per-item erase.  
- Dual path (soft forget + CE) increases UX complexity (T165/T166).  
- **DataKey wrap-nonce count** accumulates over vault lifetime under one non-CE key until operators run **DataKey rotation** ([ADR-0020](ADR-0020-datakey-rotation.md) / T189); §4 points to that ceremony. 
- No FIPS-validated module → never market CE as NIST Purge — §12.

### Follow-on tracks

| Track | Work |
|-------|------|
| T163 | Schema + projections for blobs, keys, erasure, tombstones |
| T164 | `content_envelope` + `content_key_store` seal/open/wrap/delete |
| T165 | Governed CE command + verify + keep soft forget for legacy |
| T166 | Class-based retention preferring CE for envelope classes |
| P8+ / P11 | **DataKey rotation** — **ADR-0020 / T189** (ceremony; not automatic) |
| T175+ | Multi-device key tombstones / erasure ACK (out of scope here) |

## Test plan outline (downstream)

| Class | Track |
|-------|-------|
| Seal/open known-answer or round-trip | T164 |
| Bit-flip tamper → open fails | T164 |
| Debug redaction / zeroize types | T164 |
| Wrong DataKey cannot unwrap DEK | T164 |
| After DEK destroy, open fails | T165 |
| FTS/projections lack plaintext after erase | T165 |
| Legacy soft forget unchanged | T165 |

## License and dependency gate

- Prefer **zero new dependencies**.  
- Allowed if later needed: RustCrypto `hkdf` / `aes-gcm-siv` / `chacha20poly1305` (**MIT OR Apache-2.0**) only with `cargo deny` + `cargo audit` green and ADR amendment.  
- **Forbidden:** AGPL/GPL crypto, unknown-git crypto, expanding hand-rolled primitives.

## References

- ADR-0015 — event ledger, erasure direction, encrypted replication vision  
- Master plan Task 8.0 — `.hermes/plans/2026-07-23_204630-memory-control-plane-successor.md`  
- Track T162 — `conductor/tracks/trackT162-content-envelope-crypto-spike/`  
- NIST SP 800-88 Rev. 2 (2025) — media sanitization / CE  
- NIST SP 800-38D — GCM nonce rules  
- `crates/ai-brains-crypto` — `DataKey`, `passphrase`, `recovery_kit`  
- Deferred #30 / #32 — CE wipe residuals after P7  

## Review checklist

- [x] Human accepts or revises this ADR — **Accepted 2026-07-28**  
- [x] On accept: set Status → **Accepted** with date; unblock T163  
- [x] Pin decision via `ai-brains pin` (session accept)  
- [x] OPERATIONS language: ticket ≠ CE; soft forget ≠ CE; legacy plaintext CE impossibility; future CE per this ADR (not implemented until T163–T165) — see [OPERATIONS.md](../OPERATIONS.md) §3 governed erasure honesty (T162 closeout)  

