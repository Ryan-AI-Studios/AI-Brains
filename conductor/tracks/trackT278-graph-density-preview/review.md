# T278 review log — Graph neighbor previews + honest density

**Track:** T278-GraphDensityPreview
**Status:** Completed (full gate green; Phase 6 pending this commit)
**FEATURE TX:** `13cc210d-e29b-48b0-b370-2f858a092eb9`
**HEAD (implement):** `track/T278-graph-density-preview`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC14 / DoD | **PASS** — red then green; session PREVIEW `{n} memories · first`; JSON keys frozen; density floors untouched; no live rebuild |
| R1b | Explore subagent (read-only DoD) | **PASS** — no P0–P3; omit-list clean; F33/F34/AC1–AC14 static DoD met |
| CX1 | Codex gpt-5.6-luna | **FAIL** — P1-001 process (full gate/closeout), P2-001 AC5 fail-open test, P2-002 AC3 PREVIEW cell |
| R2 | Implementer | P2-001/P2-002 fixed; P1-001 residual closeout |
| CX2 | Codex gpt-5.6-luna | **PASS** (product DoD). P2-001/P2-002 verified_fixed. P1-001 residual closeout only |
| Gate | `dev-check.ps1` + `ledgerful verify --scope full` | **PASS** nextest **3280** / 1 skipped |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

| id | severity | description | source | files | required_fix | status | evidence |
|----|----------|-------------|--------|-------|--------------|--------|----------|
| P1-001 | high (process) | Full gate, durable review, conductor closeout unfinished at CX1 time | CX1 | conductor + review.md | Complete Phase 5–6 | `verified_fixed` | `dev-check` 3280 passed / 1 skipped; `ledgerful verify --scope full` exit 0 |
| P2-001 | high | AC5 fail-open had no executable SQL-err path | CX1 | `tests/graph_live_projection.rs` | Hermetic `memory_preview` error → neighbors exit 0 | `verified_fixed` (CX2) | `pin__graph_on__neighbors_pretty__session_preview_sql_err_exit_0` PASS |
| P2-002 | high | AC3 asserted tautological non-whitespace on the whole row | CX1 | `tests/graph_live_projection.rs` | Parse PREVIEW cell (col 73) for `1 memories · DECISION` | `verified_fixed` (CX2) | `pin__graph_on__neighbors_pretty__session_preview_nonblank` PASS |

R1/R1b: no product findings.

## DoD matrix (AC1–AC14)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `format_session_neighbor_preview__zero_and_blank__zero_memories_no_dot` (`0 memories`, whitespace no ` · `); `format_session_neighbor_preview__count_and_first__dot_and_cap_80` (`1 memories` + `preview` + ` · `, 80-cap + CJK `…`) |
| AC2 | Met | `format_neighbors_pretty__session_recalls__preview_shows_memories` — header + `in` + `RECALLS` + `session` + `2 memories`; JSON helper still `incoming`/`external_id`/`label` only |
| AC3 | Met | `pin__graph_on__neighbors_pretty__session_preview_nonblank` — hermetic pin, no rebuild, pretty contains `session` + `memories` |
| AC4 | Met | `graph_neighbors__json_and_pretty__frozen_keys_and_dir` + AC2 JSON helper — object keys `memory_id`/`neighbors`; hit keys `external_id`/`label`/`direction` |
| AC5 | Met | `session_neighbor_caption` → `String`; session arm has no `?` on `get_session_memories` / `memory_preview`; `node_kind()?` stays before kind branch; memory-kind `?` unchanged |
| AC6 | Met | `graph_neighbors__format_pretty__feature_off_exit_2` exit 2 + `FEATURE_UNAVAILABLE` |
| AC7 | Met | `graph_neighbors__format_xml__clap_invalid_value` exit 2 |
| AC8 | Met | Live `cargo run --features graph -- graph update --format human`: `status: sparse` `density: warn` `edge_node_ratio: 0.130…` `remediation: ai-brains graph rebuild`. Not an unlabeled live JSON blob. **Did not rebuild.** |
| AC9 | Met | T262 AC6/AC7 still green in `graph_live_projection`; T213 sparse fixture unit untouched |
| AC10 | Met | `cargo run --features graph -- graph neighbors 3c1c3eb0-… --format human` → `session` PREVIEW `1 memories · ## Objective`. PATH may stay T270 until `cargo install` (F15). **Did not pin. Did not rebuild.** |
| AC11 | Met | CAPABILITIES graph table + command row; OPERATIONS captions vs update≠rebuild; PROTOCOL-COMPAT human-only preview; CHANGELOG T278; tracked skill `.claude/skills/ai-brains/SKILL.md` one-liner |
| AC12 | Met | `cargo clippy -p ai-brains-cli --all-targets --features graph -- -D warnings` exit 0; no production `unwrap`/`expect`/`panic` in graph.rs (tests only) |
| AC13 | Met | Diff omits `project.rs` / `preflight.rs` / `doctor.rs` / `sync.rs` / `projector.rs` / `graph_density.rs` |
| AC14 | Met | `pick_first_nonempty__blank_then_hello__some_hello` — skip blanks, `Some("hello")` / `Some("pin")` / `None`; `n=3` still `3 memories · hello` |

## Targeted gates (R1)

```text
cargo nextest run -p ai-brains-cli --features graph format_session_neighbor_preview pick_first_nonempty format_neighbors_pretty graph__help__names_session_preview
  7 passed (AC1/AC2/AC14 + existing pretty + F30 after_help)

cargo nextest run -p ai-brains-cli --features graph --test graph_live_projection --test graph_human_cli
  12 passed (incl. AC3 + AC4 + T262 AC6/AC7)

cargo nextest run -p ai-brains-cli --test graph_human_cli graph_neighbors__format_pretty__feature_off
  1 passed (AC6)

cargo nextest run -p ai-brains-cli graph_neighbors__format_xml
  1 passed (AC7)

cargo clippy -p ai-brains-cli --all-targets --features graph -- -D warnings
  exit 0

cargo fmt --check
  exit 0
```

## Manual (classify-only)

```text
cargo run -q -p ai-brains-cli --features graph -- graph update --format human
  status: sparse
  density: warn
  nodes: 23099
  edges: 3022
  pinned_memories: 39559
  memory_nodes: 20969
  edge_node_ratio: 0.1308281743798433
  note: sparse: edge/node ratio below typed-lineage floor 0.5 …
  remediation: ai-brains graph rebuild

cargo run -q -p ai-brains-cli --features graph -- graph neighbors 3c1c3eb0-3405-5001-a065-8836dccc1b8c --format human
  Neighbors of 3c1c3eb0-3405-5001-a065-8836dccc1b8c (1)
  DIR LABEL            ID                                   KIND           PREVIEW
  in  RECALLS          c8608b51-cd0f-4c6c-8987-1a537186ec59 session        1 memories · ## Objective
```

No live `graph rebuild`. No `cargo install`. No `.env` rewrite.
