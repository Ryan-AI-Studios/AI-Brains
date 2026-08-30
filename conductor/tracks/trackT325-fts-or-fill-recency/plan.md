# T325 Plan — F8 OR-fill PreferRecency

**Status:** ✅ **Completed** (published `#247` `9119c74`). Spec [spec.md](./spec.md).
**Category:** BUGFIX / RETRIEVAL
**Ledger (planning):** DOCS `e8a70f94-0beb-4b98-bc5b-50da64bdd87a`
**Ledger (fold-in):** DOCS `86f98ed2-6873-4cda-9e7e-84b86500af12`
**Ledger (mint):** T315 DOCS `ca5b1614-6849-416d-ad27-1d44a23198d7`

---

## Preflight (plan time — 2026-08-29)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `6c23288` plan commit; `origin/main` = `211c934` (ahead **1**). Plan-write was `211c934` / ahead **0** (Agy m2). Branch `track/T325-fts-or-fill-recency`. Product `src/` = T324 `#246` `211c934`. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T312–T324 **not** on PATH. PATH `recall "graph backend"` dump-first (T285). |
| `preflight --summary` (PATH) | Pinned **4656**; in-context **0/0/0**; `Total Word Count: 718` (PATH-behind T315). |
| Bugbot `#230` `3877408710` | **Still true** — F8 `or_pass` is `Prefer` only (`lexical.rs:231–238`) |
| AND recency | `:197–213` `PreferRecency` |
| PreferRecency SQL | `:390–392` `ORDER BY mp.updated_at DESC` |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; serde_json **1.0.150**; uuid `"1.13"` / lock **1.23.1**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#246` empty. `#237` → **T326**. `#230` → **this**. **No T327.** |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX) |
| Impact | **LOW** (conductor-only dirty) |
| Hotspots | CLI `project.rs` #1 — do not touch. `session_chrome.rs` **#6** — do **not** grow. |
| Line counts (physical) | `lexical.rs` **653**; retrieval `recall_rank_v3.rs` **238**; CLI `recall_rank_v3.rs` **214**. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| `#230` Bugbot F8 skips recency | **DoD** F1–F7 / AC1 |
| T312 F8/F40/F42 | **Freeze** F3/F6/F8/F24 |
| T316 list recency | **Not stolen** |
| T324 residuals / Completed note | **Not stolen** / absorb dirty conductor |
| T326 / T307 / clap 5 / H2 / depth | **Not stolen** / **Decline** |
| last-PR `#246` | **N/A empty** |
| Agy m2 HEAD snapshot | **Folded** `6c23288` / ahead **1** |
| OpenCode m1 CAPABILITIES pass-2 AND | **Folded** F25 / AC11 |
| OpenCode m2 AC6 Prefer ORDER BY | **Folded** AC6 |
| OpenCode m3 AC1 12× flood | **Folded** F35 / AC1 / Phase 1 |
| OpenCode O1/O2 AC16 / volatile | **Folded** |
| Agy m1/m3/O1/O2 | **Already** F7 / F35 / F33 / §5.1 |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [x] Confirm cwd `C:\dev\AI-Brains`
- [x] Re-read `match_query` AND recency `lexical.rs:197–213` vs F8 `:215–251`
- [x] Re-read `AuthorityFilter::PreferRecency` SQL `:390–392`
- [x] Confirm Bugbot `#230` still true (Prefer-OR only)
- [x] Confirm T312 AC5 dumps are **prose** (not this hole); T326 still Pending
- [x] Confirm rusqlite lock **0.40.2**; `candidate_depth(5)==15`
- [x] Rescan `deferred.md` open overlapping rows
- [x] `ledgerful ledger start T325-fts-or-fill-recency --category BUGFIX` → `1ea8c80d`
- [x] **Do not** `cargo install` / live pin / `.env` rewrite / raise `candidate_depth` / edit `session_chrome.rs`

## Phase 1 — Red

- [x] `match_query__or_fill_tags_flood__recency_retry_pin_first` (AC1) — **failed** (TAGS dumps lead). Fixture = `ASSISTANT: TAGS: t325` + **`12× "t325or backend "`** + short pin last.
- [x] Confirm T312 AC5 / F40 / CLI AC12 still **pass** (F8 / F24)

## Phase 2 — Green (lexical F8 arm)

- [x] PreferRecency MATCH on `or_expr` when Prefer-OR retain empty (F1/F2)
- [x] `pass2_expr = or_expr` **after** recency (F7)
- [x] Recency only when empty, not partial (F28)
- [x] Tracing `stage = "prefer_or_recency"` (optional)
- [x] AC6 Prefer `ORDER BY rank` / `!ORDER BY mp.updated_at`
- [x] AC2–AC9 / AC12–AC14 / AC16 stay-green

## Phase 3 — Docs

- [x] CHANGELOG Unreleased
- [x] CAPABILITIES pin-type row: OR-when-retain / AND-when-both-empty + OR-fill recency-retries (AC11)

## Phase 4 — Targeted gate

- [x] `cargo clippy -p ai-brains-retrieval --all-targets -- -D warnings` (AC10)
- [x] nextest `-p ai-brains-retrieval` + CLI `--test recall_rank_v3`
- [x] Manual AC15 honesty (PATH dump-first OK; **no** live pin)
- [x] Full gate + PR [#247] GHA green + squash `9119c74` (never `git push origin main`)

## DoD (after go)

- [x] AC1–AC16
- [x] T312 AC5/F40/AC12/AC14 stay-green
- [x] No `candidate_depth` / KIND / T217 gate change
- [x] T326 / T307 / H2 **not stolen**
- [x] Medium+ review findings not silently dropped

## Isolation

No daemon DTO. No H2. No `cargo install`. No live vault pins. Never `git push origin main`.
