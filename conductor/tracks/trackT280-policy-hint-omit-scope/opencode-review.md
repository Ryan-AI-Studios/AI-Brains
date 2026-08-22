# Track review: T280 — Deny HINT omit-scope

**Harness:** OpenCode (`opencode`)
**Track:** `conductor/tracks/trackT280-policy-hint-omit-scope`
**Date:** 2026-08-22
**HEAD:** `f35884e`

## Summary

Plan audit of T280 (F1 HINT rewrite + F2 markdown-next = SHORT + daemon AC11 tighten + docs).
Reviewed full `spec.md` + `plan.md` against live `src/`, lockfile pins, deferred §9,
last-PR Cursor comments, and dogfood output. No blockers, no majors. One opportunity-level
line-anchor note (AC8 spec `:546` vs live `:548`; test name matches, zero correctness
impact). The F1 string as written in the plan is exactly **172 chars**, contains `policy
bootstrap` and `omit --scope`, contains no `--scope …` (U+2026) and no `bootstrap --scope`.
No current pin blocks the plan (all load-bearing crates verified against the lockfile).

## Findings (B/M/m/O)

### B (Blocker)

None.

### M (Major)

None.

### m (minor)

None.

### O (Opportunity)

- **O1 — AC8 anchor drift (line number only).** `spec.md` AC8 cites `tests/policy_bootstrap.rs:546`;
  live test starts at `:548` with comment at `:546`. Test name `policy_bootstrap__no_scope_no_context__exit_2`
  matches spec intent exactly, so this is cosmetic. No action required; fold-in may re-anchor if cheap.

## What looks solid

- **F1 string is faithful and complete.** Verified character-exact in plan.md blockquote:
  `ensure a grant for this capability exists; run `ai-brains policy bootstrap --dry-run` then
  `ai-brains policy bootstrap` (omit --scope when project context is authoritative)` = 172 bytes.
  Satisfies AC1/AC2/AC3 needles (contains `policy bootstrap`, `omit --scope`, `--dry-run`;
  not contains `--scope …` / `bootstrap --scope`). No historical 140-char HINT cap exists
  (T275 140 is the grant-wall) — spec §8 risk row confirms substring tests, not char budget.
- **F2 (markdown next = SHORT) is correctly scoped.** `renderer.rs:13` `BRIEFING_DENIED_NEXT_STEP`
  today contains `--scope …`; `renderer.rs:16-17` `BRIEFING_DENIED_DENIAL_HINT` already SHORT
  (`--dry-run` then apply). AC4's "next precedes `## Decisions`" ordering is confirmed at
  `renderer.rs:82-85` and locked by test `render_project_markdown__denied__bootstrap_next_step_no_empty_authority`.
- **All spec anchors verified live (no invented paths).**
  - CLI `POLICY_DENIED_HINT` `governed_common.rs:51`; `policy_denied_hint_details` `:140`; T243 unit `:725`;
    AC4/F5 unit `:733-739`.
  - Daemon twin `services.rs:989`; `policy_denied_with_hint__includes_details_hint` `:1226`;
    AC11 disjunction `:1238-1245` (hint contains `policy show` OR `policy bootstrap`).
  - CP twin `query.rs:93` (function-local const inside `progressive_query`); `renderer.rs:13/:16-17`.
  - Call sites use the helper: `policy_cmd.rs:201`, `evidence.rs:158/:228`, `source.rs:135/:222`,
    `review.rs:110`. `governed_query.rs` overlays `:73/:124/:132/:186/:255`.
  - `tests/policy_bootstrap.rs` AC7 `:526`, AC8 `:548`; `main.rs` after_help dual examples
    `:1620/:2204/:2209/:2223/:2239`; `Docs/CLI-EXIT-CODES.md:94`; `Docs/CAPABILITIES.md:322`.
- **No production `unwrap`/`expect`/`panic` risk** from F1/F2 — pure const string edits + test additions.
- **F15 (file growth) honored:** HINT lives in `governed_common.rs` (+ daemon + CP twins).
  Call sites already use the shared helper; `project.rs` / `preflight.rs` / `doctor.rs` / `sync.rs`
  / `policy_cmd.rs` / `evidence.rs` / `source.rs` / `review.rs` growth is explicitly excluded in AC12.
- **DoD / non-goals are explicit and consistent.** No runtime context-aware HINT, no HINT-into-SHORT
  merge, no clap 5 / rusqlite 0.40 bump, no new DTO keys. AC6 no-context arm and AC13 soft-resolve
  keep `--scope` still-valid semantics (F26).
- **Dogfood confirms live state.** `ai-brains preflight --summary`: Pinned 3547, sessions 3, in-context 0/0/0,
  Word Count 560, grants 0 of 3, SHORT remediator no `--scope`; `policy check --capability ReadEvidence`
  → exit 3 + old hint (pre-change, as expected); `policy show --format json` → exit 0 grants [];
  `doctor --summary` → LONG `omit --scope when project context is authoritative`; `evidence list --format json`
  → exit 3 + old hint.
- **Ledger/tooling healthy.** `ledgerful ledger status --compact` → 0 pending / 0 unaudited drift;
  `ledgerful doctor` → 11 hygiene findings collapsed, models unreachable (pre-existing, not T280).
- **Pins current (no bumps).** clap 4.6.1 (crates.io 4.6.6, no clap 5), serde_json 1.0.150 (1.0.151),
  chrono 0.4.44 (0.4.45), rusqlite 0.39.0 (0.40.2), uuid 1.23.1 (1.25.0). No lockfile pin needs bumping
  for T280.
- **PR / Cursor audit clean.** Last merged PR #195 (2026-08-22): comments `[]` + reviews `[]` — spec's
  "no Cursor comments" claim exact. Open PRs are Dependabot only (#58-#72). No stray T285 PR on HEAD.
- **clig.dev alignment.** One next command, lead-with-example (`bootstrap --dry-run` then `bootstrap`),
  rewrite errors for humans — plan matches.

## Deferred fold-in table

| deferred.md row | Disposition in plan | OK? |
|-----------------|--------------------|-----|
| `deferred.md:17` (deny/`policy show` `--scope …` vs doctor omit) | Absorbed F1-F4 / AC1-AC7 / AC10 — show already SHORT (affirm); HINT + markdown next are DoD | Yes |
| `deferred.md:65` (T275 F11 "HINT still `--scope …`" leftover) | Absorbed F1 / F2 | Yes |
| `deferred.md:97` (T243 AC12 wording freeze) | **Lifted** in F1 / F27 — new freeze | Yes |

No open `deferred.md` row is in scope but unmentioned. No missing placeholder needed.

## Last-PR Cursor comments

None found on the last merged PR (#195) or on any open PR on HEAD. Spec's claim verified
via `gh pr view --comments` + `gh api pulls/{n}/comments`. No un-minted Cursor/Bugbot
leftover to roll into this track.

## Research / tools notes

- **Verified** (opened live `src/`): all spec anchors listed above; `renderer.rs` ordering
  `:82` next-step → `:85` grant-wall before Decisions; `renderer.rs` grant-wall const
  `:22-23` is 88 chars, one line, ≤140 (frozen, untouched by T280).
- **Verified** (lockfile + crates.io): clap 4.6.1 locked, 4.6.6 latest, no clap 5; serde_json
  1.0.150; chrono 0.4.44; rusqlite 0.39.0; uuid 1.23.1.
- **Verified** (dogfood): `ai-brains preflight --summary`, `policy check/show`, `doctor --summary`,
  `evidence list` all return the pre-change wording; nothing broken by review.
- **Verified** (tools): `ledgerful ledger status --compact`, `ledgerful doctor`, `ledgerful scan --impact`
  completed; no drift. Models unreachable in ledgerful doctor — pre-existing, unrelated to T280.
- **Skipped**: `ledgerful ask` (model service unreachable) and full CI gate (plan-review scope only;
  targeted `cargo clippy` deferred to implement phase).

## Verdict: Planned

T280 is planned and ready for implementation via `/fold-in`. No blockers or majors; the only
finding is a line-number anchor drift (O1) with zero behavioral impact. Pins are current, all
named `src/` anchors exist, deferred §9 and last-PR Cursor audit are complete and clean, and
the F1 string exactly matches AC1-AC3 as written.
