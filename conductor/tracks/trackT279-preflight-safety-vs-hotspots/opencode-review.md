# Track review: T279-PreflightSafetyVsHotspots

**Harness:** OpenCode (`opencode`)
**Track:** `conductor/tracks/trackT279-preflight-safety-vs-hotspots`
**Date:** 2026-08-22
**HEAD:** `448ef47` (docs commit "plan T279 preflight Safety vs live hotspots", 1 ahead of origin/main `631a8f8`; product `src/` = T278)

---

## Summary

T279 fixes the preflight Safety section-identity hole: `preflight --pretty` "Repository Bearings & Safety" currently opens with `## Objective` — the Safety SQL's `LIKE '%CONSTRAINT:%'` (anywhere-match) steals buried markers from T272 review-track dumps, and the first displayed line is the dump heading. The plan (a) replaces LIKE-anywhere with a **leading-line GLOB** (`safety_marker_glob_sql`, CONSTRAINT/INVARIANT/HOTSPOT + `ASSISTANT: ` variants, no `DECISION:`), (b) **live-injects** up to 5 `ledgerful hotspots --json --limit 5` paths (same argv as CLI `safety.rs`) project-scoped into Safety as `HOTSPOT: <path> score=0.05`, and (c) **always-emits** the Safety header with an honest empty remediator (`No in-context hotspots. next: ai-brains safety sync --dry-run`) instead of omitting the section. `--global` never live-injects (T214 F9 analog). JSON keys (`{text, word_count}` + `sections[]`) stay frozen.

This review re-verified every load-bearing anchor against the live tree at HEAD `448ef47` (docs-only ahead of `631a8f8`). All named code locations, fixtures, clap surface, caps, GLOB precedents, pin versions, deferred rows, and dogfood signals match. **No Blockers, no Majors.** Two minor items and one opportunity.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)

- **m1 — AC6 test-name drift (spec §4 AC6, spec §7 row 4, plan Phase 1).** Plan cites `preflight_pretty__summary__dual_model_unchanged`; the real tests are `preflight_pretty__summary_smoke__dual_model_unchanged` (`preflight_pretty_readability.rs:266`) and `preflight_pretty__summary_compact__dual_model_unchanged` (:532). Intent intact — both assert **no** `--- Repository Bearings` on `--summary` (:292 / :553). Cosmetic; Phase 0 re-read covers it.
- **m2 — live hotspot displayScore drift (spec §2.1).** Spec/plan row says hotspot #1 `project.rs` (**3.944**); live `ledgerful hotspots --json --limit 5` reports `displayScore` **3.934949** (raw `score` 0.050159533). The plan's F2 format uses raw `score={score:.2}` → **"0.05"**, which is exactly what `safety sync --dry-run` prints today (`score: 0.05`). The displayScore figure is a secondary reference, not a pin — no behavior change. Cosmetic.

### Opportunities (O)

- **O1 — F3 remediator wording under `--global`.** The empty string `No in-context hotspots. next: ai-brains safety sync --dry-run` is cwd-flavored. Under `--global` (F4, live inject always skipped) an operator could see that hint while the dry-run would reflect *this* project — the hint stays valid for the operator, but the one-liner could note the project scope. Optional; not a plan defect.

---

## What looks solid

1. **Leading-line GLOB is the right mechanism, with in-tree precedent.** `index_marker_glob_sql` (`session_chrome.rs:73`) and `authority_glob_sql` (`:52`) already use leading-prefix GLOB (T274 F36 identifier-checked, bind-free). SQLite GLOB is case-sensitive Unix glob; `CONSTRAINT:*` + `ASSISTANT: CONSTRAINT:*` is the T274 F8 subset. A buried `## Overview`/`## Objective` line that only contains CONSTRAINT **mid-line** no longer matches — that is the dump steal fixed. `DECISION:` correctly excluded (Safety ≠ Index pass-1).
2. **The steal is reproduced and DoD-pinned.** `preflight.rs:294/301` `LIKE '%CONSTRAINT:%'` confirmed live. T274 AC6 dump would only match via substring; AC3 hermetic proves it leaves Safety; T272 AC2/AC3 leading-CONSTRAINT fixtures (`preflight_global_isolation.rs`, leading `CONSTRAINT: A-two` etc.) stay green because GLOB matches leading.
3. **Live inject reuses the CLI's exact argv and stays fail-open.** `safety.rs` `fetch_hotspots_json` :102–128 (JSON starts at `[` line), clap `SafetyCommands::Sync` `--limit` default 5 (:2947–2956), `--dry-run`. F35 (spawn fail / non-zero / no `[` / parse err / empty array → no inject) + F13 skip-env gives the hermetic escape hatch; AC8 (TempEnv no-spawn unit) makes it deterministic. No `wait-timeout` crate (soft hang residual, accepted).
4. **Summary counts stay honest.** CLI `preflight.rs:886–888` `text.matches("HOTSPOT:")` untouched; F3 empty string contains **no** `HOTSPOT:` so `in_context_hotspots` stays 0 when the remediator fires (AC12). `--summary` never prints Bearings (AC6). File growth prohibition (F14/AC13) is consistent with the hotspot table — `project.rs` #1, CLI `preflight.rs` #7 untouched.
5. **Section identity is preserved; JSON surface frozen.** `PreflightContextResponse` `{text, word_count}` + `sections[]` (T180/T265) — no `hotspots[]` key, no new required keys (F10). Splitter keys on the Bearings header, which F3 always emits.
6. **Dogfood reproduce.** `ai-brains preflight --summary` (Pinned 3516, in-context 0/0/0, grants 0 of 3) and `ai-brains safety sync --dry-run` (**5** paths: `project.rs`, `sync.rs`, `forget.rs`, `context.rs`, `governed_common.rs`) match the spec table exactly.
7. **Pin table and online research verify.** Workspace/lock (clap 4.6.1, serde_json 1.0.150, chrono 0.4.44, rusqlite 0.39.0, uuid 1.23.1) vs ecosystem (4.6.6 / 1.0.151 / 0.4.45 / 0.40.2 / 1.25.0) — all intentionally **not** bumped; **no clap 5**, no rusqlite 0.40 (Dependabot `#61` declined, no T285). GLOB is SQL, not a 0.40 API — correct.

## Deferred fold-in table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| preflight Safety = review-track Objective dump | **Absorb** F1–F3 / AC3–AC4 / AC10 | DoD. AC3 proves dump leaves Safety. |
| T274 F23 Safety SQL leftover | **Absorb** F1 (GLOB) | Correct lift; Safety SQL stayed in `preflight.rs` since T274. |
| T274 AC6 dump CONTAIN buried CONSTRAINT safety-steal | **Absorb** AC3 | GLOB makes buried CONSTRAINT un-matchable; dumps remain eligible for Index (intended). |
| T250 F12 HOTSPOT float reformat | **Partial** F15 — live line `score={:.2}` only | Verified live: `{:.2}` of 0.050159 → `0.05`, matches dry-run. |
| T272 skip / T264 caps / `GLOBAL_SAFETY_*` | **Affirm freeze** F5/F6 | Live lines have no id; caps untouched. |
| T272 F18 session `HOTSPOT:` skip | **Decline** F32 | Soft residual, not DoD. |
| `query_ledgerful` Intelligence rewrite | **Decline** F11 | `bridge export --hotspots` is a different source; live empty this dogfood. |
| deny/policy `--scope` | **Decline → T280** | Out of scope. |
| nightly dual-probe / `context --show` / list cwd-first | **Decline → T281/T282/T283** | Out of scope. |
| leftover `7d97a456` rebind | **Decline** — T276 Completed; owner-confirm | Not this track. |
| last-PR #194 Cursor | **N/A** — comments/reviews empty | Re-verified `[]` / 0 today. |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** | Standing freeze. |
| T240 F2 / clap 5 / DTO keys | **Decline** F12/F17 | Not this. |

## Last-PR Cursor comments

- **Scanned PR:** [#194](https://github.com/Ryan-AI-Studios/AI-Brains/pull/194) (merged 2026-08-22, T278 session neighbor PREVIEW captions).
- **Comments:** 0 — `gh pr view 194 --comments` returned `[]`; `gh pr view 194 --json reviews -q '.reviews | length'` → `0`.
- **Open PRs on HEAD:** Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45).
- **Disposition:** N/A. No T285 minted.

## Research / tools notes

- **Live code opened (all resolved):**
  - `preflight.rs` (retrieval): Safety SQL `:290–305`; HOTSPOT suppress `:325–327`; skip-set `:345–349`; emit-if-empty `:351–376`; `query_ledgerful` `:695`; caps/tags. GLOB-like present in `session_chrome.rs:52–88` (precedent, not reused — `index_marker_glob_sql` includes `DECISION:`).
  - `safety.rs` (CLI): `fetch_hotspots_json` `:102–128`; `main.rs` `SafetyCommands::Sync` `:2947–2956` (`--limit` default 5, `--dry-run`).
  - `preflight.rs` (CLI): `in_context_hotspots` `:50`; counts `:886–888`; `after_help` at `main.rs`.
  - Contracts: `PreflightResponse` `{text, word_count}` + `sections[]` — no new keys.
  - Tests: `preflight_global_isolation.rs:415` (T272 AC capped_out_safety), leading-CONSTRAINT fixtures; `preflight_pretty_readability.rs:266/:292/:532/:553` (summary smoke+compact, Bearings-absent asserts).
- **Pins (lockfile, today):** clap **4.6.1** (crates.io 4.6.6, no clap 5); rusqlite **0.39.0** (0.40.2 not bumped); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); uuid **1.23.1** (1.25.0). Workspace **0.1.1**, rustc **1.95.0**, nextest **0.9.140**. All match plan; no bumps.
- **Research:** SQLite GLOB/LIKE semantics (`sqlite.org/lang_expr.html` §5) and the LIKE/GLOB prefix optimization (`sqlite.org/optoverview.html` §5) confirm leading-prefix GLOB is both correct and index-eligible; clig.dev human-first empty guidance corroborates F3. No new crates needed.
- **ai-brains / ledgerful (this session):**
  - `ai-brains preflight --summary`: Pinned **3512**, in-context **0/0/0**, grants **0 of 3** — capture independence holds.
  - `ai-brains safety sync --dry-run`: **5** paths — `project.rs` (0.05), `sync.rs` (0.04), `forget.rs` (0.02), `context.rs` (0.01), `governed_common.rs` (0.01) — matches the spec table.
  - `ledgerful hotspots --json --limit 5`: `displayScore` **3.934949** for `project.rs` (spec snapshot said 3.944 — single-point drift, cosmetic); raw `score` 0.050159533 → `{:.2}` = `0.05` == dry-run rendering.
  - `ledgerful ledger status --compact`: 0 pending / 0 drift. `ledgerful doctor`: ready (4 warnings, 1 optional).
  - `ISSUES.md` confirmed absent (F24).

## Verdict: **Planned**

The plan is accurate against live `src/` and current pins/docs; the two m's are cosmetic (test-name style, a display-score single-point drift) and belong in the Phase-0 re-read, not a re-plan. The GLOB-vs-LIKE mechanism, live-inject argv reuse, hermetic escape hatch (F13/AC8), and JSON-surface freeze are all sound. Ready for `/implement-track` on **go** after Phase 0 re-verify.
