# Track Completion Audit - T145-SystemTaskAclHardening

## Verdict: FAIL

## Scope Reviewed

Read-only review of `origin/main`, working-tree changes, untracked T145 files, `spec.md`, `plan.md`, `review.md`, implementation, tests, docs, and current `%ProgramData%` ACLs.

Ledgerful/preflight were unavailable: `unable to open database file`.

## Requirement and DoD Matrix

| Item | Status | Evidence |
|---|---|---|
| Wrapper relocation and restrictive ACL | Partial | Code targets ProgramData, but no captured nightly `icacls` evidence; wrapper is currently absent. |
| Reparse/symlink protection | Partial | Production checks exist, but regular existing files and hardlinks remain writable targets. |
| Fail-closed ACL verification | Partial | Nightly path gates registration, but registration pass/refusal behavior is not integration-tested. |
| Binary residual decision | Met | Explicitly accepted and documented. |
| Daemon/service cross-check | Partial | Review records the path, but existing `daemon.env` remains non-restrictive and hardening is conditional. |
| Docs/deferred item | Met | OPERATIONS and deferred item #8 were updated. |
| Full gate/live verification | Unmet | Phase 6 remains unchecked. |
| Finalization/ledger/conductor status | Unmet | Track remains In Progress; review and plan state this openly. |

## Findings

[P1] Existing daemon.env can remain user-readable

Confidence: High  
Requirement: DoD-5; daemon/service artifact hardening  
Location: [daemon.rs:307](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/daemon.rs:307), [windows_service.rs:121](C:/dev/AI-Brains/crates/ai-brainsd/src/windows_service.rs:121)  
Problem: `run_install` only writes and hardens `daemon.env` when `generate_env_sidecar()` returns `Some`. With no environment values, an existing sidecar is neither rewritten nor ACL-verified.  
Evidence: Current `icacls` output shows:

```text
SYSTEM:(I)(F)
Administrators:(I)(F)
Users:(I)(RX)
```

Failure scenario: An existing sidecar violates the documented “SYSTEM + Administrators only” model while the service can still consume it.  
Correction: Always verify an existing sidecar before service registration, and fail closed on mismatch.  
Verification: Test existing weak sidecar plus empty environment; confirm service installation is refused.  
Deferrable: No

[P1] Target creation policy permits existing regular files and hardlinks

Confidence: High  
Requirement: DoD-2 and plan Phase 2  
Location: [artifact_security.rs:101](C:/dev/AI-Brains/crates/ai-brains-cli/src/artifact_security.rs:101), [review.md:31](C:/dev/AI-Brains/conductor/tracks/trackT145-system-task-acl-hardening/review.md:31)  
Problem: The implementation checks only reparse/symlink status, then overwrites regular existing files. Hardlinks are explicitly accepted as a residual. This narrows the specification, which requires refusing an existing target.  
Failure scenario: A pre-created regular file or hardlink is overwritten before ACL application, leaving a race or attacker-controlled inode in the SYSTEM execution path.  
Correction: Use create-new/atomic target creation or reject every existing target, including hardlinks.  
Verification: Add tests for existing regular files and hardlinks.  
Deferrable: No

[P1] Parent directory ACL is not verified fail-closed

Confidence: High  
Requirement: DoD-1; SYSTEM-controlled location boundary  
Location: [artifact_security.rs:90](C:/dev/AI-Brains/crates/ai-brains-cli/src/artifact_security.rs:90), [artifact_security.rs:289](C:/dev/AI-Brains/crates/ai-brains-cli/src/artifact_security.rs:289)  
Problem: The parent receives `/inheritance:r` and grants, but its resulting ACL is never verified. `/inheritance:r` does not remove pre-existing explicit broad ACEs.  
Failure scenario: A parent with explicit user write/delete access remains replaceable, allowing artifact replacement after registration.  
Correction: Verify the parent ACL with the same strict parser and refuse registration if it is not exact.  
Verification: Test a parent containing explicit `Users`/`Everyone` ACEs.  
Deferrable: No

[P1] Required completion gates and evidence are absent

Confidence: High  
Requirement: DoD-1, DoD-7, DoD-8  
Location: [spec.md:55](C:/dev/AI-Brains/conductor/tracks/trackT145-system-task-acl-hardening/spec.md:55), [plan.md:74](C:/dev/AI-Brains/conductor/tracks/trackT145-system-task-acl-hardening/plan.md:74), [conductor.md:90](C:/dev/AI-Brains/conductor/conductor.md:90)  
Problem: Full workspace gate, live elevated re-registration, captured nightly ACL, manual task execution, ledger commit, and Completed status are all still open.  
Evidence: `plan.md` keeps Phase 6/7 unchecked; `review.md` calls them open; current conductor status is In Progress.  
Correction: Complete the authorized Phase 6/7 evidence and update governance artifacts.  
Verification: Record exact command output, `icacls`, `schtasks`, ledger, and final status.  
Deferrable: No

[P2] Tests do not prove the required ACL pass path or registration refusal

Confidence: High  
Requirement: DoD-3 and test requirements  
Location: [artifact_security.rs:700](C:/dev/AI-Brains/crates/ai-brains-cli/src/artifact_security.rs:700), [artifact_security.rs:642](C:/dev/AI-Brains/crates/ai-brains-cli/src/artifact_security.rs:642)  
Problem: The full write test logs an error and still passes; the symlink test also logs a creation failure and passes. No test asserts that `schtasks` is not invoked after ACL failure, nor that a successful protected write reaches registration.  
Correction: Remove success-on-error paths and add an injectable/mock scheduler boundary plus deterministic Windows ACL integration coverage.  
Verification: Force ACL mismatch and assert non-zero result with zero `schtasks` calls; test the successful path separately.  
Deferrable: No

[P2] ACL verifier accepts arbitrary principals named SYSTEM or Administrators

Confidence: High  
Requirement: ACL must contain only the well-known SYSTEM and Administrators identities  
Location: [artifact_security.rs:391](C:/dev/AI-Brains/crates/ai-brains-cli/src/artifact_security.rs:391)  
Problem: `ends_with("\\SYSTEM")` and `ends_with("\\ADMINISTRATORS")` accept domain or other principals with those names instead of requiring the well-known SIDs/canonical identities.  
Correction: Match exact SIDs (`S-1-5-18`, `S-1-5-32-544`) or exact canonical names only; add negative tests for `DOMAIN\SYSTEM` and `DOMAIN\Administrators`.  
Verification: Crafted ACL output must fail closed.  
Deferrable: No

## Completeness Sweep

No production stubs were found. The audit did find weak success-on-error tests, a conditional `None` path that skips daemon sidecar hardening, and an explicitly accepted hardlink narrowing that conflicts with the track specification.

## Wiring and Regression Review

Nightly wiring reaches:

`nightly --schedule --run-as-system` → wrapper generation → protected artifact write → ACL verification → `schtasks`.

The daemon schedule follows the analogous wrapper path. `daemon install` hardens `daemon.env` only when environment content exists; the service later reads that sidecar under SYSTEM.

## Verification Evidence

Observed:

- Working tree includes uncommitted and untracked T145 work.
- `git diff --check` completed without whitespace errors.
- Current `daemon.env` ACL includes `BUILTIN\Users:(RX)`.
- Current nightly wrapper is absent.

Reported by handoff, not independently observed:

- `cargo nextest run -p ai-brains-cli`: 153 passed.
- Package clippy clean.

Not completed/not verifiable:

- Full workspace gate.
- Elevated live task re-registration and execution.
- Ledgerful full verification and ledger transaction.

## Deferred Candidates

None. All identified issues affect security, DoD compliance, or required evidence.

## Completion Decision

T145 is not complete and should remain In Progress.