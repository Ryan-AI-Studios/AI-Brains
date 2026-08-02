# T188 — Restore Safety + Recovery Operator Surface

- **Track ID:** T188-RestoreSafetyRecoveryExport
- **Phase:** Post-P12 operator safety residual
- **Status:** 📋 **Pending / Expanded** (AI fold-in 2026-08-02; **planning only — not implementing**)
- **Depends on (hard):** T181 RECOVERY-DRILLS + T76/T99 restore path; `RecoveryKit` library; `DaemonClient::probe`
- **Depends on (soft):** **T187** page encrypt
- **Blocks / feeds:** Operator kit export; **R-DOC-CLI** partial; T181-F-03 productization
- **Category:** SECURITY / FEATURE
- **Deferred absorbed:** §59 **#1** recovery export; §59 **#6** restore hard-fail; R-DOC-CLI partial; T181 kit-export residual; T181-F-03
- **Not absorbed:** Full doctor (**#2**); #34.2 (**T189**); SQLCipher (**T187**); Argon2 params in kit (soft residual, document only)
- **Research date:** 2026-08-02 (NIST SP 800-184; OWASP secrets; argon2 0.5.3 defaults; rpassword 7.5.4)
- **AI fold-in:** AI1 §1–5 + AI2 C1–C2, H1–H4, M1–M6, L1–L5. Disposition §15.

## 1. Objective

1. **`backup restore` hard-fails** (non-zero, no vault overwrite) when the daemon/service is reachable via a **robust** IPC probe.
2. Ship **`ai-brains recovery export`** writing a RecoveryKit to a restricted file path (never kit JSON on stdout).
3. Secrets discipline: zero-echo TTY or passphrase-file; zeroize buffers; leakage tests cover ciphertext + tracing.
4. Docs/claims: R-DOC-CLI partial (export present; doctor still absent).

## 2. Live baseline (re-scan 2026-08-02)

| Asset | State |
|-------|--------|
| `run_restore` | Probe **warn only** (200ms); continues overwrite |
| `DaemonClient::probe` | Single `Ping`/`Pong` within caller timeout |
| Pipe name | Interactive + service share `\\.\pipe\ledgerful-bridge` (same probe) |
| `RecoveryKit` | Library generate/unlock/to_json; no `schema_version` field yet |
| `data_key_from_sqlcipher` | `Result<DataKey, String>`; errors use format literals (not raw hex) |
| CLI `recovery *` | Absent |
| `RecoveryKitCreated` | Payload `{ key_id }` only; `AggregateType::System` available for emit |
| AppContext | `open` + **`migrate()`** (write) — unsafe while daemon holds vault |
| doctor | Absent |

## 3. Research summary

| Source | Finding | T188 application |
|--------|---------|------------------|
| NIST SP 800-184 | Offline before destructive recovery; exercise + improvement loop | Hard-fail restore; dry-run notice; playbook post-drill line |
| OWASP secrets / CLI | No argv secrets; zeroize; file modes; min passphrase policy | F8–F10, F19–F21 |
| OWASP Argon2id mins | m=19456, t=2, p=1 match argon2 0.5.3 defaults | Docs cite tuple; not pin in kit schema (T181 F37 residual) |
| rpassword 7.5.4 | Apache-2.0; `ConfigBuilder` testability | **F17 allow one prod dep** (or hand-roll — see F17) |
| zeroize 1.8.x (workspace) | `Zeroizing` / ZeroizeOnDrop | Passphrase + file buffers |

## 4. Frozen decisions (F1–F28)

| ID | Decision |
|----|----------|
| **F1 — Daemon hard-fail** | Mutating restore: if robust probe true → non-zero; **no overwrite**. |
| **F1b — Robust probe** | Destructive restore uses dedicated helper: **timeout ≥1000ms**, **≥2 retries** with short backoff, then conclude offline. Do **not** rely on a single 200ms ping. |
| **F2 — `--force`** | Skips interactive confirm only. **Never** overrides daemon probe. No toxic `--force-with-daemon`. |
| **F3 — Dry-run restore** | Allowed while daemon up. **Must** print a **prominent notice** that a live restore will fail until daemon/service is stopped (assert in tests). Then normal dry-run integrity plan. |
| **F4 — Probe honesty** | Detects our IPC only (not third-party lockers). Document residual. |
| **F5 — Service message** | Probe cannot distinguish interactive vs Session-0 service. Error lists **both**: `ai-brains daemon stop` **and** service stop (`sc stop` / documented service name). No `sc query` branching in T188 (M1 option a). |
| **F6 — CLI name** | `ai-brains recovery export`. |
| **F7 — DataKey source** | `data_key_from_sqlcipher(&ctx._key)`. Pin test: malformed key error **contains no key hex material**. |
| **F8 — Passphrase acquisition** | (a) `--passphrase-file`; (b) TTY double-entry **zero-echo**. **Forbid** `--passphrase` argv. Never log/trace passphrase. Min length **8 bytes**. |
| **F8b — Passphrase-file safety** | Read into `Zeroizing<Vec<u8>>`; **8 KiB max**; reject non-regular files (not pipe/device); refuse symlink-to-unexpected where portable (or document best-effort); zeroize on all paths; **do not** truncate operator file. |
| **F9 — Output path** | `--output` required. Kit JSON **only** to file. Stdout: path + status (`dpapi: present\|absent`). If output **exists** → refuse unless `--force`/`--overwrite` (message class: `exists` / `output exists`). |
| **F9b — File permissions** | Unix: create with **0600**. Windows: refuse well-known public paths (`C:\Users\Public`, world-writable roots); best-effort owner-only ACL when portable; other paths allowed (USB offline kits) with optional warn. |
| **F10 — Secrets tests** | Assert DataKey bytes + **wrapped ciphertext** absent from stdout/stderr/**tracing** capture. Salt/nonce **allowed** (public in kit). Min passphrase 8 so raw UTF-8 leakage helper applies. |
| **F11 — DPAPI status** | Stdout status includes `dpapi: present` or `dpapi: absent`. Absent is **not** failure (passphrase arm is canonical). |
| **F12 — Event (mandatory with bounds)** | On successful **non-dry-run** export that appends to vault: emit `RecoveryKitCreated { key_id }` where `key_id` = new UUID; use **`AggregateType::System`** + aggregate_id = same UUID (no synthetic project_id). **Never** put kit material in event. Dry-run: no event. If daemon holds vault and write append fails → **kit file still success**; event miss → **warn** (not hard-fail export). Prefer skip migrate when daemon up (F16b). |
| **F13 — doctor** | Out of scope (#2). |
| **F14 — Export dry-run** | Validates passphrase **source** (file readable / TTY present) **without** TTY prompt content; if `--passphrase-file`, read+zeroize to validate readability. Prints would-write path. No file, no event. |
| **F15 — NIST** | Playbook alignment + post-drill improvement one-liner. No RTO/RPO SLA; no Purge/Destroy. |
| **F16 — Capture independence** | No models/graph required. |
| **F16b — Export vs live daemon** | Export **must not** call `migrate()` while daemon is up. Prefer: derive key from CLI key without full AppContext migrate path; if vault open needed, use **read-intent / no-migrate** open. Export of kit file **may proceed with daemon up** (non-destructive). Event append may soft-fail with warn if vault write blocked. |
| **F17 — Deps** | Prefer **one** new production dep: **`rpassword` 7.4+ / 7.5.x** (Apache-2.0, deny-allowlist). Alternative: hand-roll zero-echo via `windows` / termios (no new dep) — expand pick at implement; **document choice in plan**. Passphrase held in `Zeroizing<String>` / zeroized bytes after generate. |
| **F18 — vs T187** | Independent implement OK; soft re-verify under page encrypt. |
| **F19 — schema_version** | Add `schema_version: u32` default **1** on `RecoveryKit` (serde default) for T189 forward-compat. Non-breaking for old kits without field. |
| **F20 — TTY zero-echo** | Required for interactive path (rpassword or hand-roll). Double-entry must match. |
| **F21 — No secrets in tracing** | Production export/restore paths must never pass passphrase/key/kit to `tracing::*` macros. Tests capture tracing where practical; review D4 checks. |
| **F22 — Argon2 honesty** | D2 docs cite argon2 0.5.x defaults (Argon2id, m=19456, t=2, p=1, V0x13) — not pinned in kit JSON (residual). |

## 5. Error / substring classes

| Scenario | Exit | Class |
|----------|------|-------|
| Restore + daemon up | non-zero | `daemon is running` + stop guidance (daemon + service) |
| Restore dry-run + daemon up | 0 | notice: live restore will fail / daemon active |
| Output exists without force | non-zero | `exists` / `output exists` |
| Passphrase too short | non-zero | `passphrase` / `too short` / min length |
| Passphrase file not regular / too large | non-zero | actionable |
| Export success | 0 | path + `dpapi: present\|absent`; no kit JSON |

## 6. Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | Mutating restore + robust probe true → non-zero; vault unchanged |
| **AC2** | Daemon down + restore success; `--force` only skips confirm |
| **AC3** | Dry-run + daemon up → success **and** notice substring asserted |
| **AC4** | Export writes unlockable kit; min passphrase 8; schema_version=1 |
| **AC5** | Leakage: no DataKey/ciphertext/passphrase in stdout/stderr/tracing capture; salt/nonce OK |
| **AC6** | Unix 0600 on kit file; Windows public-path refuse tested or documented |
| **AC7** | Docs + R-DOC-CLI partial (export yes, doctor no) |
| **AC8** | Zero unexpected deps; deny/audit green (rpassword if chosen) |
| **AC9** | Full gate + SECURITY review |
| **AC10** | deferred #1/#6 struck; #2 doctor remains |
| **AC11** | T181-F-03 → hard-fail product language |
| **AC12** | `data_key_from_sqlcipher` malformed error has no key material (test) |
| **AC13** | Export without migrate-while-daemon-up; kit file OK; event best-effort |
| **AC14** | Output exists refuse unless overwrite force |

## 7. Non-goals

Full doctor; recovery unlock CLI; Argon2 params in kit schema (document only); DataKey rotation; SQLCipher flip; toxic force-with-daemon; third-party lock detection; `sc query` disambiguation.

## 8. Verification

```powershell
cargo nextest run -p ai-brains-cli --test recovery_drills --test smoke
cargo nextest run -p ai-brains-crypto --test crypto_recovery
cargo clippy -p ai-brains-cli -p ai-brains-crypto --all-targets -- -D warnings
# Full gate at closeout
```

### Manual evidence

1. Daemon up → restore fails; dry-run shows notice.  
2. Daemon down → restore OK.  
3. Export to kit file; unlock roundtrip; `dpapi:` line; 0600/public refuse.  
4. No secrets in output/logs.  
5. Docs R-DOC-CLI honesty.

## 9. Handoffs

| To | What |
|----|------|
| deferred #1/#6 | Strike |
| deferred #2 doctor | Remain |
| T187 | Soft re-verify |
| T189 | schema_version=1 ready |

## 10. Definition of Done

AC1–AC14; plan checked; conductor ✅; no open critical/high.

## 15. AI fold-in disposition (2026-08-02)

### 15.1 Agreed → folded

| Source | Item | Fold |
|--------|------|------|
| AI1 §1 | Robust probe timeout/retries | **F1b** |
| AI1 §2 | 0600 + Windows public refuse | **F9b** (profile containment softened — allow offline USB paths) |
| AI1 §3 / AI2 H1 | Zero-echo TTY | **F20** + **F17** (rpassword or hand-roll) |
| AI1 §5 / AI2 M2 | Dry-run daemon notice | **F3** mandatory + AC3 assert |
| AI2 C2 | Export vs daemon migrate | **F16b** + AC13 |
| AI2 H2 | Passphrase-file safety | **F8b** |
| AI2 H3 | Ciphertext leakage + min length | **F8** min 8 + **F10** |
| AI2 H4 | data_key error no material | **F7** + AC12 |
| AI2 M1 | Service message ambiguity | **F5** document both commands |
| AI2 M3 | Dry-run passphrase source | **F14** |
| AI2 M4 | dpapi present/absent | **F11** |
| AI2 M5 | Tracing leakage | **F21** + AC5 |
| AI2 M6 | schema_version | **F19** |
| AI2 L5 | Output exists refuse | **F9** + AC14 |
| AI2 L1–L3 | Playbook/docs honesty | F15, D2 Argon2 tuple, backup encryption note |

### 15.2 Agreed with reframe

| Source | Item | Disposition |
|--------|------|-------------|
| AI1 §4 / AI2 C1 | Mandatory RecoveryKitCreated | **F12**: emit via **System** aggregate + opaque UUID; **kit file is DoD**; event best-effort if vault write blocked by daemon (warn, not fail export). No synthetic project_id. No payload schema expansion this track. |
| AI1 §2 Windows must-be-under-USERPROFILE | Too strict for offline USB kits | Refuse **Public/shared**; allow other paths |
| AI2 H1 rpassword required | F17 conflict | **Allow rpassword** as sole new prod dep **or** hand-roll; freeze choice at implement |

### 15.3 Declined / deferred

| Source | Item | Why |
|--------|------|-----|
| AI2 M1 option b | `sc query` branching | Extra platform surface; F5 docs sufficient |
| Expand RecoveryKitCreated payload with vault_id | Contracts ripple | Out; System+UUID enough for audit “export happened” |
| Argon2 params in kit JSON | T181 F37 residual | Document only (F22) |
