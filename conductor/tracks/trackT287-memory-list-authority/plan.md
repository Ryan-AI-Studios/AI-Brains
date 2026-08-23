# T287 Plan — human `memory list` prefer-fill authority; JSON/store recency frozen

**Status:** **Pending** (Planned). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F38 / AC1–AC18 + §13 AI fold-in
**Category:** FEATURE / UX
**Ledger TX (planning):** `673e7322-b68f-40dd-bd34-6a91a83e7412` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `35a4042f-dd4a-40fc-b81a-6e34fdb7d903` (DOCS)
**Ledger TX (implement):** FEATURE — start on **go**

---

## AI fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F6/AC3 (Agy m1):** empty contentful → fallback `TAGS:` line, not `""`.
2. **AC16 (Agy O1):** rstest overlap / authority-only / recency-only / limit.
3. **F6/F24 (OpenCode m1):** `preview_line` inherit-only in `forget.rs` `:19`/`:24` and `graph.rs` `:248` — **do not edit** those files.
4. **Volatile (OpenCode m2):** pinned/word-count snapshots.
5. **Citation (OpenCode m3):** `run_inventory` `:137`.
6. **Affirm:** #202 N/A; T288/T293/T299 not stolen; Agy m2 GLOB already AC5; Agy O2 after_help already AC17.

---

## Preflight (plan time — 2026-08-23)

| Check | Result |
|-------|--------|
| HEAD / tree | `360139d` T286 `#202`. CLEAN. `origin/main` = HEAD |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T274. No T285/T286.** List hole is in **source**. **Do not `cargo install`.** |
| `memory list --limit 5` | Five recency chrome (`## Objective` / review ingest). **0** `DECISION:` |
| `memory list` limit 50 | Fifty recency chrome. Footer `Showing 50 of 3751` |
| `memory list --summary` | `Pinned: 3751` / `Forgotten: 0` |
| `memory list --format json --limit 1` | `items[0]` recency chrome; keys T216 F10; `total=3751` |
| clap default limit | **50** (placeholder “default 5” was wrong) |
| Last PR comments | #202 T286 — **empty** (N/A). **No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** — do not touch. `sync.rs` #2. `forget.rs` **#3** — do not grow production. `session_chrome.rs` **#6** — do not edit. Extend `memory.rs` + `query_store.rs` |
| Ledger | 0 pending / 0 drift at scan; planning TX `673e7322` |
| `ISSUES.md` | **Does not exist** (F23) |
| ledgerful search | `list_memories` `query_store.rs` + `memory.rs:177` + store tests |
| Online | clig.dev human-first + JSON stable; SQLite GLOB case-sensitive; clap 4.6.6 / no clap 5; T283 human permute analog; T286 GLOB-or-TAGS duplicate in store (F27) |
| Skill | CAPABILITIES Memory inventory sentence (F25). No pin write change |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `memory.rs` `run_inventory` `:137` + `preview_line` `:32`
- [ ] Confirm `preview_line` callers still `forget.rs:19/24` + `graph.rs:248` — **inherit only; do not edit** those files
- [ ] Re-read `query_store.rs` `list_memories` / `memory_list_from_where` — **do not** change ORDER
- [ ] Confirm `MemoryListFilter` still four fields — **do not** add `authority` field
- [ ] Confirm `classify_pin_kind` / `first_contentful_line` still exported from retrieval
- [ ] Confirm `index_pass1_glob_sql` still TAGS + ASSISTANT: TAGS only — **do not** edit `session_chrome.rs`; duplicate GLOB in store (F27)
- [ ] Confirm `pin.rs` still prepends `TAGS:` — **do not rewrite**
- [ ] Confirm T216 store recency unit + CLI JSON schema tests — **stay green**
- [ ] Confirm `forget.rs` list-forgotten still delegates to `run_inventory` — **do not grow** production
- [ ] Rescan `conductor/deferred.md` — T287 rows absorbed; T288/T293/T299 not stolen
- [ ] Confirm #202 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [ ] Re-dogfood `memory list --limit 5` + `--summary` + `--format json --limit 1` **read-only**. **Did not** pin production decisions; **did not** write `.env`
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [ ] FEATURE TX (new)
- [ ] Did **not** `cargo install`; did **not** grow `sync.rs` / `project.rs` / `forget.rs` production / `session_chrome.rs` / `ranking.rs` / `pin.rs` write / CLI `preflight.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit `memory list --limit 5` just-now ingest | **DoD** F1–F6 / AC1–AC3 / AC15 |
| T274 F13 / T285 F14 / T286 F15 list ORDER | **Lift human pinned**; store+JSON freeze F2/F3 |
| Placeholder Manual `--limit 5` + `--summary` | **DoD** AC1/AC7/AC15 |
| T216 preview `TAGS:` envelope | **DoD** F6 / AC3 |

## Declined (written)

| Item | Why |
|------|-----|
| JSON / store recency | F2 / F3 — T216 F7 + T283 analog |
| `--authority` flag | F9 |
| Briefing stanza | **T288** |
| `graph neighbors` CLI | **T293** |
| Forgotten-empty next | **T299** |
| leftover dest upsert | **T294** |
| `forget --match` two-pass | F14 |
| USER/SYSTEM TAGS GLOB | F29 — T285 F7 |
| T263 H2 / T240 F2 / clap 5 / rusqlite 0.40 | Standing |
| last-PR #202 Cursor | N/A empty — no T301 |

---

## Phase 1 — Red (required)

- [ ] AC1 CLI: tagged `DECISION:` pin vs newer `## Objective` dumps — human `--limit 5` first data row is the pin
- [ ] AC2: JSON `items[0]` stays recency dump
- [ ] AC3 unit: `preview_line` envelope → `DECISION:` not `TAGS:`; TAGS-only fallback non-empty (Agy m1)
- [ ] AC5 store: `list_authority_memories` GLOB-or-TAGS + needles
- [ ] AC16 unit: `prefer_fill_authority` rstest overlap / authority-only / recency-only / limit (Agy O1)
- [ ] Commit red (allowed)

## Phase 2 — Green

- [ ] F4 `list_authority_memories` in `query_store.rs` + trait
- [ ] F1 mix in `run_inventory` for human pinned only
- [ ] F5 retain `classify_pin_kind != Other`
- [ ] F6 `preview_line` uses `first_contentful_line` + empty fallback; forget/graph **inherit only**
- [ ] F35 helper uniqueness
- [ ] AC4 T216 store recency stays green
- [ ] AC6–AC14 stay green (JSON keys / summary / forgotten / untagged / chrome-only / exit 2 / no ASSISTANT: / list-forgotten share)
- [ ] F28: `forget_match_preview__role_prefix_stripped__max_100` / `forget_multi_preview__role_prefix_stripped__max_80` + graph human preview tests **stay green**
- [ ] Did **not** edit `forget.rs` / `graph.rs` production
- [ ] Commit green (allowed)

## Phase 3 — Docs

- [ ] CAPABILITIES: human pinned prefer-fills authority; JSON recency frozen
- [ ] CHANGELOG T287
- [ ] `memory list` after_help dual-truth sentence (F30 / AC17)
- [ ] OPERATIONS one-liner if still recency-only

## Phase 4 — Verify

- [ ] `cargo fmt --check`
- [ ] `cargo clippy -p ai-brains-cli -p ai-brains-store --all-targets -- -D warnings`
- [ ] `cargo nextest run -p ai-brains-cli -p ai-brains-store --profile ci`
- [ ] Full gate before publish (`dev-check.ps1` / implement-track)
- [ ] `ledgerful verify --scope fast` then `--scope full` at closeout
- [ ] Manual AC15 `cargo run -p ai-brains-cli -- memory list --limit 5` (not PATH unless owner asks install)

## Phase 5 — Review + closeout

- [ ] `conductor/tracks/trackT287-memory-list-authority/review.md`
- [ ] Read-only `codex-review` (F22)
- [ ] conductor.md T287 **Completed**; deferred closeout table
- [ ] implement-track Phase 6: push `track/T287-*` → PR → watch GHA `CI` green → squash-merge → prune. Never `git push origin main`. Never force-push.

---

## DoD (checkable)

- [ ] Human `--limit 5` includes ≥1 leading-line authority pin when any exist (AC1/AC15)
- [ ] JSON `items[0]` recency (AC2)
- [ ] Store `list_memories` ORDER unchanged (AC4)
- [ ] `--summary` counts unchanged (AC7)
- [ ] Forgotten / `forget --list-forgotten` recency (AC8/AC14)
- [ ] Preview skips TAGS envelope (AC3)
- [ ] No new clap flags / JSON keys / crate pins
- [ ] T288 / T293 / T299 not stolen

---

## Stop-before (even on go)

- Live `retention apply --confirm`
- Live `graph rebuild` (T300)
- Live `backup create --no-prune` (T295)
- Live `safety sync` without `--dry-run`
- T240 F2 `.env` write
- T263 H2 pin→Approved
- `cargo install` unless owner asks
- clap 5 / rusqlite 0.40
