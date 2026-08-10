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
| Internal R1 | subagent | **CLEAN** |
| Codex R1 | gpt-5.4 high | **FAIL** — P2 governance + P3 AC5 dual-side |
| Codex R1 fix | orchestrator | dual Skip/Ok units; In review governance |
| Codex R2 | gpt-5.4 high | **PASS WITH DEFERRED P3** (process closeout only) |
| Final Codex | pending closeout PR | fresh clean gate |

## AC matrix

| AC | Status |
|----|--------|
| AC1–19 engineering | **Met** |
| Process closeout | **Met** on this closeout PR |

## Findings disposition

| ID | Sev | Status |
|----|-----|--------|
| Codex R1 P2 | P2 | **verified_fixed** |
| Codex R1 P3 | P3 | **verified_fixed** |
| Codex R2 process | P3 | **verified_fixed** (this closeout) |

## Deferrals

None open. Out-of-scope residuals remain product non-goals: auto rebuild, threshold retune, Cargo default/release graph-on, rusqlite 0.40, event freshness F31.

## Gates

| Gate | Result |
|------|--------|
| Local full gate | **2497** nextest + fmt/clippy/deny/audit + ledgerful verify full |
| Manual graph-on doctor sparse | remediation = `ai-brains graph rebuild` |
| CI PR #124 | Win/Linux/macOS **SUCCESS** |
| Squash merge | `33b28d0` |
| Internal CLEAN | yes |
| Codex R2 | PASS WITH DEFERRED P3 → closeout clears process residual |

## Completion decision

Engineering DoD met; PR #124 merged; governance Completed + deferred strike on closeout. Track **Completed**.
