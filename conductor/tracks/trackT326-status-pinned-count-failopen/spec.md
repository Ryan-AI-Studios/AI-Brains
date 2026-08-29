# T326 — `status` / `graph update` must not fake `pinned=0` on COUNT fail

- **Track ID:** T326-StatusPinnedCountFailOpen
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T326`.
- **Category:** BUGFIX / UX
- **Owner:** Grok
- **Source:** Last-PR Cursor Bugbot on [#237](https://github.com/Ryan-AI-Studios/AI-Brains/pull/237) (T320, `mergedAt` **2026-08-29T03:17:43Z**). Medium: when pinned-memory COUNT fails, the glance graph section invents `pinned=0` and still runs density assessment. An empty graph on a pin-rich vault can then show `live`/`skip` and `pinned=0` instead of failing that section open. Doctor already refuses to assess this case.
- **Depends on:** T320 ✅ unified `status` (`status.rs`); T213/T308 `GatherResult::PinnedCountFailed`; T300 `graph_health_report`
- **Blocks / feeds:** Glance + `graph update` honesty when `memory_projection` COUNT fails.
- **Absorbs:** `#237` Cursor comment `pulls/comments/3885361601` (still true on HEAD `d1c3bd3`)
- **Not absorbed (DoD):** T316 list preview; T308 floors; T320 four-section compose; clap 5
- **Research date:** 2026-08-29. Live `status.rs:329–340` `PinnedCountFailed` → `pinned_memories: 0` then `assess_graph_density`. Same pattern `commands/graph.rs:445–458`. Doctor `doctor.rs:901–904` **skips** (`cannot assess empty_lag without pins`). `graph_density.rs:63–68` documents doctor skip. Snapshot — re-verify at execute.
- **Ledger:** minted with T316 planning DOCS TX `66b597f7-faf9-4f3e-bb06-6af72811bdc6`. Implement **BUGFIX** TX on go.
- **Isolation:** Do **not** implement until go / `/plan-track T326` then **go**. Do **not** steal into T316. Do **not** retune floors. Do **not** grow `doctor.rs` (already correct). Do **not** `cargo install`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Pinned COUNT fail is fail-open, not fake zero.** `GatherResult::PinnedCountFailed` must not feed `assess_graph_density` with `pinned_memories: 0`. Glance graph section emits `error` (T320 F4). `graph update` must not report `live`/`skip` from a synthetic empty-lag.
2. **Copy doctor SOOT.** Doctor skip message already exists. Do not add a 16th doctor check.
3. **North star.** Capture independence: density gather honesty only. No new events. Floors stay 0.50.

---

## 2. Live baseline (mint 2026-08-29)

| Signal | Observation |
|--------|-------------|
| HEAD | `d1c3bd3` T320 `#238` / product `#237` `c3abe19` |
| Glance | `status.rs:329–340` invents `pinned=0` + assesses |
| Graph update | `graph.rs:445–458` same fake 0 |
| Doctor | `doctor.rs:901` skip — **SOOT** |
| Cursor | Medium, still true — empty-lag cannot be judged without a real pin count |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- Floors `MIN_EDGE_NODE_RATIO=0.50` frozen.
- Doctor 15-check matrix frozen.
- T320 envelope keys frozen except this error path.

---

## 6. Non-goals

Floor retune. Auto-rebuild. T316 preview. Growing `doctor.rs`. HTTP/TCP on glance.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| `#237` Bugbot `PinnedCountFailed` fake 0 | **Absorb** (this placeholder) |
| `graph.rs` same fake 0 | **Absorb** (same hole; full plan names both sites) |
| T316 memory-list preview | **Not stolen** (this mint) |
| T325 F8 recency | **Not stolen** |
| last-PR `#238` | **N/A empty** |

---

## 12. Touch map (sketch)

`crates/ai-brains-cli/src/commands/status.rs` `build_graph_section` `PinnedCountFailed` arm. Optionally `commands/graph.rs` `graph_health_report` same arm. **Do not** change `gather_density_snapshot` success path. **Do not** edit `doctor.rs`.
