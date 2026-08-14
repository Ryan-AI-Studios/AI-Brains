# T246 Plan — Graph human CLI presentation

**Status:** 🔄 **In Progress** (go 2026-08-13; ledger `9affa6c3`)  
**Spec:** [spec.md](./spec.md) F1–F20 / AC1–AC16 + §14 AI fold-in  
**Category:** UX / FEATURE  
**Ledger TX (on go):** `ledgerful ledger start T246-graph-human-cli --category FEATURE --message "TTY pretty for graph neighbors/hierarchy/session; JSON keys frozen; update human opt-in; no live rebuild"`

---

## AI fold-in (2026-08-13) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates planned work. AI2 two mediums are **must-pin** before go (graph-crate hierarchy-depth method; PROTOCOL-COMPAT array-order note).

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1 / M2** | AI1 | **Agree** | Already F10 / F11 |
| **AI1 M3** | AI1 | **Agree dispatch / decline passthrough** | No `other` arm |
| **AI1 M4** | AI1 | **Agree JSON default / decline one-liner** | F6 labeled lines |
| **AI2 M1** | AI2 | **Agree hard** | F7 new crate method + `MIN(depth)` |
| **AI2 M2** | AI2 | **Agree hard** | F14 sort-order row |
| **AI2 L3** | AI2 | **Agree hard** | JSON `limit.is_some()` |
| **AI2 L4** | AI2 | **Agree, KIND=14** | `source_version` is 14 chars |
| **AI2 L1–L2 / L5–L16** | AI2 | **Agree / affirm** | Pinned into F1–F15 / AC13 |
| **AI1 remapped ACs** | AI1 | **Decline** | Keep AC1–AC16 |

### Pins locked by fold-in

1. **F7:** `get_synthesized_hierarchy_with_depth`; no CLI CTE.  
2. **F9/F14:** JSON arrays sorted; PROTOCOL-COMPAT documents it.  
3. **F1:** clap reject; `format: String`; `is_terminal` crate.  
4. **F8:** JSON unlimited unless `--limit` is `Some`; short `-l`.  
5. **F2/F10:** widths 3/16/36/14; preview only `kind=memory`.  
6. **F6:** update default `json`; human = labeled `println!`.

---

## Preflight (plan time — 2026-08-13)

| Check | Result |
|-------|--------|
| `graph update` | Pretty JSON; **sparse** 1314/104; E/N **0.079**; remediation rebuild |
| Doctor | `graph_feature=available`; `graph_density` warn + rebuild |
| Recent pin `5df056c0-…` | `neighbors: []` — silent empty |
| Nil UUID | **Same** empty JSON as a real leaf |
| Session `3b4e95b8-…` | **5** memory ids |
| Neighbors `5a0e0a71-…` | 1× `incoming RECALLS` → **session** id (kind invisible) |
| Hierarchy | Empty `synthesized_from` on pins (leaf / no nightly synthesis) |
| `GraphCommands` | No `--format` / `--limit` |
| `get_neighbors` callers | CLI + **recall graph-boost** — API frozen |
| PROTOCOL-COMPAT | neighbors compact; update pretty JSON |
| T74 smoke | Piped `graph update` must stay JSON |
| clap / serde_json / is-terminal / rusqlite | 4.6.1 / 1.0.150 / 0.4.17 / 0.39.0 — **no bumps** (crates.io clap 4.6.6, serde_json 1.0.151) |
| Ledger | 0 pending, 0 unaudited drift |
| T247 / T245 / T243 | Completed — no rewrite |
| Live rebuild | **Not** run (F13) |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Graph neighbors JSON-only | deferred.md / audit P2 | **DoD** F1–F5 |
| hierarchy/session human | README | **DoD** F1/F3/F7 |
| `graph update` pretty | README ≥8 | **F6** `--format human` only |
| Placeholder F1–F4 | spec draft | All absorbed, refined (kind + node-miss) |
| T213 skill one-liner | T213/T232 leftover | **F14** |
| Freshness / auto-rebuild / projector | T213/T232 | **Not absorbed** |

---

## Phase 0 — Ledger + impact (on go)

- [x] `ledgerful ledger status --compact` — 0 pending, 0 unaudited drift
- [x] `ledgerful ledger start T246-graph-human-cli --category FEATURE` — tx `9affa6c3-bd1d-42e5-8df5-9b2a217430e1`
- [x] `ledgerful scan --impact` — MEDIUM (dirty conductor/planning only; no code yet)
- [x] Confirm no other agent is editing `graph.rs` / `queries.rs`

## Phase 1 — Red → Green: pure formatters (F1 / F2 / F3 / F9 / AC1–AC6)

- [x] `resolve_graph_format(&str, is_tty)` — clap rejects unknown; `_` fail-closed json after parser
- [x] `format_neighbors_pretty` widths 3/16/36/14 + `in`/`out` + preview-if-memory
- [x] Empty strings: F3 exact copy + `next: ai-brains graph update`
- [x] Hierarchy indent 2×depth; no box-drawing
- [x] Pretty `--limit` `… and N more` via `clamp_list_limit`
- [x] JSON emit helper: frozen keys; F9 sort; compact

## Phase 2 — `node_kind` + neighbors wiring (F10 / F11 / AC7 / AC8 / AC15)

- [x] `GraphSearch::node_kind` → `Option<String>` (`query_row` + `optional()`)
- [x] F3 flow: `node_kind` then `get_neighbors` only if present
- [x] Pretty KIND + `preview_line` **only** when kind is `memory`
- [x] **Do not** change `NeighborHit` fields or `get_neighbors` signature
- [x] Existing recall graph-boost tests still pass (T74 smoke + live-graph append)

## Phase 3 — Hierarchy + session + `--limit` (F7 / F8 / F9 / AC4 / AC9 / AC16)

- [x] **`get_synthesized_hierarchy_with_depth`** + `MIN(depth)` `GROUP BY` (AC16 diamond)
- [x] JSON hierarchy still calls **old** id-only method, then lex-sort
- [x] Session pretty + previews; JSON compact lex-sorted ids (CLI-local sort)
- [x] `--limit` `-l`; JSON `if limit.is_some() { clamp } else { usize::MAX }`
- [x] clap `--format`/`--limit` on Neighbors / Hierarchy / Session only

## Phase 4 — `graph update --format human` (F6 / AC10)

- [x] Default / `json` / `auto` = existing `to_string_pretty` (no TTY switch)
- [x] `--format human` one labeled `println!` per field; `remediation:` only when `Some`
- [x] T213 serde units unchanged
- [x] T74 piped JSON parse unchanged

## Phase 5 — Feature-off + clap (F4 / F15 / AC11 / AC12)

- [x] Feature-off neighbors with `--format pretty` still exit 2 (both stubs)
- [x] Invalid `--format` clap exit 2
- [x] `after_help` on **`GraphCommands` enum**
- [x] Rebuild unchanged (no format / limit)

## Phase 6 — Docs (F14 / AC14)

- [x] `Docs/CAPABILITIES.md` graph table
- [x] `Docs/PROTOCOL-COMPAT.md` §5 TTY/pipe split + **array-order row**
- [x] `Docs/OPERATIONS.md` short paragraph
- [x] Skill one-liner (`.agents/skills/ai-brains/SKILL.md`)
- [x] `CHANGELOG.md` T246 row only

## Phase 7 — Live dogfood + gate (on go; **no rebuild**)

- [x] Re-discover a pin that has neighbors if plan-time UUIDs moved (AC13) — plan-time `5a0e0a71` still has 2 incoming RECALLS
- [x] Neighbors TTY scannable (`in RECALLS` + session kind)
- [x] Same command piped → compact JSON
- [x] Nil UUID `--format pretty` → `No graph node`
- [x] Session with memories → rows + previews (`3b4e95b8` 5 rows)
- [x] `graph update` still JSON; `--format human` labeled
- [x] Targeted nextest + clippy (full gate after review)
- [x] Review log + conductor Completed only after go+ship — CX2 PASS; PR #159 squash `06cdcde`

---

## Isolation checklist

- [x] No live `graph rebuild`
- [x] No T213/T232 assessor/remediation rewrite
- [x] No recall ranking change
- [x] No new crates / lock bumps
- [x] No `AI_BRAINS_KEY` print/commit
