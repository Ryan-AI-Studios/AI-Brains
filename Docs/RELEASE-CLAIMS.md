# Release claims checklist (normative)

**Product version:** 0.1.1  
**Document date:** 2026-08-01  
**Track:** T185 — Claims Governance, SBOM, and Release Gate  
**Status:** Normative for public/tag/binary distribution language (L1–L3)

This document is the **claims-with-evidence** checklist for AI-Brains release language. Affirmative public claims must appear here with an evidence pointer. Forbidden classes must remain non-claims. Residual IDs cite `conductor/tracks/trackT184-independent-security-review/residuals.md`.

| Role | Document |
|------|----------|
| Security hub (operator summary) | [SECURITY-LIMITS.md](SECURITY-LIMITS.md) |
| Platform + F8 SOT | [COMPATIBILITY.md](COMPATIBILITY.md) |
| Protocol honesty | [PROTOCOL-COMPAT.md](PROTOCOL-COMPAT.md) |
| Evaluation catalog | [EVALUATION/GOVERNED-MEMORY-MVP.md](EVALUATION/GOVERNED-MEMORY-MVP.md) |
| T183 seed | `conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md` |
| T184 residuals | `conductor/tracks/trackT184-independent-security-review/residuals.md` |
| Human gate order | [RELEASE-CHECKLIST.md](RELEASE-CHECKLIST.md) |

**Human sign-off:** Agent automation may prepare evidence; a named human owner must clear L1 on the release checklist (L11). This file alone is not a release approval.

---

## 1. Claim / non-claim boundary (two-column)

Import and expansion of T183 `CLAIMS-CROSSCHECK.md` for product version **0.1.1**.

| Claimed capability (honest) | Explicit non-claim boundary |
|-----------------------------|-----------------------------|
| Append-only event log as canonical source of truth | Not FIPS-validated crypto; not NIST Purge/Destroy |
| SQLCipher page-level vault encryption (T187 `bundled-sqlcipher-vendored-openssl`) + application-level Content Envelope (CE) AES-256-GCM + OS filesystem permissions | Not FIPS; not NIST Purge; page key ≠ content DEK; zero key still usable only with `AI_BRAINS_ALLOW_ZERO_KEY=1` (**R-ZERO-KEY** residual is escape-hatch honesty, not missing refuse) |
| Capture works offline without models, embeddings, or graph databases | Not “intelligence / brain / nightly features work without models” |
| Optional multi-device **replicate** of encrypted event envelopes via untrusted relay | Not metadata-private sync; not live SQLite file sync; ACK ≠ wipe proof (**R-META**, **R-ACK**) |
| Optional cloud models when policy allows (`allow_cloud` default **false**) | Not cloud-required capture; not cloud-required product |
| First-party **TrustedBuiltin** connectors only (ADR-0019) | Not sandboxed third-party plugins / WASI marketplace; TrustedBuiltin shares host process (**R-TB**) |
| CE wipe can make **live** vault content unreadable when wraps are destroyed | Not perfect deletion; not NIST SP 800-88 Purge/Destroy; pre-erase backups remain recoverable (**R-CE-PRE**, **R-WAL-CKPT**) |
| Backup create / verify / restore CLI suite; restore hard-fails if daemon up (T188) | |
| `ai-brains recovery export` (kit to file; no kit JSON on stdout) | Not `ai-brains doctor` CLI (**R-DOC-CLI** partial) |
| Contracts may include doctor DTO types | Not shipped `ai-brains doctor` CLI (**R-DOC-CLI**) |
| Windows **T1** primary; Linux core **T1** on documented runners; secondary OS per COMPATIBILITY tiers | Not equal multi-OS primary without tier evidence; macOS only as soft pin |
| Protocol fixtures and N−1 honesty policy (PROTOCOL-COMPAT) | Not runtime hard rejection of unknown `api_version`; not working schema Upcast migrations (**R-API-VER**) |
| Independent security review performed (T184 charter + residual register) | Not SOC2 / ISO / GDPR / ASVS-Level **certification**; not “no residual risk” |
| License posture: PolyForm Noncommercial 1.0.0 + Small-Entity Commercial Exception | Not a redistribution right beyond LICENSE + [COMMERCIAL-EXCEPTION.md](../COMMERCIAL-EXCEPTION.md) |
| Live CLI ≈ 30+ top-level commands (`ai-brains --help`) | Not Implementation-Plan §8 as a shipping checklist; not inventing missing commands |
| Evaluation hard gates (if cited) via T169 catalog + `report_hash` | Soft metrics / latency are **not** product quality claims |
| SCA gate: `cargo deny check` + `cargo audit` green on release commit | Not “no unmaintained transitive deps” (**R-AUDIT-UNMAINT**); clippy is not SAST (**R-CI-SAST**) |
| Optional SLSA-style build provenance **attestations** (Build L1-oriented) when repo/plan supports them | Not SLSA Build **L3**; not “tamper-proof supply chain”; not “SLSA certified” (**R-SLSA**) |

---

## 2. Allowed claim classes (must attach evidence)

Per T185 spec §6.1. Affirmative public language must map to at least one evidence pointer.

| Claim class | What may be claimed | Evidence examples |
|-------------|---------------------|-------------------|
| **Capture independence** | Capture path (CLI → daemon → event log) works without models, embeddings, or graph DB | Capture tree / independence tests in CI; AGENTS Capture Independence invariant |
| **Append-only event log** | Canonical store is append-only event sourcing; corrections use compensating events | Store/event design docs; event-log tests; ARCHITECTURE |
| **CE / cryptographic erasure (live content)** | Live CE wipe can render in-vault payloads unreadable when wraps destroyed | T165/T181 E-01 drills; ADR-0016; non-claim pre-erase backups (**R-CE-PRE**) |
| **Backup create / verify / restore** | Operator backup suite exists and is drilled | [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md); T181 evidence; recovery tests |
| **Platform T1 Windows (+ Linux core)** | Windows primary T1; Linux core T1 per runner labels | [COMPATIBILITY.md](COMPATIBILITY.md); T179 SMOKE evidence; GHA `windows-2025` / `ubuntu-24.04` |
| **Protocol N−1 / honesty** | Fixture-first N−1 policy; documented wire surfaces | [PROTOCOL-COMPAT.md](PROTOCOL-COMPAT.md); T180 suites; honesty for unenforced `api_version` (**R-API-VER**) |
| **TrustedBuiltin only** | v1 connectors are first-party TrustedBuiltin | ADR-0019; registry sandbox tests; SECURITY-LIMITS §5 |
| **Security review performed** | Independent review conducted; residuals registered | T184 charter + [residuals.md](../conductor/tracks/trackT184-independent-security-review/residuals.md) — **not** “certified” |
| **License posture** | PolyForm NC + commercial exception; deny allowlist for deps | `LICENSE`, `COMMERCIAL-EXCEPTION.md`, `deny.toml` |
| **Evaluation hard gates (if claimed)** | Scenario hard-gate outcomes from evaluate reports | T169 catalog in GOVERNED-MEMORY-MVP; `report_hash` on reports; soft metrics **non-claims** |
| **SCA hygiene (process)** | deny + audit pass on release commit | CI logs / dry-run evidence (T185 Phase F) |
| **SBOM / NOTICE (when binary ships)** | CycloneDX 1.5 per binary; third-party license text | T185 scripts + `dist/` artifacts (Phases B–C) |

---

## 3. Forbidden / non-claim classes

Per T185 L2 and §6.2. These must **not** appear as affirmative product claims in elevated docs or release notes.

| Forbidden / non-claim class | Residual / note |
|-----------------------------|-----------------|
| SOC2 / ISO / GDPR / ASVS-Level **certification** | L2 |
| “SSDF / OpenSSF **fully compliant**” | Align/document only; L2 |
| Perfect deletion / NIST Purge / Destroy as product property | **R-CE-PRE**, **R-WAL-CKPT** |
| Metadata-private sync | **R-META** |
| Third-party plugin sandbox / WASI marketplace as shipped | **R-TB**, ADR-0019 |
| Invented `ai-brains doctor` CLI (recovery export **is** shipped as of T188) | **R-DOC-CLI** |
| FIPS-validated / NIST Purge page encryption | T187 ships SQLCipher community + OpenSSL vendored — not FIPS/Purge |
| UI grants authority beyond contracts | Product invariant |
| SLSA Build L3 / “SLSA certified” / tamper-proof supply chain | **R-SLSA**, L9 |
| Dedicated SAST product claim (clippy ≠ SAST) | **R-CI-SAST** |
| Enforced branch protection (while **R-CI-BRANCH** open) | Repo admin residual |
| Multi-user pipe authentication / per-user pipe bearer | **R-MULTI**, **R-PIPE-IU** |
| Post-quantum cryptography as product property | **R-PQ** |
| Runtime hard `api_version` enforcement | **R-API-VER** |
| Working historical Upcast migrations | PROTOCOL-COMPAT §6 |
| Soft evaluation metrics / latency as quality claims | GOVERNED-MEMORY-MVP |
| MSI / notarization / App Store packaging complete | L10 packaging residual |
| Vendor conversational bench superiority (LoCoMo, etc.) | Evaluation limitations |

---

## 4. “What this release does NOT include”

Explicit product/process absences for version **0.1.1** (T185 §6.3). This section is intentional honesty, not a roadmap promise.

| Not included | Detail |
|--------------|--------|
| **MSI / WiX / MSIX / App Store installers** | Packaging residual; out of T185 DoD (L10) |
| **Apple notarization** | Future packaging |
| **SLSA Build L3** | No isolated signer infra; optional L1-oriented attest only (**R-SLSA**) |
| **SAST as a product/security claim** | Clippy is lint, not SAST (**R-CI-SAST**) |
| **Enforced GitHub branch protection** | **R-CI-BRANCH** open (repo admin); do not claim enforced protection |
| **`ai-brains doctor` CLI** | DTO may exist; CLI not shipped (**R-DOC-CLI**) |
| **`ai-brains recovery export` CLI** | **Shipped (T188)** — kit file + RECOVERY-DRILLS; doctor still absent (**R-DOC-CLI** partial) |
| **Multi-user pipe auth / per-user pipe bearer** | Single-owner desktop model; Interactive SID residual (**R-MULTI**, **R-PIPE-IU**) |
| **Post-quantum crypto** | Explicit non-claim (**R-PQ**, ADR-0018 L16) |
| **Perfect deletion** | CE live wipe only; backups/WAL residuals (**R-CE-PRE**, **R-WAL-CKPT**) |
| **Metadata-private sync** | Replication leaves metadata residual (**R-META**) |
| **Third-party plugin sandbox / WASI host** | TrustedBuiltin only (**R-TB**, ADR-0019) |
| **systemd / launchd production units** | Ops residual |
| **FIPS-validated page encryption / NIST Purge** | SQLCipher community + OpenSSL vendored are not FIPS/Purge claims (T187) |
| **DataKey rotation product feature** | **Implemented with residuals** (**R-34.2** / **T189** / ADR-0020) |

---

## 5. Encrypted vault language (F8 + T187 + R-ZERO-KEY)

**Normative F8 wording** (COMPATIBILITY §4 / SECURITY-LIMITS §1) — **updated T187**:

> Vault storage uses **SQLCipher page-level encryption** (`bundled-sqlcipher-vendored-openssl`) combined with **application-level Content Envelope AES-256-GCM** (P8) and OS filesystem permissions. Wrong key fails closed. Zero keys refused unless `AI_BRAINS_ALLOW_ZERO_KEY=1`. Not FIPS; not NIST Purge. Page key ≠ content DEK.

**Required qualifiers for any “encrypted vault” phrasing:**

1. **R-F8 (closed by T187)** — Page-level SQLCipher is **live** on default builds; still forbid FIPS / “perfect deletion” / Purge language.
2. **R-ZERO-KEY (partial close)** — Missing/zero key is refused at `VaultConnection` unless `AI_BRAINS_ALLOW_ZERO_KEY=1`. Do not claim “no escape hatch”; tests/legacy may set the env.
3. Application CE AES-256-GCM protects envelope payloads; OS file permissions still matter for the SQLite file and logs.
4. **R-K06 (closed by T187)** — Wrong-key fail-closed at the page layer is live (open / backup verify / recovery drills strict).

---

## 6. Full residual cross-walk (T184 → claims disposition)

Every row in T184 `residuals.md` is dispositioned below. Minimum cite set per L3 is included.

**Disposition keys:**

- **Cited as non-claim** — Must not be claimed affirmatively; residual appears in non-claim / “does not include” language.
- **Out of scope for claims** — Process, tooling, or doc residual that does not expand marketing surface; no product capability claim depends on ignoring it.
- **Closed** — Remediated or path-corrected; keep briefly for provenance.

### 6.1 Open residuals (full walk)

| Residual ID | Residual (short) | Disposition | One-line note |
|-------------|------------------|-------------|---------------|
| **R-12** | Path TOCTOU / openat / cap-std | **Implemented-with-residuals** (T190 / ADR-0021) | TrustedBuiltin **vault open+list** hardened: cap-std ambient root + component nofollow (Unix `O_NOFOLLOW`, Windows `FILE_FLAG_OPEN_REPARSE_POINT`); handle-bound read; `walk_vault` has zero `std::fs::read_dir`; Hermes/Honcho export dirs elevated. **Residuals:** ambient CLI paths; soft-canonicalize (non-claim for TOCTOU); api-server token path; T188 artifact/migrate write pre-check + post-write reparse (not vault-root read path). |
| **R-34.2** | DataKey rotation / wrap-nonce budget | **Implemented-with-residuals** | T189 / ADR-0020: `vault rotate-datakey` (export primary); residuals = multi-device per-device ceremony, offline old kits/backups, rekey opt-in crash residual, Argon2 not in kit JSON, Windows drop→MoveFileEx micro-window (OS handle close required). |
| **R-F8** | Page-level SQLCipher live (T187) | **Closed** (evidence) | `bundled-sqlcipher-vendored-openssl`; header not plain; COMPATIBILITY F8 rewritten. |
| **R-K06** | Wrong-key fail-closed at page layer | **Closed** (evidence) | T181 F-02/K-06 strict; dual-mode plain residual removed. |
| **R-CE-PRE** | Pre-erase backups remain recoverable | Cited as non-claim | Ticket/wipe ≠ destroy offline copies (T181 E-01). |
| **R-WAL-CKPT** | WAL checkpoint ≠ NIST Purge | Cited as non-claim | Store honesty; no Purge/Destroy product claim. |
| **R-ACK** | Sync ACK ≠ wipe proof | Cited as non-claim | ACK is attestation only (ADR-0018 / OPERATIONS). |
| **R-META** | Sync metadata residual | Cited as non-claim | No metadata-private sync claim. |
| **R-HTTP-SYS** | LocalSystem HTTP token vs desktop | Cited as non-claim | Service residual; single-owner / OPERATIONS honesty. |
| **R-MULTI** | Multi-user interactive pipe residual | Cited as non-claim | No multi-user pipe auth product claim; Interactive SID residual. |
| **R-PIPE-IU** | Pipe SDDL SY+BA+IU (not per-user SID) | Cited as non-claim | No per-user pipe bearer; see “does not include.” |
| **R-UDS-TMP** | UDS path under /tmp predictable | Cited as non-claim | Prefer HTTP+bearer multi-user Unix; mode 0o600 after bind. |
| **R-API-VER** | `api_version` unenforced runtime | Cited as non-claim | PROTOCOL-COMPAT honesty; presence ≠ hard reject. |
| **R-BRIDGE** | Bridge capture policy doc-vs-code | Out of scope for claims | Process residual; do not invent bridge policy product claims beyond PROTOCOL-COMPAT. |
| **R-DTO-GOLDEN** | DTO goldens / API_VERSION SOOT gaps | Out of scope for claims | Test/fixture hygiene; not a user-facing capability claim. |
| **R-DOC-CLI** | No doctor CLI; recovery export **present** (T188 partial) | Cited as non-claim for doctor | Explicit “does not include doctor”; DTO ≠ CLI; export is product. |
| **R-TB** | TrustedBuiltin shares process | Cited as non-claim | Design: no third-party plugin sandbox claim (ADR-0019 L1). |
| **R-CLOUDOK** | CloudOk unused; no trust-label gate | Out of scope for claims | Future flag residual; no CloudOk gate claim. |
| **R-EXTISM** | Wasmtime/Extism patch-lag class | Cited as non-claim / OOS v1 | Host forbidden in v1; no WASI product claim. |
| **R-OUTBOUND** | OutboundIndex empty in prod | Out of scope for claims | T156 honesty; evaluation limitation, not marketing claim. |
| **R-PQ** | Post-quantum not claimed | Cited as non-claim | Explicit non-claim (ADR-0018 L16). |
| **R-STATUS-STALE** | status.md historical demote residual | Out of scope for claims | Soft doc hygiene; re-confirm in elevated re-grep, not a product feature. |
| **R-CHANGELOG-PATH** | CHANGELOG is repo root `CHANGELOG.md` | Out of scope for claims | Path corrected (F-10); use root CHANGELOG for release notes. |
| **R-CI-PIN** | Actions pinned to major tag not SHA | Closed for product claims | **T186:** PR `ci.yml` + release.yml full SHA pins; Scorecard Pinned-Dependencies improved (not a product marketing claim). |
| **R-CI-SAST** | No dedicated SAST (clippy ≠ SAST) | Cited as non-claim | Do not claim SAST product coverage. |
| **R-CI-BRANCH** | Branch protection not enabled | Cited as non-claim | Open repo-admin residual; do not claim enforced protection. |
| **R-SLSA** | No SLSA L3 / optional L1 attest | Cited as non-claim (L3); soft L1 optional | Repo is **public** — GitHub Artifact Attestations may be enabled in soft `release.yml` (L1-oriented fields via `actions/attest`). **Forbidden:** SLSA L3, “certified SLSA,” tamper-proof supply chain. Disposition updates when attest ships or is skipped. |
| **R-ZERO-KEY** | Zero-key refuse + escape hatch honesty | Cited as residual honesty | T187 refuses zero key unless `AI_BRAINS_ALLOW_ZERO_KEY=1`; do not claim “no zero-key path.” |
| **R-DESKTOP-OPEN** | Desktop opener path residual | Cited as non-claim / honesty | Desktop README honesty; no overclaim of opener isolation. |
| **R-AUDIT-UNMAINT** | audit unmaintained transitive warnings | Out of scope for claims | Gate still green on exit code; document if policy tightens (L4). |

### 6.2 Closed residuals (brief)

| Residual ID | Residual (short) | Status | Note |
|-------------|------------------|--------|------|
| **R-DISCLOSURE-TL** | SECURITY.md numeric disclosure timeline | **Closed** | T184 F-9; SECURITY.md 90-day timeline. |
| **R-CI-PERM** | CI workflow least-privilege `permissions:` | **Closed** | T184 F-5; `ci.yml` `contents: read`. |
| **R-CI-DEPBOT** | Dependabot/Renovate config | **Closed** | T184 F-7; `.github/dependabot.yml`. |

### 6.3 L3 minimum cite set checklist

| ID | Present in §6.1 |
|----|-----------------|
| R-F8 | Yes |
| R-CE-PRE | Yes |
| R-ACK | Yes |
| R-META | Yes |
| R-TB | Yes |
| R-API-VER | Yes |
| R-12 | Yes |
| R-34.2 | Yes |
| R-CI-SAST | Yes |
| R-CI-BRANCH | Yes |
| R-AUDIT-UNMAINT | Yes |
| R-PIPE-IU | Yes |
| R-UDS-TMP | Yes |
| R-ZERO-KEY | Yes |
| R-DESKTOP-OPEN | Yes |
| R-SLSA | Yes |
| R-DOC-CLI | Yes |

---

## 7. Evaluation evidence pointers

**SOT catalog:** [EVALUATION/GOVERNED-MEMORY-MVP.md](EVALUATION/GOVERNED-MEMORY-MVP.md)

| Rule | Detail |
|------|--------|
| **Hard gates only (if claimed)** | Cite scenario hard-gate outcomes from evaluate reports. Catalog scenarios 1–10 (and any later active rows) are the human-readable inventory. |
| **`report_hash`** | Hex SHA-256 of canonical report JSON with `created_at` and all `latency_ms` stripped; scenarios sorted by id. Prefer attaching path + hash in evidence index at dry-run. |
| **Soft metrics** | `citation_coverage`, `budget_compliant`, `latency_ms`, and other soft-gated metrics are **not** product quality claims. |
| **Limitations (non-claims)** | Synthetic fixtures only; no LoCoMo/LongMemEval/BEAM superiority; no LLM-as-judge; scenario 8 CE wipe ≠ NIST Purge; OutboundIndex empty in prod (**R-OUTBOUND**). |
| **T170 dogfood** | Human review / shadow dogfood is separate evidence ([SHADOW-DOGFOOD-GATE.md](EVALUATION/SHADOW-DOGFOOD-GATE.md)); do not equate dogfood pass with certification. |

T185 Phase D evidence index will link concrete report paths when dry-run artifacts exist.

---

## 8. Platform, protocol, and sandbox non-claims

### 8.1 Platform (T179 / F8)

| Topic | Honesty |
|-------|---------|
| Windows T1 | Primary; evidence bar includes full local gate + required GHA `windows-2025` |
| Linux core T1 | Required GHA `ubuntu-24.04` (+ WSL smoke evidence as documented); desktop excluded from required Linux CI |
| macOS | Soft pin only (e.g. `macos-15`); not equal primary without evidence |
| Runner labels | Release platform claims must match COMPATIBILITY tier labels (L8) |
| Vault encryption | F8 wording only (**R-F8**, **R-ZERO-KEY**) |
| Models / VRAM | Environment-specific; not OS-tier guarantees |

### 8.2 Protocol (T180)

| Topic | Honesty |
|-------|---------|
| `api_version` | Often present/serialized; **not** validated/rejected at runtime today (**R-API-VER**) |
| Upcast | Stub (`UnknownVersion` for historical); forward-compat is R0 Unknown, not migrations |
| Bridge vs daemon | Bridge captures unknown payload shapes; opposite of daemon unknown-type fail-closed |
| N−1 | Fixture-first policy until public `v*` tag; do not claim full runtime enforcement |

### 8.3 Connectors / sandbox (T182 / ADR-0019)

| Topic | Honesty |
|-------|---------|
| v1 model | **TrustedBuiltin only** |
| Third-party plugins / marketplace | **Not shipped** |
| WASI / Wasmtime / Extism host | **Forbidden** in v1 product claims (**R-EXTISM**, **R-TB**) |
| Process isolation of built-ins | TrustedBuiltin **shares** host process — not a sandboxed plugin claim |

---

## 9. COMMERCIAL-EXCEPTION redistributor reminder

Product license: **PolyForm Noncommercial 1.0.0** (`LICENSE`) with **Small-Entity Commercial Exception** (`COMMERCIAL-EXCEPTION.md`).

| Reminder | Meaning |
|----------|---------|
| **Noncommercial default** | PolyForm NC governs unless the Exception applies. |
| **Qualified Small Entity** | Internal Business Use only under Exception revenue/eligibility rules (see Exception definitions). |
| **No blanket redistribute/OEM** | Exception does **not** grant rights to offer the Software, offer derivatives, provide hosted access to third parties, or distribute products incorporating substantial portions. |
| **Commercial redistribute** | Requires a **separate written OEM/redistribution agreement** with the Licensor (Ledgerful, LLC). |
| **Release assets** | Binary ship must include license texts as required (project LICENSE + generated THIRD-PARTY / NOTICE when distributing binaries — T185 L6). |
| **Do not relicense crates ad hoc** | Dependency licenses remain subject to `deny.toml` allowlist. |

Release operators and commercial redistributors must read `COMMERCIAL-EXCEPTION.md` in full before any paid or redistribution use.

---

## 10. Elevated vs soft re-grep file sets

Automated forbidden-phrase scan (L13 / `scripts/check-release-claims.ps1` when present) complements human review.

### 10.1 Elevated (hard — fail release on affirmative overclaim)

| Path |
|------|
| `README.md` |
| `Docs/ARCHITECTURE.md` |
| `Docs/CAPABILITIES.md` |
| `Docs/OPERATIONS.md` |
| `Docs/README.md` |
| `Docs/INSTALL.md` |
| `Docs/SECURITY-LIMITS.md` |
| `SECURITY.md` |
| `CHANGELOG.md` |
| `Docs/RELEASE-*.md` (including this file and RELEASE-CHECKLIST) |

### 10.2 Soft (report only)

| Path |
|------|
| `AGENTS.md` / `Agents.md` / `Claude.md` |
| `Docs/PRD.md` |
| `Docs/Implementation-Plan.md` body |
| `Docs/archive/**` |
| Completed track specs under `conductor/tracks/**` |

### 10.3 Illustrative forbidden patterns (affirmative form)

- `certified` (compliance)
- `perfect deletion`
- `metadata-private` as a product property
- `plugin sandbox` / WASI-safe plugins as shipped
- Inventing shipped `doctor` CLI (recovery export is real as of T188)
- Unqualified “full encryption” / “SQLCipher encrypts the database” without F8
- `fully compliant` (SSDF / ASVS / OpenSSF)

Mentions as **non-claims**, residual IDs, or explicit negations are allowed.

---

## 11. R-SLSA disposition (release provenance)

| Axis | Disposition for 0.1.1 |
|------|------------------------|
| **SLSA Build L3** | **Non-claim** — no isolated signer; out of default DoD |
| **“SLSA certified” / tamper-proof chain** | **Forbidden** (L9) |
| **Optional L1-oriented attestation** | Allowed soft language when attestations exist and contain Build L1 fields (`buildDefinition` with `buildType` + `externalParameters`; `runDetails.builder.id`) |
| **Repo visibility** | Repository is **public** — GitHub Artifact Attestations are available |
| **Private without Enterprise Cloud** | If that ever applies: skip attest; disposition **not shipped, tooling unavailable** — not silent failure |
| **Workflow path (T185 E3)** | Soft `.github/workflows/release.yml` ships SHA-pinned **`actions/attest@v2.4.0`** (`ce27ba3b…`) on Windows release binaries; `continue-on-error: true`; dispatch can set `skip_attest`. **Still:** no L3 / certified / tamper-proof claims. Do not assert attestations exist until a tagged run produces them. |

Preferred honest phrase when attestations ship:

> Release artifacts may include SLSA-style build provenance attestations (Build L1-oriented).

---

## 12. Evidence pointer index (seed)

Phase D completes the operational evidence index. Seed links:

| Area | Pointer |
|------|---------|
| T183 claims seed | `conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md` |
| T184 residuals | `conductor/tracks/trackT184-independent-security-review/residuals.md` |
| T179 platform / handoff | `conductor/tracks/trackT179-compatibility-matrix/evidence/` (incl. HANDOFF-T183-T185) |
| T180 protocol | [PROTOCOL-COMPAT.md](PROTOCOL-COMPAT.md) |
| T181 recovery | [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) + T181 evidence |
| T182 sandbox | [DECISIONS/ADR-0019-connector-sandbox-execution-model.md](DECISIONS/ADR-0019-connector-sandbox-execution-model.md) |
| T169 evaluation | [EVALUATION/GOVERNED-MEMORY-MVP.md](EVALUATION/GOVERNED-MEMORY-MVP.md) |
| T170 dogfood | [EVALUATION/SHADOW-DOGFOOD-GATE.md](EVALUATION/SHADOW-DOGFOOD-GATE.md) |
| Security hub | [SECURITY-LIMITS.md](SECURITY-LIMITS.md) |
| Compatibility / F8 | [COMPATIBILITY.md](COMPATIBILITY.md) |
| T185 evidence | `conductor/tracks/trackT185-claims-sbom-release-gate/evidence/` |

---

## 13. Document control

| Item | Value |
|------|-------|
| Version context | Product **0.1.1** |
| Locks satisfied | L1 (claims-with-evidence), L2 (forbidden classes), L3 (full residual cross-walk + minimum cite set) |
| AC1 | This file: claim/non-claim tables, evidence pointers, residual cross-walk, “what we don’t ship” |
| Updates | Re-grep elevated set at each RC; refresh residual dispositions when T184 register changes; update R-SLSA when attest path ships |

**End of RELEASE-CLAIMS.md**
