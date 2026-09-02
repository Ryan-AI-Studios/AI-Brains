//! Live Ledgerful hotspot inject + honest-empty Safety (T279).
//!
//! File-local helper (F29): do not share with CLI `safety.rs` this track.

use crate::ranking::{PinKind, classify_pin_kind};

/// Honest empty Safety body (F3). Must not contain `HOTSPOT:` (AC14 / F9).
pub const SAFETY_EMPTY: &str =
    "No repo-local hotspots above threshold. next: ai-brains safety sync --dry-run";

/// F2: same argv limit as CLI `safety sync` default.
pub const LIVE_HOTSPOT_LIMIT: usize = 5;

/// One parsed live hotspot row (`path` + raw `score`).
#[derive(Debug, Clone, PartialEq)]
pub struct LiveHotspot {
    pub path: String,
    pub score: f64,
}

/// Render `HOTSPOT: {path} score={score:.2}` (F2 / F15). Empty path → skip (AC2).
pub fn format_safety_hotspot_line(path: &str, score: f64) -> Option<String> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    Some(format!("HOTSPOT: {path} score={score:.2}"))
}

/// Parse `ledgerful hotspots --json` stdout (T321 F7).
///
/// Accepts (a) object with `files` array (`schemaVersion` optional) or (b) legacy
/// top-level array after first `trim_start` `[` (T279 F36). Finder: first line
/// `trim_start` `{` or `[`. Raw field `score` (ignore `displayScore`).
///
/// Copy-not-share with CLI `parse_ledgerful_hotspots_json` (F29): this inject
/// caps at `LIVE_HOTSPOT_LIMIT`; CLI uses operator-set `--limit` (default 5).
pub fn parse_hotspots_json(stdout: &str) -> Vec<LiveHotspot> {
    let json_start = stdout.lines().position(|line| {
        let t = line.trim_start();
        t.starts_with('{') || t.starts_with('[')
    });
    let Some(json_start) = json_start else {
        return Vec::new();
    };
    let json_str: String = stdout
        .lines()
        .skip(json_start)
        .collect::<Vec<_>>()
        .join("\n");
    let values = parse_hotspot_value_rows(&json_str);
    let mut out = Vec::new();
    for v in values {
        let path = v
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string();
        if path.trim().is_empty() {
            continue;
        }
        let score = v.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        out.push(LiveHotspot { path, score });
        if out.len() == LIVE_HOTSPOT_LIMIT {
            break;
        }
    }
    out
}

fn parse_hotspot_value_rows(json_str: &str) -> Vec<serde_json::Value> {
    let trimmed = json_str.trim_start();
    if trimmed.starts_with('{') {
        let Ok(obj) = serde_json::from_str::<serde_json::Value>(json_str) else {
            return Vec::new();
        };
        obj.get("files")
            .and_then(|f| f.as_array())
            .cloned()
            .unwrap_or_default()
    } else {
        serde_json::from_str::<Vec<serde_json::Value>>(json_str).unwrap_or_default()
    }
}

/// Frozen deny-list path components (T347 F1). Not filenames (`vendor.rs` stays).
const DENIED_HOTSPOT_COMPONENTS: &[&str] = &["deps_src", "third_party", "vendor", ".git"];

/// Keep a Ledgerful hotspot if it is repo-local and above the score gate (T347).
///
/// Path deny always applies. `include_zero` only disables the `score > 0.0` skip.
pub fn keep_repo_local_hotspot(path: &str, score: f64, include_zero: bool) -> bool {
    if path_has_denied_component(path) {
        return false;
    }
    include_zero || score > 0.0
}

fn path_has_denied_component(path: &str) -> bool {
    path.replace('\\', "/")
        .to_ascii_lowercase()
        .split('/')
        .filter(|seg| !seg.is_empty() && *seg != ".")
        .any(|seg| DENIED_HOTSPOT_COMPONENTS.contains(&seg))
}

/// F7: keep Intelligence substring suppress; live inject drops **leading** vault HOTSPOT only.
pub fn suppress_vault_hotspot_row(
    content: &str,
    live_inject_nonempty: bool,
    has_cg_intelligence: bool,
) -> bool {
    if has_cg_intelligence && content.contains("HOTSPOT:") {
        return true;
    }
    live_inject_nonempty && classify_pin_kind(content) == PinKind::Hotspot
}

/// `AI_BRAINS_PREFLIGHT_SKIP_LIVE_HOTSPOTS` truthy → skip F2 (F13).
pub fn skip_live_hotspots() -> bool {
    match std::env::var("AI_BRAINS_PREFLIGHT_SKIP_LIVE_HOTSPOTS") {
        Ok(v) => {
            let t = v.trim();
            t == "1"
                || t.eq_ignore_ascii_case("true")
                || t.eq_ignore_ascii_case("yes")
                || t.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// Fetch live hotspots via `spawn`. Skip-env must not call `spawn` (AC8).
pub fn fetch_live_hotspots_with(
    spawn: impl FnOnce() -> Result<String, String>,
) -> Vec<LiveHotspot> {
    if skip_live_hotspots() {
        return Vec::new();
    }
    match spawn() {
        Ok(stdout) => {
            let mut rows = parse_hotspots_json(&stdout);
            rows.retain(|h| keep_repo_local_hotspot(&h.path, h.score, false));
            rows
        }
        Err(e) => {
            tracing::warn!(error = %e, "preflight live hotspots skipped");
            Vec::new()
        }
    }
}

/// Project-scoped live inject. Fail-open (F35).
pub fn fetch_live_hotspots() -> Vec<LiveHotspot> {
    fetch_live_hotspots_with(spawn_ledgerful_hotspots_json)
}

fn spawn_ledgerful_hotspots_json() -> Result<String, String> {
    let output = std::process::Command::new("ledgerful")
        .args([
            "hotspots",
            "--json",
            "--limit",
            &LIVE_HOTSPOT_LIMIT.to_string(),
        ])
        .output()
        .map_err(|e| format!("failed to run ledgerful: {e}"))?;
    if !output.status.success() {
        return Err(format!("ledgerful exited {}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;

    #[test]
    fn format_safety_hotspot_line__path_and_score__hotspot_prefix() {
        let line = format_safety_hotspot_line("crates/foo.rs", 0.05)
            .expect("AC2: nonempty path must render");
        assert_eq!(line, "HOTSPOT: crates/foo.rs score=0.05");
        assert!(
            format_safety_hotspot_line("", 0.05).is_none(),
            "AC2: empty path skipped"
        );
        assert!(
            format_safety_hotspot_line("   ", 0.1).is_none(),
            "AC2: whitespace path skipped"
        );
        let line = format_safety_hotspot_line("crates/foo.rs", 0.049).expect("AC2: {:.2} render");
        assert_eq!(line, "HOTSPOT: crates/foo.rs score=0.05");
    }

    #[test]
    fn parse_hotspots_json__log_then_array__one_path() {
        let stdout = "log line with mid-line [bracket] noise\n[{\"path\":\"crates/ai-brains-cli/src/commands/project.rs\",\"score\":0.05,\"complexity\":21.0,\"frequency\":9.1,\"displayScore\":3.93}]\n";
        let got = parse_hotspots_json(stdout);
        assert_eq!(got.len(), 1, "AC9: one hotspot; got {got:?}");
        assert!(
            got[0].path.ends_with("project.rs"),
            "AC9: path is project.rs; got {}",
            got[0].path
        );
        assert!((got[0].score - 0.05).abs() < f64::EPSILON, "AC9: raw score");
        let missing = parse_hotspots_json("no array here [not-a-line-start]\n");
        assert!(
            missing.is_empty(),
            "AC9: missing leading [ is fail-open empty; got {missing:?}"
        );
    }

    #[test]
    fn parse_hotspots_json__envelope_v1_files__raw_score() {
        let stdout = r#"{
  "schemaVersion": 1,
  "files": [
    {
      "path": "crates/ai-brains-cli/src/commands/project.rs",
      "score": 0.037,
      "displayScore": 3.65,
      "complexity": 21,
      "frequency": 7.2
    }
  ],
  "resultCount": 1,
  "limit": 5
}"#;
        let got = parse_hotspots_json(stdout);
        assert_eq!(got.len(), 1, "AC5: one hotspot; got {got:?}");
        assert!(
            got[0].path.ends_with("project.rs"),
            "AC5: path; got {}",
            got[0].path
        );
        assert!(
            (got[0].score - 0.037).abs() < 1e-9,
            "AC5: raw score 0.037; got {}",
            got[0].score
        );
        assert!(
            (got[0].score - 3.65).abs() > 0.1,
            "AC5: must not use displayScore 3.65; got {}",
            got[0].score
        );
    }

    #[test]
    fn parse_hotspots_json__more_than_five__caps() {
        let rows: Vec<String> = (0..8)
            .map(|i| format!(r#"{{"path":"crates/f{i}.rs","score":0.01}}"#))
            .collect();
        let stdout = format!("[{}]", rows.join(","));
        let got = parse_hotspots_json(&stdout);
        assert_eq!(got.len(), LIVE_HOTSPOT_LIMIT, "F2 cap 5; got {got:?}");
        assert_eq!(got[0].path, "crates/f0.rs");
        assert_eq!(got[4].path, "crates/f4.rs");
    }

    #[test]
    fn suppress_vault_hotspot_row__live_inject__leading_only() {
        assert!(
            suppress_vault_hotspot_row("HOTSPOT: src/foo.rs score=0.05", true, false),
            "F7: leading vault HOTSPOT dropped when live inject nonempty"
        );
        assert!(
            !suppress_vault_hotspot_row(
                "CONSTRAINT: Do not confuse HOTSPOT: text with live data",
                true,
                false
            ),
            "F7: buried HOTSPOT: must not drop a CONSTRAINT bearing"
        );
        assert!(
            suppress_vault_hotspot_row(
                "CONSTRAINT: Do not confuse HOTSPOT: text with live data",
                false,
                true
            ),
            "F7: Intelligence suppress keeps substring match"
        );
    }

    #[test]
    fn safety_empty_const__no_hotspot_marker() {
        assert!(
            !SAFETY_EMPTY.contains("HOTSPOT:"),
            "AC14: empty remediator must not bump hotspot counts; got {SAFETY_EMPTY}"
        );
        assert!(
            SAFETY_EMPTY.contains("safety sync --dry-run"),
            "AC14: names next command; got {SAFETY_EMPTY}"
        );
        assert!(
            SAFETY_EMPTY.contains("No repo-local hotspots above threshold"),
            "AC4: F3 first clause; got {SAFETY_EMPTY}"
        );
    }

    #[test]
    fn safety_empty_const__repo_local_threshold__names_dry_run() {
        assert!(
            !SAFETY_EMPTY.contains("HOTSPOT:"),
            "AC14: empty remediator must not bump hotspot counts; got {SAFETY_EMPTY}"
        );
        assert!(
            SAFETY_EMPTY.contains("No repo-local hotspots above threshold"),
            "AC4: F3 first clause; got {SAFETY_EMPTY}"
        );
        assert!(
            SAFETY_EMPTY.contains("ai-brains safety sync --dry-run"),
            "AC4/AC14: names next command; got {SAFETY_EMPTY}"
        );
    }

    #[test]
    fn keep_repo_local_hotspot__deps_src_slash_and_backslash__skipped() {
        assert!(
            !keep_repo_local_hotspot("deps_src/libigl/igl/cut_to_disk.cpp", 0.05, false),
            "AC1: slash deps_src skipped at positive score"
        );
        assert!(
            !keep_repo_local_hotspot(r"deps_src\libigl\igl\active_set.cpp", 0.05, false),
            "AC1: backslash deps_src skipped at positive score"
        );
    }

    #[test]
    fn keep_repo_local_hotspot__include_zero__path_deny_still_applies() {
        assert!(
            !keep_repo_local_hotspot("deps_src/libigl/igl/cut_to_disk.cpp", 0.05, true),
            "AC1b: include_zero does not disable path deny"
        );
    }

    #[test]
    fn keep_repo_local_hotspot__score_zero__skipped_unless_include_zero() {
        let path = "scripts/orca_filament_lib.py";
        assert!(
            !keep_repo_local_hotspot(path, 0.0, false),
            "AC2: score 0 skipped"
        );
        assert!(
            keep_repo_local_hotspot(path, 0.0, true),
            "AC2: include_zero keeps score 0 repo-local"
        );
    }

    #[test]
    fn keep_repo_local_hotspot__nan_negzero_neginf__skipped() {
        let path = "crates/foo.rs";
        assert!(
            !keep_repo_local_hotspot(path, f64::NAN, false),
            "AC2b: NaN skipped"
        );
        assert!(
            !keep_repo_local_hotspot(path, -0.0, false),
            "AC2b: -0.0 skipped"
        );
        assert!(
            !keep_repo_local_hotspot(path, f64::NEG_INFINITY, false),
            "AC2b: -inf skipped"
        );
        assert!(
            keep_repo_local_hotspot(path, f64::NAN, true),
            "AC2b: include_zero does not extra-reject NaN"
        );
    }

    #[test]
    fn keep_repo_local_hotspot__mixed_orca_fixture__keeps_positive_repo_local() {
        assert!(!keep_repo_local_hotspot(
            "deps_src/libigl/igl/cut_to_disk.cpp",
            0.0,
            false
        ));
        assert!(!keep_repo_local_hotspot(
            r"deps_src\libigl\igl\active_set.cpp",
            0.0,
            false
        ));
        assert!(
            keep_repo_local_hotspot("crates/ai-brains-cli/src/commands/project.rs", 0.037, false),
            "AC3: positive repo-local kept"
        );
    }

    #[test]
    fn keep_repo_local_hotspot__third_party_vendor_git__skipped() {
        assert!(!keep_repo_local_hotspot("third_party/foo.c", 0.05, false));
        assert!(!keep_repo_local_hotspot("Vendor/pkg/a.rs", 0.05, false));
        assert!(!keep_repo_local_hotspot(".git/config", 0.05, false));
    }

    #[test]
    fn keep_repo_local_hotspot__vendor_filename__not_skipped() {
        assert!(
            keep_repo_local_hotspot("vendor.rs", 0.05, false),
            "AC6: filename vendor.rs is not a dir component"
        );
    }

    #[test]
    fn fetch_live_hotspots_with__filters_vendored_and_zero__keeps_repo_local() {
        let json = r#"{
  "schemaVersion": 1,
  "files": [
    {"path":"deps_src/libigl/igl/cut_to_disk.cpp","score":0.0},
    {"path":"deps_src/libigl/igl/active_set.cpp","score":0.12},
    {"path":"scripts/orca_filament_lib.py","score":0.0},
    {"path":"crates/ai-brains-cli/src/commands/project.rs","score":0.037}
  ]
}"#;
        let got = fetch_live_hotspots_with(|| Ok(json.to_string()));
        assert_eq!(got.len(), 1, "AC7: only positive repo-local; got {got:?}");
        assert!(
            got[0].path.ends_with("project.rs"),
            "AC7: kept project.rs; got {}",
            got[0].path
        );
        assert!((got[0].score - 0.037).abs() < 1e-9);
    }

    #[test]
    fn skip_live_hotspots_env__truthy__no_spawn() {
        let _g = TempEnv::set("AI_BRAINS_PREFLIGHT_SKIP_LIVE_HOTSPOTS", "1");
        let mut spawned = false;
        let got = fetch_live_hotspots_with(|| {
            spawned = true;
            Ok("[]".to_string())
        });
        assert!(!spawned, "AC8: skip-env must not spawn");
        assert!(got.is_empty(), "AC8: skip returns empty; got {got:?}");
    }
}
