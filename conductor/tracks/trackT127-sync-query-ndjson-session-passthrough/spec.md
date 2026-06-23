# Track T127: `sync query` NDJSON Session-ID Passthrough

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — data fidelity; NDJSON records lose session context.
**Source:** v0.1.1 verification opportunity #6.

---

## Problem Statement

In `sync.rs`, the NDJSON path for `sync query` hardcodes `session_id: None` in the ChangeGuard records section (T112 change). This was correct for the search-scope fix (don't filter by session), but the OUTPUT records should still carry their original `session_id` from the source data if present.

Currently, the NDJSON record emitted for ChangeGuard hits includes:
```json
{"session_id": null, "project_id": "...", "record_kind": "insight", ...}
```

Even when the source ChangeGuard record has a `session_id`, it's dropped to `null` because of the T112 change. This loses session context for downstream consumers.

## Acceptance Criteria

**AC1:** NDJSON records for ChangeGuard hits include the `session_id` from the source ChangeGuard record, not hardcoded `None`.

**AC2:** If the source ChangeGuard record has no `session_id` (it's optional in the bridge contract), the NDJSON record has `session_id: null` — no regression.

**AC3:** The local recall NDJSON records continue to include `session_id` from the vault hit (already works).

**AC4:** The search-scope fix from T112 is NOT regressed — `sync query` still searches across all sessions by default. The `session_id` in the OUTPUT is metadata, not a search filter.

## Design Notes

- **File:** `crates/ai-brains-cli/src/commands/sync.rs` — NDJSON path (around line 460).
- The ChangeGuard record already has `record.session_id: Option<String>` from the bridge contract. Use that value in the NDJSON output instead of `None`.
- The T112 change set `session_id: None` in the output record. Fix: use `record.session_id.clone()` instead.
- This is an output-only change; the search scope (which uses the `session_id` parameter to `recall()`) is separate and remains `None`.

## Files

- `crates/ai-brains-cli/src/commands/sync.rs` — NDJSON record: `session_id: record.session_id.clone()` instead of `None`.

## Tests (TDD)

**Red:** `sync_query_ndjson__changeguard_record_has_session_id` — run `sync query "test" --format ndjson`, assert ChangeGuard-sourced records have `session_id` matching the source record (not null, when the source has one).

**Green:** Use `record.session_id.clone()` in the NDJSON output. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains sync query "backup" --format ndjson` → ChangeGuard records include session_id.

## Out of Scope

- Adding session_id to the pretty-format output (pretty format already shows session context in the recall section).
- Changing the bridge contract schema.
- Filtering NDJSON output by session_id (that's a search-scope concern, handled by T112).