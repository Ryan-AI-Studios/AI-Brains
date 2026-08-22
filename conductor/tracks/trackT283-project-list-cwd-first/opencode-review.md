# Track review: T283-ProjectListCwdFirst

**Harness:** OpenCode (`opencode`)
**Track:** `conductor/tracks/trackT283-project-list-cwd-first`
**Date:** 2026-08-22
**HEAD:** `dd57150` (T283 docs commit, 1 ahead of `origin/main`, clean tree). Plan dogfood
was `6d3cbc5` (T282 `#198`); `origin/main` moved forward by the docs commit only.

## Summary

The track fixes a real honesty hole: `project list` human table leads with the leftover
store-dump project (`7d97a456` / `C:\dev\crawlx` / 18043 pins) while the cwd path-owner
(`3581317d` / `*C:\dev\ai-brains` / 3633) sits fourth. Every operator/agent that runs
`project list` to answer "which project is this repo?" reads the leftover dump first. The
fix is minimal and correctly scoped: a pure `promote_cwd_owner(rows, cwd_owner)` helper in a
**new** sibling module that moves the cwd path-owner to index 0 of the **human** loop only,
leaving JSON array order (T212 `memory_count DESC, project_id ASC`) and the T267 footer on
the original store vec.

Freezes are tight and DoD-accurate: no store `ORDER BY` change, no JSON reorder, no
`--sort` flag, no star-as-sort, no hardcoded leftover UUID, no `.env` write (T240 F2), no
`cargo install`, no pin bumps (clap 4.6.1 / serde_json 1.0.150 / chrono 0.4.44 / rusqlite
0.39.0 / uuid 1.23.1 / tokio 1.52.3 all verified vs `Cargo.lock`; crates.io current clap is
4.6.6 — no clap 5). Hotspot `project.rs` (#1, 1472 lines) is called-not-grown; units land in
`project_list_order.rs` matching the `project_list_footer.rs` sibling pattern.

Every plan anchor I opened in live `src/` matched the claimed line numbers and semantics;
red state is provable (no `promote_cwd_owner` symbol exists in `crates/`); live baseline
re-verified at `dd57150` via read-only `project list` + `whoami` (leftover first, cwd fourth,
path-owner `3581317d`, `mismatch:false`).

## Verdict: **Planned** (Approved — fold-in may apply the minors)

## Findings

### B (0)
None. Baseline verified live at `dd57150` (read-only, no `.env` write).

### M (0)
None found.

### m (minor — fold at `/fold-in`)

- **m-1 — AC1/AC2 leave single-row promote asserted by "already first", not dedupe.** AC1
  asserts `promote_cwd_owner(&rows, Some("a"))` equals input (already first), but no AC
  asserts the promoted row appears **exactly once** in the result of a non-trivial promote
  (e.g. `Some("c")` → `[c,a,b]`). F33/AC14 already claim once-only; the cheapest hard proof
  is a unit `promote_cwd_owner__middle_id__appears_once` that asserts `count == 1` for the
  promoted id and that `len` is unchanged. One more `#[test]`; no product change.
- **m-2 — `m-2` no-op on `.claude` skill line — confirm the target exists at plan.** F19's
  "`.claude/skills/ai-brains/SKILL.md:89` one sentence" is a word-level edit to a live row
  (verified `:89` is the "Project identity" table row listing `project list`). Trivial; just
  keep the "no new section" rule and the `.agents` no-op (verified: zero `project list`
  matches in `.agents/skills/ai-brains/SKILL.md`). No action beyond existing plan.
- **m-3 — class-only AC10 could pin the JSON-large assertion to a non-brittle form.** The
  plan's AC10 JSON claim "`projects[0].project_id` is still leftover `7d97a456-…` if that id
  remains max-count" is honest (pass-with-observed-data), but on a future machine where the
  leftover is no longer the max-count project the assertion silently degrades. Suggest
  writing it as "JSON `projects[0]` == the max-memory project (observed: leftover 18043)"
  so a re-run on a changed vault still asserts the invariant (JSON size-desc) rather than
  a hardcoded UUID. Comments-only; product behavior unchanged.
- **m-4 — Hermetic fixture sets the `*` via `AI_BRAINS_PROJECT_ID` child env; verify the
  denylist still strips it.** AC5 sets `AI_BRAINS_PROJECT_ID={leftover_id}` as a child env;
  `tests/common/mod.rs:42` denylists ambient `AI_BRAINS_PROJECT_ID` (verified `:51`-style
  strip + re-`env`). Confirm the new test re-`env`s after `hermetic_bin` (the
  `project_list_labels.rs` `pin_memory` pattern does this) so the star assertion isn't
  fighting the strip. Test-only, one line.
- **m-5 — `after_help` sentence must not imply JSON changes.** F35 additive sentence on List
  `after_help` ("human table puts the cwd path-owner first; JSON stays memory-count DESC")
  is exactly the right wording (verified the current `after_help` at `main.rs:2636–2638`
  already names the T267 footer). Keep it symmetric — the current one says "JSON stays
  size-desc" which could read as a promise; fine as-is, no change needed. Belt-and-suspenders
  only: phrase as "human table puts the cwd path-owner first; JSON order unchanged".

### O (opportunity — optional)

- **O-1 — AC3–AC6 could assert the header is row 0 and the promoted row is row 1 in one
  test** (a "first data row" read that grabs line index 1 after the header). The plan's
  "second stdout line" phrasing is already right; an explicit `stdout.lines().nth(1)` assert
  (instead of a "contains" across all lines) would make the "first data row" guarantee
  mechanical. Cheap; strengthens AC3/AC5.
- **O-2 — `resolve_path_alias_for_location` is `pub(crate)` and called from the new module**
  — verified `project.rs:226–237` is `pub(crate)`. The plan correctly reuses it rather than
  forking; consider a doc-comment tie-in pointing at the list promote (comment-only, does
  not grow `project.rs` helpers).
- **O-3 — The helper's error path: `resolve_path_alias_for_location` Err fails the command
  (F26 same as footer).** The plan already documents this parity; just keep it — do not
  downgrade to `unwrap_or_default` for the promote probe.

## What looks solid

- **F1 is precise and honest.** Exact-string compare `project_id == cwd_owner` (F25, no
  contains, no case-fold, no 8-hex prefix); `None`/empty/not-found → memory-desc (F8). The
  design note 5.2 gate is exactly the right shape. Star stays env (F7/F10); path-owner is
  T240/T258 SoT (whoami `path_alias_project_id` = `3581317d` live-verified).
- **JSON freeze is real.** `list_json` (`project.rs:69–102`) reads the same vec — passing the
  unpromoted vec keeps `projects[0]` largest-first for scripts (clig.dev "use `--json` to
  keep output stable"). Envelope `api_version:"1"`, keys frozen (`project.rs:494–512`).
- **Footer freeze is real and necessary.** `print_unaliased_footer` (`project_list_footer.rs:82–133`)
  reads `&projects` (store order); passing the original keeps T267 pick correct. Verified
  the footer test suite (`next_action_honesty.rs:324/386`) asserts leftover never pairs
  with slug — plan correctly does not touch it.
- **Store SQL untouched.** `query_store.rs:549–567` (`list_projects`) + `:584–611`
  (`list_projects_detail`) both `ORDER BY memory_count DESC, p.project_id ASC` (T212 F13);
  `list_projects_detail` path scalar subquery + last_activity COALESCE match F6/F7.
  Not editing is correct (shared by init/detect/preflight).
- **Hotspot discipline.** `project.rs` #1 (1472 lines) — new `project_list_order.rs`
  sibling, `mod.rs:44–48` precedent confirmed (`project_list_footer` next to `project`).
  Units are pure and named `promote_cwd_owner__middle_id__becomes_first` (matches TDD +
  naming).
- **Live baseline verified.** At `dd57150` read-only: first data row `(no alias)
  7d97a456-… 18043 C:\dev\crawlx`; cwd `*C:\dev\ai-brains 3581317d-… 3633` **fourth**;
  `whoami` JSON `shell_project_id=7d97a456`, `path_alias_project_id=3581317d`,
  `mismatch:false`. The plan's §2.1 table matches today's machine exactly.
- **Hermetic scaffolding is real.** `tests/common/mod.rs:120` `isolate_empty_home`, `:90`
  `hermetic_bin`, denylist strips ambient `AI_BRAINS_PROJECT_ID`; `project_register_path.rs`
  fixture pattern (`register_path` + `project list --format json` by-id lookup) is the
  right reuse. New `tests/project_list_cwd_first.rs` with `isolate_empty_home` is required
  and stated.
- **Pins re-verified today** in `Cargo.lock`: clap **4.6.1**, serde_json **1.0.150**,
  chrono **0.4.44**, rusqlite **0.39.0**, uuid **1.23.1**, tokio **1.52.3**; crates.io
  current: clap **4.6.6** (no clap 5), serde_json **1.0.151**, rusqlite **0.40.2** (Dependabot
  `#61`), chrono **0.4.45** (`#62`) — **no bump**. Toolchain **1.95.0**, nextest **0.9.140**,
  workspace **0.1.1**, edition **2024**.
- **Docs anchors verified** — CAPABILITIES `:202–203` (List + List JSON), OPERATIONS `:519–522`
  (stale T76 columns — refresh correct), clap List `main.rs:2636–2638` `after_help`, CHANGELOG
  (no T283 row today), `.claude` skill `:89`. `conductor/deferred.md:481/:487/:494` T283
  rows present.
- **F16 isolation honest.** Stop-before items are explicit; implement-track Phase 6 is the
  only publish path; no `cargo install`, no live `.env`, no `set-alias 7d97a456 AI-Brains`.

## Deferred fold-in table

| Item | Disposition (per plan §9) | Verdict |
|------|---------------------------|---------|
| Audit `project list` leftover-first (7/6) | **Absorb** F1–F8 / AC1–AC6 / AC10 | Correct; verified live today |
| Placeholder "cwd path-owner (or `*` active) first"; JSON freeze vs human-only | **Absorb** F1 human-only; **F2** JSON freeze; **F10** decline star-as-sort | Sound; star/env ≠ sort key |
| T276 F10 / closeout `project list` leftover-first | **Absorb** (this track) | `deferred.md:494` routes here |
| T282 closeout T283 peer | **Absorb** (this track) | `deferred.md:495` present |
| T267 footer leftover-as-AI-Brains | **Decline** F3 — pass original vec | Correct (footer owns its vec) |
| T212 labels / JSON keys / store ORDER BY | **Decline** F2/F11/F30 — freeze | Correct |
| T230 never-blank | **Decline** — labels unchanged | Correct |
| last-PR Cursor #198 | **N/A** — comments/reviews empty | Verified below |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` | Sound |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — no T285 | Sound |
| T240 F2 / clap 5 / DTO required keys | **Decline** F4/F12/F17 | Sound |
| Identity mismatch quiet `7d97` vs `fcb8a40f` | **Not this track** — T258 adopt-path; leftover T276; sort this track | Sound |
| JSON reorder / `--sort` / star-as-sort | **Decline** F2/F5/F10 | Sound — clig.dev prefers default-right over flags |

`deferred.md` scanned (rows 20, 481, 491, 494, 497–506 + 8.x §9). Closed/strikethrough rows
untouched. No new placeholder minted (#198 empty → N/A). T283 is **Planned** (not yet
implemented).

## Last-PR Cursor comments

- `gh pr list --state merged --base main --limit 1` → **#198** T282 (squash, 2026-08-22).
- `gh pr view 198 --comments --json comments,reviews` → `[]`; `pulls/198/comments` → `[]`.
  **N/A confirmed.**
- Open PRs on HEAD: Dependabot only (`#58`–`#72`; cargo rusqlite `#61`, chrono `#62`, tokio
  `#59`, thiserror `#60`, tower-http `#58`, GH-actions `#68`–`#72`). **No T285. No leftover
  to mint.**
- `conductor/ISSUES.md` does not exist (F23) — no issue to record.

## Research / tools notes

- **ai-brains preflight --summary** — scope `C:\dev\ai-brains` (`3581317d`), pinned 3633,
  in-context 5 hotspots / 0 decisions / 0 constraints, grants 0/3, harnesses ok. Used.
- **ledgerful** — `doctor` ready (warn: legacy sig v1 rows + timings >10k, optional model
  unreachable), `ledger status --compact` **0 pending / 0 drift** at `dd57150`,
  `scan --impact` **CLEAN**, `search list_projects_detail` → `query_store.rs:584` +
  `project.rs:27` (plan's claim verified).
- **ai-brains recall** — prior T283-context review dumps returned; not used as SoT
  (consistent with plan §2.4).
- **Online research (re-verified today):**
  - crates.io `max_stable_version`: clap **4.6.6**, serde_json **1.0.151**, rusqlite
    **0.40.2**, chrono **0.4.45** — pins table correct.
  - clig.dev (human output may change; JSON stable; default-right) — cited by plan,
    consistent with F1/F2/F10.
  - kubectl `config get-contexts` marks current with `*`, does **not** require current-first —
    plan correctly does not copy it (sort key is path-owner, not a kubectl analog).
- **Not run**: full `cargo test` gate (plan audit only); targeted clippy of the single
  crate is optional evidence and was skipped (no code changed yet). Live `project list` /
  `whoami` were run read-only to confirm §2.1 (no `.env` write).

## Notes on other harness files

- `conductor/tracks/trackT282-context-show-leftover/opencode-review.md` exists (T282 audit,
  closed). This T283 file is separate and must not be confused with it. `review.md` /
  `review.codex.md` not touched.
