**P0**
None.

**P1**
None.

**P2**
None.

**P3**
- Process closeout is still open in repo state: [conductor.md](/C:/dev/AI-Brains/conductor/conductor.md:173) still lists T226 as `Planning`, and [deferred.md](/C:/dev/AI-Brains/conductor/deferred.md:123) still carries the `policy show/check required scope` row. Per your instruction, I am not failing on this alone; I’m treating it as a deferred process residual for post-merge/orchestrator closeout, not a product defect.

**Verdict: PASS WITH DEFERRED P3**

The prior help-test finding is fixed. The regression lock now asserts both that the `Usage:` line does **not** contain `--scope` and that help mentions soft-resolve in [exit_contract.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/tests/exit_contract.rs:402) and [exit_contract.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/tests/exit_contract.rs:438). The built binary output also matches that: `policy show` prints `Usage: ai-brains.exe policy show [OPTIONS]`, while `policy check` prints `Usage: ai-brains.exe policy check [OPTIONS] --capability <CAPABILITY>`, so `--scope` is optional and `--capability` remains required.

I did not find any new product issue above P3. The wiring is coherent in [policy_cmd.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/policy_cmd.rs:81) and [policy_cmd.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/policy_cmd.rs:140), the AC7 invalid-payload lock is present in [exit_contract.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/tests/exit_contract.rs:516), and lowercase scope canonicalization is covered in [policy_soft_resolve.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/tests/policy_soft_resolve.rs:147) plus [sources.rs](/C:/dev/AI-Brains/crates/ai-brains-control-plane/src/sources.rs:400). I did not rerun the full cargo/nextest gate in this read-only session; I relied on the recorded local gate evidence and direct help-output spot checks from the built binary at current `HEAD` `49061475bfcd772e7f2e2870373b260fbf8180d0`.