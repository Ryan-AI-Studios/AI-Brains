//! Hermes / Honcho connector integration tests (T156).

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_core::ids::UserId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_sources::{
    CircularityClass, Connector, ConnectorContext, ConnectorError, ConnectorTrustLabel,
    DEFAULT_HERMES_MAX_HANDLES, DEFAULT_HONCHO_MAX_HANDLES, ENV_HERMES_CONNECTOR, ExternalItemMeta,
    HERMES_CONNECTOR_ID, HONCHO_CONNECTOR_ID, HermesConnector, HermesConnectorOptions,
    HermesSessionSummary, HonchoConfirmedItem, HonchoConnector, HonchoConnectorOptions,
    InProcessConnectorRegistry, MANIFEST_SCHEMA_VERSION, REASON_CONNECTOR_DISABLED, SandboxMode,
    WriteProposalInput, filter_independent_support, fingerprint_external, is_env_connector_enabled,
    load_hermes_export_dir, load_honcho_export_dir, may_count_as_independent_support,
    validate_manifest,
};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn personal_ctx() -> ConnectorContext {
    ConnectorContext {
        principal_id: None,
        scope: ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(7))),
        privacy: Privacy::LocalOnly,
        trust: ConnectorTrustLabel::LocalOnly,
    }
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

fn parse_meta(content: &[u8]) -> ExternalItemMeta {
    let v: serde_json::Value = serde_json::from_slice(content).expect("json");
    serde_json::from_value(
        v.get("external_item_meta")
            .cloned()
            .expect("external_item_meta"),
    )
    .expect("meta")
}

fn hermes_summary(
    session_id: &str,
    text: &str,
    origin_event_id: Option<&str>,
    privacy: Option<Privacy>,
    assert_independent: bool,
) -> HermesSessionSummary {
    HermesSessionSummary {
        schema_version: 1,
        session_id: session_id.into(),
        summary_text: text.into(),
        source_ids: vec!["s1".into()],
        occurred_at: Some("2026-07-28T00:00:00Z".into()),
        privacy,
        origin_event_id: origin_event_id.map(str::to_string),
        origin_source_id: None,
        origin_marker: None,
        ai_brains_event_id: None,
        ai_brains_source_id: None,
        assert_independent: if assert_independent { Some(true) } else { None },
    }
}

fn honcho_item(
    item_id: &str,
    kind: &str,
    statement: &str,
    origin_event_id: Option<&str>,
    privacy: Option<Privacy>,
    assert_independent: bool,
) -> HonchoConfirmedItem {
    HonchoConfirmedItem {
        schema_version: 1,
        item_id: item_id.into(),
        kind: kind.into(),
        statement: statement.into(),
        provider_timestamps: serde_json::json!("2026-07-28T00:00:00Z"),
        confidence: Some(serde_json::json!(0.8)),
        privacy,
        origin_event_id: origin_event_id.map(str::to_string),
        origin_source_id: None,
        origin_marker: None,
        ai_brains_event_id: None,
        ai_brains_source_id: None,
        assert_independent: if assert_independent { Some(true) } else { None },
    }
}

// --- Hermes ---

#[test]
fn hermes_connector__list__respects_max_handles_truncates() {
    let items: Vec<HermesSessionSummary> = (0..5)
        .map(|i| {
            hermes_summary(
                &format!("sess-{i}"),
                &format!("text-{i}"),
                None,
                None,
                false,
            )
        })
        .collect();
    let connector = HermesConnector::from_fixture(
        items,
        HermesConnectorOptions {
            max_handles: 3,
            ..Default::default()
        },
    );
    let handles = connector.list(&personal_ctx()).expect("list");
    assert_eq!(handles.len(), 3);
    assert!(connector.last_list_truncated());
    assert!(connector.last_unavailable_reason().is_none());
    const { assert!(DEFAULT_HERMES_MAX_HANDLES >= 3) };
    // Deterministic sort by locator.
    let locs: Vec<_> = handles.iter().map(|h| h.locator.as_str()).collect();
    let mut sorted = locs.clone();
    sorted.sort();
    assert_eq!(locs, sorted);
}

#[test]
fn hermes_connector__observe__fingerprint_external() {
    let item = hermes_summary(
        "sess-fp",
        "body for fingerprint",
        None,
        Some(Privacy::LocalOnly),
        false,
    );
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector
        .list(&ctx)
        .expect("list")
        .into_iter()
        .next()
        .expect("handle");
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let fp = fingerprint_external(&payload.identity, &payload.content).expect("fp");
    assert!(fp.starts_with('v'), "expected versioned external fp: {fp}");
    assert!(payload.identity.contains("HermesSession"));
    assert!(payload.identity.contains("sess-fp"));
}

#[test]
fn hermes_connector__observe__embeds_external_item_meta() {
    let item = hermes_summary("sess-meta", "meta body", None, None, false);
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let meta = parse_meta(&payload.content);
    assert_eq!(meta.schema_version, 1);
    assert_eq!(meta.provider, "hermes");
    assert_eq!(meta.provider_item_id, "sess-meta");
    assert_eq!(meta.circularity, CircularityClass::Unknown);
}

#[test]
fn hermes_connector__propose_write__unsupported() {
    let item = hermes_summary("sess-pw", "x", None, None, false);
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
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
fn hermes_connector__disabled_flag__unavailable_or_empty_contracted() {
    let item = hermes_summary("sess-off", "x", None, None, false);
    let connector = HermesConnector::from_fixture_with_enabled(
        vec![item],
        HermesConnectorOptions::default(),
        false,
    );
    let ctx = personal_ctx();
    let handles = connector.list(&ctx).expect("list soft empty");
    assert!(handles.is_empty());
    assert_eq!(
        connector.last_unavailable_reason().as_deref(),
        Some(REASON_CONNECTOR_DISABLED)
    );
    assert!(!connector.is_enabled());
}

#[test]
fn hermes_connector__echo_fixture__classified_echo() {
    let item = hermes_summary(
        "sess-echo",
        "echo body",
        Some("evt-1"),
        Some(Privacy::LocalOnly),
        false,
    );
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let meta = parse_meta(&payload.content);
    assert_eq!(meta.circularity, CircularityClass::EchoOfControlPlane);
    assert!(!may_count_as_independent_support(meta.circularity));
}

#[test]
fn hermes_connector__unmarked_summary__classified_unknown() {
    let item = hermes_summary("sess-unk", "unmarked", None, None, false);
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let meta = parse_meta(&payload.content);
    assert_eq!(meta.circularity, CircularityClass::Unknown);
    assert!(!may_count_as_independent_support(meta.circularity));
}

#[test]
fn hermes_connector__missing_privacy__defaults_sealed() {
    let item = hermes_summary("sess-priv", "no privacy field", None, None, false);
    assert!(item.privacy.is_none());
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    let ctx = personal_ctx();
    // Ambient is LocalOnly — must not be inherited.
    assert_eq!(ctx.privacy, Privacy::LocalOnly);
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let v: serde_json::Value = serde_json::from_slice(&payload.content).expect("json");
    let privacy = v.get("privacy").expect("privacy field");
    assert_eq!(privacy, &serde_json::json!("Sealed"));
}

#[test]
fn hermes_connector__passes_ops_contract() {
    let item = hermes_summary("sess-contract", "c", None, Some(Privacy::LocalOnly), false);
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    assert_eq!(connector.manifest().id, HERMES_CONNECTOR_ID);
    assert_eq!(connector.manifest().sandbox, SandboxMode::TrustedBuiltin);
    assert_connector_contract(&connector);

    let mut reg = InProcessConnectorRegistry::new();
    reg.register(Box::new(HermesConnector::from_fixture(
        vec![hermes_summary("s", "t", None, None, false)],
        HermesConnectorOptions::default(),
    )))
    .expect("register");
    assert!(reg.get(HERMES_CONNECTOR_ID).is_some());
}

#[test]
fn hermes_connector__path_load_invalid_json__errs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("bad.jsonl");
    fs::write(&file, "{not-json\n").expect("write");
    let err = load_hermes_export_dir(dir.path()).expect_err("invalid JSON must err");
    assert!(
        matches!(err, ConnectorError::Internal { ref detail } if detail.contains("invalid JSON")),
        "got {err:?}"
    );

    // Connector list surfaces soft-empty + side-channel for path load failure.
    let connector = HermesConnector::from_path(dir.path(), HermesConnectorOptions::default());
    let handles = connector.list(&personal_ctx()).expect("list soft");
    assert!(handles.is_empty());
    assert!(
        connector
            .last_unavailable_reason()
            .is_some_and(|r| r.contains("invalid JSON")),
        "reason={:?}",
        connector.last_unavailable_reason()
    );
}

#[test]
fn hermes_connector__path_load_valid_jsonl() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("export.ndjson");
    let line = serde_json::to_string(&hermes_summary(
        "from-path",
        "loaded",
        None,
        Some(Privacy::LocalOnly),
        false,
    ))
    .expect("ser");
    fs::write(&file, format!("{line}\n")).expect("write");
    let items = load_hermes_export_dir(dir.path()).expect("load");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].session_id, "from-path");

    let connector = HermesConnector::from_path(dir.path(), HermesConnectorOptions::default());
    let handles = connector.list(&personal_ctx()).expect("list");
    assert_eq!(handles.len(), 1);
    assert_eq!(handles[0].locator, "from-path");
}

// --- Honcho ---

#[test]
fn honcho_connector__list__max_handles() {
    let items: Vec<HonchoConfirmedItem> = (0..5)
        .map(|i| {
            honcho_item(
                &format!("item-{i}"),
                "profile",
                &format!("stmt-{i}"),
                None,
                None,
                false,
            )
        })
        .collect();
    let connector = HonchoConnector::from_fixture(
        items,
        HonchoConnectorOptions {
            max_handles: 2,
            ..Default::default()
        },
    );
    let handles = connector.list(&personal_ctx()).expect("list");
    assert_eq!(handles.len(), 2);
    assert!(connector.last_list_truncated());
    const { assert!(DEFAULT_HONCHO_MAX_HANDLES >= 2) };
}

#[test]
fn honcho_connector__observe__confirmed_item_meta() {
    let item = honcho_item(
        "item-meta",
        "conclusion",
        "a conclusion",
        None,
        Some(Privacy::LocalOnly),
        false,
    );
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    assert!(payload.identity.contains("Honcho"));
    assert!(payload.identity.contains("item-meta"));
    let meta = parse_meta(&payload.content);
    assert_eq!(meta.provider, "honcho");
    assert_eq!(meta.provider_item_id, "item-meta");
    let fp = fingerprint_external(&payload.identity, &payload.content).expect("fp");
    assert!(fp.starts_with('v'));
}

#[test]
fn honcho_connector__echo_of_control_plane__not_independent_support() {
    let item = honcho_item(
        "item-echo",
        "conclusion",
        "echo",
        Some("evt-honcho-1"),
        Some(Privacy::LocalOnly),
        false,
    );
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let meta = parse_meta(&payload.content);
    assert_eq!(meta.circularity, CircularityClass::EchoOfControlPlane);
    assert!(!may_count_as_independent_support(meta.circularity));
    assert!(filter_independent_support(std::slice::from_ref(&meta)).is_empty());
}

#[test]
fn honcho_connector__unmarked_profile__unknown_not_independent_support() {
    let item = honcho_item("item-unk", "profile", "prefs", None, None, false);
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let meta = parse_meta(&payload.content);
    assert_eq!(meta.circularity, CircularityClass::Unknown);
    assert!(!may_count_as_independent_support(meta.circularity));
}

#[test]
fn honcho_connector__assert_independent_item__may_support() {
    let item = honcho_item(
        "item-ind",
        "representation",
        "trusted",
        None,
        Some(Privacy::LocalOnly),
        true,
    );
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let meta = parse_meta(&payload.content);
    assert_eq!(meta.circularity, CircularityClass::Independent);
    assert!(may_count_as_independent_support(meta.circularity));
    assert_eq!(
        filter_independent_support(std::slice::from_ref(&meta)).len(),
        1
    );
}

#[test]
fn honcho_connector__missing_privacy__defaults_sealed() {
    let item = honcho_item("item-priv", "profile", "sealed default", None, None, false);
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    let ctx = personal_ctx();
    assert_eq!(ctx.privacy, Privacy::LocalOnly);
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let v: serde_json::Value = serde_json::from_slice(&payload.content).expect("json");
    assert_eq!(
        v.get("privacy").expect("privacy"),
        &serde_json::json!("Sealed")
    );
}

#[test]
fn honcho_connector__propose_write__unsupported() {
    let item = honcho_item("item-pw", "profile", "x", None, None, false);
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let err = connector
        .propose_write(
            &ctx,
            &WriteProposalInput {
                handle,
                proposed_content: "no".into(),
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
}

#[test]
fn honcho_connector__passes_ops_contract() {
    let item = honcho_item(
        "item-c",
        "profile",
        "c",
        None,
        Some(Privacy::LocalOnly),
        false,
    );
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    assert_eq!(connector.manifest().id, HONCHO_CONNECTOR_ID);
    assert_connector_contract(&connector);

    let mut reg = InProcessConnectorRegistry::new();
    reg.register(Box::new(HonchoConnector::from_fixture(
        vec![honcho_item("i", "profile", "t", None, None, false)],
        HonchoConnectorOptions::default(),
    )))
    .expect("register");
    assert!(reg.get(HONCHO_CONNECTOR_ID).is_some());
}

#[test]
fn honcho_connector__disabled_flag__empty_contracted() {
    let item = honcho_item("item-off", "profile", "x", None, None, false);
    let connector = HonchoConnector::from_fixture_with_enabled(
        vec![item],
        HonchoConnectorOptions::default(),
        false,
    );
    let handles = connector.list(&personal_ctx()).expect("list");
    assert!(handles.is_empty());
    assert_eq!(
        connector.last_unavailable_reason().as_deref(),
        Some(REASON_CONNECTOR_DISABLED)
    );
}

#[test]
fn honcho_connector__path_load_invalid_json__errs() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("bad.ndjson"), "not-json-at-all\n").expect("write");
    let err = load_honcho_export_dir(dir.path()).expect_err("must err");
    assert!(matches!(
        err,
        ConnectorError::Internal { ref detail } if detail.contains("invalid JSON")
    ));
}

#[test]
fn hermes_connector__fixture_files_roundtrip() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/external_memory/hermes_echo.json");
    let raw = fs::read_to_string(&path).expect("read fixture");
    let item: HermesSessionSummary = serde_json::from_str(&raw).expect("parse");
    assert_eq!(item.session_id, "sess-echo-1");
    assert!(item.origin_event_id.is_some());
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    let handle = connector.list(&personal_ctx()).expect("list")[0].clone();
    let meta = parse_meta(
        &connector
            .observe(&personal_ctx(), &handle)
            .expect("observe")
            .content,
    );
    assert_eq!(meta.circularity, CircularityClass::EchoOfControlPlane);
}

#[test]
fn hermes_connector__independent_fixture_file__may_support() {
    // R1-06: hermes_independent.json → Independent + may_count + fingerprint.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/external_memory/hermes_independent.json");
    let raw = fs::read_to_string(&path).expect("read fixture");
    let item: HermesSessionSummary = serde_json::from_str(&raw).expect("parse");
    assert_eq!(item.session_id, "sess-independent-1");
    assert_eq!(item.assert_independent, Some(true));
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    assert!(connector.trust_assert_independent());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let meta = parse_meta(&payload.content);
    assert_eq!(meta.circularity, CircularityClass::Independent);
    assert!(may_count_as_independent_support(meta.circularity));
    let fp = fingerprint_external(&payload.identity, &payload.content).expect("fp");
    assert!(fp.starts_with('v'), "expected versioned external fp: {fp}");
}

#[test]
fn honcho_connector__fixture_files_roundtrip() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/external_memory/honcho_independent.json");
    let raw = fs::read_to_string(&path).expect("read fixture");
    let item: HonchoConfirmedItem = serde_json::from_str(&raw).expect("parse");
    assert_eq!(item.assert_independent, Some(true));
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    let handle = connector.list(&personal_ctx()).expect("list")[0].clone();
    let meta = parse_meta(
        &connector
            .observe(&personal_ctx(), &handle)
            .expect("observe")
            .content,
    );
    assert_eq!(meta.circularity, CircularityClass::Independent);
}

// --- R1-01 trust boundary for assert_independent ---

#[test]
fn hermes_connector__path_assert_independent__not_independent() {
    // Untrusted path JSON with assert_independent:true must NOT count as Independent.
    let dir = tempfile::tempdir().expect("tempdir");
    let line = r#"{"schema_version":1,"session_id":"path-assert","summary_text":"spoof","source_ids":[],"assert_independent":true}"#;
    fs::write(dir.path().join("export.jsonl"), format!("{line}\n")).expect("write");
    let connector = HermesConnector::from_path(dir.path(), HermesConnectorOptions::default());
    assert!(!connector.trust_assert_independent());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let meta = parse_meta(&connector.observe(&ctx, &handle).expect("observe").content);
    assert_ne!(meta.circularity, CircularityClass::Independent);
    assert!(!may_count_as_independent_support(meta.circularity));
    // No markers → Unknown (classify path, never Independent from path field).
    assert_eq!(meta.circularity, CircularityClass::Unknown);
}

#[test]
fn hermes_connector__fixture_assert_independent__still_independent() {
    let item = hermes_summary(
        "fix-assert",
        "trusted",
        None,
        Some(Privacy::LocalOnly),
        true,
    );
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    assert!(connector.trust_assert_independent());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let meta = parse_meta(&connector.observe(&ctx, &handle).expect("observe").content);
    assert_eq!(meta.circularity, CircularityClass::Independent);
    assert!(may_count_as_independent_support(meta.circularity));
}

#[test]
fn honcho_connector__path_assert_independent__not_independent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let line = r#"{"schema_version":1,"item_id":"path-assert","kind":"profile","statement":"spoof","provider_timestamps":"2026-07-28T00:00:00Z","assert_independent":true}"#;
    fs::write(dir.path().join("export.ndjson"), format!("{line}\n")).expect("write");
    let connector = HonchoConnector::from_path(dir.path(), HonchoConnectorOptions::default());
    assert!(!connector.trust_assert_independent());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let meta = parse_meta(&connector.observe(&ctx, &handle).expect("observe").content);
    assert_ne!(meta.circularity, CircularityClass::Independent);
    assert!(!may_count_as_independent_support(meta.circularity));
    assert_eq!(meta.circularity, CircularityClass::Unknown);
}

#[test]
fn honcho_connector__fixture_assert_independent__still_independent() {
    let item = honcho_item(
        "fix-assert",
        "profile",
        "trusted",
        None,
        Some(Privacy::LocalOnly),
        true,
    );
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    assert!(connector.trust_assert_independent());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let meta = parse_meta(&connector.observe(&ctx, &handle).expect("observe").content);
    assert_eq!(meta.circularity, CircularityClass::Independent);
    assert!(may_count_as_independent_support(meta.circularity));
}

// --- R1-02 alternate marker keys on connector path ---

#[test]
fn hermes_connector__ai_brains_event_id_only__echo() {
    let mut item = hermes_summary("sess-alt-marker", "alt keys", None, None, false);
    item.ai_brains_event_id = Some("evt-x".into());
    assert!(item.origin_event_id.is_none());
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let meta = parse_meta(&connector.observe(&ctx, &handle).expect("observe").content);
    assert_eq!(meta.circularity, CircularityClass::EchoOfControlPlane);
    assert_eq!(meta.origin_event_id.as_deref(), Some("evt-x"));
    assert!(!may_count_as_independent_support(meta.circularity));
}

#[test]
fn honcho_connector__ai_brains_event_id_only__echo() {
    let mut item = honcho_item("item-alt-marker", "profile", "alt", None, None, false);
    item.ai_brains_event_id = Some("evt-x".into());
    assert!(item.origin_event_id.is_none());
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let meta = parse_meta(&connector.observe(&ctx, &handle).expect("observe").content);
    assert_eq!(meta.circularity, CircularityClass::EchoOfControlPlane);
    assert_eq!(meta.origin_event_id.as_deref(), Some("evt-x"));
    assert!(!may_count_as_independent_support(meta.circularity));
}

#[test]
fn hermes_connector__path_ai_brains_event_id_only__echo() {
    let dir = tempfile::tempdir().expect("tempdir");
    let line = r#"{"schema_version":1,"session_id":"path-alt","summary_text":"x","source_ids":[],"ai_brains_event_id":"evt-x"}"#;
    fs::write(dir.path().join("export.jsonl"), format!("{line}\n")).expect("write");
    let connector = HermesConnector::from_path(dir.path(), HermesConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let meta = parse_meta(&connector.observe(&ctx, &handle).expect("observe").content);
    assert_eq!(meta.circularity, CircularityClass::EchoOfControlPlane);
    assert!(!may_count_as_independent_support(meta.circularity));
}

// --- Codex P2: path load timeout_ms enforced ---

#[test]
fn hermes_connector__path_load__timeout_ms_zero__unavailable_or_err() {
    // timeout_ms=0 → immediate path-load timeout without starting work.
    // list: soft-empty + reason containing "timeout" (anti-#22).
    // observe: Err with timeout detail (not silent empty).
    let dir = tempfile::tempdir().expect("tempdir");
    let line =
        r#"{"schema_version":1,"session_id":"sess-timeout","summary_text":"x","source_ids":[]}"#;
    fs::write(dir.path().join("export.jsonl"), format!("{line}\n")).expect("write");

    let connector = HermesConnector::from_path(
        dir.path(),
        HermesConnectorOptions {
            timeout_ms: 0,
            ..Default::default()
        },
    );
    let ctx = personal_ctx();
    let handles = connector.list(&ctx).expect("list soft on timeout");
    assert!(
        handles.is_empty(),
        "timeout must soft-empty list, got {handles:?}"
    );
    let reason = connector
        .last_unavailable_reason()
        .expect("timeout must set last_unavailable_reason (not silent empty)");
    assert!(
        reason.to_ascii_lowercase().contains("timeout"),
        "reason must mention timeout, got {reason}"
    );

    let bogus = ai_brains_sources::SourceHandle {
        identity: "x|HermesSession|sess-timeout".into(),
        kind: SourceKind::HermesSession,
        locator: "sess-timeout".into(),
    };
    let err = connector
        .observe(&ctx, &bogus)
        .expect_err("observe after timed-out load must Err");
    match err {
        ConnectorError::Internal { detail } => {
            assert!(
                detail.to_ascii_lowercase().contains("timeout"),
                "observe err must mention timeout, got {detail}"
            );
        }
        other => panic!("expected Internal timeout, got {other:?}"),
    }
}

#[test]
fn honcho_connector__path_load__timeout_ms_zero__unavailable_or_err() {
    let dir = tempfile::tempdir().expect("tempdir");
    let line = r#"{"schema_version":1,"item_id":"item-timeout","kind":"profile","statement":"x","provider_timestamps":"2026-07-28T00:00:00Z"}"#;
    fs::write(dir.path().join("export.ndjson"), format!("{line}\n")).expect("write");

    let connector = HonchoConnector::from_path(
        dir.path(),
        HonchoConnectorOptions {
            timeout_ms: 0,
            ..Default::default()
        },
    );
    let ctx = personal_ctx();
    let handles = connector.list(&ctx).expect("list soft on timeout");
    assert!(handles.is_empty());
    let reason = connector
        .last_unavailable_reason()
        .expect("timeout must set last_unavailable_reason");
    assert!(
        reason.to_ascii_lowercase().contains("timeout"),
        "reason must mention timeout, got {reason}"
    );

    let bogus = ai_brains_sources::SourceHandle {
        identity: "x|Honcho|item-timeout".into(),
        kind: SourceKind::Honcho,
        locator: "item-timeout".into(),
    };
    let err = connector
        .observe(&ctx, &bogus)
        .expect_err("observe after timed-out load must Err");
    match err {
        ConnectorError::Internal { detail } => {
            assert!(
                detail.to_ascii_lowercase().contains("timeout"),
                "observe err must mention timeout, got {detail}"
            );
        }
        other => panic!("expected Internal timeout, got {other:?}"),
    }
}

#[test]
fn hermes_connector__fixture__timeout_ms_zero__unaffected() {
    // Fixture mode does no path IO; timeout_ms must not break list/observe.
    let item = hermes_summary("fix-timeout", "ok", None, None, false);
    let connector = HermesConnector::from_fixture(
        vec![item],
        HermesConnectorOptions {
            timeout_ms: 0,
            ..Default::default()
        },
    );
    let ctx = personal_ctx();
    let handles = connector.list(&ctx).expect("fixture list");
    assert_eq!(handles.len(), 1);
    assert!(connector.last_unavailable_reason().is_none());
    connector
        .observe(&ctx, &handles[0])
        .expect("fixture observe");
}

// --- R1-03 unparseable privacy → Sealed ---

#[test]
fn hermes_connector__unparseable_privacy__defaults_sealed() {
    let raw = r#"{"schema_version":1,"session_id":"sess-bad-priv","summary_text":"x","source_ids":[],"privacy":"not-a-real-privacy"}"#;
    let item: HermesSessionSummary =
        serde_json::from_str(raw).expect("must load despite bad privacy");
    assert!(item.privacy.is_none(), "invalid privacy maps to None");
    let connector = HermesConnector::from_fixture(vec![item], HermesConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let v: serde_json::Value = serde_json::from_slice(&payload.content).expect("json");
    assert_eq!(
        v.get("privacy").expect("privacy"),
        &serde_json::json!("Sealed")
    );
}

#[test]
fn honcho_connector__unparseable_privacy__defaults_sealed() {
    let raw = r#"{"schema_version":1,"item_id":"item-bad-priv","kind":"profile","statement":"x","provider_timestamps":"2026-07-28T00:00:00Z","privacy":"not-a-real-privacy"}"#;
    let item: HonchoConfirmedItem =
        serde_json::from_str(raw).expect("must load despite bad privacy");
    assert!(item.privacy.is_none(), "invalid privacy maps to None");
    let connector = HonchoConnector::from_fixture(vec![item], HonchoConnectorOptions::default());
    let ctx = personal_ctx();
    let handle = connector.list(&ctx).expect("list")[0].clone();
    let payload = connector.observe(&ctx, &handle).expect("observe");
    let v: serde_json::Value = serde_json::from_slice(&payload.content).expect("json");
    assert_eq!(
        v.get("privacy").expect("privacy"),
        &serde_json::json!("Sealed")
    );
}

#[test]
fn hermes_connector__env_flag_default_off() {
    // Without the env var set to truthy, helper reports disabled.
    // Do not mutate process env here if other tests race; just document default
    // parse semantics for missing / false values.
    assert!(!is_env_connector_enabled(
        "AI_BRAINS_CONNECTOR_HERMES_SURELY_UNSET_XYZ"
    ));
    let _ = ENV_HERMES_CONNECTOR;
}

#[test]
fn hermes_connector__unavailable__side_channel() {
    let connector = HermesConnector::unavailable("missing_export");
    let handles = connector.list(&personal_ctx()).expect("list");
    assert!(handles.is_empty());
    assert_eq!(
        connector.last_unavailable_reason().as_deref(),
        Some("missing_export")
    );
    assert!(connector.is_store_unavailable());
    let fake = ai_brains_sources::SourceHandle {
        identity: "x|HermesSession|s".into(),
        kind: SourceKind::HermesSession,
        locator: "s".into(),
    };
    let err = connector
        .observe(&personal_ctx(), &fake)
        .expect_err("observe unavailable");
    assert!(matches!(
        err,
        ConnectorError::Internal { ref detail } if detail.contains("missing_export")
    ));
}
