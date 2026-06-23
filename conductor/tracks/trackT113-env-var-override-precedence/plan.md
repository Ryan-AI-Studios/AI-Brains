# Track T113: Env-Var Override Precedence — Plan

- [x] Read spec and existing code (`crates/ai-brains-cli/src/main.rs:478-499`, `tests/smoke.rs`)
- [x] Start ChangeGuard ledger transaction (`ledgerful ledger start T113-env-var-override-precedence`)
- [x] Run `ledgerful scan --impact` to assess blast radius
- [ ] Add TDD Red test to `crates/ai-brains-cli/tests/smoke.rs`
  - [ ] `env_var_precedence__shell_overrides_env_file`: project `.env` sets `AI_BRAINS_MODEL_URL=http://127.0.0.1:9999`, shell env var sets `http://127.0.0.1:1`; `daemon status` output proves shell value wins (port :1 → Closed)
- [ ] Change `dotenvy::dotenv_override()` to `dotenvy::dotenv()` in `main.rs`
- [ ] Change `dotenvy::from_path_override(home)` to `dotenvy::from_path(home)` in `main.rs`
- [ ] Run targeted verification
  - [ ] `cargo nextest run -p ai-brains-cli env_var_precedence__shell_overrides_env_file` (Red → Green)
  - [ ] `cargo nextest run -p ai-brains-cli`
  - [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [x] Verify `--no-project-context` still skips `.env` loading (existing `test_no_project_context_preserves_env_vars`)
- [x] Write `review.md` self-review
- [x] Update `conductor/conductor.md` status to Completed
- [x] Final `ledgerful verify --scope fast`
