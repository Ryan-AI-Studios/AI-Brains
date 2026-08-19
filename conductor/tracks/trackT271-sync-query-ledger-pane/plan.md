# T271 Plan — sync query ledger pane

**Status:** **Pending** (Planned+fold-in; plan-only until go)
**Spec:** [spec.md](./spec.md) F0–F23 / AC1–AC19 + §13 fold-in
**Category:** BUGFIX / UX
**Ledger TX (planning):** `68c42d13-b398-4c36-8d1b-8fc74d3b6516` (DOCS)
**Ledger TX (fold-in):** `5eb051c2-abb3-4ab6-95f5-80bd12167d19` (DOCS)
**Ledger TX (on go):** FEATURE — open then

---

## AI fold-in (2026-08-19) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. Both verdicts **Planned**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **AC17 / F18:** `ledger_miss_copy__empty_query__did_not_run` + AC1 empty-forward.
2. **AC18 / AC19 / F19:** pure classifier; git/work-directory/layout → never-ran; other nonzero → failed; first stderr line, **140** chars.
3. **F6:** `use ai_brains_core::{contentful_tokens, extract_fts_tokens}` (retrieval re-exports only `sanitize_fts_query`).
4. **AC15:** `ledger search capture` ≥1 (count is volatile: plan 5 / fold-in 9).
5. **F10:** `pub mod sync_query_ledger` already required (Agy m2).

---

## Preflight (plan time — 2026-08-19)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `e48eaa7`. Fold-in HEAD `33f72cf` (plan commit). Local `main` 1 ahead of `origin/main`. |
| T271 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-18. **Do not `cargo install`.** |
| Live hole | `sync query "capture independence" --quiet` → vault hits + `No ledger entries found matching '"capture" "independence"'` |
| Ledgerful control | `ledger search capture` ≥1 (plan **5**, fold-in **9**, volatile); phrase / quoted phrase = `[]`; `OR` argv = `[]` (phrase wrap) |
| SoT | `sync.rs:584` `probe_ledger_search`; `sanitize_fts_query`; Ledgerful `search.rs:26` `format!("\"{query}\"")` |
| Hotspot | `sync.rs` **#2** (786 lines) → sibling `sync_query_ledger.rs`. `project.rs` #1 — do not edit |
| clap / serde_json | lock clap **4.6.1** / crates.io **4.6.6**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute |
| Last PR Cursor | #182 comments/reviews **empty**. #179 Bugbot Medium still **T272**. No open PR on `main` |
| `deferred.md` | Full scan. Overlap: audit T271 **absorb**; T90 argv **lift**; T91 **affirm**; T211 F12 display **partial**; T217 MATCH / T268–T270 / T272 / T240 F2 / T255 **decline** |
| ai-brains | `preflight --summary` Scope `3581317d`; pins **volatile** (~3099); grants 0 of 3 (T241) |
| ledgerful | doctor ready (hygiene warns). 0 pending 0 drift at scan. Index incremental completed. TX `68c42d13` |
| Research | sqlite.org/fts5.html phrases + boolean; clig.dev honesty; T217 sequential rescue pattern; live Ledgerful wrap |
| `ISSUES.md` | **Does not exist** |
| Live `.env` / bootstrap / nightly mutate / pin | **Not written** / **not run** / **not scheduled** / **not pinned** this pass |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Ledger pane false-empty (5/5) | audit T271 | **DoD** F1–F7 / AC1–AC9 / AC13 |
| never-ran vs ran-empty copy | stub F1 | **DoD** F1 / F8 / AC6 / AC8 |
| System32 cwd | stub F2 | **DoD** F2 / AC7 (guard; not live repro) |
| `--no-bridge` | stub F3 / T124 | **Affirm** F3 / AC11 / AC14 |
| Vault independence | stub F4 | **Affirm** F4 |
| T90 on ledger argv | T90 | **Lift** F5 — vault MATCH keeps sanitize |
| T91 ANSI | T91 | **Affirm** F5 / AC2 |
| T211 F12 vault-only on fail | T211 | **Partial** F9 |
| T211 F25 blend | T211 residual | **Decline** F11 |
| T217 vault OR | T217 | **Decline** — pattern only |
| T268 / T269 / T270 / T272 | series | **Decline** F12 |
| T240 F2 / T255 bag | standing | **Decline** F12 |
| last-PR Cursor #182 | empty | **N/A** |
| F18 empty-query unit | OpenCode m | **Folded** AC17 + AC1 empty-forward |
| F19 classifier + 140 cap | OpenCode m / O | **Folded** AC18 / AC19 / F19 |
| `ai_brains_core` FTS import | OpenCode O | **Folded** F6 / Phase 2 |
| Agy m2 `pub mod` | Agy | **Already** F10 |

---

## Phase 0 — on go (re-verify)

- [ ] Re-read `probe_ledger_search`, `sanitize_fts_query`, Ledgerful `search.rs` phrase wrap.
- [ ] Confirm source `sync query "capture independence"` still prints `'"capture" "independence"'`.
- [ ] Confirm `ledgerful ledger search capture` still ≥1.
- [ ] Confirm T272 still at retrieval `preflight.rs:329` + `:467`.
- [ ] Re-check lock clap **4.6.1** / crates.io clap. rustc **1.95.0**. No clap 5. serde_json lock vs crates.io.
- [ ] Rescan **entire** `conductor/deferred.md`.
- [ ] Last merged PR Cursor comments — leftover none or mint.
- [ ] `ledgerful ledger start T271-sync-query-ledger-pane --category FEATURE`

---

## Phase 1 — red (failing units)

- [ ] Add `pub mod sync_query_ledger` (empty module ok if tests compile against the planned fn names).
- [ ] `ledger_forward_query__user_phrase__not_fts_quoted`
- [ ] `ledger_forward_query__empty__returns_empty`
- [ ] `ledger_forward_query__ansi_stripped`
- [ ] `ledger_rescue_tokens__capture_independence__first_seen_capture`
- [ ] `ledger_rescue_pick__first_token_empty_second_hits__selects_second`
- [ ] `ledger_miss_copy__ran_empty__uses_user_query_not_quotes`
- [ ] `is_windows_system_cwd__system32_and_syswow64__true`
- [ ] `ledger_miss_copy__never_ran__did_not_run`
- [ ] `ledger_miss_copy__empty_query__did_not_run`
- [ ] `ledger_classify_outcome__nonzero_git_stderr__never_ran`
- [ ] `ledger_classify_outcome__nonzero_other_stderr__failed`
- [ ] `ledger_rescue_banner__phrase_empty_token_hit__locked_sentence`
- [ ] Commit red allowed.

---

## Phase 2 — green (sibling + dispatch)

- [ ] Implement forwarder (strip_ansi + trim only).
- [ ] Implement miss copy + System32 predicate.
- [ ] Move `probe_ledger_search` + `ledger_json_non_empty` + their tests into the sibling.
- [ ] Token rescue F6 (max 3, first-seen, `--json` then human of winner). Import `ai_brains_core::{contentful_tokens, extract_fts_tokens}` — not retrieval.
- [ ] F19 classifier + first-line 140-char cap (local; do not import `project.rs::truncate_chars`).
- [ ] `sync.rs` print: F7 banner; F1 miss lines; F8 quiet; T211 reorder uses rescued `non_empty`.
- [ ] No `sanitize_fts_query` on the ledger argv.
- [ ] No production `unwrap`/`expect`/`panic`.
- [ ] PowerShell `;` only in any docs snippets.

---

## Phase 3 — stay green

- [ ] `ledger_json_non_empty` units (AC10).
- [ ] `sync_query__no_bridge__skips_ledgerful_section` (AC11).
- [ ] T211 `sync_query_ranking` + T231 `sync_query_ux` hermetics (AC12).
- [ ] Grep CLI tests for `'"capture" "independence"'` snapshots — update only if they encode the bug.
- [ ] `cargo nextest run -p ai-brains-cli` targeted; `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`.

---

## Phase 4 — docs + closeout (on go)

- [ ] CAPABILITIES honesty bullet (phrase + rescue + miss classes).
- [ ] CHANGELOG T271 row.
- [ ] Manual AC13–AC15 (`cargo run`, not PATH unless user installed).
- [ ] `conductor.md` stays **Pending** until implement closeout → then Completed.
- [ ] Append residuals to `deferred.md`.
- [ ] Primary review → `review.md`. Cross-model optional (F20).
- [ ] FEATURE TX commit. Push **branch** + PR only from **implement-track**. Never `git push origin main`.

---

## Definition of done

- [ ] AC1–AC12 + AC17–AC19 automated green.
- [ ] AC13 live dogfood: no T90-quoted empty chrome; ≥1 ledger hit or F7 banner+token hits.
- [ ] AC14 `--no-bridge` still vault-only.
- [ ] AC16 docs.
- [ ] No product `unwrap`/`expect`/`panic`.
- [ ] `sync.rs` not grown with the new helpers (sibling exists).
- [ ] Medium+ review findings not silently dropped.
- [ ] Ledger FEATURE TX committed; no pending drift.

---

## Isolation (do not do)

- Edit Ledgerful sources or `fts.rs` sanitizer body.
- Edit retrieval `preflight.rs` (T272).
- Grow `project.rs`.
- `cargo install`, live `.env` write, `policy bootstrap` (non-dry), nightly/schtasks mutate.
- clap 5 / new crates / contracts DTO / `schema_version` on this pane.
- Reopen T240 F2 / T255 declines / T268–T270 / T272.
