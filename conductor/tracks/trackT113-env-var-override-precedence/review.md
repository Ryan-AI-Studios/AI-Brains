# Track T113: Env-Var Override Precedence — Self-Review

## Summary

Replaced `dotenvy::dotenv_override()` with `dotenvy::dotenv()` and `dotenvy::from_path_override(home)` with `dotenvy::from_path(home)` in `crates/ai-brains-cli/src/main.rs`. Added one TDD smoke test proving shell env vars now win over project `.env` files.

## Changes

```
crates/ai-brains-cli/src/main.rs
- dotenvy::dotenv_override().ok();
+ dotenvy::dotenv().ok();

- dotenvy::from_path_override(home).ok();
+ dotenvy::from_path(home).ok();

crates/ai-brains-cli/tests/smoke.rs
+ env_var_precedence__shell_overrides_env_file
```

## Findings

| id | severity | description | file | status |
|---|---|---|---|---|
| 1 | info | Skipped the project-vs-global `.env` test as instructed; redirecting `USERPROFILE` is risky and the non-override semantics already cover the same code path | — | out_of_scope |
| 2 | info | Existing `--no-project-context` behavior verified by `test_no_project_context_preserves_env_vars` (passes) | smoke.rs | verified_fixed |
| 3 | info | `AI_BRAINS_VAULT_PATH` / `AI_BRAINS_KEY` absolute precedence is clap's `env` attribute behavior; not changed by this track | main.rs | verified_fixed |

## Verification

- `cargo nextest run -p ai-brains-cli env_var_precedence__shell_overrides_env_file` — Red (failed before code change), then Green (passed after code change)
- `cargo nextest run -p ai-brains-cli` — targeted suite: 71 tests, 70 passed, 1 failed
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` — clean
- `ledgerful verify --scope fast` — cargo fmt/clippy clean; `cargo test --workspace` has an unrelated pre-existing failure in `sync_query__daemon_down__returns_local_results` (T115, not touched in this track)

## Unrelated Failure

- `sync_query__daemon_down__returns_local_results` fails because `sync query` local recall returns zero vault hits for the seeded content. The `sync.rs` code was modified by another pending track (T115) in the same working tree, but this failure is unrelated to env-var precedence and exists independent of T113 changes.
- Decision: leave T115 failure to T115 owner; T113 verification scoped to ai-brains-cli targeted suite and the new T113 test passes.
