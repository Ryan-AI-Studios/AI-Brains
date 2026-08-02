# Completion review: T192 Doctor CLI

## Verdict

Not complete; do not clear for merge yet.

## P0

None found.

## P1

### P1-01 — `--backup-max-age` can panic and silently fall back

`parse_duration` performs unchecked multiplication for day/week values, causing a production panic on oversized input:

[backup.rs](C:/dev/AI-Brains/crates/ai-brains-brain/src/backup.rs:497)

Reproduction against the existing debug binary:

```text
--backup-max-age 18446744073709551615d
thread 'ai-brains-main' panicked ... attempt to multiply with overflow
exit 101
```

Additionally, [doctor.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/doctor.rs:287) silently substitutes seven days when `chrono::Duration::from_std` fails. Use checked arithmetic and report invalid/overflowing input explicitly.

### P1-02 — Required claims/final-gate closure is not green

`check-release-claims.ps1` fails with three elevated claim hits, including recovery-export lines and a `perfect deletion` line. Rule #54 is removed, but the required claims gate is still red.

The track also remains operationally open:

- `plan.md`: Phase B–E checkboxes remain unchecked.
- `conductor.md`: T192 remains **In Progress**.
- `deferred.md`: deferred #2 remains open.
- No authoritative `review.md` exists.
- Full nextest/deny/audit gate has not passed.
- Ledgerful preflight/doctor/status could not open their databases in this environment.

## P2

### P2-01 — Windows reparse-path test can pass without testing the behavior

[doctor_cli.rs](C:/dev/AI-Brains/crates/ai-brains-cli/tests/doctor_cli.rs:342) prints a skip and passes when `mklink` is unavailable. Thus AC6’s Windows reparse refusal is not always regression-protected. Add an injected/unit-level reparse test or make unavailable link creation fail the test.

### P2-02 — Documentation still contains contradictory shipped-state guidance

[Docs/INSTALL.md](C:/dev/AI-Brains/Docs/INSTALL.md:215) still says the recovery export CLI is missing and advises not to invent one, contradicting T188 and the new shipped command table. This conflicts with the requirement that docs and claims agree.

## P3

### P3-01 — Daemon probe errors are indistinguishable from daemon-down

`probe_restore_daemon_busy` returns only `bool`, so doctor reports every failure as `daemon_reachable: ok / down`; it cannot emit the specified warning for unexpected probe errors. This is non-blocking and likely requires a shared probe API refinement.

### P3-02 — F16 specification text remains stale

The spec repeatedly describes JSON-quoted `event_type`, while live storage trims the quotes. The implementation is correct and AC16 passes, but the frozen spec text could cause a future regression.

## Verified areas

No findings on the requested focus areas for:

- `open_read_intent`-only vault access and early handler placement.
- No `AppContext`/migration path.
- Exit mapping and production-only `process::exit`.
- F16 live event storage/query behavior.
- F17b non-mutating backup reads.
- Core-table and cipher checks.
- Human default format and schema version 1.
- Kit reparse refusal and secret-output protections.
- Capture independence and zero new production dependencies.
- No implementation placeholders or no-op doctor paths.

The implementation is substantially present, but the overflow defect and incomplete/red closeout gates prevent completion approval.