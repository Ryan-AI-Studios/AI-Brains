# Track T122: `graph` Subcommand Hint Stub

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — discoverability; users with default install can't find graph features.
**Source:** v0.1.1 verification friction #4.

---

## Problem Statement

The `graph` subcommand (`graph update`, `graph neighbors`, etc.) is feature-gated behind `--features graph`. With the default install (`cargo install --path crates/ai-brains-cli --locked`), the subcommand doesn't exist:

```
$ ai-brains graph update
error: unrecognized subcommand 'graph'
```

Users have no way to know the feature exists. The `--help` output doesn't mention it. This is a discoverability problem — the graph feature is documented in the onboarding skill and OPERATIONS.md but invisible from the CLI itself.

## Acceptance Criteria

**AC1:** When built WITHOUT the `graph` feature, `ai-brains graph --help` prints a hint message: `The graph subcommand requires a --features graph build. Reinstall with: cargo install --path crates/ai-brains-cli --locked --features graph`

**AC2:** When built WITHOUT the `graph` feature, `ai-brains graph <anything>` prints the same hint and exits 0 (not an error — the user didn't do anything wrong, they just need a different build).

**AC3:** When built WITH the `graph` feature, `graph` works exactly as before — no change in behavior or help output.

**AC4:** The `ai-brains --help` output mentions `graph` in the commands list with a `(requires --features graph)` annotation when the feature is not enabled.

## Design Notes

- **File:** `crates/ai-brains-cli/src/main.rs` — Add a `Graph` variant to the `Commands` enum that is always present (not `#[cfg(feature = "graph")]`).
- When `graph` feature is NOT enabled: the `Graph` match arm prints the hint message and returns `Ok(())`.
- When `graph` feature IS enabled: the existing `#[cfg(feature = "graph")]` graph command handlers run.
- Use `#[cfg(not(feature = "graph"))]` for the stub and `#[cfg(feature = "graph")]` for the real implementation.
- The `Graph` subcommand should accept any trailing args (catch-all) so `graph update`, `graph neighbors <id>`, etc. all hit the stub.

## Files

- `crates/ai-brains-cli/src/main.rs` — Add unconditional `Graph` command variant with cfg-gated dispatch.

## Tests (TDD)

**Red:** `graph__default_build__prints_hint` — run `ai-brains graph update` with the default (no graph feature) build, assert stdout contains "requires a --features graph build" and exit 0.

**Green:** Add the stub. Test passes. (Test only runs in default build; the graph-feature build has the real implementation.)

## Verification

- `cargo nextest run -p ai-brains-cli` (default build, no graph feature)
- Manual: `ai-brains graph --help` → shows hint.
- Manual: `ai-brains graph update` → prints hint, exits 0.

## Out of Scope

- Making the graph feature default-on (that's a larger decision about binary size and deps).
- Adding stubs for other feature-gated functionality.
- Changing the graph command's actual behavior when the feature is enabled.