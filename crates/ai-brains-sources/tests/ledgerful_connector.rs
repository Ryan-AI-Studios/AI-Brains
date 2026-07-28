//! Ledgerful connector integration tests (T155 Phase C).

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_core::ids::UserId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_sources::{
    Connector, ConnectorContext, ConnectorError, ConnectorTrustLabel,
    DEFAULT_LEDGERFUL_MAX_RECORDS, InProcessConnectorRegistry, LEDGERFUL_CONNECTOR_ID,
    LedgerfulConnector, LedgerfulConnectorOptions, MANIFEST_SCHEMA_VERSION, SandboxMode,
    WriteProposalInput, fingerprint_ledgerful, record_locator, serialize_bridge_record,
    stable_record_key, validate_manifest,
};
use uuid::Uuid;

fn personal_ctx() -> ConnectorContext {
    ConnectorContext {
        principal_id: None,
        scope: ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(7))),
        privacy: Privacy::LocalOnly,
        trust: ConnectorTrustLabel::LocalOnly,
    }
}

fn bridge_record_json(
    project_id: &str,
    kind: &str,
    tx_id: Option<&str>,
    parent_hash: Option<&str>,
    text: &str,
) -> BridgeRecord {
    let tx = match tx_id {
        Some(t) => format!("\"{t}\""),
        None => "null".into(),
    };
    let parent = match parent_hash {
        Some(p) => format!("\"{p}\""),
        None => "null".into(),
    };
    let json = format!(
        r#"{{
            "bridge_version":"1.0",
            "direction":"inbound",
            "timestamp":"2026-05-19T00:00:00Z",
            "parent_hash":{parent},
            "project_id":"{project_id}",
            "session_id":null,
            "tx_id":{tx},
            "record_kind":"{kind}",
            "payload":{{"text":"{text}"}},
            "privacy":"ProjectLocal"
        }}"#
    );
    serde_json::from_str(&json).expect("BridgeRecord")
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
fn ledgerful_connector__list__respects_max_records_truncates() {
    let records: Vec<BridgeRecord> = (0..5)
        .map(|i| {
            bridge_record_json(
                "proj",
                "prompt",
                Some(&format!("tx-{i}")),
                Some(&format!("hash-{i}")),
                &format!("body-{i}"),
            )
        })
        .collect();

    let connector =
        LedgerfulConnector::from_records(records, LedgerfulConnectorOptions { max_records: 3 });
    let handles = connector.list(&personal_ctx()).expect("list");
    assert_eq!(handles.len(), 3);
    assert!(connector.last_list_truncated());
    assert!(connector.last_unavailable_reason().is_none());
    // Document default cap is well above the test fixture size.
    const { assert!(DEFAULT_LEDGERFUL_MAX_RECORDS >= 3) };
}

#[test]
fn ledgerful_connector__observe__fingerprint_ledgerful_authoritative() {
    let record = bridge_record_json(
        "proj-auth",
        "prompt",
        Some("tx-1"),
        Some("abc123def"),
        "hello",
    );
    let connector =
        LedgerfulConnector::from_records(vec![record], LedgerfulConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector
        .list(&ctx)
        .expect("list")
        .into_iter()
        .next()
        .expect("handle");
    let payload = connector.observe(&ctx, &handle).expect("observe");

    let fp = fingerprint_ledgerful(&payload.identity, &payload.content).expect("fp");
    assert_eq!(fp, "ledgerful:abc123def");

    // Privacy preserved in content.
    let text = String::from_utf8_lossy(&payload.content);
    assert!(
        text.contains("ProjectLocal") || text.contains("privacy"),
        "privacy field must appear in observe content: {text}"
    );
}

#[test]
fn ledgerful_connector__observe__missing_store__returns_err_or_side_channel() {
    let connector = LedgerfulConnector::unavailable("missing_store");
    let ctx = personal_ctx();
    let handles = connector.list(&ctx).expect("list soft empty");
    assert!(handles.is_empty());
    assert_eq!(
        connector.last_unavailable_reason().as_deref(),
        Some("missing_store")
    );

    let fake = ai_brains_sources::SourceHandle {
        identity: "x|Ledgerful|p|k|t".into(),
        kind: SourceKind::Ledgerful,
        locator: "p|k|t".into(),
    };
    let err = connector
        .observe(&ctx, &fake)
        .expect_err("observe unavailable");
    assert!(
        matches!(err, ConnectorError::Internal { ref detail } if detail.contains("missing_store"))
    );
}

#[test]
fn ledgerful_connector__unavailable__contract_asserts_side_channel() {
    let connector = LedgerfulConnector::unavailable("missing_store");
    let ctx = personal_ctx();
    let _ = connector.list(&ctx).expect("list");
    let reason = connector
        .last_unavailable_reason()
        .expect("side-channel must be set");
    assert_eq!(reason, "missing_store");
    assert!(connector.is_store_unavailable());
}

#[test]
fn ledgerful_connector__propose_write__not_supported() {
    let record = bridge_record_json("p", "prompt", Some("tx"), Some("h"), "t");
    let connector =
        LedgerfulConnector::from_records(vec![record], LedgerfulConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector
        .list(&ctx)
        .expect("list")
        .into_iter()
        .next()
        .expect("handle");
    let err = connector
        .propose_write(
            &ctx,
            &WriteProposalInput {
                handle,
                proposed_content: "nope".into(),
                rationale: None,
            },
        )
        .expect_err("unsupported");
    assert!(matches!(
        err,
        ConnectorError::OperationNotSupported {
            operation: "propose_write"
        }
    ));
    assert!(!connector.manifest().operations.propose_write);
}

#[test]
fn ledgerful_connector__passes_connector_contract_ops() {
    let record = bridge_record_json("p", "prompt", Some("tx"), Some("hash"), "hi");
    let connector =
        LedgerfulConnector::from_records(vec![record], LedgerfulConnectorOptions::default());
    assert_eq!(connector.manifest().id, LEDGERFUL_CONNECTOR_ID);
    assert_eq!(connector.manifest().sandbox, SandboxMode::TrustedBuiltin);
    assert_connector_contract(&connector);

    let mut reg = InProcessConnectorRegistry::new();
    reg.register(Box::new(connector)).expect("register");
    assert!(reg.get(LEDGERFUL_CONNECTOR_ID).is_some());
}

#[test]
fn ledgerful_connector__bridge_record_roundtrip_identity_stable() {
    let record = bridge_record_json("proj-rt", "insight", Some("tx-stable"), Some("ph"), "body");
    let key1 = stable_record_key(&record);
    let loc1 = record_locator(&record);
    let bytes1 = serialize_bridge_record(&record).expect("ser");

    // Re-parse and re-derive: identity components stable.
    let again: BridgeRecord = serde_json::from_slice(&bytes1).expect("de");
    assert_eq!(stable_record_key(&again), key1);
    assert_eq!(record_locator(&again), loc1);
    assert_eq!(again.privacy, Privacy::LocalOnly);
    assert_eq!(again.tx_id.as_deref(), Some("tx-stable"));

    let connector =
        LedgerfulConnector::from_records(vec![record], LedgerfulConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector
        .list(&ctx)
        .expect("list")
        .into_iter()
        .next()
        .expect("handle");
    assert!(handle.identity.contains("Ledgerful"));
    assert!(handle.identity.contains("proj-rt"));
    assert!(handle.identity.contains("insight"));
    assert!(handle.identity.contains("tx-stable"));
    assert_eq!(handle.locator, loc1);

    // Second list yields same identity.
    let handle2 = connector
        .list(&ctx)
        .expect("list2")
        .into_iter()
        .next()
        .expect("handle2");
    assert_eq!(handle.identity, handle2.identity);
    assert_eq!(handle.locator, handle2.locator);
}

#[test]
fn ledgerful_connector__healthy_empty__no_unavailable_reason() {
    let connector = LedgerfulConnector::from_records(vec![], LedgerfulConnectorOptions::default());
    let handles = connector.list(&personal_ctx()).expect("list");
    assert!(handles.is_empty());
    assert!(
        connector.last_unavailable_reason().is_none(),
        "healthy empty must not set unavailable reason"
    );
    assert!(!connector.is_store_unavailable());
}
