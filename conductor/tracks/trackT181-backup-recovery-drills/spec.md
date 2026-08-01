# T181 — Backup, Restore, Recovery-Kit Drills (P12.3)

- **Track ID:** T181-BackupRecoveryDrills
- **Phase:** P12 — Release hardening and adoption (Task 3)
- **Status:** ✅ **Completed** (2026-08-01)
- **Depends on:** Backup CLI suite (T76, T99, T104, T109, T116–T117, T119–T121, T123, T126, T131, T134, T138); RecoveryKit crypto (T04-era); P8 content envelopes **Completed** (T162–T166); CE wipe path (T165); T179 multi-OS matrix (Windows T1); soft: T186 hermetic helpers when landed
- **Blocks / feeds:** T183 ops/recovery docs; T185 claims honesty (pre-erase residual + no perfect-deletion); release gate evidence that operators can recover
- **Category:** SECURITY / RELEASE / TESTING
- **Deferred absorbed:** §34 pre-erase backup residual (productize honesty via drills); failure-drills.md F-REC-01/02 (+ expand); Implementation-Plan §10.2 recovery drills; T178 residual “pre-erase backups” formal honesty; NIST SP 800-88r2 non-claim language; T186 handoff for hermetic assert_cmd reuse. **Not** #34.2 DataKey rotation; **not** full `doctor` product; **not** multi-device CE orchestration; **not** offsite backup SaaS.
- **Research date:** 2026-08-01 (online + in-tree baseline)
- **Review fold-in:** AI1 BS1–3 + Opp1–2; AI2 F-1..F-11 + O-1..O-8 → **F33–F48** + matrix/plan amendments. See §15.

## 1. Objective

Prove operators can **backup, restore, and recover vault keys** under realistic failure modes — including **content-envelope (CE)** vaults — without magic, and document **honest limits** (pre-erase backups remain recoverable offline; not NIST Purge/Destroy).

After T181:

| Capability | Present |
|------------|---------|
| `Docs/RECOVERY-DRILLS.md` (playbook + matrix + honesty) | Yes |
| Automated vault backup → restore → content smoke | Yes |
| RecoveryKit passphrase path (lose DPAPI → unlock) | Yes (**library** + tests; Windows DPAPI optional arm) |
| Kit→SqlCipherKey primitive chain (K-05) | Yes (**library integration**, not operator CLI workflow) |
| Envelope-aware drills (pre-erase residual + post-wipe backup) | Yes |
| Failure injection (corrupt file, wrong key, missing path) | Yes (substring-class asserts) |
| Secrets never printed in CI logs | Yes (helper + policy) |
| Full `ai-brains doctor` / `recovery export` CLI product | **No** (residual → future / T183; playbook documents manual kit export gap) |
| Offsite 3-2-1 automation / SaaS agent | **No** |
| Formal RTO/RPO SLA product claims | **No** (operator guidance only) |
| Argon2 KDF params persisted in kit JSON | **No** (residual; document generation-time defaults) |

## 2. Live baseline (re-scan 2026-08-01 + fold-in)

### 2.1 Backup / restore product surface

| Asset | Live state | Track origin |
|-------|------------|--------------|
| `BackupService` (`ai-brains-brain`) | SQLite Online Backup API via `rusqlite::backup`; key pragmas; `PRAGMA integrity_check` on create; `_aibrains_backup_meta` (T109) | T99, T109 |
| CLI `backup create` | Default `--keep 10`; `--dry-run`; retention sentinel | T119, T126 |
| CLI `backup restore` | Integrity first; `--dry-run` no-write; `--force` skips prompt; daemon-running **warn** (not hard fail); DROP `_aibrains_backup_meta` on live vault | T76, T99, T109 |
| CLI `backup list` / `verify` / `prune` | Full suite; verify FAIL reasons (wrong key vs corrupt) | T104, T131, T138 |
| Error shapes | Stringly `Box<dyn Error>` (e.g. `Integrity check failed:`, `Backup file not found:`) — **not** typed enums; drills use **substring class** matches | baseline |
| SQLCipher | Encrypted-to-encrypted only (Zetetic Online Backup API constraint) | baseline |
| Workspace deps | `rusqlite` 0.39 `features = ["bundled","backup",…]`; `aes-gcm` 0.10; `argon2` 0.5; `zeroize` 1.8; `hex` 0.4 | Cargo.toml |
| `dunce` | brain-crate only; `source_vault_path` may show `/private/var` on macOS after canonicalize | T179 residual |

### 2.2 Recovery kit

| Asset | Live state |
|-------|------------|
| `RecoveryKit` (`ai-brains-crypto`) | DPAPI wrap (optional) + passphrase wrap; `unlock_with_dpapi` / `unlock_with_passphrase`; JSON ser/de |
| `PassphraseWrappedKey` | Fields: `ciphertext`, `salt`, `nonce` only — **no** Argon2 `m_cost`/`t_cost`/`p_cost`/algorithm fields |
| KDF | `Argon2::default()` at wrap **and** unwrap (argon2 0.5.x defaults); kit unlock assumes **generation-time** defaults |
| `SqlCipherKey` | `from_data_key` → `x'HEX'`; **`Zeroize` + `ZeroizeOnDrop`** already |
| Unit / integration tests | `crypto_recovery.rs` + module tests: roundtrip, wrong passphrase, missing DPAPI, no plaintext hex in JSON |
| **Production CLI create/export kit** | **Absent** — zero call sites in `ai-brains-cli` commands |
| `ai-brains doctor` | **Absent** (T180 PROTOCOL-COMPAT: do not invent) |
| Event | `RecoveryKitCreated` payload exists (`key_id` only — no kit ciphertext, no secrets) |

### 2.3 Content envelopes (P8 — landed)

| Asset | Live state |
|-------|------------|
| Seal/open + wrap under DataKey | T164 |
| Wipe = destroy wrap + purge FTS/embeddings | T165 |
| **In-process wipe API for drills** | `content_envelope::destroy_content_key_wrap` + `wal_checkpoint_truncate` (store layer; test-only direct call OK) |
| Class retention prefers CE | T166 |
| Side stores | **Tables inside vault SQLite** (`content_key_store`, encrypted blobs per mig 0026) — **not** separate files; Online Backup API copies them with the main DB |
| Honesty | ADR-0016 §12; OPERATIONS erasure section; **pre-erase backups residual** |

### 2.4 Existing automated coverage (elevate-first)

| Test / drill sketch | Location | Status |
|---------------------|----------|--------|
| create dry-run / list / verify / prune | `ai-brains-cli/tests/smoke.rs` (many `backup_*`) | Elevate → T181-B-* / R-* |
| `test_backup_restore_dry_run` | smoke.rs | **Rename** drop `test_` prefix; elevate R-02 |
| `test_backup_restore_force_skips_prompt` | smoke.rs | **Rename** + **strengthen content smoke** or **supersede by R-01** |
| RecoveryKit roundtrip / wrong passphrase / missing DPAPI | `ai-brains-crypto/tests/crypto_recovery.rs` | Elevate → T181-K-* |
| CE seal/open/destroy | store `content_envelope_crypto.rs` | Reuse helpers for E-drills |
| failure-drills.md F-REC-01..04 | `conductor/failure-drills.md` | Manual matrix; T181 automates F-REC-01 + kit path; F-REC-03/04 soft residual |

### 2.5 Gaps T181 closes

1. **No automated full roundtrip** that asserts restored vault returns seeded content.  
2. **No envelope-aware drill** proving pre-wipe residual vs post-wipe CE hold.  
3. **No productized failure-injection suite** with stable drill IDs + **substring-class** errors.  
4. **No operator playbook** consolidating backup + kit honesty + 3-2-1 + **manual kit-export residual**.  
5. **RecoveryKit ↔ vault open** primitive chain not tied (library only).  
6. **Secrets in logs** — policy + negative assertions not explicit across kit drills.  
7. **Argon2 param opacity** undocumented as residual.

## 3. Research summary (online + standards, 2026-08-01)

### 3.1 Recovery planning & drills

| Source | Finding | T181 application |
|--------|---------|------------------|
| **NIST SP 800-184** | Plan before the event; playbooks; validate with exercises | `Docs/RECOVERY-DRILLS.md` + automated drills |
| Industry DR testing | Prove **restore**, not “backup exists”; RTO/RPO then test | R-01 content smoke; no SLA claims |
| **3-2-1** | Operator copies / offsite | Guidance only |

### 3.2 Media sanitization honesty

| Source | Finding | T181 application |
|--------|---------|------------------|
| **NIST SP 800-88r2** | Clear / Purge / Destroy | **Must not** claim Purge/Destroy |
| ADR-0016 §12 | RustCrypto not FIPS-validated; pre-erase residual | Normative for E-drills + docs |

### 3.3 SQLCipher / SQLite backup

| Source | Finding | T181 application |
|--------|---------|------------------|
| Online Backup API | Consistent snapshot; encrypted↔encrypted only | Product path only for backup/restore |
| rusqlite 0.39 `backup` | Hold pin | Zero new deps |

### 3.4 Secrets in tests / CI

| Source | Finding | T181 application |
|--------|---------|------------------|
| OWASP Secrets Management | Never log secrets; CI retains dumps | Helper `assert_no_secret_leakage`; no kit JSON / passphrase in stdout/stderr |

### 3.5 Dependency posture

| Crate / tool | Action |
|--------------|--------|
| `rusqlite` 0.39 + `backup` | **Hold** |
| `aes-gcm` 0.10 / `argon2` 0.5 / `zeroize` / `hex` | **Hold** |
| New **production** deps | **Forbidden** |
| New **dev** deps | Prefer zero; reuse `assert_cmd`, `tempfile`, `predicates`, `rstest` |
| Lightweight Argon2 test params | **Declined** — no production API for reduced cost; reuse pre-generated kits if suite is slow |

## 4. Frozen design decisions (F1–F48)

| ID | Decision |
|----|----------|
| **F1** | **Elevate-first:** map existing smoke + crypto_recovery tests to T181 ids; write only gap-fill drills. |
| **F2** | **Product path only for vault backup/restore** (CLI or `BackupService`) — never raw `fs::copy` of live WAL vault. **Does not** apply to CE wipe step (F35). |
| **F3** | **Full roundtrip DoD:** seed → backup → force-restore → assert seeded content observable. |
| **F4** | **RecoveryKit DoD is library-level.** No CLI path creates or exports a kit today. K-drills prove crypto primitives, **not** a full operator CLI workflow. |
| **F5** | **No new `doctor` / `recovery export` CLI as DoD.** RECOVERY-DRILLS **must** document manual kit-export residual (operator responsibility at vault init). |
| **F6** | **Envelope drill A (pre-erase residual):** seal → backup → in-process wipe → restore pre-wipe backup → content opens. |
| **F7** | **Envelope drill B (post-wipe backup):** seal → wipe → backup → restore → open fails (wrap absent). |
| **F8** | **Separate layers:** SQLCipher vault encryption ≠ content DEK CE. Side stores are **tables in vault DB** (mig 0026), copied by Online Backup API — not separate files. |
| **F9** | **Honesty language:** not NIST Purge/Destroy; not perfect deletion; pre-erase residual; ticket/soft forget ≠ CE. |
| **F10** | **Failure injection:** non-zero exit + **required substring class** (see §5); pin empirically for wrong-key before freezing exact SQLCipher wording. |
| **F11** | **Daemon-running restore:** elevate existing warn; hard-fail residual — not DoD. |
| **F12** | **Secrets:** never print passphrase, raw DataKey, DPAPI blob, ContentDek, **or kit JSON / wrapped ciphertext** (decryptable with passphrase). |
| **F13** | **Hermetic:** tempdir vaults; explicit project/session env or `--no-project-context`; no ambient developer `.env`. |
| **F14** | **Platform:** Windows T1 full suite incl. DPAPI arm; Linux/macOS passphrase + backup/restore + envelope. Soft skip DPAPI off-Windows. |
| **F15** | **RTO/RPO:** example operator targets only — not product SLA. |
| **F16** | **3-2-1:** operator guidance only. |
| **F17** | **Zero new production deps;** prefer zero new dev-deps. |
| **F18** | **Drill IDs** `T181-R-*` / `K-*` / `E-*` / `F-*` / `D-*`. |
| **F19** | **Fixture home:** crate-local only. |
| **F20** | **Meta table:** restore DROP `_aibrains_backup_meta` from live vault; R-01 **asserts** meta present in backup file, **absent** post-restore live. |
| **F21** | **Corrupt injection:** mutate backup after create; optional header (offset 0) and body (offset ≥100) cases via `rstest`. |
| **F22** | **Wrong key:** different SQLCipher key → actionable fail; no plaintext fallback. |
| **F23** | **F-REC-03/04** soft residual. |
| **F24** | **Capture independence:** no models/graph required for pass. |
| **F25** | **PolyForm NC + deny allowlist.** |
| **F26** | **Evidence** without secrets. |
| **F27** | **Multi-device sync restore** out of scope. |
| **F28** | **#34.2 DataKey rotation** out of scope. |
| **F29** | **Scheduler auto-backup:** document only if present. |
| **F30** | **Exit codes:** existing conventions; document observed. |
| **F31** | **Naming:** `function_or_feature__condition__expected_result`; **rename** legacy `test_backup_restore_*` on elevate (drop `test_`). |
| **F32** | **deferred §59** closeout on implement. |
| **F33** | **K-05 key path:** MUST use production `SqlCipherKey::from_data_key` (`ZeroizeOnDrop`). Drill code MUST NOT hand-build bare hex for PRAGMA key. Intermediate hex in `from_data_key` is existing production hygiene (soft residual to tighten further — not a new runtime assert). |
| **F34** | **Secret leakage helper:** test-only `assert_no_secret_leakage(output, secret_bytes)` checking **hex**, **base64**, and **raw UTF-8/byte display** forms; also assert no `kit.to_json()` / wrapped ciphertext in CLI output. Prefer workspace `hex` (already present). |
| **F35** | **CE wipe for E-drills:** in-process `content_envelope::destroy_content_key_wrap` + `wal_checkpoint_truncate` (store). Not daemon CLI. F2 does not forbid this. |
| **F36** | **E-drill tier:** measure; if wall &gt;60s default, mark `__slow` with owner + reason (AGENTS.md). Prefer tiny fixtures first. |
| **F37** | **Argon2 residual:** document that `PassphraseWrappedKey` stores **no** KDF params; unlock assumes generation-time `Argon2::default()`. Future schema may pin params — **not** T181 implement. K-assert: kit JSON lacks kdf param fields. |
| **F38** | **K-05 scope honesty:** library integration test of DataKey → kit → unlock → `SqlCipherKey` → open vault/backup. **Not** proof that an operator has exported a kit in production. |
| **F39** | **Force-restore elevate:** either strengthen elevated force test with content smoke **or** explicitly supersede by R-01 and delete redundant assertion-only-prompt test. |
| **F40** | **Failure matrix parameterization:** prefer `rstest` `#[case]` for F-01/F-02/R-03 (no for-loop in one `#[test]`). |
| **F41** | **K-05 sibling (K-06):** correct passphrase unlock then open with **wrong** `SqlCipherKey` → fails (kit→vault binding). |
| **F42** | **macOS meta path note** in RECOVERY-DRILLS: `source_vault_path` may show `/private/var` vs `/var` after canonicalize — expected. |
| **F43** | **RECOVERY-DRILLS checklist:** “At vault initialization — generate and store RecoveryKit JSON out-of-band (today library-only; future: `recovery export`). Without this, lost-DPAPI machine cannot passphrase-recover.” |
| **F44** | **ADR-0016 §4 wrap-nonce residual** cross-link in playbook honesty section. |
| **F45** | **No lightweight test Argon2 cost** as product change; if kit-heavy suite is slow, reuse precomputed kit fixtures or mark `__slow` — do not fork weaker KDF in prod code. |
| **F46** | **Substring classes (normative defaults; pin empirically if SQLCipher wording differs):** see §5.1. |
| **F47** | **Verification commands** include `--test recovery_drills` and store CE tests as touched. |
| **F48** | **AI fold-in disposition** recorded in §15; declined items stay declined. |

## 5. Drill matrix (normative)

| Drill ID | Scenario | Expected | Auto | Maps |
|----------|----------|----------|------|------|
| **T181-R-01** | Seed → backup → force restore → content smoke; meta present in backup file; meta **absent** live post-restore | Seeded content present; F20 locked | Yes | F-REC-01 |
| **T181-R-02** | Restore `--dry-run` | No dest mutation; integrity ok | Elevate + rename | T76 |
| **T181-R-03** | Restore missing path | Non-zero; substring **not found** class | Yes (`rstest`) | — |
| **T181-K-01** | Kit generate → passphrase unlock | Key equals original | Elevate | F-REC-02 |
| **T181-K-02** | `dpapi=None` → DPAPI unlock fails | `RecoveryKitMissing` | Elevate | — |
| **T181-K-03** | Wrong passphrase | Fail closed | Elevate | — |
| **T181-K-04** | Kit JSON: no raw/hex/base64 plaintext key; no accidental Debug dumps in CLI | Negative + helper | Elevate | OWASP |
| **T181-K-05** | **Library:** unlock kit → `SqlCipherKey::from_data_key` → open vault/backup | Roundtrip | **New** | F-REC-02 primitive |
| **T181-K-06** | Correct unlock + **wrong** SqlCipherKey open | Fails | **New** | F41 |
| **T181-K-07** | Kit JSON has no KDF param fields (`m_cost` etc.) | Assert absence | **New** soft | F37 |
| **T181-E-01** | Pre-erase residual (F6) via destroy_wrap + checkpoint | Pre-wipe backup opens CE content | **New** (maybe `__slow`) | ADR-0016 |
| **T181-E-02** | Post-wipe backup (F7) | Cannot open wiped content | **New** (maybe `__slow`) | T165 |
| **T181-F-01** | Corrupt backup (header and/or body case) | Fail + corruption-class substring | **New** (`rstest`) | T138 |
| **T181-F-02** | Wrong SQLCipher key on restore/verify | Fail + wrong-key class substring | **New** (`rstest`) | — |
| **T181-F-03** | Daemon running during restore | Warn (elevate); no hard-fail claim | Soft | T99 |
| **T181-D-01** | RECOVERY-DRILLS.md + OPERATIONS link + kit-export residual + honesty | Docs | Docs | SP 800-184 |
| **T181-D-02** | failure-drills.md F-REC → T181 | Sync IDs | Docs | conductor |

### 5.1 Failure substring classes (F46)

Pin exact observed strings during implement if product wording differs; **classes** are normative:

| Drill | Must match **at least one** (case-insensitive OK if documented) |
|-------|------------------------------------------------------------------|
| **R-03** missing path | `not found` / `Backup file not found` |
| **F-01** corrupt | `integrity` / `corrupt` / `not a database` / `query failed` / `Integrity check failed` |
| **F-02** wrong key | Empirical SQLCipher/rusqlite open error — often `not a database` / file open failure; **capture once, pin in test** |

Non-zero exit alone is **insufficient** (must not pass on bare panic without message class).

## 6. Acceptance criteria (when implementing)

| AC | Criterion |
|----|-----------|
| **AC1** | T181-R-01 green on Windows (T1); Linux core green without DPAPI |
| **AC2** | T181-K-01..K-07 green (K-05/K-06 library chain) |
| **AC3** | T181-E-01 and T181-E-02 green (store preferred) |
| **AC4** | T181-F-01, T181-F-02, R-03 green with substring classes |
| **AC5** | Secret leakage helper used; no passphrase/key/kit JSON in CLI outputs under test |
| **AC6** | `Docs/RECOVERY-DRILLS.md` shipped with kit-export residual, CE honesty, Argon2 residual, macOS path note, wrap-nonce link; OPERATIONS link |
| **AC7** | Zero new production deps; deny + audit green |
| **AC8** | Full gate green; default tier &lt;60s or documented `__slow` |
| **AC9** | Internal review clean; Codex cross-model for SECURITY |
| **AC10** | deferred §59 Completed; residuals listed (incl. Argon2 opacity, no recovery export/doctor) |
| **AC11** | Legacy `test_backup_restore_*` renamed; force-restore content covered (R-01 and/or strengthened elevate) |

## 7. Non-goals

| Out of scope | Owner |
|--------------|--------|
| `ai-brains doctor` product | Future / T183 |
| `ai-brains recovery export` full CLI | Soft residual (not DoD) |
| Persist Argon2 params in kit schema | Future hygiene residual |
| Offsite 3-2-1 automation | Never (operator) |
| NIST Purge/Destroy / FIPS module | Honesty only |
| #34.2 DataKey rotation | Open residual |
| Multi-device CE / relay restore | T176–T178 |
| Projection/graph rebuild F-REC-03/04 | Soft residual |
| Hard-fail restore when daemon running | Residual product change |
| Lightweight test-only Argon2 costs in prod API | Declined F45 |
| Runtime memory-scan assert of zeroize | Declined (use ZeroizeOnDrop path) |

## 8. License / commercial constraints

- In-tree crypto only.  
- No AGPL backup agents.  
- Do not upload vault fixtures or recovery kits to third-party services.  
- PolyForm NC + Small-Entity Exception unchanged.  
- Test fixtures: no real user vaults or production keys.

## 9. API / contract impact

| Surface | Change expected |
|---------|-----------------|
| Daemon HTTP DTOs | **None** |
| `ai-brains-contracts` | **None** preferred |
| CLI UX | **None** required; substring pin only |
| Event log schema | **None** |
| Docs | RECOVERY-DRILLS + OPERATIONS + failure-drills |

## 10. Verification plan

### 10.1 Automated

```powershell
cargo nextest run -p ai-brains-cli --test recovery_drills --test smoke
cargo nextest run -p ai-brains-crypto --test crypto_recovery
cargo nextest run -p ai-brains-store --test content_envelope_crypto
cargo nextest run -p ai-brains-brain --lib
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
```

### 10.2 Manual (record in plan.md)

1. Temp vault → ingest/pin → backup create/verify → restore --force → content observable.  
2. Library kit unlock path (dev only); confirm playbook states **no** `recovery export` CLI.  
3. Docs honesty + kit-export residual + Argon2 residual.  
4. No secrets in command output.

### 10.3 Evidence

- Commands + exit codes in `plan.md`  
- Optional `evidence/` without secrets  
- Pin: `DECISION: T181 recovery drills prove pre-erase backup residual; not NIST Purge; kit export remains operator residual`

## 11. Sequencing

1. Inventory + elevate map (Phase A).  
2. Docs playbook (Phase B) — SP 800-184 plan-before-test.  
3. RED/GREEN drills (Phase C).  
4. Review + gate + deferred closeout (Phase D).

## 12. Risks

| Risk | Mitigation |
|------|------------|
| CE wipe needs daemon CLI | F35 in-process destroy_wrap + checkpoint |
| Hermetic env flakiness | T179 pin + T186 helper if present |
| E-drills slow on Windows | Tiny fixtures; F36 `__slow` if needed |
| Reader confuses K-05 with operator CLI | F4/F5/F38 + playbook residual |
| Wrong-key message brittle | F46 empirical pin once |
| Scope creep doctor/export CLI | F5 frozen |

## 13. Handoffs

| To | What |
|----|------|
| **T183** | RECOVERY-DRILLS; doctor + recovery export residual notes |
| **T185** | Recoverable evidence; forbid perfect-deletion / Purge; K-04 no-secrets evidence |
| **T186** | Hermetic CLI builder reuse |
| **T184** | May sample suite; kit-export gap is security-relevant residual |
| **failure-drills.md** | F-REC-01/02 → T181 auto |
| **deferred §59** | Closeout residuals list |

## 14. Definition of Done

- AC1–AC11 met.  
- Spec status → Completed; conductor T181 ✅.  
- deferred §59 completed; pre-erase residual **productized as drill** (physical residual remains).  
- No open critical/high; mediums deferred per AGENTS rules only.

## 15. AI1 / AI2 fold-in disposition (2026-08-01)

### 15.1 Agreed → folded

| Source | Item | Fold |
|--------|------|------|
| AI1 BS2 | Structured error substrings | F10, F46, §5.1 |
| AI1 BS3 | Meta present in backup / absent live | F20, R-01 |
| AI1 Opp1 | `assert_no_secret_leakage` helper | F34, AC5 |
| AI2 F-1 | Argon2 params not in kit | F37, K-07, residual |
| AI2 F-2 | No CLI kit path; K-05 library-only | F4, F5, F38, F43 |
| AI2 F-3 | Strengthen force elevate / supersede | F39, AC11 |
| AI2 F-4 | Rename `test_` prefix | F31, AC11 |
| AI2 F-5 | Name wipe API; scope F2; `__slow` | F2, F35, F36 |
| AI2 F-6 | macOS path note | F42 |
| AI2 F-7 | hex+base64+raw in leak helper | F34 |
| AI2 F-8 | Stringly errors → substring pin | F46, plan C4 |
| AI2 F-9 | Fix verification commands | F47, §10.1 |
| AI2 F-10 | No kit JSON / wrapped ciphertext in output | F12, F34 |
| AI2 F-11 | Side stores = tables not files | F8 clarify |
| AI2 O-1 | Argon2 pin future residual | Residual log |
| AI2 O-2 | Vault-init kit checklist | F43 |
| AI2 O-4 | Pre-decide `__slow` | F36 |
| AI2 O-5 | Wrap-nonce residual link | F44 |
| AI2 O-6 | Meta assert on R-01 | F20 |
| AI2 O-7 | Wrong SqlCipherKey after unlock | F41, K-06 |
| AI2 O-8 | rstest for failure matrix | F40 |
| AI2 O-3 | Header+body corrupt cases | F21 soft cases |

### 15.2 Agreed with reframe

| Source | Item | Disposition |
|--------|------|-------------|
| AI1 BS1 | Require Zeroizing on K-05 | **Reframe F33:** production `SqlCipherKey` is already `ZeroizeOnDrop`; K-05 **must** use `from_data_key`. No fragile runtime memory-scan assert. Optional future: zeroize intermediate hex inside `from_data_key` (prod residual, not DoD). |

### 15.3 Declined

| Source | Item | Why |
|--------|------|-----|
| AI1 Opp2 | Lightweight Argon2 test params in product path | Would require test-only KDF hooks or weaker prod defaults — **F45**. Prefer precomputed kits or `__slow`. |
| AI1 §4 “Fully Compliant” matrix | Marketing-complete claims for docs not yet written | Aspirational; compliance language reserved for shipped deliverables. |
| Implement Argon2 param schema in T181 | Scope / F17 | Document residual only (F37). |
| doctor / recovery export as DoD | F5 | Remains residual. |
