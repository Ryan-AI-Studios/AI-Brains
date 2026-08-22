# T278 Plan — Session neighbor PREVIEW + honest density

**Status:** **Pending** (Planned). F0 = plan-only until **go**.
**Spec:** [spec.md](./spec.md) F0–F34 / AC1–AC14 + §13 AI fold-in
**Category:** FEATURE / UX
**Ledger TX (planning):** `977c5e7e-1043-4d5d-ab52-7803cd231f6a` (DOCS)
**Ledger TX (fold-in Agy):** `384ed242-bb9d-4125-9079-3f40b8d5486a` (DOCS)
**Ledger TX (fold-in OpenCode):** `0765b916-9f38-417b-a698-b39b657078dd` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go**

---

## AI fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors either harness. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F33 / AC5:** `session_neighbor_caption` → `String`; no `?` on session-arm I/O (Agy m1).
2. **F34 / AC14:** `pick_first_nonempty` required-pure (OpenCode m2/O1; Agy m2). No I/O stub.
3. **AC1:** `(0,"")` / `(1,"preview")` / whitespace no-dot / 80-cap + CJK via `truncate_preview_chars` (Agy O1/O2).
4. **§2.3:** `graph_density.rs` crate-root `:10–16` (OpenCode m1).
5. **Already:** F2 Unicode cap; F3 skip-loop; F4 fail-open; F14 same-file units.
6. **Decline:** always-dot when first blank; JSON object = three keys only; byte-slice truncate; OpenCode O2 empty-first hermetic AC3.

---

## Preflight (plan time — 2026-08-22)

| Check | Result |
|-------|--------|
| HEAD / tree | **Plan dogfood:** `400dd78` T284 `#193`. **Agy fold-in:** `46fc872`. **This OpenCode fold-in:** `5defcc5` (docs-only; product crates identical). CLEAN |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-21 05:55. **T270** on PATH (T246 pretty + T262 projection). **Do not `cargo install`.** |
| `preflight --summary` | Pinned **volatile** (plan 3476; OpenCode 3495; this fold-in **3515**); in-context 0/0/0; grants **0 of 3**; Scope `3581317d` |
| `project whoami` | `mismatch: false`; shell leftover `7d97a456` (T282 / T258 — not this track) |
| `graph update --format human` | `status: sparse` E/N **0.130** nodes 23082 edges 3005 pinned 39382 memory_nodes 20961; rem=`ai-brains graph rebuild` |
| Doctor `graph_density` | warn + rebuild; **15** checks; `graph_feature` ok |
| PATH `graph neighbors` of `b189ad20-…` | **(3)** `in RECALLS` `KIND session` **PREVIEW blank**. JSON 3-key hits |
| `graph session 13d5625b-…` | **(4)** memories **with** previews — text exists; neighbors hide it |
| Last PR comments | #193 T284 — **empty** (N/A). #188 closed by T284. No T285 |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2); uuid lock **1.23.1** (crates.io 1.25.0) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (3.953) — do not grow. `preflight.rs` #7 — do not grow. `doctor.rs` **1855** — do not grow. CLI `graph.rs` **914**; `graph_density.rs` **725**; `projector.rs` **351**; `main.rs` **4835** |
| Ledger | 0 pending / 0 drift at scan; planning TX `977c5e7e` |
| `ISSUES.md` | **Does not exist** (F21) |
| ledgerful search | `pretty_neighbor_rows` `graph.rs:252`; `memory_preview` `:234`; `get_session_memories` `queries.rs:107` |
| Online | CLIG human-first; Neo4j Browser captions from properties; TRACE-KG/RIDE 2026 typed sparse ≠ fake density; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `pretty_neighbor_rows` (`graph.rs` ~`:252–274`), `memory_preview` ~`:234–250`, `format_neighbors_pretty` ~`:157`, `neighbors` ~`:303`
- [ ] Re-read `get_session_memories` (`queries.rs` ~`:107`) — **do not change signature**
- [ ] Re-read density floors `graph_density.rs` ~`:10–16` + `assess_graph_density` — F27 small-vault `MIN_NODES=50`
- [ ] Re-read clap `GraphCommands::Neighbors` ~`:2534` / `after_help` ~`:2528`
- [ ] Confirm T262 AC6/AC7 still present in `graph_live_projection.rs`
- [ ] Rescan `conductor/deferred.md` — T278 rows already absorbed; no new overlapping open rows
- [ ] Confirm #193 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [ ] Re-dogfood `graph update --format human` + `graph neighbors <id> --format human` **read-only**. **Did not** live rebuild. **Did not** live pin
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [ ] FEATURE TX on go
- [ ] Did **not** `cargo install`; did **not** grow `doctor.rs` / `project.rs` / `projector.rs` / `graph_density.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit neighbors PREVIEW blank on session RECALLS | **DoD** F1–F4 / AC1–AC3 |
| Audit density sparse E/N | **Honesty regression** AC8/AC9 — do not retune / rebuild |
| T246 F10 memory-only PREVIEW | **Lift** F1 session |

## Declined (written)

| Item | Why |
|------|-----|
| Live `graph rebuild` | F8 |
| T213 floor retune / projector edges / Cargo default-on | F7/F10/F11 |
| 2-hop pretty / hierarchy captions / mermaid | F18/F19 |
| last-PR #193 Cursor | N/A empty |
| T279–T283 / leftover rebind / T240 F2 / clap 5 / rusqlite 0.40 | F12/F22 |
| Dependabot rusqlite `#61` | F12 — no T285 |
| OpenCode O2 empty-first hermetic AC3 | AC14 is skip-loop DoD |

---

## Phase 1 — Red (TDD)

- [ ] `format_session_neighbor_preview__zero_and_blank__zero_memories_no_dot` — AC1
- [ ] `format_session_neighbor_preview__count_and_first__dot_and_cap_80` — AC1 (`(1,"preview")` + CJK)
- [ ] `format_neighbors_pretty__session_recalls__preview_shows_memories` — AC2
- [ ] `pin__graph_on__neighbors_pretty__session_preview_nonblank` — AC3
- [ ] `pick_first_nonempty__blank_then_hello__some_hello` — AC14
- [ ] Commit red allowed

## Phase 2 — Green

- [ ] F14 `format_session_neighbor_preview` `pub(crate)` in `graph.rs`
- [ ] F34 `pick_first_nonempty` `pub(crate)` (skip trim-empty; `None` if all blank)
- [ ] F33 `session_neighbor_caption` → `String` (match/if let + warn; no `?`; calls F34)
- [ ] F1–F4 `pretty_neighbor_rows` session arm calls F33 helper
- [ ] Update T246 session fixture empty preview → AC2 caption
- [ ] F30 `GraphCommands` `after_help` additive
- [ ] AC4/AC5/AC6/AC7/AC9/AC14 stay green
- [ ] Commit green

## Phase 3 — Docs

- [ ] CAPABILITIES graph table: session PREVIEW `{n} memories · first line`
- [ ] OPERATIONS one sentence (captions; update ≠ rebuild)
- [ ] PROTOCOL-COMPAT §5: preview human-only; keys unchanged
- [ ] CHANGELOG T278
- [ ] Skill one-liner if graph section exists
- [ ] conductor Completed only on implement closeout — **not** this planning pass

## Phase 4 — Verify

- [ ] Targeted nextest: `-p ai-brains-cli graph` units; `--test graph_human_cli`; `--test graph_live_projection` (graph feature)
- [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] Primary review → `review.md`; mediums not silently dropped
- [ ] Cross-model `codex-review` (F20)
- [ ] Full workspace gate at closeout only
- [ ] Classify-only live `graph update --format human` (AC8) + `cargo run --features graph -- graph neighbors <id> --format human` (AC10). **No** live rebuild

## DoD (checkable)

- [ ] Hermetic pin → pretty PREVIEW contains `memories` (AC3)
- [ ] Session caption unit `{n} memories · first` + 80 cap + CJK (AC1)
- [ ] `pick_first_nonempty` skip-empty unit (AC14 / F34)
- [ ] Session I/O helper returns `String` (AC5 / F33)
- [ ] JSON neighbor keys still three (AC4)
- [ ] Live `graph update --format human` still not a false unlabeled live (AC8)
- [ ] No live `graph rebuild`
- [ ] No `cargo install`
- [ ] Diff omits `doctor.rs` / `project.rs` / `projector.rs` / `graph_density.rs` (AC13)
- [ ] implement-track Phase 6: push `track/T278-*` → PR → watch GHA `CI` green → squash-merge → prune (never `git push origin main`)

## Stop-before

- Live rebuild / `.env` rewrite / schtasks mutate / `cargo install` / live pin-as-implement
- Scope exceeds T278 (do not steal T279–T283, T277 live create, T275 bootstrap, T213 floors)
- Ambiguous spec vs src after Phase 0 — halt and ask
