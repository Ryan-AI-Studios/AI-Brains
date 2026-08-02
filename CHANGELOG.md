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

- **T189 DataKey rotation:** `ai-brains vault rotate-datakey` (ADR-0020 Accepted). Primary path crash-safe `sqlcipher_export` + active content-DEK re-wrap + local device-private reseal + mandatory RecoveryKit re-export; opt-in `--accept-rekey-risk` for in-place `PRAGMA rekey` with snapshot restore. `SqlCipherKey::to_data_key`, `rotate_content_dek_wrap`, `DataKeyRotated` event (System + nil aggregate_id). Backup gate, daemon hard-fail, stale-key WARNING. Multi-device residual: per-device ceremony; peer wraps untouched. R-34.2 → implemented-with-residuals.
- **T188 restore safety + recovery export:** `backup restore` hard-fails (non-zero, no overwrite) when robust daemon IPC probe succeeds (`≥1s` timeout × ≥3 attempts); dry-run allowed with live-restore-will-fail notice; `--force` never overrides probe. New `ai-brains recovery export --output … [--passphrase-file] [--dry-run] [--force]` writes RecoveryKit JSON to file only (stdout: path + `dpapi: present|absent`); zero-echo TTY via `rpassword` 7.5.x; `schema_version: 1` on kits; `RecoveryKitCreated` event best-effort. Doctor CLI still absent.
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

- **T188:** Restore hard-fail while daemon reachable; recovery kit export never prints secrets; passphrase min 8; no `--passphrase` argv; export avoids `migrate()` while daemon up.
- **T187:** Wrong-key fail-closed at page layer; zero-key refuse by default; legacy plain vaults require explicit `vault encrypt` (no silent auto-encrypt).
- Documented honest non-claims: no perfect deletion, no metadata-private sync, no third-party plugin sandbox, no invented `doctor` product CLI (recovery export shipped T188).
- Independent security review (P12.6 / T184): charter + residual register; named-pipe SDDL hardened to SYSTEM+Administrators+Interactive (not World); Unix UDS post-bind mode `0o600`; CI least-privilege `permissions:` + Dependabot; SECURITY.md 90-day disclosure timeline.

---

## [0.1.1] — 2026-06 (workspace pin)

Baseline workspace version at documentation pack time. Historical track completion through P12.4 is recorded in `conductor/conductor.md` rather than fully restated here.

### Note

Prior 0.1.x development history lived primarily in conductor tracks and git history. This changelog is seeded for release discipline going forward; expand with dated release sections when cutting ship tags.
