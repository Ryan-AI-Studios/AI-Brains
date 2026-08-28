# T317 — Graph neighbors: RECALLS noise vs signal

- **Track ID:** T317-GraphNeighborsRecalls
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T317`.
- **Category:** UX / GRAPH
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `graph neighbors` 6/**5**; 19 `RECALLS` edges on a memory; `hierarchy` empty `synthesized_from`.
- **Depends on:** T293 ✅ human prefer-authority 1-hop (dump-session **sort**); T246 JSON freeze; T262 live projection; T67 `RECALLS` edges (do **not** delete the event type)
- **Blocks / feeds:** Graph as a daily tool. Sparse floors remain **T308** (frozen).
- **Absorbs:** Audit RECALLS spam + empty hierarchy honesty
- **Not absorbed (DoD):** T293 dump-session reorder; live `graph rebuild`; floor retune; JSON `kind`/`preview` keys; 2-hop; T312 recall graph-hop
- **Research date:** 2026-08-27. `graph.rs` `get_neighbors` pretty table includes `RECALLS` (tests require pin RECALLS survive rebuild). Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** live `graph rebuild`. Do **not** grow `projector.rs`. Human filter/cap only unless the plan proves JSON needs an opt-in flag. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Neighbors first page is signal.** Human `graph neighbors` must not be 19 undifferentiated `RECALLS` rows. Cap, group, or prefer non-RECALLS / authority (full plan). Keep at least one RECALLS visible if that is the only 1-hop (T67 meaning).
2. **Hierarchy empty is named.** `synthesized_from` empty prints why (no synth edges yet / sparse) + next-step — not a blank that looks like a bug.
3. **JSON 1-hop contract stays** unless the plan adds an explicit `--label` / `--exclude` (default JSON frozen like T293).
4. **North star.** Capture independence: pretty over existing edges. No fake SYNTHESIZED_FROM.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| T293 | Authority **sort** of existing 1-hop; does not cap RECALLS |
| Tests | `graph.rs` asserts RECALLS survive blocked rebuild — do not drop the edge type |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- T246 JSON keys/order unless opt-in flag.
- Floors `MIN_EDGE_NODE_RATIO=0.50` untouched.

---

## 6. Non-goals

Deleting `RECALLS` from the projector. Floor retune. 2-hop pretty. Cargo graph default-on.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| Audit neighbors 6/5 | **Absorb** |
| T308 R1 live E/N ~0.41 | **Decline** floor change |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

`crates/ai-brains-cli/src/commands/graph.rs` pretty neighbors/hierarchy. Not `queries.rs` `get_neighbors` unless the plan requires a filter flag for both formats.
