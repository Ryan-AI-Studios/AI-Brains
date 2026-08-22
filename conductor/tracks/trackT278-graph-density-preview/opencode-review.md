# Track review: T278-GraphDensityPreview

**Harness:** OpenCode (`opencode`)
**Track:** `conductor/tracks/trackT278-graph-density-preview`
**Date:** 2026-08-22
**HEAD:** `5defcc5` (fold-in of `agy-review.md`; product crates identical to `400dd78`)

---

## Summary

T278 fixes the T246-era graph usability hole: `graph neighbors <id> --format human` prints a blank `PREVIEW` cell for incoming `RECALLS` session neighbors (a just-pinned memory's only 1-hop is its session). The plan fills that cell with a session caption `"{n} memories · first line"` (T246 F10 lift) via a **fail-open** helper, while keeping density floors, the `graph rebuild` remediator, JSON keys, and `NeighborHit`/`get_session_memories` signatures frozen.

This review re-verified every load-bearing anchor against the live tree at HEAD `5defcc5` (post-`agy-review` fold, docs-only; product crates identical to `400dd78` T284 `#193`). All named code locations, fixtures, tests, clap surface, density constants, docs rows, and pin versions match. No Blockers, no Majors. Three minor items and two opportunities.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)

- **m1 — `graph_density.rs` line/path nit (spec §2.3).** Spec says density floors at `graph_density.rs :14–16`. The module lives at crate root `crates/ai-brains-cli/src/graph_density.rs` (not under `commands/`), and the constants are at `:10–16` (`MIN_PINNED=100` :10, `MIN_NODES=50` :12, `MIN_EDGE_NODE_RATIO=0.50` :14, `MIN_MEMORY_COVERAGE=0.10` :16). Values all match the spec exactly; only the line/prefix hints drift by a couple of lines. Phase 0 re-read already covers this.
- **m2 — AC14 skip-loop testability hedge.** The skip-empty-first-preview loop (F3 / Agy m2) is I/O-bound inside `session_neighbor_caption`, so the AC14 unit is only "testable if the skip-loop is extracted, otherwise with a small stub." Spec leaves it conditional. Tighten to a deterministic extraction: a pure `pick_first_nonempty(previews: &[String]) -> Option<String>` (or `(n, first)` pair) over already-fetched previews, with the count staying the full list length. This makes AC14 a pure unit, matches F14's "pure-helper" spirit, and avoids an I/O stub.
- **m3 — Preflight snapshot drift (spec §2.1 / plan preflight).** Plan snapshot says HEAD `400dd78` / `46fc872` and pinned **3476**; today's tree is HEAD `5defcc5` (the `agy-review.md` fold, docs-only, un-pushed) with pinned **3495** and E/N **0.131** (nodes 23093, edges 3019, pinned 39536, memory_nodes 20966). Harmless — the product `src/` is still T284, and the spec already pins "re-verify at execute." Nothing to change beyond Phase 0 re-scan.

### Opportunities (O)

- **O1 — Pure skip-loop selector (folds m2).** Extracting `pick_first_nonempty(&[String]) -> Option<String>` makes AC14 hermetic and reuses `truncate_preview_chars` unchanged for the whole caption. Cheap, on-scope, and satisfies "no I/O in the same-file unit."
- **O2 — Assert the empty-first-recap in the hermetic AC3 too.** AC3 only asserts a non-blank PREVIEW row exists; add the case where the session's first sorted memory has no content (or is whitespace) so the fallback is proven end-to-end, not only in the pure unit. Optional; the unit (AC14) is the real DoD.

---

## What looks solid

1. **Fail-open is structural, not bolted on.** F33/AC5 makes `session_neighbor_caption` return `String` with `match`/`if let` + `tracing::warn`, never `?`. Verified `pretty_neighbor_rows` at `graph.rs:252–274` currently has `kind == "memory" → memory_preview(ctx, …)?` and `else → String::new()` — the lift point matches exactly. `tracing` is already used in `graph.rs` (`tracing::info!` at `:291`), so no new dependency.
- **JSON surface truly frozen.** `NeighborsOutput { memory_id, neighbors: Vec<NeighborHit> }` (`graph.rs:14–18`); `NeighborHit` fields `external_id`/`label`/`direction` (`queries.rs:7–11`) unchanged; `format_neighbors_json` emits only those. The freeze lock test `graph_human_cli.rs:213–264` asserts the exact key sets — this plan adds no keys. `graph_density.rs` and `projector.rs` untouched (AC13).
- **Density honesty preserved end-to-end.** Floors (`MIN_EDGE_NODE_RATIO=0.50`, `MIN_MEMORY_COVERAGE=0.10`) untouched; `graph update --format human` live report re-verified today: `status: sparse`, `density: warn`, E/N **0.131**, `remediation: ai-brains graph rebuild`. AC8 keeps the honesty regression check.
- **Session walk reuse is sound.** `get_session_memories` (`queries.rs:107`, recursive IN_SESSION → RECALLS/SOURCE_FOR + legacy turn path) is exactly what `graph session` pretty uses (`graph.rs:474`), and the count/sort semantics match F3 ("sort ids lexicographic"). No signature change.
- **TDD shape is concrete and hermetic.** Red names in plan Phase 1 match ACs (AC1, AC2 fixture lift, AC3 via `graph_live_projection`, AC14). Hermetic vaults only; live classify-only is explicitly non-DoD (F26/AC10/AC8).
- **Deferred / last-PR disposition is complete.** `deferred.md` rows 239–255 dispositioned (Absorb / Lift / Decline), #193 comments verified `[]` (N/A), Dependabot `#61`/`#62` declined with no T285, `ISSUES.md` confirmed absent. No mint gap.

---

## Deferred fold-in table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Graph sparse E/N ~0.13; neighbors PREVIEW blank on session RECALLS | **Absorb** F1–F4 / AC1–AC3 / AC8 — captions DoD; density honesty regression | Solid. Live re-verify today confirms sparse + remediator. |
| T246 F10 memory-only PREVIEW | **Lift** F1 — session added, memory unchanged | Correct lift point verified at `graph.rs:260`. |
| T213 floor retune / projector edges / `graph rebuild` live / default-on | **Decline** F7/F8/F10/F11 | Honest-dens n, Stop-Before respected. |
| 2-hop pretty / hierarchy captions / mermaid / batch `node_kinds` | **Decline** F18/F19 | 1-hop stays; `graph session` already pretty. |
| Dependabot rusqlite `#61` 0.40.2 / chrono `#62` 0.4.45 | **Decline** F12 — standing freeze; **no T285** | Lock verified: rusqlite 0.39.0, chrono 0.4.44, clap 4.6.1, serde_json 1.0.150, uuid 1.23.1. Ecosystem newer (4.6.6 / 1.0.151 / 0.4.45 / 1.25.0) — intentionally not bumped. |
| last-PR Cursor #193 | **N/A** — comments `[]` | Re-verified `[]` today. |
| T279–T283 / leftover rebind / T240 F2 / clap 5 / DTO keys | **Decline** F22/F12/F31 | Peers stay out of scope. |

## Last-PR Cursor comments

- **Scanned PR:** [#193](https://github.com/Ryan-AI-Studios/AI-Brains/pull/193) (merged 2026-08-22, T284 `fix(retention): Work dispose counts and apply sample ids`).
- **Cursor comments:** 0 — `gh pr view 193 --comments` returned `[]`; review comments API also empty.
- **Open PR on HEAD:** Dependabot remotes only (`#61`, `#62`).
- **Disposition:** N/A (no pending findings). No T285 minted.

## Research / tools notes

- **Live code opened (all resolved):**
  - `graph.rs`: `pretty_neighbor_rows` `:252–274` (memory-only preview at `:260`); `memory_preview` `:234–250` (`preview_line(…, 80)`); `format_neighbors_pretty` `:157–183`; `neighbors` `:303–341`; `format_neighbors_json` `:185–193`; fixture `:686–703` (session preview `String::new()`, the AC2 supersede); `sort_neighbor_hits` `:140–147`; `node_kind` in `queries.rs:224`.
  - `queries.rs`: `get_session_memories` `:107` — signature/recursion confirmed; do not change.
  - `graph_density.rs`: floors at `:10–16`; `assess_graph_density` `:167` (env overrides at `:18–21`).
  - `doctor.rs`: `check_graph_density` `:868–909` (soft; matrix is 15 checks) — not grown.
  - `main.rs`: `GraphCommands::Neighbors` `:2530–2542`, `after_help` `:2528`; clap `--format xml` rejects with exit 2 (`:227–239`).
  - Tests: `graph_live_projection.rs:44–106` (AC6/AC7 hermetic, no PREVIEW assert yet); `graph_human_cli.rs:17–36` (feature-off exit 2 + `FEATURE_UNAVAILABLE`), `:213–264` (frozen JSON keys).
  - Docs: `PROTOCOL-COMPAT.md:94–95` (keys unchanged; array order), `CAPABILITIES.md:102` and `:459` (PREVIEW currently "memory kind only" — the sentence this track updates).
- **Pins (lockfile, today):** clap **4.6.1** (crates.io latest 4.6.6, 2026-08-06; no clap 5); rusqlite **0.39.0** (0.40.2 exists, not bumped); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); uuid **1.23.1** (1.25.0). Workspace **0.1.1**, rustc 1.95.0, nextest 0.9.140.
- **Research:** CLIG human-first output; Neo4j Browser caption-from-properties pattern (direct support for "captions, not raw ids"); TRACE-KG/RIDE 2026 / T213 research for typed-sparse graphs (do not densify with untyped edges); clap 4.6.6 `after_help` docs. All support the design; no new crate needed.
- **ai-brains / ledgerful:**
  - `ai-brains preflight --summary`: Scope `3581317d`, pinned **3495**, grants **0 of 3** (T275 hermetic) — capture independence holds.
  - `ai-brains graph update --format human` (live): `sparse` / `warn` / nodes 23093 / edges 3016 / E-N **0.131** / rem `ai-brains graph rebuild`.
  - `ledgerful ledger status --compact`: 0 pending / 0 unaudited drift. `ledgerful scan --impact`: CLEAN (HEAD `5defcc5`).
  - Did not run `ledgerful search`/`ask` again — the named symbols were opened directly in `src/` (stronger evidence).

## Verdict: **Planned**

The plan is accurate against live `src/` (product code is T284; HEAD moved only for the `agy-review.md` docs fold). Minor items are cosmetic (line anchor, test-structure tightening, snapshot drift) and belong in the F33/AC14 fold-in, not a re-plan. Ready for `/implement-track` on **go** after Phase 0 re-verify.
