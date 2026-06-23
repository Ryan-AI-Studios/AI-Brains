# T116 Plan: Backup List Full Paths + Schema Version Fix

Scope limited to filename display and clean source path. The `schema_version` query fix is owned by T117 and must not be modified.

- [x] Read `conductor/tracks/trackT116-backup-list-full-paths/spec.md`.
- [x] Inspect `crates/ai-brains-cli/src/commands/backup.rs` `run_list`.
- [x] Inspect `crates/ai-brains-brain/src/backup.rs` `source_vault_path` canonicalize and test module.
- [x] Confirm T117 already fixed the `schema_version` query; do not touch lines 121-129.
- [x] Start ChangeGuard transaction for T116.
- [x] Implement Red test: `backup__metadata_source_path_no_unc_prefix` in brain crate.
- [x] Implement Red test: `backup_list__shows_filename_not_full_path` in CLI smoke tests.
- [x] Run new tests and confirm they fail before production changes (TDD Red).
- [x] Implement Green production fix: `run_list` shows `file_name()` with "Filename" header and 35-char column.
- [x] Implement Green production fix: use `dunce::canonicalize` for `source_vault_path` and add `dunce` dependency.
- [x] Run new tests and confirm they pass (TDD Green).
- [x] Run `cargo nextest run -p ai-brains-brain --lib`.
- [x] Run `cargo nextest run -p ai-brains-cli`.
- [x] Run `cargo clippy -p ai-brains-brain -p ai-brains-cli --all-targets -- -D warnings`.
- [x] Run `cargo fmt --check -p ai-brains-brain -p ai-brains-cli`.
- [x] Write `review.md` self-review.
- [x] Manual smoke: `ai-brains backup create` then `ai-brains backup list` shows filename and clean source path.
- [ ] Update `conductor/conductor.md` T116 status on finalization (deferred to commit stage).
