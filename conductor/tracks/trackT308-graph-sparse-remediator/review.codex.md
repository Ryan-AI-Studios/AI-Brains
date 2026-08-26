# Track Completion Audit — T308-GraphSparseRemediator

## Verdict: PASS WITH DEFERRED P3

Cross-model DoD audit against live src + orchestrator-observed gates. No product DoD gap in source. Soft residuals only.

## Scope Reviewed

- `conductor/tracks/trackT308-graph-sparse-remediator/spec.md` + `plan.md`
- `crates/ai-brains-cli/src/graph_density.rs` Sparse arm + unit flips
- `crates/ai-brains-cli/src/commands/graph.rs` sparse JSON omit test (`--features graph`)
- `crates/ai-brains-cli/src/commands/doctor.rs` forward + matrix 15 (no edit)
- `Docs/OPERATIONS.md`, `Docs/CAPABILITIES.md`, `CHANGELOG.md` Unreleased
- Confirmed `has_graph_tables` still sqlite_master (T309 not stolen)

## Requirement and DoD Matrix

| AC | Result | Evidence |
|----|--------|----------|
| AC1 | Met | `…__no_rebuild_remediator` + `…ratio_0_4` assert `remediation.is_none()` + lag note |
| AC2 | Met | empty_lag / orphan / projection_lag rebuild units PASS |
| AC3 | Met | graph-off Sparse reinstall SOOT PASS |
| AC4 | Met | `…__omits_remediation` PASS with `--features graph`; JSON key absent |
| AC5 | Met | `MIN_EDGE_NODE_RATIO = 0.50`; smoke F17 PASS |
| AC6 | Met | doctor `:914` still `assessment.remediation`; matrix 15 |
| AC7 | Met | clippy `-p ai-brains-cli` exit 0; targeted nextest green |
| AC8 | Met | OPERATIONS when-to-rebuild table; CAPABILITIES graph_density; CHANGELOG T308 |
| AC9 | Met | `cargo run --features graph -- doctor --format json` Sparse warn, **no remediation key** |
| AC10 | Met | `has_graph_tables` unchanged |
| F1 floors | Met | 0.50 frozen |
| F2 None | Met | Sparse graph-on `None` |
| F7 doctor | Met | no doctor.rs edit |
| F9 T309 | Met | not stolen |

## Findings

None P0–P2.

| ID | Sev | Title | Deferrable |
|----|-----|-------|------------|
| P3-1 | P3 | Live E/N still ~0.41 (honest sparse; floors frozen) | Yes — by design F1 |
| P3-2 | P3 | Never-rebuilt Sparse has no rebuild remediator | Yes — by design F2; empty/orphan still rebuild |
| P3-3 | P3 | PATH binary still emits rebuild remediator until cargo install | Yes — soft; hermetic SoT; F12 |

## Completeness Sweep

No TODO/FIXME/stub in Sparse arm. Remediator omit is intentional `None`, not a placeholder. Emitter already `if let Some` / `skip_serializing_if`. No fake live status.

## Wiring and Regression Review

Assessor → doctor warn forward → contracts optional remediator → graph health JSON omit. Lag arms still call `density_remediation`. Graph-off Sparse still reinstall. Priority order unchanged. Capture-independent (rusqlite COUNT + text).

## Verification Evidence

**Observed by orchestrator:**

- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` exit 0
- Targeted nextest sparse/lag/F17 PASS
- `--features graph` sparse JSON omit PASS
- AC9 cargo-run doctor: severity warn, message sparse E/N=0.410, **no remediation property**
- First full `dev-check.ps1` aborted mid-suite (timeouts under parallel codex/`ledgerful verify` contention) — **unrelated**; re-run alone in progress

**Not re-run inside this audit file:** workspace deny/audit (deferred to orchestrator full gate).

## Deferred Candidates

P3-1, P3-2, P3-3 — already mirrored in `conductor/deferred.md` T308 implement residuals R1–R3.

## Completion Decision

Engineering DoD for T308 is met in source + targeted tests + AC9. Mark **Completed** only after standalone full CI gate exit 0 and Phase 6 publish. Do not treat PATH stale remediator as a product fail.
