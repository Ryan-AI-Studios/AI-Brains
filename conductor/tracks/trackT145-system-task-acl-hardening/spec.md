# T145 — Nightly/Daemon SYSTEM Task Privilege-Escalation Hardening

> Adapted from the coordinated conductor template (`C:\dev\coordinated\conductor\templates\0000-Description\`)
> at the user's request, applied to AI-Brains' own track numbering — this track stays governed inside
> `C:\dev\AI-Brains\conductor\`, not the Ledgerful coordinated tree, per the 2026-07-21 decision to keep
> AI-Brains a separate, license-matched companion project rather than fold it into Ledgerful's governance.

- **Track ID:** T145-SystemTaskAclHardening
- **Execution repo:** `C:\dev\AI-Brains`
- **Governance:** this directory in `C:\dev\AI-Brains\conductor\tracks\`
- **Plan-of-record reference:** `conductor/deferred.md` item #8 (2026-07-01 nightly investigation); triggered by codex flagging this CRITICAL on two consecutive T143 reviews
- **Cross-repo contract:** N/A — pure AI-Brains-internal, no Ledgerful contract touched
- **Status:** Ready — not started

---

## 1. Objective
Close the local privilege-escalation path in the SYSTEM-scheduled nightly (and, if applicable, daemon) task: today a SYSTEM-run scheduled task executes a wrapper script and binary that both live in ordinary user-writable locations, so any user-level process on the machine can replace either file and get SYSTEM code execution on its next scheduled run. Codex refused to clear T143 without ACL hardening on this exact gap — this track does that hardening.

## 2. Context (read before starting)
- **Verified 2026-07-21, current state:** `ai-brains nightly --schedule --run-as-system` writes a wrapper batch file via `write_wrapper_script` (`crates/ai-brains-cli/src/commands/nightly.rs:384-395`) to `<vault-parent-dir>\.ai-brains-nightly-task.bat` — on this machine, `C:\dev\ai-brains\.ai-brains-nightly-task.bat`, an ordinary user-writable project directory. Confirmed live content bakes vault path + model URLs into plaintext `set` lines, then invokes `"C:\Users\RyanB\.cargo\bin\ai-brains.exe" --no-project-context nightly --skip-import --log-format json` — also a user-writable path.
- The scheduled task registers this wrapper to run `/RU SYSTEM` (T132, refined by T143). **Any process running as the logged-in user can overwrite either the wrapper or the binary; the next SYSTEM-scheduled run then executes attacker-controlled code as SYSTEM.** That's a full local privesc, not a theoretical one — both paths are ordinary, unprotected user directories today.
- **This is distinct from T143's scope, not a duplicate of it.** T143 (status: In Progress) is about making the nightly *functionally succeed* as SYSTEM — env vars, working directory, `--no-project-context`/`--skip-import`. It already relocated the wrapper away from `%TEMP%` and added `cd /d`, but explicitly did **not** add ACL hardening. `deferred.md` item #8 records codex refusing to clear T143 on exactly this gap and recommending "a dedicated SECURITY track" — this is that track.
- **Why now, not "later":** the standing mitigation has been "single-user dev machine, risk is theoretical" — still technically true, but this repo is public (`Ryan-AI-Studios/AI-Brains`) and is about to be positioned publicly as a license-matched companion tool to Ledgerful. Shipping a known, codex-confirmed CRITICAL local-privesc path under a product name other people will actually install is a materially different risk than "my own machine, nobody else runs this."

## 3. In scope
1. Relocate the wrapper script off any user-writable path to a SYSTEM-controlled location (e.g. `C:\ProgramData\AI-Brains\nightly-task.bat`), created with an explicit ACL granting only `SYSTEM:F` + `Administrators:F` — no access for the interactive user, `Everyone`, or `Authenticated Users`. Mirrors the fix `deferred.md` #8 already sketched.
2. Cross-check T144's Windows-service daemon path (`ai-brainsd --service`, `daemon install`) for the same class of user-writable-artifact problem — T144 solved the pipe security descriptor, but verify (don't assume) there's no equivalent unprotected script/binary dependency in the service-install path. Harden if found; otherwise record the clean bill of health.
3. Decide and implement how the **invoked binary path** (`ai-brains.exe` in `~\.cargo\bin\`) is handled: either (a) verify it's ACL-protected before the SYSTEM task runs, or (b) explicitly document the residual risk and why it's accepted if a clean fix isn't feasible without real scope creep. This is a Phase-0 decision, not an assumption baked in up front.
4. Reject-on-creation guard: wrapper-script creation must refuse to write through an existing file, symlink, or reparse point at the target path (TOCTOU / symlink-attack defense — named explicitly in `deferred.md` #8).
5. Verify-before-register: after writing the wrapper and setting its ACL, `--run-as-system` scheduling reads back the resulting ACL and **refuses to register the scheduled task** if it doesn't match the expected restrictive ACL — fail closed, never fail open.
6. Update `Docs/OPERATIONS.md`'s SYSTEM-scheduling section for the new file location + ACL model.
7. Close out `conductor/deferred.md` item #8 (strike through, point at this track).

## 4. Out of scope (do NOT do here)
- T143's original functional scope (env-var baking, `--no-project-context`/`--skip-import`, working directory) — already implemented there; this track only adds the ACL/privilege layer T143 left open.
- Multi-user pipe access / granting `Everyone` or `Authenticated Users` anything — never in scope, matches T144's existing non-goal.
- Auto-elevation UX changes — `install`/`schedule --run-as-system` already requires an elevated shell; this track doesn't touch that, only what happens once elevated.
- Rebuilding T144's Windows service / pipe security model — only cross-check it for this specific gap (§3.2); don't re-open pipe security work that's already Complete.

## 5. Preconditions & dependencies
- **P1 (blocking):** T143 should be functionally working (nightly actually succeeds end-to-end as SYSTEM) before layering ACL hardening on top, so any failure after this track is attributable to this track, not a pre-existing T143 regression. Check T143's live status at Phase 0; coordinate rather than assume.
- *Verified to date:* wrapper path and binary path both confirmed user-writable in this session (§2); the two codex CRITICAL flags on T143 are the direct trigger (`deferred.md` #8).

## 6. Risks
| Risk | Mitigation |
|---|---|
| ACL hardening breaks the nightly task for the real (single-user dev) case it's tested against | DoD requires a live re-run of the actual scheduled task after hardening, not just unit tests. |
| Win32 ACL/security-descriptor code is easy to get subtly wrong | Reuse T144's already-reviewed `pipe_security.rs` pattern (`windows` crate SD construction) as the template rather than inventing a new approach. |
| Scope creep into a general SYSTEM-artifact installer / full binary packaging system | §3.3 bounds this explicitly — Phase 0 picks the narrowest fix and documents the choice; this track is not a packaging redesign. |
| `C:\ProgramData\AI-Brains\` creation needs elevation the first time | Already true of `daemon install` / `schedule --run-as-system` today — no new elevation requirement introduced. |

## 7. Definition of Done
Complete only when ALL hold:
- [ ] **DoD-1 —** Wrapper script for `nightly --schedule --run-as-system` is written to a SYSTEM-controlled location (never a user-writable path), ACL restricted to `SYSTEM:F` + `Administrators:F` — verified via captured `icacls` output in `review.md`.
- [ ] **DoD-2 —** Wrapper-script creation refuses to write through an existing file/reparse point/symlink at the target path; a test proves it.
- [ ] **DoD-3 —** `--run-as-system` scheduling reads back the ACL post-write and refuses to register the scheduled task on any mismatch (fail closed); tests prove both the pass path and the refusal path.
- [ ] **DoD-4 —** Invoked-binary-path risk is either (a) verified ACL-protected before the task runs, or (b) explicitly documented as an accepted residual risk with rationale in `review.md` — a real Phase-0 decision, recorded either way.
- [ ] **DoD-5 —** T144's daemon/service path confirmed to have no equivalent user-writable-artifact gap (or hardened if one is found) — recorded in `review.md` either way.
- [ ] **DoD-6 —** `Docs/OPERATIONS.md` updated; `conductor/deferred.md` item #8 struck through, pointing at this track.
- [ ] **DoD-7 —** Full gate green (`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo deny check`, `cargo audit`, `ledgerful verify --scope full`); the live scheduled task re-registered and manually verified to still succeed end-to-end (recorded in `review.md` — this step is Windows-elevation-dependent and can't run in CI). **STOP before this step** — re-registering the live scheduled task changes real machine state; get explicit go-ahead first.
- [ ] **DoD-8 — Recorded:** outcome in `review.md`; `conductor/conductor.md` status set to Completed; ledger transaction committed in `C:\dev\AI-Brains` (category **SECURITY**).

## 8. Verification commands (reference)
```powershell
# From an elevated PowerShell, only after DoD-1..6 are implemented and reviewed:
ai-brains nightly --unschedule
ai-brains nightly --schedule --run-as-system --start-time 03:00
icacls "C:\ProgramData\AI-Brains\nightly-task.bat"   # expect only SYSTEM:F, Administrators:F
schtasks /Query /TN "AI-Brains-Nightly" /V /FO LIST | Select-String "Task To Run"
schtasks /Run /TN "AI-Brains-Nightly"                # manual trigger, then check the documented log path

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
ledgerful verify --scope full
```
