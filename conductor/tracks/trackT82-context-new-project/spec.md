# Track T82: honor `context --new-project`

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P1 — flag was silently ignored.

---

## Problem Statement

`ai-brains context --new-project` was parsed by clap and forwarded to
`context::run` as a `bool`, but the implementation at
`crates/ai-brains-cli/src/commands/context.rs:82-94` (pre-T82) returned
early with `"Context is already initialized for project: …"` whenever a
session was already in `.env`, regardless of the `--new-project` flag.

The audit (June 2026) reproduced: a user running `context`, then
`context --new-project`, got the same `Context is already initialized`
message and the same `Project ID` in `.env` — the rotation never
happened.

## Acceptance Criteria

**AC1:** `ai-brains context --new-project` with an existing `.env`
rotates the `AI_BRAINS_PROJECT_ID` to a fresh UUID. The new ID is
written to `.env` and a new `ProjectRegistered` event is appended.

**AC2:** A clear log line is emitted: "Rotating project ID from
<old> to fresh UUID." (or "Rotating to fresh project ID." when no old
ID was recorded).

**AC3:** Default behavior is unchanged: `context` without `--new-project`
still returns early with the "already initialized" message.

**AC4:** The pre-existing `test_cli_context_idempotency` test (which
exercises `--new-session`) continues to pass.

## Design Notes

- The early-return at line 82-94 of `context.rs` is gated on
  `!new_session && !new_project`. When either flag is set, we fall
  through to the fresh-init path.
- The "Replacing existing session" log is also printed when
  `--new-project` is set, because a project rotation implicitly rotates
  the session too (the new project's session is fresh).
- The fresh `project_id` is already produced at line 45-46
  (`if new_project { ProjectId::new() }`), so the rest of the function
  naturally writes the rotated value to `.env`.

## Files

- `crates/ai-brains-cli/src/commands/context.rs` — gated the early-return
  on `!new_session && !new_project`, added the rotation log.
- `crates/ai-brains-cli/tests/smoke.rs` — `test_context_new_project_rotates_id`.

## Tests (TDD)

Red phase: `test_context_new_project_rotates_id` runs `context` twice
(once to set up, once with `--new-project`) and asserts the
`AI_BRAINS_PROJECT_ID` line in `.env` differs between the two runs.
Fails because the early-return ignored the flag.

Green phase: gate updated, rotation log added. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli test_context_new_project_rotates_id`
  — passes.
- `cargo nextest run -p ai-brains-cli test_cli_context_idempotency`
  — pre-existing test still passes.
- Manual: `ai-brains context` → `ai-brains context --new-project` in a
  project directory rotates the project_id.

## Out of Scope

- A "rename" command for the project alias (separate from the UUID).
  `--new-project` rotates the UUID; the alias (if any) is preserved.
- A `--tx-id` change tied to `--new-project` (the two are independent).
