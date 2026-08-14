# Track Re-Review — T248 (internal r2)

**Track:** T248-RetentionPlanHuman
**Category:** UX / FEATURE
**Reviewer:** Grok (read-only)
**Date:** 2026-08-14
**Spec:** `conductor/tracks/trackT248-retention-plan-human/spec.md`
**Scope:** Re-review after CX1 P3-1 fix. Product commit `920c78c` + test commit `d45a319` (`test(cli): T248 apply --format auto stays JSON`) vs `origin/main`.
**Primary files:** `crates/ai-brains-cli/src/commands/retention.rs`, `crates/ai-brains-cli/src/main.rs`, `crates/ai-brains-cli/tests/retention_plan_human.rs`

Static review only. Production code and Git were not modified. Nextest / clippy / live TTY were **not** re-executed. F13 honored: no live `retention apply --confirm`.

## Verdict: PASS

CX1 P3-1 is addressed. `retention_apply__confirm_format_auto__pretty_json_not_human` exists, is hermetic, and asserts `apply --confirm --format auto` emits pretty JSON (`api_version` / `mode=apply`) and does **not** contain `Retention apply`. No new P0–P2 on the product surface versus `origin/main`.

## Checks

| # | Check | Result |
|---|--------|--------|
| 1 | New hermetic `retention_apply__confirm_format_auto__pretty_json_not_human` | **Present.** [`retention_plan_human.rs:246`](C:\dev\AI-Brains\crates\ai-brains-cli\tests\retention_plan_human.rs) |
| 2 | Asserts apply `--format auto` pretty JSON (`api_version` / `mode=apply`) | **Met.** Parses via `parse_pretty_json_object`; `api_version == "1"`; `mode == "apply"`; exit 0 |
| 3 | Asserts stdout does **not** contain `Retention apply` | **Met.** Explicit `!stdout.contains("Retention apply")` |
| 4 | No new P0–P2 on product vs `origin/main` | **Clean.** Product remains `920c78c`; `d45a319` is test-only |
| 5 | CX1 P3-1 addressed | **Closed.** End-to-end apply `--format auto` now fails if the CLI TTY-switches to human |

## CX1 P3-1 disposition

CX1 asked for a focused regression: invoke `retention apply --confirm --format auto` and verify JSON, because resolver units + hermetic `json`/`human` left the `run_apply(..., false)` call site unguarded.

The new test:

- Isolates `AI_BRAINS_RETENTION_*` via `TempEnv` (`isolate_retention_env`).
- Uses `tempdir` + `init_vault` (same empty-fixture pattern as AC12; not a live vault).
- Runs `hermetic_bin` with `--no-project-context --vault-path … retention apply --confirm --format auto`.
- Requires exit 0.
- Rejects the apply human title (`Retention apply`).
- Parses pretty JSON and locks `api_version="1"` and `mode="apply"`.

Production wiring is unchanged: [`retention.rs:81`](C:\dev\AI-Brains\crates\ai-brains-cli\src\commands\retention.rs) still calls `resolve_retention_format(&options.format, false)`. A future flip of that `false` to `stdout().is_terminal()` would not by itself fail this hermetic (`.output()` pipes stdout), but flipping `auto` → human, or routing apply auto through `format_retention_pretty`, **would** fail the title / JSON asserts. Combined with `resolve_retention_format__apply_auto_even_on_tty__json`, F4 is now locked at both helper and CLI.

## Product vs origin/main (no new P0–P2)

Re-read of the T248 product surface (`retention.rs` resolver + pretty + apply gates, clap Plan/Apply, docs, isolation):

- Resolver fail-closed; no `OutputFormat::parse` on this command.
- Apply `is_tty: false`; clap Apply default `json`; Plan default `auto`.
- Pretty F2 order / exact Totals / HORIZON 36 / sample `", "` / F10 shorts / `next:` last / `memory_legacy` zero-row `skip`.
- JSON still `emit_json` → `to_string_pretty`; keys frozen; no DTO / planner / nightly rewrite.
- Apply refuse still `INVALID_PAYLOAD` exit 6.
- No `unwrap`/`expect` in production `retention.rs`.
- `d45a319` does not change product behavior.

CX1 P1-1 / P2-1 / P2-2 (full gate evidence, conductor finalization, ignored `.agents/` skill) are **process / completion** items from the independent completion audit. They are not new product defects on this re-review and are not re-filed here.

## P0

None.

## P1

None.

## P2

None.

## P3

None. CX1 P3-1 is **verified_fixed** at source (test present and asserts the requested contract). Residual that a piped hermetic cannot distinguish `is_tty: false` from `stdout().is_terminal()` is inherent to `.output()` and is not a new finding.

## Not findings

- This pass did not re-run nextest, clippy, or live TTY/pipe (AC13). Static match against the requested test contract is sufficient for r2.
- Hermetic apply `--confirm` is empty-fixture only (F13). Same isolation as AC12.
- Test does not re-assert `totals.candidates == 0` (sibling json test already does). CX1 asked for JSON + not-human.
- Pre-existing CX1 process findings (gate evidence, review.md / Planning status, gitignored skill) are out of this r2 product-fix scope.
