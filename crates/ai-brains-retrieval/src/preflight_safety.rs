//! Live Ledgerful hotspot inject + honest-empty Safety (T279).
//!
//! File-local helper (F29): do not share with CLI `safety.rs` this track.

use crate::ranking::{PinKind, classify_pin_kind};

/// Honest empty Safety body (F3). Must not contain `HOTSPOT:` (AC14 / F9).
pub const SAFETY_EMPTY: &str = "No in-context hotspots. next: ai-brains safety sync --dry-run";

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

/// Parse `ledgerful hotspots --json` stdout. Finder is F36 (first `trim_start` `[`).
pub fn parse_hotspots_json(stdout: &str) -> Vec<LiveHotspot> {
    let json_start = stdout
        .lines()
        .position(|line| line.trim_start().starts_with('['));
    let Some(json_start) = json_start else {
        return Vec::new();
    };
    let json_str: String = stdout
        .lines()
        .skip(json_start)
        .collect::<Vec<_>>()
        .join("\n");
    let Ok(values) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) else {
        return Vec::new();
    };
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
        Ok(stdout) => parse_hotspots_json(&stdout),
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
