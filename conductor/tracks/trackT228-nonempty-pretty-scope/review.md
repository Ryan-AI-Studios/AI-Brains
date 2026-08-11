# T228 Review Log — Non-empty pretty Scope

## Scope
Pretty `Scope:` chrome on non-empty `recall` and `sync query` vault section via shared `resolve_active_scope_line` (F29). JSON frozen. Closes T207 AC10 residual.

## Reviewers / rounds
| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| R1 internal | explore subagent | **CLEAN** | AC1–AC13 Met; no P0–P3 fix-required |
| Completeness | explore subagent | **COMPLETE** | Product checklist 1–5 met |
| Codex R1 | gpt-5.4 high | **PASS WITH DEFERRED P3** | Product complete; process-only P3 (closeout) |
| Closeout | Orchestrator | **done** | conductor/deferred/series/plan Completed; product PR #134 `e51d5e4` |
| Codex final | gpt-5.4 high | **PASS** | Fresh final gate after Completed; CX1 closed; F32/F34 intentional only |

## DoD matrix
| AC | Status |
|----|--------|
| AC1–AC13 | **Met** |

## Findings disposition
| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| CX1 | P3 process | **verified_fixed** | Conductor/deferred/series Completed on closeout |

No product P0–P2. No product P3 fix-required.

## F29 helper signature (F35)
```rust
pub(crate) fn resolve_active_scope_line(
    conn: &impl QueryStore,
    global: bool,
    project_id: Option<&ProjectId>,
) -> Result<String, Box<dyn std::error::Error>>
```
Call sites: pretty recall (empty+non-empty shared resolve), `print_pretty_empty_sync`, sync non-empty vault.

## Residuals (soft, expected)
- **F32:** sync missing/invalid `AI_BRAINS_PROJECT_ID` → random UUID (documented CHANGELOG; not fixed)
- **F34:** sync TTY-independent pretty default (out of DoD)

## Gate evidence
```
cargo fmt --check                                          # clean
cargo clippy --workspace --all-targets -- -D warnings      # clean
cargo nextest run --workspace                              # 2551 passed (1 skipped)
cargo deny check                                           # ok
cargo audit                                                # 0 exit (allowed warnings only)
ledgerful verify --scope full                              # Verification passed
CI PR #134: gate-windows / gate-linux / gate-macos         # all SUCCESS
squash-merged e51d5e4
```

## Manual dogfood
```
ai-brains recall "DECISION" --limit 2 --format pretty --no-bridge
# Scope: project=test-alias (…) \n Session: … \n hits

ai-brains recall "DECISION" --limit 2 --global --format pretty --no-bridge
# Scope: global \n Session: … \n hits

ai-brains recall "zzzznomatchesT228xyz" --format pretty --no-bridge
# Scope: … \n No results… (empty unchanged)

ai-brains sync query "DECISION" --no-bridge --format pretty
# --- AI-Brains Recall --- \n Scope: … \n hits

ai-brains recall "DECISION" --limit 1 --format json --no-bridge
# no scope field on envelope
```

## Completion decision
Engineering DoD met. Product shipped PR #134 `e51d5e4`. Governance closeout marks Completed + deferred strike + series README + coordinated deferred note. Soft residuals F32/F34 only.
