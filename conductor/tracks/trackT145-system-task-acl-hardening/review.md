# T145 Review Log — SYSTEM Task ACL Hardening

## Phase 0 Decisions (recorded before implementation)

### D0.1 — T143 health (DoD precondition §5 P1)

- T143 code is on `main` (`c7585d3`, `634249e`): wrapper generation, env bake-in, `--no-project-context --skip-import`, `/RU SYSTEM`.
- Functional success as SYSTEM still depends on elevated re-registration; ACL work does not depend on a live successful nightly run for code correctness.
- Proceed with T145; live SYSTEM re-run remains a Phase-6 elevated gate (STOP before re-register).

### D0.2 — ACL implementation approach

- **Initial choice:** `icacls` apply + verify (Phase 0).
- **Final (after live UAC failures):** **absolute DACL** via SDDL  
  `D:P(A;;FA;;;SY)(A;;FA;;;BA)` + `ConvertStringSecurityDescriptorToSecurityDescriptor` +  
  `SetNamedSecurityInfo` with `PROTECTED_DACL_SECURITY_INFORMATION` (PR #13).  
  Incremental `icacls /grant` left Windows `LogonSessionId_*` ACEs that `/remove` could not strip.
- **Verify:** still fail-closed via `icacls` query parse (`SYSTEM` + `Administrators` full only).
- Reparse detection: `GetFileAttributesW` / `FILE_ATTRIBUTE_REPARSE_POINT` + `Path::is_symlink`.

### D0.3 — DoD-4 binary path residual risk

- **Accepted residual risk (option b):** invoked `ai-brains.exe` / `ai-brainsd.exe` typically live under `%USERPROFILE%\.cargo\bin\` (user-writable by design for `cargo install` updates).
- Copying binaries into `ProgramData` at schedule time is packaging/installer scope and out of this track (§3.3, §4).
- Wrapper + env sidecar move to ACL-restricted `%ProgramData%\AI-Brains\` closes the primary hijack vector on the *script* path. Residual binary risk is documented in OPERATIONS.md and here.
- Optional future track: ship a SYSTEM-owned install root under `Program Files` with update tooling.

### D0.4 — DoD-5 T144 daemon/service cross-check

Findings:
1. **`daemon install` service `binPath`** points at user-writable cargo bin — same residual binary risk as D0.3; document, do not re-package here.
2. **`%ProgramData%\AI-Brains\daemon.env`** is written by `daemon install` **without** restrictive ACL today → **harden in this track** (same write-protected artifact path as wrappers).
3. **Deprecated `daemon schedule --run-as-system`** still writes `.ai-brains-daemon-task.bat` under vault parent / TEMP → **relocate + ACL** same as nightly.

### D0.5 — Reparse / existing-file policy (DoD-2)

- Refuse if target path **exists as symlink or reparse point** (do not write through).
- Allow replace of a **regular** existing file (re-schedule must update the wrapper), after reparse check and with ACL re-applied + verified before task registration.

---

## Implementation summary (code phase)

Implemented without live `schtasks` re-register (Phase 6 STOP still open):

1. **`crates/ai-brains-cli/src/artifact_security.rs`** — path helpers, reparse/symlink detect (`symlink_metadata` + `GetFileAttributesW` / `FILE_ATTRIBUTE_REPARSE_POINT`), `icacls` apply (`/inheritance:r` + SID grants S-1-5-18 / S-1-5-32-544 full), ACL verify via `acl_output_is_restrictive`, `write_protected_artifact` fail-closed pipeline.
2. **`nightly.rs`** — wrapper → `nightly_wrapper_path()` via `write_protected_artifact`; dry-run placeholder is ProgramData path; no `schtasks` if write/verify fails.
3. **`daemon.rs`** — daemon schedule wrapper → `daemon_wrapper_path()`; `daemon install` env sidecar → `daemon_env_path()` + `write_protected_artifact`; dry-run messages updated.
4. **Docs** — `Docs/OPERATIONS.md` SYSTEM section + residual binary risk; `conductor/deferred.md` #8 struck with T145 pointer.
5. **Tests** — pure ACL parse (ok / Everyone / Users / missing SYSTEM), path helpers, regular-file reparse false, symlink refuse when creatable, optional temp write+icacls (may skip without elevation).

### Residual risks (accepted)

- Cargo-bin `ai-brains.exe` / `ai-brainsd.exe` user-writable (D0.3 accepted).
- Codex deferred P3: no mock-`schtasks` integration test (wiring uses `?` + prepare gate + pure tests).

## Findings

### R1 HIGH — Parent directory reparse/junction not refused
- **Status:** `verified_fixed` (internal re-review 2026-07-21)
- **Fix:** `write_protected_artifact` now refuses when `path.parent()` exists as reparse/symlink/junction *before* `create_dir_all`, re-checks parent after create, then proceeds to file reparse check → write → post-write recheck → ACL.
- **Tests:** `write_protected_artifact__parent_junction__refuses` (`mklink /J`).

### R2 MEDIUM — Weak symlink test
- **Status:** `verified_fixed` (internal re-review 2026-07-21)
- **Fix:** Added pure `refuse_if_reparse(path, is_reparse)` used by production; unit tests `refuse_if_reparse__true__err` / `refuse_if_reparse__false__ok`; FS proof via parent junction test; removed hardcoded tautology soft-pass string assert.

### R3 MEDIUM — DoD-3 proof
- **Status:** `verified_fixed` (internal re-review 2026-07-21)
- **Fix:** Always-on `verify_restrictive_acl__default_user_temp_file__err` (default inherited ACLs must fail verify). Pure `acl_output_is_restrictive` tests retained. Full write+icacls temp test hard-asserts on `Ok`; documents non-elevated `Err` without soft-pass.

### R4 MEDIUM — TOCTOU check-then-write
- **Status:** `verified_fixed` (internal re-review 2026-07-21)
- **Fix:** After `std::fs::write`, immediately re-check `is_reparse_or_symlink`; on true, best-effort `remove_file` and return `Err` via `refuse_if_reparse`.

### R5 MEDIUM — empty DACL if grant fails
- **Status:** `verified_fixed` (internal re-review 2026-07-21; residual accepted)
- **Fix:** On `icacls /grant` failure after `/inheritance:r`, best-effort `std::fs::remove_file(path)` (files only; no-op error on dirs). Residual empty-DACL window documented above; registration remains fail-closed.

### R6 LOW — hardlink residual
- **Status:** `fixed_pending_verification` (superseded by codex C2 — hardlink refuse now implemented)
- **Note:** Originally accepted residual out of D0.5; codex review required refuse for nlink > 1 while still allowing regular single-link replace for re-schedule.

### Internal review rounds
1. Primary: NEEDS_FIXES (R1–R6 open)
2. Fix pass applied
3. Re-review: **CLEAN** for engineering DoD-1..6 (code/docs); DoD-7/8 process-open (Phase 6 STOP live re-register)
4. Codex cross-model: FAIL with C1–C6; C1–C5 engineering fixes applied; C6 process-open (orchestrator gate)

### Codex cross-model findings (2026-07-21)

### C1 P1 — Existing daemon.env not hardened when `generate_env_sidecar` is None
- **Disposition:** VALIDATED
- **Status:** `fixed_pending_verification`
- **Fix:** `run_install` now: `Some` → `write_protected_artifact`; `None` + path exists → `ensure_protected_artifact_acl` (apply+verify, fail closed); `None` + missing → skip. Comment documents the three branches.
- **Tests:** pure ACL apply/verify helpers already cover fail-closed; install path is structural + comment (no live sc create).

### C2 P1 — Existing regular file vs hardlink
- **Disposition:** PARTLY VALID (regular replace kept per D0.5; hardlink refuse required)
- **Status:** `fixed_pending_verification`
- **Fix:** Before overwrite, `is_hardlink` via `GetFileInformationByHandle` (`nNumberOfLinks > 1`) + pure `refuse_if_hardlink`. Regular single-link existing files still replaceable for re-schedule.
- **Tests:** `refuse_if_hardlink__*`, `write_protected_artifact__hardlink_target__refuses` (`std::fs::hard_link`).

### C3 P1 — Parent directory ACL not verified
- **Disposition:** VALIDATED
- **Status:** `fixed_pending_verification`
- **Fix:** After `apply_restrictive_acl` on the `AI-Brains` parent, call `verify_restrictive_acl(parent)` and fail closed on mismatch.

### C4 P2 — ACL principal matching too loose
- **Disposition:** VALIDATED
- **Status:** `fixed_pending_verification`
- **Fix:** `is_system_principal` / `is_administrators_principal` accept only well-known forms (`S-1-5-18`, `SYSTEM`, `NT AUTHORITY\SYSTEM`; `S-1-5-32-544`, `ADMINISTRATORS`, `BUILTIN\ADMINISTRATORS`). Removed `ends_with("\\SYSTEM")` / `ends_with("\\ADMINISTRATORS")` / broad `contains`. Also fixed ACE principal extraction so path-prefixed `NT AUTHORITY\SYSTEM` is not split on the space (was masked by the old loose matcher).
- **Tests:** `acl_output_is_restrictive__domain_system__err`, `acl_output_is_restrictive__domain_administrators__err`, `extract_ace_segment__path_prefixed_nt_authority_system__full_principal`.

### C5 P2 — Soft-pass tests
- **Disposition:** VALIDATED (narrowed)
- **Status:** `fixed_pending_verification`
- **Fix:** Symlink creation failure early-returns after `eprintln` (pure refuse + junction remain hard proof). Regular-file temp write on `Err` asserts non-empty ACL/icacls/inheritance-shaped message. Missing-SYSTEM pure fail message asserts SID/full-control detail.

### C6 P1 PROCESS — completion gates
- **Disposition:** VALIDATED (process)
- **Status:** `verified_fixed` — live Phase 6 completed 2026-07-21 (see Live verification evidence below); track marked Complete.

### Observed gate (package-scoped)
- `cargo nextest run -p ai-brains-cli` — 163 passed, 0 skipped (post C1–C5 + ensure_parent on ensure path)
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` — clean

### Codex cross-model (gpt-5.6-luna high)
1. **Round 1** (`review.codex.md`): FAIL — P1 C1–C4, P2 C5, process C6
2. **Fixes C1–C5** applied
3. **Round 2** (`review.codex.round2.md`): FAIL — residual P1: `ensure_protected_artifact_acl` bypassed parent ACL/reparse; P2 scheduler-boundary tests incomplete
4. **Round-2 fix:** `ensure_protected_artifact_acl` now runs `ensure_parent_protected` + file reparse/hardlink refuse before file ACL; pure `may_register_after_prepare` wired into nightly/daemon schedule paths + unit tests
5. **Round 3** (`review.codex.round3.md`): FAIL — `None`+missing `daemon.env` skipped parent protect
6. **Fix:** `ensure_program_data_ai_brains_dir()` always before `sc create`; dangling reparse refuse
7. **Round 4** (`review.codex.round4.md`): **PASS WITH DEFERRED P3**
   - P0/P1/P2: none open
   - Deferred P3: no mock-`schtasks` boundary integration test (wiring uses `?` + prepare gate + pure tests)
8. Follow-up PRs after codex: #10 UAC auto-elevate, #11 elevate env/cwd, #12 LogonSession strip (superseded), #13 absolute SDDL, #14 elevate wait/result UX

### Live verification evidence (DoD-1, DoD-7) — 2026-07-21

**Host:** DESKTOP (user RyanB). Recorded from user-run commands after absolute-SDDL + UAC path landed on `main`.

#### Non-elevated shell (schedule via UAC)

```text
PS C:\dev\AI-Brains> ai-brains nightly --schedule --run-as-system --start-time "03:00"
Administrator elevation is required. Showing UAC prompt (approve to continue)...
Nightly task 'AI-Brains-Nightly' scheduled daily at 03:00.
Wrapper script: C:\ProgramData\AI-Brains\nightly-task.bat
Elevated schedule finished successfully.
Task: AI-Brains-Nightly (SYSTEM)
Wrapper: C:\ProgramData\AI-Brains\nightly-task.bat
Note: that path is ACL-restricted (SYSTEM/Administrators only);
listing/icacls from a non-elevated shell may say Access denied.
Verify with an elevated shell or: schtasks /Query /TN AI-Brains-Nightly
```

#### Elevated shell (verify ACL + task + manual run)

```text
PS C:\dev\AI-Brains> icacls "$env:ProgramData\AI-Brains\nightly-task.bat"
C:\ProgramData\AI-Brains\nightly-task.bat NT AUTHORITY\SYSTEM:(F)
                                          BUILTIN\Administrators:(F)
Successfully processed 1 files; Failed processing 0 files

schtasks /Query /TN "AI-Brains-Nightly" /V /FO LIST
  TaskName:                             \AI-Brains-Nightly
  Next Run Time:                        7/22/2026 3:00:00 AM
  Status:                               Ready
  Last Run Time:                        7/21/2026 6:07:47 AM
  Last Result:                          0
  Task To Run:                          "C:\ProgramData\AI-Brains\nightly-task.bat"
  Run As User:                          SYSTEM
  Schedule Type:                        Daily
  Start Time:                           3:00:00 AM

schtasks /Run /TN "AI-Brains-Nightly"
SUCCESS: Attempted to run the scheduled task "AI-Brains-Nightly".
```

### DoD matrix (final)

| DoD | Status | Evidence |
|-----|--------|----------|
| DoD-1 Wrapper ProgramData + SYSTEM/Admins ACL | **Met** | Live `icacls` above |
| DoD-2 Reparse/symlink/hardlink refuse | **Met** | Code + unit tests (junction, hardlink, pure refuse) |
| DoD-3 Fail-closed ACL verify before schtasks | **Met** | `write_protected_artifact` + gate; live path succeeds only after verify |
| DoD-4 Binary residual | **Met** | D0.3 documented OPERATIONS + review |
| DoD-5 Daemon path | **Met** | `daemon.env` + daemon wrapper protected; residual bin documented |
| DoD-6 Docs + deferred #8 | **Met** | OPERATIONS + deferred.md struck |
| DoD-7 Full gate + live re-register | **Met** | PRs merged; live schedule/icacls/run above |
| DoD-8 review + conductor + ledger | **Met** | This closeout; conductor Complete; ledger clean at closeout |

### Merged PRs (T145 + follow-ups)

| PR | Title | Merged |
|----|-------|--------|
| #9 | feat(T145): SYSTEM task ACL hardening | 2026-07-21 |
| #10 | feat: UAC auto-elevate for SYSTEM schedule/install | 2026-07-21 |
| #11 | fix: UAC elevate cwd + env handoff | 2026-07-21 |
| #12 | fix: strip LogonSessionId ACEs (superseded by #13) | 2026-07-21 |
| #13 | fix: absolute SDDL ACL for SYSTEM task artifacts | 2026-07-21 |
| #14 | fix: reliable UAC elevate wait + parent summary | 2026-07-21 |

### Final status

- **Track: Complete** (2026-07-21)
- Codex: **PASS WITH DEFERRED P3** (only deferred P3: mock-schtasks test)
- Live SYSTEM nightly: **registered and verified**