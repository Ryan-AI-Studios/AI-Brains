**P0**
None.

**P1**
None.

**P2**
- The track is not ready for completion signoff because the required closeout evidence is still missing. `AC11` and the Definition of Done require full-gate success, manual dogfood evidence, conductor/deferred closure, and ledger finalization, but the repo still shows T226 as `Planning` in [conductor.md](C:/dev/AI-Brains/conductor/conductor.md:173), the deferred row is still open in [deferred.md](C:/dev/AI-Brains/conductor/deferred.md:123), the finalize/manual steps remain unchecked and `# pending` in [plan.md](C:/dev/AI-Brains/conductor/tracks/trackT226-policy-soft-resolve-scope/plan.md:137), and the existing review log explicitly says targeted tests and manual dogfood were not run in that round in [review.md](C:/dev/AI-Brains/conductor/tracks/trackT226-policy-soft-resolve-scope/review.md:123). That leaves [spec.md](C:/dev/AI-Brains/conductor/tracks/trackT226-policy-soft-resolve-scope/spec.md:134) and [spec.md](C:/dev/AI-Brains/conductor/tracks/trackT226-policy-soft-resolve-scope/spec.md:223) unmet as written.

**P3**
- The new help tests do not actually prove that `--scope` stopped being clap-required. Both `policy_show__help__scope_optional_soft_default` and `policy_check__help__scope_optional_soft_default` only assert that the `Usage:` line contains `[OPTIONS]`, but clap still includes `[OPTIONS]` when a required `--scope` is present, so a regression back to required scope could still pass these tests. See [exit_contract.rs](C:/dev/AI-Brains/crates/ai-brains-cli/tests/exit_contract.rs:417) and [exit_contract.rs](C:/dev/AI-Brains/crates/ai-brains-cli/tests/exit_contract.rs:446). The implementation in [main.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/main.rs:1221) and [main.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/main.rs:1237) is correct today; the gap is the regression lock.

No additional product defects stood out in the implementation itself. The soft-resolve wiring and canonicalization are coherent in [policy_cmd.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/policy_cmd.rs:73), [policy_cmd.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/policy_cmd.rs:130), the case-insensitive scope parse is narrowly implemented in [sources.rs](C:/dev/AI-Brains/crates/ai-brains-control-plane/src/sources.rs:400), and the seeded hermetic coverage for AC4/AC5/AC12 is present in [policy_soft_resolve.rs](C:/dev/AI-Brains/crates/ai-brains-cli/tests/policy_soft_resolve.rs:67).

I did not re-run `cargo`/`nextest` in this session because the workspace is read-only, so the process finding above is based on recorded repo state rather than fresh execution.