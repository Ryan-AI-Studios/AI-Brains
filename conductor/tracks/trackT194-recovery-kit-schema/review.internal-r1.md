# Track Completion Audit — T194

**Track:** T194 — Recovery Kit Schema Hygiene (Argon2 params / kit forward-compat)  
**Auditor:** Internal completion review (read-only r1)  
**Date:** 2026-08-02  
**Branch:** `track/T194-recovery-kit-schema`  
**Scope:** Spec F1–F31, AC1–AC10, plan B/C, crypto library + docs honesty

## Verdict: PASS WITH DEFERRED P3

Implementation meets every frozen decision F1–F31 and acceptance criteria AC1–AC8 / AC10 in code and tests. AC9 process gate (full workspace fmt/clippy/nextest/deny/audit + cross-model SECURITY review + ledger commit) is not fully closed in plan checkboxes; that is ship-process residual, not a code defect. No P0–P2 findings.

---

## Requirement and DoD Matrix

### Frozen decisions F1–F31

| ID | Requirement (short) | Status | Evidence |
|----|---------------------|--------|----------|
| **F1** | Self-describing wraps close F37 / brick risk | **Met** | `PassphraseWrappedKey.kdf` + dual-read |
| **F2** | KDF on `PassphraseWrappedKey`, not root/DPAPI | **Met** | `key_wrap.rs` field; DPAPI struct unchanged |
| **F3** | Nested JSON field `kdf`; absent ⇒ legacy | **Met** | `#[serde(default)] Option<KdfParams>` |
| **F4** | Fields: algorithm, version, m_cost, t_cost, p_cost | **Met** | `KdfParams` struct; no aliases |
| **F5** | Wire `"argon2id"` only; unknown fail closed | **Met** | `validate_for_unlock`; test `rejects_unknown_algorithm` |
| **F6** | version 19 = 0x13; unknown fail closed | **Met** | `LEGACY_VERSION` + `Version::V0x13`; test wrong version |
| **F7** | Generation m=19456,t=2,p=1 unchanged | **Met** | `product_generation()` → same as legacy constants |
| **F8** | No `Argon2::default()` on recovery path | **Met** | `Argon2::new(...)` only; structural test scans production |
| **F9** | Missing kdf ⇒ fixed legacy, not crate Default | **Met** | `KdfParams::legacy()` constants; dual-read tests |
| **F9b** | `legacy() -> Self` (not `const` String) | **Met** | `key_wrap.rs` `legacy()` + numeric consts |
| **F10** | `schema_version` stays 1 | **Met** | generate pins 1; comments; tests |
| **F11** | Always serialize kdf on generate; no `skip_serializing_if` | **Met** | `kdf: Some(kdf)` in generate; attribute absent on field |
| **F12** | `serde(default)` + `deny_unknown_fields` on `KdfParams` | **Met** | Both present on type/field |
| **F13** | effective = stored or legacy → validate → derive | **Met** | `unlock_with_passphrase` + `argon2_from_params` |
| **F14** | DoS caps, zeros, min m≥8p, alg/version | **Met** | `validate_for_unlock` full checks |
| **F15** | API (a): public `KdfParams`; params threaded | **Met** | `derive_key`/`wrap_key`/`unwrap_key` take `&KdfParams`; generate stamps `Some` |
| **F16** | No new CLI flags | **Met** | Export/rotate/doctor inherit library |
| **F17** | Zero new deps; argon2 0.5.x; no password-hash | **Met** | workspace `argon2 = "0.5"`; lock 0.5.3; no feature enable |
| **F18** | Capture independence | **Met** | Crypto-only change set |
| **F19** | kdf public params; no secrets on stdout | **Met** | Existing no-leak tests remain; kdf is non-secret |
| **F20** | Invert T181-K-07 | **Met** | Presence+values in unit + integration crypto_recovery |
| **F21** | Partial kdf object fail closed | **Met** | Unit + integration deserialize fail tests |
| **F22** | Docs honesty | **Met** | RECOVERY-DRILLS, ADR-0020, RELEASE-CLAIMS, CAPABILITIES, deferred, CHANGELOG |
| **F23** | No contracts/events change | **Met** | No DTO/event kit material changes |
| **F24** | SECURITY review category | **In progress** | This internal r1; cross-model still plan D3 |
| **F25** | Determinism / KAT practical | **Met** | Non-default params KAT (F29) |
| **F26** | Tampered m_cost fail closed | **Met** | `passphrase__unwrap__tampered_m_cost__fails_closed` |
| **F27** | DPAPI arm unchanged | **Met** | No KDF on DPAPI path |
| **F28** | Forced re-export out of scope | **Met** | Dual-read only; no re-export campaign |
| **F29** | Non-default stored-params proof | **Met** | m=12288,t=3,p=1 unlock OK; LEGACY fails same wrap |
| **F30** | No output_len on wire; always 32 | **Met** | Comment + `KDF_OUTPUT_LEN = 32` |
| **F31** | generate constructs `PassphraseWrappedKey` with kdf | **Met** | Sole production site updated |

### Acceptance criteria AC1–AC10

| AC | Criterion | Status | Proof |
|----|-----------|--------|-------|
| **AC1** | Generate JSON embeds kdf product tuple | **Met** | `recovery_kit__generate__embeds_kdf_params` (unit + crypto_recovery) |
| **AC2** | Non-default stored params used; LEGACY fails | **Met** | `recovery_kit__unlock__non_default_kdf_params__uses_stored_not_legacy` — **not** default-only roundtrip |
| **AC3** | Legacy without kdf (+ optional schema_version) unlocks | **Met** | dual legacy JSON tests |
| **AC4** | No `Argon2::default()` in production passphrase path | **Met** | code + structural test |
| **AC5** | Oversize m / unknown alg / partial kdf fail closed | **Met** | unit rejects + partial deserialize tests |
| **AC6** | K-07 inverted; export/rotate unlock inheritance | **Met** | inverted tests; CLI recovery/vault unlock via `RecoveryKit::generate` |
| **AC7** | Docs F37 struck; claims honesty | **Met** | see Completeness Sweep docs |
| **AC8** | Zero unexpected deps | **Met** | argon2 0.5.3 hold; no new prod deps in crypto crate |
| **AC9** | Full gate + SECURITY review | **Partial** | Library/tests complete; plan D1/D3 process residual (this r1 is internal SECURITY pass) |
| **AC10** | Doctor kit path new+legacy | **Met** | Doctor uses `from_json` + `unlock_with_passphrase` only; dual-read is pure library; existing doctor kit tests export new kits |

---

## Findings

_No P0, P1, or P2 findings._

### [P3-1] Optional edge validation tests not expanded beyond AC5 core

- **Confidence:** High  
- **Requirement:** F12 / F14 (coverage completeness, not unmet DoD)  
- **Location:** `crates/ai-brains-crypto/src/key_wrap.rs` (`validate_for_unlock`, `deny_unknown_fields`); tests in `passphrase.rs` / `recovery_kit.rs`  
- **Problem:** Caps for `t_cost`/`p_cost`, zero costs, min `m≥8p`, and `deny_unknown_fields` are implemented but lack dedicated unit tests (only m_cost over-cap, unknown algorithm, wrong version, partial missing `m_cost` covered).  
- **Evidence:** Grep shows only `rejects_m_cost_over_cap` among cap tests; no test injects extra kdf JSON field.  
- **Correction:** Optional follow-up tests: unknown field inside `kdf`, `t_cost=33`, `p_cost=0`, `m_cost < 8*p_cost`.  
- **Deferrable:** Yes (P3) — attributes/logic present; AC5 core cases proven.

### [P3-2] Ship-process checklist incomplete (not a code defect)

- **Confidence:** High  
- **Requirement:** AC9 / plan C4, D1–D4  
- **Location:** `conductor/tracks/trackT194-recovery-kit-schema/plan.md`; `conductor/conductor.md` T194 still In Progress  
- **Problem:** Full workspace gate, manual export inspect, cross-model review, ledger commit, pin DECISION, conductor status→Completed remain unchecked.  
- **Evidence:** Plan Phase C4/D open; conductor.md “In Progress”.  
- **Correction:** Run full gate; complete SECURITY cross-model if required by AGENTS; ledger commit; pin; mark Completed.  
- **Deferrable:** Yes as process residual after this code-complete PASS — **not** a product crypto defect.

### [P3-3] CLI export/rotate tests do not assert kdf wire presence

- **Confidence:** High  
- **Requirement:** Plan C1 optional; AC1 library-bound  
- **Location:** `crates/ai-brains-cli/tests/recovery_drills.rs`, vault rotate kit unlock tests  
- **Problem:** Integration unlocks kits but does not assert `"kdf"` / m_cost in exported file (plan marks assert optional).  
- **Evidence:** Export tests parse + unlock only.  
- **Correction:** Optional one-liner assert on exported JSON.  
- **Deferrable:** Yes — AC1 covered at library.

---

## Completeness Sweep

| Check | Result |
|-------|--------|
| TODO/FIXME/stubs in crypto recovery path | **None** in `key_wrap` / `passphrase` / `recovery_kit` for T194 |
| `Argon2::default` remaining | **Absent** from production `passphrase.rs` (only comments mention Default forbidding it) |
| `skip_serializing_if` on `kdf` | **Absent** (comment forbids; field has only `#[serde(default)]`) |
| `unwrap`/`expect` in production crypto path | **None** in `passphrase.rs` / `key_wrap.rs` / `recovery_kit.rs` production bodies (only tests) |
| `schema_version` still 1 | **Yes** |
| Dep bumps | **None** — `argon2` workspace `"0.5"`, lock **0.5.3** |
| Docs honesty | RECOVERY-DRILLS §6 rewritten; K-07 inverted; residual F37 struck; ADR-0020 struck; RELEASE-CLAIMS R-34.2 updated; CAPABILITIES one-liner; `conductor/deferred.md` closed by T194; CHANGELOG Security entry |
| `InvalidKdfParams` | Present on `CryptoError` |
| Re-export | `lib.rs` re-exports `KdfParams` |

---

## Wiring and Regression Review

```
generate:
  KdfParams::product_generation()
    → passphrase::wrap_key(..., &kdf)
        → derive_key → argon2_from_params (validate + Argon2::new)
    → PassphraseWrappedKey { ..., kdf: Some(kdf) }
    → schema_version: 1

to_json → from_json:
  kdf present → Some(KdfParams) [deny_unknown_fields on object]
  kdf absent → None via serde default

unlock_with_passphrase:
  effective = kdf.clone().unwrap_or_else(KdfParams::legacy)
    → passphrase::unwrap_key(..., &effective)
        → validate_for_unlock → Argon2::new → AEAD decrypt
        → InvalidPassphrase on AEAD fail (wrong passphrase or wrong params)

Legacy path:
  pre-T194 JSON without kdf → None → legacy() {argon2id,19,19456,2,1}
  equals current product_generation → strip-kdf after generate still unlocks

Non-default proof (AC2/F29):
  wrap with m=12288,t=3,p=1 + stamp kdf
  unlock_with_passphrase OK
  unwrap with KdfParams::legacy() → InvalidPassphrase
```

Call-site blast radius: sole production `passphrase::wrap_key` / `unwrap_key` consumers are `recovery_kit` (CLI export, rotate, doctor all go through `RecoveryKit`). Signature threading is complete. DPAPI path untouched.

Regression risk low: additive optional field under schema v1; dual-read preserves old kits forever under F9.

---

## Verification Evidence

| Source | Result |
|--------|--------|
| Spec/plan freeze alignment | Code matches F1–F31 freezes (API a, legacy(), F29 mandatory) |
| Unit tests (`recovery_kit.rs`, `passphrase.rs`) | Embeds kdf; F29; legacy dual-read; partial kdf; rejects alg/m/version; tampered m_cost; no Default structural |
| Integration (`tests/crypto_recovery.rs`) | K-07 inverted; F29; legacy; partial; roundtrip |
| Structural no-Default | `passphrase__no_argon2_default_in_production_path` scans production half of file |
| Orchestrator claim | Package nextest green (plan B8: 69 passed) — not re-run in this read-only pass |
| Full workspace gate (fmt/clippy/nextest/deny/audit) | **Not evidenced** in track artifacts (P3-2) |

---

## Deferred Candidates

| ID | Item | Severity | Rationale |
|----|------|----------|-----------|
| P3-1 | Extra validation unit tests (t/p caps, zero, deny_unknown) | P3 | Logic present; AC5 core covered |
| P3-2 | Full gate + cross-model + ledger commit + pin + conductor Completed | P3 | Ship process; code complete |
| P3-3 | Optional CLI kdf wire assert | P3 | Library AC1 sufficient |
| Out of scope (spec) | Stronger product m/t; PQ/HPKE; forced re-export; schema v2; argon2 0.6; PHC string; typed algorithm enum; lower interactive caps | n/a | Explicit non-goals |

---

## Completion Decision

**Code / DoD for T194 crypto hygiene: COMPLETE.**

- F37 residual correctly closed in code and honesty docs.  
- AC2/F29 non-default KAT is present and correctly distinguishes stored params from LEGACY (default generate roundtrip alone is not relied upon).  
- No production `Argon2::default()`, no `skip_serializing_if` on kdf, no unwrap/expect on production crypto path, schema_version remains 1, no dep bumps.

**Verdict: PASS WITH DEFERRED P3** — defer P3-1/P3-2/P3-3 under AGENTS.md low/process rules. Implementer may proceed to full gate, cross-model SECURITY review if required, ledger commit, pin DECISION, and mark track Completed.

**Not cleared for silent ship without:** plan D1 full gate green + ledger/process closeout (tracked as P3-2, not a code FAIL).
