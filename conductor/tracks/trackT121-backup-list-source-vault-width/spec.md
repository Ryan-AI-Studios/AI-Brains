# Track T121: `backup list` Source Vault Column Width

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — cosmetic; source vault path truncated for old backups.
**Source:** v0.1.1 verification friction #3.

---

## Problem Statement

`backup list` shows a "Source Vault" column truncated to 26 characters. For old backups with UNC paths, this truncates to `\\?\C:\dev\AI-Brains\va...` — useless. For new backups (T116), the clean path `C:\dev\AI-Brains\vault.db` fits but only barely.

The column width (26) is too narrow for real Windows paths, which can easily be 40+ characters.

## Acceptance Criteria

**AC1:** The "Source Vault" column is widened to 40 characters (matching the old "Path" column width before T116 changed it).

**AC2:** For paths longer than 40 characters, the truncation shows the END of the path (right-truncate) rather than the beginning, so the filename is visible: `...AI-Brains\vault.db` instead of `C:\dev\AI-Brains\va...`.

**AC3:** The column header remains "Source Vault".

**AC4:** No regression in other columns (Filename, Timestamp, Version, Size).

## Design Notes

- **File:** `crates/ai-brains-cli/src/commands/backup.rs` — `run_list` function.
- Change the `truncate(&source, 26)` call to `truncate_right(&source, 40)`.
- Add a `truncate_right(s: &str, max: usize)` helper that keeps the last `max` characters with a `...` prefix: `format!("...{}", &s[s.len()-max+3..])` (careful with char boundaries — use `chars().rev().take(max-3).collect::<Vec<_>>().iter().rev().collect()` or similar).
- Alternatively, since T116 already uses `dunce::canonicalize` for new backups, old backups will still have UNC paths. Widening to 40 handles most cases. Right-truncation handles the rest.

## Files

- `crates/ai-brains-cli/src/commands/backup.rs` — `run_list`: widen column, add right-truncate.

## Tests (TDD)

**Red:** `backup_list__source_vault_column_shows_path_end` — create a backup with a long source vault path (>40 chars), run `backup list`, assert the Source Vault column shows the end of the path (contains `vault.db`), not the beginning truncated.

**Green:** Implement right-truncation. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup list` → Source Vault column shows readable paths.

## Out of Scope

- Stripping UNC prefix from old backups (T116 already handles new backups; old backups can't be retroactively fixed without re-reading them).
- Changing the column to show full paths without truncation (table layout would break).