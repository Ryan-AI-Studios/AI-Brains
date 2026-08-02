# Security limits (honest hub)

One-page executive summary of **what AI-Brains does and does not claim** for operators, security reviewers, and release gates.  
Product version **0.1.1**. Index: [README.md](README.md).

This document **links** normative sources; it does not replace ADRs or COMPATIBILITY.

---

## 1. Vault storage & encryption (F8)

**SOT:** [COMPATIBILITY.md §4](COMPATIBILITY.md)

Vault storage uses **SQLCipher page-level encryption** (T187: `bundled-sqlcipher-vendored-openssl`) combined with **application-level Content Envelope AES-256-GCM** (P8) and OS filesystem permissions.

| Claim | Status |
|-------|--------|
| Page-level SQLCipher live on default builds | **Yes** (T187); `PRAGMA cipher_version` non-empty smoke |
| Wrong-key fail-closed (open / backup verify) | **Yes** (T187; elevates T181 F-02/K-06) |
| Zero-key refuse unless `AI_BRAINS_ALLOW_ZERO_KEY=1` | **Yes** (T187) |
| FIPS / NIST Purge / perfect deletion | **Forbidden non-claim** |
| Application CE AES-256-GCM for sensitive payloads | **Yes** (see ADR-0016); page key ≠ content DEK |
| OS file permissions matter | **Yes** |
| Plain→encrypted migrate | Operator `ai-brains vault encrypt` (`sqlcipher_export`) |

Also: [Deviations.md](Deviations.md) §1 (resolved T187) · [ADR-0016](DECISIONS/ADR-0016-content-envelope-cryptography.md)

---

## 2. Content envelope & cryptographic erasure

**SOT:** [ADR-0016](DECISIONS/ADR-0016-content-envelope-cryptography.md) · OPERATIONS erasure · [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md)

| Topic | Honesty |
|-------|---------|
| Live vault CE wipe | Envelope-backed; can make live content unreadable when wraps destroyed |
| Pre-erase backups | **Remain recoverable** (T181-E-01 residual) — ticket/wipe ≠ destroy offline copies |
| SQLCipher vault lock = per-item CE | **No** |
| NIST SP 800-88 Purge / Destroy | **Non-claim** (RustCrypto is not a FIPS/NIST-validated module claim) |
| Perfect deletion | **Non-claim** |

---

## 3. Optional multi-device “sync” (replication)

**SOT:** [ADR-0018](DECISIONS/ADR-0018-encrypted-event-replication-protocol.md)

| Topic | Honesty |
|-------|---------|
| Default | **Local-only**; multi-device is optional |
| What replicates | End-to-end **encrypted event envelopes** via untrusted relay — **not** live SQLite files |
| Metadata residual | **Yes** — not metadata-private |
| ACK = wipe proof | **No** |
| CLI name | `ai-brains replicate` / `device` — **not** Ledgerful `ai-brains sync` |

---

## 4. Optional cloud processing

| Topic | Honesty |
|-------|---------|
| Capture offline | **Required** path without cloud |
| `allow_cloud` | Default **false** |
| Sealed / local-strict | Cloud must not be required for capture |
| Cloud-required product | **Non-claim** |

See CAPABILITIES models section and T157 policy notes in OPERATIONS.

---

## 5. Connectors / plugins

**SOT:** [ADR-0019](DECISIONS/ADR-0019-connector-sandbox-execution-model.md)

| Topic | Honesty |
|-------|---------|
| v1 release model | **`TrustedBuiltin` only** |
| Third-party plugin host / marketplace | **Not shipped** |
| “Sandboxed plugins” / “WASI isolation” marketing | **Forbidden** until a reviewed host lands |
| Policy (grants) still applies | TrustedBuiltin is not privilege escalation |

---

## 6. Recovery kit & doctor residuals

| Product surface | Status |
|-----------------|--------|
| `ai-brains recovery export` CLI | **Shipped (T188)** — kit JSON to restricted file only; passphrase file or zero-echo TTY; no `--passphrase` argv |
| Recovery kit **library** unlock → open chain | Documented / drilled in [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) (K-05 = primitive; export is operator CLI) |
| `ai-brains doctor` CLI | **Not shipped** |
| `contracts::doctor` DTO | May exist in-tree — **DTO ≠ CLI** |
| Restore while daemon up | **Hard-fail** (T188); robust probe; `--force` never overrides |

Operator practice: RECOVERY-DRILLS + `ai-brains backup` suite + `recovery export`. Do not invent `doctor` in runbooks.

---

## 7. Multi-user / machine residuals

| Residual | Note |
|----------|------|
| Loopback HTTP + bearer | Opt-in; local multi-user residual if token shared |
| Named-pipe IPC (Windows) | Pipe SDDL is **SYSTEM + Administrators + Interactive** (not World). Any **interactive** logon on a multi-user host can open the pipe; pipe messages have **no bearer** (contrast HTTP). Primary model is single-owner desktop. |
| Unix domain socket | Default path `/tmp/ledgerful-bridge.sock` (ledgerful interop); post-bind mode **0o600**. Residual: predictable path / bind-race if another principal owns the name first. Prefer loopback HTTP + bearer for multi-user Unix hosts. |
| LocalSystem service token | Windows service residual (see OPERATIONS service notes) |
| DPAPI seed portability | Windows-only seal; not portable cross-OS |

---

## 8. Protocol honesty (clients)

**SOT:** [PROTOCOL-COMPAT.md](PROTOCOL-COMPAT.md)

- Unenforced `api_version` in some paths — do not over-claim hard rejection.  
- Payload **Upcast** stub.  
- Bridge capture policy documented separately from vault CE.

---

## 9. Forbidden marketing language (release pack)

Do **not** claim in user-facing release prose:

- “Certified” (SOC2/ISO/GDPR)  
- “Perfect deletion”  
- “Metadata-private sync”  
- “Sandboxed third-party plugins” / “WASI-safe plugins”  
- “SQLCipher page encryption” / “Full encryption” **without** F8 qualifier  
- Shipped `doctor` product CLI (recovery export **is** shipped as of T188) 

Formal claims gate: [RELEASE-CLAIMS.md](RELEASE-CLAIMS.md) + [RELEASE-CHECKLIST.md](RELEASE-CHECKLIST.md) (T185; seed evidence was T183 `CLAIMS-CROSSCHECK.md`).

---

## 10. Related documents

| Doc | Role |
|-----|------|
| [INSTALL.md](INSTALL.md) | Install how-to with F8 + transport honesty |
| [COMPATIBILITY.md](COMPATIBILITY.md) | Platform tiers + F8 SOT |
| [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) | Backup / kit / CE drills |
| [OPERATIONS.md](OPERATIONS.md) | Ops reference (erasure, daemon, keys) |
| [CAPABILITIES.md](CAPABILITIES.md) | Feature inventory |
| Root [SECURITY.md](../SECURITY.md) | GitHub Security policy stub → this file |
