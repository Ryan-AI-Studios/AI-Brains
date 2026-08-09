//! Wiring status probes and JSON status report (F5–F7, F21).

use super::detect::{
    HARNESS_ORDER, HarnessId, HarnessPresence, detect_all_with, join_rel, resolve_home,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Per-harness wiring status (F5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WiringStatus {
    Absent,
    Missing,
    Partial,
    Ok,
    BackendPending,
    Unknown,
}

pub fn wiring_status_label(s: WiringStatus) -> &'static str {
    match s {
        WiringStatus::Absent => "absent",
        WiringStatus::Missing => "missing",
        WiringStatus::Partial => "partial",
        WiringStatus::Ok => "ok",
        WiringStatus::BackendPending => "backend_pending",
        WiringStatus::Unknown => "unknown",
    }
}

/// One row of `harness status` (F21).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessStatus {
    pub id: String,
    pub display_name: String,
    pub present: bool,
    pub binary: Option<String>,
    pub home_path: Option<String>,
    pub wiring: WiringStatus,
    pub install_ready: bool,
    pub targets: Vec<String>,
    pub next_action: String,
}

/// Full JSON status document (F21).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusReport {
    pub schema_version: u32,
    pub home: String,
    pub harnesses: Vec<HarnessStatus>,
}

impl StatusReport {
    pub const SCHEMA_VERSION: u32 = 1;
}

/// Probe wiring for a present harness under `home`.
///
/// For non-ready backends: if the user requested install (`last_status=backend_pending`)
/// and there is still no managed marker, report `backend_pending` (F5/F14) — never a
/// silent claim that capture works.
pub fn probe_wiring(id: HarnessId, home: &Path, present: bool) -> WiringStatus {
    if !present {
        return WiringStatus::Absent;
    }
    let base = match id {
        HarnessId::Grok => probe_grok(home),
        HarnessId::Agy => probe_agy(home),
        HarnessId::Opencode => probe_opencode(home),
        HarnessId::Claude => probe_claude(home),
        HarnessId::Codex => probe_codex(home),
    };
    finalize_wiring_with_prefs(id, home, base)
}

/// Apply prefs-aware honesty for pending backends (F5 / F14).
fn finalize_wiring_with_prefs(id: HarnessId, home: &Path, base: WiringStatus) -> WiringStatus {
    if id.install_ready() {
        return base;
    }
    // Marker present → keep ok/partial (AC2); never downgrade to missing.
    if matches!(
        base,
        WiringStatus::Ok | WiringStatus::Partial | WiringStatus::Unknown
    ) {
        return base;
    }
    // Missing + install requested → backend_pending.
    if matches!(base, WiringStatus::Missing)
        && super::prefs::load_prefs(home).is_backend_pending_requested(id)
    {
        return WiringStatus::BackendPending;
    }
    base
}

fn probe_grok(home: &Path) -> WiringStatus {
    // F6/F7: path marker `~/.grok/hooks/ai-brains.json`
    let marker = join_rel(home, ".grok/hooks/ai-brains.json");
    if marker.is_file() {
        WiringStatus::Ok
    } else {
        WiringStatus::Missing
    }
}

fn probe_agy(home: &Path) -> WiringStatus {
    // F7: top-level key `ai-brains-capture` in either hooks.json location.
    let paths = [
        join_rel(home, ".gemini/config/hooks.json"),
        join_rel(home, ".gemini/antigravity-cli/hooks.json"),
    ];
    for p in &paths {
        if !p.is_file() {
            continue;
        }
        match std::fs::read_to_string(p) {
            Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
                Ok(v) => {
                    if v.get("ai-brains-capture").is_some() {
                        return WiringStatus::Ok;
                    }
                }
                Err(_) => return WiringStatus::Unknown,
            },
            Err(_) => return WiringStatus::Unknown,
        }
    }
    WiringStatus::Missing
}

fn probe_opencode(home: &Path) -> WiringStatus {
    // F6/F7 / F40: managed plugin under OPENCODE_CONFIG_DIR or ~/.config/opencode.
    // Header-scoped: foreign same-name file without T238 managed marker is not ok.
    let config = super::install::opencode_config_dir(home);
    let js = config.join("plugins").join("ai-brains-capture.js");
    let ts = config.join("plugins").join("ai-brains-capture.ts");
    for path in [js, ts] {
        if !path.is_file() {
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(raw) if super::install::has_opencode_managed_marker_header(&raw) => {
                return WiringStatus::Ok;
            }
            Ok(_) => {
                // Same-name file without managed header → not our wiring
                return WiringStatus::Missing;
            }
            Err(_) => return WiringStatus::Unknown,
        }
    }
    WiringStatus::Missing
}

fn probe_claude(home: &Path) -> WiringStatus {
    let settings = join_rel(home, ".claude/settings.json");
    if !settings.is_file() {
        return WiringStatus::Missing;
    }
    match std::fs::read_to_string(&settings) {
        Ok(raw) => {
            // Managed token/path under ~/.ai-brains/hooks/
            let lower = raw.to_ascii_lowercase();
            if lower.contains(".ai-brains") && lower.contains("hooks")
                || lower.contains("ai-brains-capture")
            {
                WiringStatus::Ok
            } else if raw.contains("\"hooks\"") {
                // Hooks present but not ours → partial signal not required; missing ours.
                WiringStatus::Missing
            } else {
                WiringStatus::Missing
            }
        }
        Err(_) => WiringStatus::Unknown,
    }
}

fn probe_codex(home: &Path) -> WiringStatus {
    let hooks = join_rel(home, ".codex/hooks.json");
    if !hooks.is_file() {
        // Schema may live elsewhere; soft unknown only when home present but unreadable.
        return WiringStatus::Missing;
    }
    match std::fs::read_to_string(&hooks) {
        Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(v) => {
                if v.get("ai-brains-capture").is_some()
                    || raw.to_ascii_lowercase().contains("ai-brains")
                {
                    WiringStatus::Ok
                } else {
                    WiringStatus::Missing
                }
            }
            Err(_) => WiringStatus::Unknown,
        },
        Err(_) => WiringStatus::Unknown,
    }
}

/// Install/config target paths for human + JSON (under home only).
pub fn targets_for(id: HarnessId, home: &Path) -> Vec<String> {
    match id {
        HarnessId::Grok => vec![
            join_rel(home, ".grok/hooks/ai-brains.json")
                .display()
                .to_string(),
            join_rel(home, ".ai-brains/hooks/grok-capture.ps1")
                .display()
                .to_string(),
        ],
        HarnessId::Agy => vec![
            join_rel(home, ".gemini/config/hooks.json")
                .display()
                .to_string(),
            join_rel(home, ".ai-brains/hooks/agy-stop.ps1")
                .display()
                .to_string(),
        ],
        HarnessId::Opencode => {
            // F40: honor OPENCODE_CONFIG_DIR when set (same as install/probe)
            let config = super::install::opencode_config_dir(home);
            vec![
                config
                    .join("plugins")
                    .join("ai-brains-capture.js")
                    .display()
                    .to_string(),
            ]
        }
        HarnessId::Claude => vec![
            join_rel(home, ".claude/settings.json")
                .display()
                .to_string(),
        ],
        HarnessId::Codex => vec![join_rel(home, ".codex/hooks.json").display().to_string()],
    }
}

/// Exact next-action command (F40).
pub fn next_action_for(id: HarnessId, wiring: WiringStatus) -> String {
    match wiring {
        WiringStatus::Absent => "n/a (not installed on machine)".to_string(),
        WiringStatus::Ok => "ai-brains harness status".to_string(),
        WiringStatus::BackendPending => format!(
            "backend pending ({}); see Docs/CAPABILITIES.md",
            id.pending_track().unwrap_or("track TBD")
        ),
        WiringStatus::Missing | WiringStatus::Partial | WiringStatus::Unknown => {
            if id.install_ready() {
                format!(
                    "ai-brains harness install --harness {} --dry-run",
                    id.as_str()
                )
            } else {
                format!(
                    "ai-brains harness install --harness {} --dry-run  # backend pending ({})",
                    id.as_str(),
                    id.pending_track().unwrap_or("track TBD")
                )
            }
        }
    }
}

/// Build full status report from presence + wiring probes.
pub fn collect_status_report(home: Option<&Path>) -> StatusReport {
    let home_buf: PathBuf;
    let home_ref: &Path = match home {
        Some(h) => h,
        None => {
            home_buf = resolve_home().unwrap_or_else(|| PathBuf::from("."));
            &home_buf
        }
    };
    let presence = detect_all_with(Some(home_ref));
    collect_status_from_presence(home_ref, &presence)
}

pub fn collect_status_from_presence(home: &Path, presence: &[HarnessPresence]) -> StatusReport {
    let mut harnesses = Vec::with_capacity(HARNESS_ORDER.len());
    for p in presence {
        let wiring = probe_wiring(p.id, home, p.present);
        harnesses.push(HarnessStatus {
            id: p.id.as_str().to_string(),
            display_name: p.id.display_name().to_string(),
            present: p.present,
            binary: p.binary.clone(),
            home_path: p.home_path.clone(),
            wiring,
            install_ready: p.id.install_ready(),
            targets: targets_for(p.id, home),
            next_action: next_action_for(p.id, wiring),
        });
    }
    StatusReport {
        schema_version: StatusReport::SCHEMA_VERSION,
        home: home.display().to_string(),
        harnesses,
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;
    use tempfile::tempdir;

    #[test]
    fn wiring__grok_home_only__missing() {
        // AC1
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        std::fs::create_dir_all(home.join(".grok")).expect("mkdir");
        let _path = TempEnv::set("PATH", "");
        let status = probe_wiring(HarnessId::Grok, home, true);
        assert_eq!(status, WiringStatus::Missing);
    }

    #[test]
    fn wiring__grok_managed_file__ok() {
        // AC2
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let marker = home.join(".grok").join("hooks").join("ai-brains.json");
        std::fs::create_dir_all(marker.parent().unwrap()).expect("mkdir");
        std::fs::write(&marker, b"{}").expect("write");
        let status = probe_wiring(HarnessId::Grok, home, true);
        assert_ne!(status, WiringStatus::Missing);
        assert!(matches!(
            status,
            WiringStatus::Ok | WiringStatus::Partial | WiringStatus::BackendPending
        ));
    }

    #[test]
    fn wiring__grok_after_real_install__ok() {
        // T237: Grok install_ready → real marker → wiring ok (not backend_pending).
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        std::fs::create_dir_all(home.join(".grok")).expect("mkdir");
        super::super::install::install_grok(home, false).expect("install");
        assert_eq!(probe_wiring(HarnessId::Grok, home, true), WiringStatus::Ok);
    }

    #[test]
    fn wiring__opencode_after_real_install__ok() {
        // T238: OpenCode install_ready → real plugin → wiring ok (not backend_pending).
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        std::fs::create_dir_all(home.join(".config").join("opencode")).expect("mkdir");
        super::super::install::install_opencode(home, false).expect("install");
        assert_eq!(
            probe_wiring(HarnessId::Opencode, home, true),
            WiringStatus::Ok
        );
    }

    #[test]
    fn wiring__opencode_foreign_same_name__missing() {
        // Codex R2: same-name file without managed header is not wiring ok
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let plugins = home.join(".config").join("opencode").join("plugins");
        std::fs::create_dir_all(&plugins).expect("mkdir");
        std::fs::write(
            plugins.join("ai-brains-capture.js"),
            b"export default function foreign() { return {}; }\n",
        )
        .expect("write foreign");
        assert_eq!(
            probe_wiring(HarnessId::Opencode, home, true),
            WiringStatus::Missing
        );
    }

    #[test]
    fn wiring__agy_without_key__missing() {
        // AC3
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let hooks = home.join(".gemini").join("config").join("hooks.json");
        std::fs::create_dir_all(hooks.parent().unwrap()).expect("mkdir");
        std::fs::write(&hooks, br#"{"other-hook":{}}"#).expect("write");
        assert_eq!(
            probe_wiring(HarnessId::Agy, home, true),
            WiringStatus::Missing
        );
    }

    #[test]
    fn wiring__agy_with_managed_key__ok() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let hooks = home.join(".gemini").join("config").join("hooks.json");
        std::fs::create_dir_all(hooks.parent().unwrap()).expect("mkdir");
        std::fs::write(&hooks, br#"{"ai-brains-capture":{"Stop":[]},"other":{}}"#).expect("write");
        assert_eq!(probe_wiring(HarnessId::Agy, home, true), WiringStatus::Ok);
    }

    #[test]
    fn status_report__json_schema_order() {
        // AC7
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        std::fs::create_dir_all(home.join(".grok")).expect("mkdir");
        let _path = TempEnv::set("PATH", "");
        let report = collect_status_report(Some(home));
        assert_eq!(report.schema_version, 1);
        let json = serde_json::to_string(&report).expect("ser");
        let back: StatusReport = serde_json::from_str(&json).expect("de");
        assert_eq!(back.harnesses.len(), 5);
        let ids: Vec<&str> = back.harnesses.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, vec!["grok", "agy", "opencode", "claude", "codex"]);
        // Required keys present on first row
        let first = &back.harnesses[0];
        assert!(!first.display_name.is_empty());
        assert!(!first.next_action.is_empty());
        assert!(!first.targets.is_empty());
        // Targets under home (AC12)
        for t in &first.targets {
            assert!(
                t.starts_with(&home.display().to_string()) || Path::new(t).starts_with(home),
                "target not under home: {t}"
            );
        }
    }

    #[test]
    fn targets__all_under_home() {
        let home = Path::new(r"C:\Users\test-user");
        for id in HARNESS_ORDER {
            for t in targets_for(*id, home) {
                assert!(
                    t.contains("test-user") || t.starts_with(r"C:\Users\test-user"),
                    "target not under home: {t}"
                );
            }
        }
    }
}
