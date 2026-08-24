# T293 Review Log — graph neighbors pins first

**Track:** T293-GraphNeighborsPins  
**Category:** FEATURE / UX  
**FEATURE TX:** `0d731f26-15f1-4f98-bea9-751fd18a70cb`  
**Branch:** `track/T293-graph-neighbors-pins`

## Phase-1 (implementer)

| ID | Severity | Description | Source | Files | Status | Evidence |
|----|----------|-------------|--------|-------|--------|----------|
| R1 | low-info | Clippy `useless_conversion` on `indexed.into_iter()` inside `zip` | clippy `-D warnings` | `graph.rs` prefer helper | verified_fixed | Removed `.into_iter()`; clippy clean |
| R2 | — | AC1–AC14 hermetic/units/docs | implement | `graph.rs`, `graph_human_cli.rs`, `main.rs`, Docs | verified_fixed | Targeted nextest 12/12 T293 + stay-green AC5–AC9 + AC8 feature-off |

### DoD checklist (phase-1)

- [x] Pretty prefer-authority after `pretty_neighbor_rows`; JSON untouched (F1/F2)
- [x] `sort_by_key` `(rank, original_index)` stable; no unstable
- [x] `split_once(" · ")` exact (AC13)
- [x] F31 `seed_memory_projection` new helper (not T278 DROP COLUMN)
- [x] No projector / get_neighbors / ranking.rs body / 2-hop
- [x] Dual-truth after_help + CAPABILITIES / PROTOCOL-COMPAT `:95` / OPERATIONS `:948` / CHANGELOG
- [x] Manual AC12: live `b189ad20` chrome-only F25 (`## Objective` first); JSON F9 UUID order; hermetic AC3 SoT
- [x] `ledgerful verify --scope fast` exit 0 (fmt + workspace clippy + workspace nextest + deny + audit)
- [x] Full `dev-check` + `ledgerful verify --scope full` exit 0
- [x] Codex cross-model (`review.codex.md`) — product PASS; process P1s closed by gate/publish

## Cross-model (Codex gpt-5.6-luna)

| ID | Severity | Disposition |
|----|----------|-------------|
| P1-1 | process | Gates incomplete at review time → closing with full gate + this log |
| P1-2 | process | Uncommitted / FEATURE pending / In Progress → closing with commit + publish |
| P0 / P2 / P3 | — | None |

**Verdict (product):** PASS (no product findings). Process P1s resolved by completing Phase 5–6.

### Residuals appended to `deferred.md`

| Residual | Notes |
|----------|-------|
| PATH until `cargo install --features graph` | F15 |
| Live chrome-only 1-hop still dump-first | F25 honest |
| Sparse E/N ~0.12 | T300 |
| T294–T300 placeholders | Not stolen |
