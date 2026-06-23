# T116 Review: Backup List Full Paths + Schema Version Fix

## Findings

| id | severity | description | status |
|---|---|---|---|
| 1 | info | `conductor/conductor.md` still lists T116 as **Pending**; update to **Complete** when track is finalized. | open |

## Self-Review Notes

- **Scope discipline maintained**: the `schema_version` query (lines 121-129) was already fixed by T117 and was not touched.
- **AC1/AC2**: `backup list` now shows the backup filename in the first column with header `Filename` and 35-char width, fully visible and copy-pasteable.
- **AC3/AC4**: owned by T117, already passing via `backup__metadata_has_correct_schema_version`.
- **AC5**: `source_vault_path` is now computed with `dunce::canonicalize`, removing the `\\?\` UNC prefix on Windows.
- **AC6**: no regression for pre-T109/pre-T116 backups; metadata still falls back to `(no metadata)` when keys are absent.
- Tests follow project naming `feature__condition__expected_result` with `#[allow(non_snake_case)]` and use `tempfile::tempdir()` per test.
- No `unwrap()`/`expect()`/`panic!()` introduced in production code.

## Verification Results

- `cargo nextest run -p ai-brains-brain --lib`: 12 passed, 0 skipped.
- `cargo nextest run -p ai-brains-cli`: 64 passed, 0 skipped.
- `cargo clippy -p ai-brains-brain -p ai-brains-cli --all-targets -- -D warnings`: clean.
- `cargo fmt --check -p ai-brains-brain -p ai-brains-cli`: clean.

## Severity Rationale

No critical/high/medium findings. The only open item is a conductor registry status update (info). Since the user requested "do NOT commit," the status update is recorded here and can be applied when the track is finalized.
