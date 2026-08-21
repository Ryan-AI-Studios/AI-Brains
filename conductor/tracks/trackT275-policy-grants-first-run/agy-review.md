# Track review: T275-PolicyGrantsFirstRun

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT275-policy-grants-first-run`  
**Date:** 2026-08-21  
**HEAD:** `c576b58`  

---

## Summary

Track T275 resolves a core usability and communication friction identified during the 2026-08-21 CLI dogfood audit:
When a vault has not yet been bootstrapped with discovery grants (`ReadEvidence`, `ReadConclusions`, `ReadDecisions`), `ai-brains briefing project --format human` currently displays `> **Denied:** ...` followed immediately by `## Decisions (current authority)` `_None_` and `## Conclusions (current authority)` `_None_`.
On a vault containing thousands of project memories (over 3,300 pins on this machine), this `_None_` placeholder misleads agents and operators into believing the vault has zero decisions rather than understanding that a policy grant wall is in effect.

T275 addresses this with targeted, minimal changes:
1. **Grant-Wall Display Honesty:** In `crates/ai-brains-control-plane/src/briefings/renderer.rs`, replaces the deceptive `_None_` text with `_(hidden until discovery grants)_` when `packet.denied` is true. Adds a concise grant-wall notice (`BRIEFING_DENIED_GRANT_WALL`) clarifying that ungoverned pins remain accessible via `ai-brains recall` and `search`.
2. **Preserving Authorized Empty State:** When discovery grants are present and the vault genuinely has zero decisions, `render_project_markdown` preserves `_None_` and the T263 empty authority footer.
3. **CLI Bootstrap End-to-End Test Locks:** Adds hermetic integration tests proving that running `ai-brains policy bootstrap` (for the default System principal) cleanly unlocks `ai-brains briefing project` (`denied: false`) and `ai-brains evidence list` (`exit 0`).
4. **Strict Architectural Restraint:** Does not introduce auto-granting on `init`/`preflight` (upholding least-privilege default-deny principles), does not alter `POLICY_DENIED_HINT` (leaving hint unification to T280), and makes zero changes to `project.rs` or CLI `preflight.rs`.

The plan is well-bounded, respects all invariant freezes, and directly resolves the audit friction.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Character budget on `BRIEFING_DENIED_GRANT_WALL` (AC2 / F2):** Ensure the `BRIEFING_DENIED_GRANT_WALL` string is strictly ≤140 characters and renders on a single line across standard terminal widths without wrapping awkwardly.
- **m2: Personal briefing markdown regression guard (F32 / AC1):** Verify that `render_personal_markdown` continues to use its distinct T263 recall guidance without cross-contaminating project bootstrap strings.

### Opportunities (O)
- **O1: Clear documentation of System principal default in tests (F31):** In `crates/ai-brains-cli/tests/policy_bootstrap.rs`, add an inline comment in test AC4 explaining why `--principal-id` is omitted (aligning with `cli_principal()` System default) to prevent future test writers from copying the T210 `bbbb...` principal trap.
- **O2: Co-locating denied and empty authority constants:** Keep `BRIEFING_DENIED_GRANT_WALL`, `BRIEFING_DENIED_HIDDEN`, `BRIEFING_EMPTY_AUTHORITY_NOTICE`, and `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` adjacent in `renderer.rs` with clear doc comments distinguishing their roles.

---

## What Looks Solid

1. **Upholding Default-Deny & Least Privilege:** Rejecting automatic or implicit grant issuance on `init` or `preflight` adheres to zero-trust standards (Microsoft Entra / OSO RBAC best practices) while still providing clear guidance to the operator.
2. **Clear Separation of Concerns:** T275 specifically targets the misleading `_None_` render and test coverage without prematurely adopting T280's hint wording unification or T276's leftover rebinds.
3. **Capture Independence Guaranteed:** Ungoverned search (`recall`, `search`, preflight index) remains completely functional without policy grants.
4. **Hotspot Avoidance:** Zero edits to top hotspots (`project.rs`, CLI `preflight.rs`, `doctor.rs`, `sync.rs`). Changes are localized to `renderer.rs` and hermetic test suites.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Briefing/progressive/lists POLICY_DENIED (0 of 3) | Absorbed into DoD (F1–F6 / AC1–AC5) | Solved via grant-wall rendering + CLI bootstrap locks |
| `policy bootstrap --dry-run` daily unapplied | Partial (F10) | Hermetic unlock is DoD; live apply requires owner confirm |
| T241 F21 skill one-liner | Absorbed (F23) | Documented in CAPABILITIES / skill |
| Auto-grant on `init` / `preflight` | Declined (F8 / F9) | Preserves explicit least-privilege opt-in |
| Hint omit-`--scope` unification | Declined (F11 → T280) | Properly isolated in Track T280 |
| T263 H2 pin→Approved | Declined (F12) | Preserves governed review workflow |
| Leftover `7d97a456` rebind | Declined (F25 → T276) | Properly isolated in Track T276 |
| PR #188 Bugbot Mediums | Declined (F24 → T284) | Properly tracked in Track T284 |
| Last-PR Cursor #189 | N/A (empty) | Scanned with 0 findings |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#189](https://github.com/Ryan-AI-Studios/AI-Brains/pull/189) (merged 2026-08-21, T274 `Pins beat harness session dumps`).
- **Cursor Comments:** 0 comments (`[]` on PR #189).
- **Disposition:** N/A (no pending findings).

---

## Research / Tools Notes

- **Authorization Standards:** clig.dev, Microsoft Entra, and OSO RBAC guidance affirm default-deny with explicit discovery grants. Distinguishing "permission denied" from "zero resources" is standard CLI design.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,325 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search run_bootstrap`: Located at `crates/ai-brains-cli/src/commands/policy_cmd.rs:234`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
