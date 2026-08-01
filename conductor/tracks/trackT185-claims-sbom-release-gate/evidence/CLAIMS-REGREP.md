# T185 D3 — Claims re-grep log (L13)

**Date:** 2026-08-01  
**Product version:** 0.1.1  
**Script:** `scripts/check-release-claims.ps1`  
**Commit context:** track/T185-claims-sbom-release-gate (implement Phases B–E)

## Elevated set (hard)

| File | Result |
|------|--------|
| `README.md` | clean |
| `Docs/ARCHITECTURE.md` | clean |
| `Docs/CAPABILITIES.md` | clean |
| `Docs/OPERATIONS.md` | clean |
| `Docs/README.md` | clean |
| `Docs/INSTALL.md` | clean |
| `Docs/SECURITY-LIMITS.md` | clean (forbidden inventory bullets allowed) |
| `SECURITY.md` | clean |
| `CHANGELOG.md` | clean |
| `Docs/RELEASE-CLAIMS.md` | clean (illustrative forbidden patterns allowed) |
| `Docs/RELEASE-CHECKLIST.md` | clean |

```text
.\scripts\check-release-claims.ps1
=== check-release-claims.ps1 ===
  Root: C:\dev\AI-Brains
  Elevated files: 11
[OK] No forbidden affirmative claims in 11 elevated file(s)
exit 0
```

### Scanner notes

- Denylist + **allow-if-line-matches** negation / residual / non-claim context.
- Forbidden **inventory** bullets (quoted/backticked list items under “Forbidden …” sections) are not treated as affirmative product claims.
- Patterns cover: SOC2/ISO certified, perfect deletion, metadata-private, SLSA L3 / certified, fully compliant, tamper-proof supply chain, full DB encryption, unqualified SQLCipher DB encrypt, plugin sandbox shipped, invented `ai-brains doctor` as shipped.

## Soft historical (report only)

Surfaces: `AGENTS.md` / `Agents.md` / `Claude.md`, `Docs/PRD.md`, `Docs/Implementation-Plan.md` body.

| Finding | Disposition |
|---------|-------------|
| Implementation-Plan §17 bare “Storage is encrypted.” | **Fixed in Phase D/E** → F8-honest bullet + pointer to COMPATIBILITY / SECURITY-LIMITS / RELEASE-CLAIMS + RELEASE-CHECKLIST |
| `Docs/PRD.md:1122` “Storage is encrypted and recoverable…” | **Report only** (soft set); not elevated; residual historical PRD wording — no product code change; optional future doc pass |
| Other soft-surface hits | None material beyond above |

## Follow-up

- Re-run at each RC / public tag (RELEASE-CHECKLIST §3).
- Phase F dry-run sign-off recorded on RELEASE-CHECKLIST §13 (L11: human acceptance via PR squash-merge; Binary ship = No).
