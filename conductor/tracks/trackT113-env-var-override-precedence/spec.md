# Track T113: Env-Var Override Precedence — Shell > Project .env > Global .env

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P1 — blocks testing, operations, and runtime configuration flexibility.
**Source:** T106 validation finding + non-destructive command audit 2026-06-23.

---

## Problem Statement

The CLI's env-var loading order in `main.rs:478-499` is:

1. If project `.env` exists: `dotenvy::dotenv_override()` — project .env overrides shell env vars
2. If `AI_BRAINS_VAULT_PATH` not set: `dotenvy::from_path_override(~/.ai-brains/.env)` — global .env overrides shell env vars

The `from_path_override` / `dotenv_override` functions make `.env` files WIN over shell env vars. This is backwards from standard Unix/Windows convention where explicit shell env vars take precedence over config files.

**Impact:**
- T106 nightly timeout validation blocked: setting `$env:AI_BRAINS_MODEL_URL = "http://127.0.0.1:1"` in PowerShell has no effect because `~/.ai-brains/.env` contains `AI_BRAINS_MODEL_URL=http://127.0.0.1:8081` and overrides it.
- Users cannot override config at runtime via shell env vars for testing, debugging, or temporary reconfiguration.
- CI/automation cannot override settings without editing .env files.

## Acceptance Criteria

**AC1:** Shell env vars take precedence over both project `.env` and global `~/.ai-brains/.env`. If `AI_BRAINS_MODEL_URL` is set in the shell, `.env` files do not override it.

**AC2:** Project `.env` takes precedence over global `~/.ai-brains/.env`. If a key is in both, the project `.env` value wins.

**AC3:** `--no-project-context` skips ALL `.env` loading (both project and global). Already implemented but verify it still works.

**AC4:** `AI_BRAINS_VAULT_PATH` and `AI_BRAINS_KEY` set via shell env vars or `--vault-path`/`--key` CLI flags take absolute precedence (already handled by clap's `env` attribute, but verify the .env override doesn't clobber them).

**AC5:** The loading order becomes: CLI flags > shell env vars > project `.env` > global `~/.ai-brains/.env`.

## Design Notes

- **Fix:** Replace `dotenvy::dotenv_override()` with `dotenvy::dotenv()` (non-override — only sets env vars that are NOT already set). Replace `dotenvy::from_path_override(home)` with `dotenvy::from_path(home)`.

- **File:** `crates/ai-brains-cli/src/main.rs:478-499`.

- **Current code:**
  ```rust
  if !std::path::Path::new(".env").exists() {
      std::env::remove_var("AI_BRAINS_PROJECT_ID");
      std::env::remove_var("AI_BRAINS_SESSION_ID");
  } else {
      dotenvy::dotenv_override().ok();  // .env wins over shell
  }
  if std::env::var("AI_BRAINS_VAULT_PATH").is_err() {
      if let Some(mut home) = dirs::home_dir() {
          home.push(".ai-brains");
          home.push(".env");
          if home.exists() {
              dotenvy::from_path_override(home).ok();  // global .env wins over shell
          }
      }
  }
  ```

- **New code:**
  ```rust
  if !std::path::Path::new(".env").exists() {
      std::env::remove_var("AI_BRAINS_PROJECT_ID");
      std::env::remove_var("AI_BRAINS_SESSION_ID");
  } else {
      dotenvy::dotenv().ok();  // shell wins, .env fills gaps
  }
  // Global .env fills remaining gaps (only if vault path still not set)
  if std::env::var("AI_BRAINS_VAULT_PATH").is_err() {
      if let Some(mut home) = dirs::home_dir() {
          home.push(".ai-brains");
          home.push(".env");
          if home.exists() {
              dotenvy::from_path(home).ok();  // shell wins, global .env fills gaps
          }
      }
  }
  ```

- **Risk:** Some users may rely on `.env` overriding their shell env vars. This is unlikely — the standard expectation is that shell env vars are explicit and take precedence. Document this as a breaking change in the changelog.

- **Side effect:** T106 AC4/AC5 become testable — setting `$env:AI_BRAINS_MODEL_URL` in PowerShell will now override the `.env` value.

## Files

- `crates/ai-brains-cli/src/main.rs` — Change `dotenv_override` to `dotenv` and `from_path_override` to `from_path`.

## Tests (TDD)

**Red:** `env_var_precedence__shell_overrides_env_file` — set `AI_BRAINS_MODEL_URL` in shell, create a project `.env` with a different value, run a CLI command that reads the env var, assert the shell value is used.

**Red:** `env_var_precedence__project_env_overrides_global_env` — create a project `.env` with `AI_BRAINS_VAULT_PATH=X` and a global `.env` with `AI_BRAINS_VAULT_PATH=Y`, assert the project value wins.

**Green:** Change to non-override dotenvy calls. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `$env:AI_BRAINS_MODEL_URL = "http://127.0.0.1:1"; ai-brains daemon status` → shows "Closed" for LLM backend (proves shell env var took precedence over .env).
- Manual: Without shell override, `ai-brains daemon status` → shows "Open" (proves .env still loads when shell var is absent).

## Out of Scope

- Adding `--model-url` / `--embedding-url` CLI flags (separate track if needed).
- Changing how clap's `env` attribute works (clap already respects shell env vars over its own defaults).
- Migrating to a TOML/YAML config file.