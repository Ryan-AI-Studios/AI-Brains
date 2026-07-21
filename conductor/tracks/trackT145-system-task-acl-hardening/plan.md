# T145 — SYSTEM Task ACL Hardening — Plan

> Phased checklist; phases map to the DoD items in `spec.md` §7. Execute in `C:\dev\AI-Brains`.
> Mark items `- [x]` as completed. Category **SECURITY**. **STOP before the live scheduled-task
> re-registration in Phase 6** (real machine-state change) and before any git push, per repo convention.

> **Ledger:** open a transaction before starting —
> `ledgerful ledger start system-task-acl-hardening --category SECURITY --message "Harden SYSTEM-scheduled nightly/daemon tasks against user-writable-artifact privilege escalation (T145, closes deferred.md #8)"`
> — and commit it in the final phase.

---

## Phase 0 — Baseline + decisions → DoD-4, DoD-5 (partial)

- [x] Confirm T143's current live status: does `nightly --schedule --run-as-system` actually succeed
      end-to-end today? If it's regressed, coordinate/fix before layering this on top (§5 P1).
- [x] Read T144's `pipe_security.rs` (or equivalent) in full — this is the template for Win32
      ACL/security-descriptor code in this codebase; reuse its patterns rather than inventing a new
      approach.
- [x] Investigate T144's Windows-service (`ai-brainsd --service`, `daemon install`) path for any
      user-writable script/binary dependency equivalent to the nightly wrapper. Record the finding
      (clean, or needs the same hardening) — this resolves DoD-5.
- [x] Decide DoD-4: is verifying the `~\.cargo\bin\ai-brains.exe` invocation path ACL-protected
      feasible without real scope creep, or is documenting the residual risk the right call for this
      track? Record the decision + rationale in `review.md` before writing code against it.
- [x] Decide the ACL implementation approach: raw `windows` crate SD construction (matches T144) vs.
      shelling out to `icacls.exe` (simpler, more auditable, arguably safer than hand-rolled Win32 SD
      code) — record the choice.

## Phase 1 — Relocate wrapper + apply ACL → DoD-1

- [x] Add the artifact-security module (ACL construction, modeled on Phase 0's chosen approach).
- [x] Update `write_wrapper_script` (`crates/ai-brains-cli/src/commands/nightly.rs:384-395`) to target
      `C:\ProgramData\AI-Brains\nightly-task.bat` instead of the vault-parent-dir path.
- [x] Apply the ACL (`SYSTEM:F` + `Administrators:F` only) at creation time, before the scheduled task
      is registered.
- [x] Unit test: wrapper is written to the new location with the expected ACL (mock/inspect where the
      real filesystem call can't run in CI; document what's unit-tested vs. what needs the manual
      Phase 6 verification).

## Phase 2 — Reject-on-creation guard → DoD-2

- [x] Before writing the wrapper, check whether the target path already exists as a symlink, reparse
      point, or hardlink pointing elsewhere; refuse creation if so (return a clear error, don't
      silently overwrite through it).
- [x] Test proves the refusal path (construct or simulate a reparse point at the target, assert the
      write is rejected).

## Phase 3 — Verify-before-register → DoD-3

- [x] After writing the wrapper and applying its ACL, read the ACL back and compare against the
      expected restrictive set.
- [x] `--run-as-system` scheduling refuses to call `schtasks /Create` if the ACL doesn't match — fail
      closed, with a clear error naming what was expected vs. found.
- [x] Tests prove both the pass path (correct ACL → registration proceeds) and the refusal path
      (mismatched/missing ACL → registration blocked, non-zero exit).

## Phase 4 — Binary-path decision execution → DoD-4

- [x] Implement whichever Phase-0 decision was made (verify-protected path, or documented accepted
      risk). If "accepted risk," this phase is just the `review.md` writeup — don't invent code to
      solve a problem Phase 0 decided not to solve here.

## Phase 5 — Daemon/service cross-check → DoD-5

- [x] Apply Phase 0's T144 finding: harden the equivalent gap if one was found, or record the clean
      bill of health in `review.md` with the evidence checked.

## Phase 6 — Docs, deferred closeout, gate, live verification → DoD-6, DoD-7

- [x] `Docs/OPERATIONS.md`: update the SYSTEM-scheduling section for the new file location + ACL
      model.
- [x] `conductor/deferred.md`: strike through item #8, point at T145.
- [x] Full gate (local): `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
      `cargo nextest run --workspace` (398 passed). `cargo deny` / `cargo audit` reinstalled and run before PR.
- [x] **STOP cleared by user:** live re-register performed 2026-07-21 (UAC from normal shell).
- [x] Live evidence (see `review.md`): UAC schedule → `C:\ProgramData\AI-Brains\nightly-task.bat`;
      elevated `icacls` shows only SYSTEM:(F) + Administrators:(F); `schtasks` Run As SYSTEM,
      Task To Run = that bat, Last Result 0; `schtasks /Run` SUCCESS.

## Phase 7 — Finalize → DoD-8

- [x] Write `review.md`: Phase-0 decisions, DoD matrix, codex rounds, live verification evidence.
- [x] Update `../conductor.md`: T145 status **Complete**.
- [x] Ledger clean at closeout (0 pending / 0 unaudited drift); SECURITY work landed via PRs #9–#14.
- [x] Notify: none known downstream — leaf hardening track; companion SECURITY.md readiness can reference T145.

---

## Handoff notes

- **Outward-facing/irreversible steps:** none touch anything public — this is entirely local
  machine/repo hardening. The one real "STOP" is Phase 6's live scheduled-task re-registration, which
  changes real state on the user's machine (not reversible by `git revert`, since it's OS scheduler
  state, not source).
- **Ordering that must not be violated:** Phase 0's T143-health check before anything else (don't
  build hardening on top of a broken nightly and conflate two failure sources); ACL applied **before**
  the wrapper is ever referenced by a registered task (never a window where an unprotected wrapper is
  live).
- **Do not:** touch T144's pipe security descriptor code (already Complete, different concern);
  rebuild a general SYSTEM-artifact installer; grant `Everyone`/`Authenticated Users` anything; skip
  the live manual verification in favor of unit tests alone (ACL behavior is OS-level and CI can't
  prove it end-to-end).
- **Relationship to T143:** complementary, not overlapping — T143 = "nightly works as SYSTEM," T145 =
  "the SYSTEM task can't be hijacked." Both should end up Completed; neither blocks starting the other,
  but Phase 0 here checks T143's health first per §5 P1.
