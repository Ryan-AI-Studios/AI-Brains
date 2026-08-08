//! Consent / install prefs: `~/.ai-brains/harness_hooks.json` (F11, F28).

use super::detect::{HarnessId, join_rel};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const PREFS_SCHEMA_VERSION: u32 = 1;
pub const MANAGED_KEY: &str = "ai-brains-capture";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessPrefEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declined_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessHookPrefs {
    pub schema_version: u32,
    #[serde(default)]
    pub auto_install: bool,
    #[serde(default)]
    pub harnesses: BTreeMap<String, HarnessPrefEntry>,
}

impl Default for HarnessHookPrefs {
    fn default() -> Self {
        Self {
            schema_version: PREFS_SCHEMA_VERSION,
            auto_install: false,
            harnesses: BTreeMap::new(),
        }
    }
}

impl HarnessHookPrefs {
    pub fn entry(&self, id: HarnessId) -> Option<&HarnessPrefEntry> {
        self.harnesses.get(id.as_str())
    }

    pub fn entry_mut(&mut self, id: HarnessId) -> &mut HarnessPrefEntry {
        self.harnesses.entry(id.as_str().to_string()).or_default()
    }

    pub fn is_declined(&self, id: HarnessId) -> bool {
        self.entry(id)
            .and_then(|e| e.declined_at.as_ref())
            .is_some()
    }

    pub fn mark_declined(&mut self, id: HarnessId, at: impl Into<String>) {
        let e = self.entry_mut(id);
        e.declined_at = Some(at.into());
    }

    pub fn clear_declined(&mut self, id: HarnessId) {
        if let Some(e) = self.harnesses.get_mut(id.as_str()) {
            e.declined_at = None;
        }
    }

    pub fn mark_installed(
        &mut self,
        id: HarnessId,
        at: impl Into<String>,
        version: impl Into<String>,
    ) {
        let e = self.entry_mut(id);
        e.installed_at = Some(at.into());
        e.install_version = Some(version.into());
        e.last_status = Some("installed".to_string());
        e.declined_at = None;
    }

    /// Uninstall clears installed_at / last_status (F11 / AC23).
    pub fn mark_uninstalled(&mut self, id: HarnessId) {
        let e = self.entry_mut(id);
        e.installed_at = None;
        e.install_version = None;
        e.last_status = Some("uninstalled".to_string());
    }

    /// True when user requested install but backend is not ready (F5).
    pub fn is_backend_pending_requested(&self, id: HarnessId) -> bool {
        self.entry(id)
            .and_then(|e| e.last_status.as_deref())
            .is_some_and(|s| s == "backend_pending")
    }

    /// Stamp last_status for a pending-backend install request (F5 / F14).
    pub fn mark_backend_pending(&mut self, id: HarnessId) {
        let e = self.entry_mut(id);
        e.last_status = Some("backend_pending".to_string());
    }
}

pub fn prefs_path(home: &Path) -> PathBuf {
    join_rel(home, ".ai-brains/harness_hooks.json")
}

/// Load prefs. Corrupt / missing → empty defaults for in-memory decisions (F28).
///
/// Does **not** rewrite the on-disk file. Callers that need to persist must use
/// [`save_prefs`], which **refuses** to overwrite a corrupt prefs file.
pub fn load_prefs(home: &Path) -> HarnessHookPrefs {
    let path = prefs_path(home);
    match std::fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str::<HarnessHookPrefs>(&raw).unwrap_or_default(),
        Err(_) => HarnessHookPrefs::default(),
    }
}

/// Atomic save of prefs (temp + rename; reparse refuse).
///
/// F28: if `harness_hooks.json` exists and is corrupt, **refuse rewrite** (exit path
/// returns Err with path + reason). Missing file is OK to create.
pub fn save_prefs(home: &Path, prefs: &HarnessHookPrefs) -> Result<(), String> {
    let path = prefs_path(home);
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(raw) => {
                if serde_json::from_str::<HarnessHookPrefs>(&raw).is_err() {
                    return Err(format!(
                        "refuse rewrite corrupt harness prefs at {} — fix or remove the file, then re-run",
                        path.display()
                    ));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(format!(
                    "cannot read harness prefs at {} before write: {e}",
                    path.display()
                ));
            }
        }
    }
    let body =
        serde_json::to_string_pretty(prefs).map_err(|e| format!("serialize harness prefs: {e}"))?;
    super::fs_util::atomic_write_str(&path, &body)
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_prefs__missing__default() {
        let dir = tempdir().expect("tempdir");
        let p = load_prefs(dir.path());
        assert_eq!(p.schema_version, PREFS_SCHEMA_VERSION);
        assert!(!p.auto_install);
        assert!(p.harnesses.is_empty());
    }

    #[test]
    fn load_prefs__corrupt__default_no_rewrite() {
        let dir = tempdir().expect("tempdir");
        let path = prefs_path(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
        let original = b"NOT JSON {{{";
        std::fs::write(&path, original).expect("write");
        let p = load_prefs(dir.path());
        assert!(p.harnesses.is_empty());
        // File bytes unchanged (never destructive rewrite).
        let after = std::fs::read(&path).expect("read");
        assert_eq!(after, original);
    }

    #[test]
    fn mark_uninstalled__clears_installed_at() {
        let mut p = HarnessHookPrefs::default();
        p.mark_installed(HarnessId::Agy, "2026-01-01T00:00:00Z", "0.1.0");
        assert!(p.entry(HarnessId::Agy).unwrap().installed_at.is_some());
        p.mark_uninstalled(HarnessId::Agy);
        let e = p.entry(HarnessId::Agy).unwrap();
        assert!(e.installed_at.is_none());
        assert_eq!(e.last_status.as_deref(), Some("uninstalled"));
    }

    #[test]
    fn decline_and_reset() {
        let mut p = HarnessHookPrefs::default();
        p.mark_declined(HarnessId::Agy, "t1");
        assert!(p.is_declined(HarnessId::Agy));
        p.clear_declined(HarnessId::Agy);
        assert!(!p.is_declined(HarnessId::Agy));
    }

    #[test]
    fn save_prefs__corrupt_existing__refuse_no_rewrite() {
        // F28: never destructive rewrite of corrupt prefs.
        let dir = tempdir().expect("tempdir");
        let path = prefs_path(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
        let original = b"NOT JSON {{{";
        std::fs::write(&path, original).expect("write");
        let mut p = HarnessHookPrefs::default();
        p.mark_declined(HarnessId::Agy, "t");
        let err = save_prefs(dir.path(), &p).expect_err("must refuse");
        assert!(err.contains("refuse rewrite corrupt"), "{err}");
        let after = std::fs::read(&path).expect("read");
        assert_eq!(after, original);
    }
}
