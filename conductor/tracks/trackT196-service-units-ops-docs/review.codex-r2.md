## Verdict

# PASS WITH DEFERRED P3

## P1 verification — fixed

[wrapper](C:/dev/AI-Brains/packaging/reference/launchd/ai-brainsd.wrapper.sh.example:31) now:

- Fails closed when `ls` fails.
- Fails closed on missing/unparseable mode.
- Rejects any group/other permissions.
- Requires owner read permission.

The reported dry checks are consistent: `0644`/`0640` fail; `0600`/`0400` pass.

## P0–P2 re-sweep

No new P0, P1, or engineering P2 findings in packaging, documentation, or SIGTERM wiring.

The remaining ledger/conductor/deferred closeout is process work explicitly expected before ship and is not scored as a failure here.

## AC1–AC14 matrix

| AC | Result | Evidence |
|---|---|---|
| AC1 | Met | systemd user unit, `Type=simple`, dual paths, light hardening |
| AC2 | Met | LaunchAgent, `KeepAlive` dict, `SuccessfulExit=false`, wrapper |
| AC3 | Met | Packaging README honesty, linger, UDS, vault, secrets, suspend guidance |
| AC4 | Met | `CONTRIBUTING.md` gate, license, workflow, changelog policy |
| AC5 | Met | OPERATIONS, INSTALL, and Docs README links |
| AC6 | Met | Compatibility and release-claims reworded honestly |
| AC7 | Met | No MSI/App Store/T1/multi-user overclaims |
| AC8 | Process pending | Deferred strike/conductor completion intentionally not scored as failure |
| AC9 | Met | CHANGELOG Unreleased entry |
| AC10 | Met | Known script gate passes; Bash 3.2 collector is static-safe |
| AC11 | Met | No real keys or secrets in templates |
| AC12 | Met | Non-root system unit with `ReadWritePaths` |
| AC13 | Met | No active forbidden `ProtectHome` or relative documentation path |
| AC14 | Met | SIGTERM documented and wired |

## Deferred P3

The prior SIGTERM finding remains valid: tests prove construction/abortability and module linkage, but do not deliver SIGTERM to a daemon child process and verify graceful exit.

This is non-blocking and suitable for deferred P3 status.

## Verification notes

- `git diff --check`: pass.
- Known `check-reference-units.sh`: pass.
- Known `ai-brainsd` clippy/nextest: previously green.
- Local Bash rerun was blocked by `E_ACCESSDENIED`, not a script failure.
- `ledgerful doctor`, ledger status, and AI-Brains preflight remain blocked by `unable to open database file`.
- No files were modified.