# Track T142: Migrate `.changeguard/` State Directory + Product-Name to Ledgerful

**Status:** Pending
**Started:** —
**Owner:** manager (delegated)
**Priority:** P1 — AI-Brains cannot discover `.ledgerful/`-only installs today, breaking `ai-brains context` auto-discovery on fresh ledgerful repos.
**Source:** User directive 2026-06-29; ledgerful rebrand now uses `.ledgerful/` state dir (verified `C:\dev\ledgerful\.ledgerful` exists; `.changeguard` does not). Track T97 (complete) migrated only the binary invocation, deliberately leaving the state dir and product-name references for a follow-up.

---

## Problem Statement

Ledgerful has been fully rebranded from ChangeGuard. The state directory is now `.ledgerful/` (confirmed in `c:\dev\ledgerful\.agents\skills\ledgerful\SKILL.md` lines 27, 56, 153, 170, 172, 208, 242 and on disk). T97 already migrated `Command::new("changeguard")` → `"ledgerful"` but explicitly left:

- `.changeguard/` directory references in `ai-brains-path::discovery`
- `changeguard` skill content (now stale vs the canonical ledgerful skill)
- Prose "ChangeGuard" references in AGENTS.md, onboarding/ai-brains skills, Docs/, conductor/conductor.md
- The `CHANGEGUARD_TX_ID` env var

On a fresh ledgerful install with only `.ledgerful/` present, `find_changeguard_dir` returns `None`, so `ai-brains context` falls back to the deterministic-hash ProjectId instead of reusing the ledgerful project_id. This is a functional regression on new installs.

## Acceptance Criteria

**AC1 — State dir discovery (functional fix):**
`crates/ai-brains-path/src/discovery.rs` looks for `.ledgerful/` FIRST, then falls back to `.changeguard/` for legacy installs. The functions are renamed:
- `find_changeguard_dir` → `find_ledgerful_dir`
- `extract_project_id_from_changeguard` → `extract_project_id_from_ledgerful`
Deprecated re-exports under the old names remain in `lib.rs` for one release cycle with `#[deprecated(note = "use find_ledgerful_dir / extract_project_id_from_ledgerful")]`.

**AC2 — Consumer updated:**
`crates/ai-brains-cli/src/commands/context.rs` uses the new names. The user-facing string `"Auto-discovered project ID from .changeguard: …"` becomes `"Auto-discovered project ID from .ledgerful: …"` (or reflects whichever dir was actually found).

**AC3 — Env var (non-breaking):**
`CHANGEGUARD_TX_ID` is read via a shared helper that checks `LEDGERFUL_TX_ID` first, then falls back to `CHANGEGUARD_TX_ID` (with a one-time `tracing::warn!` deprecation notice on the fallback). Affected callsites:
- `crates/ai-brains-cli/src/main.rs:226, 246` — `#[arg(long, env = "LEDGERFUL_TX_ID")]` (clap auto-reads the new name; the manual fallback in `context.rs`/`pin.rs` covers the old name)
- `crates/ai-brains-cli/src/context.rs:91`
- `crates/ai-brains-cli/src/commands/pin.rs:42`
- `crates/ai-brains-cli/src/commands/context.rs:145,156` — writes/reads `LEDGERFUL_TX_ID` in `.env`, but filters BOTH `LEDGERFUL_TX_ID` and `CHANGEGUARD_TX_ID` lines on rewrite so old `.env` files migrate cleanly.

**AC4 — .gitignore:**
`.ledgerful/` added; `.changeguard/` kept for legacy installs.

**AC5 — AGENTS.md (project rules):**
- Line 10: `changeguard ledger` → `ledgerful ledger`
- Line 40: `.changeguard/` → `.ledgerful/`
- Line 82: "changeguard ledger" → "ledgerful ledger"

**AC6 — Skill consolidation:**
`.claude/skills/changeguard/SKILL.md` content replaced with the canonical ledgerful skill from `c:\dev\ledgerful\.agents\skills\ledgerful\SKILL.md`, with `name: ledgerful` in frontmatter. Directory name `.claude/skills/changeguard/` stays (per user preference). References in `.claude/skills/onboarding/SKILL.md` and `.claude/skills/ai-brains/SKILL.md` updated from "ChangeGuard" to "Ledgerful" in prose.

**AC7 — Docs + active conductor prose:**
`ChangeGuard` → `Ledgerful` in: `README.md`, `Docs/OPERATIONS.md`, `Docs/WORKFLOWS.md`, `Docs/status.md`, `Docs/hooks-status.md`, `Docs/antigravity-rule.md`, `Docs/audit2.md`, `Docs/FTS5-catch.md`, `Docs/schemas/sync-pull-record.json`, `conductor/conductor.md`, `conductor/failure-drills.md`. **Out of scope:** `conductor/archive/**` and already-complete track specs (historical record, per user preference).

**AC8 — Rust source prose:**
Comments and user-facing error strings in `crates/` updated from "ChangeGuard" to "Ledgerful" where they refer to the product (not the legacy `.changeguard/` path, which remains valid as a fallback name). `schema.json` and migration filenames are NOT renamed (historical artifacts).

**AC9 — TDD:**
Red → Green for `find_ledgerful_dir` discovering `.ledgerful/` and falling back to `.changeguard/`. Test names follow the `function_or_feature__condition__expected_result` convention. Tests use `tempfile::tempdir()`, `#[case]` for parameterization, assert specific values, no `unwrap()`/`expect()` in production code.

**AC10 — CI gate:**
`cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit ; ledgerful verify --scope full` passes.

## Design Notes

- The `.changeguard/` directory name remains valid as a fallback. AI-Brains itself still has one on disk today; legacy installs keep working.
- The `changeguard` skill directory name stays (per user pref); only its content is replaced with the ledgerful skill's.
- Env var rename is non-breaking: new name preferred, old name accepted with deprecation warning.
- No schema/migration filename changes — those are historical artifacts.
- Provenance: `ledgerful ledger atomic` for this track; intermediate commits allowed per AGENTS.md.

## Files

- `crates/ai-brains-path/src/discovery.rs` — rename + `.ledgerful/` first
- `crates/ai-brains-path/src/lib.rs` — new exports + deprecated re-exports
- `crates/ai-brains-path/tests/ledgerful_dir_discovery.rs` — NEW red test
- `crates/ai-brains-cli/src/commands/context.rs` — consumer + strings + env var write
- `crates/ai-brains-cli/src/context.rs` — env var read helper
- `crates/ai-brains-cli/src/commands/pin.rs` — env var read helper
- `crates/ai-brains-cli/src/main.rs` — `#[arg(env = "LEDGERFUL_TX_ID")]`
- `AGENTS.md` — 3 edits
- `.gitignore` — add `.ledgerful/`
- `.claude/skills/changeguard/SKILL.md` — replace content
- `.claude/skills/onboarding/SKILL.md` — prose
- `.claude/skills/ai-brains/SKILL.md` — prose
- `README.md`, `Docs/*`, `conductor/conductor.md`, `conductor/failure-drills.md` — prose
- Rust source comments/strings across `crates/`

## Out of Scope

- Renaming the `changeguard` skill directory (kept per user pref).
- Rewriting `conductor/archive/**` or complete track specs (historical record).
- Removing the `changeguard` binary alias (ledgerful repo's decision).
- Renaming `schema.json` / migration filenames.
- The library crate name `changeguard` (Rust imports unchanged — that's the ledgerful repo's crate, not AI-Brains').