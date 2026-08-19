# T272 — Preflight `--global` Safety skip must not hide capped-out Index rows

- **Track ID:** T272-PreflightGlobalSafetyIndexSkip
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** BUGFIX / UX
- **Owner:** —
- **Source:** Cursor Bugbot on PR [#179](https://github.com/Ryan-AI-Studios/AI-Brains/pull/179) (T264) — Medium “Safety IDs over-exclude Index”
- **Depends on:** T264 Completed (label+cap+span)
- **Absorbs:** #179 inline review: `safety_ids` is filled from the full Safety fetch window (`LIMIT 40`) **before** `take_round_robin` keeps eight items. Index/Recent still skip every id in that set, so memories capped out of Safety never appear there either and can vanish from the `--global` rollup.
- **Not absorbed:** T264 leftover-project drop from `recall --global` (F11 stands); T265 envelope; T266 format maze; T219 project-scoped selection

---

## 1. Objective

Under `--global`, Index/Recent skip should match **emitted** Safety ids (post-cap), not the pre-cap fetch window.

## 2. Problem (live 2026-08-18, HEAD `4088106`)

`crates/ai-brains-retrieval/src/preflight.rs`:

- `:329` `safety_ids.insert(memory_id)` for every fetched Safety row (global `LIMIT 40`)
- `:336–342` then `take_round_robin` keeps **8**
- `:467` Index `if safety_ids.contains(&memory_id) { continue; }`

Verified still true after T264 merge. Project-scoped (`LIMIT 10`, no cap) is mostly unchanged. Fits **no** T265–T271 placeholder (those are envelope / format / next / scan / nightly split / retention classify / ledger pane).

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Populate the Index/Recent skip set from **post-cap** Safety entries (the 8 that actually print), or rebuild `safety_ids` after `take_round_robin`. |
| **F2** | Project-scoped path stays: skip what Safety shows (fetch == shown). |
| **F3** | Do not drop leftover from `recall --global`. Do not change T264 labels/caps/span. |
| **F4** | Hermetic: two-project vault where a Safety-eligible pin is capped out of the 8 still appears in Index/Recent under `--global`. |

## 4. Verification sketch

- Unit/hermetic: fetch window > cap; capped-out id is absent from Safety body and **present** in Index.
- Existing T264 AC5/AC10 (labels + per-project Safety cap) stay green.
