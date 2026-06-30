use std::path::{Path, PathBuf};

/// Searches for a ledgerful state directory starting from the given path and walking up the tree.
/// Looks for `.ledgerful/` first, then `.git/.ledgerful/`, then falls back to legacy `.changeguard/`
/// and `.git/.changeguard/` for backward compatibility.
///
/// The upward walk stops at the first directory that contains a `.git/` folder
/// (i.e. the repository/project boundary) if no state directory was found at or
/// below that level. This keeps discovery project-local and avoids picking up an
/// unrelated user-home state directory.
pub fn find_ledgerful_dir(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path.to_path_buf();
    loop {
        for candidate in [
            current.join(".ledgerful"),
            current.join(".git").join(".ledgerful"),
            current.join(".changeguard"),
            current.join(".git").join(".changeguard"),
        ] {
            if candidate.is_dir() {
                return Some(candidate);
            }
        }

        // Stop at the repository boundary so a test/project temp dir does not
        // accidentally discover a global state directory in a parent path.
        if current.join(".git").is_dir() {
            return None;
        }

        if !current.pop() {
            return None;
        }
    }
}

/// Attempts to extract a project ID from a ledgerful state directory.
/// Looks for a `project_id` file inside the given directory and returns its trimmed contents.
pub fn extract_project_id_from_ledgerful(ledgerful_dir: &Path) -> Option<String> {
    let id_file = ledgerful_dir.join("project_id");
    if id_file.exists() {
        if let Ok(content) = std::fs::read_to_string(id_file) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

/// Deprecated: use [`find_ledgerful_dir`] instead.
#[deprecated(note = "use find_ledgerful_dir instead")]
pub fn find_changeguard_dir(start_path: &Path) -> Option<PathBuf> {
    find_ledgerful_dir(start_path)
}

/// Deprecated: use [`extract_project_id_from_ledgerful`] instead.
#[deprecated(note = "use extract_project_id_from_ledgerful instead")]
pub fn extract_project_id_from_changeguard(changeguard_dir: &Path) -> Option<String> {
    extract_project_id_from_ledgerful(changeguard_dir)
}
