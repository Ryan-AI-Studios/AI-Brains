# Recovery Drills Playbook (T181 / P12.3)

Operator and CI playbook for **backup, restore, and recovery-kit** exercises.
Aligned with **NIST SP 800-184** (plan before the event; validate with exercises).

Automated drill IDs: `T181-R-*`, `T181-K-*`, `T181-E-*`, `T181-F-*`, `T181-D-*`.
See also [failure-drills.md](../conductor/failure-drills.md) F-REC-01/02 and
[OPERATIONS.md](OPERATIONS.md) Backup / Restore.

---

## 1. When to run

| Cadence | Why |
|---------|-----|
| **Release preflight** | Prove restore still works before a versioned ship |
| **Phase / track gates** | After backup, crypto, CE, or vault-open changes |
| **Ad-hoc** | After disk/path incidents, key handling changes, or operator practice |
| **CI** | Automated suite (`recovery_drills`, elevated smoke/crypto/store tests) |

**Do not** treat “a backup file exists” as recovery proof. **Do** run restore + content smoke (T181-R-01).

---

## 2. Operator drill matrix (summary)

| ID | Scenario | Expected | Automation |
|----|----------|----------|------------|
| **T181-R-01** | Seed → backup → force restore → content | Seeded content present; `_aibrains_backup_meta` in backup file only (dropped on live after restore) | Yes |
| **T181-R-02** | Restore `--dry-run` | Integrity ok; destination not mutated | Yes (smoke) |
| **T181-R-03** | Missing backup path | Non-zero; message class: `not found` / `Backup file not found` | Yes |
| **T181-K-01..04** | Kit passphrase / DPAPI / wrong passphrase / no plaintext in JSON | Library crypto contracts | Yes (`crypto_recovery`) |
| **T181-K-05** | Unlock kit → `SqlCipherKey::from_data_key` → open vault/backup | Library primitive chain (**not** full operator CLI export workflow) | Yes |
| **T181-K-06** | Correct unlock + wrong SqlCipherKey open | Open fails under live SQLCipher (T187 strict; dual-mode plain residual removed) | Yes (strict) |
| **T181-K-07** | Kit JSON has no Argon2 KDF param fields | Residual honesty (generation-time defaults) | Yes |
| **T181-E-01** | Pre-erase residual: seal → backup → wipe live → restore pre-wipe | Restored CE content still opens | Yes |
| **T181-E-02** | Post-wipe backup: seal → wipe → backup → restore | Open fails (wrap destroyed) | Yes |
| **T181-F-01** | Corrupt backup (header and/or body) | Non-zero + corruption-class substring | Yes |
| **T181-F-02** | Wrong SQLCipher key on verify/restore | Non-zero + wrong-key class under live SQLCipher (T187 strict) | Yes (strict) |
| **T181-F-03** | Daemon running during restore | **Hard-fail** (T188): non-zero exit, no vault overwrite; message includes `daemon is running` + `ai-brains daemon stop` + `sc stop AI-Brains-Daemon`. Dry-run allowed with live-restore-will-fail notice. `--force` never overrides probe. | Yes (unit-injected daemon-up hard-fail + integration daemon-down restore/export; live-daemon busy restore drill residual) |

---

## 3. Commands (product path)

Use the **CLI Online Backup API path** only — never raw `copy` of a live WAL vault.

```powershell
# Create (default retention --keep 10)
ai-brains --vault-path $Vault backup
ai-brains --vault-path $Vault backup create --output-dir D:\backups

# Verify
ai-brains --vault-path $Vault backup verify $BackupPath
ai-brains --vault-path $Vault backup verify --full   # all known backups

# Restore (stop daemon/service first for live restore — T188 hard-fail)
ai-brains --vault-path $Vault backup restore $BackupPath --dry-run
ai-brains daemon stop   # or: sc stop AI-Brains-Daemon
ai-brains --vault-path $Vault backup restore $BackupPath --force   # non-interactive; --force does not override daemon probe

# Recovery kit export (T188) — kit JSON only to file; never stdout
ai-brains --vault-path $Vault recovery export --output D:\offline\recovery-kit.json --passphrase-file $env:TEMP\kit-pw.txt
# Interactive TTY double-entry (zero-echo via rpassword) when --passphrase-file omitted
```

**Exit codes:** non-zero on missing path, integrity failure, **daemon running during mutating restore (T188)**, output-exists without force on export, short passphrase, and (when SQLCipher page encryption is active) wrong key / open failure. Exact wording is stringly (not typed enums); automation matches **substring classes** (see §7).

> **Encryption honesty (T187):** workspace `rusqlite` uses **`bundled-sqlcipher-vendored-openssl`**. New vaults are page-encrypted (header is **not** plain `SQLite format 3`). Wrong-key open/verify fails closed. Zero keys refused unless `AI_BRAINS_ALLOW_ZERO_KEY=1`. Legacy plain vaults: `ai-brains vault encrypt` (`sqlcipher_export`). **Not** FIPS / NIST Purge. See [COMPATIBILITY.md](COMPATIBILITY.md) F8 / [Deviations.md](Deviations.md) §1 (resolved).

---

## 4. At vault initialization — RecoveryKit

**`ai-brains recovery export` is shipped (T188).** Write kit JSON to a restricted offline path (USB / sealed store). **`ai-brains doctor` remains absent.**

```powershell
ai-brains --vault-path $Vault recovery export --output E:\offline\recovery-kit.json --passphrase-file $SecurePwFile
# stdout: path + "dpapi: present|absent" only — never kit JSON / DataKey / passphrase
```

RecoveryKit generation and passphrase/DPAPI unlock also remain **library** (`ai-brains-crypto`) capabilities (T181-K-* drills). Kits include `schema_version: 1` (serde default for older kits without the field).

### Checklist (operator)

- [ ] At vault initialization (or soon after), **export and store RecoveryKit JSON out-of-band** (secure offline storage). Without this, a machine that loses DPAPI wrapping cannot passphrase-recover the DataKey.
- [ ] Store the passphrase in a password manager / HSM / sealed envelope — never in the repo or CI logs. Min length **8 bytes**. Prefer `--passphrase-file` over argv (there is no `--passphrase` flag).
- [ ] Confirm kit JSON does **not** contain plaintext DataKey (hex/base64); salt/nonce are public.
- [ ] Unix kit files are created mode **0600**; on Windows, refuse well-known public paths (`C:\Users\Public`); offline USB paths are allowed.
- [ ] **Passphrase-file and kit output refuse reparse/symlink/junction paths** (F8b) — use regular files only.
- [ ] **`ai-brains doctor` is still not shipped** — recoverability checks remain operator / drill responsibility.

K-05 proves the **primitive** chain (unlock → `SqlCipherKey::from_data_key` → open). Export CLI proves the operator write path; store the file offline yourself.

---

## 5. Content-envelope honesty (pre-erase residual)

Content-envelope (CE) wipe destroys **wrap material** (`destroy_content_key_wrap` + WAL checkpoint). It does **not**:

- Claim **NIST SP 800-88r2 Purge or Destroy**
- Sanitize offline media, exports, or **pre-erase backups**
- Equal “SQLCipher vault locked” to per-item CE

**T181-E-01** productizes the residual: a backup taken **before** wipe remains decryptable after live wipe. This is an **honest residual**, not a bug to “fix” by deleting every offline copy automatically.

Side stores (`content_key_store`, encrypted blobs) live as **tables inside the vault SQLite DB** (migration 0026). The Online Backup API copies them with the main database — they are not separate sidecar files.

See **ADR-0016** §12 (CE honesty / non-claims) and §4 (DataKey wrap-nonce residual). Ticket/soft forget ≠ CE.

---

## 6. Argon2 residual

`PassphraseWrappedKey` stores `ciphertext`, `salt`, and `nonce` only — **no** KDF parameters (`m_cost`, `t_cost`, `p_cost`, algorithm id).

Wrap and unwrap both use `Argon2::default()` at the **generation-time** defaults of the linked `argon2` **0.5.x** crate: **Argon2id**, **m=19456**, **t=2**, **p=1**, version **0x13**. A future schema may pin params in kit JSON; that is **not** required for T181/T188. Automated **T181-K-07** asserts kit JSON lacks KDF param field names.

**Backups** inherit SQLCipher page encryption of the live vault when T187 is active (Online Backup API copies the encrypted DB).

---

## 7. Failure substring classes (automation)

| Class | Match at least one (case may vary) |
|-------|-------------------------------------|
| Missing path (R-03) | `not found` / `Backup file not found` |
| Corrupt (F-01) | `integrity` / `corrupt` / `not a database` / `query failed` / `Integrity check failed` |
| Wrong key (F-02) | Fail-closed under live SQLCipher (T187): non-zero + wrong-key class (`not a database` / key verification / VaultLocked) |
| Daemon busy restore (T188 F-03) | `daemon is running` + stop guidance (`ai-brains daemon stop` and `sc stop` / service name) |
| Export output exists | `exists` / `output exists` |
| Passphrase too short | `passphrase` + `short` / min length |

Non-zero exit alone is **insufficient**.

---

## 8. Secrets handling

- Never print passphrases, raw DataKey bytes, DPAPI blobs, ContentDek material, or full kit JSON / wrapped ciphertext in CI logs or command output.
- Tests use `assert_no_secret_leakage` (hex + base64 + raw forms).
- Do not upload vault fixtures or recovery kits to third-party services.

---

## 9. 3-2-1 and RTO/RPO (operator guidance only)

**3-2-1 (non-product):** keep at least three copies, on two media types, with one offsite/offline. AI-Brains does **not** automate offsite SaaS backup.

**Example RTO/RPO language (not a product SLA):**

- Example RPO: last successful verified backup age (operator-chosen schedule).
- Example RTO: time to restore + verify content smoke on a known-good host.

These are planning aids only — not contractual or marketed guarantees.

---

## 10. Platform notes

| Platform | Notes |
|----------|--------|
| **Windows (T1)** | Full suite including DPAPI kit arm |
| **Linux / macOS** | Passphrase + backup/restore + envelope drills; DPAPI soft-skip |
| **macOS paths** | Backup meta `source_vault_path` may show `/private/var` vs `/var` after canonicalize — expected (dunce/canonicalize) |

---

## 11. Related residuals (not T181 DoD)

| Residual | Owner |
|----------|--------|
| `ai-brains doctor` product | Still absent (deferred #2) |
| ~~`ai-brains recovery export` CLI~~ | **Closed by T188** |
| Argon2 params in kit schema | Future crypto hygiene (defaults documented: m=19456,t=2,p=1) |
| ~~**Wrong-key / K-06 fail-closed requires SQLCipher page encryption**~~ | **Closed by T187** — live `bundled-sqlcipher-vendored-openssl`; strict drills |
| #34.2 DataKey rotation | Open (T189) |
| F-REC-03/04 projection/graph rebuild drills | Soft residual |
| ~~Hard-fail restore while daemon running~~ | **Closed by T188** (robust probe ≥1s × 3 attempts) |
| Multi-device CE orchestration | T176–T178 |

### Post-drill improvement (NIST SP 800-184)

After each restore/export drill: record what failed, update this playbook or runbooks, and re-run until the exercise is clean. Planning + exercise + improvement loop — not a product SLA.

---

## 12. Verification commands (developers)

```powershell
cargo nextest run -p ai-brains-cli --test recovery_drills --test smoke
cargo nextest run -p ai-brains-crypto --test crypto_recovery
cargo nextest run -p ai-brains-store --test recovery_drills --test content_envelope_crypto
```

Pin after implement:

`DECISION: T181 recovery drills prove pre-erase backup residual; not NIST Purge. T188 ships recovery export + restore hard-fail; doctor remains residual.`
