# T185 — Claims Governance, SBOM, and Release Gate (P12.7)

- **Track ID:** T185-ClaimsSbomReleaseGate
- **Phase:** P12 — Release hardening and adoption (Task 7 / final P12 rollup)
- **Status:** ✅ **Completed** (2026-08-01) — P12.7 shipped; dry-run RC evidence; soft release.yml
- **Depends on (hard):** T183 release docs + `CLAIMS-CROSSCHECK`; T184 residual register + security closeout; full CI gate (`deny` / `audit` / nextest); T179 platform smoke policy
- **Depends on (soft):** T169/T170 evaluation + dogfood artifacts for evidence index; T181 recovery evidence; T180 protocol honesty language; T182 ADR-0019 non-claims; T186 for action SHA-pin residual if not absorbed here
- **Blocks / feeds:** First honest public/tag release; P12 phase acceptance rollup; future packaging (MSI/notarization) residual track
- **Category:** RELEASE / SECURITY / DOCS
- **Stop-before:** Marketing certification / perfect-deletion / metadata-private / plugin-sandbox claims; AGPL SBOM platforms as required tools; force-push of release tags; claiming SLSA Build L3 without isolated signer infra; claiming SSDF/OpenSSF “fully compliant”
- **Deferred absorbed:** §56 F26 release SHA-pin; §57 protocol honesty residuals (F35/F36/F24 language); §61 T183 claims re-grep + version-banner CI + soft historical SQLCipher re-grep; §62 R-SLSA + residual-ID citation in claims; HANDOFF-T183-T185 platform smoke; T169 evaluation artifact index; Implementation-Plan §17 release gate honesty refresh. **Not** MSI/App Store product packaging as DoD; **not** doctor/recovery-export CLI product work; **not** #34.2 DataKey rotation; **not** T186 hermetic suite (parallel residual).
- **Review fold-in:** AI1 BS1–3 + Opp1–2; AI2 F-1..F-11 + O-1..O-8 → **A1–A12**. See §16. Rejected: AI1 “Fully Compliant” SSDF matrix; AI1 `cargo-cyclonedx` as MIT (it is **Apache-2.0**); blind `--all-features` without documenting over-inclusion.

## 1. Objective

Ship a **repeatable release gate** that makes the following true before any `v*` tag / binary distribution claim:

1. **Claims only with evidence** — every affirmative product claim maps to a doc, test, drill, or review artifact; forbidden claims are grepped out of elevated surfaces.  
2. **Supply-chain hygiene** — `cargo deny` + `cargo audit` green; SBOM generated from `Cargo.lock` with a license-safe tool; third-party notice text for binary ship.  
3. **Residual honesty** — T184 residual IDs (and T180/T181/T182 residuals that affect marketing) are cited, not silent.  
4. **Platform honesty** — T1 smoke evidence exists for every claimed primary OS; runner labels match `Docs/COMPATIBILITY.md` tiers.  
5. **P12 rollup** — checkboxes for T179–T185 closed with pointers, not vague “done.”

This track is **process + scripts + docs**. It does **not** invent product features, flip SQLCipher page encryption, or run a compliance certification program.

| After T185 | Present |
|------------|---------|
| Normative claims checklist (`Docs/RELEASE-CLAIMS.md` or track evidence) | Target |
| Evaluation / security / smoke evidence index | Target |
| SBOM generation script + version pin in ci-tooling | Target |
| THIRD-PARTY / NOTICE generation path (binary ship) | Target |
| Release checklist used on a dry-run RC (no force-push) | Target |
| Optional version-banner consistency check | Soft target |
| Optional SLSA-style provenance **attestation** on release artifacts | Soft / axis (R-SLSA) |
| MSI / notarization / App Store installers | **No** (residual packaging) |
| Formal SOC2 / ISO / GDPR / ASVS Level certification | **No** |
| SLSA Build **L3** claim without isolated signer | **No** |

## 2. Live baseline (re-scan 2026-08-01)

### 2.1 What already exists

| Asset | Location / status |
|-------|-------------------|
| Full local gate | `fmt` / `clippy -D warnings` / `nextest` / `deny` / `audit` / `ledgerful verify` |
| GHA multi-OS | `.github/workflows/ci.yml` — `windows-2025` + `ubuntu-24.04` required; `macos-15` soft |
| CI permissions | `permissions: contents: read` (T184) |
| Dependabot | `.github/dependabot.yml` cargo + github-actions (T184) |
| Action pins | **Floating majors** (`actions/checkout@v4`, `dtolnay/rust-toolchain@v1`, …) — F26 / R-CI-PIN residual |
| deny allowlist | MIT, Apache-2.0 (+ LLVM-exception), BSD-3, MPL-2.0, ISC, Unicode-3.0, Zlib, CDLA, PolyForm-NC; unknown-git **deny** |
| Tool pins | nextest 0.9.140; deny 0.20.2; audit 0.22.2 (`Docs/ci-tooling.md`) |
| Product version | Workspace `0.1.1` |
| CHANGELOG | Root `CHANGELOG.md` (Keep a Changelog); banners manual |
| Claims seed | `conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md` |
| Residual register | `conductor/tracks/trackT184-independent-security-review/residuals.md` |
| Security hub | `Docs/SECURITY-LIMITS.md` + root `SECURITY.md` |
| Evaluation map | `Docs/EVALUATION/GOVERNED-MEMORY-MVP.md` → T185 section |
| Platform handoff | `trackT179-…/evidence/HANDOFF-T183-T185.md` |
| Release workflow | **None yet** (PR CI only) |

### 2.2 Ledgerful preflight (planning day)

| Check | Result |
|-------|--------|
| `ledgerful doctor` | Ready; index OK; 0 pending / 0 drift |
| `ledgerful search "SBOM"` / claims | PROTOCOL-COMPAT honesty; EVALUATION→T185; deny.toml license posture |
| `ledgerful hotspots` | CLI command modules — **not** primary edit targets for this track (docs/scripts/CI) |
| Impact | Design-only until implement; re-run `scan --impact` before release-workflow edits |

### 2.3 Gaps T185 closes

1. No frozen **release claims checklist** consumed at tag time.  
2. No **SBOM** artifact process (only deny/audit SCA).  
3. No **THIRD-PARTY notice** generation for binary distribution.  
4. No **release checklist** binding gate + evidence + residual citation.  
5. No **version-banner** automation (CHANGELOG / docs vs `Cargo.toml`).  
6. R-SLSA: no provenance attestation path; must not overclaim.  
7. F26: release jobs should SHA-pin actions (PR floating majors remain OK).  
8. Implementation-Plan §17 still says bare “Storage is encrypted” without F8 honesty — refresh pointer or banner.

## 3. Research summary (online + ecosystem, 2026-08-01)

### 3.1 Claims / release governance (process)

| Practice | Application |
|----------|-------------|
| **Evidence-backed claims** | Map each public claim → artifact path + date/hash when practical (T169 report_hash pattern). |
| **Non-claims inventory** | Elevate T183 table + master-plan forbidden claims; re-grep elevated docs at RC. |
| **Residual citation** | Cite T184 residual IDs in RELEASE-CLAIMS / SECURITY-LIMITS rather than implying “no residual risk.” |
| **Human sign-off** | Checklist requires a named human role for claims acceptance (not agent-only). |
| **NIST SSDF** (align, do not certify) | PW.4/RV.1-style verification + residual risk documentation — language only. |

### 3.2 SBOM standards & Rust tooling

| Tool / standard | License (crates.io 2026-08) | Role | Decision |
|-----------------|----------------------------|------|----------|
| **CycloneDX** | **ECMA-424**; community + Ecma | Preferred SBOM schema | **Primary format** (`*.cdx.json`); pin generator to **spec 1.5** (see below) |
| **`cargo-cyclonedx` 0.5.9** | **Apache-2.0** (not MIT) | Official CycloneDX Cargo plugin | **Preferred generator** |
| **`cargo-sbom` 0.10.0** | **MIT** | Alternate SPDX/CycloneDX generator | Acceptable fallback |
| **Syft** (Anchore) | Apache-2.0 | Filesystem/image SBOM | Soft optional second opinion; not required |
| **SPDX** | Linux Foundation | Alternate BOM format | Soft dual-emit only if zero extra product dep pain |
| **NTIA minimum elements** | Guidance | Supplier, component name/version, unique ID, dependency relationship, author, timestamp | Aim to satisfy via CycloneDX fields from lockfile |

**CycloneDX version pin (AI2 F-1 / A1):** CycloneDX is now **ECMA-424**. Spec **1.7** is current (2025-10), but **`cargo-cyclonedx` 0.5.9 supports only 1.3–1.5**. T185 **targets `--spec-version 1.5`** (latest the tool supports). Do not claim “CycloneDX 1.6/1.7” until the generator catches up. JSON already carries `specVersion`; document it in evidence.

**Generator layout (AI2 F-2 / A2):** `cargo-cyclonedx` does **not** emit a single workspace-aggregate BOM. Modes:

| `--describe` | Behavior |
|--------------|----------|
| `crate` (default) | One `*.cdx.json` **per crate** (many files; tests/benches as subcomponents) |
| **`binaries`** | **Preferred for ship** — one BOM per bin/cdylib target |
| `all-cargo-targets` | One BOM per target (verbose) |

**Release artifact shape:** one SBOM **per shipped binary** (e.g. `ai-brains-<ver>.cdx.json`, `ai-brainsd-<ver>.cdx.json`), not one phantom workspace root file.

**Best practices folded in:**

1. Generate from **committed `Cargo.lock`**, not floating resolve.  
2. Use **`--describe binaries`** and attach BOMs for each **shipped** binary (`ai-brains`, `ai-brainsd`, optional desktop).  
3. **Feature- and target-aware (AI1 BS1):** pass the **same `--target` and feature set** used to compile the release binaries (e.g. `x86_64-pc-windows-msvc` for Windows T1). Prefer the **exact release features**, not blind `--all-features` (all-features over-includes optional graph/desktop edges — only use if deliberately documenting a superset SBOM).  
4. **`SOURCE_DATE_EPOCH` (AI2 O-8 / A11 soft):** set from `git log -1 --format=%ct` so timestamps/serials are reproducible (AGENTS determinism).  
5. Attach SBOM to **release assets** (or evidence dir for dry-run); document feature set used.  
6. Validate JSON schema optionally (soft); never fail gate solely on soft validator outage.  
7. **No AGPL** SBOM platforms as required steps.  
8. SBOM tools are **dev/CI only** — install via `cargo install --locked` (not workspace product deps).  
9. Soft: Linux CI smoke `cargo cyclonedx …` exit-0 without uploading (AI2 O-3).

### 3.3 Third-party notices

| Tool | License | Role | Decision |
|------|---------|------|----------|
| **`cargo-about` 0.9.1** | MIT OR Apache-2.0 | Generate license listing from graph | **Preferred** for `THIRD-PARTY.md` (alias: `THIRD_PARTY_LICENSES.md`) |
| `cargo-license` | MIT | Simple license table | Soft fallback |
| `cargo deny check licenses` | Already gate | Policy enforcement, not human-readable NOTICE | Keep as hard gate |

**Install + config (AI2 F-3 / A3):**

```text
cargo install --locked --features cli cargo-about
cargo about init   # creates about.toml + about.hbs — commit both
cargo about generate about.hbs > dist/THIRD-PARTY.md   # or .html if template is HTML
```

Default `about.hbs` is **HTML** — decide at implement: custom **markdown** template → `THIRD-PARTY.md`, or ship `THIRD-PARTY.html`. Commit templates for **determinism**.

**CDLA spike (AI2 F-7 / A7):** `deny.toml` allows `CDLA-Permissive-2.0`. During Phase B, check `cargo-about` for unknown-license warnings; add `about.toml` clarifications if needed.

Apache-2.0 NOTICE obligations: include required NOTICE text when distributing binaries. Generate at release time under `dist/` or GitHub Release assets.

### 3.4 SLSA / provenance (R-SLSA axis)

| Source | Application to T185 |
|--------|---------------------|
| **SLSA provenance v1** (`predicateType: https://slsa.dev/provenance/v1`) | Spec under SLSA **v1.2** docs (2026). |
| **Build L1 fields (AI2 F-11 / A10)** | Required: `buildDefinition` (`buildType` + `externalParameters`) and `runDetails` (`builder.id`). Honest “L1-oriented” only if attestation JSON contains these. |
| **Build L1 meaning** | Provenance exists; **minimal** tamper resistance. |
| **Build L2+** | Hosted platform hardening; do not claim without builder isolation story. |
| **Build L3** | Isolated signer. **Out of default DoD.** |
| **GitHub Artifact Attestations (AI2 F-5 / A5)** | **New implementations:** use **`actions/attest`** with SLSA build-provenance predicate (per action README as of v4). `actions/attest-build-provenance` is a **wrapper** for existing workflows — still OK but not preferred for greenfield `release.yml`. **SHA-pin** whichever is used (F26). Actions auto-populate L1 fields from the workflow run — implementer does not hand-write provenance. |
| **Private-repo gating (AI2 F-6 / A6)** | Attestations: **public** repos (any plan) **or** private/internal on **GitHub Enterprise Cloud**. If private without Enterprise Cloud, skip attest; set R-SLSA disposition to **not shipped, tooling unavailable** — not a silent failure. |
| **Honesty rule** | Prefer: “Release artifacts may include SLSA-style build provenance attestations (Build L1-oriented).” **Forbidden:** “SLSA 3 certified,” “tamper-proof supply chain,” or equating attestations with security certification (T184 L9 remains). |

### 3.5 OpenSSF / Scorecard alignment (release job)

| Check class | T185 action |
|-------------|-------------|
| Pinned-Dependencies | **Release** workflow SHA-pins all `uses:` (F26 / R-CI-PIN partial). PR `ci.yml` may keep floating majors until T186 or a later hygiene pass. |
| Token-Permissions | Release job opts into least write scopes (`id-token: write` only if attesting; `contents: write` only if uploading release assets). Default workflow remains read. |
| Dependency-Update-Tool | Dependabot already present (T184). |
| Security-Policy | `SECURITY.md` present. |
| SAST | Still honesty residual (clippy ≠ SAST); not T185 DoD. |
| Branch-Protection | Repo admin residual (R-CI-BRANCH); checklist item “confirm protected” not auto-fix. |

### 3.6 Version / SemVer / changelog

| Practice | Application |
|----------|-------------|
| Keep a Changelog | Already root `CHANGELOG.md` |
| SemVer 0.x | Breaking changes allowed on minor — restate on release notes |
| Version banners | Soft: workspace version vs CHANGELOG; handle **`[Unreleased]`** (see L13 / plan E1) |
| Common Changelog | Explicitly **not** required (T183 decision) |

## 4. Normative locks (L1–L13)

| ID | Lock |
|----|------|
| **L1** | **Claims-with-evidence.** Affirmative public claims for a release must appear on the claims checklist with an evidence pointer (doc path, test name, report hash, residual ID, or review disposition). |
| **L2** | **Forbidden claims.** No SOC2/ISO/GDPR/ASVS-Level certification; no perfect deletion / NIST Purge-Destroy as product property; no metadata-private sync; no third-party plugin sandbox / WASI marketplace; no inventing doctor/recovery-export CLIs; no “full DB encryption” without F8 qualifier; no UI grants authority beyond contracts; no “SSDF/OpenSSF fully compliant.” |
| **L3** | **Residual citation (full cross-walk).** RELEASE-CLAIMS must disposition **every open row** in T184 `residuals.md` (cite R-ID **or** explicit “out of scope for claims”). **Minimum cite set** (user-trust floor): R-F8, R-CE-PRE, R-ACK, R-META, R-TB, R-API-VER, R-12, R-34.2, **R-CI-SAST, R-CI-BRANCH, R-AUDIT-UNMAINT, R-PIPE-IU, R-UDS-TMP, R-ZERO-KEY, R-DESKTOP-OPEN, R-SLSA, R-DOC-CLI**. R-ZERO-KEY + F8 must co-qualify any “encrypted vault” language. |
| **L4** | **Hard SCA gate.** `cargo deny check` and `cargo audit` (exit code only, F27) **must** pass on the release commit. Unmaintained warnings (R-AUDIT-UNMAINT) do not fail audit by themselves under current policy; document delta if policy tightens. |
| **L5** | **SBOM required for binary distribution.** CycloneDX JSON via **`cargo-cyclonedx`** with **`--spec-version 1.5`**, **`--describe binaries`**, and **target/features matching the release build**. One BOM per shipped binary. Source-only tags may attach SBOM as soft. |
| **L6** | **NOTICE for binary ship.** Binary distribution includes machine-generated third-party license text (`cargo-about` preferred; committed `about.toml` + template). Source-only may omit if LICENSE + deny policy cover. |
| **L7** | **Tool licenses.** Release-gate tools must be MIT/Apache/BSD/ISC (or already-allowed deny licenses). **No AGPL/GPL required tooling.** |
| **L8** | **Platform smoke match.** Claims of T1 support require smoke evidence whose **runner label** matches COMPATIBILITY tiers (Windows `windows-2025`, Linux `ubuntu-24.04`; macOS only as documented soft pin). |
| **L9** | **SLSA honesty.** Do not claim SLSA Build L3 or “certified SLSA” without isolated provenance + documented `builder.id`. Optional L1-oriented attestations only when fields present **and** repo/plan supports attestations (public or Enterprise Cloud private). |
| **L10** | **No packaging DoD.** MSI, notarization, App Store, systemd/launchd production units are **out of scope** for T185 completion (documented residual). |
| **L11** | **Human claims sign-off.** Checklist includes a human owner field; agent automation may prepare evidence but does not alone clear L1. |
| **L12** | **No force-push / no main direct release.** Release tags from agreed branch; follow `AGENTS.md` Git rules. Dry-run RC does not require a public tag if user prefers private evidence only. Note R-CI-BRANCH: branch protection may be **unenforced** until repo admin enables it — do not claim enforced protection. |
| **L13** | **Automated non-claims scan.** Release gate runs a scripted forbidden-phrase check over elevated docs (`scripts/check-release-claims.ps1`); fails on affirmative overclaims. Complements (does not replace) human checklist review. |

## 5. Deliverables

| # | Deliverable | Path (proposed) | Notes |
|---|-------------|-----------------|-------|
| D1 | Claims checklist (normative) | `Docs/RELEASE-CLAIMS.md` | Claim/non-claim + evidence + **full residual cross-walk** + **“what we don’t ship”** section |
| D2 | Release checklist | `Docs/RELEASE-CHECKLIST.md` | Ordered gate for humans + scripts |
| D3 | Evidence index | `…/evidence/EVIDENCE-INDEX.md` | Links T169/T170/T179/T181/T183/T184; note R-CHANGELOG-PATH resolved (root CHANGELOG) |
| D4 | SBOM script | `scripts/generate-sbom.ps1` (+ optional `.sh`) | `--spec-version 1.5`, `--describe binaries`, target/features, optional `SOURCE_DATE_EPOCH` → `dist/sbom/` |
| D5 | NOTICE script | `scripts/generate-notices.ps1` (+ optional) | `cargo-about` with committed templates → `dist/THIRD-PARTY.md` (or `.html`) |
| D6 | Tool pins | `Docs/ci-tooling.md` | cargo-cyclonedx + cargo-about (`--features cli`) versions |
| D7 | Soft version-sync check | `scripts/check-version-banners.ps1` | Handle `[Unreleased]`; warn if no `## [<Cargo version>]` |
| D8 | Soft release workflow | `.github/workflows/release.yml` | SHA-pinned; soft **`actions/attest`**; private-repo skip path |
| D9 | Claims scan script + evidence | `scripts/check-release-claims.ps1` + `evidence/CLAIMS-REGREP.md` | L13 automated + dry-run log |
| D10 | Implementation-Plan §17 honesty | Banner or F8-aligned bullet | No silent “Storage is encrypted” overclaim |
| D11 | Conductor + deferred closeout | `conductor.md`, `deferred.md` §63 | Expanded → Completed on ship |
| D12 | cargo-about config | `about.toml` + `about.hbs` (or md template) | **Committed**; deterministic NOTICE |
| D13 | Soft unified gate script | `scripts/dev-release-check.ps1` | Optional: full gate + claims scan + SBOM (AI1 Opp1) |

### 5.1 Artifact layout (binary release)

```
dist/
  sbom/
    ai-brains-<version>.cdx.json      # CLI binary BOM (specVersion 1.5)
    ai-brainsd-<version>.cdx.json     # daemon binary BOM
    # + desktop BOM only if desktop binary ships
  checksums/
    SHA256SUMS
  THIRD-PARTY.md                      # or THIRD-PARTY.html / THIRD_PARTY_LICENSES.md
  # optional zip layout for GitHub Release (AI1 Opp2):
  # ai-brains-v<ver>-windows-x64.zip  # binaries + LICENSE + THIRD-PARTY
```

Checksums: SHA-256 of each published binary + SBOM + NOTICE. Prefer `Get-FileHash` / `sha256sum`.

## 6. Claims checklist content (normative outline)

### 6.1 Allowed claim classes (must attach evidence)

| Claim class | Evidence examples |
|-------------|-------------------|
| Capture independence | Capture tree test in CI; no models/graph/sync edge |
| Append-only event log | Store/event design + tests |
| CE / cryptographic erasure (live content) | T165/T181 E-01; non-claim pre-erase backups |
| Backup create/verify/restore | T181 RECOVERY-DRILLS + tests |
| Platform T1 Windows (+ Linux core) | T179 COMPATIBILITY + GHA run IDs / SMOKE evidence |
| Protocol N−1 / honesty | PROTOCOL-COMPAT + T180 suites |
| TrustedBuiltin only | ADR-0019 + tests |
| Security review performed | T184 charter + residuals (not “certified”) |
| License posture | deny.toml + PolyForm + COMMERCIAL-EXCEPTION |
| Evaluation hard gates (if claimed) | T169 report_hash + catalog; soft metrics **not** quality claims |

### 6.2 Forbidden / non-claim classes (must remain non-claims)

Mirror T183 CLAIMS-CROSSCHECK + T184 closeout language + master plan. Include R-IDs where relevant.

### 6.3 “What this release does NOT include” (AI2 O-6)

RELEASE-CLAIMS must have an explicit section listing at least: no MSI/installer; no notarization/App Store; no SLSA L3; no SAST product claim; no enforced branch protection if R-CI-BRANCH open; no doctor CLI; no recovery export CLI; no multi-user pipe auth; no post-quantum; no perfect deletion; no metadata-private sync; no third-party plugin sandbox.

### 6.4 Soft re-grep set (elevated + historical)

**Hard (fail release if affirmative overclaim — L13 script + evidence):**  
`README.md`, `Docs/ARCHITECTURE.md`, `Docs/CAPABILITIES.md`, `Docs/OPERATIONS.md`, `Docs/README.md`, `Docs/INSTALL.md`, `Docs/SECURITY-LIMITS.md`, `SECURITY.md`, `CHANGELOG.md`, new `Docs/RELEASE-*.md`.

**Soft (report only):**  
`AGENTS.md`, `Docs/PRD.md`, `Docs/Implementation-Plan.md` body, archives, completed track specs.

Patterns (illustrative): `certified`, `perfect deletion`, `metadata-private` (as product property), `plugin sandbox` (as shipped), inventing `doctor` CLI, unqualified “full encryption” / “SQLCipher encrypts the database” without F8, `fully compliant` (SSDF/ASVS).

## 7. Acceptance criteria

| # | Criterion |
|---|-----------|
| **AC1** | `Docs/RELEASE-CLAIMS.md` exists with claim/non-claim tables, evidence pointers, residual cross-walk (L1–L3), and “what we don’t ship” (§6.3). |
| **AC2** | `Docs/RELEASE-CHECKLIST.md` exists and was **executed** on a dry-run RC (record date, commit SHA, operator in evidence). |
| **AC3** | Elevated-doc claims scan clean (scripted L13); evidence in `CLAIMS-REGREP.md`. |
| **AC4** | `cargo deny check` + `cargo audit` green on dry-run commit (L4). |
| **AC5** | SBOM script produces CycloneDX JSON with **`specVersion` 1.5**, **per shipped binary**, target/features documented; tool Apache-2.0/MIT (L5, L7). |
| **AC6** | NOTICE/THIRD-PARTY path works with committed `about.toml` + template (L6); CDLA spike noted; source-only skip rule documented. |
| **AC7** | Evidence index links T169/T170 (if used), T179 smoke, T181, T183 CLAIMS-CROSSCHECK, T184 residuals. |
| **AC8** | Platform claim rows match COMPATIBILITY runner labels (L8). |
| **AC9** | SLSA language satisfies L9; R-SLSA updated (attested / skipped tooling / not claimed). |
| **AC10** | No MSI/notarization/App Store as claimed complete (L10); residual listed in deferred. |
| **AC11** | Human sign-off field filled on dry-run checklist (L11). |
| **AC12** | P12 rollup table in checklist or conductor note: T179–T185 status pointers. |
| **AC13** | Implementation-Plan §17 no longer overclaims storage encryption without F8 (D10). |
| **AC14** | Conductor status **Completed** only after AC1–AC15; deferred §63 written. |
| **AC15** | `about.toml` (+ template) committed; `cargo install --features cli cargo-about` documented in ci-tooling. |

## 8. Non-goals

| Out of scope | Owner |
|--------------|--------|
| MSI / WiX / MSIX installers | Future packaging track |
| Apple notarization / App Store | Future packaging |
| systemd / launchd production units | Ops residual |
| Branch protection enablement | Repo admin (R-CI-BRANCH) |
| Full SHA-pin of **PR** `ci.yml` | T186 or later hygiene (release workflow pins in scope if D8) |
| Hermetic assert_cmd suite | **T186** |
| Runtime `api_version` enforcement | Future protocol track (honesty only here) |
| SQLCipher page-encryption feature flip | Future crypto track |
| #34.2 DataKey rotation | Future hygiene |
| doctor / recovery export CLIs | Future product tracks |
| External compliance audit engagement | Business process |
| AGPL SCA/SBOM SaaS | Forbidden as required |

## 9. License / commercial

- Product: PolyForm Noncommercial 1.0.0 + `COMMERCIAL-EXCEPTION.md` unchanged.  
- Release checklist must remind commercial redistributors of exception limits.  
- All new scripts: project license headers not required if matching repo script style; no secret material.  
- SBOM/NOTICE may list dual-licensed crates; deny policy remains source of truth for **allowed** licenses in the graph.  
- Do not relicense crates ad hoc.

## 10. Risk register (track-local)

| Risk | Mitigation |
|------|------------|
| SBOM incomplete for feature flags | Document features used for release build; regenerate when features change |
| cargo-cyclonedx workspace quirks | Spike early; fallback cargo-sbom; pin working version |
| Overclaim via CHANGELOG marketing language | AC3 re-grep includes CHANGELOG |
| Release workflow token creep | Explicit permissions; no broad `write-all` |
| Treating dry-run as production release | Checklist distinguishes dry-run vs public tag |
| SLSA L1 oversold as L3 | L9 + review language |
| Parallel T186 scope collision | T185 only pins **release** workflow actions; leaves PR ci.yml unless user expands |

## 11. Definition of Done

- All AC1–AC15 met.  
- Dry-run RC evidence recorded (even if no public `v*` tag).  
- Full local gate green on the dry-run commit.  
- No open Critical/High introduced by release scripts/workflows.  
- Mediums fixed or deferred per AGENTS caps with ISSUES/deferred entries.  
- Ledger clean after any implement ledger transaction.  
- P12.7 marked complete; phase 12 rollup honest.

## 12. Suggested sequencing

1. `mkdir evidence/`; freeze claims checklist + residual cross-walk (docs-only).  
2. Spike `cargo-cyclonedx` (`--spec-version 1.5`, `--describe binaries`) + `cargo-about --features cli` + `about init`; pin in ci-tooling.  
3. Scripts → `dist/` layout → dry-run generation (`SOURCE_DATE_EPOCH` soft).  
4. RELEASE-CHECKLIST + EVIDENCE-INDEX + `check-release-claims.ps1`.  
5. Soft: version-banner (`[Unreleased]` rules); soft: release.yml with SHA pins + `actions/attest` if plan allows.  
6. Implementation-Plan §17 honesty.  
7. Human dry-run sign-off → review → conductor Completed.

## 13. Phase 12 acceptance rollup (consumes this track)

| Track | Role | Status entering T185 |
|-------|------|----------------------|
| T179 Compatibility Matrix | Platform tiers + smoke | ✅ Completed |
| T180 Protocol Compat Tests | Wire honesty | ✅ Completed |
| T181 Backup Recovery Drills | Recovery evidence | ✅ Completed |
| T182 Connector Sandbox Decision | ADR-0019 non-claims | ✅ Completed |
| T183 Release Documentation | Docs pack + CLAIMS-CROSSCHECK | ✅ Completed |
| T184 Independent Security Review | Charter + residuals | ✅ Completed |
| **T185 Claims + SBOM Release Gate** | Claims/SBOM/gate + rollup | 📋 Expanded |

## 14. Expand checklist (design complete when)

- [x] Online research (SBOM tools, cargo-about, SLSA v1.2 provenance, Scorecard/F26, GitHub attestations)  
- [x] Deferred absorption map (§56–§62 + handoffs)  
- [x] Ledgerful doctor/status/search/hotspots  
- [x] Locks L1–L13 + AC1–AC15  
- [x] Tool preference table with licenses  
- [x] Non-goals (MSI etc.) explicit  
- [x] AI1/AI2 fold-in (A1–A12)  
- [ ] Implement only on user go-ahead  

## 15. References

- T183 evidence: `conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md`  
- T184 residuals: `conductor/tracks/trackT184-independent-security-review/residuals.md`  
- T179 handoff: `conductor/tracks/trackT179-compatibility-matrix/evidence/HANDOFF-T183-T185.md`  
- CycloneDX / ECMA-424; generator: https://github.com/CycloneDX/cyclonedx-rust-cargo  
- SLSA provenance v1 (v1.2 docs): https://slsa.dev/spec/v1.2/build-provenance  
- GitHub Artifact Attestations: **`actions/attest`** (preferred new); `actions/attest-build-provenance` wrapper  
- OpenSSF Scorecard: Pinned-Dependencies, Token-Permissions  
- Keep a Changelog / SemVer  
- Project: `deny.toml`, `Docs/ci-tooling.md`, `Docs/SECURITY-LIMITS.md`, `COMMERCIAL-EXCEPTION.md`  

## 16. AI review fold-in (2026-08-01)

### 16.1 Accepted → amendments

| ID | Source | Fold-in |
|----|--------|---------|
| **A1** | AI2 F-1 / O-1 | Pin CycloneDX **spec 1.5**; note ECMA-424 + tool lag vs 1.6/1.7 |
| **A2** | AI2 F-2 / O-2; AI1 BS1 | Per-binary BOMs via `--describe binaries`; target/features match release (not blind `--all-features`) |
| **A3** | AI2 F-3 / O-4 | `cargo install --features cli cargo-about`; commit `about.toml` + template; HTML vs MD decision |
| **A4** | AI2 F-4 / O-5 | L3 full residual cross-walk + expanded minimum cite set |
| **A5** | AI2 F-5 / O-7 | Prefer `actions/attest` for new `release.yml` |
| **A6** | AI2 F-6 | Private-repo / non-Enterprise attestation skip → honest R-SLSA |
| **A7** | AI2 F-7 | CDLA-Permissive-2.0 spike in Phase B |
| **A8** | AI2 F-8 | Version-banner handles `[Unreleased]` |
| **A9** | AI2 F-9 | `evidence/` mkdir first |
| **A10** | AI2 F-11 | Document Build L1 required fields; auto-populated by action |
| **A11** | AI2 O-8 | Soft `SOURCE_DATE_EPOCH` in SBOM script |
| **A12** | AI1 BS2–3 / Opp1–2 | Scripted claims scan (L13); NOTICE in package path; soft `dev-release-check.ps1` + zip layout notes |

### 16.2 Rejected / partial

| Item | Disposition |
|------|-------------|
| AI1 “Fully Compliant” SSDF/OpenSSF matrix | **Reject** — align/document only; L2 forbids compliance overclaims |
| AI1 cargo-cyclonedx “MIT/Apache” | **Reject** — crate is **Apache-2.0** only |
| AI1 blind `--all-features` as MUST | **Partial** — require **matching release** features; all-features only if intentionally shipping a documented superset SBOM |
| AI2 F-10 CHANGELOG path history | **No spec change** — path already correct; optional evidence note only |
| Soft CI SBOM smoke every PR (O-3) | **Soft** — not AC; optional later |

### 16.3 Status after fold-in

**Completed** on implement (2026-08-01). Fold-in A1–A12 applied during design; implementation shipped scripts/docs/release.yml + dry-run evidence.
