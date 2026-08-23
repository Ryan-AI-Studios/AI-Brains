# T290 Plan — granted-empty lists/progressive copy-paste recall + Pinned: N

**Status:** **Pending** (Planned). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F34 / AC1–AC17
**Category:** FEATURE / UX / HONESTY
**Ledger TX (planning):** `c66b1485-a4a7-4ca6-87d2-8b2e2d8b5865` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## Preflight (plan time — 2026-08-23)

| Check | Result |
|-------|--------|
| HEAD / tree | `6a8deb3` T289 `#205`. CLEAN. `origin/main` = HEAD |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T274. No T285–T289.** Lists/progressive hole is in **source**. **Do not `cargo install`.** |
| `evidence` / `source` / `review` list `--format json` | `items: []`; `next_step` ellipsis `ai-brains recall "…"`; **no `Pinned:`** |
| `--format human` | `(none)` only; **no next line** |
| `query progressive "what did we decide about SQLCipher"` | `denied: false`, `results: []`, `next_step` ellipsis (**no `SQLCipher`**) |
| `preflight --summary` | Pinned **3908**; in-context **0/0/0**; word **443** |
| Last PR comments | #205 T289 — **empty** (N/A). **No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); serde_json **1.0.150**; chrono **0.4.44**; rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** — do not touch. `governed_common.rs` **#5** — formatter only (no QueryStore). `personal.rs` **#7** — T289. CLI `preflight.rs` **#8** — do not grow. `briefing.rs` — T288, do not grow. |
| Ledger | 0 pending / 0 drift at scan |
| `ISSUES.md` | **Does not exist** (F22) |
| ledgerful search | `apply_authorized_empty_list_next` → `governed_common.rs:60` + evidence/source/review emit; `count_pinned_memories` → `query_store.rs:699` + `briefing.rs:57` |
| Online | clig.dev human-first + next-command + JSON stable; T180 string growth not new keys; T263 F8 overlay; clap 4.6.6 / no clap 5 |
| Skill | CAPABILITIES Empty row + progressive granted-empty (F25) |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `governed_common.rs` `apply_authorized_empty_list_next` `:60` + `PROGRESSIVE_RECALL_FALLBACK` `:54`
- [ ] Confirm fallback const still exact ellipsis — **do not edit the string** (F8 / AC15)
- [ ] Confirm list DTOs still have **no** `next_step` field — **do not add**
- [ ] Confirm `ProgressiveQueryResponse.next_step` still `Option<String>` skip_serializing_if — **grow string only**
- [ ] Confirm `count_pinned_memories` still on `QueryStore` `:699` — **do not** add a store method; **do not** import QueryStore into `governed_common.rs`
- [ ] Confirm `run_list_local` still has `ScopeRef` + `ctx.conn` — COUNT here
- [ ] Confirm daemon `emit_list` has no ctx — `pin_count = None`
- [ ] Confirm `apply_progressive_search_hints` empty arm `:76` still sets the const — replace with formatter
- [ ] Confirm deny stderr `:133` still prints the const — **do not change**
- [ ] Confirm clap list `--format` default `json`; progressive has no `--format`
- [ ] Confirm evidence search dispatches `run_list` — inherit overlay; F32 needle is **not** evidence `--query`
- [ ] Rescan `conductor/deferred.md` — T290 absorbed; T291–T300 / T288 / T289 not stolen
- [ ] Confirm #205 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [ ] Re-dogfood four Manual commands **read-only**. **Did not** pin production decisions; **did not** write `.env`; **did not** extra `policy bootstrap`
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [ ] FEATURE TX (new)
- [ ] Did **not** `cargo install`; did **not** grow `project.rs` / `preflight.rs` / `briefing.rs` / `personal.rs` / `query_store.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit lists/progressive empty U=6 | **DoD** F1–F7 / AC1–AC6 / AC12 |
| Placeholder Manual four commands | **DoD** AC12 |
| T263 F8 parenthetical pin hint | **DoD** F3 / F7 |
| T263 F9 progressive ellipsis | **Granted-empty string growth**; deny const freeze F8 |

## Declined (written)

| Item | Why |
|------|-----|
| H2 pin→Approved | F1 / F24 |
| T288 `vault_pin_*` on lists | F3 / AC13 |
| T289 Personal | Completed |
| `query trace` wrap | **T291** |
| `policy check` human | **T292** |
| Neighbors / leftover / forget-list | **T293 / T294 / T299** |
| last-PR #205 Cursor | N/A empty — no T301 |
| clap 5 / rusqlite 0.40 / T240 F2 | Standing |

---

## Phase 1 — Red (required)

- [ ] `format_authorized_empty_next__with_count__includes_pinned_and_copy_paste` (AC1)
- [ ] `sanitize_recall_query__cases__expected_needle` (AC4 / AC14)
- [ ] Confirm they **fail** (not compile-error-only) before green

## Phase 2 — Green (formatter)

- [ ] `LIST_RECALL_QUERY` exact `what did we decide`
- [ ] `sanitize_recall_query` F6
- [ ] `format_authorized_empty_next`
- [ ] `apply_authorized_empty_list_next(value, pin_count)` — empty items still `contains("recall")`
- [ ] No `unwrap`/`expect`/`panic` in production
- [ ] No QueryStore import in `governed_common.rs`

## Phase 3 — List emit + progressive

- [ ] Local COUNT `ScopeRef::Repository` → `count_pinned_memories(Some).ok()`
- [ ] `emit_list` JSON + human second line (evidence / source / review)
- [ ] Daemon `pin_count = None`
- [ ] Progressive empty arm: formatter + COUNT + operator query; deny stderr const frozen
- [ ] AC2/AC3 0-pin hermetic; AC5 with pin; AC6 progressive needle
- [ ] AC7/AC8/AC9/AC15 stay green

## Phase 4 — Docs + gates

- [ ] CAPABILITIES Empty + progressive granted-empty (F25)
- [ ] list + progressive after_help one sentence
- [ ] CLI-EXIT-CODES authorized-empty sentence
- [ ] OPERATIONS empty-vs-deny
- [ ] PROTOCOL-COMPAT overlay string growth
- [ ] CHANGELOG T290
- [ ] `cargo fmt --check` ; `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [ ] `cargo nextest run --workspace` (or targeted then full)
- [ ] `cargo deny check` ; `cargo audit`
- [ ] `ledgerful verify --scope full`

## Phase 5 — Manual + closeout

- [ ] Manual AC12 four commands via `cargo run -p ai-brains-cli -- …` (not PATH)
- [ ] Phase-1 `review.md` → clean
- [ ] `codex-review` (FEATURE)
- [ ] conductor.md T290 **Completed**; deferred closeout; README
- [ ] Publish: push `track/T290-*` → PR → `gh run watch --exit-status` CI green → `gh pr merge --squash --delete-branch` → fetch/prune. Never `git push origin main`. Never force-push.

## DoD

- [ ] Granted-empty JSON `next_step` is copy-paste `recall` + `Pinned: N` (when COUNT Ok)
- [ ] Progressive empty uses the operator query (not U+2026)
- [ ] Human lists print a next line
- [ ] Arrays stay `[]`; no H2
- [ ] T288 / T289 / T291–T300 / H2 not stolen
- [ ] CI green + squash-merged

## Isolation (every phase)

No `cargo install`. No live pin as implement (hermetic + optional Manual canary). No `.env` write. No live extra `policy bootstrap`. No `retention apply --confirm`. No schtasks mutate. No `graph rebuild`.
