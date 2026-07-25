#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_git::{DiffStat, GitMetadata};
use ai_brains_sources::{
    NORMALIZER_VERSION, Sha256Fingerprinter, canonicalize_git_metadata, fingerprint_bytes,
    fingerprint_external, fingerprint_file_with_identity, fingerprint_git_metadata,
    fingerprint_git_path, fingerprint_ledgerful, normalize_file_bytes, normalize_utf8_text,
};
use std::process::Command;
use tempfile::tempdir;

const ID: &str = "Personal:test|File|/doc.md";

#[test]
fn identical_file_bytes__identical_fingerprint() {
    let fp = Sha256Fingerprinter::new();
    let a = fp.fingerprint_file(ID, b"hello world\n").unwrap();
    let b = fp.fingerprint_file(ID, b"hello world\n").unwrap();
    assert_eq!(a, b);
    assert!(a.starts_with(&format!("v{NORMALIZER_VERSION}:")));
}

#[test]
fn same_bytes_different_identity__different_fingerprint() {
    let body = b"shared content\n";
    let a = fingerprint_file_with_identity("id-a|File|/a.md", body).unwrap();
    let b = fingerprint_file_with_identity("id-b|File|/b.md", body).unwrap();
    assert_ne!(a, b, "spec §3.3: identity must fold into file fingerprint");
}

#[test]
fn normalizer_version_in_prefix__same_hex_shape() {
    let fp = fingerprint_bytes(b"content");
    let prefix = format!("v{NORMALIZER_VERSION}:");
    assert!(fp.starts_with(&prefix));
    // Documented format: bumping NORMALIZER_VERSION changes the identity tuple.
    // Simulate a version bump by replacing the prefix.
    let hex = &fp[prefix.len()..];
    let bumped = format!("v{}:{hex}", NORMALIZER_VERSION + 1);
    assert_ne!(fp, bumped);
    assert!(bumped.starts_with(&format!("v{}:", NORMALIZER_VERSION + 1)));
}

#[test]
fn file_normalization__bom_and_crlf_do_not_change_fingerprint() {
    let fp = Sha256Fingerprinter::new();
    let plain = fp.fingerprint_file(ID, b"line1\nline2\n").unwrap();
    let bom_crlf = {
        let mut bytes = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        bytes.extend_from_slice(b"line1\r\nline2\r\n");
        fp.fingerprint_file(ID, &bytes).unwrap()
    };
    assert_eq!(plain, bom_crlf);
    assert_eq!(
        normalize_utf8_text("\u{FEFF}line1\r\nline2\r\n"),
        "line1\nline2\n"
    );
    assert_eq!(
        normalize_file_bytes(b"\xEF\xBB\xBFline1\r\n").unwrap(),
        b"line1\n".to_vec()
    );
}

#[test]
fn empty_content__stable_fingerprint() {
    let a = fingerprint_bytes(b"");
    let b = fingerprint_bytes(&[]);
    assert_eq!(a, b);
}

#[test]
fn unicode_content__stable() {
    let fp = Sha256Fingerprinter::new();
    let text = "café 日本語 🚀\n".as_bytes();
    assert_eq!(
        fp.fingerprint_file(ID, text).unwrap(),
        fp.fingerprint_file(ID, text).unwrap()
    );
}

#[test]
fn ledgerful__bridge_hash_preferred() {
    let json = br#"{"bridge_hash":"ff00aa","extra":1}"#;
    assert_eq!(
        fingerprint_ledgerful("any", json).unwrap(),
        "ledgerful:ff00aa"
    );
}

#[test]
fn ledgerful__fallback_folds_identity() {
    let a = fingerprint_ledgerful("L1", b"no hash here\n").unwrap();
    let b = fingerprint_ledgerful("L2", b"no hash here\n").unwrap();
    assert_ne!(a, b);
}

#[test]
fn external__etag_and_revision_and_payload() {
    assert_eq!(
        fingerprint_external("e", b"etag:v9").unwrap(),
        format!("v{NORMALIZER_VERSION}:etag:v9")
    );
    assert_eq!(
        fingerprint_external("e", b"rev:3").unwrap(),
        format!("v{NORMALIZER_VERSION}:revision:3")
    );
    let p1 = fingerprint_external("e1", b"body\n").unwrap();
    let p2 = fingerprint_external("e2", b"body\n").unwrap();
    assert_ne!(p1, p2);
}

#[test]
fn external__reordered_json_object__same_fingerprint() {
    let a = fingerprint_external("ext", br#"{"a":1,"b":2}"#).unwrap();
    let b = fingerprint_external("ext", br#"{"b":2,"a":1}"#).unwrap();
    assert_eq!(
        a, b,
        "External JSON payloads must canonicalize object key order"
    );
}

#[test]
fn git_metadata_canonicalization__same_fields_same_digest() {
    let meta = GitMetadata {
        root: Some(std::path::PathBuf::from("/repo")),
        branch: Some("main".into()),
        commit: Some("deadbeef".into()),
        remote_url_hash: Some("abc123".into()),
        remote_names: Vec::new(),
        is_dirty: false,
        untracked_files: vec!["b.rs".into(), "a.rs".into()],
        diffstat: Some(DiffStat {
            files_changed: 2,
            insertions: 10,
            deletions: 1,
            summary: "2 files changed, 10 insertions(+), 1 deletion(-)".into(),
        }),
        common_dir: None,
    };
    let once = fingerprint_git_metadata(&meta);
    let twice = fingerprint_git_metadata(&meta);
    assert_eq!(once, twice);

    let mut reordered = meta.clone();
    reordered.untracked_files = vec!["a.rs".into(), "b.rs".into()];
    assert_eq!(
        canonicalize_git_metadata(&meta),
        canonicalize_git_metadata(&reordered)
    );
    assert_eq!(once, fingerprint_git_metadata(&reordered));
}

#[test]
fn git_metadata__toggling_dirty_untracked_commit_changes_digest() {
    let base = GitMetadata {
        commit: Some("c1".into()),
        is_dirty: false,
        untracked_files: vec![],
        ..GitMetadata::default()
    };
    let dirty = GitMetadata {
        is_dirty: true,
        ..base.clone()
    };
    let with_untracked = GitMetadata {
        untracked_files: vec!["x".into()],
        ..base.clone()
    };
    let other_commit = GitMetadata {
        commit: Some("c2".into()),
        ..base.clone()
    };

    let base_fp = fingerprint_git_metadata(&base);
    assert_ne!(base_fp, fingerprint_git_metadata(&dirty));
    assert_ne!(base_fp, fingerprint_git_metadata(&with_untracked));
    assert_ne!(base_fp, fingerprint_git_metadata(&other_commit));
}

#[test]
fn git_path__collect_metadata_integration() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    run_git(root, &["init"]);
    run_git(root, &["config", "user.email", "test@example.com"]);
    run_git(root, &["config", "user.name", "Test"]);
    std::fs::write(root.join("README.md"), "hello\n").unwrap();
    run_git(root, &["add", "README.md"]);
    run_git(root, &["commit", "-m", "initial"]);

    let fp1 = fingerprint_git_path(root).expect("fingerprint path");
    let fp2 = fingerprint_git_path(root).expect("fingerprint path again");
    assert_eq!(fp1, fp2);
    assert!(fp1.starts_with(&format!("v{NORMALIZER_VERSION}:")));

    // Mutate working tree → dirty metadata → different fingerprint.
    std::fs::write(root.join("README.md"), "hello changed\n").unwrap();
    let fp_dirty = fingerprint_git_path(root).expect("fingerprint dirty");
    assert_ne!(fp1, fp_dirty);
}

fn run_git(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("spawn git");
    assert!(status.success(), "git {args:?} failed");
}
