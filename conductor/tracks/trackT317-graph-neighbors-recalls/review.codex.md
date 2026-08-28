# T317-GraphNeighborsRecalls Completion Audit

## Verdict

**NO-GO — implementation appears functionally complete, but track completion is not proven.**

The core behavior is correctly wired and targeted evidence is favorable. Mandatory closure work remains outstanding.

## Scope

Audited `track/T317-graph-neighbors-recalls` against `origin/main` (`dae7df3`), including:

- Full `spec.md` and `plan.md`
- Commits `0d1c787`, `a1eb373`, and `c3b6dfd`
- Product diff, tests, docs, help text, conductor records, and deferred records
- Current dependency pins and relevant live source

Product changes are confined to `graph.rs`, `main.rs`, and `graph_human_cli.rs`; no projector, query, contract, daemon, or hotspot files changed.

## DoD Matrix

| Requirement | Result | Evidence |
|---|---|---|
| AC1: 11 RECALLS → 3 kept, 8 hidden | Pass | `graph.rs:1433` |
| AC2: rstest boundary cases | Pass | `graph.rs:1451` |
| AC3: mixed labels retain all non-RECALLS | Pass | `graph.rs:1467` |
| AC4: T293 authority ordering preserved | Pass | `graph.rs:1506` |
| AC5: truthful header and RECALLS footer | Pass | `graph.rs:1525` |
| AC6: no footer when none hidden | Pass | `graph.rs:1548` |
| AC7: two-line hierarchy leaf | Pass | `graph.rs:1557` |
| AC8: no graph update/rebuild remediator | Pass | `graph.rs:1359` |
| AC9: JSON remains uncapped | **Partial proof** | Implementation is correct; test only asserts `>=4` despite seeding 5 |
| AC10: documentation/help updates | Pass | CAPABILITIES, PROTOCOL-COMPAT, OPERATIONS, CHANGELOG, `main.rs:3228` |
| AC11: manual neighbors output | Reported pass | `review.md:45–51`, N=12 and `+9` |
| AC12: manual hierarchy output | Reported pass | `review.md:53–58` |
| AC13: scoped diff/no projector/query changes | Pass | `git diff --name-status` |
| AC14: hermetic human cap | Pass | `graph_human_cli.rs:773` |
| AC15: T262 RECALLS behavior | Pass per targeted evidence | Review log/user-provided gate |
| AC16: feature-off exit 2 | Pass per targeted evidence | Existing feature-off test |
| AC17: limit footer precedes RECALLS footer | Pass | `graph.rs:1567` |
| Closure, full gate, ledger verification, publish | **Fail** | Plan remains unchecked; registry is `In Progress`; gates are pending |

## Findings

### P0-001 — Mandatory completion gates are outstanding

The implementation branch has not met the track’s closure Definition of Done.

Evidence:

- Every execution and DoD checkbox remains unchecked in [plan.md](/C:/dev/AI-Brains/conductor/tracks/trackT317-graph-neighbors-recalls/plan.md:118).
- Registry remains `In Progress` in [conductor.md](/C:/dev/AI-Brains/conductor/conductor.md:264).
- [review.md](/C:/dev/AI-Brains/conductor/tracks/trackT317-graph-neighbors-recalls/review.md:61) says cross-model review is pending.
- The same review log says the full `dev-check.ps1` and `ledgerful verify --scope full` are pending at line 65.
- No `review.codex.md` exists.
- The branch is five commits ahead of `origin/main`; publish/CI/merge hygiene is not complete.

Required before clearance: complete cross-model review, full gate, `ledgerful verify --scope full`, exact manual evidence/determinism recording, plan/conductor closure updates, and the required PR/CI/merge workflow.

### P2-001 — AC9 does not fully prove JSON remains uncapped

The new test seeds five RECALLS edges but asserts only:

- `neighbors.len() >= 4`
- RECALLS count `>= 4`

at [graph_human_cli.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/tests/graph_human_cli.rs:850).

A regression that capped JSON at four would pass. The test should assert exactly five neighbors and exactly five RECALLS, or seed exactly four and assert exact cardinality.

### P3-001 — Introduced diff-hygiene errors

`git diff --check origin/main...HEAD` reports:

- Trailing whitespace in [review.md](/C:/dev/AI-Brains/conductor/tracks/trackT317-graph-neighbors-recalls/review.md:3)
- A new blank line at EOF in [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT317-graph-neighbors-recalls/spec.md:409)

These are easy fixes and should not be deferred.

### P3-002 — New tests violate the repository’s no-loop test convention

New T317 tests use `for i in 0..5` in:

- [graph.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/graph.rs:1484)
- [graph_human_cli.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/tests/graph_human_cli.rs:780)
- [graph_human_cli.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/tests/graph_human_cli.rs:827)

`AGENTS.md` requires `rstest` parameterization instead of loops inside tests. Fix before closure; do not add this to `deferred.md`.

## Completeness

The product implementation contains no remaining T317 production stub or placeholder.

Implemented correctly:

- `RECALLS_PRETTY_CAP = 3`
- Stable cap preserving all non-RECALLS rows
- Cap applied after T293 preference ordering
- Truthful pre-cap header cardinality
- Separate limit and RECALLS footers in the required order
- JSON path remains uncapped except for explicit `--limit`
- Exact hierarchy leaf next-step text
- No new events, models, embeddings, projector changes, or graph-query changes
- No new CLI flags or JSON fields
- No production `unwrap()`, `expect()`, or `panic()` introduced

## Wiring

The production path is reachable end-to-end:

```text
GraphCommands::Neighbors
  → neighbors()
  → get_neighbors()
  → sort_neighbor_hits()
  → pretty_neighbor_rows()
  → prefer_authority_neighbor_rows()
  → cap_recalls_pretty_rows()
  → format_neighbors_pretty()
```

The JSON branch remains separate and uses the original sorted `NeighborHit` list. Hierarchy leaf handling calls `pretty_hierarchy_leaf()` only for an empty synthesis hierarchy.

The current implementation matches the intended design in [graph.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/graph.rs:554).

## Evidence

- `cargo fmt --all -- --check`: pass.
- Targeted nextest and graph-feature clippy: reported pass by the user and review log.
- Manual AC11/AC12/JSON: reported pass with live N=12.
- `git status`: clean.
- Current pins match the plan: clap lock `4.6.1`, serde_json `1.0.150`, rusqlite `0.40.2`, uuid `1.23.1`, Rust edition 2024.
- Current clap documentation confirms `after_help` is additive help text: [clap Command documentation](https://docs.rs/clap/latest/clap/struct.Command.html).
- CLI guidance supports human-output evolution while preserving structured JSON for scripts: [CLI Guidelines output guidance](https://clig.dev/#output).
- `ai-brains preflight --summary`: unavailable because `AI_BRAINS_KEY` is missing.
- ledgerful doctor/status/index: unavailable because the ledgerful database/lockfile could not be opened in this read-only environment.
- `gh` PR/comment recheck: unavailable because GitHub CLI configuration access was denied. The plan records PR `#233` as having no comments and `#230` as already routed to T325.

## Deferred Candidates

No new deferred candidates are recommended.

Existing documented residuals—PATH installation, live cardinality variability, sparse graph floors, retained JSON rows, PREVIEW content, and T325 recency work—are explicitly outside this track’s DoD and already recorded.

The AC9 test weakness, test-convention violations, whitespace errors, and completion gates should be fixed or completed, not deferred.

## Completion Decision

**Do not mark T317 Completed or merge yet.**

Clearance requires:

1. Strengthen AC9 to assert exact uncapped cardinality.
2. Fix introduced whitespace and test-convention violations.
3. Complete cross-model review and add its result.
4. Run and record the full CI gate and `ledgerful verify --scope full`.
5. Record exact manual commands/output and determinism evidence.
6. Mark the plan and conductor entry complete only after PR CI, squash merge, and branch hygiene.