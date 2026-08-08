**Verdict**

PASS

No P0-P2 findings in the working tree re-review.

Evidence:
- Explicit `preflight --install-hooks` now only attempts AGY install when AGY is actually present and not already wired, and it routes that path through a hard-fail reporter: [preflight.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/preflight.rs:278), [preflight.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/preflight.rs:302).
- That reporter now returns `Err` on `InstallOutcome::Refused` and raw install errors when `fail_on_error=true`, which closes the prior silent-success hole for explicit install requests while keeping consent/auto paths soft: [preflight.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/preflight.rs:368), [preflight.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/preflight.rs:393), [preflight.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/preflight.rs:405).
- The earlier non-TTY/auto-install fix is still intact in the shared prompt gate, AGY install still surfaces corrupt hooks as `Refused`, and doctor keeps `harness_wiring` soft-only: [harness.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/harness.rs:338), [install.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/install.rs:219), [doctor.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/doctor.rs:235).

Read-only limitation: `ai-brains preflight`, `ledgerful doctor/status/scan`, and write-producing verification were blocked here because they need local DB/report writes, so this is a source-based gate review of the current diff rather than an executed CI/tooling closeout.