//! T186 AC2 — ambient pollution isolation proof for hermetic CLI helpers.
//!
//! Pollutes the **parent** process with an invalid `AI_BRAINS_PROJECT_ID`, then
//! runs `common::hermetic_cmd` which must strip ambient keys and succeed with
//! the fixture project/session.

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_core::temp_env::TempEnv;
use predicates::prelude::*;
use tempfile::tempdir;

/// Invalid / distinctive ambient project id that must NOT leak into the child.
const POLLUTED_PROJECT: &str = "ffffffff-ffff-ffff-ffff-ffffffffffff";
const POLLUTED_SESSION: &str = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";
const POLLUTED_KEY: &str = "x'deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef'";

#[test]
fn hermetic_cmd__polluted_parent_project_id__child_uses_fixture_ids() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");

    // Pollute parent env (restored on drop via TempEnv).
    let _p = TempEnv::set("AI_BRAINS_PROJECT_ID", POLLUTED_PROJECT);
    let _s = TempEnv::set("AI_BRAINS_SESSION_ID", POLLUTED_SESSION);
    let _k = TempEnv::set("AI_BRAINS_KEY", POLLUTED_KEY);
    let _v = TempEnv::set("AI_BRAINS_VAULT_PATH", r"C:\nonexistent\ambient-vault.db");

    // Init under hermetic helper — must not pick ambient vault path / key.
    common::hermetic_cmd(&vault)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vault initialized successfully"));

    assert!(vault.exists(), "vault must be created at fixture path");

    // Pin with hermetic defaults; ambient POLLUTED_PROJECT / KEY must not win.
    // Init already proved KEY/VAULT_PATH strip (wrong key/path would fail open).
    // Pin requires project/session: hermetic_cmd sets DEFAULT_*; ambient polluted
    // ids are env_remove'd first so they cannot be the child's effective context.
    let pin_out = common::hermetic_cmd(&vault)
        .arg("pin")
        .arg("T186 hermetic pollution isolation seed")
        .output()
        .expect("pin under hermetic defaults");
    let pin_combined = format!(
        "{}{}",
        String::from_utf8_lossy(&pin_out.stdout),
        String::from_utf8_lossy(&pin_out.stderr)
    );
    assert!(
        pin_out.status.success(),
        "pin must succeed with fixture project/session; combined={pin_combined}"
    );
    assert!(
        !pin_combined.contains(POLLUTED_PROJECT),
        "polluted project id must not appear in child output; got: {pin_combined}"
    );
    assert!(
        !pin_combined.contains(POLLUTED_SESSION),
        "polluted session id must not appear in child output; got: {pin_combined}"
    );
    assert!(
        !pin_combined.contains(POLLUTED_KEY),
        "polluted key material must not appear in child output; got: {pin_combined}"
    );
}

#[test]
fn hermetic_bin__strips_ambient_vault_path__explicit_arg_wins() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("explicit.db");

    let _v = TempEnv::set("AI_BRAINS_VAULT_PATH", r"C:\nonexistent\should-not-use.db");

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("init")
        .assert()
        .success();

    assert!(vault.exists());
}
