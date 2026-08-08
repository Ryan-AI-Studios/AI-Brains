//! Atomic write helpers for harness prefs / hooks (F36).

use std::fs;
use std::io::Write;
use std::path::Path;

/// Atomic write: temp file + rename; refuse reparse/symlink (F36).
pub fn atomic_write_str(path: &Path, contents: &str) -> Result<(), String> {
    match ai_brains_path::is_reparse_or_symlink(path) {
        Ok(true) => {
            return Err(format!(
                "refusing write through reparse/symlink at {}",
                path.display()
            ));
        }
        Ok(false) => {}
        Err(e) => {
            return Err(format!("reparse check failed for {}: {e}", path.display()));
        }
    }
    if let Some(parent) = path.parent() {
        match ai_brains_path::is_reparse_or_symlink(parent) {
            Ok(true) => {
                return Err(format!(
                    "refusing write through reparse/symlink parent at {}",
                    parent.display()
                ));
            }
            Ok(false) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(format!(
                    "reparse check failed for parent {}: {e}",
                    parent.display()
                ));
            }
        }
        fs::create_dir_all(parent)
            .map_err(|e| format!("create_dir_all {}: {e}", parent.display()))?;
        if matches!(ai_brains_path::is_reparse_or_symlink(parent), Ok(true)) {
            return Err(format!(
                "refusing write through reparse/symlink parent at {}",
                parent.display()
            ));
        }
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp_name = format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("write"),
        uuid::Uuid::new_v4()
    );
    let tmp_path = parent.join(tmp_name);
    {
        let mut f = fs::File::create(&tmp_path)
            .map_err(|e| format!("create temp {}: {e}", tmp_path.display()))?;
        f.write_all(contents.as_bytes())
            .map_err(|e| format!("write temp {}: {e}", tmp_path.display()))?;
        f.sync_all()
            .map_err(|e| format!("sync temp {}: {e}", tmp_path.display()))?;
    }
    fs::rename(&tmp_path, path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        format!("rename {} -> {}: {e}", tmp_path.display(), path.display())
    })?;
    Ok(())
}
