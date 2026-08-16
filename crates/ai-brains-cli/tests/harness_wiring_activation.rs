//! T245 PART A — hermetic `all-ready` list token + vault-free harness CLI (AC1/AC10/AC11/AC14).

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn hermetic_harness_cmd(home: &Path) -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin_no_key(); // vault-path-free harness
    cmd.env("USERPROFILE", home);
    cmd.env("HOME", home);
    cmd.env("PATH", ""); // T235 F32 PATH scrub
    cmd
}

fn hermetic_harness_home() -> (tempfile::TempDir, assert_cmd::Command) {
    let dir = tempdir().unwrap();
    let home = dir.path();
    let cmd = hermetic_harness_cmd(home);
    (dir, cmd)
}

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    fn rec(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    rec(&p, out);
                } else {
                    out.push(p);
                }
            }
        }
    }
    rec(root, &mut out);
    out.sort();
    out
}

fn combined(output: &std::process::Output) -> String {
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&output.stdout));
    s.push_str(&String::from_utf8_lossy(&output.stderr));
    s
}

/// T253 AC12: `install --harness all-ready --dry-run` plans five ready backends and writes nothing.
#[test]
fn harness_wiring_activation__all_ready_dry_run__zero_writes() {
    let (dir, mut cmd) = hermetic_harness_home();
    let home = dir.path();
    let before = walk_files(home);

    let out = cmd
        .args(["harness", "install", "--harness", "all-ready", "--dry-run"])
        .output()
        .expect("all-ready dry-run");

    let text = combined(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "all-ready dry-run must exit 0; out={text}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
    assert!(
        stdout.contains("grok")
            && stdout.contains("agy")
            && stdout.contains("opencode")
            && stdout.contains("claude")
            && stdout.contains("codex"),
        "stdout must mention five ready plans; got: {stdout}"
    );
    assert!(
        stdout.contains("[dry-run]"),
        "must print dry-run plans; got: {stdout}"
    );
    assert!(
        stdout.contains("/hooks") && stdout.contains("ai-brains-capture"),
        "Codex dry-run must print /hooks trust next-action; got: {stdout}"
    );
    let after = walk_files(home);
    assert_eq!(
        before, after,
        "dry-run must not write files under temp home"
    );
}

/// AC10: unknown `--harness foo` exits 2.
#[test]
fn harness_wiring_activation__unknown_harness_foo__exit_2() {
    let (_dir, mut cmd) = hermetic_harness_home();
    let out = cmd
        .args(["harness", "install", "--harness", "foo", "--dry-run"])
        .output()
        .expect("unknown harness foo");
    assert_eq!(
        out.status.code(),
        Some(2),
        "unknown --harness foo must exit 2; out={}",
        combined(&out)
    );
}

/// AC10: `allready` (no hyphen) is not the list token — exit 2.
#[test]
fn harness_wiring_activation__unknown_harness_allready__exit_2() {
    let (_dir, mut cmd) = hermetic_harness_home();
    let out = cmd
        .args(["harness", "install", "--harness", "allready", "--dry-run"])
        .output()
        .expect("unknown harness allready");
    assert_eq!(
        out.status.code(),
        Some(2),
        "unknown --harness allready must exit 2; out={}",
        combined(&out)
    );
}

#[test]
fn harness_wiring_activation__all_still_accepted() {
    let (dir, mut cmd) = hermetic_harness_home();
    let home = dir.path();
    let before = walk_files(home);
    let out = cmd
        .args(["harness", "install", "--harness", "all", "--dry-run"])
        .output()
        .expect("all dry-run");
    let text = combined(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "--harness all dry-run must exit 0; out={text}"
    );
    assert_eq!(before, walk_files(home), "all dry-run writes nothing");
}

/// reset-decline does not use resolve_harness_list; `all-ready` stays unknown (exit 2).
#[test]
fn harness_wiring_activation__reset_decline_all_ready__exit_2() {
    let (_dir, mut cmd) = hermetic_harness_home();
    let out = cmd
        .args(["harness", "reset-decline", "--harness", "all-ready"])
        .output()
        .expect("reset-decline all-ready");
    assert_eq!(
        out.status.code(),
        Some(2),
        "reset-decline --harness all-ready must exit 2; out={}",
        combined(&out)
    );
}

/// AC14: `harness status` works without a vault path.
#[test]
fn harness_wiring_activation__status__vault_path_free() {
    let (_dir, mut cmd) = hermetic_harness_home();
    let out = cmd
        .args(["harness", "status"])
        .output()
        .expect("harness status");
    let text = combined(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "harness status without vault must exit 0; out={text}"
    );
    assert!(
        text.to_ascii_lowercase().contains("harness"),
        "status should mention harnesses; got: {text}"
    );
}

/// AC11 C7: existing grok writer targets stay under the hermetic home.
#[test]
fn harness_wiring_activation__yes_grok__targets_under_home() {
    let (dir, mut cmd) = hermetic_harness_home();
    let home = dir.path();
    let out = cmd
        .args(["harness", "install", "--harness", "grok", "--yes"])
        .output()
        .expect("grok --yes");
    let text = combined(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "grok --yes must exit 0 in temp home; out={text}"
    );
    let written = walk_files(home);
    assert!(
        !written.is_empty(),
        "grok --yes should write hook files under temp home"
    );
    for path in &written {
        assert!(
            path.starts_with(home),
            "C7: written path {} not under {}",
            path.display(),
            home.display()
        );
    }
}

/// AC2: `all-ready --yes` with CLI home writes IDE + plugin bundle; no top-level CLI hooks.json.
#[test]
fn harness_wiring_activation__all_ready_yes__with_cli_dir__bundle_and_ok() {
    let dir = tempdir().unwrap();
    let home = dir.path();
    std::fs::create_dir_all(home.join(".gemini").join("antigravity-cli")).expect("cli home");
    std::fs::create_dir_all(home.join(".grok")).expect("grok");
    std::fs::create_dir_all(home.join(".config").join("opencode")).expect("opencode");

    let out = hermetic_harness_cmd(home)
        .args(["harness", "install", "--harness", "all-ready", "--yes"])
        .output()
        .expect("all-ready --yes");
    let text = combined(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "all-ready --yes must exit 0; out={text}"
    );

    assert!(
        home.join(".grok")
            .join("hooks")
            .join("ai-brains.json")
            .is_file()
    );
    assert!(
        home.join(".gemini")
            .join("config")
            .join("hooks.json")
            .is_file()
    );
    let bundle = home
        .join(".gemini")
        .join("antigravity-cli")
        .join("plugins")
        .join("ai-brains-capture");
    assert!(bundle.join("plugin.json").is_file(), "plugin.json missing");
    assert!(
        bundle.join("hooks.json").is_file(),
        "bundle hooks.json missing"
    );
    assert!(
        !home
            .join(".gemini")
            .join("antigravity-cli")
            .join("hooks.json")
            .exists(),
        "must not write top-level antigravity-cli/hooks.json"
    );
    assert!(
        home.join(".config")
            .join("opencode")
            .join("plugins")
            .join("ai-brains-capture.js")
            .is_file()
    );
    assert!(
        home.join(".claude").join("settings.json").is_file(),
        "claude settings.json written"
    );
    assert!(
        home.join(".ai-brains")
            .join("hooks")
            .join("claude-capture.ps1")
            .is_file(),
        "claude wrapper written"
    );
    assert!(
        home.join(".codex").join("hooks.json").is_file(),
        "codex hooks.json written"
    );
    assert!(
        home.join(".ai-brains")
            .join("hooks")
            .join("codex-capture.ps1")
            .is_file(),
        "codex wrapper written"
    );
    assert!(
        !home.join(".codex").join("config.toml").exists(),
        "must not create Codex config.toml"
    );

    let plugin: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(bundle.join("plugin.json")).expect("read"))
            .expect("plugin json");
    assert_eq!(plugin["name"], "ai-brains-capture");

    let status = hermetic_harness_cmd(home)
        .args(["harness", "status", "--format", "json"])
        .output()
        .expect("status");
    assert_eq!(status.status.code(), Some(0), "{}", combined(&status));
    let report: serde_json::Value = serde_json::from_slice(&status.stdout).expect("status json");
    assert_eq!(report["schema_version"], 1, "AC14 schema_version frozen");
    for id in ["grok", "agy", "opencode", "claude", "codex"] {
        let row = report["harnesses"]
            .as_array()
            .expect("harnesses")
            .iter()
            .find(|h| h["id"] == id)
            .unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(row["wiring"], "ok", "{id} wiring={row}");
        assert_eq!(row["install_ready"], true, "{id} install_ready={row}");
    }
}

/// AC3: without antigravity-cli, AGY writes only IDE config/hooks.json.
#[test]
fn harness_wiring_activation__all_ready_yes__without_cli_dir__no_antigravity_cli() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let out = hermetic_harness_cmd(home)
        .args(["harness", "install", "--harness", "agy", "--yes"])
        .output()
        .expect("agy --yes");
    assert_eq!(out.status.code(), Some(0), "{}", combined(&out));
    assert!(
        home.join(".gemini")
            .join("config")
            .join("hooks.json")
            .is_file()
    );
    assert!(
        !home.join(".gemini").join("antigravity-cli").exists(),
        "must not create antigravity-cli"
    );
}

/// AC4 / AC18: uninstall removes IDE managed key + only our plugin bundle; sibling plugins stay.
#[test]
fn harness_wiring_activation__uninstall_agy__keeps_foreign_and_siblings() {
    let dir = tempdir().unwrap();
    let home = dir.path();
    let cli = home.join(".gemini").join("antigravity-cli");
    let sibling = cli.join("plugins").join("other-plugin");
    std::fs::create_dir_all(&sibling).expect("sibling");
    std::fs::write(
        sibling.join("plugin.json"),
        b"{\"name\":\"other-plugin\"}\n",
    )
    .expect("sib");

    let inst = hermetic_harness_cmd(home)
        .args(["harness", "install", "--harness", "agy", "--yes"])
        .output()
        .expect("install");
    assert_eq!(inst.status.code(), Some(0), "{}", combined(&inst));

    let hooks = home.join(".gemini").join("config").join("hooks.json");
    let mut map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&hooks).expect("read")).expect("json");
    map.as_object_mut()
        .expect("obj")
        .insert("foreign-hook".into(), serde_json::json!({"Stop":[]}));
    std::fs::write(&hooks, serde_json::to_string_pretty(&map).expect("ser")).expect("write");

    let un = hermetic_harness_cmd(home)
        .args(["harness", "uninstall", "--harness", "agy", "--yes"])
        .output()
        .expect("uninstall");
    assert_eq!(un.status.code(), Some(0), "{}", combined(&un));

    let after: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&hooks).expect("read2")).expect("json2");
    assert!(after.get("ai-brains-capture").is_none());
    assert!(after.get("foreign-hook").is_some());
    assert!(
        !cli.join("plugins").join("ai-brains-capture").exists(),
        "bundle dir removed"
    );
    assert!(
        sibling.join("plugin.json").is_file(),
        "sibling plugin remains"
    );
    assert!(cli.is_dir(), "antigravity-cli home remains");
}
