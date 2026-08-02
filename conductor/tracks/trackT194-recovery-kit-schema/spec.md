# T194 — Recovery Kit Schema Hygiene (Argon2 params / kit forward-compat)

- **Track ID:** T194-RecoveryKitSchema
- **Phase:** Post-T188 / post-T189 crypto hygiene
- **Status:** ✅ **Completed** (2026-08-02) PR #76 `2c06464`
- **Depends on (hard):** T188 RecoveryKit `schema_version=1` + export CLI; T189 rotation kit re-export; `passphrase::{wrap,unwrap}_key`; `PassphraseWrappedKey`
- **Depends on (soft):** T181 RECOVERY-DRILLS residual F37; T192 doctor kit unlock (serde dual-read only — no doctor feature work)
- **Blocks / feeds:** Closes T181/T188/T189 **F37** soft residual; kits survive future `argon2` crate default changes and future product param bumps
- **Category:** SECURITY / CRYPTO
- **Deferred absorbed:** Argon2 KDF params not in kit JSON (**F37**); soft “document only” residual across T181/T188/T189; RECOVERY-DRILLS §6 residual row; ADR-0020 residual line; RELEASE-CLAIMS R-34.2 Argon2 residual phrase
- **Not absorbed:** Raising generation strength as product policy (separate decision); PQ / HPKE; forced re-export of all operator offline kits; PHC password-hash string as sole storage format; `password-hash` crate feature enablement (not required); typed algorithm enum (O1 optional); interactive lower DoS caps (O2 future)
- **Research date:** 2026-08-02
- **AI fold-in:** AI1 §1–4 (affirm) + AI2 **M1**, **L1–L6**. Disposition §15.
- **Ledger:** TX `e8844831-decb-43c8-8850-e338dea1ba26` (SECURITY) — commit on ship

## 1. Objective

Pin **Argon2id KDF parameters** used for passphrase wrap of DataKey into RecoveryKit JSON so that:

1. **Unlock always uses the params that produced the wrap** (stored or documented legacy defaults).
2. Future `argon2` crate `Default` changes (or intentional product param bumps) **cannot silently brick** existing kits.
3. New kits are **self-describing**; old kits remain unlockable via **fixed dual-read defaults** (not live crate defaults).

## 2. Live baseline (re-scan 2026-08-02)

| Asset | Today |
|-------|--------|
| `RecoveryKit` | `schema_version: u32` default **1** (T188 F19); `dpapi` + `passphrase` |
| `PassphraseWrappedKey` | `ciphertext`, `salt` [16], `nonce` [12] only — **no KDF fields** |
| `passphrase::derive_key` | `Argon2::default()` on both wrap and unwrap |
| Workspace pin | `argon2 = "0.5"` → resolved **0.5.3** (crates.io latest stable; 0.6 only RC) |
| Crate defaults (0.5.3) | `Params::DEFAULT`: **m=19456**, **t=2**, **p=1**, output 32; algorithm **Argon2id**; version **0x13** (19) |
| OWASP Password Storage (2026) | Argon2id base mins include **m=19456, t=2, p=1** (matches crate default) |
| Export / rotate | Both call `RecoveryKit::generate` → automatic once library pins params |
| T181-K-07 | Asserts kit JSON **lacks** KDF field names (`m_cost`, `kdf`, `argon2id`, …) — **invert on ship** |
| Doctor `--kit-path` | `RecoveryKit::from_json` + unlock — benefits from dual-read; no new CLI flags |
| Docs residual | RECOVERY-DRILLS §6 / residual table; CAPABILITIES; ADR-0020 F24/F37 line |

## 3. Research summary (2026-08-02)

| Source | Finding | T194 application |
|--------|---------|------------------|
| [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) | Argon2id; store algorithm + work factors with the secret; dual-read during transitions; PHC string format for *password hashes* | Store **m/t/p + algorithm + version** with wrap; dual-read legacy kits; **do not** adopt PHC as sole kit format (we wrap AEAD ciphertext, not store a PHC hash) |
| docs.rs `argon2` **0.5.3** | `Params::new(m,t,p,output_len)`, `Argon2::new(Algorithm, Version, Params)`, `hash_password_into`; `DEFAULT_M_COST` / `DEFAULT_T_COST=2` / `DEFAULT_P_COST=1` | Use explicit `Params` + `Algorithm::Argon2id` + `Version::V0x13` — **never** `Argon2::default()` on recovery path after T194 |
| crates.io argon2 | Stable **0.5.3**; 0.6 only RC | **Hold 0.5.x**; zero dep bumps required |
| PHC / industry | Params must be stored to re-derive | Nested JSON object on wrap (not free-floating PHC string) |
| Kit threat model | Operator-supplied kit file is **untrusted input** until unlock succeeds | Cap max m/t/p on **read** to bound DoS/OOM from malicious kits |

## 4. Frozen decisions (F1–F28)

| ID | Decision |
|----|----------|
| **F1 — Problem** | Kits that depend on live `Argon2::default()` brick if crate/product defaults change. F37 closes by **self-describing wraps**. |
| **F2 — Placement** | KDF metadata lives on **`PassphraseWrappedKey`** (the wrap that uses Argon2), not on RecoveryKit root and not on DPAPI arm. |
| **F3 — Field object** | Optional nested object field name: **`kdf`** (JSON). Shape frozen in §5. Absent `kdf` ⇒ legacy dual-read. |
| **F4 — Field names** | Inside `kdf`: `algorithm` (string), `version` (u32 decimal PHC-style **19** for 0x13), `m_cost` (u32 KiB blocks), `t_cost` (u32), `p_cost` (u32). **No** alternate aliases (`memory_cost`, `parallelism`) on wire. |
| **F5 — Algorithm** | Wire value **`"argon2id"`** only for generation. Deserialize unknown algorithm → **fail closed** (`CryptoError` with actionable message, no panic). |
| **F6 — Version** | Wire `version: 19` (= Argon2 **0x13**). Map via `Version::V0x13`. Unknown/unsupported version → fail closed. |
| **F7 — Generation params (product)** | **Unchanged strength:** new kits generate with **m=19456, t=2, p=1**, Argon2id, v19. Do **not** raise OWASP profile (46 MiB / 64 MiB) in this track. |
| **F8 — No `Argon2::default()`** | After T194, recovery wrap/unwrap **must** build `Argon2::new(alg, ver, params)` from **`KdfParams`** (stored or legacy constants). Unit test or structural review forbids calling `Argon2::default()` in `passphrase.rs`. |
| **F9 — Legacy dual-read** | Kits **without** `kdf` unlock with **fixed** product historical values {argon2id, 19, 19456, 2, 1} — **not** `Params::default()` / crate Default (so crate default drift cannot brick pre-T194 kits). |
| **F9b — Legacy construction (AI2 L1)** | `KdfParams` holds `algorithm: String` (wire). **Do not** use `const LEGACY_KDF_PARAMS: KdfParams` (String is not const). Freeze: **`KdfParams::legacy() -> Self`** (or free `fn legacy_kdf_params() -> KdfParams`) that returns owned `"argon2id".into()` + numeric fields. Optional: `const` for numerics only (`LEGACY_M_COST` etc.). |
| **F10 — schema_version** | **Stay at 1.** Optional `kdf` is additive; old kits already default `schema_version=1`. Do **not** bump to 2 solely for kdf presence (avoids false “incompatible” signal; mirrors T188 additive field style). Future incompatible wire shape may bump later under a separate track. |
| **F11 — Serialize always** | `RecoveryKit::generate` / `passphrase::wrap_key` **always** populate and serialize `kdf` on new kits. **`skip_serializing_if` forbidden** for `kdf` on generate path. Deserialize: `#[serde(default)]` only. |
| **F12 — Deserialize** | `#[serde(default)]` on `kdf: Option<KdfParams>` so missing field → `None` → F9 path. Extra unknown fields inside `kdf` → **deny** (`deny_unknown_fields` on `KdfParams`) to catch typos fail-closed. |
| **F13 — Unlock path** | `unlock_with_passphrase` / `unwrap_key` resolve effective params = `kdf.clone().unwrap_or_else(KdfParams::legacy)` then validate (F14) then derive. |
| **F14 — Read-side validation / DoS bounds** | Before KDF run on unlock: reject if `m_cost < 8*p_cost` (argon2 min) or outside product caps: **`m_cost ≤ 1_048_576`** (1 GiB), **`t_cost ≤ 32`**, **`p_cost ≤ 16`**. Reject zero. **`algorithm` must be `"argon2id"`** (case-sensitive); **`version` must be 19**. Actionable error (no panic). Generation uses validated product constants well inside caps. Note (AI2 L5): argon2 **0.5.x** is single-threaded (rayon removed); `p_cost` affects memory layout only — caps still apply. |
| **F15 — API surface (AI2 L2)** | **Frozen approach (a):** public `KdfParams`; `derive_key(passphrase, salt, output, params: &KdfParams)`; thread `params` through `wrap_key` / `unwrap_key` (signatures gain `&KdfParams` or embed result in returned `PassphraseWrappedKey`). **Reject (b)** “only RecoveryKit resolves params” as the sole path — params flow must be explicit at the KDF boundary. `wrap_key` always stamps `kdf: Some(params_used)` on the wrapped struct. |
| **F16 — CLI surfaces** | **No new CLI flags.** `recovery export` and `vault rotate-datakey` inherit via `RecoveryKit::generate`. Doctor kit path inherits via `from_json` + unlock. |
| **F17 — Deps** | **Zero new production deps.** Hold `argon2` **0.5.x** (resolved 0.5.3). Do not enable `password-hash` feature solely for this track. No PHC string dependency. |
| **F18 — Capture independence** | Crypto-only; no models/graph. |
| **F19 — Secrets** | `kdf` fields are **public parameters** (not secrets). Still forbid DataKey / ciphertext / passphrase on stdout. Salt/nonce remain public kit material. |
| **F20 — Invert T181-K-07** | Replace “lacks KDF fields” with: new kits **contain** `kdf` with expected values; plus separate legacy fixture test that omits `kdf` and still unlocks. |
| **F21 — Partial kdf object** | Missing required subfield (`m_cost` etc.) → deserialize error fail-closed (no partial defaults mixed with present object). Only **entirely absent** `kdf` uses LEGACY. |
| **F22 — Docs honesty** | On ship: RECOVERY-DRILLS §6 rewritten (params pinned); residual table strike F37; CAPABILITIES one-liner; ADR-0020 residual strike/update; RELEASE-CLAIMS R-34.2 remove “Argon2 not in kit JSON”. |
| **F23 — Contracts** | Kit JSON is file schema (not daemon DTO). No `ai-brains-contracts` change unless a DTO already embeds RecoveryKit (none today). Events unchanged (`RecoveryKitCreated` stays `{key_id}` only — never kit material). |
| **F24 — Review category** | SECURITY cross-model review required (crypto schema). |
| **F25 — Determinism** | Same key+passphrase+salt+nonce+params → same derived key. Prefer fixture tests with fixed salt/nonce for KAT-style assert where practical; generate path may keep random salt/nonce. |
| **F26 — Wrong params** | Unlock with wrong stored params (tampered `m_cost`) → `InvalidPassphrase` or AEAD fail (same class as wrong passphrase) — fail closed, no key leak. |
| **F27 — DPAPI arm** | Unchanged; no KDF on DPAPI path. |
| **F28 — Forced re-export** | Out of scope. Operators may re-export anytime; pre-T194 kits remain valid forever under F9. |
| **F29 — Non-default params proof (AI2 M1)** | **Mandatory** test proves unlock reads **stored** params, not LEGACY alone. Product generation (F7) equals LEGACY (F9), so default-params roundtrip **cannot** distinguish them. Fixture must wrap with non-default-but-valid params (recommended OWASP-equivalent within caps: **m=12288, t=3, p=1**, argon2id, v19), unlock successfully with those params, and **fail** if force-unlocked under LEGACY (19456/2/1). Name: `recovery_kit__unlock__non_default_kdf_params__uses_stored_not_legacy` (or equivalent). |
| **F30 — output_len (AI2 L6)** | `KdfParams` omits `output_len`; always **32** (DataKey / AES-256). Document in code comment. Changing DataKey size is out of scope / future residual. |
| **F31 — Struct literal surface (AI2 L4)** | Live tree: sole production `PassphraseWrappedKey { ... }` construction is `recovery_kit::generate`. Test risk is low; still update generate + any future literals for the new field. |

## 5. Wire schema (frozen)

### 5.1 New kit example (illustrative)

```json
{
  "schema_version": 1,
  "dpapi": { "ciphertext": [/* bytes */] },
  "passphrase": {
    "ciphertext": [/* bytes */],
    "salt": [/* 16 bytes */],
    "nonce": [/* 12 bytes */],
    "kdf": {
      "algorithm": "argon2id",
      "version": 19,
      "m_cost": 19456,
      "t_cost": 2,
      "p_cost": 1
    }
  }
}
```

### 5.2 Rust sketch (implement may adjust module placement)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct KdfParams {
    pub algorithm: String, // wire: "argon2id"
    pub version: u32,      // 19
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl KdfParams {
    /// Historical pre-T194 / product-generation defaults (F7/F9). Not crate Default.
    pub fn legacy() -> Self {
        Self {
            algorithm: "argon2id".into(),
            version: 19,
            m_cost: 19456,
            t_cost: 2,
            p_cost: 1,
        }
    }

    /// Product generation params (F7) — currently equal to legacy(); kept named for future strength bumps.
    pub fn product_generation() -> Self {
        Self::legacy()
    }
}

// PassphraseWrappedKey gains:
//   #[serde(default)]
//   pub kdf: Option<KdfParams>,
// generate always sets Some(KdfParams::product_generation()) — never skip_serializing_if (F11)
```

### 5.3 Effective params algorithm

```
fn effective_kdf(wrapped: &PassphraseWrappedKey) -> Result<KdfParams> {
  match &wrapped.kdf {
    None => Ok(KdfParams::legacy()),
    Some(p) => { validate_for_unlock(p)?; Ok(p.clone()) }
  }
}
```

## 6. Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | New `RecoveryKit::generate` JSON contains `passphrase.kdf` with algorithm=`argon2id`, version=19, m=19456, t=2, p=1 |
| **AC2** | **Mandatory (F29):** non-default-but-valid stored params (e.g. m=12288,t=3,p=1) unlock successfully; same ciphertext **fails** under LEGACY (19456/2/1). Default-params generate roundtrip alone is **insufficient** for AC2. |
| **AC3** | Legacy kit JSON **without** `kdf` (and with/without `schema_version`) still unlocks under LEGACY constants |
| **AC4** | `Argon2::default()` absent from production recovery path (`passphrase.rs`) |
| **AC5** | Malicious oversized `m_cost` / unknown algorithm / partial `kdf` → fail closed with non-panic error |
| **AC6** | T181-K-07 inverted; export + rotate integration still unlock kits (library or CLI smoke) |
| **AC7** | Docs residual F37 struck; RELEASE-CLAIMS / ADR-0020 honesty updated |
| **AC8** | Zero unexpected deps; deny/audit green |
| **AC9** | Full gate + SECURITY review clean (or deferred P3 only under AGENTS.md rules) |
| **AC10** | Doctor `--kit-path` still works on new and legacy kits (no doctor code required if pure library) |

## 7. Non-goals

| Item | Why |
|------|-----|
| Stronger default m/t (e.g. 64 MiB) | Product policy track; not F37 |
| PQ / HPKE | Future crypto |
| Forced operator re-export campaign | Soft residual remains “old kits work via dual-read” |
| PHC string as kit body | Wrong abstraction for AEAD-wrapped DataKey |
| Kit format on stdout | Still forbidden |
| Changing `schema_version` to 2 | Additive optional field under v1 (F10) |
| Doctor feature work | T192 already shipped |
| Typed algorithm enum (O1) | String + F5 validate is frozen; enum is optional polish |
| Lower interactive DoS caps (O2) | Unlock is rare/file-based; 1 GiB F14 is enough for T194 |

## 8. Threat / risk notes

| Risk | Mitigation |
|------|------------|
| Crate default drift bricks kits | F8 + F9 fixed constants |
| Malicious kit OOM (huge m) | F14 caps |
| Typos in field names silent | F12 deny_unknown_fields + F21 |
| Old binary + new stronger params later | Out of scope for T194 generation (F7); if future track raises params, old binaries cannot unlock new kits — document residual |
| Secrets in kdf object | Params are public (F19) |
| `PassphraseWrappedKey` test literals | Low risk (AI2 L4): sole production site is `generate`; update that site |

## 9. Test plan (TDD)

### Red-first library (`ai-brains-crypto`)

| Test | Expect |
|------|--------|
| `recovery_kit__generate__embeds_kdf_params` | JSON has kdf object + exact values (19456/2/1) |
| `recovery_kit__unlock__generate_roundtrip` | Default generate → unlock (smoke; **not** sufficient for AC2) |
| `recovery_kit__unlock__non_default_kdf_params__uses_stored_not_legacy` | **Mandatory AC2/F29:** wrap with m=12288,t=3,p=1; unlock OK; LEGACY params fail on same wrap |
| `recovery_kit__legacy_json_without_kdf__unlocks_with_legacy_defaults` | Omit kdf; unlock OK |
| `recovery_kit__legacy_json_without_schema_and_kdf__unlocks` | Both optional fields missing |
| `passphrase__derive__rejects_unknown_algorithm` | Fail closed |
| `passphrase__derive__rejects_m_cost_over_cap` | Fail closed |
| `passphrase__derive__rejects_wrong_version` | version ≠ 19 fail closed |
| `passphrase__unwrap__tampered_m_cost__fails_closed` | Wrong params → not original key |
| `passphrase__no_argon2_default_in_production_path` | Optional structural: no `Argon2::default` in passphrase.rs |

### Invert / update existing

| Test | Change |
|------|--------|
| `recovery_kit__json__lacks_kdf_param_fields` (T181-K-07) | **Replace** with presence + value asserts |
| Schema version tests | Still pin `schema_version=1`; add kdf coexistence |

### Integration (CLI / drills)

| Test | Expect |
|------|--------|
| Existing `recovery export` drill unlock | Still green; optionally assert kdf in file |
| Existing `vault rotate-datakey` kit unlock | Still green |
| Doctor kit path (if cheap) | New + legacy fixture |

## 10. Docs / claims / deferred on ship

| File | Action |
|------|--------|
| `Docs/RECOVERY-DRILLS.md` §6 + residual table | Rewrite: params pinned in kit; strike F37 residual |
| `Docs/CAPABILITIES.md` | Note kit embeds Argon2id params |
| `Docs/DECISIONS/ADR-0020-datakey-rotation.md` | Strike F37 residual line |
| `Docs/RELEASE-CLAIMS.md` | Remove Argon2-not-in-kit from R-34.2 residuals |
| `conductor/deferred.md` | Strike F37 → closed by T194 |
| `conductor/conductor.md` | T194 Completed when done |

## 11. Handoffs

| To | What |
|----|------|
| F37 residual | **Close** on ship |
| Future strength bump track | May raise generation params + still dual-read legacy + stored |
| T192 doctor | No change expected; regression if serde breaks |
| T193–T196 | Unrelated |

## 12. Implementation touch list (expected)

| Path | Change |
|------|--------|
| `crates/ai-brains-crypto/src/key_wrap.rs` | `KdfParams` + field on `PassphraseWrappedKey` |
| `crates/ai-brains-crypto/src/passphrase.rs` | Explicit Params; validate; no Default |
| `crates/ai-brains-crypto/src/recovery_kit.rs` | Ensure generate fills kdf; tests |
| `crates/ai-brains-crypto/src/lib.rs` | Re-export `KdfParams` if public |
| `crates/ai-brains-crypto/src/errors.rs` | Optional typed invalid-kdf variant (or map to Encryption/Decryption) |
| `crates/ai-brains-crypto/tests/crypto_recovery.rs` | Invert K-07; legacy unlock |
| CLI recovery/vault tests | Green by inheritance; optional kdf assert |
| Docs listed in §10 | Honesty |

## 13. Manual test (implement gate)

1. Export kit → open JSON → confirm `kdf` present with frozen values.  
2. Unlock exported kit with passphrase (CLI or library).  
3. Hand-craft legacy-shaped JSON (strip `kdf`) from a known wrap — unlock succeeds.  
4. Confirm no kit JSON / key material on stdout during export.

## 14. Stop-before

- Changing generation strength (m/t) beyond frozen tuple without user decision.  
- Bumping `argon2` to 0.6 RC.  
- Adding `password-hash` feature solely for PHC formatting.  
- Implementing without TDD red on K-07 invert **and** without **F29 non-default params** test (AC2).

## 15. AI fold-in disposition (2026-08-02)

Sources: `C:\dev\AI-review.md` — **AI1** (design recap + actionable table) + **AI2** (M1, L1–L6, O1–O2).

| ID | Reviewer claim | Disposition | Spec action |
|----|----------------|------------|-------------|
| AI1 §1–4 | Self-describing kdf; no Default; DoS caps; invert K-07; zero-touch CLI | **Accepted** (already frozen F2–F20, F16) | Affirmed; no change to intent |
| AI1 table 1–5 | Touch list key_wrap / passphrase / tests | **Accepted** | Matches §12 |
| **AI2 M1** | AC2 non-default KAT was optional but must be mandatory — default roundtrip cannot prove stored-params path | **Accepted** | **F29**; AC2 wording; §9 mandatory test; plan B2b; verification matrix |
| **AI2 L1** | `const LEGACY` cannot hold `String` | **Accepted** | **F9b** `KdfParams::legacy()` |
| **AI2 L2** | Freeze API: (a) thread params vs (b) kit-only resolve | **Accepted (a)** | **F15** |
| **AI2 L3** | Sketch had confusing crossed-out serde lines | **Accepted** | §5.2 cleaned |
| **AI2 L4** | Struct-literal risk overstated (only generate) | **Accepted** | **F31** + risk note softens |
| **AI2 L5** | p_cost single-threaded in 0.5.x | **Accepted as note** | F14 note only |
| **AI2 L6** | output_len omitted | **Accepted** | **F30** |
| AI2 O1 | Typed algorithm enum | **Declined** for freeze | String + F5; optional polish residual |
| AI2 O2 | Lower interactive caps | **Declined** for T194 | Future-only |

**Verdict after fold-in:** Ship-ready for implement **go** once freezes above are followed (M1 resolved in plan docs).
