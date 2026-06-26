# Track T139: Project Env Precedence and Preflight Scope

**Status:** Complete
**Owner:** Codex
**Priority:** P0 - prevents silent memory-context contamination across repos.
**Category:** BUGFIX
**Source:** ledgerful-web bootstrap feedback, 2026-06-25.

## Problem Statement

`ai-brains preflight --summary` can silently scope to inherited
`AI_BRAINS_PROJECT_ID` / `AI_BRAINS_SESSION_ID` values even when the current
repo has a local `.env` with different project context. That makes interactive
repo-scoped commands report or write memories against the wrong project.

General runtime configuration should still follow T113 precedence:
CLI flags > shell env vars > project `.env` > global `.env`.

Project-context identity is different: when a current repo has `.env`
containing `AI_BRAINS_PROJECT_ID` or `AI_BRAINS_SESSION_ID`, those local values
must win for default interactive repo-scoped commands, and disagreements with
inherited shell values must be visible.

## Acceptance Criteria

- AC1: With local `.env` IDs and different inherited shell IDs, `preflight`
  reports the local project ID.
- AC2: Context-sensitive CLI commands emit a clear warning when local `.env`
  overrides inherited project/session IDs.
- AC3: `--no-project-context` remains an escape hatch that preserves caller
  supplied environment values.
- AC4: Non-context runtime settings, including model URLs and vault settings,
  retain the T113 shell-env precedence behavior.

## Implementation Notes

- Modify `crates/ai-brains-cli/src/main.rs`.
- Load project `.env` normally for non-context keys, then explicitly apply only
  `AI_BRAINS_PROJECT_ID` and `AI_BRAINS_SESSION_ID` from local `.env`.
- Do not apply this override when `--no-project-context` is present.
- Add a smoke test in `crates/ai-brains-cli/tests/smoke.rs`.

## Verification

- `cargo nextest run -p ai-brains-cli preflight__local_env_project_context_overrides_inherited_shell_ids`
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- Manual: run `ai-brains preflight --summary` in a repo with `.env` while stale shell IDs are set.
