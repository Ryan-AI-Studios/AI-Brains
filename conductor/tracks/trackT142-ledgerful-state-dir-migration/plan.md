# T142 Plan

## Tasks

### Track 1 — Functional (Rust) — core-engineer
1. **Red test:** `crates/ai-brains-path/tests/ledgerful_dir_discovery.rs` — asserts `find_ledgerful_dir` finds `.ledgerful/`, falls back to `.changeguard/`, returns `None` for empty tree; `extract_project_id_from_ledgerful` reads `project_id` from either dir. Uses `tempfile::tempdir()`, `rstest` `#[case]`. Fails to compile today (fn doesn't exist).
2. **discovery.rs:** add `find_ledgerful_dir` (checks `.ledgerful/` then `.changeguard/`), `extract_project_id_from_ledgerful`. Keep old fn bodies as thin wrappers delegating to new ones.
3. **lib.rs:** export new names; `#[deprecated]` re-exports of old names.
4. **context.rs:** use new fns; update user-facing string to `.ledgerful`; env var write/read → `LEDGERFUL_TX_ID` (filter both names on rewrite).
5. **context.rs (AppContext), pin.rs, main.rs:** shared helper `read_ledger_tx_id()` checking `LEDGERFUL_TX_ID` then `CHANGEGUARD_TX_ID` (warn on fallback). `#[arg(env="LEDGERFUL_TX_ID")]` in main.rs.
6. Verify: `cargo nextest run -p ai-brains-path ; cargo nextest run -p ai-brains-cli`.

### Track 2 — Prose + Skill (parallel) — general
7. **AGENTS.md:** lines 10, 40, 82.
8. **.gitignore:** add `.ledgerful/` after `.changeguard/`.
9. **Skill:** copy `c:\dev\ledgerful\.agents\skills\ledgerful\SKILL.md` → `.claude/skills/changeguard/SKILL.md`, set `name: ledgerful`. Copy `references/` files too.
10. **onboarding/ai-brains skills:** ChangeGuard → Ledgerful prose.
11. **Docs + conductor/conductor.md + failure-drills.md:** ChangeGuard → Ledgerful (skip `conductor/archive/`).

### Track 3 — Rust comments/strings (parallel) — general
12. Comments + user-facing error strings in `crates/` referring to the product (not legacy `.changeguard/` path) → "Ledgerful". Leave schema.json/migrations.

### Final (manager) — sequential after all tracks
13. `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit ; ledgerful verify --scope full`.
14. `ledgerful ledger atomic T142-ledgerful-state-dir-migration --category FEATURE --summary "Migrated state dir discovery + product name to ledgerful" --reason "Fresh ledgerful installs use .ledgerful/; T97 only did the binary name"`.
15. Report to user; offer to commit (do NOT commit without explicit ask).

## Delegation Map

| Task | Agent | Parallel? |
|---|---|---|
| 1–6 | core-engineer | Track 1 (Rust) — must complete before final gate |
| 7–11 | general | Track 2 (prose) — parallel with Track 1 |
| 12 | general | Track 3 (Rust prose) — parallel but may touch same files as Track 1; coordinate by scoping to comments/strings only, not the lines Track 1 edits |
| 13–15 | manager | sequential after all |

## Coordination Note

Track 1 (core-engineer) and Track 3 (general) both touch `crates/`. Per AGENTS.md "Parallel code edits conflict". Mitigation: Track 3 touches ONLY comments and error-message string literals, NOT the lines Track 1 edits (discovery.rs fns, context.rs env var lines, pin.rs, main.rs env attr). Manager sequences final gate.