# AI-Brains documentation index

**Product version:** 0.1.5 (`Cargo.toml` workspace)
**Platform matrix:** [COMPATIBILITY.md](COMPATIBILITY.md)
**Track status (live):** [`conductor/conductor.md`](../conductor/conductor.md)
**Security limits hub:** [SECURITY-LIMITS.md](SECURITY-LIMITS.md) · root [SECURITY.md](../SECURITY.md)
**Release claims gate:** [RELEASE-CLAIMS.md](RELEASE-CLAIMS.md) · [RELEASE-CHECKLIST.md](RELEASE-CHECKLIST.md)

This index is the **single entry point** for operators and developers. Prefer it over browsing orphan historical files.

AI-Brains follows [Diátaxis](https://diataxis.fr/): how-to and tutorial-style paths for first success; reference for lookup; explanation for understanding. Research and historical audits live at the bottom so they do not block install.

---

## Quick start

1. [Install & first vault](INSTALL.md) (Windows-first how-to)
2. [Operations reference](OPERATIONS.md) (full CLI/ops)
3. [Workflows](WORKFLOWS.md) (recipes)
4. [Capabilities](CAPABILITIES.md) (feature inventory)
5. [Security limits](SECURITY-LIMITS.md) (honest non-claims)

```powershell
# Recommended full CLI (includes graph) — INSTALL primary SOOT
cargo install --path crates/ai-brains-cli --locked --features graph

# Capture-first vault check (works without models/graph; slim build is fine too)
$vault = Join-Path $env:TEMP "aibrains-first\vault.db"
ai-brains --vault-path $vault init
ai-brains --vault-path $vault preflight --summary
```

For a **slim** graph-off binary: `cargo install --path crates/ai-brains-cli --locked` — then `ai-brains graph *` exits **2** with `FEATURE_UNAVAILABLE`. GitHub Release `ai-brains.exe` is currently graph-off; see [INSTALL.md](INSTALL.md). Capture and lexical recall work without graph.

---

## Diátaxis map

| Type | Need | Primary homes |
|------|------|----------------|
| **Tutorial / first success** | Learn by doing | Root [README.md](../README.md) quick start · [INSTALL.md](INSTALL.md) first vault |
| **How-to** | Goal-oriented steps | [INSTALL.md](INSTALL.md) · [WORKFLOWS.md](WORKFLOWS.md) · [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) |
| **Reference** | Accurate lookup | [OPERATIONS.md](OPERATIONS.md) · [CAPABILITIES.md](CAPABILITIES.md) · [CLI-EXIT-CODES.md](CLI-EXIT-CODES.md) · CLI `--help` · [PROTOCOL-COMPAT.md](PROTOCOL-COMPAT.md) · [COMPATIBILITY.md](COMPATIBILITY.md) · [ci-tooling.md](ci-tooling.md) |
| **Explanation** | Why it is this way | [ARCHITECTURE.md](ARCHITECTURE.md) · [PRD.md](PRD.md) · [MEMORY-CONTROL-PLANE-VISION.md](MEMORY-CONTROL-PLANE-VISION.md) · [DECISIONS/](DECISIONS/) · [Deviations.md](Deviations.md) |

---

## Seven release topics (P12.5 pack)

| # | Topic | Primary home | Also read |
|---|-------|--------------|-----------|
| 1 | Installation & local-only mode | [INSTALL.md](INSTALL.md) | [OPERATIONS.md](OPERATIONS.md) §1 · [COMPATIBILITY.md](COMPATIBILITY.md) |
| 2 | Source / provenance model | [CAPABILITIES.md](CAPABILITIES.md) · [ARCHITECTURE.md](ARCHITECTURE.md) | ADR-0011, T149 fingerprints (ops) |
| 3 | Agent permissions | [OPERATIONS.md](OPERATIONS.md) (policy / governed CLI) | UI has **no** extra authority beyond CLI grants |
| 4 | Correction & review | [OPERATIONS.md](OPERATIONS.md) (`review`) · [WORKFLOWS.md](WORKFLOWS.md) | Compensating events; ADR-0011 |
| 5 | Retention / erasure limits | [SECURITY-LIMITS.md](SECURITY-LIMITS.md) | [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) · OPERATIONS erasure |
| 6 | Optional cloud processing | [SECURITY-LIMITS.md](SECURITY-LIMITS.md) | CAPABILITIES models; `allow_cloud` default **false** |
| 7 | Sync / multi-device threat | [SECURITY-LIMITS.md](SECURITY-LIMITS.md) · [ADR-0018](DECISIONS/ADR-0018-encrypted-event-replication-protocol.md) | OPERATIONS multi-device; **`replicate`** not Ledgerful `sync` |

---

## Non-claims (read before marketing or security review)

| Claim you might expect | Reality |
|------------------------|---------|
| Page-level SQLCipher live (T187) | **Yes** — `bundled-sqlcipher-vendored-openssl` + app-level Content Envelope AES-256-GCM + OS permissions. **Not** FIPS / NIST Purge. **Copy:** [COMPATIBILITY.md §4 (F8)](COMPATIBILITY.md) |
| Sandboxed third-party plugins / WASI marketplace | **No** — release connectors are first-party **`TrustedBuiltin` only** ([ADR-0019](DECISIONS/ADR-0019-connector-sandbox-execution-model.md)) |
| Perfect deletion / NIST Purge·Destroy | **No** — CE wipe is envelope-backed; pre-erase backups remain recoverable |
| Metadata-private multi-device sync | **No** — optional untrusted relay; metadata residual; ACK ≠ wipe proof (ADR-0018) |
| SOC2 / ISO / GDPR certified | **No** |
| `ai-brains doctor` product CLI | **Shipped (T192)** — read-only health matrix; residual = offline kit without `--kit-path` |
| `ai-brains recovery export` CLI | **Shipped (T188)** — kit to file only; [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) |
| Unix CLI already defaults to HTTP | **No** — live Unix daemon transport is **UDS**; portable path is loopback HTTP+bearer |

Full executive summary: [SECURITY-LIMITS.md](SECURITY-LIMITS.md).

---

## Platform & protocol honesty

| Doc | Role |
|-----|------|
| [COMPATIBILITY.md](COMPATIBILITY.md) | OS tiers, smoke evidence, **F8 vault encryption SOT** |
| [PROTOCOL-COMPAT.md](PROTOCOL-COMPAT.md) | N−1 wire, unenforced `api_version`, Upcast stub, Bridge capture |
| [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) | Backup / kit / CE residual drills |
| [ADR-0016](DECISIONS/ADR-0016-content-envelope-cryptography.md) | Content envelope crypto |
| [ADR-0018](DECISIONS/ADR-0018-encrypted-event-replication-protocol.md) | Encrypted event replication |
| [ADR-0019](DECISIONS/ADR-0019-connector-sandbox-execution-model.md) | Connector sandbox decision |

---

## Three different “sync” words

| Surface | Meaning |
|---------|---------|
| `ai-brains sync` | **Ledgerful bridge** import / structured records |
| `ai-brains safety sync` | Pin repository **hotspot** safety signals |
| `ai-brains replicate` / `device` | Optional **multi-device** replication (ADR-0018); not SQLite file sync |

---

## Engineering & process

| Doc | Role |
|-----|------|
| [RELEASE-CLAIMS.md](RELEASE-CLAIMS.md) | **Normative** claim / non-claim checklist + residual cross-walk (T185) |
| [RELEASE-CHECKLIST.md](RELEASE-CHECKLIST.md) | Ordered human + script release gate (dry-run / `v*` tag) |
| [Implementation-Plan.md](Implementation-Plan.md) | Historical master plan — **§8 CLI surface may list commands never built**; live CLI = `ai-brains --help` + conductor |
| [status.md](status.md) | **Historical** freeze (T72 / 2026-06-02) — not live status |
| [ci-tooling.md](ci-tooling.md) | CI tool pins (+ SBOM / NOTICE tools) |
| Root [CONTRIBUTING.md](../CONTRIBUTING.md) | Contributor gate, license, conductor/ledgerful, changelog policy |
| [../packaging/reference/README.md](../packaging/reference/README.md) | Reference systemd / launchd units (not product Unix install) |
| [hooks.md](hooks.md) / harness hook docs | Adapter integration notes |
| Root [CHANGELOG.md](../CHANGELOG.md) | Keep a Changelog |
| Root [README.md](../README.md) | Product intro |

---

## Research / historical (not on the install path)

| Doc | Note |
|-----|------|
| [RESEARCH/](RESEARCH/) | Comparative research (e.g. memory systems 2026-07) |
| Hook research (`*-Hooks-Research.md`, Gemini/Claude/Codex/Opencode) | Historical harness notes |
| [Audit.md](Audit.md) · [audit2.md](audit2.md) | Point-in-time audits — **stale**; see banners |
| [archive/](archive/) | Closed phase review artifacts |
| [antigravity-memory-review.md](antigravity-memory-review.md) | Historical |

---

## Version & license

- **SemVer:** AI-Brains follows Semantic Versioning. **While at 0.x, minor version bumps may include breaking changes.**
- **License:** PolyForm Noncommercial 1.0.0 (`LICENSE`) + [COMMERCIAL-EXCEPTION.md](../COMMERCIAL-EXCEPTION.md).
- Version banners in docs are maintained manually until a release-gate track automates them.
