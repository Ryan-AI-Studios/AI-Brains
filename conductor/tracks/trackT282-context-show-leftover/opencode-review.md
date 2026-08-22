# Track review: T282-ContextShowLeftover

**Harness:** OpenCode (`opencode`)
**Track:** `conductor/tracks/trackT282-context-show-leftover`
**Date:** 2026-08-22
**HEAD:** `d370ea1` (T282 docs commit, 1 ahead of `origin/main`, clean tree)

## Summary

The track closes a real honesty hole: `context --show` dumps cwd `.env` `AI_BRAINS_*` lines
and `Repository:` but never mentions a pre-dotenv shell `AI_BRAINS_PROJECT_ID` that the
file overrides. `project whoami` already surfaces `shell_project_id` (`7d97a456…` vs
effective `3581317d…` on this machine); agents that only run `--show` never see it, and
T242's stderr warn is session-quiet after the first fingerprint — exactly why a durable
stdout line is the right fix.

F1 is precise and honest: print exactly
`shell leftover PROJECT_ID: <id> (.env overrides)` (27+36+17 = **80** chars) iff captured
shell is Some nonempty **and** file `AI_BRAINS_PROJECT_ID` is Some nonempty **and**
`shell != file` (exact string, no case-fold). Crucially it does **not** print the suffix
when there is no file PROJECT_ID (F26) — the suffix would lie. F3 redacts
`AI_BRAINS_KEY` / `AI_BRAINS_VAULT_KEY` file lines to `(redacted)`. Freezes are tight:
no T240 F2 write, no `--format`, no whoami/environment/help restyle, no new crates, no
pins, `project.rs` (#1 hotspot) called-not-grown.

Every plan anchor I opened in live `src/` matched the line numbers and semantics claimed;
red state is provable (no pre-existing `SHELL_LEFTOVER_PREFIX` / `leftover_shell_vs_file`
/ `map_show_env_line` / `SHOW_REDACTED_KEY` anywhere in `crates/`).

## Verdict: **Planned** (Approved — fold-in may apply the minors)

## Findings

### B (0)
None. Baseline verified live.

### M (0)
None found.

### m (minor — fold at `/fold-in`)

- **m-1 — Skill scan is stale for the `.claude` skill.** F19/§2.3 say "Skill: no-op (no
  `context --show` subsection)". True for `.agents/skills/ai-brains/SKILL.md` (no
  `context` match at all), **but** `.claude/skills/ai-brains/SKILL.md:50/:57/:88` already
  tells harnesses to run `ai-brains context --show` to "confirm project id / vault env
  warnings" and to *"trust `context --show`"*. When the leftover line ships, that existing
  guidance should gain a one-line pointer ("a `shell leftover PROJECT_ID` line names the
  pre-dotenv shell id the `.env` overrides") so a harness that sees the line isn't
  confused. Cheap, on-scope, and it only edits an already-existing `--show` mention.
  Keep F19's "no new section" decision; this is a word-level addition.

- **m-2 — `AI_BRAINS_VAULT_KEY` is not a live var.** F3/AC3 redact both `AI_BRAINS_KEY`
  and `AI_BRAINS_VAULT_KEY`, but no `AI_BRAINS_VAULT_KEY` symbol exists in `crates/`
  (the canonical product var is `AI_BRAINS_KEY`, legacy `n`). Harmless belt-and-suspenders;
  add a comment so a future reader knows why the alias arm exists, or drop it. One line.

- **m-3 — AC1/AC2 could pin exact-once semantics.** AC4 asserts the leftover line is on
  stdout and "after `Repository:`", but not that it appears **exactly once** (a duplicated
  `println!` or a second `format!` call elsewhere would pass AC4). Suggest
  `== 1`-count assert for the exact leftover string on stdout. Cheap.

### O (opportunity — optional)

- **O-1** — AC4–AC8 could add `isolate_empty_home()` (already in `tests/common/mod.rs:120`)
  so the hermetic child cannot even *see* a developer's real `~/.ai-brains/.env` vault key.
  Not required: `hermetic_bin` sets `AI_BRAINS_ALLOW_ZERO_KEY=1` + zero key, and the
  `context` write-test already runs this way today.
- **O-2** — `context.rs:19–35` is the only `--show` reader of the file; when F1 lands, a
  one-line comment tying the leftover to `main.rs:3256–3263` + `project.rs:156–163` (and
  pointing at the whoami differ at `project.rs:704–709`) would make the capture contract
  discoverable without growing `project.rs`. Comment-only, prefer-zero still honored.
- **O-3** — `map_show_env_line` passthrough for `AI_BRAINS_SESSION_ID` / `AI_BRAINS_HARNESS_ID`
  is correct (not credentials); a doc line saying "model URLs / IDs stay" mirrors §5.4.

## What looks solid

- **F0 go-gate.** Planning TX `fe4e6895` (DOCS) recorded; implement starts a **FEATURE** TX
  only on user `go`. Phase-0 re-verification list is concrete and read-only. No execute
  smuggled into the plan.
- **Live-code line numbers verified** (`crates/ai-brains-cli/src/commands/context.rs`):
  dump loop at `:19–35` (prints `AI_BRAINS_*` file lines then `Repository:`, early
  `return Ok(())` at `:35`, write at `:170`); no-`.env` sentence at `:29–34`.
- **Capture helpers are real and `pub`:** `project.rs:156–163`
  (`record_shell_project_id` / `shell_project_id_captured`) and the whoami differ at
  `project.rs:704–709` match F1's gate exactly (`(Some(shell), Some(env)) if shell != env`).
- **Capture site is before any force-set/clear** at `main.rs:3256–3263`; `--show` reads the
  same OnceLock, so `--show` and `whoami` see the same captured value.
- **`hide_env_values = true`** at `main.rs:997` confirmed; clap 4.6 `hide_env_values` is
  help-only (docs.rs, re-verified) — the plan correctly does **not** lean on it for the
  file dump.
- **T242 freeze respected:** `should_warn_project_context_override` includes `"context"`
  at `main.rs:3019`; `env_warn.rs:124–155` SOOT/stderr shape untouched. F6.
- **Dispatch confirmed vault-opening** (`main.rs:4557–4562` builds `AppContext` even for
  `--show`) — plan's F11 decline of "vault-free `--show`" is accurate and consistent with
  the T256/T242 hermetic pattern already in CI.
- **Dependency pins re-verified in `Cargo.lock`** — clap **4.6.1** (builder 4.6.0),
  serde_json **1.0.150**, chrono **0.4.44**, rusqlite **0.39.0**, uuid **1.23.1**,
  tokio **1.52.3**; crates.io current for clap is **4.6.6**; no clap 5. No bump needed.
  edition **2024**, toolchain **1.95.0** (rust-toolchain.toml).
- **Hermetic scaffolding is real** — `tests/common/mod.rs:90` `hermetic_bin`,
  `:120` `isolate_empty_home`, denylist strips ambient `AI_BRAINS_PROJECT_ID` (`:51`)
  with explicit re-`env` after — matching AC4/AC5 exactly; `tests/common/mod.rs` is the
  right reuse target; existing `warning_json_stdout_hygiene.rs` uses `context` **write**
  only (plan correctly avoids growing it).
- **No pre-existing coverage for `context --show`** in `tests/` (grep empty) — red state
  real. T256's `DUMMY_KEY` fixture (`cli_help_secret_redaction.rs:11`) + 
  `ai_brains_crypto::test_support::assert_no_secret_leakage` both exist (AC3/AC6/AC30
  re-use real symbols).
- **`project_identity_convergence.rs:325/:359`** already asserts `shell_project_id` — AC9
  (whoami JSON regression) has a live home.
- **Doc anchors** — `Docs/CAPABILITIES.md:199` (Show-only row), `Docs/OPERATIONS.md:513`
  (`--show` bullet), CHANGELOG T282 placeholder absent today (add on go). `conductor.md:229`
  T282 Pending row.
- **Redact carve-outs are explicit** — model URLs / PROJECT_ID / SESSION_ID stay; only
  `AI_BRAINS_KEY`/`AI_BRAINS_VAULT_KEY` redacted. `AI_BRAINS_*` prefix loop preserved.

## Deferred fold-in table

| Item | Disposition (per plan §9) | Verdict |
|------|---------------------------|---------|
| Audit `context --show` misses leftover shell vs `.env` (whoami has it) | **Absorb** F1–F4 / AC1–AC5 / AC10 | Correct; row `deferred.md:19` planned; hermdocs rows `:435–447` present |
| Placeholder "without printing `AI_BRAINS_KEY`" | **Absorb** F3 / AC3 / AC6 / AC30 | Correct; KEY redact DoD |
| T276 F10/F11 / closeout shell leftover → T282 | **Absorb** (this track) | `deferred.md:104/:108/:143` all route to T282 |
| T206 CHANGELOG / L3 `context --show` mismatch warn | **Decline** F10 — T240 stderr; cwd `mismatch:false` | Sound (env-vs-path ≠ shell-vs-file) |
| T242 session-quiet hides override | **Partial** — motivation only; restyle **declined** (F6) | Sound |
| T256 `--help` env values | **Decline** F7 — already Completed | Sound |
| last-PR Cursor #197 | **N/A** — comments/reviews empty | Verified below |
| last-PR #192/#188 Work/apply samples | **Decline** — T284 Completed `#193` | Sound |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — no T285 | Sound |
| T283 list cwd-first | **Decline** peers | `deferred.md:19/:109/:142` stay placeholder |
| Identity mismatch quiet `7d97` vs `fcb8a40f` | **Not this track** — T258 adopt-path; T276; shell leftover T282 | Sound |
| SESSION / no-`.env` leftover suffix | **Decline as DoD** F26/F27 | Sound; suffix would lie |

`deferred.md` entirely scanned (rows 19, 108, 143, 255, 299, 306, 353, 357, 401, 404).
Closed/strikethrough rows untouched. T283 remains Placeholder. No new placeholder needed.

## Last-PR Cursor comments

- `gh pr list --state merged --base main --limit 1` → **#197** T281 (squash, 2026-08-22).
- `gh pr view 197 --comments` → `[]`; `pulls/197/comments` → `[]`;
  `pulls/197/reviews` → `[]`; `issues/197/comments` → `[]`. **N/A confirmed.**
- Open PRs on HEAD: Dependabot only (`#58`–`#72`; cargo rusqlite `#61`, chrono `#62`,
  tokio `#59`, thiserror `#60`, tower-http `#58`, GH-actions `#68`–`#72`). **No T285.
  No leftover to mint.**
- `conductor/ISSUES.md` does not exist (F23) — no issue to record.

## Research / tools notes

- **ai-brains preflight --summary** — scope `C:\dev\ai-brains` (`3581317d`), pinned 3619,
  in-context 5 hotspots / 0 decisions / 0 constraints, grants 0/3, harnesses ok. Used.
- **ledgerful** — `doctor` ready (4 warn, 1 optional; `gemini` optional CLI absent —
  not the Cloud backend), `ledger status --compact` **0 pending / 0 drift**,
  `scan --impact` **CLEAN** at `d370ea1`, `hotspots` ranked `project.rs` #1 (3.91),
  `context.rs` #5 (2.65) — matches plan's blast-radii note.
- **ai-brains recall** — returned prior T276 review-track dumps; not used as SoT
  (consistent with plan).
- **Online research (re-verified today):**
  - crates.io clap `max_stable_version` = **4.6.6** (2026-08-06). docs.rs `Arg::hide_env_values`
    still help-only. Verified.
  - clig.dev (human-first, just-enough, discovery) + 12-factor Config (no credentials in
    dumped config) — cited by plan, consistent.
  - Pins table: all load-bearing pins confirmed vs lockfile; no bump proposed.
- **Not run**: full `cargo test` gate (plan audit only); targeted clippy of the
  single crate is optional evidence and was skipped (no code changed yet).

## Notes on other harness files

- `conductor/tracks/trackT281-nightly-probe-vs-tcp/opencode-review.md` exists (T281 audit);
  this T282 file is separate and must not be confused with it. `review.md` /
  `review.codex.md` not touched.
