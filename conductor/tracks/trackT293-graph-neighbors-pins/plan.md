# T293 Plan — graph neighbors pins first (human-only)

**Status:** **Pending** (Planned; implement on **go**). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F30 / AC1–AC14
**Category:** UX / FEATURE
**Ledger TX (planning):** `83553530-cc14-4e4a-ad5e-cf366cf11a03` (DOCS)
**Ledger TX (implement):** FEATURE on **go** (new)

---

## Preflight (plan time — 2026-08-23)

| Check | Result |
|-------|--------|
| HEAD / tree | `80a10d9` (`main`, T292 `#208`). CLEAN. `origin/main` = HEAD (`00`). |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T274/T278. No T285–T292.** Hole is in **source + PATH**. **Do not `cargo install`.** |
| `graph neighbors d9183790-… --format human --limit 8` | **(12)** all `in RECALLS` session; PREVIEW `# Review of Track 254` / ````json` |
| T278 pin `b189ad20-… --format human` | **(3)** all `## Objective` sessions |
| Same pin `--format json` | three incoming `RECALLS` UUID order (`13d5625b`, `9c866cec`, `fd6035c8`) |
| `graph update --format human` | sparse E/N **0.123** (T300; not this track) |
| `--help` | default `auto`; no dual-truth prefer sentence |
| `preflight --summary` | Pinned **4042** (volatile); in-context **0/0/0**; word **280** |
| Last PR comments | #208 T292 — Cursor/Bugbot/reviews **empty**. **N/A. No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); serde_json **1.0.150** (crates.io 1.0.151); chrono **0.4.44**; rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** (3.932) — do not touch. `sync.rs` **#2**. `governed_common.rs` **#3**. `session_chrome.rs` **#6**. CLI `preflight.rs` **#8**. `graph.rs` **1073** not top-10 — prefer-fill here. |
| Ledger | 0 pending / 0 drift at scan; planning TX `83553530` |
| `ISSUES.md` | **Does not exist** (F22) |
| ledgerful search | `pretty_neighbor_rows` → `graph.rs:308`; `sort_neighbor_hits` → `:140` / `:386` |
| Online | clig.dev human-first + JSON stable; Neo4j captions; Wikidata preferred-rank (don’t strip); knowgraph grounding never strips; T180 pretty-order lift documented; T246 F9 JSON freeze |
| Skill | CAPABILITIES graph neighbors T278 PREVIEW row (F14) |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `graph.rs` `neighbors` `:361` — pretty still uses F9-sorted hits; JSON still `format_neighbors_json`
- [ ] Confirm `sort_neighbor_hits` still direction→label→id (`:140`)
- [ ] Confirm `pretty_neighbor_rows` session arm still T278 caption (`:318`)
- [ ] Confirm `NeighborHit` still three keys; `get_neighbors` SQL unchanged
- [ ] Confirm clap Neighbors still default `auto` + five-token `value_parser` (`:2850`) — **no new flags**
- [ ] Confirm `classify_pin_kind` still leading-line after envelope (`ranking.rs:122`) — **do not edit ranking.rs**
- [ ] Confirm T278 AC3 still asserts PREVIEW contains `DECISION`, not first-row identity
- [ ] Re-scan hotspots — `graph.rs` still not top-10; do not grow `project.rs`
- [ ] Rescan `conductor/deferred.md` — T293 absorbed + #208 N/A; T294–T300 / T292 not stolen
- [ ] Confirm #208 still empty Cursor; no mint; Dependabot `#61` still not this track
- [ ] Re-dogfood `graph neighbors <id> --format human --limit 8` **read-only**. **Did not** pin production decisions; **did not** write `.env`; **did not** `graph rebuild`
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [ ] FEATURE TX (new)
- [ ] Did **not** `cargo install`; did **not** grow `projector.rs` / `queries.rs` / `ranking.rs` body / `session_chrome.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit `graph neighbors` dump sessions U=7 | **DoD** F1–F6 / AC1–AC4 / AC12 |
| Placeholder Manual `--format human --limit 8` | **DoD** AC12 |
| Placeholder JSON freeze vs human-only | **DoD** F2 / AC4 |
| Placeholder PREVIEW `{n} memories · first line` | **DoD** F6 / AC5 |
| T278 2-hop | **Decline** F3 |
| last-PR #208 Cursor | **N/A empty** — no T301 |

---

## Phase 1 — red (required first)

- [ ] `prefer_authority_neighbor_rows__dump_then_decision_memory__memory_first` (AC1)
- [ ] `session_caption_body__memories_dot_decision__strips_prefix` (AC13)
- [ ] `graph_neighbors__pretty__authority_before_dump_session` (AC3) — fail while pretty is F9 incoming-first
- [ ] Commit red allowed

## Phase 2 — green pretty reorder

- [ ] `pub(crate)` helpers in `graph.rs` (F18): `session_caption_body`, `neighbor_authority_rank`, `prefer_authority_neighbor_rows`
- [ ] Pretty path in `neighbors`: after `pretty_neighbor_rows`, prefer, then `format_neighbors_pretty`
- [ ] JSON path: **no** prefer (F2)
- [ ] AC2 rstest `#[case]`
- [ ] AC4 JSON dump session still `neighbors[0]`
- [ ] AC14 pretty `--limit 1` is authority; JSON `--limit 1` is dump

## Phase 3 — stay-green + peers

- [ ] AC5 T278 session PREVIEW
- [ ] AC6 T246 JSON keys
- [ ] AC7 T262 pin neighbors json/pretty
- [ ] AC8 feature-off exit 2
- [ ] AC9 `sort_neighbor_hits` unit
- [ ] T285 `recall_rank_v2_graph.rs` untouched / still green if run

## Phase 4 — docs + gate

- [ ] CAPABILITIES graph row; PROTOCOL-COMPAT §5 array-order pretty note; OPERATIONS one sentence; GraphCommands after_help dual-truth (keep session PREVIEW sentence + json example); CHANGELOG
- [ ] AC10 hermetic `--help` substring
- [ ] `cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Targeted nextest (`-p ai-brains-cli` + graph feature tests) then workspace gate (`dev-check` / nextest + deny + audit)
- [ ] Manual AC12 `cargo run -p ai-brains-cli --features graph -- graph neighbors <id> --format human --limit 8` and `--format json`
- [ ] `ledgerful verify --scope full`

## Phase 5 — review + publish

- [ ] `conductor/tracks/trackT293-graph-neighbors-pins/review.md` phase-1
- [ ] Codex/cross-model when FEATURE
- [ ] Mark conductor **Completed**; append closeout residuals to `deferred.md`
- [ ] Push `track/T293-*` ; PR ; `gh run watch --exit-status` ; `gh pr merge --squash --delete-branch`
- [ ] Hygiene: `git fetch --all --prune`; point local `main` at `origin/main`; delete merged local `track/T293-*` only. Never `git push origin main`. Never force-push.

---

## DoD (checkable)

- [ ] AC1–AC14 green (hermetic + units + docs + Manual AC12)
- [ ] JSON neighbors still F9 direction→label→id (AC4/AC6/AC9)
- [ ] T278 PREVIEW unchanged (AC5)
- [ ] No 2-hop rows; no projector/rebuild; no `get_neighbors` SQL change
- [ ] No clap 5 / rusqlite 0.40 / `.env` write / `graph rebuild` / `cargo install`
- [ ] Medium+ review findings not silently dropped
- [ ] FEATURE TX committed; conductor Completed only after publish hygiene
