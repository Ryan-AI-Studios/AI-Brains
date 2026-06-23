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

- **Approach:** Define a private `DryRunIngestRequest` struct in `ingest.rs` where all UUID-typed fields are `String` instead of `SessionId`/`ProjectId`/etc. Derive `Deserialize` on it. This avoids schema drift — the struct mirrors `IngestRequest` but with relaxed types. If `IngestRequest` adds a field, the dry-run struct will fail to compile if it's missing the field (with `#[serde(deny_unknown_fields)]` to catch new fields).

- **Code change in `crates/ai-brains-cli/src/commands/ingest.rs`:**
  ```rust
  #[derive(serde::Deserialize)]
  #[serde(deny_unknown_fields)]
  struct DryRunIngestRequest {
      turn_id: String,
      session_id: String,
      project_id: String,
      harness_id: String,
      role: String,
      content: String,
      privacy: String,
      #[serde(default)]
      thinking: Option<String>,
      #[serde(default)]
      tx_id: Option<String>,
  }

  pub fn run(ctx: &AppContext, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
      let mut input = String::new();
      io::stdin().read_to_string(&mut input)?;

      if dry_run {
          let req: DryRunIngestRequest = serde_json::from_str(&input)
              .map_err(|e| format!("Invalid JSON: {}", e))?;
          if req.content.is_empty() {
              return Err("content field is empty".into());
          }
          let preview = truncate_preview(&req.content);
          println!(
              "[dry-run] Would ingest turn {} for project {} / session {} (role={}): {}",
              req.turn_id, req.project_id, req.session_id, req.role, preview
          );
          return Ok(());
      }

      let request = parse_ingest_request(&input)?;
      // ... existing non-dry-run logic
  }
  ```

- **Why a struct, not `serde_json::Value`:** Manual `v.get("turn_id")` duplicates field names as string literals and risks drifting from the real `IngestRequest` struct. A dedicated `DryRunIngestRequest` struct keeps the extraction declarative, leverages Serde's error handling, and `#[serde(deny_unknown_fields)]` catches schema drift at the deserialization level.

- **Required fields for dry-run:** All fields that `IngestRequest` requires are still required by `DryRunIngestRequest` (they must be present as strings). Only the UUID format validation is skipped. `content` must be non-empty.

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