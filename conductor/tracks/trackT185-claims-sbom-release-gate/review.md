# T185 Internal Review

**Track:** T185 Claims + SBOM Release Gate (P12.7)  
**Reviewer:** Grok (internal, read-only)  
**Date:** 2026-08-01  
**Branch:** `track/T185-claims-sbom-release-gate`  
**Scope:** Spec L1–L13 / AC1–AC15 / D1–D13; scripts; `release.yml`; about config; evidence pack; residual cross-walk vs T184; deferred §63; Impl-Plan §17; `.gitignore`

## Verdict: PASS WITH DEFERRED P3

Core DoD for a **dry-run RC** is met: normative claims + residual cross-walk, RELEASE-CHECKLIST + executed dry-run evidence, CycloneDX **1.5** per-binary SBOMs, cargo-about NOTICE path, L13 claims scan green, deny/audit green on dry-run record, soft SHA-pinned `release.yml` without SLSA L3 / secret leaks, Impl-Plan §17 F8-honest, deferred §63 written.

No P0. No P1. **P2-1 verified_fixed** after re-review (2026-08-01). **P2-2** remains process residual for public `v*` only (accepted for dry-run). Remaining open items are **P3** only (deferred).

---

## Requirement/AC matrix

| AC | Criterion (short) | Status | Evidence |
|----|-------------------|--------|----------|
| **AC1** | `Docs/RELEASE-CLAIMS.md` claim/non-claim, residual cross-walk, “don’t ship” | **Met** | Full file: §§1–4, §6 residual walk, L3 min cite table §6.3 |
| **AC2** | `RELEASE-CHECKLIST.md` exists + executed dry-run (date, SHA, operator) | **Met** | Checklist §13 dry-run fields; `evidence/dry-run-2026-08-01/DRY-RUN.md` (date, SHA `e7195925…`, branch, operator) |
| **AC3** | Elevated L13 claims scan clean + `CLAIMS-REGREP.md` | **Met** | Script exit 0 logged in `CLAIMS-REGREP.md` (11 elevated files) |
| **AC4** | `cargo deny` + `cargo audit` green on dry-run commit | **Met** | `DRY-RUN.md` gate table exit 0 (audit exit-code only / R-AUDIT-UNMAINT noted) |
| **AC5** | SBOM script → CycloneDX JSON `specVersion` 1.5, per binary, target/features documented; tool license OK | **Met** | `generate-sbom.ps1`/`.sh`; dry-run JSON `"specVersion":"1.5"`, tool `cargo-cyclonedx` 0.5.9 Apache-2.0; features: package defaults (not `--all-features`); TOOL-SPIKE |
| **AC6** | NOTICE path + committed about config; CDLA noted; source-only skip | **Met** | `about.toml` + `about.md.hbs`; `generate-notices.ps1` `-o`; CDLA in TOOL-SPIKE + NOTICE overview; checklist source-only skip |
| **AC7** | Evidence index links T169/T170/T179/T181/T183/T184 | **Met** | `EVIDENCE-INDEX.md` upstream table (also T180/T182) |
| **AC8** | Platform claim rows match COMPATIBILITY runner labels | **Met** | RELEASE-CLAIMS §8.1 + CHECKLIST §8: `windows-2025`, `ubuntu-24.04`, soft `macos-15`; T179 SMOKE/handoff + run `30683807812` in conductor notes. Checklist run-ID blanks are incomplete fill-in only (P3-1) |
| **AC9** | SLSA language L9-honest; R-SLSA disposition | **Met** | RELEASE-CLAIMS §11; ci-tooling; soft `actions/attest` + `continue-on-error` + no L3/certified; dry-run did not publish attestations |
| **AC10** | No MSI/notarization/App Store as complete | **Met** | “does not include” + L10; deferred §63 packaging residual |
| **AC11** | Human sign-off field filled (L11) | **Partial → accept for dry-run** | Checklist §13 filled (mode, date, claims Yes, evidence path); **signature line blank**; operator is agent prepare + “human merge of PR as acceptance” (prospective). See **P2-2** |
| **AC12** | P12 rollup T179–T185 | **Met** | CHECKLIST §11 table + conductor T185 Completed note |
| **AC13** | Impl-Plan §17 no bare storage-encrypt overclaim | **Met** | §17 F8-honest bullet + RELEASE-CHECKLIST pointer (line ~2019) |
| **AC14** | Conductor Completed only after ACs; deferred §63 | **Met with process note** | `conductor.md` Completed + `deferred.md` §63; plan F6/F7 were open pending this review (P3-2) |
| **AC15** | `about.toml` (+ template) committed; cargo-about `--features cli` in ci-tooling | **Met** | Root `about.toml`, `about.md.hbs`; `Docs/ci-tooling.md` install pin 0.9.1 + `--features cli` |

### Locks L1–L13 (summary)

| Lock | Status | Note |
|------|--------|------|
| L1 claims-with-evidence | Met | RELEASE-CLAIMS allowed classes + evidence |
| L2 forbidden claims | Met | §3 + elevated scan |
| L3 residual cross-walk | Met | All T184 open rows dispositioned; min cite set complete |
| L4 SCA hard gate | Met | Dry-run deny/audit |
| L5 SBOM binary ship | Met | 1.5 per binary scripts + samples |
| L6 NOTICE binary ship | Met | cargo-about path |
| L7 tool licenses | Met | cyclonedx Apache-2.0; about MIT/Apache; no AGPL |
| L8 platform smoke labels | Met | Labels match; evidence via T179 |
| L9 SLSA honesty | Met | No L3/certified overclaim |
| L10 no packaging DoD | Met | Residual explicit |
| L11 human sign-off | Partial | See P2-2 |
| L12 no force-push / no main release | Met | Dry-run no public tag |
| L13 automated non-claims scan | Met | Script + green log; coverage gaps P3 |

### Deliverables D1–D13

| D | Path | Status |
|---|------|--------|
| D1 | `Docs/RELEASE-CLAIMS.md` | Present |
| D2 | `Docs/RELEASE-CHECKLIST.md` | Present |
| D3 | `evidence/EVIDENCE-INDEX.md` | Present |
| D4 | `scripts/generate-sbom.ps1` (+ `.sh`) | Present |
| D5 | `scripts/generate-notices.ps1` (+ `.sh`) | Present |
| D6 | `Docs/ci-tooling.md` release section | Present |
| D7 | `scripts/check-version-banners.ps1` | Present (`[Unreleased]` aware) |
| D8 | `.github/workflows/release.yml` | Present, SHA-pinned, soft attest |
| D9 | `check-release-claims.ps1` + `CLAIMS-REGREP.md` | Present |
| D10 | Impl-Plan §17 honesty | Present |
| D11 | conductor + deferred §63 | Present |
| D12 | `about.toml` + `about.md.hbs` | Present |
| D13 | `scripts/dev-release-check.ps1` | Present |

---

## Findings (P0–P3)

### P2-1 — SBOM cleanup deletes track evidence `*.cdx.json`

| Field | Value |
|-------|--------|
| **id** | P2-1 |
| **severity** | P2 |
| **status** | **`verified_fixed`** (re-review 2026-08-01) |
| **location** | `scripts/generate-sbom.ps1` (cleanup block ~L141–151); `scripts/generate-sbom.sh` (`find … -delete`) |
| **problem** | After copying BOMs to `dist/sbom/`, both scripts recursively **delete every `*.cdx.json` outside `dist/`, `target/`, and `.git/`**. Dry-run evidence lives at `conductor/tracks/trackT185-…/evidence/dry-run-2026-08-01/*.cdx.json`. Re-running the SBOM generator **wipes archived release evidence**. Same risk for any future force-added BOM under `conductor/` or docs. |
| **correction** | Exclude `conductor/`, `**/evidence/**`, or only delete paths matching the crate-local generator pattern (`*_bin_*.cdx.json` next to crates), not all `*.cdx.json`. Prefer: clean only files matching `${bin}_bin_${target}.cdx.json` under `crates/` / `apps/`. |
| **fix evidence** | Cleanup now scoped to `crates/` + `apps/` only (ps1 L141–154; sh L91–93). Dry-run SBOMs still present under `evidence/dry-run-2026-08-01/`. |

### P2-2 — L11 human sign-off is agent-prepared; signature incomplete

| Field | Value |
|-------|--------|
| **id** | P2-2 |
| **severity** | P2 (process) |
| **status** | **Accepted for dry-run**; **open until public `v*`** (not a code defect) |
| **location** | `Docs/RELEASE-CHECKLIST.md` §13 |
| **problem** | Dry-run sign-off fills operator as “Grok orchestrator (prepare) + human merge of PR as acceptance,” marks **Claims accepted (L1) = Yes**, leaves **Signature / attestation of review (human)** blank, and leaves **Binary ship?** / **Notes** empty. L11: agent may prepare evidence but **does not alone clear L1**. For a dry-run this is acceptable process debt only if a named human completes sign-off before any public `v*` / binary claim. |
| **correction** | Before public tag: named human fills signature, Binary ship Yes/No, and any exceptions. For dry-run closeout, either obtain a real human signature or explicitly mark claims “prepared — human acceptance pending PR merge” without claiming L1 cleared. |
| **re-review note** | Unchanged post-P2 fixes. Does **not** block dry-run **PASS WITH DEFERRED P3**. Must resolve before any public binary / `v*` claim. |

### P3-1 — Platform smoke GHA run IDs blank on checklist

| Field | Value |
|-------|--------|
| **id** | P3-1 |
| **severity** | P3 |
| **location** | `Docs/RELEASE-CHECKLIST.md` §8 |
| **problem** | Windows/Linux evidence pointer cells are blank (`________`). Labels are correct; T179 evidence + GHA run `30683807812` exist elsewhere but are not pasted into the checklist dry-run row. |
| **correction** | Fill run IDs / `SMOKE-windows.md` / `SMOKE-linux.md` paths on the executed checklist copy or DRY-RUN.md. |

### P3-2 — Spec/plan status drift vs conductor Completed

| Field | Value |
|-------|--------|
| **id** | P3-2 |
| **severity** | P3 |
| **location** | `spec.md` (still “Proposed / Expanded”); `plan.md` (still “Implementing”; F6/F7 open); `conductor.md` T185 ✅ Completed |
| **problem** | Track registry says Completed while track-local status headers lag; F6/F7 were intentionally open for review. Not a product overclaim, but confuses audit trail. |
| **correction** | After review clearance, set spec/plan status to Completed and check F6/F7 with review disposition. |

### P3-3 — `about.toml` accepts licenses not in `deny.toml` (incl. CC0)

| Field | Value |
|-------|--------|
| **id** | P3-3 |
| **severity** | P3 |
| **location** | `about.toml` `accepted`; `deny.toml` `[licenses].allow` |
| **problem** | `about.toml` accepts **BSD-2-Clause**, **0BSD**, and **CC0-1.0** which are **absent** from `deny.toml` allow. Dry-run NOTICE did not list those licenses (no live graph hit). SCA remains deny-gated, so a sole-CC0 dep would fail deny before ship — but the dual policy is undocumented; audit asked for an explicit CC0 note. |
| **correction** | Either align `accepted` with deny allow (plus compound dual-license note), or document in `about.toml` / ci-tooling: “wider than deny for dual-license edges only; deny remains SOT for allowed graph.” |

### P3-4 — Dry-run `SHA256SUMS` paths assume `dist/` layout; archive is flat

| Field | Value |
|-------|--------|
| **id** | P3-4 |
| **severity** | P3 |
| **location** | `evidence/dry-run-2026-08-01/SHA256SUMS` vs co-located files |
| **problem** | SUMS entries are `sbom/ai-brains-0.1.1.cdx.json`, `sbom/ai-brainsd-…`, `THIRD-PARTY.md` (relative to `dist/`). Evidence dir is flat (`ai-brains-0.1.1.cdx.json` at root). Content hashes may still match files, but `sha256sum -c` from the evidence directory fails on path layout. |
| **correction** | Archive full `dist/` tree or rewrite SUMS paths when flattening. |

### P3-5 — L13 scanner coverage gaps (recovery export; portable path seps)

| Field | Value |
|-------|--------|
| **id** | P3-5 |
| **severity** | P3 |
| **location** | `scripts/check-release-claims.ps1` |
| **problem** | (1) Spec illustrative forbidden set includes inventing **`recovery export` CLI**; rules only partially cover `ai-brains doctor`. (2) Elevated paths hardcode `Docs\…` backslashes — fine on Windows release job; fragile if script is run under pwsh on Linux. Elevated re-grep is still green for current docs (human non-claim language present). |
| **correction** | Add a high-confidence affirmative `recovery export` shipped pattern with same allow-if negation; use `Join-Path` segments instead of embedded `\` in elevated list. |

### P3-6 — Soft historical PRD still says “Storage is encrypted”

| Field | Value |
|-------|--------|
| **id** | P3-6 |
| **severity** | P3 |
| **location** | `Docs/PRD.md:1122`; `evidence/SOFT-HISTORICAL-REGREP.md` |
| **problem** | Soft re-grep correctly reports only; elevated set is clean; Impl-Plan §17 fixed. Residual historical PRD wording remains. |
| **correction** | Optional future doc pass; no release-gate block (soft set by design). |

### P3-7 — `IncludeDesktop` dead/wrong fallback in `generate-sbom.ps1`

| Field | Value |
|-------|--------|
| **id** | P3-7 |
| **severity** | P3 |
| **location** | `scripts/generate-sbom.ps1` ~L118–121 |
| **problem** | On missing desktop BOM, code assigns `Find-BinBom -BinName "ai-brains"` with comment “avoid false match” — that **is** a false match if used. Variable is unused afterward (`deskFiles` search drives behavior). Dead / confusing. |
| **correction** | Remove dead fallback or fail closed without CLI binary name reuse. |

### P3-8 — NOTICE lists many PolyForm product crates as “third-party”

| Field | Value |
|-------|--------|
| **id** | P3-8 |
| **severity** | P3 |
| **location** | dry-run `THIRD-PARTY.md` overview (~22 PolyForm crates); `about.toml` `private = { ignore = true }` |
| **problem** | Workspace members generally lack `publish = false`, so cargo-about may not treat them as private. Redistributors still get correct product license text (also in `LICENSE`); listing is slightly noisy under “third-party.” |
| **correction** | Mark product crates `publish = false` and/or refine about filters; optional. |

### P3-9 — Full workspace nextest/clippy not recorded on dry-run commit

| Field | Value |
|-------|--------|
| **id** | P3-9 |
| **severity** | P3 |
| **location** | `DRY-RUN.md` notes; plan F2 |
| **problem** | AC4 only requires deny/audit (met). Spec DoD also mentions full local gate; dry-run defers nextest/clippy/fmt to PR CI. Acceptable for docs/scripts track if PR CI is green before merge. |
| **correction** | Ensure PR CI matrix green before merge; optional local `dev-check.ps1` log in evidence. |

---

## Completeness sweep

### Residual cross-walk (T184 → RELEASE-CLAIMS)

T184 `residuals.md` open rows (30) all appear in RELEASE-CLAIMS §6.1 with disposition (cited non-claim or out-of-scope). Closed (3): R-DISCLOSURE-TL, R-CI-PERM, R-CI-DEPBOT in §6.2.

| L3 minimum cite | Present |
|-----------------|---------|
| R-F8, R-CE-PRE, R-ACK, R-META, R-TB, R-API-VER, R-12, R-34.2 | Yes |
| R-CI-SAST, R-CI-BRANCH, R-AUDIT-UNMAINT, R-PIPE-IU, R-UDS-TMP | Yes |
| R-ZERO-KEY, R-DESKTOP-OPEN, R-SLSA, R-DOC-CLI | Yes |

No missing residual IDs from the T184 register.

### CycloneDX / SLSA honesty

| Check | Result |
|-------|--------|
| Claimed generator max 1.5 (not 1.6/1.7) | Honest in ci-tooling + TOOL-SPIKE |
| Dry-run BOM `specVersion` | `"1.5"` confirmed in sample headers |
| Per-binary ship names | `ai-brains-<ver>.cdx.json`, `ai-brainsd-<ver>.cdx.json` |
| Blind `--all-features` | Not used; package defaults documented |
| SLSA L3 / certified / tamper-proof | Forbidden in claims + checklist; not asserted as shipped |
| Attest path | Soft `actions/attest` SHA-pinned; default provenance mode is SLSA build provenance when subject-path only; `continue-on-error`; skip via dispatch; “do not assert attestations exist until tagged run” |

### Scripts correctness

| Item | Result |
|------|--------|
| Fail-closed on missing tool / missing BOM / wrong specVersion | Yes (`Write-Error` / exit non-zero) |
| PowerShell notices via `cargo about … -o` (not stdout redirect) | Yes |
| Claims scan exit 1 on hits | Yes |
| Checksums fail if no dist artifacts | Yes |
| Crate-local BOM cleanup | **Scoped to crates/ + apps/** → P2-1 **`verified_fixed`** |
| No blind `--all-features` | Yes |

### `release.yml`

| Item | Result |
|------|--------|
| Every `uses:` SHA-pinned | Yes (checkout, rust-toolchain, rust-cache, attest, gh-release, upload-artifact) |
| Default `permissions: contents: read`; job elevates write scopes | Yes |
| `id-token: write` + `attestations: write` for attest | Yes |
| Tool install pins (cyclonedx 0.5.9, about 0.9.1 + cli, deny, audit) | Yes |
| No secrets / tokens in file | Yes |
| Attest honesty | Soft; continue-on-error; skip path; comments forbid L3 claim |
| Windows-first only | Intentional soft scope |

### `about.toml` vs `deny.toml`

| deny allow | about accepted |
|------------|----------------|
| MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-3-Clause, MPL-2.0, ISC, Unicode-3.0, Zlib, CDLA-Permissive-2.0, PolyForm-NC | Superset + **BSD-2-Clause, 0BSD, CC0-1.0** → **P3-3** |
| CDLA | Present in both; spike OK |

### Track evidence / gitignore (force-add)

| Pattern | Effect |
|---------|--------|
| `conductor/` | Entire track tree ignored by default |
| `*.cdx.json` | Dry-run SBOMs ignored even outside conductor |
| `/dist/` | Root release output ignored |

**Ship concern:** Spec/plan/review/evidence under `conductor/tracks/trackT185-…` and dry-run `*.cdx.json` **will not enter the git index** without `git add -f`. Reviewers/mergers must force-add intentional track evidence or the PR will lack T185 proof artifacts. Not a logic bug in product code; **release-process risk** if force-add is forgotten.

### CHANGELOG / deferred

- Root `CHANGELOG.md` `[Unreleased]` documents T185 tools/docs; `## [0.1.1]` exists (version-banner soft path).
- `deferred.md` §63 Completed with R-SLSA disposition, absorbed items, and explicit non-DoD residuals (MSI, R-CI-PIN→T186, R-CI-BRANCH, doctor/export, T186 suite).

### Placeholders / overclaims not found

- No CycloneDX 1.6/1.7 ship claim.
- No SLSA L3 / SSDF fully-compliant affirmative in elevated docs (scan + spot check).
- No “SQLCipher encrypts the database” unqualified in elevated set.
- “What this release does NOT include” present and aligned with L10 / R-IDs.

---

## Deferrable candidates

| ID | Severity | Defer? | Rationale |
|----|----------|--------|-----------|
| **P2-1** | P2 | **Fixed** → **`verified_fixed`** | Cleanup scoped to crates/ + apps/ |
| **P2-2** | P2 (process) | **No before public tag**; dry-run accepted | L11 human signature residual |
| **P3-1** | P3 | Yes | Cosmetic checklist fill-in; T179 evidence exists |
| **P3-2** | P3 | Yes | Status header hygiene after review |
| **P3-3** | P3 | Yes (≤3 deferred mediums policy N/A — these are P3) | Document or align licenses |
| **P3-4** | P3 | Yes | Evidence packaging polish |
| **P3-5** | P3 | Yes | Scanner hardening |
| **P3-6** | P3 | Yes | Soft historical PRD |
| **P3-7** | P3 | Yes | Dead code path |
| **P3-8** | P3 | Yes | NOTICE noise |
| **P3-9** | P3 | Yes | Covered by PR CI expectation |

**Suggested deferred set if implementer defers all P3s:** P3-1, P3-3, P3-4, P3-5, P3-6, P3-7, P3-8, P3-9 (and P3-2 after status flip). Append non-fixed P3s to `conductor/ISSUES.md` per AGENTS if deferred past track close.

---

## Reviewer notes (non-findings)

1. Soft `release.yml` correctly leaves PR `ci.yml` floating majors as **R-CI-PIN / T186**.
2. `actions/attest` with subject-path only defaults to SLSA build provenance generation (per action docs); combined with continue-on-error this is consistent with soft R-SLSA.
3. `SOURCE_DATE_EPOCH` soft path present in both SBOM scripts.
4. Commercial redistributor reminder present (RELEASE-CLAIMS §9 + checklist §12).
5. Cross-model Codex review (plan F7) not executed in this pass — still recommended for SECURITY/RELEASE category before public tag.

---

## Disposition for track clearance

| Gate | Status |
|------|--------|
| Critical / High (P0/P1) | None open |
| Medium (P2 code) | **None open** — P2-1 **`verified_fixed`** |
| Medium (P2 process) | **P2-2** open until public `v*` (accepted for dry-run) |
| Low (P3) | Deferrable (P3-1…P3-9) |
| **Verdict** | **PASS WITH DEFERRED P3** |

---

## Re-review after P2 fixes

**Reviewer:** Grok (internal re-review)  
**Date:** 2026-08-01  
**Branch:** `track/T185-claims-sbom-release-gate`  
**Scope:** Verify claimed P2 / hygiene fixes only; re-check claims scan posture + dry-run SBOM evidence; no product-logic edits beyond this log.

### Claimed fixes — disposition

| ID / item | Prior status | Re-review disposition | Evidence |
|-----------|--------------|----------------------|----------|
| **P2-1** SBOM cleanup wipes evidence `*.cdx.json` | open | **`verified_fixed`** | `scripts/generate-sbom.ps1` L141–154: `$searchRoots = crates, apps` then `Get-ChildItem … Filter "*.cdx.json"` + `Remove-Item` only under those roots; comment explicitly excludes `conductor/`, evidence, dist, target. `scripts/generate-sbom.sh` L91–93: `find ./crates ./apps -name '*.cdx.json' -type f -print -delete`. No recursive delete from repo root. |
| **P2-2** L11 human sign-off incomplete | open (process) | **Accepted for dry-run; open until public `v*`** | `Docs/RELEASE-CHECKLIST.md` §13 still: operator agent-prepare, **Signature** blank, **Binary ship?** / Notes empty. Not a code defect; does not reopen dry-run DoD. |
| **R2-hygiene** `check-release-claims.ps1` `$allowIf` | (scanner looseness) | **`verified_fixed`** | L57–61 comment + regex: **no** residual-ID free pass; **no** bare `without`; keep real negation / non-claim / `without\s+F8` / inventory guards (`Test-IsForbiddenInventoryLine`, forbidden-section window). |
| **R2-hygiene** `Docs/README.md` release links | missing index entry risk | **`verified_fixed`** | Header L7: **Release claims gate** → `RELEASE-CLAIMS.md` · `RELEASE-CHECKLIST.md`. Engineering table L104–105 same pair. |
| **R2-hygiene** `SECURITY-LIMITS` formal claims pointer | stale / seed-only risk | **`verified_fixed`** | `Docs/SECURITY-LIMITS.md` L127: “Formal claims gate: [RELEASE-CLAIMS.md] + [RELEASE-CHECKLIST.md] (T185; seed evidence was T183 …)”. |
| **R2-hygiene** `about.hbs` non-release HTML default | template ambiguity | **`verified_fixed`** | `about.hbs` L1 HTML comment: default cargo-about **HTML** template; release NOTICE uses **`about.md.hbs`** via `generate-notices.ps1`/`.sh` (templates confirm `about.md.hbs` only). |

### Claims scan posture (L13)

| Check | Result |
|-------|--------|
| Elevated set (11 paths) | Unchanged in `check-release-claims.ps1` L26–38 |
| Hard rules | Unchanged denylist (SOC2/ISO certified, perfect deletion, metadata-private, SLSA L3/certified, fully compliant, tamper-proof supply chain, full DB encryption, unqualified SQLCipher, plugin sandbox shipped, invented `ai-brains doctor`) |
| `$allowIf` after R2 | **Stricter** than residual-ID / bare-`without` free pass; elevated prose still uses explicit negation / forbidden-inventory framing + section windows |
| Prior green log | `evidence/CLAIMS-REGREP.md`: exit **0**, 11 elevated files clean |
| Re-run note | Full re-execution not required for static re-review; tightened `$allowIf` does not broaden false-green risk (only narrows free-pass). Residual scanner gaps remain **P3-5** (recovery-export affirmative; portable path seps) — deferred low |

**Disposition:** Claims scan remains **green** for current elevated docs (prior log + static rule/doc alignment).

### Dry-run evidence SBOMs

| Path | Present | Notes |
|------|---------|-------|
| `evidence/dry-run-2026-08-01/ai-brains-0.1.1.cdx.json` | **Yes** | Header: `"bomFormat":"CycloneDX"`, `"specVersion":"1.5"`, tool `cargo-cyclonedx` 0.5.9 |
| `evidence/dry-run-2026-08-01/ai-brainsd-0.1.1.cdx.json` | **Yes** | Co-located with CLI BOM |
| `evidence/dry-run-2026-08-01/THIRD-PARTY.md` | **Yes** | Listed in `DRY-RUN.md` + `SHA256SUMS` |
| `evidence/dry-run-2026-08-01/SHA256SUMS` | **Yes** | Paths still use `sbom/…` layout (**P3-4** deferred packaging polish) |
| `evidence/dry-run-2026-08-01/DRY-RUN.md` | **Yes** | Date 2026-08-01; SHA `e7195925…`; gates exit 0 |

**Conclusion:** Archived dry-run SBOMs **still present** under `evidence/dry-run-2026-08-01/`. P2-1 fix correctly leaves `conductor/**` outside cleanup roots, so re-running generators will not delete this archive.

### Open findings after re-review

| ID | Severity | Status | Block dry-run? |
|----|----------|--------|----------------|
| **P2-1** | P2 | **`verified_fixed`** | No |
| **P2-2** | P2 process | Open until public `v*` / human signature | **No** (dry-run accepted) |
| **P3-1** | P3 | Deferred — blank platform smoke run IDs on checklist | No |
| **P3-2** | P3 | Deferred — spec/plan status header drift vs conductor Completed | No |
| **P3-3** | P3 | Deferred — `about.toml` license superset vs `deny.toml` | No |
| **P3-4** | P3 | Deferred — flat evidence vs `sbom/` SUMS paths | No |
| **P3-5** | P3 | Deferred — L13 recovery-export + path-sep coverage | No |
| **P3-6** | P3 | Deferred — soft historical PRD “Storage is encrypted” | No |
| **P3-7** | P3 | Deferred — dead/confusing `IncludeDesktop` fallback still at `generate-sbom.ps1` ~L121 | No |
| **P3-8** | P3 | Deferred — NOTICE lists product PolyForm crates | No |
| **P3-9** | P3 | Deferred — full nextest/clippy not on dry-run commit (PR CI) | No |

### Remaining open **>low** findings

| Severity | Open after re-review |
|----------|----------------------|
| P0 / P1 | **None** |
| P2 **code** | **None** |
| P2 **process** | **P2-2** only — L11 human signature / Binary ship before public `v*` (not a product-logic defect; not dry-run blocking) |
| P3 | P3-1…P3-9 deferred |

### Re-review verdict

**PASS WITH DEFERRED P3 only.**

- All claimed **code/doc hygiene** fixes verified.  
- **P2-1** closed as **`verified_fixed`**.  
- No new P0–P2 engineering findings.  
- **P2-2** process residual acknowledged for public tag; dry-run clearance stands.  
- Deferred lows: **P3-1** through **P3-9** (append to `conductor/ISSUES.md` if track closes without fixing).

*End of re-review after P2 fixes.*  
*End of T185 internal review.*

---

## Final cross-model gate (Codex R3)

**Verdict: PASS WITH DEFERRED P3** (2026-08-01)

- All R1 P2 fixed; full local gate recorded (fmt/clippy/nextest 1710/deny/audit).
- R2 easy P3 fixed (status consistency, desktop SBOM fallback, CLAIMS-REGREP stale line).
- R3 residual P3 only: L11 signature tense tightened to Pending-until-merge; NOTICE first-party PolyForm noise deferred (§63 item 8).
- No open Critical/High/Medium code findings.
