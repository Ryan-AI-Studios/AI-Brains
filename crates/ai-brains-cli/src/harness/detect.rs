//! Pure harness presence detection (PATH + home roots). Hermetic via TempEnv.

use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

/// Fixed harness set and report order (F1 / F21).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessId {
    Grok,
    Agy,
    Opencode,
    Claude,
    Codex,
}

/// Canonical iteration order — never HashMap order.
pub const HARNESS_ORDER: &[HarnessId] = &[
    HarnessId::Grok,
    HarnessId::Agy,
    HarnessId::Opencode,
    HarnessId::Claude,
    HarnessId::Codex,
];

impl HarnessId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Grok => "grok",
            Self::Agy => "agy",
            Self::Opencode => "opencode",
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Grok => "Grok Build",
            Self::Agy => "Antigravity 2 / AGY",
            Self::Opencode => "OpenCode",
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
        }
    }

    /// Primary PATH binary name (F3).
    pub fn binary_name(self) -> &'static str {
        match self {
            Self::Grok => "grok",
            Self::Agy => "agy",
            Self::Opencode => "opencode",
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }

    /// AGY (T235/T236) and Grok (T237) have real install writers.
    pub fn install_ready(self) -> bool {
        matches!(self, Self::Agy | Self::Grok)
    }

    /// Pending track id when install_ready is false.
    pub fn pending_track(self) -> Option<&'static str> {
        match self {
            Self::Grok | Self::Agy => None,
            Self::Opencode => Some("T238"),
            Self::Claude => Some("T238+"),
            Self::Codex => Some("T238+"),
        }
    }

    /// Relative home/config roots under user home (F4).
    pub fn home_rel_paths(self) -> &'static [&'static str] {
        match self {
            Self::Grok => &[".grok"],
            Self::Agy => &[".gemini/antigravity-cli", ".gemini/config"],
            Self::Opencode => &[".config/opencode"],
            Self::Claude => &[".claude"],
            Self::Codex => &[".codex"],
        }
    }
}

/// Parse a harness id string; unknown → error for CLI exit 2.
pub fn parse_harness_id(raw: &str) -> Result<HarnessId, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "grok" => Ok(HarnessId::Grok),
        "agy" | "antigravity" => Ok(HarnessId::Agy),
        "opencode" => Ok(HarnessId::Opencode),
        "claude" => Ok(HarnessId::Claude),
        "codex" => Ok(HarnessId::Codex),
        other => Err(format!(
            "unknown harness '{other}'; expected one of: grok, agy, opencode, claude, codex"
        )),
    }
}

/// Presence probe result for one harness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessPresence {
    pub id: HarnessId,
    pub present: bool,
    pub binary: Option<String>,
    pub home_path: Option<String>,
}

/// Resolve user home: USERPROFILE then HOME (T205 / F4).
pub fn resolve_home() -> Option<PathBuf> {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// First successful PATH resolve for `binary` (F39).
///
/// Skips PermissionDenied entries; skips non-UTF8 display but still probes Path.
/// No shell-out-per-entry.
pub fn resolve_on_path(binary: &str) -> Option<PathBuf> {
    let path_os = env::var_os("PATH")?;
    for entry in env::split_paths(&path_os) {
        // Skip empty PATH segments.
        if entry.as_os_str().is_empty() {
            continue;
        }
        let candidates = path_binary_candidates(&entry, binary);
        for candidate in candidates {
            match std::fs::metadata(&candidate) {
                Ok(meta) if meta.is_file() => return Some(candidate),
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => continue,
                _ => continue,
            }
        }
    }
    None
}

fn path_binary_candidates(dir: &Path, binary: &str) -> Vec<PathBuf> {
    let mut out = Vec::with_capacity(2);
    #[cfg(windows)]
    {
        out.push(dir.join(format!("{binary}.exe")));
        out.push(dir.join(binary));
    }
    #[cfg(not(windows))]
    {
        out.push(dir.join(binary));
    }
    let _ = binary;
    out
}

/// Detect with injectable home root (PATH still read from env — scrub via TempEnv).
pub fn detect_all_with(home: Option<&Path>) -> Vec<HarnessPresence> {
    HARNESS_ORDER
        .iter()
        .copied()
        .map(|id| detect_one(id, home))
        .collect()
}

fn detect_one(id: HarnessId, home: Option<&Path>) -> HarnessPresence {
    let binary_path = resolve_on_path(id.binary_name());
    let home_path = home.and_then(|h| first_existing_home_root(h, id));
    let present = binary_path.is_some() || home_path.is_some();
    HarnessPresence {
        id,
        present,
        binary: binary_path.map(|p| p.display().to_string()),
        home_path: home_path.map(|p| p.display().to_string()),
    }
}

fn first_existing_home_root(home: &Path, id: HarnessId) -> Option<PathBuf> {
    for rel in id.home_rel_paths() {
        let p = join_rel(home, rel);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Join `home` with a `/`-separated relative path.
pub(crate) fn join_rel(home: &Path, rel: &str) -> PathBuf {
    let mut p = home.to_path_buf();
    for part in rel.split(['/', '\\']).filter(|s| !s.is_empty()) {
        p.push(part);
    }
    p
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;
    use tempfile::tempdir;

    #[test]
    fn harness_order__fixed_f1() {
        let ids: Vec<&str> = HARNESS_ORDER.iter().map(|h| h.as_str()).collect();
        assert_eq!(ids, vec!["grok", "agy", "opencode", "claude", "codex"]);
    }

    #[test]
    fn parse_harness_id__unknown__err() {
        let err = parse_harness_id("foo").expect_err("must reject");
        assert!(err.contains("unknown harness"), "{err}");
    }

    #[test]
    fn parse_harness_id__antigravity_alias__agy() {
        assert_eq!(parse_harness_id("antigravity").unwrap(), HarnessId::Agy);
        assert_eq!(parse_harness_id("agy").unwrap(), HarnessId::Agy);
    }

    #[test]
    fn detect__only_grok_home__present_true_others_false() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        std::fs::create_dir_all(home.join(".grok")).expect("mkdir .grok");
        let _up = TempEnv::set("USERPROFILE", home.as_os_str());
        let _home = TempEnv::set("HOME", home.as_os_str());
        let _path = TempEnv::set("PATH", "");

        let rows = detect_all_with(Some(home));
        assert_eq!(rows.len(), 5);
        let grok = rows.iter().find(|r| r.id == HarnessId::Grok).unwrap();
        assert!(grok.present, "grok home must make present=true");
        assert!(grok.home_path.is_some());
        for r in &rows {
            if r.id != HarnessId::Grok {
                assert!(!r.present, "{:?} should be absent", r.id);
            }
        }
    }

    #[test]
    fn detect__path_scrub__host_binary_ignored_when_path_empty() {
        // AC17: empty PATH + fixture home only → present follows fixture only.
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        // No harness homes.
        let _up = TempEnv::set("USERPROFILE", home.as_os_str());
        let _home = TempEnv::set("HOME", home.as_os_str());
        let _path = TempEnv::set("PATH", "");

        let rows = detect_all_with(Some(home));
        for r in &rows {
            assert!(
                !r.present,
                "{:?} must not be present with empty PATH and empty home: {:?}",
                r.id, r
            );
        }
    }

    #[test]
    fn detect__path_binary__present() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path().join("home");
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&bin).expect("bin");
        #[cfg(windows)]
        let exe = bin.join("grok.exe");
        #[cfg(not(windows))]
        let exe = bin.join("grok");
        std::fs::write(&exe, b"").expect("write binary stub");

        let _up = TempEnv::set("USERPROFILE", home.as_os_str());
        let _home = TempEnv::set("HOME", home.as_os_str());
        let _path = TempEnv::set("PATH", bin.as_os_str());

        let rows = detect_all_with(Some(&home));
        let grok = rows.iter().find(|r| r.id == HarnessId::Grok).unwrap();
        assert!(grok.present);
        assert!(grok.binary.is_some());
    }

    #[test]
    fn install_ready__agy_and_grok() {
        assert!(HarnessId::Agy.install_ready());
        assert!(HarnessId::Grok.install_ready());
        assert!(!HarnessId::Opencode.install_ready());
        assert!(!HarnessId::Claude.install_ready());
        assert!(!HarnessId::Codex.install_ready());
        assert!(HarnessId::Grok.pending_track().is_none());
        assert_eq!(HarnessId::Claude.pending_track(), Some("T238+"));
        assert_eq!(HarnessId::Codex.pending_track(), Some("T238+"));
    }
}
