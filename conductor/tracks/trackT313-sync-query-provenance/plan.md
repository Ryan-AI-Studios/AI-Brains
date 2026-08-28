# T313 Plan — `sync query` rescued heading

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** UX / HONESTY
**Ledger (planning):** DOCS `bdf8fddd-84f9-4d9d-9b7d-64887dd834e2`

---

## Preflight (plan time — 2026-08-28)

| Check | Result |
|-------|--------|
| HEAD / tree | `cd7bfde` T314 `#232` **CLEAN**. `main` == `origin/main`. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T271 banner **is** on PATH. T312/T315/T314 **not**. |
| `preflight --summary` (PATH) | Pinned **4544**; in-context **0/0/0**; `Total Word Count: 737` (PATH-behind T315) |
| Phrase `"graph backend"` | `ledgerful ledger search --json -- "graph backend"` → `[]` |
| Token `graph` | JSON hits (~10 KB) |
| Phrase `T314` | JSON **3** hits (AC12 control) |
| PATH `sync query "graph backend" --limit 3 --quiet` | Generic heading + F7 banner + `10 matching entries for 'graph':` |
| `print_ledger` | `sync.rs:563–571` hardcoded `--- Ledgerful Ledger Search ---` |
| Rescue arm | `sync_query_ledger.rs:307–312` sets `banner` only |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#232` `mergedAt` **2026-08-28T11:05:01Z**; comments/reviews **[]** — **N/A empty**. `#230` → **T325** already. |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan |
| Hotspots | `project.rs` #1 / `sync.rs` #2 (3.420) — do not grow / `governed_common.rs` #3 |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit phrase→fuzzy opacity | **DoD** F1 / AC1–AC3 / AC8 / AC11 |
| T271 F7 looks like a phrase hit | **Heading** F1; banner exact F2 / AC4 |
| T271 F6 first-seen | **Affirm** F4 — no scoring |
| T273 `--` | **Affirm** F6 / AC5 |
| T231 always-pretty / ndjson | **Affirm** F7 / AC14 |
| T124 `--no-bridge` | **Affirm** AC6 |
| last-PR `#232` Cursor | **N/A empty** F20 |
| last-PR `#230` F8 recency | **T325** — not stolen |
| T312 / T314 / T315 / T316–T324 / clap 5 | **Not stolen** / **Decline** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [ ] Re-read `probe_ledger_search` rescue arm + `LedgerProbeResult`
- [ ] Re-read `sync.rs` `print_ledger` `:563–571` + T211 ledger-first `:574`
- [ ] Re-read `ledger_rescue_banner` + unit `:663–667`
- [ ] Confirm Ledgerful `search.rs` still phrase-wraps
- [ ] Re-dogfood `sync query "graph backend"` — generic heading still the hole
- [ ] Confirm `T314` (or Phase 0 pick) is still a phrase hit for AC12
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] Confirm T325 placeholder still Pending (do not steal F8 recency)
- [ ] `ledgerful ledger start T313-sync-query-provenance --category FEATURE`
- [ ] **Do not** `cargo install` / live production `pin` / `.env` rewrite / Ledgerful edits in Phase 0

## Phase 1 — Red

- [ ] `ledger_section_heading__rescued_token__names_token` (AC1)
- [ ] `ledger_section_heading__phrase_hit__generic` (AC2)
- [ ] `ledger_section_heading__empty_token__generic` (AC3)
- [ ] `format_ledger_section_lines__rescued__heading_then_banner` (AC8)
- [ ] Confirm those tests **fail** while heading is hardcoded generic / helper missing
- [ ] AC4 T271 banner + AC5 T273 argv + AC6 T124 no-bridge are **stay-green** (not Phase-1 red)

## Phase 2 — Green

- [ ] F1 `ledger_section_heading` + F10 `format_ledger_section_lines` / `print_ledger_section`
- [ ] F3 `rescued_token: Option<String>` on `LedgerProbeResult`; F6 hit arm `Some(token.clone())`; other arms `None`
- [ ] `sync.rs` delete closure; call `print_ledger_section` (file must not grow)
- [ ] Match today’s spacing (heading `println!("\n{…}")` then banner then display)
- [ ] Do **not** edit `project.rs` / retrieval / contracts / Ledgerful
- [ ] Do **not** change F7 banner string
- [ ] Do **not** pass `--limit` to ledger argv
- [ ] Do **not** add `--format json` combined envelope

## Phase 3 — Stay-green + docs

- [ ] AC4 banner exact
- [ ] AC5 T273 argv
- [ ] AC6 T124 `--no-bridge`
- [ ] AC7 isolation vault header
- [ ] AC14 ndjson no ledger heading
- [ ] `ledger_json_non_empty` units
- [ ] CAPABILITIES pane bullet + OPERATIONS two-section sentence (AC10)
- [ ] CHANGELOG Unreleased T313
- [ ] No PROTOCOL-COMPAT DTO row

## Phase 4 — Manual + gate

- [ ] AC11 `cargo run` `sync query "graph backend"` rescued heading + F7 + `matching entries for 'graph'` (PATH-behind not a fail)
- [ ] AC12 phrase-hit control (`T314` or Phase 0 pick) generic heading, no banner
- [ ] AC13 empty diff on forbidden crates
- [ ] Review log `review.md`; medium+ not dropped
- [ ] Cross-model optional (F21) — skip unless Phase-1 review asks
- [ ] Full gate: `cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run --workspace` ; `cargo deny check` ; `cargo audit` ; `ledgerful verify --scope full`
- [ ] Conductor **Completed**; `deferred.md` closeout row; FEATURE TX commit
- [ ] implement-track Phase 6: push `track/T313-*`, PR, watch GHA `CI` green, squash-merge, prune. Never `git push origin main`.

## DoD (checkable)

- [ ] Rescued pane heading is `--- Ledgerful Ledger Search (rescued token: '<tok>') ---`
- [ ] Phrase-hit / miss heading stays `--- Ledgerful Ledger Search ---`
- [ ] F7 banner sentence exact
- [ ] Rescue still runs (F6 first-seen, cap 3)
- [ ] `--no-bridge` still skips the ledger section
- [ ] ndjson still has no ledger pane
- [ ] `sync.rs` did not grow (print extracted)
- [ ] Status **Pending** until go; **Completed** only after merge hygiene

## Isolation

Do not grow `sync.rs` / `project.rs`. No `cargo install`. Never `git push origin main`.
