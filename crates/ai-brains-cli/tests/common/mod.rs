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
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
    // T279 F13: strip ambient skip so hermetic_bin can force-on without host leak.
    "AI_BRAINS_PREFLIGHT_SKIP_LIVE_HOTSPOTS",
    // Ledgerful TX id (preferred + deprecated alias) — strip ambient pollution
    "LEDGERFUL_TX_ID",
    "CHANGEGUARD_TX_ID",
    // T208 F29: ambient developer RUST_LOG=debug must not false-fail quiet-log ACs.
    // Tests that need a directive re-set via `.env("RUST_LOG", …)` after strip.
    "RUST_LOG",
    // T218 F38 / AC19: ambient score/RRF floors must not pollute hermetic dual-floor ACs.
    // Tests that need non-default floors set via `.env(...)` after strip.
    "AI_BRAINS_SEMANTIC_MIN_SCORE",
    "AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE",
    "AI_BRAINS_RRF_K",
    // T344 F5: ambient auto-bind disable must not leak into hermetic bind ACs.
    "AI_BRAINS_NO_AUTO_BIND",
    // T354 F8: ambient governed briefing must not skip the legacy Index helper.
    "AI_BRAINS_GOVERNED_BRIEFING",
];

/// `cargo_bin("ai-brains")` with ambient denylist stripped.
///
/// Callers set `--vault-path` / project / session as needed.
/// Prefer `hermetic_cmd` or `hermetic_vault` when a vault path is known.
///
/// Deterministic all-zero SQLCipher product key used by hermetic fixtures.
pub const ZERO_SQLCIPHER_KEY: &str =
    "x'0000000000000000000000000000000000000000000000000000000000000000'";

/// T187/T197: sets explicit zero key + `AI_BRAINS_ALLOW_ZERO_KEY=1` so hermetic
/// tests open under live SQLCipher without relying on silent zero defaults.
/// Production refuses zero keys and missing keys (T197 F2).
///
/// **T203 soft-resolve (AC4/AC5):** always **strips** ambient `AI_BRAINS_PROJECT_ID`
/// (denylist). AC4 tests must `.env("AI_BRAINS_PROJECT_ID", …)` after this returns
/// so soft-resolve sees High/authoritative. AC5 must leave it unset and prefer
/// `--no-project-context` so workspace `.env` cannot re-inject a project id.
/// Note: `--no-project-context` alone does **not** clear a shell-exported
/// `AI_BRAINS_PROJECT_ID` — only this strip (or an explicit `env_remove`) does.
pub fn hermetic_bin() -> Command {
    let mut cmd = Command::cargo_bin("ai-brains").expect("ai-brains bin must be built for tests");
    strip_ambient(&mut cmd);
    cmd.env("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    cmd.env("AI_BRAINS_KEY", ZERO_SQLCIPHER_KEY);
    // T279 F13: repo-cwd `ledgerful hotspots` must not leak into hermetic Safety.
    cmd.env("AI_BRAINS_PREFLIGHT_SKIP_LIVE_HOTSPOTS", "1");
    cmd
}

/// Process-lifetime empty home used to isolate missing-key / no-key tests.
///
/// **T205 F11:** CLI always merges `~/.ai-brains/.env` for gaps (including under
/// `--no-project-context`). Developers with a real global KEY would re-inject it
/// after `env_remove("AI_BRAINS_KEY")` unless home is redirected. This path has
/// **no** `.ai-brains/.env` and lives for the whole test process.
fn empty_test_home() -> &'static Path {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let dir = tempfile::tempdir().expect("empty test home tempdir");
        // Keep the directory for process lifetime (do not delete while tests run).
        dir.keep()
    })
    .as_path()
}

/// Redirect `USERPROFILE` and `HOME` to an empty home with no global dotenv KEY.
///
/// Use after stripping `AI_BRAINS_KEY` when the test must observe a true missing-key
/// path on machines that have a real `~/.ai-brains/.env`. Sets both env vars (F22).
pub fn isolate_empty_home(cmd: &mut Command) {
    let home = empty_test_home();
    cmd.env("USERPROFILE", home);
    cmd.env("HOME", home);
}

/// Ambient-stripped binary that **never** sets `AI_BRAINS_KEY` or
/// `AI_BRAINS_ALLOW_ZERO_KEY` — for proving vault-independent commands (e.g.
/// `daemon status`) and true missing-key paths.
///
/// Isolation strategy (T205 F11 / AC12):
/// - Strips KEY / ALLOW_ZERO_KEY from the child env.
/// - Passes `--no-project-context` so **project** `.env` is not loaded.
/// - Sets `USERPROFILE` + `HOME` to a process-lifetime **empty home** so the
///   always-on global `~/.ai-brains/.env` merge cannot re-inject a developer KEY.
///
/// Global dotenv **still merges** under `--no-project-context`; isolation is the
/// empty home, not skipping the global loader.
pub fn hermetic_bin_no_key() -> Command {
    let mut cmd = Command::cargo_bin("ai-brains").expect("ai-brains bin must be built for tests");
    strip_ambient(&mut cmd);
    isolate_empty_home(&mut cmd);
    // Explicit remove in case cargo/runner injects keys outside the denylist path.
    cmd.env_remove("AI_BRAINS_KEY");
    cmd.env_remove("AI_BRAINS_ALLOW_ZERO_KEY");
    cmd.arg("--no-project-context");
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
