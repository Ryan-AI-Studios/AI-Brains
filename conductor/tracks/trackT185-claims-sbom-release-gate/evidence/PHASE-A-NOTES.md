# T185 Phase A notes — Claims freeze

**Date:** 2026-08-01  
**Product version:** 0.1.1  
**Scope:** Docs only (no product Rust, no scripts)

## Files created / updated

| Path | Action |
|------|--------|
| `Docs/RELEASE-CLAIMS.md` | **Created** — AC1 + L1–L3 claims checklist |
| `conductor/tracks/trackT185-claims-sbom-release-gate/evidence/` | **Ensured** (directory already present) |
| `conductor/tracks/trackT185-claims-sbom-release-gate/evidence/PHASE-A-NOTES.md` | **Created** (this file) |
| `conductor/tracks/trackT185-claims-sbom-release-gate/evidence/EVIDENCE-INDEX.md` | **Created** — skeleton links; Phase D completes checklists |
| `conductor/tracks/trackT185-claims-sbom-release-gate/plan.md` | **Updated** — Phase A checkboxes → `[x]` |

## Residual cross-walk

- Source: `conductor/tracks/trackT184-independent-security-review/residuals.md`
- Open residuals dispositioned: **30** (cite as non-claim or out of scope for claims)
- Closed residuals noted: **3** (R-DISCLOSURE-TL, R-CI-PERM, R-CI-DEPBOT)
- L3 minimum cite set: all **17** IDs present in RELEASE-CLAIMS §6.1 / §6.3

## Phase A checklist map

| ID | Content |
|----|---------|
| A0 | evidence/ directory |
| A1 | T183 two-column table imported + expanded |
| A2 | Full residual cross-walk |
| A3 | Evaluation evidence pointers (T169 / report_hash / soft non-claims) |
| A4 | Platform / protocol / sandbox non-claims |
| A5 | COMMERCIAL-EXCEPTION redistributor reminder |
| A6 | Elevated vs soft re-grep sets |
| A7 | “What this release does NOT include” |
| A8 | Encrypted vault qualified with F8 + R-ZERO-KEY |

## Not done in Phase A (later phases)

- Scripts (SBOM, NOTICE, claims-scan)
- RELEASE-CHECKLIST execution / dry-run
- Full EVIDENCE-INDEX checklists and CLAIMS-REGREP log
- release.yml / attest / R-SLSA disposition update after soft automation
