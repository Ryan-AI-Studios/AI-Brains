# T249 Internal R1b — Correctness / tests

**Track:** T249-ScopeDaemonDoctorPresentation  
**Reviewer:** correctness subagent  
**Verdict:** **PASS** with 3 easy test locks (not product defects)

## Findings

| ID | Sev | Status | Action |
|----|-----|--------|--------|
| R1b-1 | suggestion | **verified_fixed** | Clap unit `scope_resolve__default_format__auto` locks omitted `--format` → `"auto"` |
| R1b-2 | suggestion | **verified_fixed** | AC11 hermetic asserts `checks.len() == 15` on both JSON-win paths |
| R1b-3 | nit | **verified_fixed** | Authoritative human unit asserts ` (authoritative)` and not `NOT authoritative` |

Hunt negatives (unwrap, OutputFormat::parse on scope, missing DoctorOptions.summary, JSON not winning, next: on Running/authoritative, live-daemon Stopped hermetic, contracts, pin bumps, doctor TTY-switch, shared resolver): clean.

Re-run: 3/3 PASS.
