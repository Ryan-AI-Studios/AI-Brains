# T227 Review Log — Briefing format honesty + substance

## Scope
- Format honesty: aliases → markdown; unknown → exit 2; trim+lower
- Empty honesty: empty_authority / empty_continuity only when `!denied`
- Denied bootstrap next-step; personal Denied spacing
- AC6 granted substance (System reads + decision + conclusion)
- Preflight accepts shared-renderer flow-through
- Docs: CAPABILITIES, OPERATIONS, CLI-EXIT-CODES, CHANGELOG, PROTOCOL-COMPAT soft

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| R1 | Internal (subagent) | **CLEAN** | P0–P2 none; 3 optional P3 test hygiene |
| R1 fix | Orchestrator | fixed | Strict AC4 stdout; denied JSON no empty_*; preflight bootstrap token |
| R2 | Internal recheck | **CLEAN** (targeted tests green) | P3 closed as verified via nextest |
| Cross-model R1 | Codex gpt-5.4 high | **PASS (product)** | No product P0–P3; process P2 = closeout pending (now closed) |
| Final Codex | Codex gpt-5.4 high | **PASS** | Fresh final gate after Completed closeout; no P0–P3; process P2 resolved; F34 residual intentional |

## Findings

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| IR1 | P3 | **verified_fixed** | AC4 banana stdout strict empty |
| IR2 | P3 | **verified_fixed** | Soft-deny JSON asserts no empty_* kinds |
| IR3 | P3 | **verified_fixed** | Preflight governed deny asserts `policy bootstrap` |
| CX1 | P2 | **verified_fixed** (process) | Conductor/deferred/series Completed on closeout |

## Residual (intentional / soft)
- F34: `OutputFormat::parse` silent-JSON surface-wide — not T227 DoD
- #18 continuity synthesis; typed constraints; ValueEnum; `--quiet` footers

## Gate evidence
```
# Targeted
cargo nextest run -p ai-brains-cli --test briefing_format_substance  # 7/7
cargo nextest run -p ai-brains-retrieval --test preflight_governed_flag  # 5/5

# Full local gate (2026-08-11)
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace   # 2548 passed
cargo deny check
cargo audit
ledgerful verify --scope full   # Verification passed

# CI (PR #132)
gate-windows / gate-linux / gate-macos  # all pass
squash-merged 40c7cd1
```

## Manual dogfood
```
ai-brains briefing project --help   # lists human,pretty,text,markdown,md,json + human example
ai-brains briefing project --format banana …  # exit 2; stderr accepted list; zero product stdout
```

## Completion decision
Engineering DoD met. Product shipped PR #132. Governance closeout marks Completed + deferred strike + coordinated.
