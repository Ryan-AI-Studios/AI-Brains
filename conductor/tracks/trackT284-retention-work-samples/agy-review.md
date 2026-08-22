# Track review: T284-RetentionWorkSamples

**Harness:** Antigravity (`agy`)
**Track:** `conductor/tracks/trackT284-retention-work-samples`
**Date:** 2026-08-21
**HEAD:** `da6f316`

---

## Summary

Track T284 resolves two Medium severity findings reported by Cursor Bugbot on PR [#188](https://github.com/Ryan-AI-Studios/AI-Brains/pull/188) (T270) regarding retention reporting honesty and audit sample accuracy:
1. **Work Table Hiding Dispose Rows:** In `ai-brains retention plan --format human`, the `Work` table previously filtered rows based on the class's *dominant* mechanism (`c.mechanism`). In classes where held items outnumber or tie CE items (such as `secret` with 10 pinned and 2 expired envelopes), the class is classified as `held`. Because `totals.would_ce_wipe > 0`, the CLI printed the `Work` header and `next: apply`, but printed zero data rows underneath.
2. **Apply Audit Samples Overrun by Overlay Pins:** When `prepare_retention_apply` appended a `RetentionApplied` event, it populated `sample_ids` by iterating classes in alphabetical order. Because `memory_legacy` (the T270 inventory overlay) was sorted first and contained 5 sample pin IDs, the event's capped 5 sample IDs were filled entirely by retained pins, completely omitting the IDs of the content keys or turns actually slated for disposal.

T284 fixes both issues cleanly:
- Adds optional, backward-compatible fields to `RetentionClassBucket` (`would_ce_wipe`, `would_projection_delete`, `dispose_sample_ids`) with `#[serde(default, skip_serializing_if = "...")]`.
- Updates `format_retention_pretty` to render Work data rows whenever a class has non-zero dispose counts, regardless of its dominant mechanism.
- Updates `append_retention_applied` (`audit_sample_ids`) to prioritize dispose sample IDs from active disposal classes over retained overlay pins.
- Preserves `Nothing to dispose.` on inventory-only vaults (0 CE / 0 projection delete).

The plan is well-bounded, adheres to ISO 27001:2022 A.8.10 / GDPR deletion logging best practices, and maintains full protocol compatibility.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Fallback in `format_retention_pretty` when `dispose_sample_ids` is empty (F6):** In `format_retention_pretty`, if `c.dispose_sample_ids` is empty but `class_dispose_count(c) > 0` (such as when deserializing a legacy report), fall back to `&c.sample_ids` so the SAMPLES column is never blank.
- **m2: De-duplication in `audit_sample_ids` (F7):** In `audit_sample_ids`, ensure duplicate IDs across classes or streams are deduplicated before capping at 5 items.

### Opportunities (O)
- **O1: Contract test for serde field omission (AC5 / F37):** In `ai-brains-contracts`, add an explicit test verifying that when `would_ce_wipe == 0`, `would_projection_delete == 0`, and `dispose_sample_ids` is empty, serialized JSON matches the exact 5 keys of the baseline schema.
- **O2: Stale rustdoc comment cleanup in `class_based_retention.rs` (F38):** Update the comment above `dominant_mechanism` to accurately describe majority selection with `BTreeMap` last-wins tie-breaking.

---

## What Looks Solid

1. **Directly Addresses Both Bugbot Mediums:** Solves the exact issues identified in PR #188 without introducing breaking changes to the core retention engine.
2. **DTO Protocol Compatibility:** Using `skip_serializing_if` ensures the live inventory JSON output remains byte-compatible with existing consumers on 0-dispose vaults.
3. **Audit Compliance Alignment:** Ensuring `RetentionApplied.sample_ids` records actual disposal targets aligns directly with ISO 27001:2022 A.8.10 deletion evidence requirements.
4. **Hotspot Restraint:** Zero changes to `project.rs`, CLI `preflight.rs`, `sync.rs`, or `doctor.rs`. Edits are strictly isolated to `retention.rs` and `class_based_retention.rs`.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| PR #188 Work table hides dispose rows | Absorbed into DoD (F1/F6 / AC1/AC3) | Solved via class-level dispose counters in Work |
| PR #188 Apply audit samples prefer inventory | Absorbed into DoD (F7 / AC2) | Solved via `audit_sample_ids` prioritization |
| T270 F9 Work dispose-only | Lifted (F1) | Filtered by dispose counts instead of dominant string |
| T270 F8 `Nothing to dispose.` | Affirmed (F1 / AC4) | Preserves clean inventory-only display |
| Live `retention apply --confirm` | Declined (F16) | Hermetic tests provide sufficient DoD |
| Last-PR Cursor #192 | N/A (empty) | Scanned with 0 findings |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#192](https://github.com/Ryan-AI-Studios/AI-Brains/pull/192) (merged 2026-08-22, T277 `Fail-closed usable create under current key`).
- **Cursor Comments:** 0 comments (`[]` on PR #192).
- **PR #188 Comments:** 2 Medium findings (`Work table hides dispose rows` and `Apply audit samples prefer inventory`) — both are the exact subject of this track.

---

## Research / Tools Notes

- **Deletion Audit Standards:** ISO 27001:2022 A.8.10 and GDPR logging guidelines require recording what was deleted (sample identifiers, method, class) rather than sampling unaffected retained inventory.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44), `uuid` (1.23.1).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,429 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search dominant_mechanism`: Located at `crates/ai-brains-control-plane/src/class_based_retention.rs:686`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
