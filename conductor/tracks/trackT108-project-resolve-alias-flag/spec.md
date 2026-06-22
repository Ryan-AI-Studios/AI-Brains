# Track T108: `project resolve --alias` Flag

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — UX papercut; positional arg is unintuitive.
**Source:** Systematic command test 2026-06-22.

---

## Problem Statement

`ai-brains project resolve` takes a positional `<ALIAS>` argument. Users intuitively try `--alias <name>` first and get a clap error: `unexpected argument '--alias' found`. The help text doesn't make the positional usage obvious enough. This is a minor UX papercut but easy to fix.

## Acceptance Criteria

**AC1:** `ai-brains project resolve --alias <name>` works identically to `ai-brains project resolve <name>`.

**AC2:** The positional `<ALIAS>` argument continues to work (backward compatibility).

**AC3:** Both forms are documented in the `--help` output.

**AC4:** If both `--alias` and a positional arg are provided, `--alias` wins (or clap rejects the conflict — either is acceptable as long as it's not ambiguous).

## Design Notes

- Use clap's declarative conflict resolution: add `#[arg(long = "alias", conflicts_with = "alias_positional")]` as a separate field. Clap auto-generates correct help text and handles the XOR validation declaratively — no manual `Some` matching needed.
- The positional arg field name should be `alias_positional` (or similar) so `conflicts_with` references it clearly.
- In the handler, use whichever field is `Some` — since clap guarantees only one is set via `conflicts_with`, this is safe.
- Minimal code change: ~5 lines in main.rs + ~3 lines in project.rs.

## Files

- `crates/ai-brains-cli/src/main.rs` — add `--alias` option to `ProjectCommands::Resolve`.
- `crates/ai-brains-cli/src/commands/project.rs` — update `resolve` to accept both positional and `--alias`.

## Tests (TDD)

**Red:** `project_resolve__alias_flag__returns_correct_id` — run `project resolve --alias test-alias`, assert it returns the correct project ID.

**Green:** Add `--alias` flag. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains project resolve --alias test-alias` → returns UUID.
- Manual: `ai-brains project resolve test-alias` → returns UUID (backward compat).

## Out of Scope

- Adding `--alias` to other subcommands.
- Changing the positional arg name.
