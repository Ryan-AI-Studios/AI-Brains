# Track Completion Audit — T196 (Internal R2)

## Verdict: PASS

**Reason:** Prior R1 easy P3 (non-portable `mapfile` in `scripts/check-reference-units.sh`) is **fixed**. Engineering DoD for AC1–AC7 and AC9–AC14 remains met with no new packaging/docs/SIGTERM regressions. AC8 ship-process residuals (deferred.md implement strike, conductor Completed, ledger commit) stay intentionally open and are **not** coded as P-findings.

## Scope Reviewed

| Item | Value |
|------|--------|
| Track | T196 — Service Units + Ops Docs Hygiene |
| Directory | `C:\dev\AI-Brains\conductor\tracks\trackT196-service-units-ops-docs` |
| Prior review | `review.internal-r1.md` (**FAIL**, 1× P3 mapfile portability) |
| Spec / plan | `spec.md`, `plan.md` (F1–F40, AC1–AC14) |
| Repo | `C:\dev\AI-Brains` |
| Mode | Read-only internal R2 (no code/git/deferred.md edits) |
| Fix claimed | `scripts/check-reference-units.sh`: mapfile → bash 3.2-safe while-read + heredoc; header Bash 3.2+; orchestrator reports all checks passed |
| Surfaces re-checked | check script; packaging/reference units+plist+wrapper+env+README; shutdown_signal + main wiring; CONTRIBUTING / OPERATIONS / INSTALL / COMPATIBILITY §8 #11 / RELEASE-CLAIMS / Docs+CHANGELOG; forbidden greps; deferred.md process residual |

**Intentionally open (ship process only — not code defects):**

- AC8 deferred.md strike for systemd/launchd + CONTRIBUTING implement rows
- Common Changelog “strike on ship” residual close
- Conductor Completed + ledger TX commit

---

## Prior Finding Verification

### [P3] Soft unit check script uses non-portable `mapfile` — **FIXED**

| Check | Result |
|-------|--------|
| Live `mapfile` / `readarray` invocation | **Absent** (repo-wide: only two comment mentions in the check script) |
| Process substitution `< <(...)` collect | **Removed** |
| Replacement | `FILES=()` + `while IFS= read -r _f || [ -n "${_f}" ]` + heredoc over `$(find … \| sort)` |
| Header honesty | Line 3: “Bash 3.2+ portable (macOS stock bash); no mapfile/bash-4-only features” — no longer claims POSIX while using bash-only features |
| Remaining bashisms under bash shebang | `set -o pipefail`, `local`, arrays — acceptable for `#!/usr/bin/env bash` + 3.2 target |
| Orchestrator runtime | Script re-run: all checks passed (accepted as operator evidence; this R2 is static re-verify on Windows agent) |

```48:56:C:\dev\AI-Brains\scripts\check-reference-units.sh
# Collect all text files under reference (bash 3.2-safe: no mapfile)
FILES=()
while IFS= read -r _f || [ -n "${_f}" ]; do
  [ -n "${_f}" ] && FILES+=("${_f}")
done <<EOF
$(find "${REF}" -type f \( \
  -name '*.md' -o -name '*.service' -o -name '*.plist' -o -name '*.example' -o -name '*.sh' -o -name '*.env*' \
\) | sort)
EOF
```

**No reopen** of this finding.

---

## Requirement and DoD Matrix (AC1–AC14) — R2 re-confirm

| AC | Criterion | Status | R2 note |
|----|-----------|--------|---------|
| **AC1** | systemd user: Type=simple; dual ExecStart comments; EnvironmentFile; HTTP off; light harden; no ProtectSystem/ProtectHome default-on; StartLimit* | **Met** | `ai-brainsd.user.service` unchanged and correct |
| **AC2** | LaunchAgent Label/ProgramArguments/RunAtLoad; KeepAlive dict SuccessfulExit=false; no secrets; wrapper example | **Met** | plist + wrapper present |
| **AC3** | packaging README honesty set (linger, XDG/UDS, HTTP, single-owner, cargo-bin, absolute vault, wrapper, KeepAlive/10s, no-daemonize, SIGINT/SIGTERM, not T1) | **Met** | README needles still present |
| **AC4** | Root CONTRIBUTING gate + INSTALL/AGENTS/ledgerful/conductor; changelog policy | **Met** | CONTRIBUTING present; Common Changelog declined |
| **AC5** | OPERATIONS + INSTALL + Docs README link units | **Met** | OPERATIONS “Unix service units (reference only — T196)”; INSTALL Linux #6 + macOS pointer; Docs/README rows |
| **AC6** | COMPATIBILITY §8 #11 + RELEASE-CLAIMS F2 reword | **Met** | Reference templates; not product-managed Unix install; not multi-OS T1 |
| **AC7** | No MSI/App Store/R-CI-BRANCH elevation; no platform tier elevation | **Met** | No forbidden marketing in T196 surfaces |
| **AC8** | deferred strike systemd/launchd + CONTRIBUTING; Common Changelog declined | **Partial (process)** | Decline wording landed; implement-row strike + ship closeout intentionally open |
| **AC9** | CHANGELOG Unreleased | **Met** | T196 Unreleased Added bullet present |
| **AC10** | Soft `check-reference-units.sh` | **Met** | Present; linked; **portability P3 cleared** |
| **AC11** | No secrets / real keys in templates | **Met** | KEY assignments commented with REPLACE/NEVER_COMMIT only |
| **AC12** | System unit: non-root User + ReadWritePaths | **Met** | User/Group=ai-brains; ProtectSystem=strict + ReadWritePaths |
| **AC13** | Forbid active ProtectHome; relative Documentation=file:packaging | **Met** | Comments/docs only; check script enforces |
| **AC14** | F36 signal residual documented ± soft SIGTERM | **Met** | packaging README + `shutdown_signal.rs` + three main select sites + lib export |

### Freezes / §9.3 (spot re-check)

No freeze violations observed. Forbidden table still clean: no active ProtectHome, Type=notify, relative Documentation, bare KeepAlive true in shipped plist, live HTTP non-loopback assignment, or live keys.

---

## Findings (P0–P3)

**None.**

- Prior P3: **fixed / verified**.
- No new P0–P3 validated on R2 surfaces.
- Style residual only (not a finding): empty `for needle in …; do :; done` loop in check script ~204–206 (same as R1).

---

## Completeness Sweep (high issues only)

| Sweep item | Result |
|------------|--------|
| TODO / FIXME / stub in T196 surfaces | None observed |
| Live secrets / real keys | None |
| Active ProtectHome / Type=notify / relative Documentation | None |
| Bare KeepAlive true in shipped plist | None (dict SuccessfulExit=false) |
| daemonize / double-fork / setsid in ai-brainsd | Docs/comments only |
| MSI / multi-user safe IPC / T1 Linux service claims elevation | Not elevated |
| Soft SIGTERM wiring regression | No regression — module + main sites intact; no production `unwrap`/`expect` in helper |
| Check-script bash-4-only features after fix | None (`mapfile`/`readarray`/`process-subst` collect gone) |
| High severity packaging/docs drift vs R1 | None |

---

## Wiring and Regression Review (R2)

### Soft SIGTERM (F36) — no regression

| Site | Status |
|------|--------|
| `crates/ai-brainsd/src/shutdown_signal.rs` | Unix: SIGTERM raced with Ctrl-C; fallback Ctrl-C-only; non-Unix Ctrl-C; no daemonize |
| `main.rs` Windows interactive accept | `wait_shutdown_signal()` |
| `main.rs` Unix UDS accept | `wait_shutdown_signal()` + T196 F36 comment |
| `main.rs` main select | `wait_shutdown_signal()` vs internal shutdown_rx |
| `lib.rs` | `pub mod shutdown_signal` |
| Windows SCM / Unix `--service` refuse | Unchanged (correct) |

### Claims honesty — no regression

F2 reference wording remains in packaging README, COMPATIBILITY #11, RELEASE-CLAIMS, OPERATIONS. Windows SCM remains only product-managed service path.

### Soft check script dry-reason (still sound after collect rewrite)

Forbidden/required checks unchanged after file-list portability fix: ProtectHome, ProtectSystem without ReadWritePaths, Type=notify, relative Documentation, bare KeepAlive, live KEY assignments, HTTP non-loopback assignment, required artifacts / Type=simple / StartLimit* / SuccessfulExit=false / README needles.

---

## Deferred Candidates

Reviewer proposes **no** deferred.md engineering entries from R2:

- Prior P3 is fixed (do not defer).
- AC8 deferred strike / conductor Completed / ledger commit remain **ship process**, not deferred engineering findings.

---

## Completion Decision

| Dimension | Decision |
|-----------|----------|
| **Engineering DoD (AC1–7, 9–14)** | **Complete** — prior P3 cleared; soft SIGTERM + claims + §9.3 clean |
| **AC8 process** | Partial by design until ship closeout |
| **Blocking findings** | **None** |
| **Cross-model review** | Not required (no security-sensitive default regression) |
| **Verdict** | **PASS** |
| **Ship closeout remaining (orchestrator)** | deferred.md implement strike; Common Changelog residual close if still open; conductor Completed; ledger commit; optional pin |

### Recommended orchestrator next steps (process only)

1. Ship closeout: strike T196 implement rows in deferred.md; mark conductor Completed; ledger commit.
2. Optional: soft targeted nextest for `ai-brainsd` `shutdown_signal` tests if full gate not yet run on this tree.
3. No further engineering fix required for T196 DoD.
