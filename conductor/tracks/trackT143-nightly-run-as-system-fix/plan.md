# T143 Plan

## Tasks

### Track 1 — Implementation (core-engineer)
1. **TDD red:** Test that `nightly --schedule --run-as-system` generates a task command containing `--no-project-context`, `--skip-import`, and the env var names. Test that without `--run-as-system` those flags are absent (no regression).
2. **nightly.rs:** When `--run-as-system` is passed:
   - Read `AI_BRAINS_VAULT_PATH`, `AI_BRAINS_MODEL_URL`, `AI_BRAINS_COMPLETION_MODEL`, `AI_BRAINS_EMBEDDING_URL`, `AI_BRAINS_EMBEDDING_MODEL` from env/`.env`.
   - Generate a wrapper `.bat` file at `scripts\nightly-task.bat` (or temp) with `set VAR=val` lines + the ai-brains invocation with `--no-project-context --skip-import --log-format json`.
   - Register the scheduled task to run the wrapper script as SYSTEM.
   - Set "Start In" to the vault's parent directory.
3. **daemon.rs:** Same env-var baking for `daemon --schedule --run-as-system`.
4. **main.rs:** Add `--dry-run` to nightly if not present. `--dry-run` prints the schtasks command + wrapper script content without registering.
5. Verify: `cargo nextest run -p ai-brains-cli`.

### Track 2 — Docs (general, parallel)
6. **Docs/OPERATIONS.md:** Update nightly scheduling section to document `--run-as-system` behavior (bakes env vars, adds `--no-project-context --skip-import`).
7. **conductor/deferred.md:** Mark nightly issue as "Fixed by T143".

### Final (manager)
8. CI gate: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit ; ledgerful verify --scope full`.
9. Re-schedule the live task: `ai-brains nightly --unschedule` (may need elevation) then `ai-brains nightly --schedule --run-as-system --start-time 01:00` (from elevated shell).
10. Verify next morning that the nightly ran successfully.