# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Stability note:** AI-Brains follows Semantic Versioning. **While at 0.x, minor version bumps may include breaking changes.**

Version banners in documentation are maintained manually from the workspace `Cargo.toml` version until a release-gate track automates checks.

> **Note on Common Changelog:** a stricter “Common Changelog” style is intentionally **not** used here yet — it is incompatible with Keep a Changelog’s separate **Security** / **Deprecated** categories and top-level **Unreleased** section for our release process.

---

## [Unreleased]

### Added

- **T187 SQLCipher page encryption (live):** workspace `rusqlite` uses `bundled-sqlcipher-vendored-openssl` (+ `backup`, `fallible_uint`); plain-header sniff + `LegacyPlaintextVault`; `ai-brains vault encrypt` via `sqlcipher_export`; zero-key refuse unless `AI_BRAINS_ALLOW_ZERO_KEY=1`; `SqlCipherKey::validate` / `is_zero`; `PRAGMA cipher_version` smoke; Perl prereq docs + `dev-check.ps1`.
- Claims / SBOM release gate (P12.7 / T185): `Docs/RELEASE-CLAIMS.md`, `Docs/RELEASE-CHECKLIST.md`, `scripts/generate-sbom.ps1`, `scripts/generate-notices.ps1`, `scripts/check-release-claims.ps1`, `scripts/check-version-banners.ps1`, `scripts/generate-checksums.ps1`, `scripts/dev-release-check.ps1`, soft SHA-pinned `.github/workflows/release.yml`, `about.toml` + `about.md.hbs`.
- Release documentation pack (P12.5): `Docs/README.md` index, `Docs/INSTALL.md`, `Docs/SECURITY-LIMITS.md`, root `SECURITY.md`, claims cross-check evidence.
- Compatibility matrix and platform honesty docs (`Docs/COMPATIBILITY.md`) with GHA multi-OS soft/required jobs (P12.1).
- Protocol compatibility documentation and N−1 / honesty notes (`Docs/PROTOCOL-COMPAT.md`) (P12.2).
- Backup, restore, and recovery-kit drills playbook (`Docs/RECOVERY-DRILLS.md`) (P12.3).
- Connector sandbox decision **ADR-0019** — v1 release = `TrustedBuiltin` only (P12.4).
- Governed memory control-plane surfaces (briefings, progressive query, policy, review, erasure, retention, migrate/evaluate/dogfood).
- Optional multi-device enrollment and file-relay replication CLI (`device`, `replicate`) per ADR-0018.
- Optional loopback HTTP daemon API (bearer, default off).

### Changed

- **T187:** F8 docs/claims flipped to live SQLCipher page encryption (COMPATIBILITY, SECURITY-LIMITS, RELEASE-CLAIMS, ARCHITECTURE, CAPABILITIES, INSTALL, OPERATIONS, Deviations §1 resolved). Not FIPS/Purge.
- Elevated docs reworded for F8 vault encryption honesty (historical pre-T187 wording retained only in archive tracks).
- Operations and status docs demoted/banners where historical CLI counts drifted.
- Backup create keys source connection; backup list surfaces key failures; T181 dual-mode plain residual branches removed from recovery drills.

### Security

- **T187:** Wrong-key fail-closed at page layer; zero-key refuse by default; legacy plain vaults require explicit `vault encrypt` (no silent auto-encrypt).
- Documented honest non-claims: no perfect deletion, no metadata-private sync, no third-party plugin sandbox, no invented `doctor` / `recovery export` product CLIs.
- Independent security review (P12.6 / T184): charter + residual register; named-pipe SDDL hardened to SYSTEM+Administrators+Interactive (not World); Unix UDS post-bind mode `0o600`; CI least-privilege `permissions:` + Dependabot; SECURITY.md 90-day disclosure timeline.

---

## [0.1.1] — 2026-06 (workspace pin)

Baseline workspace version at documentation pack time. Historical track completion through P12.4 is recorded in `conductor/conductor.md` rather than fully restated here.

### Note

Prior 0.1.x development history lived primarily in conductor tracks and git history. This changelog is seeded for release discipline going forward; expand with dated release sections when cutting ship tags.
