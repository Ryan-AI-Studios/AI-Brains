# Track review: T278-GraphDensityPreview

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT278-graph-density-preview`  
**Date:** 2026-08-22  
**HEAD:** `46fc872`  

---

## Summary

Track T278 addresses a concrete graph usability flaw identified during the 2026-08-21 CLI audit:
When inspecting the 1-hop graph neighborhood of a memory via `ai-brains graph neighbors <memory_id> --format human`, incoming `RECALLS` edges from sessions display as `KIND session` with a completely blank `PREVIEW` column. Because the primary 1-hop relationship of a pinned memory is to its session, operators and agents were presented with uninformative UUID-only tables.

T278 resolves this through targeted caption formatting and strict density honesty:
1. **Session-Kind Neighbor Previews (T246 F10 Lift):** In `crates/ai-brains-cli/src/commands/graph.rs` (`pretty_neighbor_rows`), when `kind == "session"`, the CLI queries the session's associated memories via `get_session_memories` and formats the PREVIEW column as `"{n} memories · {first_preview}"` (capped at 80 characters).
2. **Fail-Open Error Handling:** If memory lookups for a session encounter database or lock errors, the preview gracefully falls back to `"{n} memories"` or `"0 memories"` (logging a warning) rather than crashing the command.
3. **Density Honesty Invariant:** Preserves existing density floors (`MIN_EDGE_NODE_RATIO = 0.50`, `MIN_MEMORY_COVERAGE = 0.10`, `MIN_NODES = 50`) and avoids artificial edge inflation or unverified live vault rebuilds.
4. **Frozen Protocols & Contracts:** Machine-readable JSON output for `graph neighbors` remains frozen with the exact three keys (`external_id`, `label`, `direction`), ensuring human-only preview improvements never break downstream automations.

The specification and test plan are well-bounded, maintain capture independence, and adhere to CLIG output standards.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Fail-open isolation in `pretty_neighbor_rows` (F4 / AC5):** Ensure all database and lock operations inside the session branch of `pretty_neighbor_rows` are isolated with `match` or `if let` and log via `tracing::warn!`, avoiding `?` propagation so `neighbors` always completes successfully even if a session's preview lookup fails.
- **m2: Iterative candidate fallback for session previews (F3):** If the first sorted memory ID in a session yields an empty or whitespace-only preview, iterate through subsequent memory IDs in the session until a non-empty preview is found before formatting the caption.

### Opportunities (O)
- **O1: Consistent UTF-8 preview truncation (F2):** Ensure `truncate_preview_chars` handles multi-byte UTF-8 character boundaries cleanly and mirrors the 80-character cap used across `memory_preview`.
- **O2: Pure unit tests for `format_session_neighbor_preview` (F14 / AC1):** Add focused unit tests covering `(0, "")`, `(1, "preview")`, `(5, "long...")`, and whitespace inputs directly in `graph.rs`.

---

## What Looks Solid

1. **Meaningful Captions Over UUIDs:** Adopts graph browser best practices (such as Neo4j Browser entity property captions) to make session neighbor rows immediately informative.
2. **Strict Density Invariant Maintenance:** Avoids tempting anti-patterns like relaxing density thresholds or inventing synthetic graph edges to artificially inflate edge/node ratios.
3. **Zero Mutation on Read Commands:** All preview generation is performed purely against existing SQLite projections (`graph_node`, `graph_edge`, `memory_projection`) without mutating graph state or event logs.
4. **Hotspot Restraint:** Zero edits to `project.rs`, CLI `preflight.rs`, `sync.rs`, or `doctor.rs`. Changes are localized to `crates/ai-brains-cli/src/commands/graph.rs`.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Graph sparse E/N ~0.11; neighbors PREVIEW blank | Absorbed into DoD (F1–F4 / AC1–AC3 / AC8) | Solved via session captions; density stays honest |
| T246 F10 memory-only PREVIEW | Lifted (F1) | Extended to support session-kind rows |
| Live `graph rebuild` | Declined (F8) | Operator action out-of-band; Stop-Before |
| T213 floor retuning / projector edge inflation | Declined (F7 / F11) | Preserves honest typed-provenance density |
| 2-hop neighbor rows / hierarchy captions | Declined (F18 / F19) | Preserves 1-hop model; `graph session` available |
| Last-PR Cursor #193 | N/A (empty) | Scanned with 0 findings |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#193](https://github.com/Ryan-AI-Studios/AI-Brains/pull/193) (merged 2026-08-22, T284 `Retention Work dispose counts and apply sample ids`).
- **Cursor Comments:** 0 comments (`[]` on PR #193).
- **Disposition:** N/A (no pending findings).

---

## Research / Tools Notes

- **Graph UX & Density Standards:** CLIG output standards recommend property captions for human tables while keeping machine-readable JSON stable. TRACE-KG (2026) and Adaptive GraphRAG research confirm that compact, typed provenance graphs naturally have lower edge-to-node ratios than untyped open-domain graphs.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44), `uuid` (1.23.1).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,476 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search pretty_neighbor_rows`: Located at `crates/ai-brains-cli/src/commands/graph.rs:252`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
