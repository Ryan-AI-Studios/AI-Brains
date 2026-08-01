# T185 Evidence index

**Status:** Phase D complete (operational links); dry-run archive filled in Phase F  
**Product version:** 0.1.1  
**Date:** 2026-08-01

Normative claims: [`Docs/RELEASE-CLAIMS.md`](../../../../Docs/RELEASE-CLAIMS.md)  
Human gate: [`Docs/RELEASE-CHECKLIST.md`](../../../../Docs/RELEASE-CHECKLIST.md)

## Upstream track evidence

| Track | Role | Pointer |
|-------|------|---------|
| **T169** | Evaluation catalog + `report_hash` rules | `Docs/EVALUATION/GOVERNED-MEMORY-MVP.md` (+ evaluate report artifacts when cited) |
| **T170** | Dogfood / human review gate | `Docs/EVALUATION/SHADOW-DOGFOOD-GATE.md` |
| **T179** | Platform smoke / runner labels / handoff | `conductor/tracks/trackT179-compatibility-matrix/evidence/` (incl. `HANDOFF-T183-T185.md`); runners: `windows-2025`, `ubuntu-24.04`, soft `macos-15` |
| **T180** | Protocol honesty (api_version, N−1) | `Docs/PROTOCOL-COMPAT.md` + T180 track suites |
| **T181** | Recovery drills / backup evidence | `Docs/RECOVERY-DRILLS.md` + T181 track evidence |
| **T182** | Connector sandbox non-claims | `Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md` |
| **T183** | Claims seed / elevated re-grep | `conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md` |
| **T184** | Residual register + security closeout | `conductor/tracks/trackT184-independent-security-review/residuals.md` |

## Release docs (root / Docs)

| Doc | Role |
|-----|------|
| `Docs/RELEASE-CLAIMS.md` | Normative claims + residual cross-walk + “don’t ship” |
| `Docs/RELEASE-CHECKLIST.md` | Ordered human/script gate + P12 rollup + sign-off |
| `Docs/SECURITY-LIMITS.md` | Operator security hub |
| `Docs/COMPATIBILITY.md` | Platform tiers + **F8** vault honesty |
| `Docs/ci-tooling.md` | Gate + **release tool** pins (cyclonedx, cargo-about) |
| Root `CHANGELOG.md` | Release notes SOT — **R-CHANGELOG-PATH closed** (root path is correct; historical mis-path residual resolved in T184) |
| `SECURITY.md` | GitHub security policy stub |
| `about.toml` + `about.md.hbs` | Deterministic NOTICE generation (D12) |

## T185 local artifacts

| Artifact | Path | Status |
|----------|------|--------|
| Phase A notes | `evidence/PHASE-A-NOTES.md` | Done |
| Tool spike (B1–B9) | `evidence/TOOL-SPIKE.md` | Done |
| Claims re-grep log | `evidence/CLAIMS-REGREP.md` | Phase D (run log) |
| This index | `evidence/EVIDENCE-INDEX.md` | Done |
| Dry-run archive | `evidence/dry-run-<date>/` | Phase F |

## Scripts (Phases C–E)

| Script | Role |
|--------|------|
| `scripts/generate-sbom.ps1` (+ `.sh`) | CycloneDX 1.5 per shipped binary → `dist/sbom/` |
| `scripts/generate-notices.ps1` (+ `.sh`) | `cargo-about` → `dist/THIRD-PARTY.md` |
| `scripts/check-release-claims.ps1` | L13 elevated forbidden-phrase scan |
| `scripts/check-version-banners.ps1` | Soft Cargo vs CHANGELOG (`[Unreleased]` aware) |
| `scripts/generate-checksums.ps1` | `dist/checksums/SHA256SUMS` |
| `scripts/dev-release-check.ps1` | Soft unified wrapper |
| `.github/workflows/release.yml` | Soft tag/`workflow_dispatch` release job (SHA-pinned) |

## Notes

- Soft evaluation metrics and latency are **not** product quality claims (RELEASE-CLAIMS).
- **R-SLSA:** optional L1-oriented attest when public/Enterprise; no L3 / certified claim. See RELEASE-CLAIMS §11.
- **R-CHANGELOG-PATH:** closed — canonical changelog is root `CHANGELOG.md`.
