# T317 Plan — neighbors RECALLS cap + hierarchy leaf next-step

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** UX / GRAPH
**Ledger (planning):** DOCS `0db2a64d-6ae6-4c25-b2fc-3a6db62d0dfa`

---

## Preflight (plan time — 2026-08-28)

| Check | Result |
|-------|--------|
| HEAD / tree | Product `dae7df3` T313 `#233` CLEAN. Branch `track/T317-graph-neighbors-recalls`. `origin/main` = `dae7df3` (ahead **0** at plan-write). |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. **T293 on PATH.** T312/T315/T314/T313 **not**. T317 hole **is**. |
| `preflight --summary` (PATH) | Pinned **4549**; in-context **0/0/0**; `Total Word Count: 785` (PATH-behind T315) |
| `graph update --format human` | sparse; E/N **0.414**; floors frozen |
| `graph neighbors 431f6505-… --format human` | **11/11** `in RECALLS` dump sessions; header `(11)` |
| `graph neighbors … --format json` | `n=11` `RECALLS=11` |
| `graph hierarchy 431f6505-…` | `No SYNTHESIZED_FROM children (leaf).` no `next:`; JSON `synthesized_from: []` |
| `graph.rs` | **1539** lines; `neighbors` `:524`; `format_neighbors_pretty` 3-arg `:158`; `pretty_hierarchy_leaf` `:125`; prefer `:364` |
| `queries.rs` `get_neighbors` | `:62` UNION ALL — **do not edit** |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; serde_json **1.0.150**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#233` `mergedAt` **2026-08-28T12:28:19Z**; comments/reviews **[]** — **N/A empty**. `#230` → **T325** already. |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; this TX `0db2a64d` |
| Hotspots | `project.rs` #1 **3.724** — do not touch. `graph.rs` not top-10. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin / live rebuild | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit neighbors RECALLS spam | **DoD** F1 / F9 / AC1 / AC5 / AC11 / AC14 |
| Audit hierarchy empty | **DoD** F2 / AC7 / AC12 |
| T293 prefer-authority | **Stay-green** F5 — cap after prefer |
| T246 JSON / T262 AC9 no graph remediator on leaf | **Affirm** F3 / AC8 |
| T308 R1 E/N ~0.41 | **Decline** F7 |
| last-PR `#233` Cursor | **N/A empty** F19 |
| last-PR `#230` F8 recency | **T325** — not stolen |
| T316 / T318–T324 / clap 5 | **Not stolen** / **Decline** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains`
- [ ] Re-read `neighbors()` `:524–571` + `format_neighbors_pretty` `:158–184` + `pretty_hierarchy_leaf` `:125–127`
- [ ] Re-read T293 `prefer_authority_neighbor_rows` `:364` + AC9 `:1321` + `graph_human_cli.rs:532`
- [ ] Confirm `queries.rs:62` and `projector.rs:70–81` still RECALLS as today
- [ ] Re-dogfood `431f6505-50d7-5176-8cda-f8ba2534fe14` (record live N; AC11 observed-data)
- [ ] Confirm clap lock still **4.6.1**; floors still `0.50`
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] Confirm T325 placeholder still Pending (do not steal F8 recency)
- [ ] `ledgerful ledger start T317-graph-neighbors-recalls --category FEATURE`
- [ ] **Do not** `cargo install` / live `graph rebuild` / live production `pin` / `.env` rewrite / clap 5

## Phase 1 — Red

- [ ] `cap_recalls_pretty_rows__eleven_recalls__keeps_three_hidden_eight` (AC1)
- [ ] `cap_recalls_pretty_rows__at_or_under_cap__unchanged` rstest 0/1/3/4 (AC2)
- [ ] `cap_recalls_pretty_rows__mixed_labels__keeps_all_non_recalls` (AC3)
- [ ] `cap_recalls_pretty_rows__authority_recalls__kept_before_dumps` (AC4)
- [ ] `format_neighbors_pretty__recalls_hidden__header_total_and_footer` (AC5) — must **fail** on 3-arg helper
- [ ] `pretty_hierarchy_leaf__nightly_status_next` (AC7) — must **fail** on one-line leaf
- [ ] Hermetic AC14 `graph_neighbors__human__caps_recalls_with_footer`
- [ ] **Write AC9** `graph_neighbors__json__no_recalls_cap` (green-on-arrival — passes on HEAD; do **not** skip as stay-green)
- [ ] Confirm red tests **fail** on current tree (no cap helper; leaf has no `next:`)

## Phase 2 — Green

- [ ] `RECALLS_PRETTY_CAP = 3` + `cap_recalls_pretty_rows` (F1 / F32)
- [ ] `format_neighbors_pretty` arity F31; header `full_hop_count`; footer `+{n} more RECALLS` (F9)
- [ ] `pretty_hierarchy_leaf` two-line F2
- [ ] Wire `neighbors()` : prefer → cap → format (spec §5.2)
- [ ] Update existing `format_neighbors_pretty__incoming_and_outgoing__…` to pass `2, 0`
- [ ] Production: no `unwrap`/`expect`/`panic`
- [ ] `graph.rs` production net <80 **or** split `graph_neighbors_pretty.rs` (F18)

## Phase 3 — Stay-green

- [ ] AC8 AC9 unit (`pretty_hierarchy_leaf__no_graph_update_or_rebuild__ac9`)
- [ ] T262 AC6/AC7 (`graph_live_projection.rs`)
- [ ] T293 prefer rstest + dump-then-decision
- [ ] T246 JSON key units + `sort_neighbor_hits`
- [ ] Rebuild RECALLS survive (`graph.rs:1017` / `:1065`)
- [ ] `graph_human_cli.rs` leaf `contains("No SYNTHESIZED_FROM children (leaf).")`
- [ ] Feature-off exit 2 (AC16)
- [ ] AC17 two footers (`--limit` + RECALLS hidden)

## Phase 4 — Docs

- [ ] CAPABILITIES `:104` + `:464–465`
- [ ] PROTOCOL-COMPAT `:94–95` (pretty cap + hierarchy leaf next lift)
- [ ] OPERATIONS `:960` (leaf only; no-neighbors / empty-session still no `next:`)
- [ ] GraphCommands `after_help` F26 sentence
- [ ] CHANGELOG Unreleased

## Phase 5 — Manual + isolation

- [ ] AC11 `cargo run --features graph -- graph neighbors 431f6505-50d7-5176-8cda-f8ba2534fe14 --format human` (or live N)
- [ ] AC12 hierarchy pretty + JSON on the same id
- [ ] AC13 in-repo `crates/` name-only; `projector.rs` / `queries.rs` empty in product diff
- [ ] PATH-behind is **not** a fail

## Phase 6 — Review + gate + Complete (only after go)

- [ ] Phase-1 review log `review.md` until clean
- [ ] Cross-model `codex-review` (F23 FEATURE)
- [ ] Targeted nextest + clippy `-p ai-brains-cli --all-targets -- -D warnings` (graph feature on)
- [ ] Full gate (`dev-check` / AGENTS.md)
- [ ] `ledgerful verify --scope full`
- [ ] Conductor **Completed**; deferred residuals; FEATURE TX commit
- [ ] Publish: push `track/T317-*` → PR → watch GHA `CI` green → squash-merge → prune. Never `git push origin main`.

---

## DoD (checkable)

- [ ] Human neighbors of a 4+ RECALLS memory is not a RECALLS-only wall: header is full 1-hop N, **3** RECALLS rows, `+{N-3} more RECALLS`
- [ ] JSON still lists every 1-hop RECALLS
- [ ] Empty hierarchy pretty names `next: ai-brains nightly --status` and still omits graph update/rebuild
- [ ] T293 prefer, T246 JSON, T262 RECALLS, AC9 forbids stay-green
- [ ] No projector / `get_neighbors` SQL change
- [ ] Status stays **Pending** until go; after go + merge, **Completed**

## Isolation

No live rebuild. No floor retune. No `cargo install`. No T316–T325 steal. Never `git push origin main`.
