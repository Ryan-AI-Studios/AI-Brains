# T194 Plan — Recovery Kit Schema Hygiene

Status: **Implementation complete** (2026-08-02). Spec: [spec.md](./spec.md).  
Ship closeout (C4/D4 Completed + pin + ledger commit) after PR merge.

## Preconditions

- [x] Read T188 F19/F22 + RecoveryKit / PassphraseWrappedKey / passphrase.rs live code
- [x] Research argon2 0.5.3 Params API + OWASP 2026 dual-read guidance
- [x] Expand freezes (schema fields, dual-read, DoS caps, no schema_version bump)
- [x] AI fold-in (AI1 affirm + AI2 M1/L1–L6) — disposition spec §15
- [x] `ledgerful ledger start T194-RecoveryKitSchema --category SECURITY` *(TX e8844831-decb-43c8-8850-e338dea1ba26)*
- [ ] Pin decision after implement: `ai-brains pin "DECISION: T194 — RecoveryKit passphrase.kdf pins Argon2id m/t/p/version; legacy dual-read via KdfParams::legacy(); no Argon2::default(); F29 non-default params test"` 

## Deferred rolled in

| Item | Disposition |
|------|-------------|
| **F37** Argon2 params not in kit JSON | **Absorb** — core of track; strike on ship |
| RECOVERY-DRILLS §6 “document only” residual | **Absorb** — rewrite on ship |
| T188 F22 / T189 F24 honesty | **Absorb** — docs + code |
| T181-K-07 “lacks kdf fields” test | **Invert** in Phase B |
| Stronger Argon2 product profile | **Not absorbed** |
| Forced re-export of offline kits | **Not absorbed** |
| Typed algorithm enum (O1) | **Not absorbed** (String + validate) |
| Lower interactive DoS caps (O2) | **Not absorbed** |
| T193 path / T195 multi-user / T196 units | **Not absorbed** |

## Phases

### Phase A — Design freeze (plan-only ✅)

- [x] **A1** Schema fields + version policy (`kdf` on `PassphraseWrappedKey`; `schema_version` stays 1)
- [x] **A2** Legacy dual-read = `KdfParams::legacy()` (not crate Default) — **F9/F9b**
- [x] **A3** DoS caps on unlock (m≤1GiB, t≤32, p≤16) + algorithm/version checks — **F14**
- [x] **A4** Generation strength unchanged (19456/2/1)
- [x] **A5** Zero new deps; hold argon2 0.5.3
- [x] **A6** API approach **(a):** `derive_key(..., &KdfParams)` threaded through wrap/unwrap — **F15**
- [x] **A7** Mandatory non-default params test (AC2/F29) — AI2 M1
- [x] **A8** ADR-style freezes in spec §4–5 (optional ADR-0022 not required)

### Phase B — Crypto library TDD (implement)

- [x] **B0** Ledger start SECURITY
- [x] **B1 Red:** `recovery_kit__generate__embeds_kdf_params` (fails today)
- [x] **B2 Red:** `recovery_kit__legacy_json_without_kdf__unlocks_with_legacy_defaults`
- [x] **B2b Red (F29/AC2 mandatory):** `recovery_kit__unlock__non_default_kdf_params__uses_stored_not_legacy`  
  - Wrap with **m=12288, t=3, p=1** (within caps); unlock OK with those params  
  - Assert unlock with LEGACY (19456/2/1) **fails** on the same ciphertext  
  - *Why mandatory:* F7 generation == F9 legacy; default roundtrip cannot prove stored-params path*
- [x] **B3 Red:** reject unknown algorithm / over-cap m_cost / wrong version
- [x] **B4 Green:** `KdfParams` + `KdfParams::legacy()` / `product_generation()` + `PassphraseWrappedKey.kdf: Option<KdfParams>` with `#[serde(default)]` (no `skip_serializing_if`)
- [x] **B5 Green:** Signatures — `derive_key(..., &KdfParams)`; `wrap_key` / `unwrap_key` take `&KdfParams` (or wrap embeds params); `Argon2::new(Algorithm::Argon2id, Version::V0x13, params)`; **remove** `Argon2::default()`
- [x] **B6** Invert T181-K-07 → presence + value asserts
- [x] **B7** Existing recovery_kit / crypto_recovery roundtrips green
- [x] **B8** Targeted: `cargo nextest run -p ai-brains-crypto` + clippy package (**74** passed after edge-test P3 fix)

### Phase C — CLI inheritance + docs

- [x] **C1** Confirm export + rotate still unlock (existing tests); optional assert kdf in exported file
- [x] **C2** Doctor kit path regression (existing doctor tests / minimal fixture) if cheap
- [x] **C3** Docs (exact residual sites from AI2):  
  - `Docs/RECOVERY-DRILLS.md` §6 (~116–120), residual row ~178, T181-K-07 row ~35  
  - `Docs/DECISIONS/ADR-0020-datakey-rotation.md` ~145–146  
  - `Docs/RELEASE-CLAIMS.md` R-34.2 ~154  
  - `Docs/CAPABILITIES.md` ~257  
  - `conductor/deferred.md` F37 rows  
- [ ] **C4** `conductor.md` status → Completed on ship

### Phase D — Gate + review

- [x] **D1** Full gate: fmt OK; clippy workspace -D warnings OK; nextest workspace **1841** passed (1 skipped); deny ok; audit allowed warnings only; ledgerful verify fast (orchestrator)
- [x] **D2** Manual/library: generate embeds kdf 19456/2/1; unlock; strip-kdf legacy unlock; F29 non-default KAT (embeds_kdf + legacy_json_without_kdf + non_default tests)
- [x] **D3** SECURITY review: internal r1 PASS WITH DEFERRED P3 (easy P3 edge tests fixed); Codex r1 process FAIL → fixed metadata; Codex final re-review before ship
- [ ] **D4** Ledger commit; pin DECISION; strike deferred (on ship)

## Verification matrix

| AC | Proof |
|----|-------|
| AC1 embed | Unit generate JSON |
| **AC2 stored params** | **Mandatory** F29 non-default KAT (m=12288,t=3,p=1); LEGACY fail on same wrap — **not** optional |
| AC3 legacy | Fixture omit kdf |
| AC4 no Default | Code review + no call site in passphrase.rs |
| AC5 fail closed | Unit caps / algorithm / version |
| AC6 invert K-07 + CLI green | Tests |
| AC7 docs | Diff review (sites above) |
| AC8 deps | deny/audit |
| AC9 gate + review | Full gate + review.md |
| AC10 doctor | Existing or light process test |

## Out of scope checklist

- [ ] PQ / HPKE
- [ ] Forced re-export of all existing kits
- [ ] Raising default m_cost for new kits
- [ ] schema_version=2
- [ ] argon2 0.6 RC bump
- [ ] PHC string as kit body
- [ ] New doctor CLI flags
- [ ] Typed algorithm enum (O1)
- [ ] Lower interactive DoS caps (O2)

## Risk notes for implementer

1. **Production construction site:** only `recovery_kit::generate` builds `PassphraseWrappedKey` today (AI2 L4) — update that site; struct-literal blast radius is small.
2. **PartialEq/Eq** on RecoveryKit remains; include kdf in equality.
3. **DoS caps:** generation constants must stay ≤ caps (they do). argon2 0.5.x is single-threaded; high `p_cost` only inflates layout (F14 note).
4. **Windows DPAPI** tests unaffected.
5. Prefer `thiserror` variant for invalid KDF if it improves messaging; else map to existing Encryption/Decryption without unwrap/expect.
6. **AC2 trap:** do not ship without B2b — default generate roundtrip cannot prove F13/F15.

## AI fold-in summary

| Item | Action |
|------|--------|
| M1 non-default test | **Folded** — F29, AC2, B2b, verification matrix |
| L1 legacy String | **Folded** — F9b `KdfParams::legacy()` |
| L2 API (a) | **Folded** — F15 |
| L3 sketch cleanup | **Folded** — spec §5.2 |
| L4–L6 notes | **Folded** — F30/F31/F14 note |
| O1/O2 | **Declined** for this track |
