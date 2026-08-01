# T185 Dry-run RC evidence

| Field | Value |
|-------|-------|
| Date | 2026-08-01 |
| Mode | Dry-run RC (no public `v*` tag; **Binary ship = No**) |
| Commit SHA (pre-PR worktree base at evidence capture) | e7195925f49e4df1710e607320ef433bd3a17dbd |
| Branch | track/T185-claims-sbom-release-gate |
| Prepared by | Grok orchestrator |
| Human L1 acceptance | Repo owner via squash-merge of T185 PR after CI green |
| Product version | 0.1.1 |

## Gate results (this dry-run)

| Check | Result |
|-------|--------|
| `cargo fmt --check` | exit **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit **0** |
| `cargo nextest run --workspace` | exit **0** — **1710 passed**, 1 skipped |
| `cargo deny check` | exit **0** |
| `cargo audit` | exit **0** (warnings allowed; R-AUDIT-UNMAINT) |
| `scripts/check-release-claims.ps1` | exit 0 (11 elevated files) |
| `scripts/check-version-banners.ps1` | exit 0 (Unreleased + ## [0.1.1]) |
| `scripts/generate-sbom.ps1` | exit 0; CycloneDX **specVersion 1.5**; per-binary ai-brains + ai-brainsd |
| `scripts/generate-notices.ps1` | exit 0; dist/THIRD-PARTY.md ~361 KB |
| `scripts/generate-checksums.ps1` | exit 0 under `dist/`; archive `SHA256SUMS` uses **flat** basenames for this directory |

## Platform smoke pointers (L8)

| Platform | Runner | Evidence |
|----------|--------|----------|
| Windows T1 | `windows-2025` | T179 GHA run **30683807812** (PR #51) |
| Linux T1 | `ubuntu-24.04` | same run |
| macOS soft | `macos-15` | soft pin only |

## Artifacts in this directory

- `ai-brains-0.1.1.cdx.json`
- `ai-brainsd-0.1.1.cdx.json`
- `THIRD-PARTY.md`
- `SHA256SUMS` (flat paths: `ai-brains-0.1.1.cdx.json`, `ai-brainsd-0.1.1.cdx.json`, `THIRD-PARTY.md`)

## Notes

- R-SLSA: `.github/workflows/release.yml` soft `actions/attest` job with least-write scopes split from `build-scan` (read) / `publish` (contents write on tags only); no L3 claim; dry-run did not publish attestations.
- No MSI / notarization / public tag in this dry-run.
- Multi-OS PR CI re-confirms matrix before merge.
