# T195 Plan — Daemon / Multi-User Residuals

Status: **Completed** (2026-08-02; PR #78 `bd375a8`). Spec: [spec.md](./spec.md).

## Preconditions

- [x] Read T184 residuals + OPERATIONS multi-user / service HTTP notes
- [x] Re-scan live: `pipe_security.rs` SY+BA+IU; UDS `/tmp` + 0o600; service HTTP warn; hardcoded transport paths
- [x] Research: XDG_RUNTIME_DIR for UDS; keep IU default for Session 0↔1; no shared token without ADR
- [x] Expand freezes F1–F28 + AC1–AC10 + residual disposition matrix
- [x] Roll deferred R-PIPE-IU / R-UDS-TMP / R-HTTP-SYS / R-MULTI → this track
- [x] **AI fold-in** (AI1 affirm; AI2 M1–M7, L1–L7, O4; O1 deferred; O2 declined) — disposition spec §14
- [x] Pin fold-in decision (`ai-brains pin`)
- [x] `ledgerful ledger start T195-DaemonMultiuserResiduals --category SECURITY` *(TX open; commit = ship closeout)*

## Deferred rolled in

| Item | Disposition |
|------|-------------|
| R-PIPE-IU | **Absorb** — default IU + optional `service-only` |
| R-MULTI | **Absorb** — permanent product fence + ADR-0022 |
| R-UDS-TMP | **Absorb** — XDG + env + pre-bind/shutdown hygiene |
| R-HTTP-SYS | **Absorb** — refuse service HTTP unless opt-in |
| Host-header rebinding (T161) | **Soft** F13 concrete |
| Shared multi-session token | **Not absorbed** |
| T196 units | **Not absorbed** |
| Abstract UDS (O1) | **Future deferred** |
| SY-only pipe (O2) | **Declined** |

## Research pins (2026-08-02)

| Fact | Pin |
|------|-----|
| Default pipe SDDL | `D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)` unchanged |
| Optional pipe ACL | `AI_BRAINS_PIPE_ACL=interactive\|service-only` |
| Pipe name | `\\.\pipe\ledgerful-bridge` **unchanged** |
| UDS resolver home | **`ai-brains-daemon-api`** shared helper |
| UDS order | `AI_BRAINS_DAEMON_SOCKET` → XDG (F30) → `/tmp` + warn |
| Socket type check | `MetadataExt::mode()` + `S_IFSOCK` — **no libc/nix** |
| UDS mode | post-bind **0o600** |
| Service HTTP gate | **`windows_service` before `maybe_start_http`** |
| Service HTTP opt-in | `AI_BRAINS_HTTP_SERVICE` (truthy per F33) |
| ADR | **ADR-0022 required** |
| New deps | **Zero** direct |

## Phases

### Phase A — Design freeze (plan-only ✅)

- [x] **A1** Residual matrix (spec §5)
- [x] **A2** Keep IU default; no per-user SID; no pipe bearer
- [x] **A3** UDS XDG + override + fallback + F30 validation
- [x] **A4** Service HTTP refuse-by-default; gate location F10
- [x] **A5** Product fence R-MULTI; ADR-0022 required
- [x] **A6** Host rebinding soft concrete (F13)
- [x] **A7** AI fold-in M1–M7 / L1–L7 / O4

### Phase B — Shared resolver + pipe ACL (TDD)

- [x] **B0** Ledger start SECURITY
- [x] **B1 Red→Green:** `ai-brains-daemon-api::resolve_daemon_socket_path` — env absolute, XDG valid, XDG invalid→fallback, relative env fail closed
- [x] **B2 Red→Green:** wire **daemon** Unix bind + CLI `DaemonClient::new` (`#[cfg(not(windows))]`) to same helper; Windows pipe const unchanged
- [x] **B3 Red→Green:** pre-bind: socket+euid only unlink; foreign/non-socket fail closed; **shutdown** same rule
- [x] **B4 Red→Green:** `AI_BRAINS_PIPE_ACL=service-only` SDDL without IU; default with IU; **parse + IsValidSecurityDescriptor** only (not live pipe CI)
- [x] **B5** Targeted: daemon-api + ai-brainsd + ai-brains-cli nextest + clippy (248 passed Windows; Unix cfg tests compile)

### Phase C — Service HTTP gate (TDD)

- [x] **C1 Red→Green:** pure/logic test: service startup path skips `maybe_start_http` without opt-in
- [x] **C2** Opt-in path: warn retained (R-HTTP-SYS residual)
- [x] **C3** Interactive daemon HTTP path unchanged (regression)
- [x] **C4** Soft if free: Host middleware (F13) — **skipped** (soft residual; not DoD)

### Phase D — Claims + ADR + ops docs

- [x] **D1** ADR-0022 single-owner daemon IPC fence (**required**)
- [x] **D2** RELEASE-CLAIMS residual rows + principal_id honesty if free
- [x] **D3** SECURITY-LIMITS §7 matrix
- [x] **D4** OPERATIONS: env table; service HTTP; UDS path; **service-only CLI NotRunning**
- [x] **D5** CHANGELOG migration note for UDS XDG default (F18)
- [x] **D6** deferred.md strike; conductor Completed on ship

### Phase E — Gate + review

- [x] **E1** Full gate: fmt / clippy -D / nextest 1870 / deny / audit; CI Win/Linux/macOS green
- [x] **E2** SECURITY review — internal R1 + Codex R2 **PASS WITH DEFERRED P3**
- [x] **E3** Manual notes: default pipe still interactive; document service-only + XDG
- [x] **E4** Ledger commit; pin DECISION

## Verification matrix

| AC | Proof |
|----|-------|
| AC1 residual matrix | Spec §5 + claims rows |
| AC2 pipe ACL modes | Unit SDDL parse + IsValidSD (F29) |
| AC3 UDS resolver | Unit cases + CLI/daemon wire via daemon-api |
| AC4 UDS mode + pre-bind + shutdown | Unix tests |
| AC5 service HTTP refuse | Gate before maybe_start_http (not bind.rs) |
| AC6 no shared token | Code review |
| AC7 docs + ADR-0022 + CHANGELOG | Diff |
| AC8 deferred | Diff |
| AC9 gate + no new direct deps | Process |
| AC10 pipe name | Grep/assert unchanged |
| AC11 service-only CLI docs | OPERATIONS |

## Out of scope checklist

- [ ] Multi-tenant vault
- [ ] OAuth / IdP
- [ ] Per-user pipe bearer
- [ ] Shared Session0/Session1 token file
- [ ] Drop IU by default
- [ ] SY-only pipe mode
- [ ] Abstract-namespace UDS
- [ ] T196 systemd/launchd units
- [ ] “Multi-user safe” marketing
- [ ] Direct libc/nix for socket check

## Implement notes (for go-ahead)

1. **Order:** daemon-api resolver → UDS wire + hygiene → pipe ACL mode → service HTTP gate → ADR/docs.
2. **High findings if:** default IU dropped; CLI/daemon path mismatch; gate inside shared maybe_start_http; XDG without 0700/uid check; foreign unlink; silent residual delete; shared token invented; pipe name changed; new libc/nix dep.
3. **Stop-before:** multi-user product scope creep; destructive git.
4. **After ship residual order:** **T196** (ops units/docs).
