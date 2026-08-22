# Security limits (honest hub)

One-page executive summary of **what AI-Brains does and does not claim** for operators, security reviewers, and release gates.
Product version **0.1.2**. Index: [README.md](README.md).

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
| DataKey rotation ceremony | **Yes** (T189 / ADR-0020 `vault rotate-datakey`); multi-device = per-device residual; not auto-rotate; not Purge of offline kits |

Also: [Deviations.md](Deviations.md) §1 (resolved T187) · [ADR-0016](DECISIONS/ADR-0016-content-envelope-cryptography.md) · [ADR-0020](DECISIONS/ADR-0020-datakey-rotation.md)

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

**SOT:** [ADR-0019](DECISIONS/ADR-0019-connector-sandbox-execution-model.md) · [ADR-0021](DECISIONS/ADR-0021-path-capability-open.md)

| Topic | Honesty |
|-------|---------|
| v1 release model | **`TrustedBuiltin` only** |
| Third-party plugin host / marketplace | **Not shipped** |
| “Sandboxed plugins” / “WASI isolation” marketing | **Forbidden** until a reviewed host lands |
| Policy (grants) still applies | TrustedBuiltin is not privilege escalation |
| Vault path open / TOCTOU (T190 + **T193**) | **Connector vault open+list hardened** (cap-std + component nofollow; no ambient `std::fs::read` fallback). **Write SOOT (T193):** shared `cap_open` CreateNew\|Replace nofollow (temp-rename replace; never Windows TRUNCATE+OPEN_REPARSE); elevated P0 artifact / token / recovery kit + P1 migrate/shadow/dogfood/evaluate operator reports. Handle-bound hardlink refuse. **Not** claimed: product-wide path TOCTOU closed; soft-canonicalize as security open; parent `create_dir_all` chain; ambient CLI long-tail (adapters/context/elevation/discovery/cozo_proxy); perfect all-API Windows TOCTOU. |

---

## 6. Recovery kit & doctor residuals

| Product surface | Status |
|-----------------|--------|
| `ai-brains recovery export` CLI | **Shipped (T188)** — kit JSON to restricted file only; passphrase file or zero-echo TTY; no `--passphrase` argv |
| Recovery kit **library** unlock → open chain | Documented / drilled in [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) (K-05 = primitive; export is operator CLI) |
| `ai-brains doctor` CLI | **Shipped (T192)** — read-only health matrix (vault / cipher / backup / kit event / optional kit file / daemon info / zero-key honesty / optional integrity) |
| `contracts::doctor` DTO | Live product report shape (`schema_version=1`) used by the CLI |
| Restore while daemon up | **Hard-fail** (T188); robust probe; `--force` never overrides |

Operator practice: RECOVERY-DRILLS + `ai-brains backup` suite + `recovery export` + `doctor` (optional `--kit-path`). Residual: offline kit without `--kit-path` remains operator responsibility; doctor does not invent a default kit path.

---

## 7. Multi-user / machine residuals

**Product fence (ADR-0022):** single-owner desktop / single-vault. Not multi-user-safe IPC. Wire `principal_id` is a **policy label**, not pipe/HTTP authentication.

| Residual | Disposition (T195) | Note |
|----------|--------------------|------|
| Loopback HTTP + bearer | Residual if token shared | Opt-in; owner-only token file |
| **R-PIPE-IU** Named-pipe IPC (Windows) | **(b) opt-in harden + residual** | Default SDDL **SY+BA+IU** (not World). Any **interactive** logon can open the pipe; no pipe bearer. Opt-in `AI_BRAINS_PIPE_ACL=service-only` → SY+BA only (interactive CLI expects **NotRunning** against SYSTEM service pipe). Pipe name `\\.\pipe\ledgerful-bridge` unchanged. |
| **R-MULTI** | **(c) permanent fence** | No multi-user pipe auth / per-user pipe bearer product claim. See [ADR-0022](DECISIONS/ADR-0022-single-owner-daemon-ipc-fence.md). |
| **R-UDS-TMP** Unix domain socket | **(a)/(b) mitigate + residual** | Shared resolver: absolute `AI_BRAINS_DAEMON_SOCKET` → valid `$XDG_RUNTIME_DIR` (0700, uid==euid; not created by us) → `/tmp/ledgerful-bridge.sock` + warn. Post-bind **0o600**. Pre-bind/shutdown unlink only owned sockets. Residual: `/tmp` fallback (esp. macOS when XDG unset); not TOCTOU-closed under `/tmp`. Prefer loopback HTTP + bearer on multi-user Unix hosts. |
| **R-HTTP-SYS** LocalSystem service HTTP | **(a)/(b) mitigate + residual** | Service host **refuses** HTTP unless `AI_BRAINS_HTTP_SERVICE` is truthy (`1`/`true`/`yes`). Opt-in keeps SYSTEM-profile token residual (not Session 1 desktop-readable). Interactive `ai-brainsd --http` unchanged. |
| DPAPI seed portability | Residual | Windows-only seal; not portable cross-OS |

**Forbidden marketing:** “multi-user safe,” “per-user pipe isolation,” “service HTTP ready for desktop clients,” “UDS TOCTOU-closed under /tmp.”

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
- Auto-remediation or inventing default recovery kit paths (doctor is read-only; kit path is explicit)

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
