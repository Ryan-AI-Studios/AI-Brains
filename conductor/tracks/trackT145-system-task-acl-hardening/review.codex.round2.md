## Verdict: FAIL

The C1–C5 fixes address the original findings in the nightly write path, but a new P1 bypass remains in the `daemon.env` repair path.

| DoD | Result |
|---|---|
| DoD-1 | Engineering implementation present; live `icacls` evidence deferred |
| DoD-2 | Met: reparse, junction, symlink, and hardlink guards exist |
| DoD-3 | Partially met: fail-closed code exists; scheduler-boundary tests are incomplete |
| DoD-4 | Met: binary residual risk explicitly accepted and documented |
| DoD-5 | **Failed: daemon sidecar path bypasses parent ACL hardening** |
| DoD-6 | Met: operations docs and deferred item updated |
| DoD-7 | Deferred process gate: live task/full workspace gate open |
| DoD-8 | Deferred process gate: track remains In Progress; ledger unavailable |

### P1 — Existing `daemon.env` branch bypasses parent and link protections

`run_install` calls `ensure_protected_artifact_acl` when `daemon.env` already exists and no environment content is generated ([daemon.rs:318](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/daemon.rs:318)).

That helper only applies/verifies the file ACL ([artifact_security.rs:64](C:/dev/AI-Brains/crates/ai-brains-cli/src/artifact_security.rs:64)). It does not:

- verify or harden the `%ProgramData%\AI-Brains` parent directory;
- reject a reparse point, symlink, or hardlink;
- guarantee the sidecar path is within a protected directory.

The parent currently has user access:

```text
BUILTIN\Users:(I)(OI)(CI)(RX)
BUILTIN\Users:(I)(CI)(WD,AD,WEA,WA)
```

The parent ACL verification exists only in `write_protected_artifact` ([artifact_security.rs:120](C:/dev/AI-Brains/crates/ai-brains-cli/src/artifact_security.rs:120)), so the C1 `None + existing daemon.env` path bypasses the C3 fix. This leaves DoD-5 incomplete.

Required fix: make the existing-sidecar path perform the same parent validation and target reparse/hardlink checks, or route it through a shared protected-artifact validation routine.

### P2 — DoD-3 registration-boundary proof remains incomplete

The tests cover ACL parser success/failure and protected-artifact behavior, but no test injects or observes the scheduler boundary to prove:

- a failed ACL verification results in zero `schtasks` invocation;
- a successful protected write reaches registration with the expected arguments.

The tightened tests resolve the prior soft-pass concern, but they do not fully prove the required scheduling behavior.

### Deferred P3/process items

- `nightly-task.bat` is currently absent, so no live captured `icacls` evidence exists.
- Live elevated re-registration and execution were not performed.
- `cargo fmt --check` passed.
- `cargo nextest run -p ai-brains-cli` could not be independently rerun because `target\debug\.cargo-lock` returned `Access denied`; the reported 159-pass result remains handoff evidence.
- `ai-brains preflight`, Ledgerful doctor/status, and impact/verification were unavailable with `unable to open database file`.
- No files were modified during this review.