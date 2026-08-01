# T184 — Independent Security Review (P12.6)

- **Track ID:** T184-IndependentSecurityReview
- **Phase:** P12 — Release hardening and adoption (Task 6)
- **Status:** 📋 **Proposed / Expanded** (2026-08-01) — design ready; **AI1/AI2 fold-in applied**; implement = charter freeze + review execution on go-ahead
- **Depends on (hard):** Shipped surfaces — P6 connectors (T153–T156 + ADR-0019), P7 HTTP/IPC (T158–T161), P8 CE/keys (T162–T166 + ADR-0016), P11 sync if in release (T175–T178 + ADR-0018), models/cloud (T157), capture independence, `deny`/`audit` CI
- **Depends on (soft):** T183 **Completed** (2026-08-01) — use `Docs/SECURITY-LIMITS.md`, root `SECURITY.md`, and `evidence/CLAIMS-CROSSCHECK.md` in the reviewer packet; re-verify claims vs product during execute
- **Blocks / feeds:** T185 “security reviewed” / claims language; release residual register; remediation sub-tracks for critical/high
- **Category:** SECURITY / RELEASE
- **Stop-before:** Marketing “security reviewed,” “audited,” “certified,” or “ASVS Level N compliant” without charter + findings disposition + residual honesty; AGPL-required scanners; reviewing live vaults with real secrets
- **Deferred absorbed (seed residual register):** #12 TOCTOU; #34.2 DataKey rotation; T161 multi-user / SYSTEM HTTP token; T180 F34–F36; T181 doctor/recovery-export + K-06 + pre-erase + WAL checkpoint honesty; T182 CloudOk/TrustedBuiltin; COMPATIBILITY F8; sync ACK/metadata; OutboundIndex empty; **CI/workflow Scorecard-class gaps** (permissions, action pins, Dependabot, SAST honesty, branch protection); disclosure timeline. **Not** full pentest all OS; **not** MSI/notarization; **not** SBOM/SLSA productization (T185); **not** implement plugin host.
- **Review fold-in:** AI1 BS1–3 + Opp1–2; AI2 F-1..F-11 + O-1..O-8 → **A1–A13**. See §16.

## 1. Objective

Perform (or commission) an **independent, time-boxed, evidence-based security review** of AI-Brains release surfaces so that:

1. Critical/high findings are **fixed or release-blocked** under `AGENTS.md` Review & Severity.  
2. Accepted residuals are **explicit** with owners (not silent).  
3. Public docs/claims (T183/T185) can cite a **charter + disposition log**, not vague “we did security.”  
4. The review is **repeatable**: scope, method, independence, and out-of-scope are written first.

This track answers **what was reviewed, how, by whom, and what remains**. It does **not** invent features, flip SQLCipher page encryption, or claim formal certification.

| After T184 | Present |
|------------|---------|
| Review charter (scope, RoE, independence, time-box) | Target |
| Reviewer packet (paths, ADRs, tests, no secrets) | Target |
| Finding log (`review.md` discipline) | Target |
| Residual risk register (seeded + post-review) | Target |
| Critical/high disposition (fixed or blocked) | Target |
| Re-sync note vs T183 security hub | Target |
| Formal ASVS/SOC2/ISO certification | **No** |
| Full multi-OS red-team / pentest | **No** |
| New production deps for review tooling (unless MIT/Apache optional) | Prefer none |

## 2. Live baseline (re-scan 2026-08-01)

### 2.1 Surfaces in product (review targets)

| Surface | Primary crates / docs | Prior security work |
|---------|----------------------|---------------------|
| **Capture path** | capture, store event log, CLI ingest | Capture independence mandate; no CoT/tool logs |
| **Loopback HTTP** | `ai-brains-api-server`, daemon | T161 bearer + loopback bind; double opt-in non-loopback |
| **Named pipe / UDS IPC** | daemon, CLI | T144 SDDL; Unix UDS honesty (T179) |
| **Policy / grants** | control-plane T151 | Principal + grant matrix; UI no extra authority |
| **Connectors / path** | sources, path | Reparse refuse; TrustedBuiltin only (ADR-0019); #12 residual |
| **Models / cloud** | models, brain | T157 local-first; `allow_cloud` default false; Sealed |
| **Content envelopes** | crypto, store | ADR-0016; T163–T165; wipe = destroy wrap |
| **Recovery kit / backup** | crypto, brain, CLI | T181 drills; kit library-only; secret-leak helper |
| **Sync / replicate** | ai-brains-sync | ADR-0018; T178 F1–F28 security suite |
| **Desktop shell** | apps/desktop | Tauri capability-mirror; CSP; Isolation Windows-only residual |
| **Supply chain** | deny.toml, audit, CI | MIT/Apache/BSD allowlist; unknown-git deny; unsound/unmaintained workspace |
| **Secret scanning** | `ai-brains-security` | Bearer/key pattern scan + privacy escalate |

### 2.2 Ledgerful preflight (planning day)

| Check | Result |
|-------|--------|
| `ledgerful doctor` | Ready; index OK |
| `ledgerful ledger status` | 0 pending / 0 drift |
| `ledgerful search "bearer"` | Hits `ai-brains-security` secret detection tests |
| `ledgerful search "content envelope"` | Hits crypto/store CE paths |
| `ledgerful hotspots` | CLI `sync`, `context`, `forget`, `daemon`, `ai-brainsd` — prioritize for authz/privacy review |
| Semantic `ask` | Prefer hybrid search + ADRs over ask if completion flaky |

### 2.3 Prior threat models (elevate, do not rewrite)

| Artifact | Use in T184 |
|----------|-------------|
| T175 `threat-model.md` + ADR-0018 | Sync STRIDE SOT |
| T182 `threat-model.md` + ADR-0019 | Connector sandbox SOT |
| ADR-0016 | CE / erasure honesty |
| COMPATIBILITY F8 | Vault encryption honesty |
| OPERATIONS multi-device residuals | ACK ≠ wipe proof |
| T178 security tests | Sync proof suite — elevate findings, don’t re-prove everything |
| T181 RECOVERY-DRILLS | CE residual + kit honesty |

### 2.4 Gaps T184 closes

1. No frozen **charter** (scope / out-of-scope / independence / RoE).  
2. No **single residual register** spanning deferred security items for release.  
3. No **independent** pass (same-track implementer reviews are not T184).  
4. No link from release claims (T185) to review disposition.  
5. Soft: reviewer packet not assembled without live secrets.

## 3. Research summary (online + standards, 2026-08-01)

### 3.1 Process standards (align, do not certify)

| Source | Application to T184 |
|--------|---------------------|
| **NIST SSDF** (SP 800-218) | Map PO/PW/RV practices (charter §6.3); define requirements, review, test, residual risk. **Do not** claim SSDF conformance. |
| **OWASP ASVS 5.0.0** (2025; not 4.0.3) | Checklist inspiration only. Pin v5 chapter renumbering; map surfaces in charter §6.4. Soft optional L1 JSON filter. **Do not** claim ASVS Level N. |
| **DASVS** (desktop ASVS-inspired) | Optional lens for Tauri IPC, local storage, update path — soft if desktop in release scope. |
| **OpenSSF concise guide + Scorecard** | CI gates (deny/audit), no secrets, review before merge, vulnerability reporting; Scorecard-named checks inspire CI hygiene scope (Token-Permissions, Pinned-Dependencies, Dependency-Update-Tool, SAST, Branch-Protection, Security-Policy). Soft optional Scorecard CLI (Apache-2.0). |
| **SLSA v1.2** (v1.1 retired) | Provenance levels are **T185** scope. T184 L9 no-cert lock covers **security certification**, not supply-chain provenance claims. |
| **Product security charter practice** | Decision rights for residual risk; finding schema; qualitative severity rubric; PoC hygiene. |
| **Supply-chain SCA** | deny+audit first line; review ignored advisories; SBOM attach T185. |

### 3.2 Independence & review models

| Model | When | License / cost notes |
|-------|------|----------------------|
| **Cross-model internal** (Codex/Claude/Grok read-only) | Default for this repo culture | Must be **different** agent/session than implementer of surface under review |
| **Second human** (Ryan + external) | High-stakes release | Prefer NDA without AGPL tooling |
| **External firm / bug bounty** | Out of scope for T184 DoD | Optional later |

**Independence rule (L3):** Reviewer must not be the primary implementer of the code under review for that finding cycle. Prefer a **fresh** reviewer session (no implementer track memory). Fix-forward may be implementer; **verified_fixed** for Critical/High requires a **separate read-only** re-audit (second model/session or human) with evidence in `review.md`.

### 3.3 Method

1. **Charter freeze** (scope, assets, actors, out-of-scope, time-box, severity rules).  
2. **Automated baseline:** `cargo deny check`; `cargo audit` (+ ignore/unmaintained delta); clippy `-D warnings`; existing security suites; soft Scorecard read-only.  
3. **Document + design review:** ADRs, threat models, T183 SECURITY-LIMITS + CLAIMS-CROSSCHECK, COMPATIBILITY F8.  
4. **Targeted code review** ordered by **`ledgerful hotspots`** + charter surfaces (daemon HTTP/IPC, forget/erasure, sync, crypto, path/connectors, CI workflow).  
5. **STRIDE / ASVS 5.0.0-mapped checklist** walk (not full ASVS).  
6. **Finding log** (schema in charter §6.1; sanitize open Critical PoCs) + residual register.  
7. **Remediate** critical/high; **separate** re-verifier for `verified_fixed`; mediums per caps.  
8. **Residuals ↔ CLAIMS-CROSSCHECK** cross-check.  
9. **Closeout** evidence for T185.

### 3.4 Dependencies / tooling (review track)

| Tool | Decision |
|------|----------|
| `cargo deny` / `cargo audit` | **Required** baseline (already CI) |
| Existing nextest security suites | Elevate |
| `cargo-cyclonedx` / Syft SBOM | **Soft** — prefer T185; MIT/Apache only if used here |
| OpenSSF Scorecard CLI | **Soft optional** — Apache-2.0; read-only checks |
| ASVS 5.0.0 JSON (CC-BY-SA) | Soft optional checklist import — not a product dep |
| AGPL SAST platforms | **Forbidden** as required |
| Live vault with user secrets | **Forbidden** in reviewer packet |
| New Rust production deps | **None** for T184 DoD |

### 3.5 Non-claims (review output language)

| Forbidden after T184 | Allowed |
|----------------------|---------|
| “SOC2 / ISO / GDPR certified” | “Independent design/code review completed under charter X; residual register Y” |
| “ASVS Level 2 compliant” | “ASVS-inspired checklist used for guidance” |
| “Fully secure” / “unbreakable sandbox” | Cite ADR-0019 / CE honesty |
| “Perfect deletion” | Cite T181 / ADR-0016 residual |
| “Pentested on all platforms” | “Windows T1 focus; secondary OS per COMPATIBILITY” |

## 4. Design locks

| # | Lock |
|---|------|
| L1 | **Charter first** — no “security reviewed” claim without Accepted charter + disposition. |
| L2 | **Scope = shipping release features** — optional sync/desktop called out in/out of scope explicitly. |
| L3 | **Independence** — reviewer ≠ implementer; fresh session preferred; Critical/High `verified_fixed` only by **separate** re-reviewer. |
| L4 | **Hard deps shipped**; T183 Complete packet **required** at execute. |
| L5 | **Seed residual register** from deferred + CI Scorecard-class seeds; review may add/close. |
| L6 | **Critical/high** must be `verified_fixed` or release-blocked; qualitative rubric in charter; mediums ≤ deferral rules. |
| L7 | **No live secrets** in packet; pre-hand-off secret scan recorded; temp vaults only. |
| L8 | **No AGPL-required** scanner; deny/audit first line; Scorecard optional Apache-2.0. |
| L9 | **No formal security certification** language; does **not** block future **SLSA provenance** claims under T185. |
| L10 | Findings in track `review.md` with charter schema; open Critical PoCs sanitized until fixed. |
| L11 | **Residuals ↔ CLAIMS-CROSSCHECK** consistency before closeout. |
| L12 | **Binary** sync Y/N and desktop Y/N at charter freeze. |

## 5. Review scope matrix (normative for charter)

### 5.1 In scope (default release)

| Domain | Focus questions |
|--------|-----------------|
| **AuthN/Z (HTTP/IPC)** | Bearer storage/ACL; loopback bind; non-loopback double opt-in; SYSTEM service token residual; confused deputy principal_id |
| **Policy** | Grant bypass; scope leakage; erasure/wipe authz |
| **Path / connectors** | Reparse/junction escape; vault containment; TrustedBuiltin process model; CloudOk unused gap |
| **Crypto / CE** | AAD binding; wipe semantics; key zeroize; RecoveryKit offline residual; F8 bundled SQLite honesty |
| **Privacy / models** | Sealed vs CloudOk routing; capture no CoT; briefing overshare |
| **Sync (if shipping)** | Relay untrusted; signature fail-closed; revoke future exclusion; ACK attestation residual; metadata |
| **Supply chain (deps)** | deny/audit green; known allowlist entries; no unknown-git; ignored advisory disposition |
| **CI/CD workflow hygiene** | `permissions:` least-privilege; action SHA vs tag pinning; Dependabot/Renovate presence; branch-protection awareness; SAST honesty (clippy ≠ SAST); Scorecard-inspired — **file** findings; remediations often T186/T185 |
| **Desktop (if shipping)** | Capability mirror; opener path; CSP; Isolation residual honesty |
| **Ops residuals** | No doctor CLI; no recovery export CLI; restore-while-daemon warn |
| **Disclosure path** | SECURITY.md reporting + numeric timeline honesty |

### 5.2 Explicit out of scope (default)

| Item | Why |
|------|-----|
| Full pentest every OS / arm64 | T179 tiers; T3 residual |
| Formal ASVS certification program | Process cost; honesty |
| Physical media destruction / NIST Purge | Product non-claim |
| Plugin host / WASI | ADR-0019 forbids untrusted loaders in v1 |
| Multi-tenant IdP / OAuth | Residual #30 |
| MSI / App Store / notarization | Packaging T185 |
| Social engineering of operators | Out of product boundary |
| Full source audit of all transitive deps | deny/audit + sample; not full formal SCA program |

### 5.3 STRIDE × surface (charter summary)

| STRIDE | HTTP/IPC | Connectors | CE/crypto | Sync | Models |
|--------|----------|------------|-----------|------|--------|
| S | Stolen bearer; principal spoof | Fake connector id | Kit forgery offline | Fake device enroll | Cloud endpoint spoof |
| T | Request tamper | Path reparse race | AAD swap | Metadata swap under sig | Prompt injection into cloud |
| R | Missing audit of wipe | Silent empty | Soft forget vs CE confusion | Forged ACK claim | — |
| I | Token on multi-user host | Preview exfil | Pre-erase backup residual | Relay metadata | Sealed leakage |
| D | Flood HTTP | Unbounded list (caps) | — | Gap flood | Model hang |
| E | Policy bypass | Plugin DLL (forbidden) | Key material to connector | Revoked device future decrypt | Cloud when Sealed |

## 6. Residual risk register — seed

**Normative seed:** [`residuals.md`](./residuals.md) (expanded with T180 F34–F36 class, CI Scorecard-class R-CI-*, disclosure timeline, etc.).

Summary categories (not exhaustive — use `residuals.md`):

| Category | Example IDs |
|----------|-------------|
| Path / connectors | R-12, R-TB, R-CLOUDOK, R-EXTISM (OOS v1) |
| Crypto / CE / backup | R-34.2, R-F8, R-K06, R-CE-PRE, R-WAL-CKPT |
| Sync | R-ACK, R-META, R-PQ, R-SLSA (T185) |
| HTTP / multi-user | R-HTTP-SYS, R-MULTI |
| Protocol / ops honesty | R-API-VER, R-BRIDGE, R-DTO-GOLDEN, R-DOC-CLI |
| CI / disclosure / docs | R-CI-*, R-DISCLOSURE-TL, R-STATUS-STALE, R-CHANGELOG-PATH |

## 7. Deliverables

| Item | Path | Notes |
|------|------|-------|
| Expanded spec / plan | this track | Design freeze |
| Charter | `charter.md` | Scope, RoE, independence, time-box, in/out |
| Reviewer packet index | `evidence/PACKET.md` | Links only; no secrets |
| Finding log | `review.md` | Charter §6.1 schema; PoC hygiene §6.2 |
| Residual register | `residuals.md` or § in review | Seed + post-review |
| Closeout note | plan / conductor | Disposition for T185 |
| Optional automated baseline log | `evidence/DENY-AUDIT.md` | Commands + exit codes |

## 8. Reviewer packet contents (no secrets)

1. Charter (this track) + frozen Y/N sync/desktop.  
2. ADRs 0011–0012, 0016, 0018, 0019.  
3. T175 + T182 threat models (incl. T182 verification hooks for T184).  
4. COMPATIBILITY.md (F8), PROTOCOL-COMPAT honesty notes.  
5. OPERATIONS erasure + multi-device residual sections; RECOVERY-DRILLS.  
6. T183: `Docs/SECURITY-LIMITS.md`, root `SECURITY.md`, `Docs/INSTALL.md`, **`CHANGELOG.md` (repo root)**, `conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md`.  
7. Pointers to T178 / T181 / connector contract tests.  
8. Hotspot file list from **`ledgerful hotspots`** (daemon, HTTP, forget, sync, crypto).  
9. Residual seed (`residuals.md` incl. R-CI-*).  
10. How to run deny/audit/nextest subset — **not** a production vault path.  
11. Pre-hand-off secret scan attestation on packet files.

## 9. Finding disposition workflow

Finding **schema**, **qualitative severity rubric**, and **PoC hygiene** are normative in `charter.md` §6–§7.

| Severity | Rule |
|----------|------|
| Critical / High | Fix before “release hardened”; `fixed_pending_verification` then **`verified_fixed` only by a separate read-only re-reviewer**; or **release-block** documented |
| Medium | Fix by default; defer ≤3 with justification + deferred.md / ISSUES; **doc/claim honesty defaults Medium** |
| Low / info | Defer freely with register entry |

Statuses: `open` | `fixed_pending_verification` | `verified_fixed` | `deferred` | `out_of_scope` | `accepted_risk` (requires owner + non-claim language).

## 10. Non-goals

- Implementing missing features (doctor, recovery export, DataKey rotation, openat)  
- Full ASVS certification project  
- Replacing T178/T181 automated suites  
- SBOM release packaging (T185)  
- External paid audit firm as DoD  
- Public vulnerability disclosure program build-out (document contact path soft)  

## 11. Deferred items rolled into this track

| Deferred | Action in T184 |
|----------|----------------|
| #12 TOCTOU | Residual seed R-12 |
| #34.2 DataKey rotation | Residual seed R-34.2 |
| T161 HTTP SYSTEM / multi-user | Residual seed R-HTTP-SYS, R-MULTI |
| T180 api_version | Residual seed R-API-VER |
| T181 doctor / kit export / K-06 / pre-erase | Residuals + packet honesty |
| T182 TrustedBuiltin / CloudOk / #12 | Residuals R-TB, R-CLOUDOK, R-12 |
| F8 SQLCipher | Residual R-F8; verify T183 elevated fixes if done |
| Sync ACK / metadata / PQ | Residuals from ADR-0018 |
| T183 SECURITY-LIMITS | Soft re-sync before execute |
| T185 SBOM | Out of scope (handoff) |

## 12. Acceptance criteria

- [ ] AC1: Charter frozen (scope, out-of-scope, independence, time-box ≤4h+2h, severity rubric, **sync Y/N + desktop Y/N**)  
- [ ] AC2: Residual register seeded (incl. R-CI-*, T180 F34–F36 class) + updated post-review  
- [ ] AC3: Independent review pass recorded (fresh reviewer id/model + date)  
- [ ] AC4: Finding log complete with charter schema; no open critical/high without block decision; Critical/High `verified_fixed` by **separate** re-reviewer  
- [ ] AC5: Automated baseline deny + audit green (or findings filed); ignored advisories dispositioned  
- [ ] AC6: T183 hub + CLAIMS-CROSSCHECK in packet; **residuals ↔ CLAIMS-CROSSCHECK cross-check** complete; claim-vs-product deltas dispositioned  
- [ ] AC7: Closeout language has **no** security certification / perfect-security overclaim (SLSA axis left to T185)  
- [ ] AC8: Conductor Completed; deferred §62 promotions; T185 can cite charter + residual IDs  
- [ ] AC9: No live secrets in packet (scan attested); open Critical PoCs sanitized pre-commit; no AGPL-required tooling  
- [ ] AC10: Disclosure path reviewed (SECURITY.md timeline residual dispositioned)

## 13. Verification plan

```powershell
cargo deny check
cargo audit
# Targeted suites (examples — adjust to filter names):
cargo nextest run -p ai-brains-sync -- security
cargo nextest run -p ai-brains-store -- content_envelope
cargo nextest run -p ai-brains-sources -- connector_contract
cargo nextest run -p ai-brains-security
# Review is primarily document + code read + finding log — not a full workspace gate requirement unless code remediations land
```

If remediations change code: full workspace gate + ledgerful verify per AGENTS.md.

## 14. Definition of Done

Charter accepted; independent review executed; critical/high cleared or release-blocked; residuals owned; automated baseline clean or dispositioned; no overclaim language; T185 handoff ready; conductor updated.

## 15. T183 status (re-scan 2026-08-01)

T183 **Completed**. Packet **must** include:

- `Docs/SECURITY-LIMITS.md`
- root `SECURITY.md`
- `conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md`
- `Docs/INSTALL.md` honesty notes (doctor/kit export absence, F8, three sync names)

Execute phase should treat claim-vs-code mismatches in those docs as **findings** (doc fix or product fix), not ignore them.

**CHANGELOG path:** repository root `CHANGELOG.md` (Keep a Changelog; not under `Docs/`).

## 16. AI fold-in disposition (2026-08-01)

### AI1

| ID | Disposition | Action |
|----|-------------|--------|
| **BS1** separate re-verify Critical/High | **Accept** | Charter §2; L3; AC4 |
| **BS2** sanitize open Critical PoCs pre-commit | **Accept** | Charter §6.2; L10 |
| **BS3** residuals ↔ CLAIMS-CROSSCHECK | **Accept** | L11; AC6; Phase E |
| **Opp1** SSDF mapping table | **Accept** | Charter §6.3 |
| **Opp2** ledgerful hotspots prioritization | **Accept** | Method step 4; plan Phase C |

### AI2 findings

| ID | Sev | Disposition | Action |
|----|-----|-------------|--------|
| **F-1** | Med | **Accept** | ASVS **5.0.0** pin + chapter map; no v4 IDs |
| **F-2** | Med | **Accept** | SLSA v1.2 note; L9 disambiguate vs provenance |
| **F-3** | Med | **Accept** | CI hygiene in §5.1; R-CI-* seeds (file, prefer T186 fix) |
| **F-4** | Med | **Accept** | Finding schema charter §6.1 |
| **F-5** | Med | **Accept** | Qualitative rubric (not full CVSS) charter §7 |
| **F-6** | Med | **Accept** | Disclosure §11; R-DISCLOSURE-TL |
| **F-7** | Low | **Accept** | Docs/CHANGELOG.md path |
| **F-8** | Low | **Accept** | R-BRIDGE, R-DTO-GOLDEN, R-WAL-CKPT, R-STATUS-STALE, R-EXTISM OOS |
| **F-9** | Low | **Accept** | Full CLAIMS-CROSSCHECK path in charter |
| **F-10** | Low | **Accept** | ≤4h review + ≤2h triage |
| **F-11** | Info | **Accept** | evidence/ mkdir in plan Phase B |

### AI2 opportunities

| ID | Disposition | Action |
|----|-------------|--------|
| **O-1** ASVS JSON L1 | **Accept soft** | Optional Phase C |
| **O-2** Scorecard CLI | **Accept soft** | Optional Apache-2.0 |
| **O-3** reviewer credential / fresh session | **Accept** | Charter §2 |
| **O-4** SSDF practice IDs | **Accept** | Charter §6.3 |
| **O-5** STRIDE worksheet | **Accept soft** | Optional evidence artifact |
| **O-6** audit ignore delta | **Accept** | Phase C |
| **O-7** binary sync/desktop | **Accept** | L12; charter §10 |
| **O-8** packet secret scan | **Accept** | L7; packet §8 item 11 |

### Amendment map (A1–A13)

| A# | Applied |
|----|---------|
| A1 | ASVS 5.0.0 |
| A2 | SLSA v1.2 / L9 disambiguation |
| A3 | CI scope + R-CI-* |
| A4 | Finding schema |
| A5 | Qualitative rubric |
| A6 | Disclosure + R-DISCLOSURE-TL |
| A7 | CHANGELOG path |
| A8 | Expanded residuals |
| A9 | CLAIMS-CROSSCHECK full path |
| A10 | Numeric time-box |
| A11 | evidence/ mkdir |
| A12 | Soft ASVS JSON |
| A13 | Soft Scorecard |

### Cross-track handoff (fold-in)

| Track | Note |
|-------|------|
| **T185** | Consumes charter + residual IDs; CI R-CI-* checklist; SLSA provenance; disclosure timeline |
| **T186** | Prefer owner for CI workflow remediations T184 files |
| **T183 Complete** | Do not rewrite pack; file path/honesty deltas as findings only |
