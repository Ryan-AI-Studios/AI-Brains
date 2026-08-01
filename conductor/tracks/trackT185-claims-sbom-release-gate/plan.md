# T185 Plan — Claims Governance, SBOM, and Release Gate

**Status:** ✅ **Completed** (2026-08-01) — Phases A–F done; internal + Codex review loop clean  
**Spec:** [spec.md](./spec.md) (§16 fold-in)  
**Category:** RELEASE / SECURITY / DOCS  
**Ledger category (when implement):** `RELEASE` or `SECURITY` + `DOCS`

## Phase overview

| Phase | Name | Outcome |
|-------|------|---------|
| **A** | Claims freeze | RELEASE-CLAIMS + residual cross-walk + “don’t ship” |
| **B** | Tooling spike | cargo-cyclonedx 1.5 binaries + cargo-about cli + about.toml |
| **C** | Scripts + dist layout | SBOM + NOTICE + claims-scan + checksums |
| **D** | Checklists + evidence | RELEASE-CHECKLIST, EVIDENCE-INDEX, re-grep log |
| **E** | Soft automation | Version-banner; optional release.yml + attest |
| **F** | Dry-run RC + closeout | Human sign-off; Impl-Plan §17; conductor Completed |

---

## Phase A — Claims freeze (docs)

- [x] **A0** Create `conductor/tracks/trackT185-claims-sbom-release-gate/evidence/` directory (AI2 F-9)
- [x] **A1** Import T183 `CLAIMS-CROSSCHECK.md` two-column table into `Docs/RELEASE-CLAIMS.md`
- [x] **A2** **Full residual cross-walk:** every open row in T184 `residuals.md` → cite in RELEASE-CLAIMS **or** “out of scope for claims” (AI2 F-4 / O-5). Minimum cite set per L3 (includes R-CI-SAST, R-CI-BRANCH, R-AUDIT-UNMAINT, R-PIPE-IU, R-UDS-TMP, R-ZERO-KEY, R-DESKTOP-OPEN, R-SLSA, …)
- [x] **A3** Add evaluation evidence pointers (T169 catalog + report_hash rules; soft metrics non-claims)
- [x] **A4** Add platform / protocol / sandbox non-claims (T179 F8, T180 api_version honesty, ADR-0019)
- [x] **A5** Add COMMERCIAL-EXCEPTION redistributor reminder section
- [x] **A6** Define elevated vs soft re-grep file sets (spec §6.4)
- [x] **A7** Add **“What this release does NOT include”** section (AI2 O-6 / spec §6.3)
- [x] **A8** Qualify “encrypted vault” language with F8 + **R-ZERO-KEY** where relevant

**Exit:** RELEASE-CLAIMS draft reviewable; no product code.

---

## Phase B — Tooling spike (local)

- [x] **B1** Install `cargo-cyclonedx` **0.5.9** (Apache-2.0): `cargo install --locked cargo-cyclonedx`
- [x] **B2** Spike: `cargo cyclonedx --format json --spec-version 1.5 --describe binaries` (+ release `--target` / features). Record actual output filenames for `ai-brains` / `ai-brainsd`
- [x] **B3** Confirm JSON `specVersion` is `"1.5"`; note tool max is 1.5 vs CycloneDX 1.7 / ECMA-424 (tool lag)
- [x] **B4** If B2 fails: try `cargo-sbom` 0.10.0 (MIT) and record fallback in ci-tooling — primary path works; fallback documented only
- [x] **B5** Install cargo-about: `cargo install --locked --features cli cargo-about` (AI2 F-3 — **cli feature required**)
- [x] **B6** `cargo about init`; commit **`about.toml` + `about.md.hbs`** (markdown → `THIRD-PARTY.md`)
- [x] **B7** Generate sample NOTICE; check for **CDLA-Permissive-2.0** / unknown-license warnings (AI2 F-7)
- [x] **B8** Confirm neither tool is AGPL; pin versions in `Docs/ci-tooling.md`
- [x] **B9** Optional: try `SOURCE_DATE_EPOCH` from `git log -1 --format=%ct` for reproducible SBOM timestamps
- [ ] **B10** `ledgerful scan --impact` before editing CI/scripts (implement day — operator / later)

**Exit:** Known-good commands + pins; spike notes in `evidence/TOOL-SPIKE.md`.

---

## Phase C — Scripts + artifact layout

- [x] **C1** `scripts/generate-sbom.ps1` — PowerShell first; `$ErrorActionPreference = "Stop"`; `;` separators
  - `--spec-version 1.5 --describe binaries`
  - target/features matching release build (document; **not** blind `--all-features`)
  - soft `SOURCE_DATE_EPOCH`
  - copy BOMs to `dist/sbom/ai-brains-<ver>.cdx.json`, `ai-brainsd-<ver>.cdx.json`
- [x] **C2** Optional `scripts/generate-sbom.sh` for Linux release runners
- [x] **C3** `scripts/generate-notices.ps1` (+ optional `.sh`) using committed about template
- [x] **C4** `scripts/check-release-claims.ps1` — forbidden-phrase scan of elevated set; exit non-zero on hit (L13 / AI1 BS2)
- [x] **C5** Emit under `dist/`; gitignore bulk `/dist/` + `*.cdx.json`; dry-run samples under track `evidence/`
- [x] **C6** Checksum helper → `dist/checksums/SHA256SUMS`
- [x] **C7** Document feature set / target / packages in script header or RELEASE-CHECKLIST
- [x] **C8** Soft: `scripts/dev-release-check.ps1` wrapping full gate + C1 + C3 + C4 (AI1 Opp1)

**Exit:** One-command local generation + claims scan from clean tree.

---

## Phase D — Checklists + evidence index

- [x] **D1** Write `Docs/RELEASE-CHECKLIST.md` (gate → deny/audit → claims scan → SBOM → NOTICE → platform smoke → residuals → human sign-off)
- [x] **D2** Write `evidence/EVIDENCE-INDEX.md` linking T183/T184/T179/T181/T169/T170/T180/T182; note root `CHANGELOG.md` (R-CHANGELOG-PATH closed in T184)
- [x] **D3** Run `check-release-claims.ps1`; write `evidence/CLAIMS-REGREP.md`
- [x] **D4** Soft historical re-grep (AGENTS/PRD/Impl-Plan); report only; **Impl-Plan §17 F8 honesty applied**
- [x] **D5** Platform smoke checkbox rows match runner labels (L8)
- [x] **D6** P12 rollup table in checklist (T179–T185)

**Exit:** Docs pack complete enough for dry-run.

---

## Phase E — Soft automation (optional but preferred)

- [x] **E1** Version-banner script: Cargo.toml version vs docs
  - If CHANGELOG latest is **`[Unreleased]`**, look for `## [<Cargo version>]`; **warn soft** if missing (AI2 F-8)
  - At public release, rename Unreleased → versioned section with date
- [x] **E2** Decide: version mismatch **soft-warn by default**; hard only with `-Strict` / public-tag checklist item
- [x] **E3** Optional `.github/workflows/release.yml` **shipped**:
  - trigger: `v*` tags or `workflow_dispatch`
  - **SHA-pin** every `uses:` (F26)
  - least-privilege permissions; `id-token: write` for attest
  - build release binaries (Windows-first)
  - run SBOM + NOTICE + claims scan + checksums
  - upload assets via softprops/action-gh-release (SHA-pinned)
  - soft attest: **`actions/attest@v2.4.0`** SHA-pinned; `continue-on-error`
  - public repo → attest allowed; dispatch `skip_attest` path
- [x] **E4** Do **not** require changing PR `ci.yml` floating majors (T186 residual) unless user expands scope
- [ ] **E5** Soft optional: PR CI smoke `cargo cyclonedx` exit-0 only (AI2 O-3) — **skipped** (not DoD)
- [x] **E6** Update R-SLSA disposition in RELEASE-CLAIMS / residual note

**Exit:** Soft items either shipped or explicitly deferred with reason.

---

## Phase F — Dry-run RC + closeout

- [x] **F1** Dry-run on branch `track/T185-claims-sbom-release-gate` (no force-push; no public tag)
- [x] **F2** SCA + release scripts green; full nextest/clippy deferred to PR CI matrix (same as prior doc tracks) + local deny/audit exit 0
- [x] **F3** RELEASE-CHECKLIST executed; L11 sign-off filled (dry-run mode)
- [x] **F4** Archive under `evidence/dry-run-2026-08-01/` (SBOMs + THIRD-PARTY + SHA256SUMS + DRY-RUN.md)
- [x] **F5** Implementation-Plan §17 F8-honest + RELEASE-CHECKLIST pointer
- [x] **F6** Internal review of docs/scripts; fix Critical/High/Mediums per AGENTS
- [x] **F7** Cross-model read-only review (Codex R1 FAIL→fix, R2 FAIL→easy P3 fix, final R3); release.yml attest documented
- [x] **F8** `deferred.md` §63 Completed + residual packaging/T186 notes
- [x] **F9** Conductor T185 → ✅ Completed; P12.7 note
- [x] **F10** Optional: `ai-brains pin` at ledger commit

**Exit:** AC1–AC15 satisfied; track Completed after F6–F7 clean.

---

## Deferred absorption checklist

| Source | Item | Phase |
|--------|------|-------|
| §56 T179 | F26 release SHA-pin | E3 |
| §56 T179 | Platform smoke + runner label match | D5, F3 |
| §57 T180 | Honesty language in claims (api_version, Upcast, Bridge) | A4 |
| §61 T183 | CLAIMS-CROSSCHECK consumption | A1 |
| §61 T183 | Version-banner CI | E1–E2 |
| §61 T183 | Soft historical SQLCipher re-grep | D4 |
| §61 T183 | MSI/notarization residual (not DoD) | A7 / non-goal |
| §62 T184 | Residual ID full cross-walk | A2 |
| §62 T184 | R-SLSA axis | E3, E6, L9 |
| HANDOFF-T183-T185 | F8 + deny/audit exit code | D/F |
| T169 | Evaluation artifact index | D2 |
| Impl-Plan §17 | Encrypt overclaim honesty | F5 |
| AI1/AI2 | A1–A12 tool/process amendments | B–E |

**Explicitly not absorbed (remain other tracks):**

- T186 hermetic CLI suite  
- #34.2 DataKey rotation  
- doctor / recovery export product CLIs  
- R-CI-BRANCH admin action (cite only)  
- R-CI-PIN full PR workflow pin (unless E3 only release)  

---

## Verification matrix

| Check | When |
|-------|------|
| `cargo deny check` / `cargo audit` | F2 + release |
| SBOM JSON parseable; `specVersion` 1.5; per-binary files | C + F4 |
| NOTICE non-empty for release graph | C + F4 |
| Claims scan script exit 0 | C4 + D3 + F3 |
| Full nextest workspace | F2 |
| ledgerful verify full | F2 after edits |
| Manual: dry-run checklist walkthrough | F3 |

---

## Review expectations

| Severity | Rule |
|----------|------|
| Critical/High | Fix before Completed (e.g. release workflow secret leak, overclaim in RELEASE-CLAIMS) |
| Medium | Fix by default; ≤3 deferred with ISSUES |
| Low/Info | Packaging residual, soft attest skip OK |

Review log: `conductor/tracks/trackT185-claims-sbom-release-gate/review.md` (create on implement).

---

## Stop-before (restate)

Halt and ask user before:

- Publishing a public `v*` tag or GitHub Release  
- Enabling attestations that need org policy / Enterprise Cloud  
- Force-push / rewrite of tags  
- Expanding into MSI/notarization product work  
- Adding AGPL tooling  
- Claiming SLSA L3 or SSDF/OpenSSF “fully compliant”  

---

## Implement go-ahead checklist

When user says **go on T185 implement**:

1. `ledgerful doctor` + `ledgerful ledger status`  
2. `ledgerful ledger start T185-claims-sbom-release-gate --category RELEASE --message "…"`  
3. `ledgerful scan --impact`  
4. Execute phases A→F  
5. `ledgerful verify` + ledger commit on clean gate  

Until then: **design only** (this plan + spec).
