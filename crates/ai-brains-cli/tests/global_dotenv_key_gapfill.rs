#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T205 — Global dotenv KEY gap-fill hermetic suite (AC1–AC4).
//!
//! Pattern: dual USERPROFILE+HOME tempdir, write `.ai-brains/.env`, separate
//! vault tempdir (see smoke `env_var_precedence__project_env_overrides_global_env`).

mod common;

use ai_brains_contracts::doctor::{CheckSeverity, DoctorReport};
use std::fs;
use tempfile::tempdir;

/// Non-zero hermetic product key (not a real user secret).
const KEY_A: &str = "x'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'";
/// Alternate non-zero key for precedence conflicts.
const KEY_B: &str = "x'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'";

fn write_global_env(home: &std::path::Path, body: &str) {
    let ai = home.join(".ai-brains");
    fs::create_dir_all(&ai).expect("create .ai-brains");
    fs::write(ai.join(".env"), body).expect("write global .env");
}

fn vault_open_ok(report: &DoctorReport) -> bool {
    report
        .checks
        .iter()
        .any(|c| c.name == "vault_open" && c.severity == CheckSeverity::Ok)
}

fn vault_open_skip_missing(report: &DoctorReport) -> bool {
    report.checks.iter().any(|c| {
        c.name == "vault_open"
            && c.severity == CheckSeverity::Skip
            && c.message
                .as_deref()
                .is_some_and(|m| m.contains("key missing") || m.contains("AI_BRAINS_KEY"))
    })
}

/// AC1: process KEY unset; KEY only in isolated global `.env`; `--vault-path` set
/// → key resolves (init + doctor vault_open not skip-missing).
#[test]
fn ac1__key_only_in_global_env__vault_path_set__resolves() {
    let home_dir = tempdir().unwrap();
    let vault_dir = tempdir().unwrap();
    let vault = vault_dir.path().join("vault.db");

    write_global_env(home_dir.path(), &format!("AI_BRAINS_KEY=\"{KEY_A}\"\n"));

    // Init using global KEY only (shell KEY stripped).
    let init = common::hermetic_bin()
        .current_dir(vault_dir.path())
        .env("USERPROFILE", home_dir.path())
        .env("HOME", home_dir.path())
        .env_remove("AI_BRAINS_KEY")
        .env_remove("AI_BRAINS_ALLOW_ZERO_KEY")
        .env_remove("AI_BRAINS_VAULT_PATH")
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("init")
        .output()
        .expect("init with global KEY only");

    assert!(
        init.status.success(),
        "init must succeed with KEY only in global .env; stderr={}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(vault.exists(), "vault must exist after init");

    let doctor = common::hermetic_bin()
        .current_dir(vault_dir.path())
        .env("USERPROFILE", home_dir.path())
        .env("HOME", home_dir.path())
        .env_remove("AI_BRAINS_KEY")
        .env_remove("AI_BRAINS_ALLOW_ZERO_KEY")
        .env_remove("AI_BRAINS_VAULT_PATH")
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor with global KEY only");

    assert!(
        doctor.status.success(),
        "doctor must exit 0 when global KEY opens vault; stderr={}",
        String::from_utf8_lossy(&doctor.stderr)
    );
    let report: DoctorReport = serde_json::from_slice(&doctor.stdout).expect("DoctorReport");
    assert!(
        vault_open_ok(&report),
        "vault_open must be Ok (global KEY gap-fill); report={report:?}"
    );
    assert!(
        !vault_open_skip_missing(&report),
        "must not skip vault_open as key-missing when global KEY is present"
    );
}

/// AC2: shell/process KEY wins over a different global KEY.
#[test]
fn ac2__shell_key_wins_over_global_key() {
    let home_dir = tempdir().unwrap();
    let vault_dir = tempdir().unwrap();
    let vault = vault_dir.path().join("vault.db");

    // Global has the *wrong* key for this vault.
    write_global_env(home_dir.path(), &format!("AI_BRAINS_KEY=\"{KEY_B}\"\n"));

    // Init vault with shell KEY_A (correct key).
    common::hermetic_bin()
        .current_dir(vault_dir.path())
        .env("USERPROFILE", home_dir.path())
        .env("HOME", home_dir.path())
        .env("AI_BRAINS_KEY", KEY_A)
        .env_remove("AI_BRAINS_ALLOW_ZERO_KEY")
        .env_remove("AI_BRAINS_VAULT_PATH")
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("init")
        .assert()
        .success();

    // Doctor with shell KEY_A must open even though global has KEY_B.
    let doctor = common::hermetic_bin()
        .current_dir(vault_dir.path())
        .env("USERPROFILE", home_dir.path())
        .env("HOME", home_dir.path())
        .env("AI_BRAINS_KEY", KEY_A)
        .env_remove("AI_BRAINS_ALLOW_ZERO_KEY")
        .env_remove("AI_BRAINS_VAULT_PATH")
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor shell key wins");

    assert!(
        doctor.status.success(),
        "doctor with correct shell KEY must succeed despite wrong global KEY; stderr={}",
        String::from_utf8_lossy(&doctor.stderr)
    );
    let report: DoctorReport = serde_json::from_slice(&doctor.stdout).expect("DoctorReport");
    assert!(
        vault_open_ok(&report),
        "shell KEY must win over global; report={report:?}"
    );
}

/// AC3: project `.env` KEY wins over global when shell KEY unset.
#[test]
fn ac3__project_env_key_wins_over_global() {
    let home_dir = tempdir().unwrap();
    let project_dir = tempdir().unwrap();
    let vault = project_dir.path().join("vault.db");

    write_global_env(home_dir.path(), &format!("AI_BRAINS_KEY=\"{KEY_B}\"\n"));
    fs::write(
        project_dir.path().join(".env"),
        format!("AI_BRAINS_KEY=\"{KEY_A}\"\n"),
    )
    .expect("write project .env");

    // Init with project KEY (no shell KEY, no --no-project-context so project loads).
    let init = common::hermetic_bin()
        .current_dir(project_dir.path())
        .env("USERPROFILE", home_dir.path())
        .env("HOME", home_dir.path())
        .env_remove("AI_BRAINS_KEY")
        .env_remove("AI_BRAINS_ALLOW_ZERO_KEY")
        .env_remove("AI_BRAINS_VAULT_PATH")
        .arg("--vault-path")
        .arg(&vault)
        .arg("init")
        .output()
        .expect("init project KEY");

    assert!(
        init.status.success(),
        "init with project KEY must succeed; stderr={}",
        String::from_utf8_lossy(&init.stderr)
    );

    let doctor = common::hermetic_bin()
        .current_dir(project_dir.path())
        .env("USERPROFILE", home_dir.path())
        .env("HOME", home_dir.path())
        .env_remove("AI_BRAINS_KEY")
        .env_remove("AI_BRAINS_ALLOW_ZERO_KEY")
        .env_remove("AI_BRAINS_VAULT_PATH")
        .arg("--vault-path")
        .arg(&vault)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor project KEY wins");

    assert!(
        doctor.status.success(),
        "doctor with project KEY must open vault; stderr={}",
        String::from_utf8_lossy(&doctor.stderr)
    );
    let report: DoctorReport = serde_json::from_slice(&doctor.stdout).expect("DoctorReport");
    assert!(
        vault_open_ok(&report),
        "project .env KEY must win over global; report={report:?}"
    );
}

/// AC4: `--no-project-context` still gap-fills global KEY (KEY only in global).
#[test]
fn ac4__no_project_context__still_gapfills_global_key() {
    let home_dir = tempdir().unwrap();
    let project_dir = tempdir().unwrap();
    let vault = project_dir.path().join("vault.db");

    write_global_env(home_dir.path(), &format!("AI_BRAINS_KEY=\"{KEY_A}\"\n"));
    // Project .env has a *different* key — must be ignored under --no-project-context.
    fs::write(
        project_dir.path().join(".env"),
        format!("AI_BRAINS_KEY=\"{KEY_B}\"\n"),
    )
    .expect("write project .env");

    let init = common::hermetic_bin()
        .current_dir(project_dir.path())
        .env("USERPROFILE", home_dir.path())
        .env("HOME", home_dir.path())
        .env_remove("AI_BRAINS_KEY")
        .env_remove("AI_BRAINS_ALLOW_ZERO_KEY")
        .env_remove("AI_BRAINS_VAULT_PATH")
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("init")
        .output()
        .expect("init under no-project-context + global KEY");

    assert!(
        init.status.success(),
        "init must use global KEY under --no-project-context; stderr={}",
        String::from_utf8_lossy(&init.stderr)
    );

    let doctor = common::hermetic_bin()
        .current_dir(project_dir.path())
        .env("USERPROFILE", home_dir.path())
        .env("HOME", home_dir.path())
        .env_remove("AI_BRAINS_KEY")
        .env_remove("AI_BRAINS_ALLOW_ZERO_KEY")
        .env_remove("AI_BRAINS_VAULT_PATH")
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor under no-project-context + global KEY");

    assert!(
        doctor.status.success(),
        "doctor must open with global KEY under --no-project-context; stderr={}",
        String::from_utf8_lossy(&doctor.stderr)
    );
    let report: DoctorReport = serde_json::from_slice(&doctor.stdout).expect("DoctorReport");
    assert!(
        vault_open_ok(&report),
        "global KEY must gap-fill under --no-project-context; report={report:?}"
    );
}
