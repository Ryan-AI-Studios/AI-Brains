# Track T114: Ingest --dry-run Skip UUID Validation

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — UX friction; dry-run requires valid UUIDs just to preview.
**Source:** Non-destructive command audit 2026-06-23.

---

## Problem Statement

`ai-brains ingest --dry-run` reads stdin JSON and calls `parse_ingest_request()` BEFORE checking `dry_run`. The parser validates all fields including UUID format for `turn_id`, `session_id`, `project_id`, `harness_id`. Users testing the dry-run must supply valid UUIDs:

```
$ echo '{"turn_id":"test","session_id":"test",...}' | ai-brains ingest --dry-run
Error: UUID parsing failed: invalid character: found `t` at 0
```

The purpose of `--dry-run` is to preview what would happen. Requiring valid UUIDs for a preview is unnecessary friction. The user just wants to see "you would ingest this content for this role".

## Acceptance Criteria

**AC1:** `ingest --dry-run` accepts JSON with placeholder string values for UUID fields (e.g. `"turn_id": "test"`) and prints a preview without error.

**AC2:** The preview output shows the raw string values as-is (not parsed UUIDs): `[dry-run] Would ingest turn test for project test / session test (role=user): <content preview>`.

**AC3:** The non-dry-run path still validates UUIDs strictly — invalid UUIDs cause an error as before.

**AC4:** The dry-run path still validates that required fields are PRESENT (non-empty strings for `content`, `role`; objects for `privacy` if required). Missing fields still error. Only UUID format validation is skipped.

## Design Notes

- **Approach:** In `ingest.rs`, parse the stdin as a `serde_json::Value` first. If `dry_run`, extract the fields as strings for preview and return early. If not `dry_run`, call `parse_ingest_request()` for strict validation.

- **Code change in `crates/ai-brains-cli/src/commands/ingest.rs`:**
  ```rust
  pub fn run(ctx: &AppContext, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
      let mut input = String::new();
      io::stdin().read_to_string(&mut input)?;

      if dry_run {
          let v: serde_json::Value = serde_json::from_str(&input)
              .map_err(|e| format!("Invalid JSON: {}", e))?;
          let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("(missing)");
          if content.is_empty() {
              return Err("content field is missing or empty".into());
          }
          let role = v.get("role").and_then(|r| r.as_str()).unwrap_or("(missing)");
          let turn_id = v.get("turn_id").and_then(|t| t.as_str()).unwrap_or("(missing)");
          let session_id = v.get("session_id").and_then(|s| s.as_str()).unwrap_or("(missing)");
          let project_id = v.get("project_id").and_then(|p| p.as_str()).unwrap_or("(missing)");
          let preview = truncate_preview(content);
          println!(
              "[dry-run] Would ingest turn {} for project {} / session {} (role={}): {}",
              turn_id, project_id, session_id, role, preview
          );
          return Ok(());
      }

      let request = parse_ingest_request(&input)?;
      // ... existing non-dry-run logic
  }
  ```

- **Required fields for dry-run:** `content` (must be present and non-empty). All other fields are shown as-is or `(missing)`.

## Files

- `crates/ai-brains-cli/src/commands/ingest.rs` — Split dry-run and non-dry-run parsing paths.

## Tests (TDD)

**Red:** `ingest__dry_run__accepts_placeholder_uuids` — pipe JSON with string UUIDs like `"test"`, assert dry-run preview prints without error.

**Red:** `ingest__dry_run__errors_on_missing_content` — pipe JSON without `content` field, assert error.

**Red:** `ingest__non_dry_run__still_validates_uuids` — pipe JSON with invalid UUIDs (not dry-run), assert UUID parse error.

**Green:** Implement the split parsing path. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `'{"turn_id":"test","content":"hello","role":"user"}' | ai-brains ingest --dry-run` → prints preview.
- Manual: `'{"turn_id":"test","content":"hello","role":"user"}' | ai-brains ingest` → UUID parse error.

## Out of Scope

- Relaxing validation for `pin --dry-run` or `forget --dry-run` (those don't have UUID validation friction).
- Adding a `--validate-only` flag.
- Changing the IngestRequest schema.