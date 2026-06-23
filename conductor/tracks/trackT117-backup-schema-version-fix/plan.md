# T117 Plan: Backup Schema Version Fix

- [x] Read `conductor/tracks/trackT117-backup-schema-version-fix/spec.md`.
- [x] Inspect `crates/ai-brains-brain/src/backup.rs` lines 121-138 and test module around line 357+.
- [x] Confirm `schema_migrations` table schema in `crates/ai-brains-store/src/migrations.rs:86-91`.
- [x] Start ChangeGuard transaction for T117.
- [x] Implement Red test: `backup__metadata_has_correct_schema_version`.
- [x] Run new test and confirm it fails before production change (TDD Red).
- [x] Implement Green production fix: single `SELECT MAX(name) FROM schema_migrations` query.
- [x] Run new test and confirm it passes (TDD Green).
- [x] Run full `cargo nextest run -p ai-brains-brain --lib`.
- [x] Run `cargo clippy -p ai-brains-brain --all-targets -- -D warnings`.
- [x] Run `cargo fmt --check -p ai-brains-brain`.
- [x] Write `review.md` self-review.
