# Track T137: Bridge Hits Populate Session-ID

**Status:** Complete
**Started:** —
**Owner:** —
**Priority:** P3 — data completeness; bridge hits have empty session_id.
**Source:** T119-T132 non-destructive command audit.

---

## Problem Statement

The `BridgeRecord` contract (`ai-brains-contracts/src/bridge.rs`) already has `session_id: Option<String>`, and outbound push records populate it from the vault. However, the inbound bridge query path (`query_changeguard_bridge` in recall.rs) does not read `session_id` from the `BridgeRecord` when building `RecallHit`s — it always sets `session_id: None` for bridge hits.

This means:
- JSON recall results from the bridge show `"session_id": ""` (empty).
- Pretty format shows `session=` (empty) for bridge hits.
- Downstream consumers (sync query NDJSON) lose session context for bridge-sourced records.

The fix is straightforward: the `BridgeRecord` response from `ledgerful search --json` may include `session_id`, and `query_changeguard_bridge` should populate `RecallHit.session_id` from it when present.

## Acceptance Criteria

**AC1:** When a bridge `BridgeRecord` has `session_id: Some(non_empty)`, the resulting `RecallHit` has `session_id` populated with that value.

**AC2:** When a bridge `BridgeRecord` has `session_id: None` OR `Some("")` (empty string), the `RecallHit` has `session_id: None` — empty strings are normalized to `None` to prevent printing `session=` with no value. No regression.

**AC3:** The JSON recall output shows `session_id` for bridge hits when the source record has a non-empty value (not empty string).

**AC4:** The pretty format shows `session=xxxxxxxx` for bridge hits when available, and omits the `session=` prefix entirely when `session_id` is `None` (instead of showing `session=` with empty value).

## Design Notes

- **File:** `crates/ai-brains-retrieval/src/recall.rs` — in `query_changeguard_bridge`, when building `RecallHit::bridge(...)`, pass `record.session_id.clone()` as the `session_id` field. **Important:** normalize `Some("")` to `None` — use `.filter(|s| !s.is_empty())` to prevent empty strings from being treated as a valid session ID. This handles downstream systems that return `""` instead of `null`.
- The `RecallHit::bridge` constructor currently sets `session_id: None`. Either add a parameter or set the field after construction.
- Check the `BridgeRecord` struct in `ai-brains-contracts/src/bridge.rs` to confirm `session_id` is available on the deserialized record.
- For the pretty format: when `session_id` is `None`, omit the `session=` part entirely instead of showing `session=`. This applies to both bridge and local hits — update the pretty format logic in `recall.rs` (CLI).

## Files

- `crates/ai-brains-retrieval/src/recall.rs` — populate `session_id` from `BridgeRecord` in `query_changeguard_bridge`.
- `crates/ai-brains-cli/src/commands/recall.rs` — pretty format: omit `session=` when None.

## Tests (TDD)

**Red:** `recall_json__bridge_hit_has_session_id` — mock or simulate a bridge response with `session_id`, run recall, assert the bridge hit's `session_id` is populated. This may require a test fixture or mock since it depends on `ledgerful` being available.

Note: If integration testing is impractical (requires ledgerful running), add a unit test for the mapping logic in recall.rs that verifies `RecallHit::bridge` is constructed with the `session_id` from the `BridgeRecord`.

**Red:** `recall_pretty__bridge_hit_no_session_omits_prefix` — verify pretty format omits `session=` when session_id is None.

**Green:** Populate session_id from BridgeRecord. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-retrieval`
- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains recall "backup" --format json` → bridge hits have session_id when available.

## Out of Scope

- Changing the BridgeRecord contract schema (already supports session_id).
- Adding session_id to bridge payloads that don't have it (that's ChangeGuard's concern).
- Filtering by session_id in bridge queries (that's a search-scope concern).