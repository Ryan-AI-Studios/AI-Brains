# T312 Plan — Recall rank v3 (pins over dumps)

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** FEATURE / UX / RETRIEVAL
**Ledger (planning):** DOCS `8b1b418b-acbb-4398-b867-7ea297d10e41`
**Ledger (fold-in):** DOCS `2e553fb4-57c6-459e-b5b7-ea774cd74021`

---

## Preflight (plan time — 2026-08-27)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `413aa33` plan commit CLEAN; `origin/main...HEAD` **ahead 2**. Plan-write was `27731be` / ahead **1** (Agy m1 / OpenCode m3). Branch `track/T312-T324-cli-dogfood`. Product `src/` = T285 + T311. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM** |
| `preflight --summary` | Pinned **4513**; in-context **0/0/0**; word count **587** |
| `recall "graph backend" --limit 3` | #1 audit dump **−4.060**; #2 `## Objective` **−3.824**; #3 `# Review of Track 253` **−1.325** |
| JSON `--limit 15` | **`n == 3`** — AND MATCH only 3 rows (pass-1 empty) |
| `KIND_*` / chrome / bonus | 4.0 / 2.0 / −16 / +16 frozen in `ranking.rs` |
| Detector | `session_chrome.rs` `:14–44`; hotspot **#6** |
| Two-pass | `lexical.rs` `:171–221`; T217 R0-non-empty skip `:85–87`; ≥3 token rescue `:90` |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#229` `mergedAt` **2026-08-27T23:50:34Z**; comments **`[]`** — N/A; no T325 |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; this TX `8b1b418b` |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit ranking + T285 live dumps | **DoD** F5/F6/F8 / AC1–AC6 / AC12 |
| T285 “more chrome prefixes as vault grows” | **F5** ATX token set (not infinite prose prefixes) |
| T217 OR helpers | **Reuse**; gate unchanged F9 |
| T218 floors / `candidate_depth` / H2 / clap 5 | **Declined** F4 / F20 / F24 |
| T315 / T313 / T317 / T316 | **Not stolen** |
| last-PR `#229` Cursor | **N/A empty** |
| OpenCode M1–M3 needle/bonus redness | **F42** / AC2 / AC4 stay-green |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [ ] Re-read `lexical.rs` `match_query` retain/recency `:171–221` and T217 early-return `:85–87`
- [ ] Re-read `session_chrome.rs` `is_session_chrome` `:14–44` + `parent_seeds_graph_neighbors` `:180`
- [ ] Re-read `ranking.rs` `rerank_hits_with_query` `:293–364`; confirm `KIND_DECISION == 2.0`
- [ ] Confirm `candidate_depth` clamp 15..50 and T218 0.55/0.60
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] `ledgerful ledger start T312-recall-rank-v3 --category FEATURE`
- [ ] **Do not** `cargo install` / live production `pin` / `.env` rewrite in Phase 0

## Phase 1 — Red

- [ ] `is_session_chrome__atx_tokens__ac1` rstest (`# Preview of graph` **false**)
- [ ] `rerank_hits_with_query__verbose_other_dump_loses_to_pin__ac2` (pin first line **no** query tokens)
- [ ] Retrieval hermetic `match_query__and_retain_empty__authority_or_fills_pin__ac5` (query `"t312or backend"` — F42; **no** UUID in query)
- [ ] `parent_seeds_graph_neighbors__verbose_other__false__ac6`
- [ ] CLI `crates/ai-brains-cli/tests/recall_rank_v3.rs` AC12 / AC13 (same F42 fixture)
- [ ] Confirm those tests **fail** on current T285 tree (AND-miss pin absent; AC2 dump leads without F6)
- [ ] AC3 crumb index 0 + AC4 full-needle pin #1 are **stay-green** (not Phase-1 red)

## Phase 2 — Green

- [ ] F5 ATX token detector in `is_session_chrome` (token set, not substring)
- [ ] Define `DUMP_OTHER_CHAR_FLOOR` / `DUMP_OTHER_PENALTY` in `session_chrome.rs`; apply F6 in `rerank_hits_with_query` (F7 no stack)
- [ ] F8 authority-OR fill in `match_query` after recency-empty; thread `raw_query`; ≥2 contentful tokens
- [ ] F10 `parent_seeds_graph_neighbors` skips verbose-Other
- [ ] Reuse `match_or` / `select_or_tokens` from `ai-brains-core` (no new crate)
- [ ] Do **not** change T217 R0/≥3 gate
- [ ] Do **not** bump `KIND_*` / floors / `candidate_depth`
- [ ] Do **not** edit `project.rs` / `sync.rs` / CLI `preflight.rs` / `pin.rs` write / `hybrid.rs` floors

## Phase 3 — Stay-green + docs

- [ ] AC7 T285 `recall_rank_v2` + onboarding chrome unit
- [ ] AC8 T260 stub exclude
- [ ] AC9 T207/T261 empty
- [ ] AC10 `forget --match` still finds the dump
- [ ] AC11 JSON keys frozen
- [ ] AC13 `sync query` vault (hermetic; do not grow `sync.rs`)
- [ ] AC14 `--semantic` fallback no HTTP
- [ ] AC15 OR SQL `?` only
- [ ] AC16 freeze consts
- [ ] AC17 chrome long dump −16 once
- [ ] AC18 list recency
- [ ] CAPABILITIES pin-type row additive + CHANGELOG T312
- [ ] PROTOCOL-COMPAT: no new required keys

## Phase 4 — Gate + review

- [ ] `cargo fmt --check` ; clippy workspace `-D warnings` ; nextest workspace ; `cargo deny check` ; `cargo audit`
- [ ] `ledgerful verify --scope full`
- [ ] Phase-1 review log `review.md` until clean
- [ ] `codex-review` (FEATURE) until clean
- [ ] Optional Manual canary (F42: `recall "t312or backend"`; uuid in pin body only) — not live `graph backend` as SoT

## Phase 5 — Closeout

- [ ] Conductor T312 **Completed** with evidence
- [ ] deferred.md T312 row struck; residuals appended
- [ ] FEATURE TX commit
- [ ] Phase 6: push `track/T312-*` → PR → watch GHA `CI` green → `gh pr merge --squash --delete-branch`. Never `git push origin main`.

---

## DoD (checkable)

- [ ] AC2: 2000-char prose dump loses to pin whose first line has **no** query tokens
- [ ] AC4 stay-green: full-needle tagged pin still #1 vs prose dumps (T285)
- [ ] AC5/AC12: two-token AND-miss (`t312or backend`) pin #1 via F8; UUID only in bodies
- [ ] AC1: `# Preview of graph` is **not** chrome
- [ ] T285 chrome tests stay green
- [ ] `KIND_DECISION` still 2.0; floors still 0.55/0.60; depth still 15..50
- [ ] No new Recall JSON keys
- [ ] `forget --match` unfiltered
- [ ] CAPABILITIES + CHANGELOG
- [ ] Full gate + Codex PASS
- [ ] PATH install **not** required for Completed (F21)

---

## Isolation

No `cargo install`. No live vault production pins as implement SoT. Never `git push origin main`. T307 / T313–T324 / H2 / clap 5 / floor retune not stolen.
