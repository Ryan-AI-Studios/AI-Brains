# T196 Plan — Service Units + Ops Docs Hygiene

Status: **Expanded + AI fold-in** (plan-only 2026-08-02). Spec: [spec.md](./spec.md).

## Preconditions

- [x] Read deferred packaging residual + T183/T185 claims non-include  
- [x] Re-scan live: Windows `daemon install` T1; **no** systemd/launchd files; **no** CONTRIBUTING.md  
- [x] Align with T195 UDS/HTTP/pipe env SOOT + ADR-0022 (**shipped** — AI2 11/11 baseline OK)  
- [x] Research: systemd user vs system; launchd Agent vs Daemon; hardening vs vault-under-home; Keep a Changelog policy  
- [x] Expand freezes F1–F32 + AC1–AC12 + residual disposition matrix  
- [x] Roll deferred: systemd/launchd + CONTRIBUTING; Common Changelog → **declined**  
- [x] **AI fold-in** (AI1 affirm; AI2 M1–M7, L1–L7, O2–O4 soft; O1 deferred; O5 no-op) — disposition spec §14; freezes F33–F40; AC13–AC14  
- [ ] Pin fold-in decision (`ai-brains pin`)  
- [ ] `ledgerful ledger start T196-ServiceUnitsOpsDocs --category DOCS` *(only on implement go-ahead)*

## Deferred rolled in

| Item | Disposition |
|------|-------------|
| systemd / launchd units | **Absorb** — reference templates under `packaging/reference/` + claims reword |
| CONTRIBUTING hygiene | **Absorb** — root `CONTRIBUTING.md` |
| Common Changelog residual | **Declined** — Keep a Changelog stays; document in CONTRIBUTING |
| MSI / notarization / App Store | **Not absorbed** |
| R-CI-BRANCH | **Not absorbed** (admin) |
| T145 cargo-bin → ProgramData | **Document only** (not installer) |
| Nightly timer/calendar units | **Not absorbed** (F31) |
| Abstract UDS (T195 O1) | **Not absorbed** |
| Product Unix `daemon install` | **Not absorbed** |
| LoadCredential daemon code | **Future** (F38) |
| graphical-session.target | **Future** (F39) |
| Unix SIGTERM graceful | **Document + soft code if free** (F36) |

## Research pins (2026-08-02 + fold-in)

| Fact | Pin |
|------|-----|
| Primary Linux | **systemd --user** → `~/.config/systemd/user/` |
| Secondary Linux | system unit + honesty |
| Primary macOS | **LaunchAgent** 0600 |
| Unit location | `packaging/reference/{systemd,launchd}/` + README |
| systemd Type | **simple** (+ StartLimitBurst=5 / Interval=60) |
| Hardening default | NoNewPrivileges + PrivateTmp only |
| ProtectHome | **Forbid yes and read-only** as defaults |
| ProtectSystem=strict | **Commented off** + ReadWritePaths placeholder |
| KeepAlive | Dict **SuccessfulExit=false** recommended |
| Vault path | **Absolute** required for headless |
| launchd secrets | Wrapper + 0600 env; not system-wide plist |
| Signals | SIGINT live; SIGTERM residual/soft |
| Validation | **`.sh` primary** |
| UDS | T195 resolver SOOT |
| HTTP in templates | **off** |
| CONTRIBUTING | Root; gate + links |
| Changelog | Keep a Changelog; Common Changelog **declined** |
| New deps | **Zero** |
| Product Unix install CLI | **No** |

## Phases

### Phase A — Design freeze (plan-only ✅)

- [x] **A1** Residual matrix (spec §5)  
- [x] **A2** User-first units; system secondary honesty  
- [x] **A3** Template paths + env conventions (F7–F12)  
- [x] **A4** Hardening freezes (F13–F15); launchd keys (F16)  
- [x] **A5** CONTRIBUTING + changelog policy (F18–F19)  
- [x] **A6** Claims rewrite language (F21); non-goals  
- [x] **A7** AI fold-in M1–M7 / L1–L7 / O* disposition §14; F33–F40; AC13–AC14  

### Phase B — Reference packaging artifacts

- [x] **B0** Ledger start DOCS (or INFRA) on go  
- [x] **B1** `packaging/reference/README.md` — honesty banner + **linger tradeoff (M1)** + absolute vault + launchd secrets/wrapper + KeepAlive/10s/suspend + no-daemonize + SIGINT/SIGTERM residual + cargo-bin residual + not T1  
- [x] **B2** `systemd/ai-brainsd.user.service` per F8/F13/F14/F33 (§9.1 sketch)  
- [x] **B3** `systemd/ai-brainsd.system.service` secondary (AC12)  
- [x] **B4** `launchd/dev.ledgerful.ai-brainsd.plist` — KeepAlive dict SuccessfulExit=false (M4)  
- [x] **B5** `daemon.env.example` (dummy absolute paths; no secrets)  
- [x] **B6** `launchd/ai-brainsd.wrapper.sh.example` (F34)  

### Phase C — CONTRIBUTING + docs graph

- [x] **C1** Root `CONTRIBUTING.md` (F18; changelog line = CHANGELOG.md:12)  
- [x] **C2** OPERATIONS: Unix service units section (link packaging; Windows still primary product install)  
- [x] **C3** INSTALL: short pointer + residual packaging honesty  
- [x] **C4** COMPATIBILITY §8 #11 reword (F2)  
- [x] **C5** RELEASE-CLAIMS “not included” row reword (F2)  
- [x] **C6** Docs/README + root README Development → CONTRIBUTING  
- [x] **C7** CHANGELOG Unreleased  

### Phase D — Soft validation + optional signal + closeout

- [x] **D1** Soft: `scripts/check-reference-units.sh` primary (F24/M7); optional `.ps1` mirror  
- [x] **D2** Soft if free: Unix SIGTERM alongside `ctrl_c` in `ai-brainsd` (F36) — else residual only  
- [ ] **D3** Soft claims re-grep if elevated set touched  
- [ ] **D4** deferred.md strike; conductor **Completed** on ship  
- [ ] **D5** Manual optional smoke notes in `evidence/` if free — not CI DoD  
- [ ] **D6** Pin DECISION; ledger commit  

### Phase E — Gate + review

- [ ] **E1** Docs-primary gate; if scripts/soft SIGTERM code: targeted nextest + clippy on `ai-brainsd`  
- [ ] **E2** Internal review vs AC1–AC14  
- [ ] **E3** Cross-model only if security-sensitive default regressions (F25)  

## Verification matrix

| AC | Proof |
|----|-------|
| AC1 user systemd | File (simple, dual ExecStart comments, light harden, StartLimit*) |
| AC2 LaunchAgent | File (KeepAlive dict, no secrets, wrapper) |
| AC3 packaging README | Review (M1–M6 honesty) |
| AC4 CONTRIBUTING | Review |
| AC5 OPERATIONS/INSTALL links | Diff |
| AC6 COMPATIBILITY + RELEASE-CLAIMS | Diff |
| AC7 no tier/MSI/branch claim | Grep |
| AC8 deferred strike | Diff |
| AC9 CHANGELOG | Diff |
| AC10 process + soft .sh | Process |
| AC11 no secrets | Grep |
| AC12 system unit honesty | File if present |
| AC13 ProtectHome/Documentation bans | Grep |
| AC14 F36 signal residual (± soft code) | Docs ± code |

## Out of scope checklist

- [ ] MSI / WiX / MSIX / App Store / notarization  
- [ ] R-CI-BRANCH admin  
- [ ] Elevating Linux/macOS daemon T1  
- [ ] `ai-brains daemon install` Unix product CLI  
- [ ] systemd socket activation / Type=notify  
- [ ] Nightly timer units  
- [ ] Abstract UDS  
- [ ] Multi-user / shared token  
- [ ] Common Changelog conversion  
- [ ] Closing cargo-bin residual via installer  
- [ ] LoadCredential daemon code  
- [ ] graphical-session.target as default  

## Implement notes (for go-ahead)

1. **Order:** packaging README (honesty first) → units + wrapper + env → CONTRIBUTING → doc graph → CHANGELOG → soft `.sh` → soft SIGTERM if free → deferred/conductor.  
2. **High findings if:** T1 overclaim; secrets in plist/env; ProtectHome or ProtectSystem footgun defaults; bare KeepAlive without suspend docs; missing linger/secrets/SIGTERM honesty; non-loopback HTTP; Unix product install CLI; MSI claim.  
3. **Stop-before:** packaging installer scope; multi-user product; LoadCredential code.  
4. **After ship residual order:** MSI/notarization/App Store + R-CI-BRANCH admin (+ SIGTERM residual if not soft-fixed).  
)
