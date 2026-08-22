# Track review: T283-ProjectListCwdFirst

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT283-project-list-cwd-first`  
**Date:** 2026-08-22  
**HEAD:** `dd57150`  

---

## Summary

Track T283 resolves a human-interface prioritization defect in `ai-brains project list`:
Currently, `ai-brains project list` renders projects sorted strictly by memory count (`ORDER BY memory_count DESC, project_id ASC`). On operator vaults with large historical or imported datasets (such as a leftover `7d97a456` project with ~18k memories), the leftover project is printed as the very first data row, while the active repository’s project (`3581317d`) is buried further down the table (e.g. 4th row).

T283 fixes this while strictly preserving machine-readable contracts:
1. **Human CWD-First Reordering (F1):** On human-formatted output (`--format human` and default TTY), the CLI resolves the current working directory’s registered path owner (`resolve_path_alias_for_location`) and promotes that project to index 0 of the displayed table. All other projects retain their stable memory-descending relative ordering.
2. **Frozen JSON Machine Contract (F2):** Machine-readable `--format json` remains strictly ordered by `memory_count DESC, project_id ASC`, ensuring automated ingestion pipelines and scripts that expect size-descending ordering are not broken.
3. **Preserving Footer & Store Behavior (F3 / F11):** The store SQL query and the unaliased suggestion footer (`project_list_footer.rs`) continue to receive the original unpromoted slice.
4. **Hotspot Restraint (F9):** Implementation is isolated to a new module `crates/ai-brains-cli/src/commands/project_list_order.rs`, avoiding helper sprawl in hotspot `project.rs`.

The plan is well-bounded, adheres to CLIG guidelines, and maintains capture independence.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Fail-open resolution of `cwd_owner` in `list()` (F1 / F26):** Ensure that any non-critical errors during `std::env::current_dir()`, `collect_git_identity`, or `resolve_path_alias_for_location` fail-open to `None`, cleanly falling back to default memory-descending output without terminating the command.
- **m2: Single-allocation stable promote implementation (F1 / AC1):** In `promote_cwd_owner`, construct the output `Vec` with exact capacity `rows.len()`, pushing the matched item to index 0 followed by all other elements to guarantee that no row is duplicated or dropped.

### Opportunities (O)
- **O1: Documentation refresh for `OPERATIONS.md` (F19 / AC11):** Update the `OPERATIONS.md` Listing Projects documentation from legacy column headers to the current T212 format (`label`, `project_id`, `memories`, `last_activity`, `path`).
- **O2: Pure unit tests with parameterized edge cases (AC1–AC2):** Add unit tests in `project_list_order.rs` testing promotion of the first element, a middle element, the last element, an unknown ID, an empty string, and an empty input slice.

---

## What Looks Solid

1. **Human-Centric Discoverability:** Solves the exact confusion where operators inspecting project lists from a repository were presented with unrelated leftover projects as the primary entry.
2. **Stable JSON Wire Contract:** Correctly decouples human visual prioritization from machine JSON outputs, honoring CLIG standards.
3. **Footer Stability:** Passing the original unpromoted vector to `print_unaliased_footer` preserves T267 unaliased suggestion logic.
4. **Hotspot Restraint:** Encapsulates the permutation logic in `project_list_order.rs`, adding only ~15 lines to `project.rs`.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| `project list` leftover-first | Absorbed into DoD (F1–F8 / AC1–AC6 / AC10) | Solved via human-only cwd path-owner promotion |
| JSON freeze vs human-only | Absorbed (F1 / F2) | Human is cwd-first; JSON stays size-descending |
| T267 footer suggestion logic | Declined (F3) | Preserved by passing original unpromoted vector |
| Reorder JSON `projects[]` | Declined (F2) | Preserves stable machine contract |
| `--sort` CLI flag | Declined (F5) | Keeps human default sensible without flag bloat |
| Last-PR Cursor #198 | N/A (empty) | Scanned with 0 findings |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#198](https://github.com/Ryan-AI-Studios/AI-Brains/pull/198) (merged 2026-08-22, T282 `context --show leftover shell vs .env plus KEY redact`).
- **Cursor Comments:** 0 comments (`[]` on PR #198).
- **Disposition:** N/A (no pending findings).

---

## Research / Tools Notes

- **CLI Listing Guidelines:** clig.dev recommends placing the most relevant contextual information first for human readers while maintaining stable, deterministic ordering for machine parsers opting into `--json`.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44), `uuid` (1.23.1), `tokio` (1.52.3).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,633 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search list_projects_detail`: Located at `crates/ai-brains-store/src/query_store.rs:584`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
