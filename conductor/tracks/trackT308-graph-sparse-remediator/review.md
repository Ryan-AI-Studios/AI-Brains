# T308 Review Log — Sparse remediator honesty

**Status:** **Completed** (full `dev-check.ps1` exit 0; Phase 6 publish follows)
**Category:** FEATURE / CLI
**Ledger:** FEATURE TX `d62a3884-5af8-44fc-9434-3b8c31a656af`
**Date:** 2026-08-26
**Branch:** `track/T308-graph-sparse-remediator`

## Phase 0 evidence

| Check | Result |
|-------|--------|
| cwd | `C:\dev\AI-Brains` |
| Floors | `MIN_EDGE_NODE_RATIO = 0.50` unchanged (`graph_density.rs:14`) |
| Sparse arm (before) | `remediation: Some(remediation.into())` |
| Sparse arm (after) | graph-on `None`; graph-off `Some(GRAPH_REINSTALL_SOOT)` |
| Doctor | still forwards `assessment.remediation` (`doctor.rs:914`); matrix **15** |
| `has_graph_tables` | sqlite_master unchanged (T309 not stolen) |
| PATH doctor (pre-install) | still shows remediator rebuild — expected until PATH refresh |
| AC9 cargo-run graph-on | Sparse warn E/N=0.410; **no `remediation` key** in JSON |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| F1 | — | Graph-on Sparse remediator was rebuild loop | `verified_fixed` | AC1/AC4 units green; Sparse arm `None` |
| R1 | low-info | Live E/N still ~0.41 after T300 | deferred | Floors frozen F1; expected honest sparse |
| R2 | low-info | Never-rebuilt Sparse has no rebuild remediator | deferred | By design F2; empty_lag/orphan still rebuild |
| R3 | low-info | PATH binary still emits rebuild remediator until `cargo install` | deferred | Soft; hermetic/source SoT; F12 |
| R4 | low-info | `recovery_kit_event` doctor warn | deferred | Not this track (T306 R5) |
| R5 | low-info | Floor retune 0.50 declined | deferred | F1 standing |

No critical / high / medium. No regression on lag arms (AC2 stay-green). No doctor.rs growth.

## Cross-model

FEATURE → **codex-review**. `review.codex.md` verdict **PASS WITH DEFERRED P3** (P3-1 live E/N; P3-2 never-rebuilt omit by design; P3-3 PATH stale). No P0–P2.

## DoD

| AC | Result |
|----|--------|
| AC1 | **Met** — `…__no_rebuild_remediator` + `…ratio_0_4` remediator None + lag note |
| AC2 | **Met** — empty_lag / orphan / projection_lag rebuild units PASS |
| AC3 | **Met** — graph-off Sparse reinstall SOOT PASS |
| AC4 | **Met** — `…__omits_remediation` with `--features graph` PASS |
| AC5 | **Met** — floor 0.50; smoke F17 PASS |
| AC6 | **Met** — matrix 15; doctor forward unchanged |
| AC7 | **Met** — clippy `-p ai-brains-cli` exit 0; targeted nextest green |
| AC8 | **Met** — OPERATIONS / CAPABILITIES / CHANGELOG updated |
| AC9 | **Met** — `cargo run --features graph -- doctor --format json` Sparse omits remediator |
| AC10 | **Met** — `has_graph_tables` unchanged |

## Closure

Conductor → **Completed** after full CI gate exit 0 + Phase 6 publish. Residuals → `conductor/deferred.md`. T306 R4 marked Done. T307 / T309 / T310 not stolen.
