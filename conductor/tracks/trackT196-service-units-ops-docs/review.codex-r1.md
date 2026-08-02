## Verdict: FAIL

## Scope

Read-only audit of the uncommitted T196 changes, including `spec.md`, `plan.md`, packaging templates, docs, claims, daemon SIGTERM wiring, tests, and track governance.

## Requirement matrix

| Requirement | Result |
|---|---|
| AC1 systemd user unit | Pass |
| AC2 launchd LaunchAgent / KeepAlive / wrapper | Partial — wrapper permission flaw |
| AC3 packaging README honesty | Pass |
| AC4 CONTRIBUTING.md | Pass |
| AC5 documentation links | Pass |
| AC6 compatibility/release claims | Pass |
| AC7 no MSI/T1/multi-user overclaims | Pass |
| AC8 deferred disposition | Partial — registry/closeout still open |
| AC9 CHANGELOG | Pass |
| AC10 validation script | Pass per reported gate; not rerun locally because Bash execution is denied |
| AC11 no committed secrets | Pass statically |
| AC12 system unit non-root/RW guidance | Pass |
| AC13 ProtectHome/ProtectSystem/Documentation footguns | Pass |
| AC14 SIGTERM documentation and wiring | Pass implementation; test proof incomplete |

## Findings

### P1 — Launchd wrapper does not enforce owner-only secret-file permissions

[ai-brainsd.wrapper.sh.example](C:/dev/AI-Brains/packaging/reference/launchd/ai-brainsd.wrapper.sh.example:28) only rejects group/world-writable files. It permits `0640` and `0644`, allowing other users to read `AI_BRAINS_VAULT_KEY`. Additionally, `ls` failure is swallowed via `|| true`, leaving an empty mode that is accepted.

This conflicts with F34’s 0600 secrets-wrapper requirement and creates a local secret-disclosure footgun.

Required: fail closed when mode detection fails and require owner-only permissions before sourcing the environment file.

### P2 — Track completion and provenance are not complete

[plan.md](C:/dev/AI-Brains/conductor/tracks/trackT196-service-units-ops-docs/plan.md:14) still leaves the decision pin and ledger transaction unchecked. D6, final verification, and closeout remain incomplete. [conductor.md](C:/dev/AI-Brains/conductor/conductor.md:142) still says “Expanded + AI fold-in” and “Implement on go-ahead,” not Completed.

`ledgerful doctor`, `ledgerful ledger status`, and `ai-brains preflight --summary` all failed with `unable to open database file`, so provenance and verification state could not be confirmed. No final `review.md` is present.

### P3 — SIGTERM behavior lacks a real delivery test

The new tests only prove that the signal future can be spawned/aborted and that the Unix module links. They do not send SIGTERM and verify graceful completion. This is non-blocking because the code is wired and F36 permits a soft implementation, but a real Unix child-process signal test would improve proof.

## Completeness

The reference units, documentation, claims rewrite, Keep a Changelog decision, absolute vault guidance, XDG/UDS notes, linger tradeoff, KeepAlive dictionary, and ProtectHome/ProtectSystem safeguards are present.

`REPLACE_ME` appears only in the intentionally operator-filled launchd paths. No MSI, App Store, Unix installer CLI, multi-user product claim, or new production dependency was introduced.

## Wiring

SIGTERM is reachable through:

- `shutdown_signal::wait_shutdown_signal`
- Unix UDS accept-loop shutdown
- top-level daemon shutdown select
- `lib.rs` module export

Windows SCM remains on its separate service-control path. The daemon remains foreground-only.

## Verification

Observed:

- `git diff --check`: pass
- Reported `scripts/check-reference-units.sh`: pass
- Reported `cargo clippy -p ai-brainsd`: pass
- Reported `cargo nextest run -p ai-brainsd --lib`: 43/43 pass
- Bash execution locally: unavailable due access denial
- Ledgerful and AI-Brains preflight: database-open failure

## Deferred candidates

Only the SIGTERM delivery test is a reasonable difficult, non-blocking P3 candidate. The wrapper security defect and provenance/track closeout must not be deferred.

## Completion Decision

Do not mark T196 complete yet. Fix the wrapper permission check, restore ledgerful access and complete provenance/closeout, then rerun verification.