# T187 — SQLCipher Page Encryption (live)

- **Track ID:** T187-SqlCipherPageEncryption
- **Phase:** Post-P12 security residual (closes Deviations §1 / R-F8 live path)
- **Status:** 📋 **Pending / Expanded** (AI fold-in 2026-08-02; **planning only — not implementing**)
- **Depends on:** T181 drills (dual-mode ready); T179 multi-OS CI; T185 claims SOOT; **Windows build: Perl (+ NASM if not using no-asm)**
- **Blocks / feeds:** Honest page-encryption claims; T181-F-02/K-06 strict mode; R-ZERO-KEY harden
- **Category:** SECURITY / INFRA
- **Deferred absorbed:** `deferred.md` §59 #8; RELEASE-CLAIMS **R-F8** / **R-K06**; COMPATIBILITY F8; Deviations §1; **R-ZERO-KEY** partial (refuse zero-key unless escape hatch)
- **Not absorbed:** #34.2 (**T189**); recovery export / daemon restore hard-fail (**T188**); #12 (**T190**); `cipher_integrity_check` productization (soft residual)
- **Research date:** 2026-08-02 (Zetetic + rusqlite + live MSVC probe: `perl` required for openssl-src)
- **AI fold-in:** AI1 (items 1–5) + AI2 (C1–C2, H1–H4, M1–M6, L1–L4). Disposition §15.
- **Ledger TX (accidental start during overshoot):** `dc9f932a-ca0b-40c0-8d42-dcf556501633` — abandon before real implement if still pending.

## 1. Objective

Make **default production builds** use SQLCipher page-level encryption so:

1. Wrong key fails closed on vault open / backup verify / restore (T181-F-02, K-06).
2. File header is no longer plain `SQLite format 3` for new vaults under correct key workflow.
3. Claims docs flip from “feature-gated residual” to “live with evidence” without claiming FIPS or NIST Purge/Destroy.
4. Legacy plaintext vaults fail with an **actionable migrate hint**, and operators have **one named encrypt path**.

## 2. Live baseline (re-scan 2026-08-02)

| Asset | State |
|-------|--------|
| Workspace `rusqlite` | Target: `0.39.0` + `bundled-sqlcipher-vendored-openssl` + `backup` + `fallible_uint` (hold 0.40) |
| MSVC vendored OpenSSL | **Requires Perl** (`openssl-src` runs `perl ./Configure`); live probe failed without Strawberry Perl. NASM optional if `no-asm` used |
| `VaultConnection::open` | `apply_pragmas` + `SELECT count(*) FROM sqlite_master` → generic `VaultLocked` (no plain-header branch yet) |
| `open_read_intent` | Same key path; **must** share plain-header sniff |
| `BackupService::run_backup` | Source open **without** key (F6) |
| `has_core_tables` / list path | `let _ = apply_key_pragmas(...)` — wrong key becomes silent “missing” |
| T181-F-02 / K-06 | Dual-mode `if plain` residual still present |
| CLI default key | All-zero hex when `--key` omitted (**R-ZERO-KEY**) |
| Scratch bins | `ai-brains-brain/src/bin/check_db.rs` (hardcoded user path, no key); `scratch/check_vault.rs` (`unwrap`) |
| PRAGMA key construction | String `execute_batch` with double quotes — keep with documented reason; prefer single-quote form matching Zetetic examples |

## 3. Frozen decisions (F1–F22)

| ID | Decision |
|----|----------|
| **F1** | `rusqlite` features = `bundled-sqlcipher-vendored-openssl`, `backup`, `fallible_uint` (replace bare `bundled`). |
| **F2** | Hold rusqlite **0.39.0** (no 0.40 bump unless spike forces). |
| **F3** | Mandatory post-key schema read on vault open paths; shared `verify_key` helper. |
| **F4 — Plain→encrypted migrate** | **Mandate `sqlcipher_export` (ATTACH + export), not Online Backup**, for plaintext→encrypted conversion. Online Backup is page-copy and does **not** re-encrypt plaintext pages into a SQLCipher codec (Zetetic / AI1). Sequence (normative): (1) `PRAGMA wal_checkpoint(TRUNCATE)` on source; (2) open source **unkeyed** (plaintext); (3) `ATTACH DATABASE '…enc…' AS encrypted KEY "x'…'"`; (4) `SELECT sqlcipher_export('encrypted');` (5) `DETACH`; (6) operator-confirmed atomic replace of original (preserve original until confirm). **No** in-place `PRAGMA rekey` on plaintext. |
| **F5** | **Encrypted↔encrypted** vault backup/restore remains Online Backup API only (product path). Distinct from F4 migrate. |
| **F6** | All vault/backup **keyed** opens must apply key + verify. `run_backup` must key source. Workspace **unkeyed-open audit** (vault/backup vs plain-exempt vs test) is in-scope. |
| **F7** | T181-F-02 / K-06 **strict** on default build: **delete** `if plain` residual branches (explicit). |
| **F8** | Update claims/docs; forbid FIPS / NIST Purge / perfect deletion. Record observed `PRAGMA cipher_version` in D2. |
| **F9** | Refuse all-zero key unless `AI_BRAINS_ALLOW_ZERO_KEY=1`. Enforcement in **`VaultConnection::open` + `open_read_intent`** (not only `AppContext`). |
| **F10** | Empty/blank key refused. |
| **F11** | No AGPL; deny/audit green; **re-verify** SQLCipher community + OpenSSL SPDX after flip (paste deny output into review log). |
| **F12** | Capture independence unchanged. |
| **F13** | Windows T1 + Linux core CI build SQLCipher path. **Document Perl** (and NASM if required) in `Docs/ci-tooling.md` + `scripts/dev-check.ps1` capability check. GHA windows job: ensure Perl on PATH. |
| **F14** | `cipher_compatibility = 4` retained. Do **not** set `cipher_plaintext_header_size` (full header encrypted; AC2 depends on this). Document in D2. |
| **F15** | No CE / DataKey rotation; page key ≠ content DEK. |
| **F16** | Plain→encrypted is operator-triggered only (not automatic on open). |
| **F17 — Header sniff** | Before `PRAGMA key` on **existing non-empty** files: if first 16 bytes == `SQLite format 3\0` → `StoreError::LegacyPlaintextVault { migrate_hint }` (exact message class pinned). Else apply key + verify; failure → `VaultLocked` (wrong key / corrupt). Shared helper used by `open` and `open_read_intent`. Lift pattern from T181 `is_plain_sqlite_header`. |
| **F18 — Operator command** | Named CLI: **`ai-brains vault encrypt`** (minimal surface): source path (default live vault), dest or in-place-with-confirm, requires key, dry-run default or `--confirm` for replace. Implements F4 export sequence. |
| **F19 — Zero-key tests** | Hermetic CLI helper + `TempEnv` RAII set `AI_BRAINS_ALLOW_ZERO_KEY=1` for tests (AGENTS.md: **no** bare `std::env::set_var`). Add `SqlCipherKey::is_zero()`. |
| **F20 — Key validation** | `SqlCipherKey::validate()` / fallible constructors: product keys match `^x'[0-9a-fA-F]{64}'$` (32-byte raw). Reject malformed `from_raw`. |
| **F21 — Cipher live smoke** | Unit/integration test: `PRAGMA cipher_version` returns non-empty (proves codec linked). Guards feature-flag drift back to plain `bundled`. |
| **F22 — Scratch hygiene** | Delete or feature-gate `check_db.rs` (remove hardcoded user path) and `scratch/check_vault.rs` during T187 so they are not unrelated-gate failures. |

### F4 vs F5 (layers)

| Path | Mechanism |
|------|-----------|
| Daily backup of **already encrypted** vault | Online Backup (`BackupService`) — F5 |
| One-time **plain → encrypted** convert | `sqlcipher_export` via `vault encrypt` — F4/F18 |
| Wrong | Online Backup plain→keyed as “encrypt” (AI1 C1 / AI2 M5 corrected) |

## 4. Error classes (normative for AC3/AC5)

| Condition | Error | Substring class (pin empirically) |
|-----------|--------|-----------------------------------|
| Existing file header plain SQLite | `LegacyPlaintextVault` | `plaintext` / `Legacy plaintext` / `vault encrypt` |
| Wrong key / corrupt after header looks encrypted | `VaultLocked` | `Key verification failed` / `encrypted or is not a database` |
| Zero key without escape hatch | `VaultLocked` or dedicated | `zero key` / `AI_BRAINS_ALLOW_ZERO_KEY` |
| Blank / invalid key format | Construction error | `invalid` / `key format` |

## 5. Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | Workspace uses SQLCipher vendored-openssl feature set. |
| **AC2** | New vaults after init/write are not plain `SQLite format 3`. |
| **AC3** | Wrong key open/verify fails non-zero with pinned substring class (`VaultLocked` class). |
| **AC4** | T181-F-02 and K-06 **strict**; `if plain` branches **removed** from recovery_drills (cli + store). |
| **AC5** | Plain legacy vault: `LegacyPlaintextVault` + migrate hint; `vault encrypt` content smoke. |
| **AC6** | Backup create/restore green under SQLCipher; `run_backup` keys source; list/has_core_tables surfaces key failures (not silent skip). |
| **AC7** | Zero-key refuse at `VaultConnection`; escape hatch + hermetic `TempEnv`/helper. |
| **AC8** | Docs/claims updated; Deviations §1 closed; `cipher_version` recorded; no FIPS/Purge overclaim. |
| **AC9** | Full gate green Windows T1 + Linux; Perl documented + CI PATH; deny/audit after flip. |
| **AC10** | SECURITY review clean; deferred §59 #8 struck. |
| **AC11** | `PRAGMA cipher_version` smoke green (F21). |
| **AC12** | `SqlCipherKey` validate/is_zero shipped (F19–F20). |
| **AC13** | Scratch `check_db` / `check_vault` deleted or gated (F22). |

## 6. Non-goals

| Out | Owner |
|-----|--------|
| DataKey rotation | T189 |
| recovery export / daemon restore hard-fail | T188 |
| #12 TOCTOU | T190 |
| `PRAGMA cipher_integrity_check` on backup verify | Soft residual (L1) |
| FIPS / NIST Purge / MSI | Never / packaging |
| Silent auto-encrypt on open | Forbidden |
| argon2 0.6-rc | Out |

## 7. Verification

```powershell
# Tooling preflight
perl -v
# Optional: nasm -v
.\scripts\dev-check.ps1 -CheckOnly   # after Perl/NASM checks added

cargo build -p ai-brains-store -p ai-brains-cli -p ai-brains-brain
cargo nextest run -p ai-brains-store -p ai-brains-cli --test recovery_drills
cargo nextest run --workspace --profile ci
cargo deny check ; cargo audit
# Manual: header not plain; wrong key; plain → vault encrypt; cipher_version
```

## 8. Unkeyed-open audit (implement inventory — seed)

Classify every `Connection::open` / `open_with_flags` (ledgerful/grep ~90 sites):

| Class | Rule |
|-------|------|
| **Vault** | Must key + verify (or go through `VaultConnection`) |
| **Backup file** | Must key + verify; fail closed on wrong key |
| **Plain-exempt** | Ledgerful / non-vault SQLite only if proven non-SQLCipher |
| **Test** | Keyed or zero-key **with** `AI_BRAINS_ALLOW_ZERO_KEY` via RAII helper |

**Known vault/backup gaps (must fix):**

| Site | Issue |
|------|--------|
| `backup.rs` `run_backup` | Source unkeyed |
| `backup.rs` `has_core_tables` | Ignores `apply_key_pragmas` error |
| CLI backup verify/restore paths | Audit for shared key apply |
| Daemon / shadow / migrate / live_graph / symbol_bridge | Share `VaultConnection` zero-key policy |

## 9. Handoffs

| To | What |
|----|------|
| **T188** | Strict wrong-key env for restore/export |
| **T185 claims** | R-F8 / R-K06 evidence + cipher_version cite |
| **deferred §59 #8** | Strike on complete |
| **ci-tooling / dev-check** | Perl (+ NASM) prerequisites |

## 10. AI fold-in disposition (2026-08-02)

### 10.1 Agreed → folded

| Source | Item | Fold |
|--------|------|------|
| AI1 §1 / AI2 M5 | Online Backup ≠ plain→encrypt | **F4** mandates `sqlcipher_export` |
| AI1 §2 / AI2 C2 | `run_backup` unkeyed source | **F6** + plan B2 audit |
| AI1 §3 / AI2 H1 | Magic-byte header before key | **F17** + error table §4 |
| AI1 §4 / AI2 C1 | Perl/NASM MSVC prereq | **F13** + preconditions |
| AI1 §5 / AI2 H2 | Zero-key test blast radius | **F9** + **F19** VaultConnection + TempEnv |
| AI2 H3 | Delete `if plain` branch | **F7** + plan C2 explicit |
| AI2 H4 | `SqlCipherKey::validate` | **F20** |
| AI2 M1 | scratch binaries | **F22** / AC13 |
| AI2 M2 | `cipher_version` smoke | **F21** / AC11 |
| AI2 M3 | PRAGMA key quoting note | §2 baseline + soft implement note |
| AI2 M4 | Pin cipher_version in docs | **F8** D2 |
| AI2 M5 | Name operator command | **F18** `vault encrypt` |
| AI2 M6 | has_core_tables silent fail | §8 audit + plan B2 |
| AI2 L2 | plaintext_header_size unset | **F14** doc |
| AI2 L3 | License re-verify | **F11** + A3 |
| AI2 L4 | OPENSSL_* env hermetic note | Plan D / hermetic note |

### 10.2 Agreed with reframe

| Source | Item | Disposition |
|--------|------|-------------|
| AI1 §1 step 6 atomic replace | Auto-replace original | **Operator-confirmed** only (F16); dry-run / `--confirm` |
| AI2 H2 enforce only AppContext | — | **Reframe:** enforce in `VaultConnection` so daemon/shadow share policy |

### 10.3 Declined / deferred out of T187

| Source | Item | Why |
|--------|------|-----|
| AI2 L1 | `cipher_integrity_check` on verify | Valuable; soft residual, not AC (scope) |
| Online Backup as plain→encrypt preferred | — | **Declined** (technically incorrect) |
