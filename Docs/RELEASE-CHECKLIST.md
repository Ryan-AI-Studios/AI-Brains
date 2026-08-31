# Release checklist (human + scripts)

**Product version context:** workspace `Cargo.toml` → currently **0.1.5**
**Track:** T185 — Claims Governance, SBOM, and Release Gate
**Normative claims:** [RELEASE-CLAIMS.md](RELEASE-CLAIMS.md)
**Tool pins:** [ci-tooling.md](ci-tooling.md)

This is the **ordered gate** for a dry-run RC or a public `v*` tag. Agent automation may prepare evidence; a **named human** must clear sign-off (L11). Do **not** force-push tags or claim MSI/App Store packaging.

---

## 0. Mode

| Mode | Public `v*` tag? | GitHub Release assets? | Notes |
|------|------------------|------------------------|-------|
| **Dry-run RC** | No (preferred for first pass) | Optional private archive under track `evidence/dry-run-<date>/` | Records commit SHA + operator without publishing |
| **Public tag / binary ship** | Yes (`v0.1.2` style) | Yes when binaries ship | Full L5 SBOM + L6 NOTICE + claims scan required |

**R-CI-BRANCH honesty:** branch protection may be **unenforced** until a repo admin enables it. Do **not** claim “protected main” solely because this checklist exists.

**No MSI claim:** installers (MSI/WiX/MSIX), notarization, and App Store packages are **out of scope** (L10 packaging residual).

---

## 1. Preflight

- [ ] `ledgerful doctor` healthy (or record why skipped)
- [ ] `ledgerful ledger status` clean / understood pending
- [ ] Working tree intentional (no secret `.env` / credentials)
- [ ] Confirm **mode** (dry-run vs public tag)
- [ ] Confirm product version from `[workspace.package]` matches intended release notes

```powershell
# PowerShell — use ; not &&
git status
git rev-parse HEAD
```

---

## 2. Full local CI gate

Prefer:

```powershell
.\scripts\dev-check.ps1
```

Or the soft unified wrapper (gate + SBOM + NOTICE + claims + checksums):

```powershell
.\scripts\dev-release-check.ps1
# artifacts only:
.\scripts\dev-release-check.ps1 -SkipGate
```

Manual equivalent:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
```

- [ ] `fmt` pass
- [ ] `clippy -D warnings` pass
- [ ] `nextest` workspace pass
- [ ] `cargo deny check` pass (**L4**)
- [ ] `cargo audit` exit 0 only (**L4**, F27 — do not grep for summary line)
- [ ] `ledgerful verify` (full or scope required by process)

---

## 3. Claims scan (L13)

```powershell
.\scripts\check-release-claims.ps1
```

Elevated set: `README.md`, `Docs/ARCHITECTURE.md`, `Docs/CAPABILITIES.md`, `Docs/OPERATIONS.md`, `Docs/README.md`, `Docs/INSTALL.md`, `Docs/SECURITY-LIMITS.md`, `SECURITY.md`, `CHANGELOG.md`, `Docs/RELEASE-CLAIMS.md`, `Docs/RELEASE-CHECKLIST.md`.

- [ ] Script exit 0
- [ ] Human spot-check of [RELEASE-CLAIMS.md](RELEASE-CLAIMS.md) residual cross-walk still accurate
- [ ] No forbidden: SOC2/ISO certified, perfect deletion as product, metadata-private sync, SLSA L3, “fully compliant,” unqualified full DB encryption, doctor auto-remediation / invented default kit path (doctor is shipped T192 as read-only)

Soft historical re-grep (report only): `AGENTS.md`, `Docs/PRD.md`, `Docs/Implementation-Plan.md` body, archives.

---

## 4. Version banners (soft)

```powershell
.\scripts\check-version-banners.ps1
# public tag hard mode:
.\scripts\check-version-banners.ps1 -Strict
```

- [ ] Soft-warn reviewed
- [ ] At **public** release: rename `## [Unreleased]` → `## [<version>] — <date>` and ensure workspace version matches

---

## 5. SBOM (L5 — required for binary distribution)

```powershell
.\scripts\generate-sbom.ps1
# optional desktop BOM:
.\scripts\generate-sbom.ps1 -IncludeDesktop
```

- [ ] `cargo-cyclonedx` **0.5.9+** (Apache-2.0) available
- [ ] `dist/sbom/ai-brains-<ver>.cdx.json` present, `specVersion` **1.5**
- [ ] `dist/sbom/ai-brainsd-<ver>.cdx.json` present, `specVersion` **1.5**
- [ ] Target/features documented (default: host MSVC / package defaults; **not** blind `--all-features`)
- [ ] Source-only tag: SBOM optional/soft

---

## 6. THIRD-PARTY / NOTICE (L6 — required for binary ship)

```powershell
.\scripts\generate-notices.ps1
```

- [ ] `cargo-about` **0.9.1+** with `--features cli`
- [ ] `dist/THIRD-PARTY.md` non-empty
- [ ] Committed `about.toml` + `about.md.hbs` used
- [ ] Source-only: may omit if LICENSE + deny policy cover

---

## 7. Checksums

```powershell
.\scripts\generate-checksums.ps1
```

- [ ] `dist/checksums/SHA256SUMS` includes SBOM + NOTICE (+ binaries if present)

---

## 8. Platform smoke (L8)

Runner labels must match [COMPATIBILITY.md](COMPATIBILITY.md) tiers:

| Platform | Runner label | Required? | Evidence pointer |
|----------|--------------|-----------|------------------|
| Windows T1 | **`windows-2025`** | Yes | T179 GHA run **30683807812** (PR #51) · `conductor/tracks/trackT179-compatibility-matrix/evidence/` |
| Linux core T1 | **`ubuntu-24.04`** | Yes | T179 GHA run **30683807812** (PR #51) · same handoff |
| macOS soft | **`macos-15`** | Soft (`continue-on-error`) | Soft pin only; not claimed equal T1 primary |

- [x] Windows smoke evidence present for claimed T1
- [x] Linux smoke evidence present for claimed T1
- [x] macOS not over-claimed beyond soft pin

Handoff: `conductor/tracks/trackT179-compatibility-matrix/evidence/HANDOFF-T183-T185.md`

---

## 9. Residual review

- [ ] Open T184 residuals still cited in RELEASE-CLAIMS (not silent)
- [ ] Minimum cite set (L3) still covered: R-F8, R-CE-PRE, R-ACK, R-META, R-TB, R-API-VER, R-12, R-34.2, R-CI-SAST, R-CI-BRANCH, R-AUDIT-UNMAINT, R-PIPE-IU, R-UDS-TMP, R-ZERO-KEY, R-DESKTOP-OPEN, R-SLSA, R-DOC-CLI
- [ ] **R-SLSA:** L3 non-claim; optional L1-oriented attest only if produced; no “certified SLSA”
- [ ] **R-CI-BRANCH:** do not claim enforced branch protection if still open
- [ ] Packaging residual: no MSI/notarization/App Store as complete

Register: `conductor/tracks/trackT184-independent-security-review/residuals.md`

---

## 10. Soft release workflow (optional)

When using `.github/workflows/release.yml` on `v*` tags / `workflow_dispatch`:

- [ ] All `uses:` **SHA-pinned** (F26) — floating majors forbidden **in this file**
- [ ] Permissions least-privilege (`contents: write` for assets; `id-token: write` only if attesting)
- [ ] Attest via SHA-pinned `actions/attest` only on public (or Enterprise Cloud private) repos
- [ ] If attest skipped: record R-SLSA “tooling unavailable / skipped” — not silent
- [ ] PR `ci.yml` third-party `uses:` remain full SHA pins (T186 / R-CI-PIN; Dependabot bumps)

---

## 11. P12 phase rollup (T179–T185)

| Track | Role | Status | Pointer |
|-------|------|--------|---------|
| T179 | Compatibility matrix + smoke | ✅ | `conductor/tracks/trackT179-compatibility-matrix/` |
| T180 | Protocol compat honesty | ✅ | `Docs/PROTOCOL-COMPAT.md` |
| T181 | Backup / recovery drills | ✅ | `Docs/RECOVERY-DRILLS.md` |
| T182 | Connector sandbox ADR-0019 | ✅ | `Docs/DECISIONS/ADR-0019-…` |
| T183 | Release documentation + CLAIMS-CROSSCHECK | ✅ | `conductor/tracks/trackT183-release-documentation/evidence/` |
| T184 | Independent security review + residuals | ✅ | `…/trackT184-…/residuals.md` |
| T185 | Claims + SBOM release gate | 📋 / ✅ | this checklist + track evidence |

- [ ] Rollup reviewed for honesty (no vague “all done” without pointers)

---

## 12. Commercial / license reminder

- [ ] Redistributors of binaries include product `LICENSE` + generated THIRD-PARTY text
- [ ] Commercial redistributors re-read `COMMERCIAL-EXCEPTION.md` limits
- [ ] No ad-hoc crate relicense

---

## 13. Human sign-off (L11)

| Field | Value |
|-------|-------|
| **Mode** | **dry-run** (no public `v*` tag; no binary distribution claim) |
| **Prepared by (agent)** | Grok orchestrator — evidence + elevated L13 scan |
| **Human owner (L1)** | Repo owner (Ryan) — acceptance via **squash-merge of the T185 PR** after CI green |
| **Date (UTC)** | 2026-08-01 |
| **Commit SHA** | Recorded in `evidence/dry-run-2026-08-01/DRY-RUN.md` at capture; PR head supersedes for merge |
| **Product version** | 0.1.1 |
| **Claims prepared (agent)** | **Yes** — RELEASE-CLAIMS residual cross-walk + `check-release-claims.ps1` exit 0 |
| **Claims accepted (L1 human)** | **Yes upon PR squash-merge** (this dry-run does not publish marketing claims alone) |
| **Evidence archive** | `conductor/tracks/trackT185-claims-sbom-release-gate/evidence/dry-run-2026-08-01/` |
| **Binary ship?** | **No** (dry-run samples only under track evidence; no GitHub Release assets) |
| **Notes / exceptions** | Full workspace gate recorded in DRY-RUN.md; soft PRD “Storage is encrypted” remains soft historical (non-elevated) |

**Signature / attestation of review (human):** **Pending** — becomes accepted when the repo owner squash-merges PR `track/T185-claims-sbom-release-gate` → `main` after CI green (L11 dry-run process).

---

## Quick script index

| Script | Role |
|--------|------|
| `scripts/dev-check.ps1` | Full CI gate |
| `scripts/dev-release-check.ps1` | Soft wrapper: gate + SBOM + NOTICE + claims + checksums |
| `scripts/generate-sbom.ps1` / `.sh` | CycloneDX 1.5 per shipped binary |
| `scripts/generate-notices.ps1` / `.sh` | `cargo-about` → `dist/THIRD-PARTY.md` |
| `scripts/check-release-claims.ps1` | L13 elevated re-grep |
| `scripts/check-version-banners.ps1` | Soft Cargo vs CHANGELOG |
| `scripts/generate-checksums.ps1` | `dist/checksums/SHA256SUMS` |

**End of RELEASE-CHECKLIST.md**
