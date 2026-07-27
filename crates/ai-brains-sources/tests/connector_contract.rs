//! Connector port contract suite (T153).
//!
//! Every built-in (including mock) must pass `assert_connector_contract`.

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
    Connector, ConnectorContext, ConnectorError, ConnectorOperations, ConnectorTrustLabel,
    InProcessConnectorRegistry, MANIFEST_SCHEMA_VERSION, MOCK_CONNECTOR_ID, ManifestError,
    MarkdownObsidianConnector, MockConnector, OBSIDIAN_CONNECTOR_ID, RegistryError, SandboxMode,
    VaultOptions, WriteProposalInput, fingerprint_file_with_identity, parse_manifest_json,
    parse_manifest_str, principal_id_for_connector, validate_manifest,
};
use tempfile::tempdir;
use uuid::Uuid;

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn personal_ctx() -> ConnectorContext {
    ConnectorContext {
        principal_id: None,
        scope: ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(42))),
        privacy: Privacy::LocalOnly,
        trust: ConnectorTrustLabel::LocalOnly,
    }
}

/// Shared contract assertions for any [`Connector`] implementor.
fn assert_connector_contract(c: &dyn Connector) {
    let m = c.manifest();
    assert_eq!(m.schema_version, MANIFEST_SCHEMA_VERSION);
    validate_manifest(m).expect("manifest must validate");
    assert!(!m.source_kinds.is_empty(), "source_kinds must be non-empty");
    assert_eq!(m.sandbox, SandboxMode::TrustedBuiltin);

    let ctx = personal_ctx();
    let ops = m.operations;

    // list
    match c.list(&ctx) {
        Ok(handles) => {
            assert!(ops.list, "list succeeded but operations.list is false");
            for h in &handles {
                assert!(
                    m.source_kinds.iter().any(|k| k == &h.kind),
                    "list returned undeclared kind {:?}",
                    h.kind
                );
            }
        }
        Err(ConnectorError::OperationNotSupported { operation }) => {
            assert!(!ops.list, "list denied but operations.list is true");
            assert_eq!(operation, "list");
        }
        Err(e) => panic!("unexpected list error: {e}"),
    }

    // For ops that need a handle, use list when available or skip observe checks.
    let handles = if ops.list {
        c.list(&ctx).expect("list")
    } else {
        Vec::new()
    };

    if let Some(handle) = handles.first() {
        // observe
        match c.observe(&ctx, handle) {
            Ok(payload) => {
                assert!(ops.observe);
                assert!(
                    m.source_kinds.iter().any(|k| k == &payload.handle.kind),
                    "observe returned undeclared kind"
                );
                assert!(!payload.identity.is_empty());
            }
            Err(ConnectorError::OperationNotSupported { operation }) => {
                assert!(!ops.observe);
                assert_eq!(operation, "observe");
            }
            Err(e) => panic!("unexpected observe error: {e}"),
        }

        // preview
        match c.preview(&ctx, handle) {
            Ok(_) => assert!(ops.preview),
            Err(ConnectorError::OperationNotSupported { operation }) => {
                assert!(!ops.preview);
                assert_eq!(operation, "preview");
            }
            Err(e) => panic!("unexpected preview error: {e}"),
        }

        // propose_write
        let input = WriteProposalInput {
            handle: handle.clone(),
            proposed_content: "proposed".into(),
            rationale: Some("contract".into()),
        };
        match c.propose_write(&ctx, &input) {
            Ok(proposal) => {
                assert!(ops.propose_write);
                assert_eq!(proposal.proposed_content, "proposed");
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

// --- Phase A: manifest schema ---

#[test]
fn manifest__schema_v1_golden__deserializes() {
    let bytes = fs::read(fixture_path("manifest_v1_mock.json")).expect("read golden");
    let m = parse_manifest_json(&bytes).expect("golden must parse");
    assert_eq!(m.schema_version, 1);
    assert_eq!(m.id, MOCK_CONNECTOR_ID);
    assert_eq!(m.display_name, "Mock Connector");
    assert_eq!(m.connector_version, "0.1.0");
    assert_eq!(m.source_kinds, vec![SourceKind::File]);
    assert!(m.operations.list);
    assert!(m.operations.observe);
    assert!(m.operations.preview);
    assert!(m.operations.propose_write);
    assert_eq!(m.sandbox, SandboxMode::TrustedBuiltin);
    assert_eq!(m.default_trust, ConnectorTrustLabel::LocalOnly);
    assert!(m.principal_id.is_none());
}

#[test]
fn manifest__schema_v99__rejected() {
    let json = r#"{
        "schema_version": 99,
        "id": "builtin.bad",
        "display_name": "Bad",
        "connector_version": "9.9.9",
        "source_kinds": ["File"],
        "operations": { "list": false, "observe": false, "preview": false, "propose_write": false },
        "scope_affinity": ["Personal"],
        "freshness": "Fingerprint",
        "credentials": "None",
        "sandbox": "TrustedBuiltin",
        "default_trust": "Unknown"
    }"#;
    let err = parse_manifest_str(json).expect_err("v99 must reject");
    assert!(matches!(
        err,
        ManifestError::UnsupportedSchemaVersion {
            found: 99,
            expected: 1
        }
    ));
}

#[test]
fn manifest__empty_source_kinds__validation_fails() {
    let json = r#"{
        "schema_version": 1,
        "id": "builtin.empty",
        "display_name": "Empty",
        "connector_version": "0.0.1",
        "source_kinds": [],
        "operations": { "list": false, "observe": false, "preview": false, "propose_write": false },
        "scope_affinity": [],
        "freshness": "Fingerprint",
        "credentials": "None",
        "sandbox": "TrustedBuiltin",
        "default_trust": "LocalOnly"
    }"#;
    let err = parse_manifest_str(json).expect_err("empty kinds must fail");
    assert_eq!(err, ManifestError::EmptySourceKinds);
}

#[test]
fn manifest__serde_round_trip__preserves_fields() {
    let bytes = fs::read(fixture_path("manifest_v1_mock.json")).expect("read golden");
    let m = parse_manifest_json(&bytes).expect("parse");
    let encoded = serde_json::to_string(&m).expect("serialize");
    let again = parse_manifest_str(&encoded).expect("re-parse");
    assert_eq!(m, again);
}

// --- Phase B: trait + mock ---

#[test]
fn connector_mock__passes_shared_contract() {
    let mock = MockConnector::new();
    assert_connector_contract(&mock);
}

#[test]
fn connector_mock__declared_ops_match_behavior() {
    // Full ops
    let full = MockConnector::new();
    assert_connector_contract(&full);

    // Observe-only: list/preview/propose_write unsupported
    let limited = MockConnector::with_operations(ConnectorOperations {
        list: false,
        observe: true,
        preview: false,
        propose_write: false,
    });
    let ctx = personal_ctx();
    assert!(matches!(
        limited.list(&ctx),
        Err(ConnectorError::OperationNotSupported { operation: "list" })
    ));
    // Need a handle without list — use known fixture locator
    let handle = ai_brains_sources::SourceHandle {
        identity: "Personal:mock|File|/fixture/notes.md".into(),
        kind: SourceKind::File,
        locator: "/fixture/notes.md".into(),
    };
    let observed = limited
        .observe(&ctx, &handle)
        .expect("observe allowed when operations.observe");
    assert_eq!(observed.handle.locator, "/fixture/notes.md");
    assert_eq!(observed.content, b"# Mock notes\nhello from mock\n");
    assert!(!observed.identity.is_empty());
    assert!(matches!(
        limited.preview(&ctx, &handle),
        Err(ConnectorError::OperationNotSupported {
            operation: "preview"
        })
    ));
    let input = WriteProposalInput {
        handle: handle.clone(),
        proposed_content: "x".into(),
        rationale: None,
    };
    assert!(matches!(
        limited.propose_write(&ctx, &input),
        Err(ConnectorError::OperationNotSupported {
            operation: "propose_write"
        })
    ));
    assert_connector_contract(&limited);
}

#[test]
fn connector_mock__propose_write__no_fs_mutation() {
    let dir = tempdir().expect("tempdir");
    let marker = dir.path().join("vault-root-marker.txt");
    fs::write(&marker, b"original").expect("write marker");
    let before_meta = fs::metadata(&marker).expect("meta");
    let before_mtime = before_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let before_len = before_meta.len();
    let before_count = count_files(dir.path());

    let mock = MockConnector::new();
    let ctx = personal_ctx();
    let handles = mock.list(&ctx).expect("list");
    let handle = handles.first().expect("fixture handle").clone();
    let proposal = mock
        .propose_write(
            &ctx,
            &WriteProposalInput {
                handle,
                proposed_content: "SHOULD NOT HIT DISK".into(),
                rationale: Some("contract no-fs".into()),
            },
        )
        .expect("propose_write");

    assert!(
        proposal.artifact_id.starts_with("mock-proposal:"),
        "artifact id present"
    );

    // Tree unchanged
    let after_count = count_files(dir.path());
    assert_eq!(before_count, after_count, "file count must not change");
    let after_meta = fs::metadata(&marker).expect("meta after");
    assert_eq!(after_meta.len(), before_len);
    let after_mtime = after_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    assert_eq!(before_mtime, after_mtime);
    let body = fs::read(&marker).expect("read marker");
    assert_eq!(body, b"original");
    // No new files under vault root matching proposal content
    assert!(
        !dir_contains_bytes(dir.path(), b"SHOULD NOT HIT DISK"),
        "propose_write must not write proposed content to disk"
    );
}

#[test]
fn connector_mock__observe__fingerprintable_payload() {
    let mock = MockConnector::new();
    let ctx = personal_ctx();
    let handle = mock.list(&ctx).expect("list")[0].clone();
    let payload = mock.observe(&ctx, &handle).expect("observe");
    let fp = fingerprint_file_with_identity(&payload.identity, &payload.content)
        .expect("fingerprintable");
    assert!(
        fp.starts_with('v'),
        "expected versioned fingerprint, got {fp}"
    );
    // Stable across calls
    let fp2 = fingerprint_file_with_identity(&payload.identity, &payload.content).expect("fp2");
    assert_eq!(fp, fp2);
}

// --- Phase C: registry ---

#[test]
fn registry__duplicate_id__fails() {
    let mut reg = InProcessConnectorRegistry::new();
    reg.register(Box::new(MockConnector::new()))
        .expect("first register");
    let err = reg
        .register(Box::new(MockConnector::new()))
        .expect_err("duplicate");
    assert!(matches!(err, RegistryError::DuplicateId(id) if id == MOCK_CONNECTOR_ID));
}

#[test]
fn registry__list_manifests__sorted_by_id() {
    let mut reg = InProcessConnectorRegistry::new();
    reg.register(Box::new(MockConnector::new().with_id("builtin.z")))
        .expect("z");
    reg.register(Box::new(MockConnector::new().with_id("builtin.a")))
        .expect("a");
    reg.register(Box::new(MockConnector::new().with_id("builtin.m")))
        .expect("m");
    let ids: Vec<&str> = reg.list_manifests().iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, vec!["builtin.a", "builtin.m", "builtin.z"]);
}

#[test]
fn registry__principal_id_bound_after_register() {
    let mut reg = InProcessConnectorRegistry::new();
    reg.register(Box::new(MockConnector::new()))
        .expect("register");
    let bound = reg
        .get_manifest(MOCK_CONNECTOR_ID)
        .expect("manifest")
        .principal_id
        .expect("principal_id must be Some after register");
    let expected = principal_id_for_connector(MOCK_CONNECTOR_ID);
    assert_eq!(bound, expected);
    // Stable across recomputation
    assert_eq!(bound, principal_id_for_connector(MOCK_CONNECTOR_ID));
    // Dual-view is intentional: trait object manifest stays unbound; registry
    // bound_manifest carries principal_id for policy (R1-01).
    let connector = reg.get(MOCK_CONNECTOR_ID).expect("connector registered");
    assert_eq!(connector.manifest().id, MOCK_CONNECTOR_ID);
    assert!(
        connector.manifest().principal_id.is_none(),
        "pre-register connector.manifest() is not the policy-bound view"
    );
    assert_eq!(
        reg.get_manifest(MOCK_CONNECTOR_ID)
            .expect("bound manifest")
            .principal_id,
        Some(bound)
    );
}

#[test]
fn connector_mock__insert_source__undeclared_kind__rejected() {
    let mut mock = MockConnector::new();
    let err = mock
        .insert_source(ai_brains_sources::MockSource {
            handle: ai_brains_sources::SourceHandle {
                identity: "x".into(),
                kind: SourceKind::GitRepository,
                locator: "/other".into(),
            },
            content: b"nope".to_vec(),
        })
        .expect_err("undeclared kind must fail");
    assert!(matches!(err, ConnectorError::UndeclaredSourceKind { .. }));
}

#[test]
fn connector_trust_label__parity_names() {
    // Document DTO parity with control-plane ConnectorTrust variant names.
    assert_eq!(ConnectorTrustLabel::LocalOnly.as_str(), "LocalOnly");
    assert_eq!(ConnectorTrustLabel::CloudOk.as_str(), "CloudOk");
    assert_eq!(ConnectorTrustLabel::Unknown.as_str(), "Unknown");
}

#[test]
fn connector_obsidian__passes_shared_contract() {
    let dir = tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join(".obsidian")).expect("obsidian");
    fs::write(dir.path().join(".obsidian/app.json"), b"{}").expect("app.json");
    fs::create_dir_all(dir.path().join("notes")).expect("notes");
    fs::write(
        dir.path().join("notes/hello.md"),
        b"---\ntitle: Hello\n---\n# Hi\n",
    )
    .expect("note");
    let connector = MarkdownObsidianConnector::open(dir.path(), VaultOptions::default())
        .expect("open vault");
    assert_eq!(connector.manifest().id, OBSIDIAN_CONNECTOR_ID);
    assert_connector_contract(&connector);
}

// --- helpers ---

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
