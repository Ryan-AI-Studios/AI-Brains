# Track review: T281-NightlyProbeVsTcp

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT281-nightly-probe-vs-tcp`  
**Date:** 2026-08-22  
**HEAD:** `550f3eb`  

---

## Summary

Track T281 resolves an operational ambiguity between `ai-brains nightly --status` and `ai-brains daemon status`:
When a local LLM runner (such as `llama.cpp`) is busy processing an inference slot, its TCP port remains actively listening, so `ai-brains daemon status` reports `LLM backend: Open`. However, because `llama.cpp` queues `/health` requests behind inference (upstream issue #20684), `ai-brains nightly --status` times out on its 750 ms HTTP probe, reporting `Completion: ... probe=timeout (750ms)`.

While T269 previously added `probe=timeout (750ms)` and explained the difference in `--help` and `OPERATIONS.md`, operators inspecting live `--status` were still presented with two seemingly contradictory reports on their terminal.

T281 makes this difference immediately obvious on the status block:
1. **Human Contrast Line (F1):** When the raw Completion probe label is exactly `"timeout"`, `nightly --status` prints an explanatory line immediately underneath:
   ```
   HTTP /health 750ms ≠ daemon TCP
   ```
2. **Strictly Conditional & Minimal (F1 / F4):** When Completion probe is `"ok"`, `"down"`, `"error"`, or when running with `--quick` (`"skipped"`), no contrast line is printed.
3. **Preserving Probe Budgets & Protocols:** Keeps `NIGHTLY_STATUS_PROBE_TIMEOUT` frozen at 750 ms (avoiding status command hangs under load), keeps the status exit code at 0, and leaves machine-readable JSON outputs (`schema_version: 1`, `FROZEN_KEYS`) untouched.
4. **Preserving Daemon Architecture:** Avoids coupling `daemon status` with HTTP probes or introducing complex TCP probing loops into nightly status.

The plan is well-bounded, maintains capture independence, and leaves hotspots (`project.rs`, `sync.rs`, `daemon.rs`, `doctor.rs`) untouched.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Unicode character exactness in `HTTP_VS_TCP_CONTRAST` (F1 / F25 / AC1):** Ensure `HTTP_VS_TCP_CONTRAST` uses the exact mathematical inequality character `≠` (U+2260) and that unit tests assert byte-exact match without converting to ASCII `!=`.
- **m2: Raw probe label input to `completion_timeout_contrast_line` (F1 / AC2):** Ensure the call site passes the raw `completion_label` (`"timeout"`, `"ok"`, `"skipped"`) rather than the human-formatted string `completion_human` (`"timeout (750ms)"`) to prevent false negatives.

### Opportunities (O)
- **O1: Documentation alignment in `OPERATIONS.md` and `CAPABILITIES.md` (F19 / AC11):** Document the contrast line under Completion endpoint status so operators know what to expect during inference slot contention.
- **O2: Parametric unit tests for `completion_timeout_contrast_line` (AC2):** Use `rstest` `#[case]` to assert `Some` for `"timeout"` and `None` for `"ok"`, `"down"`, `"error"`, `"skipped"`, `""`, and uppercase `"TIMEOUT"`.

---

## What Looks Solid

1. **Directly Addresses Real Operational Friction:** Resolves the exact dual-truth scenario observed live on the operator machine (`probe=timeout (750ms)` coexisting with `LLM backend: Open`).
2. **Zero Protocol or Timeout Bloat:** Does not inflate the 750 ms probe timeout or modify JSON schemas.
3. **Saying Just Enough:** Conforms to clig.dev guidelines by printing the contrast line only when the timeout condition occurs, keeping normal `--status` output clean.
4. **Hotspot Restraint:** Edits are strictly isolated to ~3 lines in `crates/ai-brains-cli/src/commands/nightly.rs` and helper definitions in `nightly_status.rs`.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Nightly Completion timeout vs daemon Open | Absorbed into DoD (F1–F5 / AC1–AC2 / AC7 / AC10) | Solved via conditional `HTTP /health 750ms ≠ daemon TCP` line |
| T269 closeout status-block contrast | Absorbed (F1) | Human status block now displays contrast directly |
| Raise 750 ms timeout | Declined (F2) | Kept frozen at 750 ms (T255 F18) |
| Unify daemon TCP with HTTP | Declined (F10) | Preserves fast TCP liveness for daemon |
| JSON contrast field | Declined (F3) | Preserves frozen JSON schema |
| Last-PR Cursor #196 | N/A (empty) | Scanned with 0 findings |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#196](https://github.com/Ryan-AI-Studios/AI-Brains/pull/196) (merged 2026-08-22, T280 `deny HINT omit-scope`).
- **Cursor Comments:** 0 comments (`[]` on PR #196).
- **Disposition:** N/A (no pending findings).

---

## Research / Tools Notes

- **Network Probe Distinctions:** Kubernetes and distributed systems distinguish between TCP socket liveness (port open) and application-layer HTTP readiness (`/health` 200 OK). llama.cpp issue #20684 confirms `/health` queues behind active inference slots.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44), `uuid` (1.23.1), `tokio` (1.52.3).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,581 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search format_probe_label_human`: Located at `crates/ai-brains-cli/src/commands/nightly_status.rs:11`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
