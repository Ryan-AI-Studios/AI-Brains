# T175 Review Log — Sync Protocol Threat Model + ADR

- **Track:** T175-SyncThreatModelAdr
- **Branch:** `feat/t175-sync-threat-model-adr`
- **Scope:** Design / docs only
- **ADR status:** **Accepted** — 2026-07-30 (after Codex R3 PASS)
- **Track status:** ✅ **Completed**

---

## R1 — Internal Review (NEEDS FIXES) — 2026-07-30

| ID | Severity | Finding | Disposition | Status |
|----|----------|---------|-------------|--------|
| **M1** | Medium | Enrollment fingerprint bound Ed25519 alone; X25519 not under OOB | Normative dual-key enrollment package + `fingerprint = SHA-256(package)`; mismatch → no DEK wrap; matrix `T178-L3-enroll-binds-x25519` | `verified_fixed` (R2) |
| **M2** | Medium | Signer of `DeviceEnrolled` / `DeviceRevoked` not frozen | Both signed by already-enrolled device after OOB; first device RecoveryKit-local only; self-enroll from unknown → L8 reject; matrix `T178-L3-enroll-signer-must-be-enrolled`, `T178-L4-revoke-signer-must-be-enrolled` | `verified_fixed` (R2) |
| **M3** | Medium | Wrap HKDF/AAD encoding not byte-level | ADR §17: salt empty; info/AAD length-prefix order frozen; T178 WRAP KATs required | `verified_fixed` (R2) |
| **L1** | Low | Bad cross-ref §12.1 for control stream | Point control record stream to **§22 Q1** | `verified_fixed` (R2) |
| **L2** | Low | `spec.md` status not aligned with Implementing | Status → **Implementing — pending security review** | `verified_fixed` (R2) |
| **L3** | Low | Re-enrollment after revoke underspecified | Full OOB again; **prefer new DeviceId**; tombstone old id | `verified_fixed` (R2) |
| **L4** | Low | Spec §12 open questions not linked to ADR defaults | Note: defaults live in ADR-0018 §22 pending Accept | `verified_fixed` (R2) |

### Fix anchors (implementer)

| ID | Primary anchors |
|----|-----------------|
| M1 | ADR-0018 §3 L3 enrollment package + fingerprint; threat-model §4.2/§4.4/§7.1 `T178-L3-enroll-binds-x25519`; spec L3 |
| M2 | ADR-0018 §3 L3 signer rules + §4 L4 + §8 L8 gate; §22 Q5; threat-model matrix L3/L4; spec L3/L4 |
| M3 | ADR-0018 §17.1–17.4; threat-model §7.4 `T178-WRAP-kat-info-aad-bytes`; spec §3.5.1 |
| L1 | ADR-0018 §3 L3 → §22 Q1 (was §12.1) |
| L2 | `spec.md` header Status line |
| L3 | ADR-0018 §3 re-enrollment; §4 after-revoke; spec L3/L4 |
| L4 | `spec.md` §12 note → ADR-0018 §22 |

---

## R2 — Internal re-review (CLEAN) — 2026-07-30

**Verdict:** CLEAN. All R1 findings `verified_fixed`. No new engineering defects. Process gates remain: ADR Accept, conductor Complete, ledger/git closeout.

---

## Codex R1 — Independent completion audit (FAIL) — 2026-07-30

**Source:** `conductor/tracks/trackT175-sync-threat-model-adr/review.codex.md`  
**Verdict:** **FAIL** (P0 none; P1 one; P2 one; P3 none). ADR remains **Proposed**; conductor not Complete; no git/ledger commit.

| ID | Severity | Finding | Disposition | Status |
|----|----------|---------|-------------|--------|
| **P1-001** | P1 | Outer envelope signature not byte-level / exhaustive — ADR §5 listed a partial “at least” field set without encoding, UUID layout, `event_id` / `content_key_id` / wrap-list binding, or outer-sig KATs | Freeze complete `signed_bytes` (BE ints, 16B UUIDs) including `event_id`, `content_key_id`, ciphertext len+bytes, wrap_count + sorted wrap records; wraps = outer signed metadata; control envelopes N=0 / zero content_key_id; T178 `T178-L5-sig-canonical-bytes`, `T178-L5-meta-swap-fails`, `T178-L5-wrap-list-tamper`; threat-model L5 + attack cases updated | `verified_fixed` (Codex R2: prior closed) |
| **P2-001** | P2 | Broken section references (ADR HPKE `§5.2`; spec HPKE `§3.5.1`; phantom `§3.6.8` / `§3.6.6` / `§5.7`) | ADR → §18; spec HPKE → §3.5.2; multi-device nonce → §3.6 item 8; 800-88r2 → L11 / §3.6 item 6; matrix → threat-model §7 | `verified_fixed` (Codex R2: prior closed) |

### Fix anchors (implementer — Codex R1)

| ID | Primary anchors |
|----|-----------------|
| P1-001 | ADR-0018 **§5 / §5.1–5.3** (`signed_bytes` freeze, wrap placement, T178 outer-sig ids); threat-model **§4.2** wrap-list STRIDE, **§4.4** attack cases, **§7.1 L5** matrix; spec **L5**; ADR §23 + test plan outline ids |
| P2-001 | ADR-0018 “What we are not” HPKE row → **§18**; `spec.md` crate table → **§3.5.2**; fold-in table B4/B11/B12 → **§3.6 item 8**, **§3.6 item 6 / L11**, **threat-model §7** |

### Explicit non-actions after this fix pass

- ADR-0018 Status still **Proposed** (not Accepted)
- Conductor track not marked Complete
- No git commit / no ledger commit
- No Cargo/Rust production code

---

## Codex R2 — Independent re-audit (FAIL) — 2026-07-30

**Source:** `conductor/tracks/trackT175-sync-threat-model-adr/review.codex.r2.md`  
**Verdict:** **FAIL** (P0 none; P1 two; P2 three; P3 none). Prior P1-001 / P2-001 closed. ADR remains **Proposed**; conductor not Complete; no git/ledger commit.

| ID | Severity | Finding | Disposition | Status |
|----|----------|---------|-------------|--------|
| **P1-002** | P1 | Control envelopes not decryptable when `N=0` + encrypted payload (no shared DataKey / no wraps) | Freeze **public signed cleartext control**: control set payloads not DEK-encrypted; wire field may stay `ciphertext` offset (prose `payload`); `wrap_count=0`; `content_key_id` zero except erasure/ACK target; peers verify sig+enrolled-set then parse; data path remains AEAD+wraps; matrix `T178-L5-control-cleartext-parse`, `T178-L7-ack-cleartext-signed` | `verified_fixed` (post-fix re-review CLEAN) |
| **P1-003** | P1 | Content AEAD nonce missing from wire freeze | Freeze data body = **`nonce(12) ‖ ciphertext ‖ tag(16)`** single opaque blob; `ciphertext_len` covers full blob; nonce covered by outer sig; no separate unsigned nonce; local store may split columns and convert at wire; KAT `T178-L5-content-nonce-in-blob` | `verified_fixed` (post-fix re-review CLEAN) |
| **P2-002** | P2 | ADR §17.3 “fails open” contradicts fail-closed | Wording → **fail-closed** for AEAD/sig/metadata swap everywhere (AAD mismatch rejects) | `verified_fixed` (post-fix re-review CLEAN) |
| **P2-003** | P2 | Re-enrollment contradiction (tombstone forever vs same-DeviceId reuse allowed) | **Policy freeze:** after `DeviceRevoked`, DeviceId **permanently retired**; same physical machine always **new DeviceId + new keys + full OOB**; matrix `T178-L4-deviceid-permanently-retired` | `verified_fixed` (post-fix re-review CLEAN) |
| **P2-004** | P2 | ACK authenticity residual underspecified | Document residual: signed ACK = enrolled peer **attestation** of apply/CE steps, not remote wipe / malware-free; compromised peer can false-ACK until revoke; UX not “proven wiped everywhere”; honest projection states; residual + non-claim + matrix notes | `verified_fixed` (post-fix re-review CLEAN) |

### Fix anchors (implementer — Codex R2)

| ID | Primary anchors |
|----|-----------------|
| P1-002 | ADR-0018 **§5 / §5.1.1 / §5.2** (control vs data table); **§5.4** tests `T178-L5-control-cleartext-parse`, `T178-L7-ack-cleartext-signed`; **§7** L7 cleartext ACK; threat-model **§4.2/§4.4/§7.1 L5/L7**; spec **L5/L7**; T178 index |
| P1-003 | ADR-0018 **§5.3** content AEAD nonce packing; **§5.2** body interpretation table; **§5.4** `T178-L5-content-nonce-in-blob`; threat-model L5 matrix + attack case; spec **L5** |
| P2-002 | ADR-0018 **§17.3** AAD “fail-closed”; **§5.1** metadata-swap fail-closed; **L8** / threat-model L8; spec L5/L8 |
| P2-003 | ADR-0018 **§3** re-enrollment permanent retirement; **§4** DeviceId retirement; threat-model L3/L4 + attack case; spec L3/L4; `T178-L4-deviceid-permanently-retired` |
| P2-004 | ADR-0018 **§7** ACK authenticity residual + **§24** non-claim; threat-model assets/actors/STRIDE residual/non-claim/matrix; spec L7 + §3.6; `T178-R-ack-attestation-not-wipe`, `T178-NC-ack-not-wipe-proof` |

### Explicit non-actions after this fix pass

- ADR-0018 Status still **Proposed** (not Accepted)
- Conductor track not marked Complete
- No git commit / no ledger commit
- No Cargo/Rust production code

---

## Post-fix re-review (CLEAN) — 2026-07-30

**Verdict:** CLEAN. All Codex R2 findings (P1-002, P1-003, P2-002, P2-003, P2-004) are **`verified_fixed`**. Easy lows from prior rounds already closed; no new engineering defects on the control-cleartext / nonce-in-blob / DeviceId retirement / ACK residual freezes.

**Easy lows note:** R1 L1–L4 and other low-severity wording/cross-ref items remain `verified_fixed` (R2); no open easy lows from Codex R2 scope.

Process gates remain (no Accept / no commit in this pass):

- ADR-0018 Status still **Proposed**
- Conductor track not Complete
- No git / ledger closeout

---

### Not done (by design / gate) — pre-Accept

- [x] ADR-0018 Status → **Accepted** (2026-07-30 after Codex R3 PASS)
- [x] Conductor track marked Complete
- [ ] Ledger commit on doc closeout — **orchestrator** (tx open; closeout agent does not commit)

---

## Codex R3 — Independent re-audit (PASS) — 2026-07-30

**Source:** `conductor/tracks/trackT175-sync-threat-model-adr/review.codex.r3.md`  
**Verdict:** **PASS**. No new P0–P3 findings. Prior findings closed (R1 P1-001/P2-001; R2 P1-002/P1-003/P2-002/P2-003/P2-004). DoD content satisfied. Procedural Accept / conductor Complete / ledger closeout deferred to this closeout pass + orchestrator.

### Gate disposition

| Gate | Outcome |
|------|---------|
| Engineering Accept of ADR-0018 | **Cleared** — Status **Accepted** 2026-07-30 |
| Security design review | **Codex R3 PASS** (fresh clean Codex) after R1 FAIL→fix + R2 FAIL→fix |
| T176 hard-block on Accept | **Lifted** — T176 **Proposed / Unblocked** (not implementing in T175) |
| Production sync / Cargo | Still none (correct for docs track) |
| Ledger / git commit | **Orchestrator** |

---

## Final review-round summary

| Round | Kind | Verdict | Outcome |
|-------|------|---------|---------|
| R1 | Internal | NEEDS FIXES | M1–M3 + L1–L4 fixed → `verified_fixed` at R2 |
| R2 | Internal re-review | CLEAN | All R1 findings closed |
| Codex R1 | Independent audit | **FAIL** | P1-001 (signed_bytes), P2-001 (broken refs) → fixed |
| Codex R2 | Independent re-audit | **FAIL** | P1-002/P1-003 (control cleartext, nonce packing); P2-002/P2-003/P2-004 (fail-closed, DeviceId retirement, ACK residual) → fixed; post-fix CLEAN |
| Codex R3 | Independent re-audit | **PASS** | No new findings; Accept + T175 Complete |

### Closure rule

Code/doc change alone is not closure. Implementer marks `fixed_pending_verification`; reviewer marks `verified_fixed`. Loop continues until clean. **T175 design review loop closed at Codex R3 PASS.**
