# T183 Claims cross-check (for T185)

**Date:** 2026-08-01  
**Scope:** New release pack prose + elevated docs reworded in T183  
**Product version:** 0.1.1

## 1. Two-column claim / non-claim boundary

| Claimed capability (honest) | Explicit non-claim boundary |
|-----------------------------|-----------------------------|
| Append-only event log as canonical source of truth | Not “SQLCipher page-level encryption is live by default” |
| Bundled SQLite vault + CE AES-256-GCM + OS permissions | Not “full DB encryption” / page-level SQLCipher without F8 qualifier |
| Capture works offline without models/graph | Not “intelligence features work without models” |
| Optional multi-device **replicate** of encrypted envelopes | Not metadata-private sync; not SQLite file sync; ACK ≠ wipe proof |
| Optional cloud models when allowed | Not cloud-required capture; `allow_cloud` default false |
| First-party **TrustedBuiltin** connectors only (ADR-0019) | Not sandboxed third-party plugins / WASI marketplace |
| CE wipe can make **live** content unreadable when wraps destroyed | Not perfect deletion; not NIST Purge/Destroy; pre-erase backups recoverable |
| Backup create/verify/restore CLI suite | Not `recovery export` operator CLI (kit library + RECOVERY-DRILLS) |
| Contracts may include doctor DTO | Not shipped `ai-brains doctor` CLI |
| Windows T1 primary; secondary OS per COMPATIBILITY tiers | Not equal multi-OS primary without tier evidence |
| PolyForm NC + commercial exception | Not SOC2/ISO/GDPR certified |
| Live CLI ≈ 30+ top-level commands (`ai-brains --help`) | Not Implementation-Plan §8 as shipping checklist |

## 2. F8 grep results (elevated + pack)

Command (PowerShell): search `SQLCipher`, `Full encryption`, forbidden marketing phrases in:

`README.md`, `Docs/ARCHITECTURE.md`, `Docs/CAPABILITIES.md`, `Docs/OPERATIONS.md`, `Docs/README.md`, `Docs/INSTALL.md`, `Docs/SECURITY-LIMITS.md`, `SECURITY.md`, `CHANGELOG.md`

### Result summary

| File | SQLCipher mentions | Overclaim residual? |
|------|--------------------|---------------------|
| README.md | Honest F8 wording | **No** |
| Docs/ARCHITECTURE.md | Honest + explicit not “full encryption” | **No** |
| Docs/CAPABILITIES.md | F8-qualified | **No** |
| Docs/OPERATIONS.md | Banner + erasure non-claims + key env note | **No** (pre-existing long doc; banner fixed) |
| Docs/README.md | Non-claims table only | **No** |
| Docs/INSTALL.md | Quotes COMPATIBILITY F8 | **No** |
| Docs/SECURITY-LIMITS.md | Hub non-claims | **No** |
| SECURITY.md | Points to hub + F8 | **No** |
| CHANGELOG.md | Security non-claims in Unreleased | **No** |

**Forbidden positive claims** (`certified`, `perfect deletion`, `metadata-private` as a product claim, `plugin sandbox` as shipped, inventing doctor CLI): **none** in affirmative product-claim form. Mentions appear only as **non-claims** or residual honesty.

## 3. T185 handoff

- Re-grep elevated docs at release time.  
- Expect root `SECURITY.md` and `Docs/SECURITY-LIMITS.md`.  
- Version-banner CI sync is **out of scope for T183** (optional later).  
- Do not promote Implementation-Plan §8 phantoms into claims.
