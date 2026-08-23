# T286 Plan — preflight Index TAGS-or-GLOB + envelope titles

**Status:** **Pending** (Planned). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F34 / AC1–AC16
**Category:** FEATURE / UX / RETRIEVAL
**Ledger TX (planning):** `397f9c55-5953-402b-95fc-db431f5a037c` (DOCS)
**Ledger TX (implement):** FEATURE — start on **go**

---

## Preflight (plan time — 2026-08-23)

| Check | Result |
|-------|--------|
| HEAD / tree | `16ee1aa` T285 `#201`. CLEAN. `origin/main` = HEAD |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T274. No T285.** Hole is in **source** Index SQL. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **3716**; in-context 5/0/0; Scope `3581317d`; word count **255** |
| `preflight --pretty -m 1500` | Safety hotspots OK (T279). Index **`1. ## Objective -- just now`** |
| Last PR comments | #201 T285 — **empty** (N/A). **No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** — do not touch. `sync.rs` #2. `session_chrome.rs` **#6** — extend. CLI `preflight.rs` **#8** — do not grow production |
| Ledger | 0 pending / 0 drift at scan; planning TX `397f9c55` |
| `ISSUES.md` | **Does not exist** (F26) |
| ledgerful search | `index_marker_glob_sql` `session_chrome.rs:90` + `preflight.rs:462`; `truncate_index_summary` `preflight.rs:1002` |
| Online | FTS5 bm25 N/A (Index is projection GLOB); clig.dev dual-count honesty; clap 4.6.6 latest / no clap 5; T285 lexical Prefer join is the pattern |
| Skill | CAPABILITIES Index/summary sentence only (F29). No pin write change |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `preflight.rs` Index two-pass `:458–545` + `drain_index_pass` `:682`
- [ ] Re-read `session_chrome.rs` `index_marker_glob_sql` / `tags_envelope_sql`
- [ ] Re-read `ranking.rs` `first_contentful_line` (T285 F2)
- [ ] Confirm CLI `preflight.rs` `:884–888` still `matches("DECISION:")` — **do not grow** production
- [ ] Confirm `pin.rs` still prepends `TAGS:` — **do not rewrite**
- [ ] Confirm T274 `preflight_index_pin_rank.rs` untagged AC6 — **stay green**
- [ ] Confirm T279 Safety SQL / `preflight_safety.rs` — **do not edit**
- [ ] Confirm T220 JSON keys / 9-arg formatters — **do not add keys**
- [ ] Rescan `conductor/deferred.md` — T286 rows absorbed; T287/T288/T293 not stolen
- [ ] Confirm #201 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [ ] Re-dogfood `preflight --pretty -m 1500` + `--summary` **read-only**. **Did not** pin production decisions; **did not** write `.env`
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [ ] FEATURE TX (new) — category FEATURE
- [ ] Did **not** `cargo install`; did **not** grow `sync.rs` / `project.rs` / CLI `preflight.rs` production / `pin.rs` write / `lexical.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit Index `## Objective` / summary 0 vs 3k pins | **DoD** F1–F4 / AC1–AC6 / AC16 |
| T285 F13 Index/summary | **DoD** this track |
| T274 AC6/AC7 untagged | **Regression** F5 / AC3 |
| Placeholder Manual `--pretty` + `--summary` | **DoD** AC5/AC6/AC16 |

## Declined (written)

| Item | Why |
|------|-----|
| Session-section chrome | F12 — recency of active work |
| `memory list` ORDER | **T287** |
| Briefing stanza | **T288** |
| `graph neighbors` CLI | **T293** |
| New summary JSON key | F10 T220 freeze |
| T279 Safety SQL | Completed freeze |
| T263 H2 / T240 F2 / clap 5 / rusqlite 0.40 | Standing |
| last-PR #201 Cursor | N/A empty — no T301 |

---

## Phase 1 — Red (required)

- [ ] AC1 retrieval: TAGS envelope pin vs newer `## Objective` dump — Index item 1 is the pin
- [ ] AC2: Index title is `DECISION:` not `TAGS:`
- [ ] AC4 unit: `index_pass1_glob_sql` single `AND (` with TAGS **OR** marker+HOTSPOT
- [ ] AC5 CLI pretty Index item 1
- [ ] AC6 CLI summary JSON `in_context_decisions >= 1` tagged
- [ ] Commit red (allowed)

## Phase 2 — Green

- [ ] F2 `index_pass1_glob_sql` in `session_chrome.rs`
- [ ] Pass-1 extra uses it (`preflight.rs`)
- [ ] F4 Index title `first_contentful_line`; empty → `Untitled Memory`
- [ ] AC3 T274 untagged stays green
- [ ] AC7 T220 untagged summary stays green
- [ ] AC8–AC15 stay green (Safety / sections / pretty strip / global skip / no new keys / list ORDER)
- [ ] Commit green (allowed)

## Phase 3 — Docs

- [ ] CAPABILITIES: Index pass-1 marker-GLOB **or** TAGS; titles after envelope
- [ ] CHANGELOG T286
- [ ] PROTOCOL-COMPAT: no new required keys (N/A sentence)

## Phase 4 — Verify

- [ ] `cargo fmt --check`
- [ ] `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`
- [ ] `cargo nextest run -p ai-brains-retrieval -p ai-brains-cli -E "test(preflight)"`
- [ ] `.\scripts\dev-check.ps1` (or workspace gate) before closeout
- [ ] `ledgerful verify --scope full`
- [ ] `codex-review` (FEATURE)

## Phase 5 — Closeout

- [ ] `conductor.md` T286 **Completed**
- [ ] `review.md` findings closed
- [ ] Append residuals to `deferred.md`
- [ ] FEATURE TX commit
- [ ] **Did not** `cargo install` unless owner asked

## Phase 6 — Publish (standing)

- [ ] Push `track/T286-*` (never `git push origin main`)
- [ ] PR to `main`; watch GHA `CI` until every job is green
- [ ] `gh pr merge --squash --delete-branch`
- [ ] `git fetch --all --prune`; point local `main` at `origin/main`; delete merged local `track/T286-*` only

---

## Manual DoD (on go)

```powershell
ai-brains preflight --pretty -m 1500 --no-hook-prompt
ai-brains preflight --summary
ai-brains preflight --summary --format json
```

Pass:

- `--pretty` **Memory Index** line `1.` does **not** start with `## Objective` when ≥1 in-scope pin exists (hermetic proof is SoT; live canary with `--tag` allowed).
- `--summary` stdout contains `Pinned memories:` and `In context decisions` is ≥1 when that pin is in the window (or hermetic JSON AC6).
- Safety block still matches `safety sync --dry-run` paths (T279).
- Hermetic: chrome dump in Index is not item 1.
- Exit **0**.

Optional live canary (not architecture): `ai-brains pin "DECISION: T286-canary-<uuid>" --tag t286-canary` then the three commands. Do **not** pin production decisions as implement.

---

## DoD

- [ ] Index first item is not `## Objective` when a tagged or untagged in-scope pin exists
- [ ] Index title is the marker line, not `TAGS:`
- [ ] Summary does not hide 3k pins behind a chrome window of decisions 0 **when a pin is in-scope**
- [ ] T274 untagged Index + T220 untagged summary JSON stay green
- [ ] T279 Safety unchanged
- [ ] No new JSON required keys
- [ ] No `cargo install` unless asked
