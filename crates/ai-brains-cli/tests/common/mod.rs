//! Shared hermetic CLI helpers for integration tests (T186).
//!
//! Each `tests/*.rs` binary is a separate crate: add `mod common;` (or
//! `#[path = "common/mod.rs"] mod common;`) to consume these helpers.
//!
//! Design:
//! - Prefer `env_remove` over `env_clear` (L3/L4).
//! - Denylist covers product elevation keys + CLI env args (SCOPE, PREFLIGHT).
//! - `.expect` is acceptable in **test** helpers only (not production).
//!
//! Usage:
//! ```ignore
//! let mut cmd = common::hermetic_cmd(&vault);
//! cmd.arg("init").assert().success();
//!
//! // Or vault-only (no default project/session env):
//! let mut cmd = common::hermetic_vault(&vault);
//!
//! // Or bare binary after ambient strip (caller sets vault/env):
//! let mut cmd = common::hermetic_bin();
//! ```

// not every helper is used by every integration binary
#![allow(dead_code)]
// Test-only helper: expect on cargo_bin is intentional (A9 / L2); not production.
#![allow(clippy::disallowed_methods)]

use assert_cmd::Command;
use std::path::Path;

/// Deterministic default project UUID (smoke-style).
pub const DEFAULT_PROJECT: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

/// Deterministic default session UUID (smoke-style).
pub const DEFAULT_SESSION: &str = "11111111-1111-1111-1111-111111111111";

/// Ambient keys stripped from the child process before spawn.
///
/// Seeded from `elevation.rs` `ELEVATE_ENV_KEYS` plus CLI `#[arg(env=…)]`
/// keys that are not in the elevation list (`SCOPE`, `PREFLIGHT_PRINCIPAL_ID`).
pub const AMBIENT_DENYLIST: &[&str] = &[
    // elevation.rs ELEVATE_ENV_KEYS
    "AI_BRAINS_VAULT_PATH",
    "AI_BRAINS_KEY",
    "AI_BRAINS_VAULT_KEY",
    "AI_BRAINS_MODEL_URL",
    "AI_BRAINS_COMPLETION_MODEL",
    "AI_BRAINS_EMBEDDING_URL",
    "AI_BRAINS_EMBEDDING_MODEL",
    "AI_BRAINS_PROJECT_ID",
    "AI_BRAINS_SESSION_ID",
    // CLI env args not in elevation list
    "AI_BRAINS_SCOPE",
    "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID",
];

/// `cargo_bin("ai-brains")` with ambient denylist stripped.
///
/// Callers set `--vault-path` / project / session as needed.
/// Prefer `hermetic_cmd` or `hermetic_vault` when a vault path is known.
pub fn hermetic_bin() -> Command {
    let mut cmd = Command::cargo_bin("ai-brains").expect("ai-brains bin must be built for tests");
    strip_ambient(&mut cmd);
    cmd
}

/// Hermetic binary with `--vault-path` only (no project/session env).
pub fn hermetic_vault(vault: &Path) -> Command {
    let mut cmd = hermetic_bin();
    cmd.arg("--vault-path").arg(vault);
    cmd
}

/// Hermetic binary with vault path + default project/session env (L1).
///
/// Uses `--no-project-context` so the CLI does **not** clear `AI_BRAINS_PROJECT_ID`
/// / `SESSION_ID` when no cwd `.env` exists (CI runners). Local developer
/// workspaces often have a repo `.env`, which masks the bug without this flag.
///
/// Override project/session with further `.env(...)` calls after this returns
/// (later env wins in assert_cmd).
pub fn hermetic_cmd(vault: &Path) -> Command {
    let mut cmd = hermetic_vault(vault);
    // T80: preserve explicit env on clean runners (no project-local .env).
    cmd.arg("--no-project-context");
    cmd.env("AI_BRAINS_PROJECT_ID", DEFAULT_PROJECT);
    cmd.env("AI_BRAINS_SESSION_ID", DEFAULT_SESSION);
    cmd
}

/// Hermetic binary with vault + explicit project/session.
pub fn hermetic_cmd_with_ids(vault: &Path, project_id: &str, session_id: &str) -> Command {
    let mut cmd = hermetic_vault(vault);
    cmd.arg("--no-project-context");
    cmd.env("AI_BRAINS_PROJECT_ID", project_id);
    cmd.env("AI_BRAINS_SESSION_ID", session_id);
    cmd
}

fn strip_ambient(cmd: &mut Command) {
    for key in AMBIENT_DENYLIST {
        cmd.env_remove(key);
    }
}
