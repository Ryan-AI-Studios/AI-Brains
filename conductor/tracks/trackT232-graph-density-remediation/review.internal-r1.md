# T232 Internal Review R1

**Track:** T232-GraphDensityRemediation  
**Branch:** `feat/T232-graph-density-remediation`  
**Reviewer:** internal-r1 (read-only)  
**Date:** 2026-08-10  
**Scope:** Implementation vs `spec.md` F1–F31 / AC1–AC19; audit focus checklist

## Verdict: CLEAN

No critical / high / medium findings. Implementation matches the capability-aware remediation design: dual SOOT, pure `_with` API, hybrid empty-lag retired, doctor gather-error via `density_remediation`, H1 test migration, F17 smoke extension, docs/skill honesty, thresholds/matrix frozen. Only residual low/info notes below (process / non-blocking completeness).

## AC Matrix

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** Graph-on sparse/orphan/projection_lag → exact rebuild | **PASS** | `assess_graph_density_with__sparse_1304_95_graph_on__rebuild`, `…orphan_graph_on__rebuild`, `…projection_lag_graph_on__rebuild`, `…ratio_0_4__warn_sparse` — `assert_eq!(…, Some(REMEDIATION_REBUILD))` |
| **AC2** Graph-off sparse/orphan/projection_lag → exact `GRAPH_REINSTALL_SOOT` | **PASS** | dual `…_graph_off__reinstall_soot` units; `assert_eq!(…, Some(GRAPH_REINSTALL_SOOT))`; empty_lag also `assert_ne!(…, Some(REMEDIATION_REBUILD))` |
| **AC3** Graph-on empty_lag → rebuild only (no SOOT parenthetical) | **PASS** | `assess_graph_density_with__empty_lag_graph_on__rebuild_only` asserts rebuild + `!contains(GRAPH_REINSTALL_SOOT)` |
| **AC4** Graph-off empty_lag → `GRAPH_REINSTALL_SOOT` only | **PASS** | `…empty_lag_graph_off__reinstall_soot_only` |
| **AC5** Skip/Ok → remediation `None` | **PASS** | skip `status=live` + `remediation.is_none()`; Ok paths `remediation.is_none()`. Code is capability-blind on these arms (see residual L1 for dual-bool unit gap) |
| **AC6** Verdicts/density/status/thresholds/priority; Skip `status=live` | **PASS** | floors `MIN_PINNED=100`, `MIN_NODES=50`, `MIN_EDGE_NODE_RATIO=0.50`, `MIN_MEMORY_COVERAGE=0.10`; priority units empty→orphan→sparse; skip `status=live` / `density=skip` (F31 reject of empty) |
| **AC7** Doctor gather-error → `density_remediation(cfg!(…))` | **PASS** | `doctor.rs` ~704: `Some(crate::graph_density::density_remediation(cfg!(feature = "graph")).into())`; helper units true/false; O2 cfg-not-graph integration optional (skipped — allowed) |
| **AC8** Feature-off still emits `graph_density` | **PASS** | always-compiled gather/assessor; existing doctor open-failed / vault regressions; no `ai-brains-graph` dep on density path |
| **AC9** Feature-off `graph *` exit 2 + FEATURE_UNAVAILABLE | **PASS** | existing smoke (unchanged surface); T232 does not touch stubs |
| **AC10** Docs OPERATIONS ~717–736 capability-aware; hybrid line retired; CAPABILITIES; CHANGELOG | **PASS** | OPERATIONS “When to rebuild (T232)” table on/off; hybrid empty-lag line gone; CAPABILITIES `graph_density` capability-aware sentence; CHANGELOG Unreleased **Changed** T232 entry |
| **AC11** Soft skill one-liner | **PASS** (soft) | `.agents/skills/ai-brains/SKILL.md` rebuild only on graph-capable binary; else reinstall SOOT / `graph_feature` |
| **AC12** Full gate green | **PROCESS** | Plan claims graph-off nextest 771 + graph-on 38 + clippy package; full workspace gate / `ledgerful verify` still orchestrator checklist (not a code defect) |
| **AC13** Manual dual-build dogfood | **PROCESS** | Plan open: orchestrator manual sparse → rebuild; optional feature-off |
| **AC14** No clap/rusqlite bump; zero new crates | **PASS** | workspace pins clap **4.5**, rusqlite **0.39.0**; cli `Cargo.toml` features `default=[]`; no new deps on density surface |
| **AC15** Claims: no auto-remediation; no “always rebuild” | **PASS** | CHANGELOG + OPERATIONS state capability-aware next action; no auto-rebuild language |
| **AC16** Single reinstall SOOT | **PASS** | `GRAPH_REINSTALL_SOOT` in `governed_common`; density references by name; smoke F17 forbids free-standing install literal in `graph_density.rs` |
| **AC17** H1: remediation tests use `_with(…, true\|false)` | **PASS** | All remediation/note asserts call `assess_graph_density_with`; production-only wrapper in `doctor.rs` / `graph.rs`; `graph_health_output__sparse_fixture` migrated to `_with(…, true)` (L3) |
| **AC18** Note templates F7 / M5 | **PASS** | graph-on empty/orphan/projection: `; run graph rebuild`; sparse: `rebuild if projection lag suspected`; graph-off uniform: `; see remediation to install a graph-capable binary`; Ok note capability-blind informational |
| **AC19** Smoke F17 extended | **PASS** | `graph_stub__reinstall_hint__matches_install_soot`: density names `GRAPH_REINSTALL_SOOT`; no install literal; doctor has no `"ai-brains graph rebuild"` + must call `density_remediation`; `REMEDIATION_REBUILD` value present |

## Findings

None open at critical / high / medium.

## Completeness

### Audit focus (1–10)

| # | Focus | Result |
|---|--------|--------|
| 1 | Every AC; capability remediations exact SOOT | Met (see matrix). `density_remediation(true\|false)` + warn arms use exact consts |
| 2 | No hybrid empty_lag left | Met. No `remediation_empty_lag` / hybrid parenthetical; empty_lag uses F4/F5 |
| 3 | Doctor gather-error → `density_remediation`; no hardcoded rebuild in `doctor.rs` | Met. Grep: zero `"ai-brains graph rebuild"` in `doctor.rs`; smoke guards same |
| 4 | H1: tests use `_with` explicit bool | Met. No remediation-asserting wrapper tests |
| 5 | Thresholds / priority / matrix 13 | Met. Floors frozen; priority units; doctor matrix 13 order `graph_feature` → `graph_density` → `harness_wiring` → `integrity` |
| 6 | Ok note capability-blind; skip `status=live` | Met. Ok arm fixed note; skip `status: "live"` |
| 7 | Smoke F17 extended | Met |
| 8 | Docs OPERATIONS / CAPABILITIES / CHANGELOG honest | Met for AC10 hard surface; secondary ops quick-table residual L2 |
| 9 | No `unwrap`/`expect` in production T232 paths; no dep bumps | Met. `unwrap`/`expect` only in `#[cfg(test)]`; `unwrap_or` in gather helpers pre-existing. No pin bumps observed on workspace clap/rusqlite |
| 10 | Placeholders / TODO stubs incomplete wiring | Met. No TODO/FIXME/`todo!` on T232 surface; wiring complete |

### Key SOOT / API shape (production)

```139:161:crates/ai-brains-cli/src/graph_density.rs
/// Graph-on primary remediation SOOT for density warn paths (T232 F4).
pub(crate) const REMEDIATION_REBUILD: &str = "ai-brains graph rebuild";

/// Capability-aware primary remediation (warn paths + doctor gather-error).
pub(crate) fn density_remediation(graph_cli_available: bool) -> &'static str {
    if graph_cli_available {
        REMEDIATION_REBUILD
    } else {
        crate::commands::governed_common::GRAPH_REINSTALL_SOOT
    }
}

fn density_warn_note(message: &str, graph_cli_available: bool, sparse_nuance: bool) -> String {
    // … F7 templates …
}
```

```698:707:crates/ai-brains-cli/src/commands/doctor.rs
    let gather = match gather_density_snapshot(&conn) {
        Ok(g) => g,
        Err(e) => {
            return HealthCheck::warn(
                "graph_density",
                format!("graph count query failed: {e}"),
                Some(crate::graph_density::density_remediation(cfg!(feature = "graph")).into()),
            );
        }
    };
```

## Residual notes

Process / low-info only (do **not** block CLEAN; orchestrator ship hygiene):

| ID | Severity | Note |
|----|----------|------|
| **L1** | low/info | AC5 says Skip/Ok “both capability sides”; units only pass `true` for skip/ok. Arms are capability-blind (`remediation: None`); optional `_with(…, false)` cases would close the letter of AC5 without behavior change. |
| **L2** | low/info | `Docs/OPERATIONS.md` ~826 quick-ref still says “rebuild if sparse/empty” without capability caveat. Hard AC10 surface (~717–736) is correct; secondary table polish optional. |
| **L3** | process | Full workspace gate, `ledgerful verify`, manual dogfood AC13, ledger commit, deferred.md strike (“Doctor graph rebuild vs graph-off”), conductor Completed — still open on plan Phase 0/5; not implementation defects. |
| **L4** | process | `spec.md` header still “Planning (plan-only until go)” while `plan.md` / `conductor.md` say Implementing — reconcile status on ship. |
| **L5** | info | O2 soft cfg-not-graph doctor gather integration intentionally skipped; pure `density_remediation` units cover SOOT matrix. |

## Residual notes (non-findings affirm)

- Capture independence preserved (rusqlite COUNT only; no graph crate on density).
- No auto-rebuild; contracts DTO unchanged; Cargo `default = []` unchanged.
- Graph update remains `#[cfg(feature = "graph")]` and uses convenience wrapper → rebuild (F13).
- AI2 reject honored: Skip stays `status=live` + `density=skip`.
