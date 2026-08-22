//! Live Ledgerful hotspot inject + honest-empty Safety (T279).
//!
//! File-local helper (F29): do not share with CLI `safety.rs` this track.

/// Honest empty Safety body (F3). T279 red stub includes `HOTSPOT:` so AC14 fails.
pub const SAFETY_EMPTY: &str = "HOTSPOT: none";

/// One parsed live hotspot row (`path` + raw `score`).
#[derive(Debug, Clone, PartialEq)]
pub struct LiveHotspot {
    pub path: String,
    pub score: f64,
}

/// Render `HOTSPOT: {path} score={score:.2}` (F2 / F15). Empty path → skip (AC2).
/// T279 red stub: always `None` so AC2 fails.
pub fn format_safety_hotspot_line(_path: &str, _score: f64) -> Option<String> {
    None
}

/// Parse `ledgerful hotspots --json` stdout. Finder is F36 (first `trim_start` `[`).
/// T279 red stub: always empty so AC9 fails.
pub fn parse_hotspots_json(_stdout: &str) -> Vec<LiveHotspot> {
    Vec::new()
}

/// `AI_BRAINS_PREFLIGHT_SKIP_LIVE_HOTSPOTS` truthy → skip F2 (F13).
pub fn skip_live_hotspots() -> bool {
    false
}

/// Fetch live hotspots via `spawn`. Skip-env must not call `spawn` (AC8).
/// T279 red stub: ignores skip so AC8 fails.
pub fn fetch_live_hotspots_with(
    spawn: impl FnOnce() -> Result<String, String>,
) -> Vec<LiveHotspot> {
    let _ = skip_live_hotspots();
    match spawn() {
        Ok(stdout) => parse_hotspots_json(&stdout),
        Err(_) => Vec::new(),
    }
}

/// Project-scoped live inject. Fail-open (F35).
#[allow(dead_code)]
pub fn fetch_live_hotspots() -> Vec<LiveHotspot> {
    fetch_live_hotspots_with(spawn_ledgerful_hotspots_json)
}

fn spawn_ledgerful_hotspots_json() -> Result<String, String> {
    let output = std::process::Command::new("ledgerful")
        .args(["hotspots", "--json", "--limit", "5"])
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
    fn safety_empty_const__no_hotspot_marker() {
        assert!(
            !SAFETY_EMPTY.contains("HOTSPOT:"),
            "AC14: empty remediator must not bump hotspot counts; got {SAFETY_EMPTY}"
        );
        assert!(
            SAFETY_EMPTY.contains("safety sync --dry-run"),
            "AC14: names next command; got {SAFETY_EMPTY}"
        );
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
