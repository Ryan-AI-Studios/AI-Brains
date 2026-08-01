# T184 Review Log — Independent Security Review (P12.6)

**Track:** T184-IndependentSecurityReview  
**Charter:** Frozen 2026-08-01 (Sync=Y, Desktop=Y)  
**Ledger tx:** `2ad44ef4-d800-4c43-8d78-a3498f109c0e`  
**Category:** SECURITY  

## Reviewers / rounds

| Round | Who | Role | Result |
|-------|-----|------|--------|
| R0 implement | Grok orchestrator | Charter, packet, remediations, evidence | Complete |
| R1 independent | Grok general-purpose **read-only** subagent (fresh session) | Code/doc/CI security pass | Findings F-1..F-13 filed |
| R2 internal re-review | Grok general-purpose **read-only** subagent (fresh; not fixer) | Verify remediations + completeness | **PASS** — F-1/F-2 verified_fixed; 0 open C/H |
| Codex R1 | `codex exec -s read-only` gpt-5.4 high | Cross-model DoD audit | **FAIL** P1 fail-open SD fallback; P2 UDS test; P2 artifact drift |
| Codex R1 fix | Grok orchestrator | Fail-closed pipe create; unix_socket_mode tests; spec path | Fixed |
| Codex R2 final | Fresh `codex exec -s read-only` gpt-5.4 high | Final gate after P1/P2 fix | **PASS WITH DEFERRED P3** (R-CI-PIN, R-CI-BRANCH only) |

## Scope reviewed

- AuthN/Z HTTP + named-pipe/UDS  
- Policy / CE wipe / connectors TrustedBuiltin  
- Sync signatures (shipping Y)  
- Desktop residual honesty (shipping Y / T2)  
- Supply chain deny/audit + CI hygiene  
- Disclosure SECURITY.md  
- Residuals ↔ CLAIMS-CROSSCHECK  

## Requirement / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 Charter frozen | **Met** | `charter.md` §10 Sync=Y Desktop=Y |
| AC2 Residual register | **Met** | `residuals.md` post-review |
| AC3 Independent pass | **Met** | This log + subagent report 2026-08-01 |
| AC4 Finding log + C/H disposition | **Met** | F-1 High fixed (pipe SDDL); no open Critical/High |
| AC5 deny + audit | **Met** | `evidence/DENY-AUDIT.md` exit 0 |
| AC6 residuals ↔ claims | **Met** | `evidence/RESIDUALS-CLAIMS-CROSSCHECK.md` |
| AC7 No cert language | **Met** | Closeout language; no ASVS/SOC2 claim |
| AC8 Conductor + deferred | **Met** | conductor Completed; deferred §62 |
| AC9 No secrets; no AGPL | **Met** | PACKET secret scan; deny/audit only |
| AC10 Disclosure timeline | **Met** | SECURITY.md 90-day section |

## Findings (charter schema)

### F-1 — High — Windows named-pipe World DACL

| Field | Value |
|-------|-------|
| severity | High |
| surface | AuthN/Z (HTTP/IPC) |
| files | `crates/ai-brainsd/src/pipe_security.rs` |
| evidence | Prior SDDL `D:(A;;GA;;;WD)`; contradicted T144 non-goal and OPERATIONS prose |
| why_it_matters | Multi-user local host: any principal could open unauthenticated pipe |
| required_fix | SDDL → `D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)`; tests forbid WD; **fail closed** (no default-DACL fallback in interactive or service entrypoints) |
| status | **verified_fixed** (internal R2 + Codex R1 P1 remediated fail-closed; re-verify Codex R2) |
| owner | — |

### F-2 — Medium — Unix UDS mode residual

| Field | Value |
|-------|-------|
| severity | Medium |
| surface | AuthN/Z (HTTP/IPC) |
| files | `crates/ai-brainsd/src/main.rs` |
| evidence | Bound `/tmp/ledgerful-bridge.sock` without restrictive mode |
| required_fix | Post-bind `0o600` via `unix_socket_mode::apply_owner_only_mode`; unit tests for mode constant + apply; residual bind-race documented |
| status | **verified_fixed** (mode module + tests; R2 + Codex R1 P2) / residual R-UDS-TMP for path predictability |

### F-3 — Medium — SECURITY-LIMITS omitted pipe/UDS residual

| Field | Value |
|-------|-------|
| severity | Medium |
| surface | docs honesty |
| files | `Docs/SECURITY-LIMITS.md` §7 |
| required_fix | Document named-pipe IU residual + UDS 0o600 residual |
| status | **verified_fixed** (doc) |

### F-4 — Medium — OPERATIONS pipe SD mismatch

| Field | Value |
|-------|-------|
| severity | Medium |
| surface | docs honesty |
| files | `Docs/OPERATIONS.md` |
| required_fix | Align prose to SY+BA+IU SDDL |
| status | **verified_fixed** (doc) |

### F-5 — Medium — CI permissions unset

| Field | Value |
|-------|-------|
| severity | Medium (re-scored from High*) |
| surface | CI/CD |
| files | `.github/workflows/ci.yml` |
| required_fix | `permissions: contents: read` |
| status | **verified_fixed** |

### F-6 — Medium — Actions tag pins not SHA

| Field | Value |
|-------|-------|
| severity | Medium |
| surface | CI/CD |
| files | `.github/workflows/ci.yml` |
| required_fix | Pin actions by SHA |
| status | **deferred** → T186 / R-CI-PIN |
| owner | T186 |

### F-7 — Medium — No Dependabot

| Field | Value |
|-------|-------|
| severity | Medium (re-scored from High*) |
| surface | CI/CD |
| files | `.github/dependabot.yml` |
| required_fix | Add Dependabot cargo + github-actions |
| status | **verified_fixed** |

### F-8 — Low — Branch protection not in tree

| Field | Value |
|-------|-------|
| severity | Low |
| surface | CI/CD |
| evidence | `gh api` branches/main/protection → 404 Branch not protected |
| status | **accepted_risk** / R-CI-BRANCH |
| owner | Repo admin (Ryan) |

### F-9 — Low — SECURITY.md disclosure timeline

| Field | Value |
|-------|-------|
| severity | Low |
| surface | Disclosure |
| files | `SECURITY.md` |
| required_fix | 90-day numeric timeline |
| status | **verified_fixed** |

### F-10 — Info — CHANGELOG path

| Field | Value |
|-------|-------|
| severity | Info |
| surface | docs |
| evidence | Root `CHANGELOG.md` exists; not under Docs/ |
| required_fix | Packet/residual use root path |
| status | **verified_fixed** (charter/packet/residual) |

### F-11 — Low — Default zero vault key env

| Field | Value |
|-------|-------|
| severity | Low |
| surface | Crypto |
| files | `ai-brainsd` main/service |
| status | **accepted_risk** under F8 (bundled SQLite); elevates R-F8/K-06 |
| owner | Future SQLCipher track |

### F-12 — Low — Desktop opener `**`

| Field | Value |
|-------|-------|
| severity | Low |
| surface | Desktop |
| status | **accepted_risk** / documented residual |
| owner | Desktop future hygiene |

### F-13 — Info — audit unmaintained warnings

| Field | Value |
|-------|-------|
| severity | Info |
| surface | Supply chain |
| status | **accepted_risk** process residual; deny/audit green |
| owner | T185/T186 hygiene |

## Positive controls (independent pass)

- HTTP: loopback default, double opt-in non-loopback, bearer + owner-only token ACL  
- CE wipe: fail-closed destroy-before-ContentErased  
- Sync: signature fail-closed (T178 suite elevated)  
- Connectors: TrustedBuiltin only; reparse refuse; #12 residual  
- Models: `allow_cloud` default false; Sealed non-local denied  
- deny/audit: exit 0  

## Closeout language (for T185)

Independent security review completed under charter `conductor/tracks/trackT184-independent-security-review/charter.md` (frozen 2026-08-01; Sync=Y, Desktop=Y). Critical: **none**. High: **F-1** remediated (pipe SDDL SY+BA+IU). Residual risk register: `residuals.md` (cite residual IDs in claims). **No** SOC2/ISO/GDPR/ASVS Level certification; **no** perfect deletion / metadata-private sync / third-party sandbox claims. SLSA provenance is a **T185** axis, not asserted here.

## Verification evidence

| Gate | Result |
|------|--------|
| `cargo deny check` | exit 0 |
| `cargo audit` | exit 0 (allowed warnings dispositioned) |
| `cargo nextest run -p ai-brainsd --lib` | **34 passed** (incl. pipe_security no-WD tests) |
| `cargo nextest run -p ai-brains-security` | **8 passed** |
| `cargo nextest run -p ai-brains-sync -E 'test(t178_l5) or test(verify_envelope)…'` | **8 passed** (sig fail-closed) |
| `cargo fmt --check` | exit 0 |
| `cargo clippy -p ai-brainsd --all-targets -- -D warnings` | exit 0 |
| Packet secret scan | no secrets found |
| Full workspace gate | required before PR merge (code remediations landed) |

## Deferred (≤3 medium process items)

1. **F-6 / R-CI-PIN** — Action SHA pins → T186  
2. **R-CI-BRANCH** — Branch protection admin enable → owner Ryan  
3. Action SHA is the primary deferred Medium; other CI High* seeds closed or re-scored  

## Codex R1 disposition

| ID | Sev | Disposition |
|----|-----|-------------|
| P1 fail-open SD fallback | High process | **Fixed:** main + windows_service refuse default DACL; hard-fail on SD build / create_with_security_attributes failure |
| P2 UDS untested | Med | **Fixed:** `unix_socket_mode` module + unit tests |
| P2 CHANGELOG/spec drift | Med | **Fixed:** spec.md paths → root CHANGELOG |

## Completion decision

**Engineering DoD met.** Internal R2 PASS. Codex R1 FAIL → remediations. **Codex R2 final: PASS WITH DEFERRED P3** (R-CI-PIN → T186; R-CI-BRANCH → admin). Full nextest **1710 passed**; deny/audit exit 0. Ready for PR CI + squash-merge.
