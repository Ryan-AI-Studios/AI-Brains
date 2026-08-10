# T232 Review Log — Graph density remediation path

## Scope

Capability-aware `graph_density` / assessor remediations:
- Graph-on → exact `REMEDIATION_REBUILD` (`ai-brains graph rebuild`)
- Graph-off → exact `GRAPH_REINSTALL_SOOT`
- Empty-lag hybrid retired; doctor gather-error uses `density_remediation`
- Thresholds / matrix 13 / soft severity / capture independence frozen

## Reviewers / rounds

| Round | Reviewer | Verdict |
|-------|----------|---------|
| Internal R1 | subagent | **CLEAN** (0 critical/high/medium; process lows only) |
| Codex | pending | — |

## AC matrix (engineering)

| AC | Status | Evidence |
|----|--------|----------|
| AC1–6, AC17–18 | Met | Dual-capability units in `graph_density.rs` |
| AC7 | Met | `doctor.rs` gather-error → `density_remediation(cfg!(…))` |
| AC8–9 | Met | Existing feature-off / smoke |
| AC10–11 | Met | OPERATIONS / CAPABILITIES / CHANGELOG / skill one-liner |
| AC12 | Process | Full gate (orchestrator) |
| AC13 | Met (graph-on) | Manual: sparse → `ai-brains graph rebuild` |
| AC14 | Met | No lockfile/dep bump |
| AC15–16, AC19 | Met | Review + smoke F17 extension |

## Findings

None open above low. Internal R1 residuals (process): full gate, deferred strike, conductor Completed, Codex optional-but-run per user process.

## Deferrals

None from T232 product surface. Residual out-of-scope stays: auto rebuild, threshold retune, Cargo default, rusqlite 0.40, event freshness F31.

## Gates (observed)

| Gate | Result |
|------|--------|
| `cargo fmt --check` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass (clip) |
| `cargo nextest run -p ai-brains-cli` graph-off | 771 passed (implementer) |
| Full workspace nextest / deny / audit | pending orchestrator |
| Manual graph-on doctor sparse | remediation = `ai-brains graph rebuild` |
