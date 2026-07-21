Verdict: **PASS WITH DEFERRED P3**

P1 is closed. `run_install` now calls `ensure_program_data_ai_brains_dir()` before any `sc create`; it refuses parent reparses, applies/verifies the restrictive ACL, and all `daemon.env` branches are fail-closed. The missing-sidecar branch also rejects dangling reparses.

Fresh sweep:

- DoD-1–6 engineering implementation is present.
- DoD-2 guards and tests cover reparses, junctions, symlinks, and hardlinks.
- DoD-3 wiring uses `?` plus `may_register_after_prepare`; no scheduler-boundary mock test exists, deferred as P3.
- `cargo fmt --check` and `git diff --check` pass.
- Fresh clippy/nextest attempts were blocked by `target\debug\.cargo-lock` access denied; supplied evidence reports workspace nextest 398 passed and 163 CLI tests.
- Live tasks are absent; the existing service/artifacts remain unrefreshed. No live `schtasks` re-registration was performed, per instruction.
- Ledgerful/preflight remain unavailable due `unable to open database file`.