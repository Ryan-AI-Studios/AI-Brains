# Track review: T276-Leftover7d97Rebind

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT276-leftover-7d97-rebind`  
**Date:** 2026-08-21  
**HEAD:** `61fd3cb`  

---

## Summary

Track T276 resolves a significant retrieval and multi-project visibility issue identified during the 2026-08-21 CLI audit:
The vault contains ~18,000 legacy memories owned by a leftover project ID (`7d97a456`) spanning 11 unregistered repository roots. When running `ai-brains recall --global`, the volume of leftover memories monopolizes the initial lexical candidate window (`candidate_depth = 15`), pushing out high-value pins belonging to the current active repository. Furthermore, `--global` hits lacked project labels, making it impossible for operators and agents to distinguish between memories from the current project and leftover dumps.

T276 addresses this without mutative memory migrations or silent exclusions:
1. **Candidate Prefer-Fill:** When running `--global` from an active project context, `recall_full` fetches candidates scoped to the preferred project first, followed by unscoped global candidates, merging them up to `candidate_depth` before re-ranking.
2. **Preserving True Global Scope:** Leftover memories and other project memories are *not* dropped or filtered out; they remain searchable and appear in the global candidate set.
3. **Pretty Tagging for Global Hits:** In pretty output under `--global`, each hit is prefixed with a project tag (`[8hex]` or upgraded `display_label`), matching the convention established in T264 for preflight.
4. **General & Idiomatic Design:** Does not hardcode `7d97a456` anywhere in retrieval code; the preferred project is determined dynamically from the active project context (`preferred_project_id`).

The specification and test plan are well-bounded, preserve capture independence, and maintain event sourcing integrity.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Project tag display ordering in CLI recall pretty (AC4 / F4):** When formatting pretty hit lines under `--global`, ensure the leading project tag (`[3581317d]` or `[C:\dev\ai-brains]`) appears before the `[score=...]` bracket with a single separating space (e.g. `[3581317d] [score=-21.238 | session=c6555534] <uuid>: ...`).
- **m2: Safe deduplication in `prefer_project::merge_preferred_then_global` (AC1):** Ensure `merge_preferred_then_global` tracks seen memory IDs with a `HashSet<String>` so that a memory ID present in both preferred and global candidate sets is never duplicated in the merged result.

### Opportunities (O)
- **O1: Clean signature evolution for `format_pretty_hit_line`:** Extend `format_pretty_hit_line` with an optional `project_tag: Option<&str>` parameter (or a dedicated wrapper) to ensure zero code duplication between `recall.rs` and `sync.rs`.
- **O2: Early exit in candidate merging:** If `preferred.len() >= depth`, `merge_preferred_then_global` can immediately truncate `preferred` to `depth` and return without iterating through `global`.

---

## What Looks Solid

1. **Prefer-Fill vs Hard Exclusion:** Rejecting hard-exclusion of leftover memories (T264 F11) keeps `--global` truly global while ensuring the active project's pins are never starved of candidate slots.
2. **Dynamic Project Resolution:** Deriving `preferred_project_id` from the pre-clear active project context avoids brittle UUID hardcoding and works universally across all multi-project workflows (e.g. `fcb8a40f` in Ledgerful).
3. **Consistent Pretty UX:** Reusing `preflight_pretty` tagging (`display_label` + `truncate_chars(32)`) creates a unified visual language across `preflight --global` and `recall --global`.
4. **Zero Event Store Pollution:** Avoids complex data migrations or `MemoryMoved` events, keeping `rebind-path` (T259) as the dedicated compensating mechanism.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Leftover `7d97a456` ~18k / `--global` junk | Absorbed into DoD (F1–F6 / AC1–AC5) | Solved via prefer-fill + pretty project tags |
| Recall leftover-first under `--global` | Absorbed (prefer-fill + label) | Resolved without dropping leftover data |
| `--exclude-project` clap flag | Declined (F20) | Prefer-fill makes flag unnecessary |
| Memory reclassify / `MemoryMoved` | Declined (F7) | Preserves immutable event log (T259 F5) |
| Live leftover `rebind-path --write --yes` | Declined (F9) | Operator action out-of-band |
| Shell leftover on `context --show` | Declined (F11 → T282) | Properly isolated in Track T282 |
| `project list` cwd-first | Declined (F11 → T283) | Properly isolated in Track T283 |
| PR #188 Bugbot Mediums | Declined (F26 → T284) | Properly tracked in Track T284 |
| Last-PR Cursor #190 | N/A (empty) | Scanned with 0 findings |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#190](https://github.com/Ryan-AI-Studios/AI-Brains/pull/190) (merged 2026-08-21, T275 `Discovery grants first-run`).
- **Cursor Comments:** 0 comments (`[]` on PR #190).
- **Disposition:** N/A (no pending findings).

---

## Research / Tools Notes

- **Multi-Tenant Search:** In multi-project repositories, biasing search toward the active workspace while maintaining visibility across the global vault is standard local-first practice.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44), `uuid` (1.23.1).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,352 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search rebind_path_alias`: Located at `crates/ai-brains-control-plane/src/grants.rs:287`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
