# Track T118: eprintln! to tracing! Migration for User-Facing Warnings

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — cosmetic friction; PowerShell renders `eprintln!` as red RemoteException.
**Source:** Non-destructive command audit 2026-06-23.

---

## Problem Statement

The CLI has 62 `eprintln!` calls across 11 files. On PowerShell, every `eprintln!` output is rendered as a `RemoteException` with red error formatting, making informational warnings look like fatal errors. This alarms users and makes it hard to distinguish real errors from warnings.

Examples:
- `backup create` prints "Creating vault backup..." via `eprintln!` → looks like an error in PowerShell
- `daemon stop` warnings show as red exceptions
- `project resolve` "Ambiguous alias" messages show as red exceptions
- `nightly` progress messages (18 `eprintln!` calls) show as red exceptions

T102 already migrated `recall` session noise from `eprintln!` to `tracing::debug!`. T118 extends this pattern across all CLI commands.

## Acceptance Criteria

**AC1:** All `eprintln!` calls that produce **informational/progress messages** (not errors) are migrated to `tracing::info!` or `tracing::debug!`. These include:
- Progress messages ("Creating vault backup...", "Starting nightly intelligence sweep...")
- Warning messages that are non-fatal ("WARNING: Daemon is running...", "Ambiguous alias...")
- Status messages ("Detected project from .env...")

**AC2:** `eprintln!` is retained ONLY for:
- Actual error messages that precede `std::process::exit(1)` or `return Err(...)`
- Interactive prompts (`eprint!("Type 'yes' to continue: ")`) — these MUST stay as `eprint!`/`eprintln!` for stdin reading
- The global error handler in `main.rs` that prints the JSON error envelope

**AC3:** `println!` is retained for all stdout output (JSON, table data, dry-run previews, etc.) — only `eprintln!` (stderr) is migrated.

**AC4:** No regression — all existing tests pass. Tests that assert on stderr content may need updating if they check for specific `eprintln!` strings.

**AC5:** The `--quiet` flag suppresses `tracing::info!`/`tracing::debug!` output where applicable (already handled by some commands, extend to migrated messages).

## Design Notes

- **Categorization of eprintln! calls:**

  | File | Count | Category | Action |
  |------|-------|----------|--------|
  | `nightly.rs` | 18 | Progress/status | → `tracing::info!` |
  | `daemon.rs` | 14 | Status/warnings | → `tracing::info!` or `tracing::warn!` |
  | `project.rs` | 9 | Errors + info | Errors keep `eprintln!`, info → `tracing::info!` |
  | `main.rs` | 4 | Errors + interrupt | Keep `eprintln!` (error handler, ctrl-c) |
  | `sync.rs` | 4 | Warnings | → `tracing::warn!` |
  | `backup.rs` | 3 | Progress + warning | Progress → `tracing::info!`, WARNING → `tracing::warn!` |
  | `forget.rs` | 3 | Prompts + info | Prompts keep `eprint!`, info → `tracing::info!` |
  | `symbol_bridge.rs` | 3 | Info/warnings | → `tracing::info!` or `tracing::warn!` |
  | `graph.rs` | 2 | Errors | Keep `eprintln!` |
  | `context.rs` | 1 | Info | → `tracing::info!` |
  | `recall.rs` | 1 | Error | Keep `eprintln!` (already migrated by T102) |

- **Decision per eprintln!:** For each call, determine:
  1. Is it followed by `exit(1)` or `return Err(...)`? → Keep `eprintln!`
  2. Is it a prompt for stdin input? → Keep `eprint!`/`eprintln!`
  3. Is it a WARNING about a non-fatal condition? → `tracing::warn!`
  4. Is it informational/progress? → `tracing::info!`
  5. Is it debug-level detail? → `tracing::debug!`

- **Note:** `tracing::info!` output goes to stderr by default with the `tracing_subscriber::fmt::init()` configuration. But it's formatted as structured log lines, not raw text, so PowerShell doesn't render it as RemoteException. The user can control verbosity with `RUST_LOG`.

- **Consideration:** Some users may prefer seeing progress messages by default. With `tracing::info!`, they only appear if `RUST_LOG=info` is set. This is a behavior change. To mitigate, set the default log level to `info` for AI-Brains crates ONLY, keeping external dependencies at `warn`:

  ```rust
  // Replace: tracing_subscriber::fmt::init();
  // With:
  tracing_subscriber::fmt()
      .with_env_filter(
          tracing_subscriber::EnvFilter::try_from_default_env()
              .unwrap_or_else(|_| {
                  tracing_subscriber::EnvFilter::new("warn,ai_brains=info,ai_brains_cli=info")
              })
      )
      .init();
  ```

- **CRITICAL — EnvFilter scoping:** The default filter MUST be scoped to application crates (`ai_brains=info,ai_brains_cli=info`), NOT a blanket `info` level. A blanket `EnvFilter::new("info")` would dump all `INFO`-level logs from every dependency (`reqwest`, `hyper`, `tokio`, `sqlcipher`, `rusqlite`, etc.) into the terminal, creating far more noise than the original `eprintln!` calls. The scoped filter ensures external crates stay quiet (`warn`) while AI-Brains emits its progress messages (`info`).

  The filter targets:
  - `warn` — default level for all crates (external deps, stdlib)
  - `ai_brains=info` — all `ai_brains_*` crates at info level (brain, store, retrieval, models, etc.)
  - `ai_brains_cli=info` — the CLI crate specifically

  Users can override with `RUST_LOG=debug,ai_brains=debug` for full debugging, or `RUST_LOG=warn` to suppress progress messages.

## Files

- `crates/ai-brains-cli/src/main.rs` — Update `tracing_subscriber::fmt::init()` to default to `info` level.
- `crates/ai-brains-cli/src/commands/nightly.rs` — 18 `eprintln!` → `tracing::info!`
- `crates/ai-brains-cli/src/commands/daemon.rs` — 14 `eprintln!` → `tracing::info!` or `tracing::warn!`
- `crates/ai-brains-cli/src/commands/project.rs` — 9 `eprintln!` → mixed
- `crates/ai-brains-cli/src/commands/sync.rs` — 4 `eprintln!` → `tracing::warn!`
- `crates/ai-brains-cli/src/commands/backup.rs` — 3 `eprintln!` → mixed
- `crates/ai-brains-cli/src/commands/forget.rs` — 3 `eprintln!` → mixed (keep prompts)
- `crates/ai-brains-cli/src/commands/symbol_bridge.rs` — 3 `eprintln!` → `tracing::info!`
- `crates/ai-brains-cli/src/commands/context.rs` — 1 `eprintln!` → `tracing::info!`
- `crates/ai-brains-cli/src/commands/graph.rs` — 2 `eprintln!` — evaluate, likely keep

## Tests (TDD)

**Red:** `nightly__progress_goes_to_tracing_not_stderr` — run nightly, capture stderr, assert no raw "Starting nightly intelligence sweep..." text (it's now a tracing log line, not raw eprintln).

**Red:** `backup_create__progress_goes_to_tracing_not_stderr` — run backup create, capture stderr, assert no raw "Creating vault backup..." text.

**Red:** `tracing_filter__external_deps_stay_quiet` — run any command that triggers reqwest/tokio/hyper activity, capture stderr, assert no `INFO` logs from external crates (only `ai_brains` and `ai_brains_cli` crate messages appear).

**Green:** Migrate eprintln! calls + set scoped EnvFilter. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Manual: `ai-brains backup create` in PowerShell → no red RemoteException for "Creating vault backup...".
- Manual: `ai-brains nightly --status` → clean output, no red text.
- Manual: `$env:RUST_LOG = "warn"; ai-brains backup create` → progress messages suppressed, only warnings/errors shown.

## Out of Scope

- Migrating `println!` (stdout) — that's user-facing output, not warnings.
- Changing the tracing subscriber format (JSON logs, etc.).
- Adding structured fields to all tracing calls (focus on migration first).
- Migrating `eprintln!` in non-CLI crates (brain, store, etc.).