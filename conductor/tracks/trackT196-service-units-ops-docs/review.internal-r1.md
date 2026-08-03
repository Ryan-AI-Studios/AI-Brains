# Track Completion Audit — T196

## Verdict: FAIL

**Reason:** Engineering DoD for AC1–AC7, AC9–AC14 is largely met with solid claims honesty and complete SIGTERM wiring, but one easy **P3** remains (soft validation script non-portable bashism). Ship-process residuals (AC8 deferred strike, conductor Completed, ledger commit) are intentionally open and are not coded as P-findings.

## Scope Reviewed

| Item | Value |
|------|--------|
| Track | T196 — Service Units + Ops Docs Hygiene |
| Directory | `C:\dev\AI-Brains\conductor\tracks\trackT196-service-units-ops-docs` |
| Spec / plan | `spec.md`, `plan.md` (F1–F40, AC1–AC14) |
| Repo | `C:\dev\AI-Brains` |
| Scope | Working tree (uncommitted implementation per orchestrator) |
| Mode | Read-only internal R1 (no code/git/deferred.md edits) |
| Reviewed surfaces | `packaging/reference/**`, `CONTRIBUTING.md`, `scripts/check-reference-units.sh`, `crates/ai-brainsd` shutdown signal + main wiring, OPERATIONS / INSTALL / COMPATIBILITY §8 #11 / RELEASE-CLAIMS / Docs+root README / CHANGELOG Unreleased, claims greps, §9.3 forbidden table |

**Orchestrator-stated intentionally open (not treated as code defects):**

- AC8 deferred.md strike + conductor Completed
- Ledger TX open (`19c21fe3`, uncommitted)
- Plan D3–D6 / E* process closeout

## Requirement and DoD Matrix (AC1–AC14)

| AC | Criterion | Status | Evidence | Gap |
|----|-----------|--------|----------|-----|
| **AC1** | systemd user: `Type=simple`; dual ExecStart comments; optional EnvironmentFile; HTTP off; light hardening; ProtectSystem/ProtectHome not default-on; StartLimit* | **Met** | `packaging/reference/systemd/ai-brainsd.user.service`: `Type=simple`, cargo+system ExecStart comments, live `ExecStart=%h/.cargo/bin/ai-brainsd`, `EnvironmentFile=-%h/.config/ai-brains/daemon.env`, `StartLimitIntervalSec=60` / `StartLimitBurst=5`, `NoNewPrivileges`+`PrivateTmp` only, ProtectSystem commented, ProtectHome only in ban comments, HTTP not enabled | — |
| **AC2** | LaunchAgent: Label, ProgramArguments, RunAtLoad; KeepAlive dict `SuccessfulExit=false`; no secrets; wrapper example | **Met** | `launchd/dev.ledgerful.ai-brainsd.plist` Label `dev.ledgerful.ai-brainsd`, ProgramArguments → wrapper path, RunAtLoad true, KeepAlive dict SuccessfulExit=false, ProcessType=Background (soft O3), no KEY env; `launchd/ai-brainsd.wrapper.sh.example` present | — |
| **AC3** | packaging README honesty (linger, XDG/UDS, HTTP off, single-owner, cargo-bin, absolute vault, secrets wrapper, KeepAlive/10s/suspend, no-daemonize, SIGINT/SIGTERM, not T1) | **Met** | `packaging/reference/README.md` sections cover all AC3 topics + F2 banner | — |
| **AC4** | Root CONTRIBUTING: gate + INSTALL + AGENTS + ledgerful/conductor; changelog policy line | **Met** | `CONTRIBUTING.md`: license, Rust 1.95.0/Perl/nextest/deny/audit, `dev-check.ps1`/`.sh`, conductor+ledgerful, no push main, doc map, soft onboarding skill; Common Changelog note matches CHANGELOG.md:12 | — |
| **AC5** | OPERATIONS + INSTALL + Docs README link reference units | **Met** | OPERATIONS “Unix service units (reference only — T196)”; INSTALL Linux #6 + macOS LaunchAgent pointer; Docs/README packaging + CONTRIBUTING rows; root README → CONTRIBUTING | — |
| **AC6** | COMPATIBILITY §8 #11 + RELEASE-CLAIMS reword (F2) | **Met** | COMPATIBILITY #11 reference templates + no automated Unix install / not T1 multi-OS; RELEASE-CLAIMS systemd/launchd row: reference templates, not product-managed Unix install, not multi-OS T1 | — |
| **AC7** | No MSI/App Store/R-CI-BRANCH elevation; no platform tier elevation | **Met** | Grep: no T1 Linux service / production systemd support / launchd installer claims; INSTALL residual MSI/App Store/notarization still out; packaging README omits table | — |
| **AC8** | deferred strike systemd/launchd + CONTRIBUTING; Common Changelog declined | **Partial** | Common Changelog declined in CONTRIBUTING + CHANGELOG note + deferred.md “Declined by T196 freezes — strike on ship”. systemd/launchd + CONTRIBUTING row still “T196 Expanded… implement on go-ahead” (not struck). Orchestrator: intentionally not done yet | Ship strike + conductor Completed remaining |
| **AC9** | CHANGELOG Unreleased | **Met** | CHANGELOG Unreleased Added: T196 bullet (units, CONTRIBUTING, docs, soft check script, soft SIGTERM) | — |
| **AC10** | Soft `check-reference-units.sh` if free; process gate | **Met (with P3)** | `scripts/check-reference-units.sh` present; linked from packaging README, CONTRIBUTING, OPERATIONS. Dry-reason: fails active ProtectHome, Type=notify, relative Documentation=, bare KeepAlive true, live KEY assignments; asserts user unit + plist positives | Soft script portability P3 (below) |
| **AC11** | No secrets / real keys in templates | **Met** | KEY/VAULT_KEY only commented with REPLACE/NEVER_COMMIT placeholders; REPLACE_ME paths only | — |
| **AC12** | Secondary system unit: non-root User honesty + ReadWritePaths | **Met** | `ai-brainsd.system.service`: bold not-recommended banner, `User=`/`Group=ai-brains`, `ProtectSystem=strict` + active `ReadWritePaths=/var/lib/ai-brains /run/ai-brains` | — |
| **AC13** | Forbid active ProtectHome yes/read-only; relative Documentation=file:packaging | **Met** | No active ProtectHome; Documentation only as ban comments; check script enforces | — |
| **AC14** | F36 signal residual documented (± soft SIGTERM code) | **Met** | packaging README SIGINT/SIGTERM section; soft code `shutdown_signal.rs` + three `main.rs` select sites + `lib.rs` export | Soft tests do not fire real SIGTERM (not blocking; soft path) |

### Freezes F1–F40 (violations?)

| Freeze cluster | Result |
|----------------|--------|
| F1–F7 scope/location/names | No violation — reference tree only; no Unix product install CLI; zero new crates |
| F8–F15 units/env/hardening | No violation — dual ExecStart comments, absolute vault docs, Type=simple, StartLimit*, light user harden, system secondary honesty |
| F16–F17 launchd + linger | No violation — KeepAlive dict, wrapper, 0600 docs, linger table, no bare KeepAlive in shipped plist |
| F18–F21 CONTRIBUTING/changelog/claims | No violation — wording honest; Common Changelog declined |
| F22–F23 capture independence / no new deps | No evidence of new prod deps or model coupling in units |
| F24 soft validation | Present; portability P3 vs “POSIX” claim |
| F25–F32 out-of-scope fences | MSI / R-CI-BRANCH / abstract UDS / Unix install CLI / timers not absorbed |
| F33–F40 vault absolute, secrets wrapper, install honesty, signals, env inheritance, LoadCredential/graphical-session future, §9.3 forbids | No freeze violations in shipped templates |

### Spec §9.3 forbidden table

| Forbidden | Status in templates |
|-----------|---------------------|
| `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK=1` assignment | Absent (docs ban only) |
| Committed real `AI_BRAINS_KEY` | Absent |
| Secrets in system-wide LaunchAgent/Daemon | No system LaunchDaemon template; agent plist has no secrets |
| Active ProtectHome=yes / read-only (user default) | Absent |
| ProtectSystem=strict without ReadWritePaths | User: commented; system: paired with ReadWritePaths |
| Type=notify | Absent (`simple` only) |
| Relative `Documentation=file:packaging/...` | Absent (ban comments only) |
| Bare KeepAlive=true without README suspend warning | Plist uses dict; README documents bare-true suspend risk |
| World/group-writable sample secrets | Wrapper checks mode; examples have no live secrets |
| Multi-user safe IPC claims | Explicit single-owner / ADR-0022 fence |

## Findings (P0–P3 format)

### [P3] Soft unit check script uses non-portable `mapfile` (bash 4+) while claiming POSIX

**Confidence:** High  
**Requirement:** F24 / AC10 soft validation; script header claims “POSIX shell”  
**Location:** `C:\dev\AI-Brains\scripts\check-reference-units.sh:1–3`, `:48–50`  
**Problem:** Script is `#!/usr/bin/env bash` and uses `mapfile` (Bash ≥4). Header/comments advertise POSIX. macOS stock `/bin/bash` is 3.2 and has no `mapfile`, so a macOS operator validating launchd artifacts with default bash gets an immediate runtime failure before any content checks run.  
**Evidence:**

```48:50:C:\dev\AI-Brains\scripts\check-reference-units.sh
mapfile -t FILES < <(find "${REF}" -type f \( \
  -name '*.md' -o -name '*.service' -o -name '*.plist' -o -name '*.example' -o -name '*.sh' -o -name '*.env*' \
\) | sort)
```

Also `set -o pipefail` / `local` are bash-oriented (acceptable under bash shebang) but conflict with the “POSIX” claim.  
**Failure scenario:** On macOS without Homebrew bash 4+, `./scripts/check-reference-units.sh` exits with `mapfile: command not found` instead of validating plists/wrapper.  
**Correction:** Replace `mapfile` with a portable `while IFS= read -r` loop (or document Bash ≥4 requirement and drop “POSIX” wording). Prefer portable collect so Linux+macOS operators both succeed.  
**Verification:** Run script under bash 3.2 or `sh` path constraints; confirm it still fails on forbidden patterns (ProtectHome, Type=notify, bare KeepAlive, live keys).  
**Deferrable:** No (easy fix; soft tooling but real, limited defect)

---

No P0/P1/P2 findings validated.

## Completeness Sweep

| Sweep item | Result |
|------------|--------|
| TODO / FIXME / stub / not-implemented in T196 surfaces | None in packaging units, wrapper, shutdown_signal, check script |
| `REPLACE_ME` | Intentional operator placeholders in plist / env / system unit comments — correct for reference templates |
| Secrets / real keys | None live |
| ProtectHome active | None |
| Type=notify | None |
| Bare KeepAlive true in shipped plist | None (dict SuccessfulExit=false) |
| Relative Documentation=file:packaging | None active |
| daemonize / double-fork / setsid in ai-brainsd | None (docs + comments only) |
| MSI / multi-user safe IPC / T1 Linux service claims | Not elevated |
| Dead code in check script | Harmless empty `for needle in …; do :; done` loop at ~198–200 (style only; not a finding) |
| F6 optional LaunchDaemon | Soft-if-free; not shipped; README honesty sufficient |

### Key env naming honesty (`AI_BRAINS_KEY` vs `AI_BRAINS_VAULT_KEY`)

**Met for T196 honesty.** Live daemon reads `AI_BRAINS_VAULT_KEY` (`main.rs`, `windows_service.rs`). CLI interactive/common path uses `AI_BRAINS_KEY`. `daemon.env.example` documents both and tells operators to set what the binary expects; plist/wrapper forbid secrets in system-wide plists and name both vars. Pre-existing KEY/VAULT_KEY split is outside T196 scope; templates do not hide it.

## Wiring and Regression Review

### Soft SIGTERM (F36)

| Site | Wiring |
|------|--------|
| `crates/ai-brainsd/src/shutdown_signal.rs` | Unix: `SignalKind::terminate` raced with `ctrl_c`; fallback Ctrl-C-only if SIGTERM install fails; non-Unix: Ctrl-C only; no daemonize |
| `main.rs` Windows interactive accept loop | `wait_shutdown_signal()` |
| `main.rs` Unix UDS accept loop | `wait_shutdown_signal()` + comment T196 F36 |
| `main.rs` main select | `wait_shutdown_signal()` vs internal `shutdown_rx` |
| `lib.rs` | `pub mod shutdown_signal` |
| Windows SCM | Still `--service` → `windows_service::run_service()`; does not use this helper (correct) |
| Unix `--service` | Still refused (correct) |

**Regression risk:** Dual concurrent `wait_shutdown_signal()` (accept loop + main select) matches prior dual-`ctrl_c` pattern; not a new structural hazard. Unit tests only spawn/abort and link-smoke — do not prove SIGTERM delivery (acceptable for soft micro-fix; not raised as DoD gap).

### Claims honesty

| Claim class | Status |
|-------------|--------|
| F2 reference wording | Present in packaging README, COMPATIBILITY #11, RELEASE-CLAIMS, OPERATIONS |
| Windows SCM only product-managed service | Consistent |
| No multi-user safe IPC | ADR-0022 / packaging single-owner fence |
| No production systemd / launchd installer marketing | Absent from T196 surfaces |

### CONTRIBUTING ↔ F18 + CHANGELOG Common decline

CONTRIBUTING changelog note matches CHANGELOG.md:12 wording (Keep a Changelog retained; Common Changelog declined for Security/Deprecated/Unreleased incompatibility). Gate, links, and soft packaging check documented.

### Soft check script dry-reason (forbidden patterns)

| Pattern | Would fail? |
|---------|-------------|
| Active `ProtectHome=yes` / `read-only` in `.service` | Yes |
| `ProtectSystem=strict` without active `ReadWritePaths` | Yes |
| Active `Type=notify` | Yes |
| Active `Documentation=file:packaging/` | Yes |
| Bare `<key>KeepAlive</key><true/>` in plist | Yes |
| Live uncommented KEY/VAULT_KEY with long hex | Yes |
| Assignment of `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK` | Yes |
| Missing required artifacts / Type=simple / StartLimit* / SuccessfulExit=false / README needles | Yes |

## Verification Evidence

| Class | Observed now | Reported by orchestrator | Not verifiable here |
|-------|--------------|--------------------------|---------------------|
| Static file review | Full AC/freeze/forbidden/wiring review above | Implementation summary matches tree | — |
| `check-reference-units.sh` execution | Not executed in this read-only review (Windows agent; dry-reason only) | Soft script shipped | Runtime pass on Linux/macOS bash |
| nextest / clippy on `ai-brainsd` | Not observed | Soft SIGTERM + unit tests present | Gate results |
| Manual systemctl/launchctl smoke | Not required (spec: not CI DoD) | Optional | — |
| Ledger TX / deferred strike / conductor Completed | deferred.md still open for T196 implement strike; Common Changelog “strike on ship” | Intentionally open | TX id content beyond summary |

## Deferred Candidates

Reviewer proposes **no** deferred.md entries:

- The sole finding (P3 mapfile portability) is **easy** → fix, do not defer.
- AC8 deferred strike / conductor Completed / ledger commit are **ship process**, not deferred engineering findings.

## Completion Decision

| Dimension | Decision |
|-----------|----------|
| **Engineering DoD (AC1–7, 9–14)** | Substantively complete; soft SIGTERM shipped and documented; claims honest; §9.3 clean |
| **AC8 process** | Partial — Common Changelog decline landed; deferred.md strike + conductor Completed intentionally pending ship |
| **Blocking findings** | **1× easy P3** — fix `check-reference-units.sh` portability (or honest Bash≥4 wording + portable file collect) |
| **Cross-model review** | Not required under F25 unless security-sensitive defaults regress (they did not) |
| **Verdict** | **FAIL** until easy P3 fixed and re-reviewed (or reclassified with evidence it is non-actionable) |
| **After P3 fix + ship process** | Expected path: re-review → PASS (or PASS with only intentional process residuals closed at ship) |

### Recommended orchestrator next steps

1. Fix P3 in `scripts/check-reference-units.sh` (portable file list; align header with reality).  
2. Optionally smoke: `bash scripts/check-reference-units.sh` on Linux; assert intentional forbidden-pattern dry-runs fail.  
3. Soft: targeted nextest for `ai-brainsd` shutdown_signal tests if code gate not yet run.  
4. Ship closeout (not this review’s edits): deferred.md strike, Common Changelog residual close, conductor Completed, ledger commit, pin if required.  
5. Reinvoke internal review with fix summary for prior-finding verification.
