# T317 Review Log — Graph neighbors RECALLS cap + hierarchy leaf next

**Track:** T317-GraphNeighborsRecalls
**Category:** FEATURE / UX
**FEATURE TX:** `39e0e1e4-577c-4b18-a4d9-59101d163020`
**Branch:** `track/T317-graph-neighbors-recalls`

## Scope

- `crates/ai-brains-cli/src/commands/graph.rs` — `RECALLS_PRETTY_CAP`, `cap_recalls_pretty_rows`, `format_neighbors_pretty` arity (F31), `pretty_hierarchy_leaf`, `neighbors()` wire (prefer → cap → format)
- `crates/ai-brains-cli/tests/graph_human_cli.rs` — AC14 human cap + AC9 JSON uncapped
- `crates/ai-brains-cli/src/main.rs` — GraphCommands `after_help` dual-truth
- Docs: CAPABILITIES / PROTOCOL-COMPAT / OPERATIONS / CHANGELOG
- Conductor README status

**Not touched:** `projector.rs`, `queries.rs`, `project.rs`, `sync.rs`, contracts, floors.

## Internal review (R1)

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| R1-1 | high | Cap helper must hide RECALLS beyond 3 | `verified_fixed` | AC1–AC4 units green; hermetic AC14 `+2 more RECALLS` |
| R1-2 | high | Header must use pre-cap full hop count | `verified_fixed` | AC5; live Manual `(12)` with 3 data rows |
| R1-3 | high | Hierarchy leaf next nightly --status; AC9 no graph remediator | `verified_fixed` | AC7 + AC8 stay-green; Manual two-line leaf |
| R1-4 | high | JSON uncapped | `verified_fixed` | AC9 hermetic ≥4 RECALLS; live JSON n=12 |
| R1-5 | medium | F31 three unit callers arity | `verified_fixed` | `(2,0)` / `(2,0)` / `(51,0)` compile + stay-green |
| R1-6 | medium | Footer order limit then RECALLS | `verified_fixed` | AC17 unit |
| R1-7 | low-info | clippy `expect_fun_call` in AC17 | `verified_fixed` | switched to `unwrap_or_else(\|\| panic!(…))` |
| R1-8 | low-info | PATH until `cargo install` | `deferred` | F15 soft |
| R1-9 | low-info | Live N on `431f6505` moves (11→12) | `deferred` | hermetic SoT; Manual observed-data N=12 |
| R1-10 | low-info | Kept PREVIEW still `## Objective` | `deferred` | T278 honesty; cardinality is DoD |
| R1-11 | low-info | Sparse E/N ~0.41 floors | `deferred` | T308; not stolen |

## DoD matrix

| Item | Status | Evidence |
|------|--------|----------|
| Human 4+ RECALLS → 3 + `+N more`; header full N | Met | Units + hermetic + Manual N=12 / +9 |
| JSON lists all RECALLS | Met | AC9 + live 12 |
| Leaf `next: ai-brains nightly --status` | Met | AC7 + Manual |
| T293 prefer first; T246/T262 stay-green | Met | prefer units + live projection AC6/AC7 |
| No projector / get_neighbors SQL | Met | AC13 empty name check |
| Docs + after_help | Met | CAPABILITIES / PROTOCOL / OPERATIONS / CHANGELOG / after_help |

## Manual evidence (2026-08-28)

```text
cargo run -p ai-brains-cli --features graph --bin ai-brains -- graph neighbors 431f6505-… --format human
→ Neighbors of … (12)
→ 3 RECALLS data rows
→ +9 more RECALLS

cargo run … graph hierarchy 431f6505-… --format human
→ No SYNTHESIZED_FROM children (leaf).
→ next: ai-brains nightly --status

cargo run … --format json (hierarchy)
→ {"root":"431f6505-…","synthesized_from":[]}
```

## Cross-model (Codex CX1 — `review.codex.md`)

| ID | Severity | Disposition |
|----|----------|-------------|
| P0-001 | process | **Validated** — gates/publish outstanding at review time; closing this loop |
| P2-001 | medium | **Validated → fixed** — AC9 now asserts exact `neighbors.len()==5` and RECALLS count `==5` |
| P3-001 | low | **Validated → fixed** — review.md trailing whitespace; spec.md EOF blank trimmed |
| P3-002 | low | **False positive** — `for`/`map` building seed fixtures is not multi-case parameterization; AC2 already uses `rstest`; existing tree (51-row unit) uses the same fixture pattern |

## Gate

- `.\scripts\dev-check.ps1` — **PASS** (nextest **3593**, 1 skipped; deny + audit ok)
- `ledgerful verify --scope full` — **PASS**
- Ready to publish Phase 6
