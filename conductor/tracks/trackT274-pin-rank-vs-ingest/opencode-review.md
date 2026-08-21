# T274 — Pins vs harness ingest ranking — OpenCode review

Reviewed: 2026-08-21 · HEAD `9a99117` · clean tree · branch `main`
Scope: plan audit only (no implementation, no folding). Harness file — not `review.md`.

## Summary

T274 is a well-scoped, root-cause-correct plan for the daily-recall hole: harness session
dumps (review-track `## Objective`, JSON summaries, chat crumbs) outrank `DECISION:` /
`CONSTRAINT:` / `INVARIANT:` pins because (a) T211 F4 classifies anywhere-in-body markers
(JSON `"decisions": [` gets Decision +2), (b) `candidate_depth(5)=15` can be all chrome so
`rerank_hits` never sees the pin, and (c) the preflight Index is a pure recency scan with no
rank. The plan's fix is three-pronged and internally consistent: **leading-line classification
(F2/F3) + session-chrome penalty (F6) + lexical/Index two-pass (F7/F11) + first-line chrome
collapse (F10)**. Every code claim I could verify against live `src/` checked out; the
penalty math is sound; the isolation rules protect the hotspots; the research is current.

Verdict: **Planned** — no B/M findings. Two m/O drift notes (HEAD staleness; live preflight
counts moved 0/0/0 → 1/1/1 since the plan's dogfood).

## Findings

### Blockers (B)
None.

### Mediums (M)
None.

### Minor (m)
- **m1 — HEAD staleness (plan predates its own docs commit).** Plan §2.1 claims HEAD
  `deabae7` (mint T274–T284 placeholders), CLEAN, `main` ahead of `origin/main` by 1.
  Live: HEAD = `9a99117` (`docs(conductor): plan T274 pins vs harness ingest ranking`),
  parent `deabae7`; `origin/main` = `14d42af` (T270 #188). Same class of m-note as the
  T270/T272 reviews. Product `src/` is unchanged since `14d42af` (verified:
  `git diff 14d42af HEAD --stat` on the four named files is empty), so the plan's code
  baseline is intact. Phase 0 re-verify already covers this.

### Opportunities (O)
- **o1 — Live preflight counts drifted.** Plan §2.1 dogfood: `preflight --summary`
  in-context hotspots/decisions/constraints = **0/0/0**. Live re-run (2026-08-21):
  **1/1/1** (Pinned 3297, Active sessions 3, grants 0 of 3). T270 merged since the plan's
  dogfood and its review-track Objective is now a pinned memory. Still a hole (1 decision
  vs 3297 pins) — the plan's "0" is a stale snapshot, not a wrong conclusion.
- **o2 — Hotspot score drift.** Plan §2.1: `project.rs` #1 at 3.990. T270 review measured
  3.999. Cosmetic; #1 unchanged.
- **o3 — `classify_pin_kind` test-suite claim.** Plan §8 risk says "update only tests that
  asserted anywhere-in-body (none found besides leftmost-on-leading-line)". I verified the
  named T211 test `rerank_hits__plan_below_shipped_same_track__ac1` (ranking.rs:458) uses a
  leading `DECISION:` and is consistent with the F2 lift, but the full retrieval test suite
  was not exhaustively audited for buried-marker assertions. Phase 0 re-read + AC1 required-red
  covers this; flagging for the implementer to grep `classify_pin_kind` callers/tests on go.

## What looks solid

- **Root cause is correct and code-verified.** `classify_pin_kind` (ranking.rs:84) is
  anywhere-in-body leftmost (`lower.find(needle)` over the whole body after one
  `ASSISTANT: ` strip) — so JSON `"decisions": [` and skill-body mentions get Decision +2.
  `INVARIANT:` is absent (F3 claim accurate). `rerank_hits` (ranking.rs:248) is the single
  post-blend entry point (F40), and `recall.rs:510-514` runs `rerank_hits` → `dedupe_symbol_stubs`
  → truncate, matching the plan's pipeline claim.
- **Two-pass (F7) directly answers the T260 lesson.** `candidate_depth` (hybrid.rs:20) is
  `limit.saturating_mul(3).clamp(15, 50)` with tests at :374-379. If MATCH LIMIT 15 is all
  chrome, `rerank_hits` cannot surface a pin that never entered — the plan's pass-1 authority
  GLOB + pass-2 fill is the correct remedy, and "if pass 1 is empty, pass 2 is today's
  behavior" preserves T207 empty honesty.
- **Penalty math checks out.** F6 `SESSION_CHROME_PENALTY = 16.0` (same scale as
  `SYMBOL_PENALTY`, symbol_stub.rs). Plan §5.4: chrome BM25 −12 → base 12 + recency ~1 − 16
  ≈ −3 vs a DECISION at ~5 (base 2 + KIND_DECISION 2 + recency 1). Pin wins if present;
  two-pass is what makes it present. KIND_* magnitudes frozen (F4) is the right call given
  the Elastic Labs 2025-12 additive-boost-brittleness research.
- **GLOB-as-subset + detector-as-SoT (F8) mirrors the proven T260 F19 pattern.**
  `symbol_stub_sql_exclusion` (symbol_stub.rs:66-74) is bind-free, identifier-checked, GLOB
  prefix; the plan's authority GLOB (§5.2) is the same shape. Lowercase `decision:` caught
  in-memory by the detector.
- **Preflight claims all verified.** Index loop (preflight.rs:437-507) is a recency scan with
  `safety_ids` skip (:470-473, T272) and `GLOBAL_INDEX_FETCH` = 80 (preflight_global.rs).
  Safety SQL (:288-303) is LIKE-anywhere LIMIT 10/40 — correctly declined to T279, not
  retuned. CLI summary counts (:886-888) are `text.matches("DECISION:")` on the assembled
  window — F12 keeps them, so a pin in Index ⇒ `in_context_decisions >= 1`.
- **Contracts discipline.** `RecallResult` (contracts/recall.rs:18-36) has only additive
  optional fields (`staleness`, `score_kind`, `cosine`); no `is_session`/`is_authority` wire
  key (F16). AC13 freezes this.
- **Isolation and hotspot protection are explicit.** Do-not-touch list covers `project.rs`
  (#1), CLI `preflight.rs` (#7, 2027 lines), `sync.rs` (#2), `forget.rs` MATCH, contracts
  required keys, `class_based_retention.rs`, live `.env`. F22 stops live-vault pinning;
  hermetic needle is the proof. F18 keeps `forget --match` unfiltered (verified:
  forget.rs:89,197 uses `lexical_search`).
- **Dependency research is current and verified live.** Cargo.lock: clap 4.6.1, serde_json
  1.0.150, chrono 0.4.44, rusqlite 0.39.0. crates.io today: clap 4.6.6, rusqlite 0.40.2,
  chrono 0.4.45. Plan's "no bump" is correct; Dependabot PRs #61/#62 exist but are correctly
  declined.
- **Line counts verified canonically** (`git show HEAD:`): ranking 858, recall 861,
  retrieval preflight 1003, CLI preflight 2027 — exact match.
- **Live dogfood reproduces the hole.** `ai-brains recall "what did we decide about
  retention" --no-bridge --limit 5` returns T248 reviews (BM25 −8.67/−8.65), a JSON
  `"decisions": [` summary, the T270 `## Objective`, and a chat crumb — no pin in top-5.
  The plan's §2.1 table is accurate.
- **AC1–AC17 are concrete and testable** with required-red markers (AC1/AC3/AC4/AC6/AC14),
  hermetic fixtures, and a unit-only semantic arm (AC16) that avoids HTTP flake.

## Deferred fold-in table

| Item | Disposition | Verified |
|------|-------------|----------|
| recall/search/semantic/preflight Index/summary dumps over pins | **Absorb** F1–F12 / AC1–AC7 / AC14–AC15 | deferred.md:11,28; plan §9 |
| `memory list` just-now ingest | **Partial F13** — recency stays (query_store.rs:240 `updated_at DESC, memory_id ASC`) | deferred.md:29 |
| `sync query` vault dumps | **Absorb F14** via `recall_full` (sync.rs:487) / AC15 | deferred.md:30 |
| Preflight Safety = `## Objective` | **Decline F23 → T279** | deferred.md:33 |
| briefing/progressive POLICY_DENIED | **Decline F24 → T275** | deferred.md:34 |
| leftover `7d97a456` / `--global` junk | **Decline F24 → T276** | deferred.md:35 |
| #188 Work hides CE / apply samples | **Decline F26 → T284** (last-PR Cursor) | deferred.md:36 |
| T260 symbol monopoly | **Affirm F / AC9** — do not reopen | plan §9 |
| T211 F4 leftmost-anywhere | **Lift F2** (this is the hole) | plan §9 |
| T211 F9 KIND_+2 | **Affirm F4** — do not bump | plan §9 |
| T211 F25 vault↔ledger RRF | **Decline F25** | plan §9 |
| T218 floors / ANN | **Affirm F17** | plan §9 |
| T216 list ORDER | **Affirm F13** | plan §9 |
| T264 leftover recall drop | **Decline** (F11 there) | plan §9 |
| T240 F2 / T263 H2 / T255 750 ms / T266 JSON | **Decline F25** | plan §9 |
| clap 5 / rusqlite 0.40 / DTO / `cargo install` | **Decline F20 / F21** | plan §9 |
| last-PR Cursor #188 | **T284** — still true on `14d42af`; not this track | plan §9 |
| Open PR on HEAD | **N/A** — none (Dependabot remotes) | plan §9 |

No new fold-in items from this review. The plan's §9 table is complete and matches
deferred.md.

## Last-PR Cursor comments

PR #188 (T270, merged 2026-08-21T02:42:31Z, squash `14d42af`). Cursor Bugbot review
(commit `fd5b274`, 2026-08-21T02:29:54Z) found **2 Mediums**, both verified live via the
GitHub API:
1. **Work table hides dispose rows** — `Nothing to dispose.` keys off
   `would_ce_wipe + would_projection_delete` but the Work table only prints classes whose
   dominant mechanism is `ce_wipe`/`projection_delete`; R11 pin-holds keep `held` candidates
   in the same class, so `dispose_work > 0` can print an empty Work header.
2. **Apply audit samples prefer inventory** — the `memory_legacy` overlay + name-sorted
   classes makes `append_retention_applied` fill capped `sample_ids` from inventory pins,
   omitting the content keys/turns actually disposed.

Both are correctly routed to **T284** (deferred.md:21,36) and are out of T274 scope (F26).
No new Cursor leftover to mint. Open PRs are Dependabot-only (actions #68-72, cargo #58-62).

## Research / tools notes

- **crates.io (live, via API):** clap **4.6.6**, rusqlite **0.40.2**, chrono **0.4.45** —
  exact match with plan §2.4. No clap 5. No bumps warranted.
- **Cargo.lock:** clap 4.6.1, serde_json 1.0.150, chrono 0.4.44, rusqlite 0.39.0 — match.
- **ledgerful:** `ledger status --compact` = 0 pending / 0 unaudited drift. `doctor` = 5
  warnings (legacy `.changeguard`, sig-pin, timings, :8081, gemini not found) — ready for
  publish env. `ISSUES.md` does not exist (F29 accurate).
- **Live dogfood:** `preflight --summary` = Pinned 3297, Active sessions 3, in-context
  1/1/1 (drift o1), grants 0 of 3 (T275). `recall "what did we decide about retention"`
  reproduces the hole exactly (T248 reviews + JSON summary + T270 Objective + chat crumb).
- **gh:** #188 merged, Cursor Bugbot 2 Mediums → T284. Open PRs Dependabot-only.
- **Line counts:** canonical `git show HEAD:` method (PowerShell `Get-Content .Count` is
  unreliable — inflated; not used).

## Verdict

**Planned** — with m1 (HEAD staleness, same class as T270/T272) and o1–o3 (live-count drift,
hotspot score drift, test-suite re-check on go). No B/M findings; no re-plan needed. The
plan is ready for `/fold-in T274` and, on user go, implementation.
