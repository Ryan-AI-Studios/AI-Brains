use crate::unc;
use crate::windows;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Best-effort absolute/symlink resolution for path equality and containment.
///
/// 1. If `path` exists → `canonicalize` (resolves macOS `/var` → `/private/var`).
/// 2. Else soft-canonicalize: resolve the longest existing ancestor, then rejoin
///    the missing suffix. Prevents false negatives when comparing an existing
///    parent dir against a not-yet-created child (common on macOS CI tempdirs).
pub fn resolve_best_effort(input: &str) -> String {
    let path = Path::new(input);
    match path.canonicalize() {
        Ok(resolved) => finish_resolved(&resolved.to_string_lossy()),
        Err(_) => soft_resolve(path).unwrap_or_else(|| input.to_string()),
    }
}

fn finish_resolved(resolved_str: &str) -> String {
    let stripped = windows::strip_extended_length_prefix(resolved_str);
    if unc::is_unc_path(&stripped) {
        unc::normalize_unc(&stripped)
    } else {
        windows::normalize_drive_path(&stripped).unwrap_or(stripped)
    }
}

/// Canonicalize longest existing ancestor; rejoin remaining components (owned).
fn soft_resolve(path: &Path) -> Option<String> {
    let mut remainder: Vec<OsString> = Vec::new();
    let mut cursor = path.to_path_buf();

    loop {
        if let Ok(canon) = cursor.canonicalize() {
            let mut out = PathBuf::from(finish_resolved(&canon.to_string_lossy()));
            for part in remainder.iter().rev() {
                out.push(part);
            }
            return Some(out.to_string_lossy().into_owned());
        }
        let file_name = cursor.file_name()?.to_os_string();
        remainder.push(file_name);
        if !cursor.pop() {
            return None;
        }
    }
}
