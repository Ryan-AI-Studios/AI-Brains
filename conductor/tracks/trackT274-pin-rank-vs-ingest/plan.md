# T274 Plan — Pins vs harness ingest ranking

**Status:** **In Progress** (go 2026-08-21; FEATURE TX `a5e94797`)
**Spec:** [spec.md](./spec.md) F0–F36 / AC1–AC17 + §13 AI fold-in
**Category:** FEATURE / UX / RETRIEVAL
**Ledger TX (planning):** `9c1049c0-5520-430d-a1f0-01aba355082e` (DOCS)
**Ledger TX (fold-in):** `c483e45a-cf54-4d50-b15b-3d7128f9b5d0` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go**

---

## AI fold-in (2026-08-21) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F35 / AC17:** pass-2 `NOT IN` is `?` placeholders + `params_vec` (Agy m1).
2. **F36:** `authority_glob_sql(column)` helper (Agy O1 as helper, not `mp.content` const).
3. **F10 / AC5:** chrome-only first-line collapse (Agy m2 already).
4. **F7:** skip pass 2 when pass1 full (Agy O2 already).
5. **§2.1:** HEAD `9a99117`; summary counts volatile.

---

## Preflight (plan time — 2026-08-21)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `deabae7`; fold-in `9a99117` (docs-only; product tree identical to `14d42af`). CLEAN |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-21 05:55. T270 on PATH. **Do not `cargo install`.** |
| Source debug | Older (2026-08-20 22:16). Dogfood used PATH |
| `preflight --summary` | Plan 3297 / **0/0/0**. OpenCode **1/1/1**. Fold-in **3324 / 0/0/0**. Hole stands |
| `recall "what did we decide about retention"` | T248 reviews + JSON + `## Objective` — **no pin top-5** |
| `recall "DECISION: T270"` | Five Objective/review dumps; pin **not in candidate set** |
| `memory list --limit 5` | All `## Objective` / Track Plan Review / T248 Review, 18m, `status=pinned` |
| `preflight --pretty -m 400` | Safety = T272 `## Objective` (**T279**); Sessions = dumps |
| Last PR comments | #188 Cursor Bugbot **2 Mediums** → **T284** (already minted). Still true on `14d42af` |
| Open PR on HEAD | none (Dependabot remotes only) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150**; chrono **0.4.44**; rusqlite **0.39.0** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (3.990) — do not grow. CLI `preflight.rs` #7 **2027** lines — do not grow. `sync.rs` #2 — do not grow. `ranking.rs` **858**; retrieval `preflight.rs` **1003**; `recall.rs` **861** |
| Ledger | 0 pending at scan; planning TX `9c1049c0`; fold-in TX `c483e45a` |
| `ISSUES.md` | **Does not exist** (F29) |
| ledgerful search | `rerank_hits` `ranking.rs:248` / `recall.rs:512`; `classify_pin_kind` `:84` |
| Online | SQLite FTS5 BM25; ES function_score + 2025-12 multiplicative boost warning; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact`
- [x] Re-read `classify_pin_kind` / `rerank_hits` / `match_query` / Index loop `:437` / `recall.rs` post-blend
- [x] Grep `classify_pin_kind` callers/tests (OpenCode o3). Fold-in: ranking units leading-only; `sync.rs:523` inherits F2 — do not edit `sync.rs`
- [x] Rescan `conductor/deferred.md` for new open rows that overlap
- [x] Confirm #188 Mediums still T284-only / no new Cursor leftover that needs a mint
- [x] Re-dogfood `recall "what did we decide about retention"` + `preflight --summary` (expect hole until green) — PATH still 0/0/0 @ 3324 until this binary is installed (F21)
- [x] Re-check clap/rusqlite/chrono lock vs crates.io (**no bump** unless execute proves otherwise) — clap lock 4.6.1 / crates.io 4.6.6; rusqlite 0.39.0
- [x] FEATURE TX start `a5e94797-f17d-45bc-b591-a2399fa42da5`
- [x] Do **not** pin the live vault; do **not** `cargo install`; do **not** bootstrap grants; do **not** retune Safety SQL

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit recall/search/semantic dumps over pins | **DoD** F1–F10 / AC1–AC5 / AC14 |
| Preflight Index/summary 0 decisions | **DoD** F11–F12 / AC6–AC7 |
| `sync query` vault dumps | **DoD** F14 / AC15 |
| T211 F4 anywhere-in-body | **Lift** F2 / AC1 |
| T260 demote-only fails when depth is all noise | **Two-pass** F7 / AC4 |

---

## Declined (written)

| Item | Why |
|------|-----|
| Safety = Ledgerful hotspots | F23 → **T279** |
| Grants / briefing Denied | F24 → **T275** |
| Leftover `7d97a456` | F24 → **T276** |
| #188 Work / apply samples | F26 → **T284** |
| Memory-list pin-first | F13 / T216 |
| `--transcripts` / DTO `is_session` | F15 / F16 |
| KIND_DECISION bump / `candidate_depth` raise | F4 / F17 |
| T218 floors / T211 F25 / T240 F2 / T263 H2 | F17 / F25 |
| clap 5 / rusqlite 0.40 / new crates / `cargo install` | F20 / F21 |
| last-PR Cursor #188 as this DoD | Already T284 |

---

## Tasks (on go)

### Red (commit allowed)

- [x] AC1 `classify_pin_kind__buried_decision__other` (and leading/INVARIANT cases)
- [x] AC3 `rerank_hits__leading_decision_outranks_session_chrome`
- [x] AC4 `recall_full__chrome_monopoly__authority_pin_is_hit_one`
- [x] AC6 `preflight__index_prefers_leading_decision_over_objective_dump`
- [x] AC14 hermetic CLI `recall__unique_pin_needle__hit_one`

### Green

- [x] `session_chrome.rs` detector + GLOB fragment + first-line dedupe (AC2 / AC5 / AC17)
- [x] F2 leading-line + F3 INVARIANT in `classify_pin_kind`
- [x] F6 `SESSION_CHROME_PENALTY` inside `rerank_hits`
- [x] F7 two-pass `match_query` (authority GLOB then fill; skip pass 2 if pass1 full)
- [x] F35 bound `NOT IN` `?` placeholders (no UUID interpolation)
- [x] F36 `authority_glob_sql(column)`
- [x] F9 semantic in-memory prefer (AC16)
- [x] F10 `dedupe_session_chrome` after `rerank_hits` (chrome only)
- [x] F11 Index two-pass in `retrieval/src/preflight.rs`
- [x] Docs F32 (CAPABILITIES / CHANGELOG / PROTOCOL-COMPAT one-liner)
- [x] Confirm AC8–AC13 still green (T211 / T260 / T216 / T207 / forget unfiltered / JSON keys)

### Verify

- [x] `cargo fmt --check`
- [x] `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`
- [x] targeted nextest (retrieval lib + hermetic AC14/AC6)
- [x] `ledgerful verify --scope fast` then full gate at closeout — fmt/clippy/deny/audit + nextest **3247** passed (daemon Stopped)
- [x] Phase-1 review → `codex-review` (F28) — `review.codex.md`; P2-2 fixed
- [ ] conductor **Completed** only after implement-track Phase 6 (push / PR / GHA / squash)

---

## DoD (checkable after go)

- [x] Unique hermetic pin needle is recall hit **#1** (AC14)
- [x] Chrome monopoly still admits the pin into `candidate_depth` (AC4)
- [x] Preflight Index lists the pin; `--summary` `in_context_decisions >= 1` (AC6/AC7)
- [x] `sync query` vault top is the pin (AC15)
- [x] Safety SQL unchanged (T279); `memory list` ORDER unchanged (T216)
- [x] No new Recall JSON required keys; no clap 5; no rusqlite 0.40
- [x] `project.rs` / CLI `preflight.rs` / `sync.rs` untouched
- [x] Medium+ review findings not silently dropped
