# Track review: T269-NightlyRouterStatusSplit

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT269-nightly-router-status-split`  
**Date:** 2026-08-20  
**HEAD:** `5bfc088`  

---

## Summary

Track T269 addresses operational and human presentation ambiguities in `ai-brains nightly --status` identified during CLI dogfood audits:
1. **Disambiguating Scheduled Tasks:** Human `nightly --status` output previously printed `Last task result: 0` immediately adjacent to `Router: Running  last result: 267009`. Operators misread `267009` (`SCHED_S_TASK_RUNNING` — the expected active keep-alive status for the ONLOGON router task) as a failed nightly sweep. T269 introduces a distinct `Nightly: AI-Brains-Nightly` section header to clearly separate the two tasks.
2. **Clarifying Probe Timeout vs Daemon Liveness:** Full status checks sometimes reported Completion `probe=timeout` (due to the 750ms HTTP `/health` check expiring when llama.cpp is busy generating) while `daemon status` reported LLM ports as `Open` (TCP connect). T269 labels the human probe result as `probe=timeout (750ms)` so operators understand it was a bounded latency budget rather than a dead endpoint.
3. **Preserving Downstream Stability:** Retains all T247 / T255 JSON contracts (`schema_version: 1`, `FROZEN_KEYS`, raw `timeout` token), keeps the 750ms parallel probe budget, avoids CLI dependencies on `reqwest`, and makes no live task scheduler mutations.

The specification and plan are thorough, well-bounded, and completely aligned with codebase reality.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Exact token matching in `format_probe_label_human` (AC2 / F27):** Ensure `format_probe_label_human` only appends the `(750ms)` budget suffix when `label == "timeout"`. All other tokens (`"skipped"`, `"ok"`, `"down"`, `"error"`, or custom error details) must pass through unchanged.
- **m2: Non-Windows heading visibility (F26):** On non-Windows platforms where `schtasks` is omitted, the `Nightly: AI-Brains-Nightly` heading should still print immediately under `=== Nightly Status ===` before `Scheduled: (unknown on non-Windows)`. Unit and hermetic tests running in CI (Ubuntu/macOS) should assert that `Nightly: AI-Brains-Nightly` is always present.

### Opportunities (O)
- **O1: Centralized constant `NIGHTLY_TASK_HEADING`:** Defining `pub(crate) const NIGHTLY_TASK_HEADING: &str = "Nightly: AI-Brains-Nightly";` in `nightly_status.rs` and reusing it across `nightly.rs`, unit tests, and hermetic assertions guarantees zero string drift.
- **O2: Clear distinction in `main.rs` `after_help` (F10 / AC6):** Explicitly documenting in `nightly --help` that `daemon status` probes TCP port connectivity while `nightly --status` probes HTTP `/health` within 750ms provides immediate troubleshooting guidance for operators.

---

## What Looks Solid

1. **Root-Cause Accuracy:** Research into Windows Task Scheduler constants (`SCHED_S_TASK_RUNNING` = `0x00041301` = `267009`) and llama.cpp HTTP queueing behavior (#20684) confirms that neither `267009` nor 750ms timeout indicates a broken installation. Labeling is the correct architectural remediator over artificial budget inflation or status mangling.
2. **Contract & Signature Safety:** Leaving `format_status_schedule_block`, `format_router_status_lines`, and `format_endpoint_line` signatures intact prevents breaking existing T247/T255 unit tests.
3. **JSON Isolation:** Suffixing `(750ms)` only on the human call site preserves frozen JSON tokens (`ok|down|timeout|error|skipped`) and keeps machine-readable output stable.
4. **Hotspot Avoidance:** Zero edits to top hotspots (`project.rs`, `sync.rs`, `daemon.rs`). All helpers and tests remain localized in `nightly_status.rs` and `nightly.rs`.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Nightly human mixes Router 267009 | Absorbed into DoD (F1 / AC1 / AC8 / AC10) | Resolved with `Nightly: AI-Brains-Nightly` heading |
| Completion `probe=timeout` vs daemon Open | Absorbed into DoD (F3 / F4 / AC2 / AC6 / AC10) | Resolved by labeling `probe=timeout (750ms)` |
| `--quick` skips probes | Absorbed into DoD (F2 / AC8 / AC13) | Preserves `probe=skipped` without budget suffix |
| Raise 750ms probe timeout | Declined (F4) | Correctly preserves T247 status latency bounds |
| JSON `probe_budget_ms` / token suffix | Declined (F5 / F21) | Avoids breaking JSON schema consumers |
| Unify daemon TCP to HTTP `/health` | Declined (F18) | Preserves fast liveness probe in daemon |
| Doctor 16th check / persist probe | Declined (F19) | Retains frozen 15-check doctor matrix |
| Peer tracks (T270, T272) | Declined (F20) | Kept strictly isolated |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#185](https://github.com/Ryan-AI-Studios/AI-Brains/pull/185) (merged 2026-08-20, T273 `sync query dash-leading Ledgerful flags`).
- **Cursor Comments:** None (`[]` on PR #185).
- **Prior PR #184 (T268):** Bugbot comment on Linux path units was already resolved on HEAD `5bfc088` with `#[cfg(windows)]`.
- **Disposition:** N/A (no pending Bugbot findings).

---

## Research / Tools Notes

- **Task Scheduler:** `SCHED_S_TASK_RUNNING` (`0x00041301` = `267009`) indicates an actively running keep-alive task.
- **llama.cpp:** Issue #20684 documents `/health` requests queueing behind in-flight token generation under heavy inference load.
- **`clap`:** Locked at `4.6.1`. Additive `after_help` in derive `Commands::Nightly` operates without dependency changes.
- **`tokio` / `serde_json`:** Locked at `1.52.3` and `1.0.150`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,205 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search format_router_status_lines`: Located in `crates/ai-brains-cli/src/commands/nightly_status.rs:146`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
