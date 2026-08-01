# T184 Security Review Charter (Frozen)

**Status:** **Accepted / Frozen** (execute 2026-08-01).  
**Track:** T184-IndependentSecurityReview (P12.6)  
**Date opened:** 2026-08-01  
**Date frozen:** 2026-08-01  

## 1. Purpose

Independent, time-boxed, evidence-based security review of AI-Brains **shipping** surfaces so critical/high issues are fixed or release-blocked and residual risks are explicit for T185 claims.

## 2. Independence

| Role | Who |
|------|-----|
| Implementer of surfaces under review | Track owners of P6–P11 / P12.1–4 code (prior sessions) |
| **Reviewer (must differ)** | Fresh read-only subagent session (Grok general-purpose, capability read-only) — no implementer carryover for surface code under review; orchestrator synthesizes |
| Re-verifier (Critical/High `verified_fixed`) | Separate read-only Codex (`codex-review` skill) and/or distinct internal re-review session — **not** the fixer |
| Risk acceptance (`accepted_risk`) | Product owner (Ryan) + recorded in `residuals.md` |

**Rules:**

1. Reviewer must not mark their own implementation `verified_fixed`.  
2. Critical/High transitions `fixed_pending_verification` → `verified_fixed` **require** a separate read-only re-audit with evidence recorded in `review.md`.  
3. Reviewer session: prefer freshly initialized context; findings are delivered into the track; do not retain vault paths or secret material in long-lived reviewer storage.  
4. Fix-forward may be the implementer; verification must not be.

## 3. Time-box

| Phase | Target |
|-------|--------|
| Charter freeze | ≤ 1 session |
| Automated baseline | ≤ 1 hour wall |
| Design + hotspot code review | **≤ 4 hours wall** (reviewer) + **≤ 2 hours** triage |
| Remediation of critical/high | Until clear or release-block (code work unbounded by review time-box) |

## 4. In scope (default)

See `spec.md` §5.1. Summary:

- Loopback HTTP bearer + bind policy; IPC pipe/UDS  
- Policy/grants/erasure authz  
- Connectors path safety + TrustedBuiltin model  
- Content envelopes / keys / RecoveryKit honesty  
- Model/cloud local-first gates  
- Sync/replicate **Y** (shipping optional multi-device replication)  
- Desktop **Y** (shipping shell; T2 matrix honesty on non-Windows / Linux exclude)  
- Dependency supply chain: deny/audit  
- **CI/CD workflow hygiene:** `permissions:` least-privilege, action pin posture, Dependabot/Renovate presence, branch-protection awareness, SAST honesty (Scorecard-inspired; disposition not necessarily full fix in T184)  
- Documented ops residuals (no doctor CLI, F8 encryption honesty)  
- Disclosure path: `SECURITY.md` reporting + timeline honesty  

## 5. Out of scope (default)

See `spec.md` §5.2. Explicitly **not** claiming: full multi-OS pentest, ASVS Level certification, NIST Purge, plugin marketplace, OAuth multi-tenant, MSI/App Store.  
**SLSA provenance levels** are **T185** scope (not blocked by T184’s no-cert security lock).

## 6. Method

1. Automated: `cargo deny check`, `cargo audit` (+ advisory/ignore delta review), selected nextest security suites.  
2. Soft optional: OpenSSF Scorecard **read-only** checks (Apache-2.0) for Token-Permissions, Pinned-Dependencies, Security-Policy.  
3. Document review: ADR-0016/0018/0019, T175 + T182 threat models (incl. T182 verification hooks for T184), COMPATIBILITY F8, T183 SECURITY-LIMITS + CLAIMS-CROSSCHECK.  
4. Hotspot code review prioritized by **`ledgerful hotspots`** (e.g. daemon, forget, sync CLI) + charter list.  
5. STRIDE / **ASVS 5.0.0**-inspired checklist (guidance only — not Level certification). Soft optional: filter ASVS 5.0 L1 JSON.  
6. Finding log + residual register + **residuals ↔ CLAIMS-CROSSCHECK** cross-check.

### 6.1 Finding schema (`review.md`)

| Field | Required |
|-------|----------|
| `id` | Yes — `F-N` |
| `severity` | Yes — Critical / High / Medium / Low / Info (rubric §7) |
| `title` | Yes |
| `surface` | Yes — domain from `spec.md` §5.1 |
| `files` | Yes — paths + line refs when known |
| `evidence` | Yes — commands/output or doc quotes |
| `why_it_matters` | Yes |
| `required_fix` | Yes |
| `status` | Yes — open \| fixed_pending_verification \| verified_fixed \| deferred \| out_of_scope \| accepted_risk |
| `owner` | Yes when deferred / accepted_risk |
| `cvss_or_qualitative` | Optional note; qualitative primary |

### 6.2 PoC / exploit detail hygiene

During the active review time-box for **open** Critical/High findings:

1. **Do not** commit full weaponized PoC scripts or step-by-step exploit recipes to shared git until the fix is merged and `verified_fixed`.  
2. Commit **sanitized** descriptions (impact, affected surface, root cause class, remediation direction).  
3. Full PoC detail may live in local/transient notes until verification; then attach minimal re-test evidence only.

### 6.3 NIST SSDF (SP 800-218) mapping — guidance only

| SSDF practice (examples) | T184 activity |
|--------------------------|---------------|
| **PO.2** Define security requirements | Charter freeze (§10) |
| **PW.4** Reuse well-secured components | deny.toml allowlist review |
| **PW.7** Review design / implementation | Design + hotspot code review |
| **PW.8** Configure secure compile/build options | clippy `-D warnings`; edition 2024 safety rules |
| **RV.1** Identify and confirm vulnerabilities | Automated suites + independent pass |
| **RV.1 / residual risk** | `residuals.md` + accepted_risk decisions |

**Do not** claim “SSDF conformant.”

### 6.4 ASVS 5.0.0 surface map — guidance only

Pin: **OWASP ASVS 5.0.0** (2025; major renumbering vs 4.0.3). Do **not** use v4 IDs.

| AI-Brains surface | ASVS 5.0 chapters (inspiration) |
|-------------------|----------------------------------|
| HTTP/IPC | V4 API, V6 Authentication, V7 Session, V8 Authorization, V9 Tokens |
| Path / connectors | V5 File Handling, V8 Authorization |
| CE / crypto / RecoveryKit | V11 Cryptography, V14 Data Protection |
| Sync / replicate | V12 Secure Communication, V11 Cryptography |
| Desktop (if in scope) | V3 Web Frontend (as applicable to Tauri/WebView) |
| Supply chain / CI | V13 Configuration, V15 Secure Development Lifecycle |
| Logging / errors | V16 Security Logging & Error Handling |

## 7. Severity & disposition

Per `AGENTS.md` Review & Severity + qualitative rubric:

| Severity | Qualitative rubric (local-first single-owner product) |
|----------|--------------------------------------------------------|
| **Critical** | Exploitable on a **shipping** surface with **no** meaningful user interaction by an unauthenticated or remote-equivalent attacker (or total vault key compromise via product bug) |
| **High** | Exploitable with limited conditions on a shipping surface (e.g. local multi-user token theft, policy bypass, crypto fail-open) **or** enables clear vault/data compromise |
| **Medium** | Requires local access, misconfiguration, or user action; **or** **doc/claim honesty gap** that would mislead operators (default for honesty findings) |
| **Low** | Defense-in-depth gap, incomplete evidence, or minor process issue |
| **Info** | Non-claim reminder, process note, out-of-scope pointer |

**Critical/High:** fix → `fixed_pending_verification` → **separate** re-reviewer → `verified_fixed`, **or** release-block with product-owner sign-off.  
**Accepted residual** requires: owner, non-claim language for public docs, entry in `residuals.md`.

## 8. Reviewer packet (Completed T183 artifacts)

| Artifact | Path / action |
|----------|----------------|
| `Docs/SECURITY-LIMITS.md` | **Required** — verify vs product |
| root `SECURITY.md` | **Required** — reporting path + timeline |
| CLAIMS-CROSSCHECK | **Required** — `conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md` |
| `Docs/INSTALL.md` honesty | Doctor/kit/F8/sync-name notes |
| **`CHANGELOG.md` (repo root)** | Actual path (Keep a Changelog). Packet uses **root**, not `Docs/CHANGELOG.md` |
| ADRs 0016/0018/0019 | **Required** |
| T175 + T182 threat models | **Required** (incl. T182 § verification hooks for T184) |
| COMPATIBILITY F8 | **Required** |
| Residual seed | `residuals.md` |
| Packet index | `evidence/PACKET.md` |

**Pre-hand-off check (L7 auditable):** track owner greps packet files (or runs `ai-brains-security` patterns) for high-entropy secrets; record “no secrets found” in `evidence/PACKET.md`.

## 9. Non-claims from this review

This charter does **not** authorize marketing language that claims certification, perfect security, perfect deletion, metadata-private sync, or ASVS compliance.  
It does **not** forbid future **SLSA Build L1-style provenance** claims under T185 (separate axis from security certification).

## 10. Sign-off (frozen)

| Field | Value |
|-------|-------|
| Charter frozen by | Grok orchestrator (T184 execute) |
| Date | 2026-08-01 |
| Reviewer model/session (fresh) | Grok general-purpose read-only subagent (session fresh; no surface implementer memory) |
| Re-verifier model/session (Critical/High) | Codex `codex exec -s read-only` (separate from fix-forward) |
| **Sync in release candidate?** | **Y** — optional multi-device replication (ADR-0018 / `device` + `replicate`) ships as optional feature |
| **Desktop in release candidate?** | **Y** — Tauri shell ships; Linux/macOS matrix T2 honesty (CI excludes desktop on Linux); Isolation residual remains |
| If N: residual note | n/a |

## 11. Disclosure

| Rule | Detail |
|------|--------|
| Internal by default | T184 findings stay internal until release disposition |
| Critical/High exploitable | Prefer private **GitHub Security Advisory** before public release when product ships with the issue |
| Timeline target | **90 days** from fix to public disclosure, or next release, whichever is later — align `SECURITY.md` numeric timeline |
| Residual | R-DISCLOSURE-TL if SECURITY.md lacks numeric timeline |

## 12. SSDF / Scorecard soft tools

Optional at execute (not DoD): ASVS 5.0 JSON L1 filter; Scorecard Apache-2.0 read-only checks. Remediation of CI workflow may hand to **T186** / packaging tracks — T184 **files** findings; easy High workflow hygiene may be fixed in-track when low risk.
