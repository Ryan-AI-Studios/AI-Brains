# OpenCode Review — T272 Preflight `--global` Safety skip vs Index

- **Harness:** opencode
- **Review date:** 2026-08-20
- **Audit target:** `conductor/tracks/trackT272-preflight-global-safety-index-skip/spec.md` (F0–F25 / AC1–AC11) + `plan.md`
- **Category:** BUGFIX / UX
- **Status:** **Planned** (Pending in registry; plan-only until go)
- **Scope:** Plan audit ONLY. No fold-in, no implementation, no edits to spec/plan/conductor/deferred/product `src/`.
- **Baseline:** live `src/` at HEAD `9fcfcd8` (`docs(conductor): plan T272 ...`; parent `9008074` = T269 #186).

## Summary

The T272 plan is a correct, well-bounded diagnosis of a real `--global` preflight defect: the Index/Recent skip set is built from the **pre-cap fetch** (`LIMIT 40`) rather than the **emitted** Safety set. Every core claim was verified against live source this pass:

- `safety_ids.insert(memory_id)` fires on **every** fetched Safety row, at `crates/ai-brains-retrieval/src/preflight.rs:329`, before `dedup_hotspots_keyed` (`:335`) and before `take_round_robin` (`:337–342`, per-project 2 / max 8).
- Index skip `if safety_ids.contains(&memory_id)` at `preflight.rs:467` hides any CONSTRAINT that lost the round-robin slot.
- `ledgerful search --json -- "safety_ids"` (run this pass) reports exactly `:286 / :329 / :467` — matching the spec's claim.
- AC10 hermetic fixture (`preflight_global_isolation__three_a_one_b__b_appears_a_capped` at `preflight_global_isolation.rs:334`) is real and, per `take_round_robin` semantics in `preflight_global.rs:14–50`, caps A-one out of Safety: round-robin output = A-three, B-only, A-two. A-one is the capped needle. Recency analysis in spec §5.3 confirmed against `ORDER BY m.updated_at DESC`.

The root cause is that `memory_id` is inserted into the skip set at `:329` and then **dropped** (`safety_raw` is `(content, ts, project)`). The fix — carrying `(project_id, memory_id)` as the extra `T` through dedup/round-robin and rebuilding the `HashSet` post-pipeline — is the minimal correct shape, and it resolves both the cap hole (#179) and the latent post-dedup over-exclude in one SOOT.

Isolation constraints (no `cargo install`, no `.env` write, no leftover drop, no `AI_BRAINS_GOVERNED_BRIEFING`, no CLI hotspot growth, no cap retune) are respected in both spec §2.1 and plan Phase 0/4. The "absorb but do not steal" boundary (T264 caps/LIMIT/LIKE/span, T265 json-v2/T180 2-key, session `HOTSPOT:` content skip, T270/T273 F7, clap 5/rusqlite 0.40) is consistent across spec §9, deferred.md `:346`/`:586–600`, and plan.md.

**Verdict: Planned** — no B (blocker) or M (major) findings. Three m/minor + O notes below are non-blocking and ready to fold.

## 2. Findings

### B (blocker) — none

### M (major) — none

### m (minor)

1. **m — Plan preflight HEAD `9008074` predates the plan's own commit.** `plan.md:14` and `spec.md:38` say `HEAD = 9008074` (T269) / `main == origin/main` / tree CLEAN. Live tree at audit time is `9fcfcd8` ("docs(conductor): plan T272 …"), a docs-only commit **ahead 1 of `origin/main`**. Non-material: the plan was written before its own planning commit landed and the diff (`conductor.md`/`deferred.md`/spec/plan) touches no product `src/`. Harmless, but reviewers should not treat the "tree CLEAN" claim as current at go.

2. **m — AC3 is listed as a Phase-1 "red" test but is likely a green-guard on the current tree.** AC3 asserts a project-scoped (no `--global`) run shows both CONSTRAINT pins in Safety and neither in Index. On the current tree, the project-scoped fetch is `LIMIT 10` with no round-robin, so the skip set (`safety_ids` pre-cap fetch) approximately equals shown ids. The plan's own language in Phase 1 says "Prove they fail on current tree (A-one missing from Index…)" — that proof only holds for AC2. AC3 may already pass today, which would make it a regression guard rather than a red test. Non-blocking; implementer should verify on current tree before Phase-1 red. If it is already green, it is still worth keeping as a guard but should not be claimed as a red test.

3. **m — The retrieval unit AC1 needs a concrete failure claim.** AC1 is specified as "the skip set must contain only the kept id (not the dropped duplicate)". The test writes against `dedup_hotspots_keyed` output or the rebuild. The spec §7 says "write the unit against a helper or against `dedup` output". That is sufficient for red-first (the unit on current `dedup` + pre-cap `safety_ids` semantics would fail). No issue — informational only, confirming the red is provable.

### O (opportunity)

1. **O — Consider asserting the exact capped needle string.** AC2 rightly asserts "A-one" present in Index, which the spec recommends over a count. Good. Could also assert that `A-one` appears in Index but *not* in Safety, which AC2 already covers. No change needed.

2. **O — Optional `emitted_ids` helper.** F9 allows a 5-line helper next to the rebuild. The plan keeps the diff minimal; an extraction helper would improve readability but is explicitly optional. Keep as optional.

## 3. What looks solid

- The root cause analysis is exact and reproduces the Bugbot #179 observation (`safety_ids` inserted at `:329` before `dedup` `:335`/round-robin `:337`; Index skip `:467`). Verified live.
- Fix shape (carry `memory_id` in extra `T`, rebuild post-pipeline) is minimal and does not touch caps, LIKE, tags, or span formula (F4/F9/F24).
- AC2's recency analysis in §5.2 exactly matches `take_round_robin` output on the AC10 fixture (A-three, B-only, A-two). Confirmed.
- `safety_for_skip` (session CONSTRAINT skip) stays post-cap — `:346–350` builds from emitted bodies, session skip at `:404`/`:409` uses that. F5/AC11 preserved.
- HOTSPOT-suppress `continue` at `:325–327` stays before insert — F6 respected; those ids are never in the skip set (pre and post fix).
- `GLOBAL_*` constants (FETCH 40/80, per-project 2/3/1, max 8/15/3/40) untouched. Verified `preflight_global.rs:97–106`.
- Hermetic evidence: `safety_section()` helper exists (`:122`); `index_section()` does not exist yet — plan correctly adds it. AC10 fixture confirmed at `:334`.
- Pins verified: lock clap 4.6.1 / crates.io 4.6.6; serde_json 1.0.150 / 1.0.151; rusqlite 0.39.0 / 0.40.2; tokio 1.52.3 / 1.53.1. Plan's claims all match (lock vs ecosystem). No bumps proposed. Correct.
- Deferred absorb/decline table matches deferred.md exactly (mint row `:346`, absorb block `:586–600`). `ISSUES.md` confirmed absent.
- No `cargo install` / `.env` / leftover-drop / `AI_BRAINS_GOVERNED_BRIEFING` / T240 F2 / T255 — all fenced.
- DoD / AC mapping is complete AC1–AC11; span `N` not frozen (F24) — correct per AC7.

## 4. Deferred fold-in table

| Item | Severity | Disposition at fold | Note |
|------|----------|---------------------|------|
| 1 | m | Folded | Not a plan defect. Rebase plan's HEAD snapshot `9008074` -> `9fcf1a` and its "CLEAN" claim at go. |
| 2 | m | Folded | Ensure AC3 red/green verified before Phase-1 red claim; keep as guard if green. |
| 3 | m | No change | AC1 failure proof is available; leave red. |
| 4 | O | No change | Optional `emitted_ids` helper; keep optional. |

None blocked, none rejected. Defer table empty.

## 5. Last-PR Cursor comments

- `gh pr list --state merged --base main --limit 1` → **#186** (T269), merged 2026-08-20. Comments **0**, reviews **[]** → **N/A** (empty), matching plan.
- No open PR on `main` — open PRs are Dependabot remotes (actions/checkout, chrono, rusqlite 0.40.2, tokio 1.53.1, tower-http …). No T274. Matches plan claim.

## 6. Research / tools notes

- **Standing-order tools (this lead):** `ledgerful doctor` ready (5 warns); `ledgerful ledger status --compact` 0 pending / 0 drift; `ledgerful search "safety_ids"` -> `:286/:329/:467`; `ai-brains recall` (T264 audit session noise only; the #179 Bugbot body + code are SoT). No `ISSUES.md` (absent — deferrals go to `deferred.md`, confirmed). All tool calls explicit; `rg` unavailable — used `Select-String`/`Get-Content` per constraint.
- **Pins (Cargo.lock / crates.io, this lead):** clap 4.6.1 / 4.6.6; serde_json 1.0.150 / 1.0.151; rusqlite 0.39.0 / 0.40.2; tokio 1.52.3 / 1.53.1; rustc 1.95.0 (edition 2024). Plan snapshot **correct**; no bumps proposed; `cargo install` fenced.
- **Hermetic fixture evidence:** `take_round_robin` confirmed at `preflight_global.rs:14–50` (first-seen project order then round i-th from each bucket, stop at max_total). A-one capped confirmed. Safety LIMIT 40 (global `:295`) vs project LIMIT 10 (`:303`); assert `GLOBAL_SAFETY_FETCH == 40` (`:305`).
- **Fix-shape check:** carry `(Option<String>, String)` extra; key = `project_key(pid)`; rebuild HashSet; remove fetch-loop insert; keep HOTSPOT-suppress before push; keep `safety_for_skip` post-cap. All consistent with current destructuring (`(content, ts, project)`).
- **Not done:** `cargo build` / `cargo nextest` (plan-only audit — no code change; the existing suite must run at implement). No file was edited or created except this harness file.

## 7. Verdict

**Planned** — no blockers/majors. 2 minor + 1 O informational notes for the fold (HEAD snapshot refresh; AC3 red-vs-guard classification; optional helper). **Recommend `/fold-in T272`.**
