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
| Internal R1 | subagent | **CLEAN** (0 critical/high/medium) |
| Codex R1 | gpt-5.4 high | **FAIL** — P2 governance lag + P3 AC5 dual-side Skip/Ok |
| Codex R1 fix | orchestrator | P3 dual-capability Skip/Ok tests; governance In review / PR #124 |
| Codex R2 | pending | re-review after fix push |

## AC matrix (engineering)

| AC | Status | Evidence |
|----|--------|----------|
| AC1–6, AC17–18 | Met | Dual-capability units in `graph_density.rs` (AC5 both sides after R1 fix) |
| AC7 | Met | `doctor.rs` gather-error → `density_remediation(cfg!(…))` |
| AC8–9 | Met | Existing feature-off / smoke |
| AC10–11 | Met | OPERATIONS / CAPABILITIES / CHANGELOG / skill one-liner |
| AC12 | Met (local) | Full gate 2497 + ledgerful verify full |
| AC13 | Met (graph-on) | Manual: sparse → `ai-brains graph rebuild` |
| AC14 | Met | No lockfile/dep bump |
| AC15–16, AC19 | Met | Review + smoke F17 extension |

## Findings disposition

| ID | Sev | Status | Disposition |
|----|-----|--------|-------------|
| Codex R1 P2 | P2 | fixed_pending_verification | Governance reconciled to **In review** + PR #124; deferred strike deferred to post-merge closeout (standard series pattern) |
| Codex R1 P3 | P3 | fixed_pending_verification | Skip/Ok units loop `true|false` |

## Deferrals

None product-surface. Post-merge closeout will strike deferred row + mark conductor Completed (same pattern as T222 #122/#123).

## Gates (observed)

| Gate | Result |
|------|--------|
| `cargo fmt --check` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo nextest run --workspace` | **2497** passed |
| `cargo deny check` / `cargo audit` | pass |
| `ledgerful verify --scope full` | pass |
| Manual graph-on doctor sparse | remediation = `ai-brains graph rebuild` |
| CI PR #124 | pending |
