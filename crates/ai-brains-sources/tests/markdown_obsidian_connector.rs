//! Markdown / Obsidian connector integration tests (T154).

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use ai_brains_core::ids::UserId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_sources::{
    Connector, ConnectorContext, ConnectorError, ConnectorTrustLabel, InProcessConnectorRegistry,
    MANIFEST_SCHEMA_VERSION, MarkdownObsidianConnector, OBSIDIAN_CONNECTOR_ID, SandboxMode,
    VaultOptions, WriteProposalInput, fingerprint_file_with_identity, validate_manifest,
};
use tempfile::tempdir;
use uuid::Uuid;

fn personal_ctx() -> ConnectorContext {
    ConnectorContext {
        principal_id: None,
        scope: ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(42))),
        privacy: Privacy::LocalOnly,
        trust: ConnectorTrustLabel::LocalOnly,
    }
}

/// Build a synthetic Obsidian-style vault under `root`.
fn write_fixture_vault(root: &Path) {
    fs::create_dir_all(root.join(".obsidian")).expect("obsidian dir");
    fs::write(root.join(".obsidian/app.json"), b"{}").expect("app.json");
    fs::create_dir_all(root.join("notes")).expect("notes");
    fs::write(
        root.join("notes/alpha.md"),
        b"---\ntitle: Alpha\n---\n# Alpha body\nshared content\n",
    )
    .expect("alpha");
    fs::write(
        root.join("notes/gamma.md"),
        b"---\ntitle: Shared Title\n---\n# Shared Title\ngamma body\n",
    )
    .expect("gamma");
    fs::write(
        root.join("notes/dup.md"),
        b"# Shared Title\nduplicate title different path\n",
    )
    .expect("dup");
    fs::write(root.join("readme.txt"), b"not markdown").expect("txt");
    fs::create_dir_all(root.join(".trash")).expect("trash");
    fs::write(root.join(".trash/gone.md"), b"should not list").expect("trash md");
}

fn open_vault(root: &Path) -> MarkdownObsidianConnector {
    MarkdownObsidianConnector::open(root, VaultOptions::default()).expect("open")
}

fn open_vault_opts(root: &Path, opts: VaultOptions) -> MarkdownObsidianConnector {
    MarkdownObsidianConnector::open(root, opts).expect("open")
}

/// Shared contract assertions (mirrors connector_contract suite).
fn assert_connector_contract(c: &dyn Connector) {
    let m = c.manifest();
    assert_eq!(m.schema_version, MANIFEST_SCHEMA_VERSION);
    validate_manifest(m).expect("manifest must validate");
    assert!(!m.source_kinds.is_empty());
    assert_eq!(m.sandbox, SandboxMode::TrustedBuiltin);

    let ctx = personal_ctx();
    let ops = m.operations;

    match c.list(&ctx) {
        Ok(handles) => {
            assert!(ops.list);
            for h in &handles {
                assert!(
                    m.source_kinds.iter().any(|k| k == &h.kind),
                    "list returned undeclared kind {:?}",
                    h.kind
                );
            }
        }
        Err(ConnectorError::OperationNotSupported { operation }) => {
            assert!(!ops.list);
            assert_eq!(operation, "list");
        }
        Err(e) => panic!("unexpected list error: {e}"),
    }

    let handles = if ops.list {
        c.list(&ctx).expect("list")
    } else {
        Vec::new()
    };

    if let Some(handle) = handles.first() {
        match c.observe(&ctx, handle) {
            Ok(payload) => {
                assert!(ops.observe);
                assert!(!payload.identity.is_empty());
            }
            Err(ConnectorError::OperationNotSupported { operation }) => {
                assert!(!ops.observe);
                assert_eq!(operation, "observe");
            }
            Err(e) => panic!("unexpected observe error: {e}"),
        }

        match c.preview(&ctx, handle) {
            Ok(_) => assert!(ops.preview),
            Err(ConnectorError::OperationNotSupported { operation }) => {
                assert!(!ops.preview);
                assert_eq!(operation, "preview");
            }
            Err(e) => panic!("unexpected preview error: {e}"),
        }

        let input = WriteProposalInput {
            handle: handle.clone(),
            proposed_content: "proposed".into(),
            rationale: Some("contract".into()),
        };
        match c.propose_write(&ctx, &input) {
            Ok(proposal) => {
                assert!(ops.propose_write);
                assert!(!proposal.artifact_id.is_empty());
            }
            Err(ConnectorError::OperationNotSupported { operation }) => {
                assert!(!ops.propose_write);
                assert_eq!(operation, "propose_write");
            }
            Err(e) => panic!("unexpected propose_write error: {e}"),
        }
    }
}

fn count_files(root: &Path) -> usize {
    let mut n = 0;
    if root.is_file() {
        return 1;
    }
    let Ok(rd) = fs::read_dir(root) else {
        return 0;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            n += count_files(&p);
        } else {
            n += 1;
        }
    }
    n
}

fn dir_contains_bytes(root: &Path, needle: &[u8]) -> bool {
    if root.is_file() {
        return fs::read(root)
            .map(|b| b.windows(needle.len()).any(|w| w == needle))
            .unwrap_or(false);
    }
    let Ok(rd) = fs::read_dir(root) else {
        return false;
    };
    for entry in rd.flatten() {
        if dir_contains_bytes(&entry.path(), needle) {
            return true;
        }
    }
    false
}

#[test]
fn obsidian_connector__list__skips_dot_obsidian() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let c = open_vault(dir.path());
    let ctx = personal_ctx();
    let handles = c.list(&ctx).expect("list");
    assert!(
        handles
            .iter()
            .all(|h| !h.locator.contains(".obsidian") && !h.locator.contains(".trash")),
        "handles: {:?}",
        handles.iter().map(|h| &h.locator).collect::<Vec<_>>()
    );
    // Vault root handle has empty locator; no note path under .obsidian.
    assert!(
        !handles
            .iter()
            .any(|h| h.locator.contains("app.json")),
        "config must not be listed"
    );
}

#[test]
fn obsidian_connector__list__md_notes_only() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let c = open_vault(dir.path());
    let ctx = personal_ctx();
    let handles = c.list(&ctx).expect("list");
    let file_handles: Vec<_> = handles
        .iter()
        .filter(|h| h.kind == SourceKind::File)
        .collect();
    assert!(file_handles.iter().all(|h| {
        let lower = h.locator.to_ascii_lowercase();
        lower.ends_with(".md") || lower.ends_with(".markdown")
    }));
    assert!(
        !file_handles.iter().any(|h| h.locator.ends_with(".txt")),
        "txt must not list"
    );
    let locs: Vec<&str> = file_handles.iter().map(|h| h.locator.as_str()).collect();
    assert!(locs.contains(&"notes/alpha.md"));
    assert!(locs.contains(&"notes/gamma.md"));
    assert!(locs.contains(&"notes/dup.md"));
}

#[test]
fn obsidian_connector__observe__fingerprintable_payload() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let c = open_vault(dir.path());
    let ctx = personal_ctx();
    let handles = c.list(&ctx).expect("list");
    let note = handles
        .iter()
        .find(|h| h.locator == "notes/alpha.md")
        .expect("alpha")
        .clone();
    let payload = c.observe(&ctx, &note).expect("observe");
    assert_eq!(payload.handle.locator, "notes/alpha.md");
    assert!(!payload.content.is_empty());
    let fp = fingerprint_file_with_identity(&payload.identity, &payload.content)
        .expect("fingerprintable");
    assert!(fp.starts_with('v'), "got {fp}");
    let fp2 =
        fingerprint_file_with_identity(&payload.identity, &payload.content).expect("fp2");
    assert_eq!(fp, fp2);
}

#[test]
fn obsidian_connector__rename__new_locator_identity() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let c = open_vault(dir.path());
    let ctx = personal_ctx();

    let before = c.list(&ctx).expect("list");
    let alpha = before
        .iter()
        .find(|h| h.locator == "notes/alpha.md")
        .expect("alpha")
        .clone();
    let old_identity = alpha.identity.clone();

    fs::rename(dir.path().join("notes/alpha.md"), dir.path().join("notes/beta.md"))
        .expect("rename");

    let after = c.list(&ctx).expect("list after");
    assert!(
        !after.iter().any(|h| h.locator == "notes/alpha.md"),
        "old locator gone"
    );
    let beta = after
        .iter()
        .find(|h| h.locator == "notes/beta.md")
        .expect("beta");
    assert_ne!(beta.identity, old_identity);
    assert!(beta.identity.ends_with("|notes/beta.md"));
}

#[test]
fn obsidian_connector__content_change__same_identity() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let c = open_vault(dir.path());
    let ctx = personal_ctx();
    let handle = c
        .list(&ctx)
        .expect("list")
        .into_iter()
        .find(|h| h.locator == "notes/alpha.md")
        .expect("alpha");
    let p1 = c.observe(&ctx, &handle).expect("obs1");
    let id1 = p1.identity.clone();
    let fp1 = fingerprint_file_with_identity(&p1.identity, &p1.content).expect("fp1");

    fs::write(
        dir.path().join("notes/alpha.md"),
        b"---\ntitle: Alpha\n---\n# Alpha body\nCHANGED\n",
    )
    .expect("rewrite");

    let p2 = c.observe(&ctx, &handle).expect("obs2");
    assert_eq!(p2.identity, id1, "same path → same identity");
    let fp2 = fingerprint_file_with_identity(&p2.identity, &p2.content).expect("fp2");
    assert_ne!(fp1, fp2, "content change → different fingerprint");
    assert_ne!(p1.content, p2.content);
}

#[test]
fn obsidian_connector__duplicate_title__two_handles() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let c = open_vault(dir.path());
    let ctx = personal_ctx();
    let handles = c.list(&ctx).expect("list");
    let dup = handles
        .iter()
        .filter(|h| h.locator == "notes/dup.md" || h.locator == "notes/gamma.md")
        .count();
    assert_eq!(dup, 2, "two files with shared title remain distinct handles");
    let id_dup = handles
        .iter()
        .find(|h| h.locator == "notes/dup.md")
        .expect("dup")
        .identity
        .clone();
    let id_gamma = handles
        .iter()
        .find(|h| h.locator == "notes/gamma.md")
        .expect("gamma")
        .identity
        .clone();
    assert_ne!(id_dup, id_gamma);
}

#[test]
fn obsidian_connector__path_traversal__refused() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let c = open_vault(dir.path());
    let ctx = personal_ctx();

    let escape = ai_brains_sources::SourceHandle {
        identity: "x".into(),
        kind: SourceKind::File,
        locator: "../escape.md".into(),
    };
    let err = c.observe(&ctx, &escape).expect_err("traversal");
    assert!(
        matches!(
            err,
            ConnectorError::Internal { .. } | ConnectorError::HandleNotFound { .. }
        ),
        "{err}"
    );

    let abs = ai_brains_sources::SourceHandle {
        identity: "x".into(),
        kind: SourceKind::File,
        locator: r"C:\Windows\win.ini".into(),
    };
    let err2 = c.observe(&ctx, &abs).expect_err("absolute");
    assert!(
        matches!(err2, ConnectorError::Internal { .. }),
        "{err2}"
    );
}

#[test]
fn obsidian_connector__symlink_escape__refused() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());

    // Outside target
    let outside = tempdir().expect("outside");
    let outside_file = outside.path().join("secret.md");
    fs::write(&outside_file, b"SECRET OUTSIDE VAULT").expect("write outside");

    let link = dir.path().join("notes/escape-link.md");
    let created = create_file_symlink(&outside_file, &link);
    if !created {
        eprintln!(
            "soft-skip: could not create symlink/junction (privilege missing). \
             Pure refuse path covered by vault_fs + path crate unit tests."
        );
        return;
    }

    let c = open_vault(dir.path());
    let ctx = personal_ctx();
    // Symlinks should not appear in list.
    let handles = c.list(&ctx).expect("list");
    assert!(
        !handles.iter().any(|h| h.locator.contains("escape-link")),
        "symlink must not be listed: {:?}",
        handles.iter().map(|h| &h.locator).collect::<Vec<_>>()
    );

    // Forced observe must error (reparse refuse).
    let forced = ai_brains_sources::SourceHandle {
        identity: "x".into(),
        kind: SourceKind::File,
        locator: "notes/escape-link.md".into(),
    };
    let err = c.observe(&ctx, &forced).expect_err("symlink observe");
    let msg = err.to_string().to_ascii_lowercase();
    assert!(
        msg.contains("reparse")
            || msg.contains("symlink")
            || msg.contains("junction")
            || msg.contains("not found")
            || msg.contains("not a regular"),
        "unexpected err: {err}"
    );
}

/// Create a file symlink (Unix) or attempt Windows symlink; return false if unsupported.
fn create_file_symlink(target: &Path, link: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(windows)]
    {
        // File symlink may need Developer Mode / elevation.
        std::os::windows::fs::symlink_file(target, link).is_ok()
            || std::os::windows::fs::symlink_dir(target, link).is_ok()
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (target, link);
        false
    }
}

#[test]
fn obsidian_connector__propose_write__no_fs_mutation() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let before_count = count_files(dir.path());
    let marker = dir.path().join("notes/alpha.md");
    let before_meta = fs::metadata(&marker).expect("meta");
    let before_mtime = before_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let before_len = before_meta.len();
    let before_body = fs::read(&marker).expect("read");

    let c = open_vault(dir.path());
    let ctx = personal_ctx();
    let handle = c
        .list(&ctx)
        .expect("list")
        .into_iter()
        .find(|h| h.locator == "notes/alpha.md")
        .expect("alpha");
    let proposal = c
        .propose_write(
            &ctx,
            &WriteProposalInput {
                handle,
                proposed_content: "SHOULD NOT HIT DISK".into(),
                rationale: Some("no-fs".into()),
            },
        )
        .expect("propose");
    assert!(!proposal.artifact_id.is_empty());

    assert_eq!(count_files(dir.path()), before_count);
    let after_meta = fs::metadata(&marker).expect("meta after");
    assert_eq!(after_meta.len(), before_len);
    assert_eq!(
        after_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        before_mtime
    );
    assert_eq!(fs::read(&marker).expect("read after"), before_body);
    assert!(
        !dir_contains_bytes(dir.path(), b"SHOULD NOT HIT DISK"),
        "propose_write must not write proposed content"
    );
}

#[test]
fn obsidian_connector__oversized_file__skipped_or_errors() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    // Large note under small max_file_bytes.
    let big = dir.path().join("notes/huge.md");
    fs::write(&big, vec![b'x'; 2048]).expect("huge");

    let opts = VaultOptions {
        max_file_bytes: 512,
        ..VaultOptions::default()
    };
    let c = open_vault_opts(dir.path(), opts);
    let ctx = personal_ctx();
    let handles = c.list(&ctx).expect("list");
    assert!(
        !handles.iter().any(|h| h.locator == "notes/huge.md"),
        "oversized skipped on list"
    );

    let forced = ai_brains_sources::SourceHandle {
        identity: "x".into(),
        kind: SourceKind::File,
        locator: "notes/huge.md".into(),
    };
    let err = c.observe(&ctx, &forced).expect_err("oversized observe");
    let msg = err.to_string().to_ascii_lowercase();
    assert!(
        msg.contains("oversized") || msg.contains("exceeds"),
        "{err}"
    );
}

#[test]
fn obsidian_connector__list__max_files__truncates() {
    let dir = tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join(".obsidian")).expect("obs");
    fs::write(dir.path().join(".obsidian/app.json"), b"{}").expect("json");
    fs::create_dir_all(dir.path().join("notes")).expect("notes");
    for i in 0..20 {
        fs::write(
            dir.path().join(format!("notes/n{i:02}.md")),
            format!("# note {i}\n"),
        )
        .expect("write note");
    }

    let opts = VaultOptions {
        max_files: 5,
        ..VaultOptions::default()
    };
    let c = open_vault_opts(dir.path(), opts);
    let ctx = personal_ctx();
    let handles = c.list(&ctx).expect("list");
    assert!(handles.len() <= 5, "len={}", handles.len());
    assert!(
        c.last_list_truncated(),
        "truncation signal must be set when more notes exist"
    );
}

#[test]
fn obsidian_connector__passes_connector_contract() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let c = open_vault(dir.path());
    assert_eq!(c.manifest().id, OBSIDIAN_CONNECTOR_ID);
    assert_eq!(c.manifest().display_name, "Markdown / Obsidian");
    assert_connector_contract(&c);

    // Registry registration.
    let mut reg = InProcessConnectorRegistry::new();
    reg.register(Box::new(open_vault(dir.path())))
        .expect("register");
    let m = reg.get_manifest(OBSIDIAN_CONNECTOR_ID).expect("manifest");
    assert!(m.principal_id.is_some());
}

#[test]
fn obsidian_connector__preview__bounded_with_body_anchors() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let c = open_vault(dir.path());
    let ctx = personal_ctx();
    let handle = c
        .list(&ctx)
        .expect("list")
        .into_iter()
        .find(|h| h.locator == "notes/alpha.md")
        .expect("alpha");
    let preview = c.preview(&ctx, &handle).expect("preview");
    assert!(preview.text.contains("Alpha body") || preview.text.contains("shared"));
    assert!(preview.line_start.is_some());
    assert!(preview.text.chars().count() <= 4096);
}

#[test]
fn obsidian_connector__vault_root_handle__when_obsidian_present() {
    let dir = tempdir().expect("tempdir");
    write_fixture_vault(dir.path());
    let c = open_vault(dir.path());
    assert!(c.is_obsidian_vault());
    let ctx = personal_ctx();
    let handles = c.list(&ctx).expect("list");
    assert!(
        handles
            .iter()
            .any(|h| h.kind == SourceKind::ObsidianVault && h.locator.is_empty()),
        "expected ObsidianVault root handle"
    );
}
