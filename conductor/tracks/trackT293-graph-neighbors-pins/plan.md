# T293 Plan — graph neighbors pins first (human-only)

**Status:** **Completed** (FEATURE TX `0d731f26`; publish in progress). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F31 / AC1–AC14 + §13 AI fold-in
**Category:** UX / FEATURE
**Ledger TX (planning):** `83553530-cc14-4e4a-ad5e-cf366cf11a03` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `13843d9e-33be-4288-8979-534f1593d3ed` (DOCS)
**Ledger TX (implement):** FEATURE on **go** (new)

---

## AI fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F4/AC13:** `split_once(" · ")` exact; dots in first-line stay.
2. **F1:** `sort_by_key` `(rank, original_index)`; no `sort_unstable_by_key`.
3. **F31:** new `seed_memory_projection`; not T278 DROP COLUMN.
4. **AC2 case 6:** four-tier mixed exact order.
5. **AC3:** first pretty data row is not dump UUID `00000000-…0001`.
6. **F14/AC11:** PROTOCOL-COMPAT `:95`; OPERATIONS extend `:948`.
7. **F5 slip:** chrome-only is F25.

---

## Preflight (plan time — 2026-08-23; fold-in refresh)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `fe9fb89` (`main`, T293 plan). Parent `80a10d9` T292 `#208`. CLEAN at fold-in start. `origin/main` = `fe9fb89` until this commit. |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T274/T278. No T285–T292.** Hole is in **source + PATH**. **Do not `cargo install`.** |
| `graph neighbors d9183790-… --format human --limit 8` | **(12)** all `in RECALLS` session; PREVIEW `# Review of Track 254` / ````json` |
| T278 pin `b189ad20-… --format human` | **(3)** all `## Objective` sessions |
| Same pin `--format json` | three incoming `RECALLS` UUID order (`13d5625b`, `9c866cec`, `fd6035c8`) |
| `graph update --format human` | sparse E/N **0.123** (T300; not this track) |
| `--help` | default `auto`; no dual-truth prefer sentence |
| `preflight --summary` | Pinned **4048** (volatile; plan 4042 / OpenCode 4042). In-context **1/2/1** (plan 0/0/0). Word **1416** (plan 280 / OpenCode 314) |
| Last PR comments | #208 T292 — Cursor/Bugbot/reviews **empty**. **N/A. No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); serde_json **1.0.150** (crates.io 1.0.151); chrono **0.4.44**; rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** (3.932) — do not touch. `sync.rs` **#2**. `governed_common.rs` **#3**. `session_chrome.rs` **#6**. CLI `preflight.rs` **#8**. `graph.rs` **1073** not top-10 — prefer-fill here. |
| Ledger | 0 pending / 0 drift at scan; planning TX `83553530`; fold-in TX `13843d9e` |
| `ISSUES.md` | **Does not exist** (F22) |
| ledgerful search | `pretty_neighbor_rows` → `graph.rs:308`; `sort_neighbor_hits` → `:140` / `:386` |
| Online | clig.dev human-first + JSON stable; Neo4j captions; Wikidata preferred-rank (don’t strip); knowgraph grounding never strips; T180 pretty-order lift documented; T246 F9 JSON freeze |
| Skill | CAPABILITIES graph neighbors T278 PREVIEW row (F14); OPERATIONS `:948` already has neighbors paragraph |
| doctor | **4** warn (legacy `.changeguard` / sig-pin / sig-version / timings). :8081/:8083 **ok** at fold-in (OpenCode: unreachable — volatile) |

---

## Phase 0 — on go (re-verify)

- [x] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read `graph.rs` `neighbors` `:361` — pretty still uses F9-sorted hits; JSON still `format_neighbors_json`
- [x] Confirm `sort_neighbor_hits` still direction→label→id (`:140`)
- [x] Confirm `pretty_neighbor_rows` session arm still T278 caption (`:318`)
- [x] Confirm `NeighborHit` still three keys; `get_neighbors` SQL unchanged
- [x] Confirm clap Neighbors still default `auto` + five-token `value_parser` (`:2850`) — **no new flags**
- [x] Confirm `classify_pin_kind` still leading-line after envelope (`ranking.rs:122`) — **do not edit ranking.rs**
- [x] Confirm T278 AC3 still asserts PREVIEW contains `DECISION`, not first-row identity
- [x] Confirm `format_session_neighbor_preview` still pushes `" · "` (`:256`) — F4 `split_once` must match
- [x] Confirm **no** `memory_projection` insert helper in `graph_human_cli.rs` — AC3 writes **new** F31 helper; T278 DROP COLUMN stays fail-open only
- [x] Confirm PROTOCOL-COMPAT array-order is **`:95`** (not `:103` scan-roots)
- [x] Confirm OPERATIONS neighbors paragraph still **`:948`** — extend, do not add a second block
- [x] Confirm dispatch still `main.rs:5121` `GraphCommands::Neighbors`
- [x] Re-scan hotspots — `graph.rs` still not top-10; do not grow `project.rs`
- [x] Rescan `conductor/deferred.md` — T293 absorbed + #208 N/A; T294–T300 / T292 not stolen
- [x] Confirm #208 still empty Cursor; no mint; Dependabot `#61` still not this track
- [x] Re-dogfood `graph neighbors <id> --format human --limit 8` **read-only**. **Did not** pin production decisions; **did not** write `.env`; **did not** `graph rebuild`
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [x] FEATURE TX (new)
- [x] Did **not** `cargo install`; did **not** grow `projector.rs` / `queries.rs` / `ranking.rs` body / `session_chrome.rs`

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

- [x] `prefer_authority_neighbor_rows__dump_then_decision_memory__memory_first` (AC1)
- [x] `session_caption_body__memories_dot_decision__strips_prefix` (AC13) including `"1 memories · 1.2.3 dump"` remainder
- [x] `graph_neighbors__pretty__authority_before_dump_session` (AC3) — fail while pretty is F9 incoming-first
- [x] Commit red allowed

## Phase 2 — green pretty reorder

- [x] `pub(crate)` helpers in `graph.rs` (F18): `session_caption_body` (`split_once(" · ")`), `neighbor_authority_rank`, `prefer_authority_neighbor_rows` (`sort_by_key` `(rank, original_index)`)
- [x] Pretty path in `neighbors`: after `pretty_neighbor_rows`, prefer, then `format_neighbors_pretty`
- [x] AC2 case 6 four-tier mixed exact order
- [x] F31 `seed_memory_projection` in hermetic test file (not T278 DROP COLUMN)
- [x] JSON path: **no** prefer (F2)
- [x] AC2 rstest `#[case]`
- [x] AC4 JSON dump session still `neighbors[0]`
- [x] AC14 pretty `--limit 1` is authority; JSON `--limit 1` is dump

## Phase 3 — stay-green + peers

- [x] AC5 T278 session PREVIEW
- [x] AC6 T246 JSON keys
- [x] AC7 T262 pin neighbors json/pretty
- [x] AC8 feature-off exit 2
- [x] AC9 `sort_neighbor_hits` unit
- [x] T285 `recall_rank_v2_graph.rs` untouched / still green if run

## Phase 4 — docs + gate

- [x] CAPABILITIES graph row; PROTOCOL-COMPAT §5 array-order pretty note; OPERATIONS one sentence; GraphCommands after_help dual-truth (keep session PREVIEW sentence + json example); CHANGELOG
- [x] AC10 hermetic `--help` substring
- [x] `cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings`
- [x] Targeted nextest (`-p ai-brains-cli` + graph feature tests) then workspace gate (`dev-check` / nextest + deny + audit)
- [x] Manual AC12 `cargo run -p ai-brains-cli --features graph -- graph neighbors <id> --format human --limit 8` and `--format json`
- [x] `ledgerful verify --scope full`

## Phase 5 — review + publish

- [x] `conductor/tracks/trackT293-graph-neighbors-pins/review.md` phase-1
- [x] Codex/cross-model when FEATURE
- [x] Mark conductor **Completed**; append closeout residuals to `deferred.md`
- [x] Push `track/T293-*` ; PR ; `gh run watch --exit-status` ; `gh pr merge --squash --delete-branch`
- [x] Hygiene: `git fetch --all --prune`; point local `main` at `origin/main`; delete merged local `track/T293-*` only. Never `git push origin main`. Never force-push.

---

## DoD (checkable)

- [x] AC1–AC14 green (hermetic + units + docs + Manual AC12)
- [x] JSON neighbors still F9 direction→label→id (AC4/AC6/AC9)
- [x] T278 PREVIEW unchanged (AC5)
- [x] No 2-hop rows; no projector/rebuild; no `get_neighbors` SQL change
- [x] No clap 5 / rusqlite 0.40 / `.env` write / `graph rebuild` / `cargo install`
- [x] Medium+ review findings not silently dropped
- [x] FEATURE TX committed; conductor Completed only after publish hygiene


