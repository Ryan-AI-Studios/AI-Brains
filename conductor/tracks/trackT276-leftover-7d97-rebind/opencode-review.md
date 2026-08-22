# Track review: T276-Leftover7d97Rebind

**Harness:** OpenCode (`opencode`)
**Track:** `conductor/tracks/trackT276-leftover-7d97-rebind`
**Date:** 2026-08-21
**HEAD:** `61fd3cb`

## Summary

T276 closes the daily-product hole from the 2026-08-21 CLI audit: `recall --global` scores **7/3** because the leftover identity `7d97a456` (~18,038 pins across 11 `C:\dev\*` roots) monopolizes the unscoped FTS5 `MATCH LIMIT candidate_depth(5)=15` window, and `--global` hits carry **no project tag**, so agents cannot tell this repo's pins from leftover dumps.

The plan does the right thing by refusing every unsafe escape hatch:
- **Prefer-fill, not exclude.** When `--global` runs from an active project, `recall_full` issues a second `lexical_search` scoped to the pre-clear effective project and merges it ahead of the unscoped global set (preferred first, then global minus seen ids, truncate to `depth`, then the existing `rerank_hits`). Leftover is never filtered (T264 F11), no `AND project_id != leftover`.
- **Label, don't hide.** Pretty `--global` hits get the T264-class `[8hex]` / upgraded `display_label` leading tag. Project-scoped recall stays untagged (AC9).
- **No mutation.** No memory reclassify/`MemoryMoved` (T259 F5), no live `rebind-path --write --yes` (F9 Stop-Before), no `.env` write (T240 F2), no `--exclude-project` flag (F20), no DTO key growth (F5, JSON E1 frozen), no clap 5 / rusqlite 0.40 / new crates (F21), no `ranking.rs` / `project.rs` growth (F14/F18), `candidate_depth` untouched (F13).

Live dogfood independently confirms the gap is real: `ai-brains recall "what did we decide" --global --limit 3 --pretty --no-bridge` returns hit #1 = T263 DECISION then unlabeled dumps — exactly the "unlabeled" claim in the plan.

The one point of care — the load-bearing F17 "pre-clear effective project" wiring — survives verification: `.env` force-set (`main.rs:3025`) runs before clap binds `--project-id` from `AI_BRAINS_PROJECT_ID`, so in the daily scenario (`recall "…" --global` with `.env`/`--project-id` present) the pre-clear id is populated before the T112 clear at `main.rs:4322`. I've noted the residual (see **B2** below) for the implementer to re-verify on `go` and to pin into the plan table; it does not rise to **M** given the AC2 red-test is mandatory, F2 explicitly derives the id from "CLI `project_id` before T112 clear (env / AppContext effective)", and the plan already carries both documents.

**Verdict: Planned.** No B, no M. Two low-info residual notes (B2, B3) to fold into the Phase-0 checklist at go-time.

## Findings (B/M/m/O)

### Blockers (B)

None.

### Major (M)

None.

### Minor (m)

- **m1 — Prefer-fill redesign vs spec baseline**: The design fell back to **in-memory merge of two `lexical_search` results** (each carrying the T274 two-pass) rather than the SQL-level route in the delivered spec. This is the stronger choice: it changes only `RecallOptions`/`RecallHit` plumbing and the merge in `recall_full`, keeps `lexical.rs``s two-pass and `project_id` filter untouched, avoids touching `match_sql_and_params` at all (lexical SELECT from `:259–265` keeps its column list — no COALESCE needed), and the "second lexical_search" is exactly a *second call to the existing public* `lexical_search` (live at `lexical.rs:57`, `pub fn`, signature `(conn, raw_query, project_id: Option<ProjectId>, session_id: Option<SessionId>, opts: LexicalSearchOptions)`). This requires **one small `spec.md` touch at fold-in**: the implementation route is now "`RecallOptions.preferred_project_id` → second `lexical_search` call → merge module → rerank", not the "SELECT COALESCE" route. Low-risk, on-scope, and doesn't affect any frozen decision.
- **m2 — `recall_full` bridge-call signature.** The `lexical_search` call at `recall.rs:283-305` passes `project_id` as the "seed" option `recall_full` receives; the bridge call at `:275-276` also uses `project_id`. With `--global` the seed is `None` today; the plan's merge adds a `preferred_project_id` carrying the id while `project_id` stays `None`. During implement, confirm the "authority prefer for the preferred pass" and that `LexicalSearchOptions { prefer_authority: true }` remains set on both passes (the existing flag already applies `prefer_authority` for the global pass too; it only matters for "targeted" helper), and confirm the bridge/SQL `project_id` filter stays `None` for global even when `preferred` is `Some` (AC5 relies on the JSON results not gaining a key, and bridge is not project-scoped).
- **m3 — Ranking/prefer interaction is intra-merge.** The plan and this review both hold that `rerank_hits` is **not** a leftover-demotion hook (F14): prefer-fill is the only lever. That means "leftover still appears" (AC3) is only guaranteed at **candidate** level (before `rerank_hits`), and top-5 output may still be all-owner. This is intended ("label, do not drop"), but should be stated in `recall.md` (or a one-line comment in `prefer_project.rs`/`recall.rs` next to the merge) so a future reader doesn't mistake a leftover-free top-5 for a "drop regression".

### Opportunities (O)

- **O1 — Prefer-aware empty-hint.** With `--global` + `preferred_project_id = Some` and FTS empty on the preferred pass, the empty-state hint (`build_recall_hint`) currently advises "Try `--global`" — which is wrong (already global) and would send the user to the leftover-heavy path. Cheap, on-scope follow-up if seen during green.

## What looks solid

1. **Prefer-fill vs drop is the right call.** T264 F11 (label not drop) is respected; the merge only reorders + dedupes candidates, so `--global` stays all-projects. This keeps 18k historical pins reachable and never hides the dump.
2. **Dynamic preferred id.** No hardcoded leftover UUID in retrieval code (F2/F5). It works for `3581317d` (this repo) and for `fcb8a40f`/any other project when it's the cwd — leftover and non-leftover siblings alike.
3. **Priority of append-only + no mutation.** No `MemoryMoved`, no CE, no reclassify (T259 F5), no live rebind (F9), no `.env` write (T240 F2). T276 is **retrieval + pretty chrome only** (F30/F37).
4. **AC2 is a mandatory red before green** (owner pin must be #1 when preferred is set — currently it won't be since `preferred` is ignored). TDD gate is real.
5. **TDD names are concrete and hermetic**; each AC has an explicit proof; tests already cited as `preflight_global_isolation.rs` etc. are consistent with the current tree (verified).
6. **Deferred §9 is a genuine full scan** — every overlapping open row is dispositioned (absorb / partial / decline), not just the same three leftovers.
7. **Last-PR Cursor audit is correct:** `#190` (T275) has zero comments/reviews (verified via `gh api`), and `#188`'s two Bugbot Mediums are properly **T284** (not being stole from the track). Dependabot remotes `#61`/`#62` are correctly noted (not leftover for this track).

## Deferred fold-in table

| Deferred item | Spec/Plan disposition | Assessment |
|---------------|------------------------|------------|
| Leftover `7d97a456` ~18k / `--global` junk | **Absorb** F1–F6 / AC1–AC5 / AC12 | Correct; prefer-fill + tags address the root cause |
| T264 leftover-first recall / "filter flag" | **Partial:** prefer-fill + label DoD; `--exclude-project` **Decline F20** | Correct; default-right without a new clap flag |
| T259 leftover memory reclassify by path | **Decline F7** (soft residual) | Correct; append-only + `rebind-path` stays the remediator |
| Live leftover 11 roots (T270 closeout) | **Partial:** document; F9 Stop-Before | Correct; DoD is `--global` usable, not a live split |
| Identity `7d97` vs `fcb8a40f` | **Partial:** leftover volume this track; adopt-path T258; shell T282 | Correct; no new identity model / no T285 mint |
| `project list` cwd-first | **Decline → T283** | Correct (matches my T283 placeholder) |
| `context --show` shell leftover | **Decline → T282** | Correct (matches my T282 placeholder) |
| #188 Bugbot (Work-table hides dispose / samples prefer inventory) | **Decline → T284** (2 inline comments) | Correct; minted `trackT284-retention-work-samples` exists |
| T275 grants (0/3 briefing) | **Decline → T275 Completed** (#190) | Correct |
| Last-PR Cursor `#190` | **N/A** — comments/reviews empty | Verified via `gh api` (`[]`/`[]`) |

Nothing else on `conductor/deferred.md` (scanned in full) overlaps. No leftover dumped in chat.

## Last-PR Cursor comments

- **Scanned PR:** [#190](https://github.com/Ryan-AI-Studios/AI-Brains/pull/190) (merged 2026-08-21, T275 grant-wall Discovery — on `main` as of today).
- **Cursor comments:** 0 review comments (`gh api …/pulls/190/comments` → `[]`; issue comments `[]`).
- **#188 Bugbot Mediums (2):** Work-table hides dispose rows + Apply audit samples prefer inventory → **T284** (verified `trackT284-retention-work-samples` exists with plan.md/spec.md).
- **Open PRs on HEAD:** Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, actions). No open product PR → **N/A**, no mint.
- **No T285.**

## Research / tools notes

- **Multi-tenant search:** The plan correctly distinguishes **security tenants** (never leak — filter-always) from **dump identity** (leftover is a same-vault dump, filter-always would hide its only unscoped path). Prefer-fill is a query-time boost, not a confidentiality gate — matches clig.dev "make the default the right thing".
- **Ranking as policy:** Boosting/candidate-entering is the elastic/Qdrant lesson — you don't demote the dump via scoring, you get the owner pin *into* the window. This is exactly the "prefer the current tenant without hiding others" approach; no BM25 IDF retune.
- **Append-only/event sourcing:** Azure/Fowler compensating-facts-on-the-old-stream is the T259 pattern. No `MemoryMoved`.
- **Pins (verified today vs lockfile + crates.io):**
  - `clap` **lock 4.6.1** → crates.io **4.6.6** — plan pins match; **no clap 5** exists. ✓
  - `serde_json` **lock 1.0.150** → crates.io **1.0.151** — ✓
  - `chrono` **lock 0.4.44** → crates.io **0.4.45** — ✓
  - `rusqlite` **lock 0.39.0** → crates.io **0.40.2** — ✓ (no bump; SELECT column unchanged).
  - `uuid` lock 1.23.1 — not changing.
  - rustc 1.95.0 / edition 2024 / workspace 0.1.1 — unchanged.
- **Toolchain (live this session):**
  - `ai-brains preflight --summary`: **3352** pinned, scope `3581317d-…`, in-context **0/0/0**, grants **0 of 3**, active sessions **3** — exactly matches the plan preflight table.
  - `ai-brains project whoami --format json`: effective/env/path_alias/detect = `3581317d`; `shell_project_id = 7d97a51a-f2f4-43ea-1f13-211af684ad37` (leftover); `mismatch: false`. ✓
  - `ai-brains memory list --summary --global`: live **38856** (plan says 38833 — +23 drift, low-info), leftover `7d97a51a` **18038** ✓, this repo **3352** ✓, `fcb8a40f` **4875** ✓.
  - `ai-brains project list-paths --shared-only --format json`: **11** leftover roots, all `exists: true` (crawlx, dedupe, degoo, family, gimp, homebrew-tap, kinledger, ledgerful-action, ledgerful-frontend, ledgerful-web, wondermaker) ✓.
  - `ledgerful doctor`: 4 legacy `.changeguard` / sig-pin / timings / 8081-warnings; `ledgerful ledger status --compact`: **0 pending / 0 drift**.
  - `ledgerful scan --impact` and `ledgerful search` run at plan time and this review (no drift).
- **Line-count caveat:** plan numbers for `ranking.rs` **939** (disk 994), CLI `recall.rs` **1438** (disk 1539), CLI `preflight.rs` **2027** (disk 2148), `project.rs` 1332 (disk 1472), retrieval `recall.rs` 866 (disk 936) — the existing, known `Measure-Object -Line` CRLF undercount (~8%). **Hotspot relative ranking still holds** (project.rs #1, sync.rs #2, preflight.rs #7). `recall_global_prefer` test filenames: plan says `crates/ai-brains-retrieval/tests/recall_global_prefer.rs` + `crates/ai-brains-cli/tests/recall_global_prefer.rs` (new; don't exist today).

## Verdict: Planned

The plan is approved as **Planned**. On `go` (with `/implement-track`), the Phase-0 list already re-rechecks the pre-clear id and the dual-pass `lexical_search`; fold in the two checklist items **B2** and **m2** above before starting the red test. No fixes needed to the plan itself.

Reference verified live: `crates/ai-brains-retrieval/src/{recall.rs,lexical.rs}`, `crates/ai-brains-cli/src/main.rs` (`:3025`, `:4320-4334`, `:1017`), `crates/ai-brains-cli/src/commands/recall.rs` (`:166-214`, `:473-497`), `crates/ai-brains-cli/src/context.rs` (`:30-67`), `crates/ai-brains-retrieval/src/hybrid.rs` (`:20`, `:98`), `crates/ai-brains-retrieval/src/semantic.rs` (`:57`, `:127`), `crates/ai-brains-contracts/src/recall.rs`, `crates/ai-brains-store/migrations/{0006,0015}.sql`, `crates/ai-brains-cli/src/commands/preflight_pretty.rs`, `crates/ai-brains-control-plane/src/grants.rs` (`:287`), `crates/ai-brains-cli/src/commands/project_rebind.rs`, `crates/ai-brains-control-plane/src/briefings/project.rs` (`display_label` `:383`). Names and line numbers match the plan's touch map.
