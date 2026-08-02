# T187 Plan — SQLCipher Page Encryption (live)

Status: **Completed** (2026-08-02).
Spec: [spec.md](./spec.md) (F1–F22, AC1–AC13).  
Ledger TX (overshoot — abandon before real implement if still open): `dc9f932a-ca0b-40c0-8d42-dcf556501633`.

## Preconditions (before Phase A code)

- [x] Expand freezes F1–F22 + AI fold-in disposition  
- [ ] **Perl on PATH** (Strawberry Perl or equiv.) — required for `openssl-src` on Windows MSVC  
- [ ] Document NASM: required if OpenSSL builds with asm; note if build uses `no-asm`  
- [ ] `Docs/ci-tooling.md` + `scripts/dev-check.ps1` capability checks for Perl (and NASM if needed)  
- [ ] GHA `windows-2025`: verify Perl on PATH for gate job  
- [ ] Abandon accidental ledger TX if still pending; start fresh on implement  
- [ ] `ledgerful doctor` + `scan --impact` at implement start  
- [ ] `ledgerful ledger start T187-SqlCipherPageEncryption --category SECURITY`

## License / toolchain gate

- [ ] No AGPL  
- [ ] After feature flip: `cargo deny check` output archived in review log (SQLCipher community + OpenSSL SPDX)  
- [ ] `cargo audit` green  

---

## Phase A — Spike (build)

- [ ] **A1** Workspace `Cargo.toml`: `bundled-sqlcipher-vendored-openssl` + `backup` + `fallible_uint`  
- [ ] **A2** `cargo build -p ai-brains-store -p ai-brains-cli -p ai-brains-brain` on Windows MSVC **with Perl**  
- [ ] **A3** `cargo deny check` + `cargo audit`; paste deny into review log  
- [ ] **A4** Record `PRAGMA cipher_version` from a probe binary/test once build works  
- [ ] **A5** Note OPENSSL_NO_VENDOR / OPENSSL_CONFIG_DIR / OPENSSL_STATIC sensitivity for CI hermeticity  

**Exit:** clean compile Windows + (local or CI) Linux.

---

## Phase B — Open / backup / zero-key / migrate

### B0 Inventory

- [ ] **B0.1** Workspace unkeyed-open audit (spec §8): classify vault / backup / plain-exempt / test  
- [ ] **B0.2** List every site that needs key+verify or allow-zero-key  

### B1 Plain header + errors

- [ ] **B1.1** Shared `is_plain_sqlite_header(path)` in `ai-brains-store` (lift T181 helper)  
- [ ] **B1.2** `StoreError::LegacyPlaintextVault { migrate_hint }` with pinned message mentioning `vault encrypt`  
- [ ] **B1.3** Wire pre-key sniff in `VaultConnection::open` **and** `open_read_intent`  
- [ ] **B1.4** Wrong-key path remains `VaultLocked` after encrypted-looking header  

### B2 Keyed backup paths

- [ ] **B2.1** `run_backup`: `apply_key_pragmas` + verify on **source** before backup API  
- [ ] **B2.2** `has_core_tables` / list: **do not** ignore apply_key errors; surface wrong-key reason  
- [ ] **B2.3** Audit remaining backup.rs open sites  

### B3 Zero-key + validation

- [ ] **B3.1** `SqlCipherKey::is_zero()` + `validate()` (`^x'[0-9a-fA-F]{64}'$`)  
- [ ] **B3.2** Refuse zero/blank/invalid in `VaultConnection::open` / `open_read_intent` unless `AI_BRAINS_ALLOW_ZERO_KEY=1`  
- [ ] **B3.3** AppContext / daemon / shadow / migrate: no silent bypass of VaultConnection policy  

### B4 Plain→encrypted operator path (**sqlcipher_export**, not Online Backup)

- [ ] **B4.1** Implement F4 sequence (checkpoint → unkeyed open plain → ATTACH KEY → `sqlcipher_export` → DETACH)  
- [ ] **B4.2** CLI **`ai-brains vault encrypt`**: dry-run / `--confirm` for replace; never silent overwrite  
- [ ] **B4.3** Content smoke: post-encrypt open with key; header not plain  

### B5 Test hermetic zero-key

- [ ] **B5.1** Hermetic CLI helper sets `AI_BRAINS_ALLOW_ZERO_KEY=1`  
- [ ] **B5.2** Library tests: `TempEnv` RAII (not bare `set_var`)  
- [ ] **B5.3** Document env in OPERATIONS  

### B6 Scratch hygiene

- [ ] **B6.1** Delete or feature-gate `check_db.rs` (no hardcoded user path)  
- [ ] **B6.2** Remove/gate `scratch/check_vault.rs`  

### B7 PRAGMA key hygiene (soft)

- [ ] **B7.1** Prefer single-quoted key form matching Zetetic; comment why not bind params  
- [ ] **B7.2** Do not set `cipher_plaintext_header_size`  

---

## Phase C — Tests

- [ ] **C1** T187-H-01: new vault header not plain after write  
- [ ] **C2** T187-H-02 / elevate F-02/K-06: **strict** wrong-key; **delete** `if plain { … }` in:
  - `crates/ai-brains-cli/tests/recovery_drills.rs` (`backup_verify__wrong_key__wrong_key_class`)
  - `crates/ai-brains-store/tests/recovery_drills.rs` (K-06 dual-mode)  
- [ ] **C3** T187-P-01: plain open → `LegacyPlaintextVault` + migrate hint  
- [ ] **C4** T187-P-02: `vault encrypt` content smoke  
- [ ] **C5** T187-B-01: backup create/restore roundtrip under SQLCipher  
- [ ] **C6** T187-Z-01: zero-key refuse / escape hatch  
- [ ] **C7** T187-V-01: `PRAGMA cipher_version` non-empty  
- [ ] **C8** Secrets: existing `assert_no_secret_leakage` still green  

---

## Phase D — Docs / claims

- [ ] **D1** Deviations §1 → resolved by T187  
- [ ] **D2** COMPATIBILITY F8, SECURITY-LIMITS, RELEASE-CLAIMS R-F8/R-K06/R-ZERO-KEY, INSTALL, OPERATIONS, CAPABILITIES, ARCHITECTURE, RECOVERY-DRILLS  
- [ ] **D2b** Record exact `PRAGMA cipher_version` string; note `cipher_plaintext_header_size` unset  
- [ ] **D3** CHANGELOG  
- [ ] **D4** `Docs/ci-tooling.md` + `dev-check.ps1`: Perl (+ NASM if needed)  
- [ ] **D5** CLAIMS-CROSSCHECK touch if required by T185 process  

---

## Phase E — Closeout

- [ ] **E1** Full gate: fmt, clippy -D warnings, nextest workspace/profile ci, deny, audit  
- [ ] **E2** Internal review + Codex **SECURITY**  
- [ ] **E3** Strike deferred §59 #8; conductor ✅ Completed  
- [ ] **E4** Pin: `DECISION: T187 SQLCipher live via bundled-sqlcipher-vendored-openssl; plain→encrypt via sqlcipher_export (vault encrypt); wrong-key fail-closed; zero-key refuse unless AI_BRAINS_ALLOW_ZERO_KEY; not FIPS/Purge`  

---

## Manual evidence (at closeout)

| Check | Result |
|-------|--------|
| New vault header not plain | |
| Wrong `--key` fails | |
| Plain vault → migrate hint | |
| `vault encrypt` content smoke | |
| Backup create/restore | |
| `cipher_version` non-empty | |
| No secrets in CLI output | |
| deny output in review log | |

---

## Out of scope checklist

- [ ] DataKey rotation (T189)  
- [ ] recovery export / daemon restore hard-fail (T188)  
- [ ] #12 TOCTOU (T190)  
- [ ] `cipher_integrity_check` on backup verify (soft residual)  
- [ ] rusqlite 0.40 bump  
- [ ] FIPS / NIST Purge  

---

## AI fold-in plan deltas (vs pre-review plan)

| # | Delta |
|---|--------|
| 1 | B4: **sqlcipher_export** only (not Online Backup) for plain→encrypt |
| 2 | B2: workspace unkeyed-open audit + has_core_tables |
| 3 | B1: exact `LegacyPlaintextVault` + both open paths |
| 4 | B3: enforce zero-key at VaultConnection + TempEnv |
| 5 | C2: explicit delete of `if plain` branches |
| 6 | B3.1: `validate` / `is_zero` |
| 7 | C7: cipher_version smoke |
| 8 | B4.2: named `vault encrypt` |
| 9 | B6: scratch binary hygiene |
| 10 | Preconditions: Perl/NASM/CI |
| 11 | A3/F11: license re-verify |
