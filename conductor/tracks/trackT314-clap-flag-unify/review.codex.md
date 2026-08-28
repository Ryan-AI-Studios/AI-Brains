# T314 Fresh Re-review

**Verdict: PASS WITH DEFERRED P3**

## P0

None.

## P1

None.

- Prior P1-01 is fixed: Denied human output now supplies `Access denied.` when the DTO preview is empty ([governed_query.rs:223](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/governed_query.rs:223)). The new AC16 hermetic test is present ([governed_first_run_deny_exit.rs:445](/C:/dev/AI-Brains/crates/ai-brains-cli/tests/governed_first_run_deny_exit.rs:445)).
- Prior P1-02 is process-only and excluded from this verdict per instruction.

## P2

None.

- AC7 now asserts exactly `ErrorKind::UnknownArgument` ([main.rs:721](/C:/dev/AI-Brains/crates/ai-brains-cli/src/main.rs:721)).

## P3

- **P3-01 — Deferred test-name drift:** AC4 names a separate `query_expand__format_human__parses` test, but human parsing is covered inside `query_expand__format_json__parses` ([main.rs:601](/C:/dev/AI-Brains/crates/ai-brains-cli/src/main.rs:601)). Behavior is covered; this is non-blocking naming cleanup and is already documented in `conductor/deferred.md`.

## Regression sweep

- F1: PASS — optional `--dry-run` applied to progressive and both briefing commands; bare, explicit false, true, and omitted-default unit tests pass.
- F6: PASS — progressive remains JSON-only; AC7 exact guard passes.
- F7: PASS — expand uses the exact seven case-sensitive format tokens and defaults to JSON.
- F9: PASS — human routing emits `kind` then `preview`; JSON retains `applied_scope`.
- F11: PASS — scan-roots accepts and discards the no-op flag; underlying `scan_roots` implementation is unchanged.
- AC16: PASS — Unknown and Denied human paths have two nonempty lines; Denied uses the new fallback.

## Evidence

- Compiled binary unit probes: all relevant parse tests passed, including AC7 and scan-roots conflict guards.
- `cargo fmt --all -- --check`: PASS.
- `git diff --check`: PASS.
- Forbidden-file diff: empty.
- Locked clap version remains 4.6.1; current clap documentation confirms `default_missing_value`/`num_args` are the relevant optional-value mechanisms, with ambiguity considerations documented upstream ([clap Arg docs](https://docs.rs/clap/latest/clap/struct.Arg.html), [ArgAction docs](https://docs.rs/clap/latest/clap/enum.ArgAction.html)).
- Hermetic integration execution was blocked by the managed read-only sandbox: tempdir creation returned `PermissionDenied` across the affected test binaries. This is environmental, not a product failure.
- `ai-brains preflight`, Ledgerful database checks, and GitHub CLI were unavailable because required vault/config/database access is restricted. No files or Git state were modified.