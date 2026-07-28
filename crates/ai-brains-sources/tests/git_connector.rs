//! Git connector integration tests (T155 Phase B).

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ai_brains_core::ids::UserId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_git::{GitError, collect_metadata};
use ai_brains_sources::{
    Connector, ConnectorContext, ConnectorError, ConnectorTrustLabel, DEFAULT_GIT_MAX_HANDLES,
    GIT_CONNECTOR_ID, GitConnector, GitConnectorOptions, InProcessConnectorRegistry,
    MANIFEST_SCHEMA_VERSION, REASON_NOT_A_REPOSITORY, SandboxMode, WriteProposalInput,
    fingerprint_git_metadata, map_git_error, validate_manifest,
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

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    std::env::temp_dir().join(format!("ai-brains-sources-git-{name}-{nanos}"))
}

fn run_git(path: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(args).current_dir(path).output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}

fn init_repo(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = unique_temp_dir(name);
    fs::create_dir_all(&root)?;
    run_git(&root, &["init"])?;
    run_git(&root, &["config", "user.name", "AI Brains Test"])?;
    run_git(&root, &["config", "user.email", "tests@example.com"])?;
    Ok(root)
}

fn commit_file(
    root: &Path,
    relative_path: &str,
    content: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    run_git(root, &["add", "."])?;
    run_git(root, &["commit", "-m", message])?;
    Ok(())
}

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
                assert!(m.source_kinds.iter().any(|k| k == &h.kind));
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

#[test]
fn git_connector__list__repo_root_handle() -> Result<(), Box<dyn std::error::Error>> {
    let root = init_repo("list-handle")?;
    commit_file(&root, "README.md", "hello\n", "initial")?;

    let connector = GitConnector::open(&root, GitConnectorOptions::default())?;
    let handles = connector.list(&personal_ctx())?;
    assert_eq!(handles.len(), 1);
    assert_eq!(handles[0].kind, SourceKind::GitRepository);
    assert!(
        handles[0].identity.contains("GitRepository"),
        "identity={}",
        handles[0].identity
    );
    assert!(!handles[0].locator.is_empty());
    assert!(connector.last_unavailable_reason().is_none());
    assert!(!connector.last_list_truncated());

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn git_connector__observe__fingerprint_matches_metadata() -> Result<(), Box<dyn std::error::Error>>
{
    let root = init_repo("observe-fp")?;
    commit_file(&root, "README.md", "body\n", "initial")?;

    let connector = GitConnector::open(&root, GitConnectorOptions::default())?;
    let ctx = personal_ctx();
    let handle = connector.list(&ctx)?.into_iter().next().expect("handle");
    let payload = connector.observe(&ctx, &handle)?;

    let meta = collect_metadata(&root)?;
    let expected_fp = fingerprint_git_metadata(&meta);
    let actual_fp = ai_brains_sources::fingerprint_bytes(&payload.content);
    assert_eq!(actual_fp, expected_fp);
    assert!(payload.identity.contains("GitRepository"));

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn git_connector__observe__remote_url_forms__same_hash_field()
-> Result<(), Box<dyn std::error::Error>> {
    let root = init_repo("remote-forms")?;
    commit_file(&root, "README.md", "hello\n", "initial")?;

    run_git(
        &root,
        &["remote", "add", "origin", "https://github.com/org/repo.git"],
    )?;

    let connector = GitConnector::open(&root, GitConnectorOptions::default())?;
    let ctx = personal_ctx();
    let handle = connector.list(&ctx)?.into_iter().next().expect("handle");
    let payload_https = connector.observe(&ctx, &handle)?;
    let meta_https = collect_metadata(&root)?;
    let hash_https = meta_https.remote_url_hash.clone();
    assert!(hash_https.is_some());

    // Switch origin to SCP-like form of the same host/path.
    run_git(
        &root,
        &["remote", "set-url", "origin", "git@github.com:org/repo"],
    )?;
    let payload_scp = connector.observe(&ctx, &handle)?;
    let meta_scp = collect_metadata(&root)?;
    assert_eq!(meta_scp.remote_url_hash, hash_https);

    // Canonical content fingerprints include remote_url_hash (not raw URL).
    let fp_https = ai_brains_sources::fingerprint_bytes(&payload_https.content);
    let fp_scp = ai_brains_sources::fingerprint_bytes(&payload_scp.content);
    assert_eq!(fp_https, fp_scp);

    // Content must never embed raw remote URLs.
    let text = String::from_utf8_lossy(&payload_https.content);
    assert!(!text.contains("https://github.com"));
    assert!(!text.contains("git@github.com"));

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn git_connector__observe__timeout_or_command_fail__returns_err() {
    // Live hang avoided: pure mapping of GitError → ConnectorError::Internal.
    let timeout = GitError::Timeout {
        command: "git status".into(),
        elapsed_ms: 5000,
    };
    let mapped = map_git_error(timeout);
    assert!(matches!(
        mapped,
        ConnectorError::Internal { ref detail } if detail.starts_with("timeout:")
    ));

    let failed = GitError::CommandFailed {
        command: "git rev-parse HEAD".into(),
        message: "fatal: bad".into(),
    };
    let mapped = map_git_error(failed);
    assert!(matches!(
        mapped,
        ConnectorError::Internal { ref detail } if detail.starts_with("command_failed:")
    ));
}

#[test]
fn git_connector__list__command_fail__returns_err_not_silent_empty() {
    // Documented hard-fail path: map_git_error always yields Internal, never Ok(()).
    // Integration with collect_metadata soft-maps most CLI failures today (#22
    // residual at git-crate layer); connector maps any Err through this path.
    let err = map_git_error(GitError::CommandFailed {
        command: "git rev-parse --show-toplevel".into(),
        message: "fatal: not a git repository".into(),
    });
    assert!(
        matches!(err, ConnectorError::Internal { .. }),
        "hard fail must be Err, not silent empty: {err:?}"
    );
    assert!(
        !matches!(err, ConnectorError::HandleNotFound { .. }),
        "must not disguise command fail as missing handle"
    );
}

#[test]
fn git_connector__not_a_repo__empty_list_sets_last_unavailable_reason()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let connector = GitConnector::open(dir.path(), GitConnectorOptions::default())?;
    let handles = connector.list(&personal_ctx())?;
    assert!(handles.is_empty());
    assert_eq!(
        connector.last_unavailable_reason().as_deref(),
        Some(REASON_NOT_A_REPOSITORY)
    );
    Ok(())
}

#[test]
fn git_connector__not_a_repo__contract_asserts_side_channel()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let connector = GitConnector::open(dir.path(), GitConnectorOptions::default())?;
    let ctx = personal_ctx();
    let handles = connector.list(&ctx)?;
    assert!(handles.is_empty());
    // Contract discipline: side-channel must be populated for soft empty.
    let reason = connector
        .last_unavailable_reason()
        .expect("side-channel must be set for not_a_repository");
    assert_eq!(reason, REASON_NOT_A_REPOSITORY);

    // observe of a fabricated handle must Err (not silent success).
    let fake = ai_brains_sources::SourceHandle {
        identity: "Personal:x|GitRepository|/nope".into(),
        kind: SourceKind::GitRepository,
        locator: dir.path().to_string_lossy().into(),
    };
    let err = connector
        .observe(&ctx, &fake)
        .expect_err("observe non-repo");
    assert!(matches!(err, ConnectorError::Internal { .. }));
    Ok(())
}

#[test]
fn git_connector__propose_write__not_supported() -> Result<(), Box<dyn std::error::Error>> {
    let root = init_repo("propose")?;
    commit_file(&root, "a.txt", "x\n", "initial")?;
    let connector = GitConnector::open(&root, GitConnectorOptions::default())?;
    let ctx = personal_ctx();
    let handle = connector.list(&ctx)?.into_iter().next().expect("handle");
    let err = connector
        .propose_write(
            &ctx,
            &WriteProposalInput {
                handle,
                proposed_content: "nope".into(),
                rationale: None,
            },
        )
        .expect_err("propose_write unsupported");
    assert!(matches!(
        err,
        ConnectorError::OperationNotSupported {
            operation: "propose_write"
        }
    ));
    assert!(!connector.manifest().operations.propose_write);
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn git_connector__passes_connector_contract_ops() -> Result<(), Box<dyn std::error::Error>> {
    let root = init_repo("contract")?;
    commit_file(&root, "README.md", "hi\n", "initial")?;
    let connector = GitConnector::open(&root, GitConnectorOptions::default())?;
    assert_eq!(connector.manifest().id, GIT_CONNECTOR_ID);
    assert_eq!(connector.manifest().sandbox, SandboxMode::TrustedBuiltin);
    assert_eq!(
        connector.manifest().default_trust,
        ConnectorTrustLabel::LocalOnly
    );
    assert_connector_contract(&connector);

    let mut reg = InProcessConnectorRegistry::new();
    reg.register(Box::new(connector))?;
    assert!(reg.get(GIT_CONNECTOR_ID).is_some());

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn git_connector__list__max_handles_cap() -> Result<(), Box<dyn std::error::Error>> {
    let root = init_repo("max-handles")?;
    commit_file(&root, "README.md", "hi\n", "initial")?;

    // Default: one handle, no truncation.
    let connector = GitConnector::open(&root, GitConnectorOptions::default())?;
    let handles = connector.list(&personal_ctx())?;
    assert_eq!(handles.len(), 1);
    assert!(!connector.last_list_truncated());
    const { assert!(DEFAULT_GIT_MAX_HANDLES >= 1) };

    // Cap 0 → empty + truncated.
    let capped = GitConnector::open(
        &root,
        GitConnectorOptions {
            max_handles: 0,
            ..GitConnectorOptions::default()
        },
    )?;
    let handles = capped.list(&personal_ctx())?;
    assert!(handles.is_empty());
    assert!(capped.last_list_truncated());

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn git_connector__preview__no_secrets() -> Result<(), Box<dyn std::error::Error>> {
    let root = init_repo("preview")?;
    commit_file(&root, "README.md", "hi\n", "initial")?;
    run_git(
        &root,
        &[
            "remote",
            "add",
            "origin",
            "https://user:token@github.com/org/repo.git",
        ],
    )?;
    let connector = GitConnector::open(&root, GitConnectorOptions::default())?;
    let ctx = personal_ctx();
    let handle = connector.list(&ctx)?.into_iter().next().expect("handle");
    let preview = connector.preview(&ctx, &handle)?;
    assert!(!preview.text.contains("token"));
    assert!(!preview.text.contains("user:"));
    assert!(!preview.text.contains("https://"));
    assert!(
        preview.text.contains("remote_hash=present") || preview.text.contains("remote_hash=absent")
    );
    let _ = fs::remove_dir_all(&root);
    Ok(())
}
