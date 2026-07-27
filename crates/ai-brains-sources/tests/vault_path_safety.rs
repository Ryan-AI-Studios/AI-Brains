//! Vault path-safety unit-style integration tests (T154 Phase A).

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use std::path::Path;

use ai_brains_sources::{VaultFsError, is_reserved_windows_stem, resolve_under_root};
use tempfile::tempdir;

#[test]
fn resolve_under_root__parent_escape__errors() {
    let dir = tempdir().expect("tempdir");
    let err = resolve_under_root(dir.path(), "notes/../../outside.md").expect_err("parent escape");
    assert!(matches!(err, VaultFsError::PathEscape(_)), "{err}");
}

#[test]
fn resolve_under_root__absolute_outside__errors() {
    let dir = tempdir().expect("tempdir");
    let err = resolve_under_root(dir.path(), r"C:\Windows\System32\drivers\etc\hosts")
        .expect_err("absolute");
    assert!(matches!(err, VaultFsError::AbsolutePath(_)), "{err}");
}

#[test]
fn resolve_under_root__normalized_relative__ok() {
    let dir = tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("notes")).expect("mkdir");
    let resolved = resolve_under_root(dir.path(), "notes/alpha.md").expect("ok");
    assert!(ai_brains_path::path_is_same_or_inside(
        &resolved,
        dir.path()
    ));
    assert!(resolved.ends_with(Path::new("notes").join("alpha.md")));
}

#[test]
fn reserved_stem__aux_md__refused() {
    assert!(is_reserved_windows_stem("aux.md"));
    let dir = tempdir().expect("tempdir");
    let err = resolve_under_root(dir.path(), "aux.md").expect_err("aux");
    assert!(matches!(err, VaultFsError::ReservedStem(_)), "{err}");
}

#[test]
fn reserved_stem__nul_md__refused() {
    assert!(is_reserved_windows_stem("nul.md"));
}

#[test]
fn reserved_stem__con_uppercase__refused() {
    assert!(is_reserved_windows_stem("CON"));
    assert!(is_reserved_windows_stem("Con.MD"));
}

#[test]
fn reserved_stem__com1_meeting_notes_md__allowed() {
    assert!(!is_reserved_windows_stem("com1-meeting-notes.md"));
    let dir = tempdir().expect("tempdir");
    resolve_under_root(dir.path(), "notes/com1-meeting-notes.md").expect("allowed");
}

#[test]
fn reserved_stem__notes_con_md__refused() {
    assert!(is_reserved_windows_stem("con.md"));
    let dir = tempdir().expect("tempdir");
    let err = resolve_under_root(dir.path(), "notes/con.md").expect_err("con");
    assert!(matches!(err, VaultFsError::ReservedStem(_)), "{err}");
}
