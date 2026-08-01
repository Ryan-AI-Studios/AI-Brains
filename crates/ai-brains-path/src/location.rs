//! Path location comparison helpers for shadow-vault and backup safety.
//!
//! Uses best-effort symlink resolution and Windows drive/`\\?\` normalization
//! consistent with the rest of `ai-brains-path` (no third canonicalize stack).

use crate::symlink::resolve_best_effort;
use crate::unc::{is_unc_path, normalize_unc};
use crate::windows::{has_drive_prefix, normalize_drive_path, strip_extended_length_prefix};
use std::path::Path;

/// Map WSL `/mnt/<drive>/...` (or `\mnt\<drive>\...`) to a Windows drive path.
///
/// Example: `/mnt/c/Dev/Project` → `C:\Dev\Project`.
fn map_wsl_mnt_to_drive(path: &str) -> Option<String> {
    let slash_normalized = path.replace('/', "\\");
    let lower = slash_normalized.to_ascii_lowercase();
    let rest = lower.strip_prefix(r"\mnt\")?;
    let (drive, remainder) = match rest.split_once('\\') {
        Some((d, r)) => (d, r),
        None => (rest, ""),
    };
    if drive.len() != 1 {
        return None;
    }
    let drive_letter = drive.chars().next()?;
    if !drive_letter.is_ascii_alphabetic() {
        return None;
    }
    // Preserve remainder casing from the slash-normalized form after the prefix.
    // Prefix length in chars: "\mnt\" (5) + drive (1) + optional "\".
    let prefix_len = 5 + drive.len();
    let remainder_original = if slash_normalized.len() > prefix_len {
        let after = &slash_normalized[prefix_len..];
        after.strip_prefix('\\').unwrap_or(after)
    } else {
        remainder
    };
    let letter = drive_letter.to_ascii_uppercase();
    if remainder_original.is_empty() {
        Some(format!("{letter}:\\"))
    } else {
        Some(format!("{letter}:\\{remainder_original}"))
    }
}

/// Normalize a path string for equality / containment checks.
///
/// Steps: **WSL `/mnt/c` map first** → best-effort resolve → strip `\\?\` →
/// UNC or drive normalize. Non-existing paths fall back to the input string
/// (then still strip/normalize).
///
/// WSL must run **before** soft-resolve: on Windows, a bare `/mnt/c/...` path is
/// treated as drive-relative (`\mnt\c\...` under the current volume, e.g.
/// `D:\mnt\c\...` on GHA). Mapping after soft-resolve misses that form (T179
/// Phase F / gate-windows `path_locator_wsl_and_windows`).
pub fn normalize_for_location_compare(input: &str) -> String {
    // Map WSL before resolve so soft-canonicalize never binds /mnt/c to CWD drive.
    let pre = if let Some(mapped) = map_wsl_mnt_to_drive(input) {
        mapped
    } else {
        input.to_string()
    };

    let resolved = resolve_best_effort(&pre);
    let mut stripped = strip_extended_length_prefix(&resolved).replace('/', "\\");

    // Second pass: if soft-resolve already joined a mistaken `\mnt\...` under a
    // drive (legacy callers / intermediate forms), still map when possible.
    if let Some(mapped) = map_wsl_mnt_to_drive(&stripped) {
        stripped = mapped.replace('/', "\\");
    } else if let Some((_, rest)) = stripped.split_once(r":\mnt\") {
        // e.g. D:\mnt\c\Dev\... from pre-fix soft-resolve of /mnt/c/...
        if let Some(mapped) = map_wsl_mnt_to_drive(&format!(r"\mnt\{rest}")) {
            stripped = mapped.replace('/', "\\");
        }
    } else if let Some((_, rest)) = stripped.split_once(r":/mnt/") {
        if let Some(mapped) = map_wsl_mnt_to_drive(&format!("/mnt/{rest}")) {
            stripped = mapped.replace('/', "\\");
        }
    }

    if is_unc_path(&stripped) {
        return normalize_unc(&stripped);
    }

    if has_drive_prefix(&stripped) {
        return normalize_drive_path(&stripped).unwrap_or_else(|_| stripped.to_ascii_lowercase());
    }

    // Relative or non-drive absolute: lowercase for case-insensitive compare on Windows.
    #[cfg(windows)]
    {
        stripped.to_ascii_lowercase()
    }
    #[cfg(not(windows))]
    {
        stripped
    }
}

/// True when `a` and `b` refer to the same filesystem location under
/// best-effort resolve + `\\?\` strip + drive normalization.
pub fn paths_refer_to_same_location(a: impl AsRef<Path>, b: impl AsRef<Path>) -> bool {
    let a_s = a.as_ref().to_string_lossy();
    let b_s = b.as_ref().to_string_lossy();
    normalize_for_location_compare(&a_s) == normalize_for_location_compare(&b_s)
}

/// True when `candidate` is the same path as `root`, or is strictly inside `root`.
///
/// Containment uses normalized string prefixes with a path separator boundary
/// (so `C:\foo` is not inside `C:\foobar`).
pub fn path_is_same_or_inside(candidate: impl AsRef<Path>, root: impl AsRef<Path>) -> bool {
    let cand = normalize_for_location_compare(&candidate.as_ref().to_string_lossy());
    let root_n = normalize_for_location_compare(&root.as_ref().to_string_lossy());

    if cand == root_n {
        return true;
    }

    let root_with_sep = if root_n.ends_with('\\') || root_n.ends_with('/') {
        root_n
    } else {
        format!("{root_n}\\")
    };

    cand.starts_with(&root_with_sep)
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn paths_refer_to_same_location__extended_length_prefix__equal() {
        let a = r"\\?\C:\Dev\Project";
        let b = r"C:\Dev\Project";
        // Non-existing: resolve_best_effort returns input; normalize still strips prefix.
        assert!(paths_refer_to_same_location(a, b));
    }

    #[test]
    fn paths_refer_to_same_location__drive_case__equal() {
        assert!(paths_refer_to_same_location(r"C:\Dev\Foo", r"c:\dev\foo"));
    }

    #[test]
    fn paths_refer_to_same_location__different_paths__false() {
        assert!(!paths_refer_to_same_location(r"C:\Dev\A", r"C:\Dev\B"));
    }

    #[test]
    fn path_is_same_or_inside__same_path__true() {
        assert!(path_is_same_or_inside(
            r"C:\vaults\live.db",
            r"C:\vaults\live.db"
        ));
    }

    #[test]
    fn path_is_same_or_inside__child_of_root__true() {
        assert!(path_is_same_or_inside(
            r"C:\Users\me\.ai-brains\shadow\vault.db",
            r"C:\Users\me\.ai-brains"
        ));
    }

    #[test]
    fn path_is_same_or_inside__sibling_prefix__false() {
        // C:\foo is not inside C:\foobar
        assert!(!path_is_same_or_inside(r"C:\foobar\x", r"C:\foo"));
    }

    #[test]
    fn path_is_same_or_inside__outside__false() {
        assert!(!path_is_same_or_inside(
            r"C:\other\vault.db",
            r"C:\Users\me\.ai-brains"
        ));
    }

    #[test]
    fn normalize_for_location_compare__wsl_mnt_c__equals_windows_drive() {
        // Must not depend on CWD (GHA Windows often runs on D:).
        let wsl = normalize_for_location_compare("/mnt/c/Dev/Project/readme.md");
        let win = normalize_for_location_compare(r"C:\Dev\Project\readme.md");
        assert_eq!(wsl, win, "WSL and Windows forms must normalize equal");
        assert!(
            wsl.starts_with(r"c:\") || wsl.starts_with(r"C:\"),
            "expected C: drive after WSL map, got {wsl}"
        );
        assert!(
            !wsl.to_ascii_lowercase().contains(r"\mnt\c\"),
            "must not leave residual \\mnt\\c under another drive: {wsl}"
        );
    }

    #[test]
    fn normalize_for_location_compare__mistaken_drive_mnt_c__still_maps() {
        // Pre-fix soft-resolve artifact: current-drive + \mnt\c\...
        let mistaken = normalize_for_location_compare(r"D:\mnt\c\Dev\Project\readme.md");
        let win = normalize_for_location_compare(r"C:\Dev\Project\readme.md");
        assert_eq!(mistaken, win);
    }

    #[test]
    fn normalize_for_location_compare__non_existing__strips_prefix() {
        let missing = r"\\?\C:\definitely\does\not\exist\xyz-t147";
        let norm = normalize_for_location_compare(missing);
        assert!(!norm.starts_with(r"\\?\"));
        assert!(
            norm.to_ascii_lowercase()
                .contains(r"c:\definitely\does\not\exist\xyz-t147")
        );
    }

    #[test]
    fn resolve_best_effort__non_existing__returns_input() {
        let missing = r"C:\this\path\should\not\exist\t147-path-test";
        let resolved = crate::resolve_best_effort(missing);
        assert_eq!(resolved, missing);
    }

    #[test]
    fn paths_refer_to_same_location__pathbuf_overloads() {
        let a = PathBuf::from(r"C:\Dev\X");
        let b = PathBuf::from(r"c:\dev\x");
        assert!(paths_refer_to_same_location(&a, &b));
    }

    /// Soft-resolve: not-yet-created child under an existing temp parent still
    /// compares as inside that parent (macOS /var vs /private/var honesty).
    #[test]
    fn path_is_same_or_inside__missing_child_under_existing_parent__true() {
        let dir = tempfile::tempdir().expect("tempdir");
        let parent = dir.path().join("live-home");
        std::fs::create_dir_all(&parent).expect("parent");
        let missing_child = parent.join("migrate-sibling.db");
        assert!(!missing_child.exists(), "fixture child must not exist yet");
        assert!(
            path_is_same_or_inside(&missing_child, &parent),
            "missing child under existing parent must still be inside"
        );
    }
}
