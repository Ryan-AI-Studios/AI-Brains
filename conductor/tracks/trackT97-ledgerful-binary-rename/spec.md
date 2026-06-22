# Track T97: Migrate Shell-Out Calls from `changeguard` to `ledgerful`

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P2 — the `changeguard` binary alias may be removed in a future ChangeGuard release.
**Source:** `C:\dev\testing_report.md` §5 "Binary Rename"; confirmed both binaries present 2026-06-22.

---

## Problem Statement

As of ChangeGuard Track TA1 (2026-06-21), the ChangeGuard CLI binary was renamed from `changeguard` to `ledgerful`. The library crate is still `changeguard` (Rust imports unchanged), but the binary is `ledgerful.exe`. A `changeguard` alias currently exists (`C:\Users\RyanB\.cargo\bin\changeguard.exe` → `ledgerful 0.1.6`), but this alias may be removed in a future release.

AI-Brains shells out to `changeguard` in ~15 callsites across 7 crates:

- `crates/ai-brains-cli/src/commands/sync.rs` (lines 54, 348, 494)
- `crates/ai-brains-cli/src/commands/nightly.rs` (line 227)
- `crates/ai-brains-cli/src/commands/symbol_bridge.rs` (line 118)
- `crates/ai-brains-cli/src/commands/safety.rs` (lines 102, 130)
- `crates/ai-brains-retrieval/src/preflight.rs` (lines 284, 383)
- `crates/ai-brains-retrieval/src/recall.rs` (line 248)
- `crates/ai-brains-capture/src/verification_gate.rs` (line 173)
- `crates/ai-brains-brain/src/intervention.rs` (line 227)
- `crates/ai-brains-graph/src/cozo_proxy.rs` (lines 104, 157, 229)

All of these should call `ledgerful` instead. The `changeguard` alias is a grace period, not a permanent interface.

## Acceptance Criteria

**AC1:** Every `std::process::Command::new("changeguard")` call in AI-Brains source is replaced with `std::process::Command::new("ledgerful")`.

**AC2:** User-facing strings that mention `changeguard` CLI commands (e.g. error messages like `"changeguard CLI not available: {e}"`, `"changeguard bridge export failed: {stderr}"`) are updated to `ledgerful`.

**AC3:** The `.changeguard/` directory references (state dir discovery, path joins) are NOT renamed — that is a directory name, not a binary name, and remains `.changeguard/` in both tools. Only the binary invocation changes.

**AC4:** The `cozo_proxy.rs` availability check (`Command::new("changeguard").arg("--version")`) uses `ledgerful`.

**AC5:** Test files that check for `changeguard` binary availability (`cross_repo_bridge_smoke.rs:171`) are updated to try `ledgerful` first, falling back to `changeguard` for backward compatibility with older installs.

**AC6:** `AGENTS.md`, `.agents/skills/onboarding/SKILL.md`, and `.agents/skills/changeguard/SKILL.md` references to the `changeguard` CLI command are updated to use `ledgerful` (or `ledgerful`/`changeguard`/`ldg` — all valid aliases per the changeguard skill).

**AC7:** Full CI gate passes. No behavior change — the same commands run, just via the canonical binary name.

## Design Notes

- This is a mechanical find-and-replace of the binary name in `Command::new(...)` calls. The CLI args are unchanged (`ledgerful scan --impact` is identical to `changeguard scan --impact`).
- The `.changeguard/` directory name is NOT touched. That is a filesystem path convention shared with ChangeGuard's state, not a binary name.
- The `changeguard` skill name and its path (`.agents/skills/changeguard/`) are NOT renamed — the skill documents the tool, and the tool's library crate is still `changeguard`. Only the shell-out binary name changes.
- For test backward compatibility, tests that gate on binary availability should try `ledgerful` first and fall back to `changeguard` so they work on both old and new installs.

## Files

- `crates/ai-brains-cli/src/commands/sync.rs`
- `crates/ai-brains-cli/src/commands/nightly.rs`
- `crates/ai-brains-cli/src/commands/symbol_bridge.rs`
- `crates/ai-brains-cli/src/commands/safety.rs`
- `crates/ai-brains-retrieval/src/preflight.rs`
- `crates/ai-brains-retrieval/src/recall.rs`
- `crates/ai-brains-capture/src/verification_gate.rs`
- `crates/ai-brains-brain/src/intervention.rs`
- `crates/ai-brains-graph/src/cozo_proxy.rs`
- `crates/ai-brains-cli/tests/cross_repo_bridge_smoke.rs`
- `AGENTS.md` — update CLI command references.
- `.agents/skills/onboarding/SKILL.md` — update CLI command references.
- `.agents/skills/changeguard/SKILL.md` — note that `ledgerful` is the canonical binary name.

## Tests (TDD)

**Red:** `test_ledgerful_binary_preferred_over_changeguard_alias` — asserts that `Command::new("ledgerful").arg("--version")` succeeds. This already passes (the binary exists), but the test documents the expectation.

**Green:** All `Command::new("changeguard")` → `Command::new("ledgerful")`. Existing tests pass unchanged (they don't gate on binary name, they test AI-Brains behavior).

## Verification

- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Manual: `ai-brains recall "test"` — confirm bridge query still works (now via `ledgerful`).
- Manual: `ai-brains nightly` — confirm MADR + symbol ingestion still works.
- `grep -r "Command::new(\"changeguard\")" crates/` returns zero results.

## Out of Scope

- Renaming the `.changeguard/` state directory (not part of the binary rename).
- Renaming the `changeguard` skill or its directory (the skill documents the tool, which is still named `changeguard` at the library level).
- Removing the `changeguard` binary alias from the ChangeGuard repo (that's a ChangeGuard decision).
- Updating `ai-brains-path` discovery functions (`find_changeguard_dir`, `extract_project_id_from_changeguard`) — those look for the `.changeguard/` directory, not the binary.