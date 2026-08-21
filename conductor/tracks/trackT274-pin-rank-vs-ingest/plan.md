# T274 Plan — Pins vs harness ingest ranking

**Status:** **Pending** (Planned; F0 until **go**)
**Spec:** [spec.md](./spec.md) F0–F34 / AC1–AC17
**Category:** FEATURE / UX / RETRIEVAL
**Ledger TX (planning):** `9c1049c0-5520-430d-a1f0-01aba355082e` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go**

---

## Preflight (plan time — 2026-08-21)

| Check | Result |
|-------|--------|
| HEAD / tree | `deabae7` CLEAN; `main` ahead of `origin/main` by **1** (placeholder mint). Product `src/` = T270 `#188` `14d42af` |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-21 05:55. T270 on PATH. **Do not `cargo install`.** |
| Source debug | Older (2026-08-20 22:16). Dogfood used PATH |
| `preflight --summary` | 3297 pinned; **0** in-context decisions/hotspots/constraints |
| `recall "what did we decide about retention"` | T248 reviews + JSON + `## Objective` — **no pin top-5** |
| `recall "DECISION: T270"` | Five Objective/review dumps; pin **not in candidate set** |
| `memory list --limit 5` | All `## Objective` / Track Plan Review / T248 Review, 18m, `status=pinned` |
| `preflight --pretty -m 400` | Safety = T272 `## Objective` (**T279**); Sessions = dumps |
| Last PR comments | #188 Cursor Bugbot **2 Mediums** → **T284** (already minted). Still true on `14d42af` |
| Open PR on HEAD | none (Dependabot remotes only) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150**; chrono **0.4.44**; rusqlite **0.39.0** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (3.990) — do not grow. CLI `preflight.rs` #7 **2027** lines — do not grow. `sync.rs` #2 — do not grow. `ranking.rs` **858**; retrieval `preflight.rs` **1003**; `recall.rs` **861** |
| Ledger | 0 pending at scan; planning TX `9c1049c0` |
| `ISSUES.md` | **Does not exist** (F29) |
| ledgerful search | `rerank_hits` `ranking.rs:248` / `recall.rs:512`; `classify_pin_kind` `:84` |
| Online | SQLite FTS5 BM25; ES function_score + 2025-12 multiplicative boost warning; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact`
- [ ] Re-read `classify_pin_kind` / `rerank_hits` / `match_query` / Index loop `:437` / `recall.rs` post-blend
- [ ] Rescan `conductor/deferred.md` for new open rows that overlap
- [ ] Confirm #188 Mediums still T284-only / no new Cursor leftover that needs a mint
- [ ] Re-dogfood `recall "what did we decide about retention"` + `preflight --summary` (expect hole until green)
- [ ] Re-check clap/rusqlite/chrono lock vs crates.io (**no bump** unless execute proves otherwise)
- [ ] FEATURE TX start
- [ ] Do **not** pin the live vault; do **not** `cargo install`; do **not** bootstrap grants; do **not** retune Safety SQL

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

- [ ] AC1 `classify_pin_kind__buried_decision__other` (and leading/INVARIANT cases)
- [ ] AC3 `rerank_hits__leading_decision_outranks_session_chrome`
- [ ] AC4 `recall_full__chrome_monopoly__authority_pin_is_hit_one`
- [ ] AC6 `preflight__index_prefers_leading_decision_over_objective_dump`
- [ ] AC14 hermetic CLI `recall__unique_pin_needle__hit_one`

### Green

- [ ] `session_chrome.rs` detector + GLOB fragment + first-line dedupe (AC2 / AC5 / AC17)
- [ ] F2 leading-line + F3 INVARIANT in `classify_pin_kind`
- [ ] F6 `SESSION_CHROME_PENALTY` inside `rerank_hits`
- [ ] F7 two-pass `match_query` (authority GLOB then fill)
- [ ] F9 semantic in-memory prefer (AC16)
- [ ] F10 chrome first-line collapse after `rerank_hits`
- [ ] F11 Index two-pass in `retrieval/src/preflight.rs`
- [ ] Docs F32 (CAPABILITIES / CHANGELOG / PROTOCOL-COMPAT one-liner)
- [ ] Confirm AC8–AC13 still green (T211 / T260 / T216 / T207 / forget unfiltered / JSON keys)

### Verify

- [ ] `cargo fmt --check`
- [ ] `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`
- [ ] targeted nextest (retrieval lib + hermetic AC14/AC6)
- [ ] `ledgerful verify --scope fast` then full gate at closeout
- [ ] Phase-1 review → `codex-review` (F28)
- [ ] conductor **Completed** only after implement-track Phase 6 (push / PR / GHA / squash)

---

## DoD (checkable after go)

- [ ] Unique hermetic pin needle is recall hit **#1** (AC14)
- [ ] Chrome monopoly still admits the pin into `candidate_depth` (AC4)
- [ ] Preflight Index lists the pin; `--summary` `in_context_decisions >= 1` (AC6/AC7)
- [ ] `sync query` vault top is the pin (AC15)
- [ ] Safety SQL unchanged (T279); `memory list` ORDER unchanged (T216)
- [ ] No new Recall JSON required keys; no clap 5; no rusqlite 0.40
- [ ] `project.rs` / CLI `preflight.rs` / `sync.rs` untouched
- [ ] Medium+ review findings not silently dropped
