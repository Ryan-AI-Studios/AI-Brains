# T178 — Sync Security Tests + Acceptance Gates (P11.3)

- **Track ID:** T178-SyncSecurityTests
- **Phase:** P11 Task 11.3
- **Status:** 📋 **Proposed / Unblocked (design)** — T175 Complete / ADR-0018 **Accepted**; still depends on T176–T177 harness (not implementing yet)
- **Depends on:** T176–T177; P8 erase path for erasure-propagation cases when available; ADR-0018 **Accepted** ✅
- **Category:** SECURITY

## Handoff freezes from T175 / ADR-0018 (Accepted — design unblocked; needs T176–T177 harness)

| Freeze | Value |
|--------|--------|
| **Authoritative claim→test map** | `trackT175-sync-threat-model-adr/threat-model.md` **§7** |
| **Must-cover attack cases** | Unknown `device_id` pre-verify (L8); metadata-swap fail-closed (L5); content nonce-in-blob (L5); control cleartext parse (L5); forged ACK (L7); ACK cleartext signed (L7); replay idempotent (L5); revoke no future wrap + DeviceId permanent retirement (L4); gap buffer (L13) |
| **Wrap tests** | `T178-WRAP-per-recipient-roundtrip`; wrong-recipient fails; no shared DataKey over relay |
| **Non-claims** | Metadata-private; perfect multi-device deletion; NIST Purge/remote wipe; PQ resistance — assert honesty / docs |
| **Optionality** | Sync default off; capture works without sync (`T178-L12-capture-without-sync`) |
| **ACK model** | Signed **cleartext** control ACK; local projection states `pending\|acked\|failed\|unreachable`; timeout N default 3 cycles; ACK = peer attestation not wipe proof |

### Proposed test id index (from threat-model §7)

| Area | Example ids |
|------|-------------|
| L1–L2 | `T178-L1-local-only-default`, `T178-L1-relay-opaque`, `T178-L2-device-pub-only-relay` |
| L3–L5 | `T178-L3-enroll-fingerprint`, `T178-L5-sig-canonical-bytes`, `T178-L5-meta-swap-fails`, `T178-L5-wrap-list-tamper`, `T178-L5-content-nonce-in-blob`, `T178-L5-control-cleartext-parse`, `T178-L5-tamper-ct`, `T178-L5-replay-idempotent` |
| L4 retirement | `T178-L4-deviceid-permanently-retired` (with revoke suite) |
| L6–L9 | `T178-L6-no-lww-conflict`, `T178-L7-forged-ack-reject`, `T178-L7-ack-cleartext-signed`, `T178-L8-unknown-device-preverify`, `T178-L9-relay-no-decrypt` |
| L11–L14 | `T178-L11-partial-ce-ux`, `T178-L13-gap-buffer`, `T178-L14-pad-buckets` |
| Wrap | `T178-WRAP-per-recipient-roundtrip`, `T178-WRAP-wrong-recipient-fails`, `T178-WRAP-kat-info-aad-bytes`, `T178-WRAP-no-shared-datakey-over-relay` |
| Residual (R) | `T178-R-metadata-doc`, `T178-R-offline-ce-pending-ack`, `T178-R-ack-attestation-not-wipe`, `T178-R-revoke-past-still-open` |
| Non-claims (NC) | `T178-NC-metadata`, `T178-NC-partial-erase`, `T178-NC-ack-not-wipe-proof`, `T178-NC-no-purge-claim`, `T178-NC-no-pq-claim` |

Must-include residual/control/wrap ids (Codex R2 handoff): `T178-R-ack-attestation-not-wipe`, `T178-NC-ack-not-wipe-proof`, `T178-WRAP-kat-info-aad-bytes`, `T178-L5-control-cleartext-parse`, `T178-L5-content-nonce-in-blob`, `T178-L7-ack-cleartext-signed`, `T178-L4-deviceid-permanently-retired`.

Explicit defers (docs/claims/product fence) remain in threat-model §7 — do not invent executable tests for L15 multi-user absence or FIPS certification.

## Placeholder objective

Prove **privacy and integrity properties** of the sync protocol under the T175 threat model. Document residual **metadata leakage**. Keep sync **optional**.

## Master-plan security tests

| Test | Intent |
|------|--------|
| Relay cannot decrypt payload | Ciphertext opacity; no keys on relay |
| Tampered envelope rejected | AEAD/signature fail closed |
| Metadata-swapped envelope rejected | Signature covers metadata+ciphertext (L5) |
| Replayed envelope idempotent | No double side effects |
| Unknown device_id rejected pre-verify | Enrolled-set DoS gate (L8) |
| Revoked device excluded from new keys | Forward exclusion (L4) |
| Erasure reaches all enrolled devices | ACK status explicit (pending/failed/acked) |
| Forged ACK rejected | Signed ACK only (L7) |
| Metadata leakage documented | Sizes, counts, timing — honest residual risk |

## Phase acceptance (master plan)

- Convergence + privacy threat-model gates pass  
- Sync remains **optional**  
- **Local-only mode unchanged**  

## Expected deliverables (sketch)

| Item | Notes |
|------|--------|
| Security test module | nextest; deterministic keys |
| Negative tests | Wrong key, bit-flip, stale revocation race, forged ACK |
| Erasure multi-device fixture | May mark `__slow` |
| Threat residual doc section | Link from OPERATIONS / EVALUATION |
| Feature flag proof | Default off; capture works without sync crate features |

## Standards alignment

| Source | Application |
|--------|-------------|
| NIST-style CE (P8) | Erasure via key destroy must propagate; don’t claim physical wipe of peer disks offline forever without ACK model honesty |
| Untrusted server E2EE | Relay is storage+ordering helper only |
| Test hygiene | No real network; tempdir vaults; no live vault |

## License / commercial constraints

- Tests use in-tree crypto + fake relay only.  
- Do not require AGPL pen-test frameworks.  
- Optional `cargo audit` / deny remain green after any test-only deps (`tempfile` already present).  
- No uploading customer envelopes to third-party “security SaaS” as part of CI.

## Non-goals

- Formal verification / machine-checked proofs (nice later)  
- Production relay hardening checklist (separate ops track under P12)  
- Claiming compliance certifications  

## Expand before implement

- [x] ADR-0018 Accepted (T175 Complete 2026-07-30)
- [ ] T176–T177 harness  
- [ ] Map each security test to ADR claim (threat-model §7)  
- [ ] Revocation race cases  
- [ ] ACK timeout semantics (default N=3)  
- [ ] What “excluded from new keys” means operationally  
- [ ] Mark which tests need P8 erase  

## Definition of Done (when fleshed out)

All listed security properties tested or explicitly deferred with risk acceptance; metadata leakage documented; default build local-only; deny/audit green.

## Phase 11 acceptance rollup

- [x] T175 threat model + ADR reviewed (**Accepted** 2026-07-30; Codex R3 PASS)  
- [ ] T176 crate/schema (no plaintext)  
- [ ] T177 fake relay convergence  
- [ ] T178 security tests  
- [ ] Sync optional; local-only intact  
