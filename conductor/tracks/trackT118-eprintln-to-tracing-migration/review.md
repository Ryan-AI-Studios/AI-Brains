# Track T118 Review Log

## Self-Review

### Summary
Migrated informational/progress/warning `eprintln!` calls in `ai-brains-cli` to scoped `tracing::info!`/`tracing::warn!` events. Retained `eprintln!` only for actual errors, interactive prompts, and the global error handler. Replaced the default `tracing_subscriber::fmt::init()` with a scoped `EnvFilter` (`warn,ai_brains=info,ai_brains_cli=info`) so external crates stay quiet.

### Findings

| id | severity | description | file | status |
|--|--|--|--|--|
| 1 | — | Verified no `unwrap()`/`expect()` added in production code. | — | pass |
| 2 | — | Test naming follows `feature__condition__result`; no `test_` prefix; `#[allow(non_snake_case)]` applied. | `tests/smoke.rs` | pass |
| 3 | — | `println!` stdout output untouched (JSON, tables, dry-run previews). | — | pass |
| 4 | — | Prompts retained as `eprint!` in `backup.rs` and `forget.rs`. | — | pass |
| 5 | — | Global error handler / ctrl-c / runtime init failures in `main.rs` retain `eprintln!`. | — | pass |
| 6 | low | `commands/recall.rs` no-results hint still uses `eprintln!`; this is intentional per T102/T118 AC2 (error-adjacent user-facing hint on empty results) but is technically informational. Left unchanged to avoid changing T102 behavior. | `commands/recall.rs:152` | out_of_scope |

### Verification
- `cargo nextest run -p ai-brains-cli` → 74 passed / 0 failed
- `cargo clippy --workspace --all-targets -- -D warnings` → clean
- `cargo fmt --check` → clean
- Manual: `ai-brains backup create` in PowerShell shows structured INFO line, not raw red RemoteException.

### Migration Counts
- Migrated to `tracing::info!` / `tracing::warn!`: **50** `eprintln!` calls
- Retained `eprintln!` / `eprint!`: **14** calls (errors, prompts, global handler, recall hint)

### Deferred Items
None. The one low-info finding is out-of-scope and tracked as intentional T102 behavior.
