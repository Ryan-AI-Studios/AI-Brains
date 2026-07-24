//! Path location comparison helpers for shadow-vault and backup safety.
//!
//! Uses best-effort symlink resolution and Windows drive/`\\?\` normalization
//! consistent with the rest of `ai-brains-path` (no third canonicalize stack).

use crate::symlink::resolve_best_effort;
use crate::unc::{is_unc_path, normalize_unc};
use crate::windows::{has_drive_prefix, normalize_drive_path, strip_extended_length_prefix};
use std::path::Path;

/// Normalize a path string for equality / containment checks.
///
/// Steps: best-effort resolve → strip `\\?\` → UNC or drive normalize.
/// Non-existing paths fall back to the input string (then still strip/normalize).
pub fn normalize_for_location_compare(input: &str) -> String {
    let resolved = resolve_best_effort(input);
    let stripped = strip_extended_length_prefix(&resolved).replace('/', "\\");

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
}
