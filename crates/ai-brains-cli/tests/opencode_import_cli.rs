//! T371 AC3/AC9 — hermetic `opencode-import` this-project vs `--global`.
#![allow(clippy::disallowed_methods, non_snake_case)]

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;

const SID_A: &str = "ses_t371_a";
const SID_B: &str = "ses_t371_b";

fn init_vault(vault: &Path) {
    common::hermetic_bin()
        .arg("--vault-path")
        .arg(vault)
        .arg("--no-project-context")
        .arg("init")
        .assert()
        .success();
}

fn strip_harness_homes(cmd: &mut assert_cmd::Command) {
    cmd.env_remove("CURSOR_HOME");
    cmd.env_remove("GROK_HOME");
    cmd.env_remove("CLAUDE_HOME");
    cmd.env_remove("CODEX_HOME");
    cmd.env_remove("OPENCODE_BIN_PATH");
    cmd.env_remove("OPENCODE_CONFIG_DIR");
}

fn register_project(vault: &Path, work_dir: &Path) -> String {
    fs::create_dir_all(work_dir).expect("work");
    let out = common::hermetic_bin()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("context")
        .output()
        .expect("context");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let content = fs::read_to_string(work_dir.join(".env")).expect(".env");
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_PROJECT_ID=") {
            return rest.trim().to_string();
        }
    }
    panic!("no project id in .env: {content}");
}

fn register_path(vault: &Path, project_id: &str, path: &str) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("register-path")
        .arg(project_id)
        .arg(path)
        .assert()
        .success();
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("epoch")
        .as_millis()
}

fn write_stub(dir: &Path, dir_a: &str, dir_b: &str) -> (PathBuf, PathBuf, PathBuf) {
    fs::create_dir_all(dir).expect("stub dir");
    let list_path = dir.join("list.json");
    let argv_path = dir.join("argv.log");
    let list = format!(
        r#"[{{"id":"{SID_A}","directory":{},"updated":{u}}},{{"id":"{SID_B}","directory":{},"updated":{u}}}]"#,
        serde_json::to_string(dir_a).expect("a"),
        serde_json::to_string(dir_b).expect("b"),
        u = now_ms(),
    );
    fs::write(&list_path, list).expect("list");
    #[cfg(windows)]
    let stub = {
        let stub = dir.join("opencode.cmd");
        fs::write(
            &stub,
            "@echo off\r\n>>\"%OPENCODE_STUB_ARGV%\" echo %*\r\nif /I \"%1\"==\"export\" exit /b 1\r\ntype \"%OPENCODE_STUB_LIST%\"\r\n",
        )
        .expect("stub");
        stub
    };
    #[cfg(not(windows))]
    let stub = {
        let stub = dir.join("opencode");
        fs::write(
            &stub,
            "#!/bin/sh\necho \"$@\" >> \"$OPENCODE_STUB_ARGV\"\n[ \"$1\" = export ] && exit 1\ncat \"$OPENCODE_STUB_LIST\"\n",
        )
        .expect("stub");
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&stub).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&stub, perms).expect("chmod");
        stub
    };
    (stub, argv_path, list_path)
}

fn found_from_stderr(stderr: &str) -> Option<usize> {
    for line in stderr.lines() {
        if let Some(rest) = line.strip_prefix("[OpenCode] Import stats: found=") {
            let n = rest.split_whitespace().next()?;
            return n.parse().ok();
        }
    }
    None
}

fn apply_stub_env(
    cmd: &mut assert_cmd::Command,
    stub: &Path,
    isolated_home: &Path,
    argv_path: &Path,
    list_path: &Path,
) {
    strip_harness_homes(cmd);
    cmd.env("AI_BRAINS_OPENCODE_BIN", stub);
    cmd.env("OPENCODE_STUB_ARGV", argv_path);
    cmd.env("OPENCODE_STUB_LIST", list_path);
    cmd.env("APPDATA", isolated_home);
    cmd.env("PATH", isolated_home);
}

#[cfg(windows)]
#[test]
fn opencode_import__this_project_directory__found_one_global_two() {
    let root = tempdir().expect("root");
    let vault = root.path().join("v.db");
    init_vault(&vault);
    let work = root.path().join("work");
    let pid = register_project(&vault, &work);
    register_path(&vault, &pid, r"C:\dev\ai-brains");

    let isolated = root.path().join("isolated");
    let (stub, argv_path, list_path) = write_stub(&isolated, r"C:\dev\AI-Brains", r"C:\dev\other");

    let mut cmd = common::hermetic_bin();
    apply_stub_env(&mut cmd, &stub, &isolated, &argv_path, &list_path);
    let out = cmd
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .arg("opencode-import")
        .arg("--days")
        .arg("30")
        .arg("--dry-run")
        .output()
        .expect("run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "stderr={stderr}");
    assert_eq!(found_from_stderr(&stderr), Some(1), "stderr={stderr}");
    assert!(stderr.contains("scope=this-project"), "stderr={stderr}");
    assert!(stderr.contains("unfiltered vendor list"), "stderr={stderr}");
    let argv = fs::read_to_string(&argv_path).unwrap_or_default();
    assert!(argv.contains("session"), "argv={argv}");
    assert!(argv.contains("list"), "argv={argv}");
    assert!(!argv.to_ascii_lowercase().contains("export"), "argv={argv}");

    let mut cov = common::hermetic_bin();
    apply_stub_env(&mut cov, &stub, &isolated, &argv_path, &list_path);
    let cov_out = cov
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .arg("capture")
        .arg("coverage")
        .arg("--format")
        .arg("json")
        .arg("--days")
        .arg("30")
        .output()
        .expect("coverage");
    let cov_stdout = String::from_utf8_lossy(&cov_out.stdout);
    assert!(cov_out.status.success(), "cov={cov_stdout}");
    let json: serde_json::Value = serde_json::from_str(&cov_stdout).expect("json");
    let oc = json["sources"]
        .as_array()
        .expect("sources")
        .iter()
        .find(|s| s["source"] == "opencode")
        .expect("oc");
    assert_eq!(oc["vault_sessions"].as_u64(), Some(0), "oc={oc}");

    fs::write(&argv_path, "").expect("reset argv");
    let mut cmd_g = common::hermetic_bin();
    apply_stub_env(&mut cmd_g, &stub, &isolated, &argv_path, &list_path);
    let out_g = cmd_g
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .arg("opencode-import")
        .arg("--days")
        .arg("30")
        .arg("--dry-run")
        .arg("--global")
        .output()
        .expect("run global");
    let stderr_g = String::from_utf8_lossy(&out_g.stderr);
    assert!(out_g.status.success(), "stderr={stderr_g}");
    assert!(
        found_from_stderr(&stderr_g).is_some_and(|n| n >= 2),
        "stderr={stderr_g}"
    );
    assert!(
        stderr_g.contains("unfiltered vendor list"),
        "stderr={stderr_g}"
    );
}

#[test]
fn opencode_import__help__mentions_global_and_dry_run() {
    let mut cmd = common::hermetic_bin();
    strip_harness_homes(&mut cmd);
    let out = cmd
        .arg("opencode-import")
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "stdout={stdout}");
    assert!(stdout.contains("--global"), "stdout={stdout}");
    assert!(stdout.contains("--dry-run"), "stdout={stdout}");
}

#[cfg(windows)]
#[test]
fn opencode_import__pid_without_aliases__scopes_cwd_not_global() {
    let root = tempdir().expect("root");
    let vault = root.path().join("v.db");
    init_vault(&vault);
    let work = root.path().join("work");
    fs::create_dir_all(&work).expect("work");
    let pid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaa71";
    let ws = work.to_string_lossy();
    let isolated = root.path().join("isolated");
    let (stub, argv_path, list_path) = write_stub(&isolated, &ws, r"C:\dev\other");

    let mut cmd = common::hermetic_bin();
    apply_stub_env(&mut cmd, &stub, &isolated, &argv_path, &list_path);
    let out = cmd
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", pid)
        .arg("opencode-import")
        .arg("--days")
        .arg("30")
        .arg("--dry-run")
        .output()
        .expect("run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "stderr={stderr}");
    assert_eq!(found_from_stderr(&stderr), Some(1), "stderr={stderr}");

    let mut cmd_g = common::hermetic_bin();
    apply_stub_env(&mut cmd_g, &stub, &isolated, &argv_path, &list_path);
    let out_g = cmd_g
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", pid)
        .arg("opencode-import")
        .arg("--days")
        .arg("30")
        .arg("--dry-run")
        .arg("--global")
        .output()
        .expect("run global");
    let stderr_g = String::from_utf8_lossy(&out_g.stderr);
    assert!(out_g.status.success(), "stderr={stderr_g}");
    assert!(
        found_from_stderr(&stderr_g).is_some_and(|n| n >= 2),
        "stderr={stderr_g}"
    );
}
