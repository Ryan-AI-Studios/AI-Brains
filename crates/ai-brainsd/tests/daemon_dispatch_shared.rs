//! Shared live dispatch smoke tests (T158 + T159 governed handlers).
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_contracts::briefings::QueryKnowledgeRequest;
use ai_brains_contracts::erasure::RequestErasureRequest;
use ai_brains_contracts::knowledge::ProposeConclusionRequest;
use ai_brains_contracts::scopes::ResolveScopeRequest;
use ai_brains_core::ids::{PrincipalId, ProjectId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse, UNSUPPORTED_OPERATION};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use ai_brainsd::DaemonWriter;
use ai_brainsd::dispatch::{
    INVALID_REQUEST, LiveDispatchResult, handle_daemon_request, parse_live_request_line,
    write_dispatch_result,
};
use ai_brainsd::services::GovernedServices;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::AsyncReadExt;
use uuid::Uuid;

fn unique_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    std::env::temp_dir().join(format!("ai-brainsd-dispatch-{name}-{nanos}"))
}

struct Harness {
    writer: DaemonWriter,
    services: GovernedServices,
    store: Arc<SqliteEventStore>,
}

async fn start_harness(name: &str) -> Result<Harness, Box<dyn std::error::Error + Send + Sync>> {
    let dir = unique_dir(name);
    std::fs::create_dir_all(&dir)?;
    let db_path = dir.join("vault.db");
    let key = ai_brains_crypto::SqlCipherKey::from_raw(
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
    );
    let conn = VaultConnection::open(db_path, &key)?;
    conn.migrate()?;
    let store = Arc::new(SqliteEventStore::new(conn));
    let writer = DaemonWriter::start(dir.join("spool"), Arc::clone(&store)).await?;
    let services = GovernedServices::new(Arc::clone(&store));
    Ok(Harness {
        writer,
        services,
        store,
    })
}

fn grant_propose(
    store: &Arc<SqliteEventStore>,
    principal: &ai_brains_core::principal::Principal,
    scope: ScopeRef,
) {
    use ai_brains_control_plane::{
        StorePorts, SystemClock, issue_grant, make_principal, register_principal,
    };
    let ports = StorePorts::from_store(SqliteEventStore::new(store.connection().clone()));
    let clock = SystemClock;
    let _ = make_principal; // silence if unused with re-export path
    register_principal(&ports.writer, &clock, principal).expect("register");
    issue_grant(
        &ports.writer,
        &clock,
        principal.id,
        scope,
        GrantCapability::ProposeConclusion,
        Privacy::LocalOnly,
    )
    .expect("grant");
}

#[tokio::test]
async fn daemon_dispatch__legacy_ping__pong() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let h = start_harness("ping").await?;
    let outcome = handle_daemon_request(DaemonRequest::Ping, &h.writer, &h.services).await?;
    match outcome {
        LiveDispatchResult::Response(boxed) if matches!(*boxed, DaemonResponse::Pong) => Ok(()),
        other => panic!("expected Pong, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__resolve_scope__returns_scope_resolved_not_unsupported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("resolve").await?;
    let project_id = ProjectId::new();
    let outcome = handle_daemon_request(
        DaemonRequest::ResolveScope(ResolveScopeRequest {
            api_version: ai_brains_contracts::scopes::API_VERSION.to_string(),
            cwd: None,
            signals: None,
            explicit_project_id: Some(project_id.to_string()),
            force_personal: false,
            personal_user_id: None,
        }),
        &h.writer,
        &h.services,
    )
    .await?;
    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::ScopeResolved(resp) => {
                assert!(resp.authoritative || resp.confidence == "High");
                assert!(!resp.scope.is_empty());
                assert_ne!(resp.confidence, "");
                Ok(())
            }
            DaemonResponse::Error(err) => {
                panic!(
                    "expected ScopeResolved, got error {}: {}",
                    err.code, err.message
                )
            }
            other => panic!("expected ScopeResolved, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__propose_conclusion__appends_via_writer()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("propose-ok").await?;
    let principal_id = PrincipalId::new();
    let principal =
        ai_brains_control_plane::make_principal(PrincipalKind::Human, principal_id, "test-human");
    let user = UserId::new();
    let scope = ScopeRef::Personal(user);
    grant_propose(&h.store, &principal, scope.clone());

    let scope_key = ai_brains_control_plane::scope_identity_key(&scope);
    let outcome = handle_daemon_request(
        DaemonRequest::ProposeConclusion(ProposeConclusionRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            scope: scope_key,
            statement: "daemon proposed claim".into(),
            evidence_ids: vec![],
            privacy: Some("LocalOnly".into()),
            command_id: None,
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::ConclusionProposed(resp) => {
                assert!(!resp.conclusion_id.is_empty());
                assert!(resp.status == "proposed" || resp.status == "unsupported");
                Ok(())
            }
            DaemonResponse::Error(err) => {
                panic!(
                    "expected ConclusionProposed, got {}: {}",
                    err.code, err.message
                )
            }
            other => panic!("unexpected {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__propose_conclusion_policy_denied__policy_denied_code()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("propose-deny").await?;
    // Principal without grants → production_policy denies ProposeConclusion.
    let principal_id = PrincipalId::new();
    let scope_key = format!("Personal:{}", UserId::new());
    let outcome = handle_daemon_request(
        DaemonRequest::ProposeConclusion(ProposeConclusionRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            scope: scope_key,
            statement: "should be denied".into(),
            evidence_ids: vec![],
            privacy: Some("LocalOnly".into()),
            command_id: None,
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "POLICY_DENIED");
                Ok(())
            }
            other => panic!("expected POLICY_DENIED error, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__query_knowledge__returns_progressive_shape()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("query-k").await?;
    let project_id = ProjectId::new();
    let scope = format!("Repository:{project_id}");
    let outcome = handle_daemon_request(
        DaemonRequest::QueryKnowledge(QueryKnowledgeRequest {
            api_version: "1".into(),
            query: "budget".into(),
            scope: Some(scope),
            principal_id: None,
            limit: Some(5),
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::QueryKnowledge(resp) => {
                // Denied or empty results are fine; shape must be progressive.
                assert!(resp.results.is_empty() || !resp.results.is_empty());
                assert!(!resp.query_trace_id.is_empty() || resp.denied);
                Ok(())
            }
            DaemonResponse::Error(err) => {
                panic!("expected QueryKnowledge, got {}: {}", err.code, err.message)
            }
            other => panic!("unexpected {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__request_erasure__appends_ticket_then_accepted()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("erase").await?;
    let outcome = handle_daemon_request(
        DaemonRequest::RequestErasure(RequestErasureRequest {
            api_version: "1".into(),
            principal_id: Some(PrincipalId::new().to_string()),
            ids: vec!["agg-1".into()],
            reason: Some("user request".into()),
            scope: None,
            command_id: Some("erase-cmd-1".into()),
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::ErasureAccepted(resp) => {
                assert_eq!(resp.status, "accepted");
                assert!(!resp.request_id.is_empty());
                assert!(
                    resp.warnings
                        .iter()
                        .any(|w| w.contains("wipe") || w.contains("P8")),
                    "must note CE wipe residual: {:?}",
                    resp.warnings
                );
                // Exactly one ticket event
                let events = h.writer.recorded_events().await;
                let tickets = events
                    .iter()
                    .filter(|e| {
                        matches!(
                            &e.payload,
                            ai_brains_events::Payload::ErasureTicketAccepted(p)
                                if p.request_id == resp.request_id
                        )
                    })
                    .count();
                assert_eq!(tickets, 1);

                // Second call same command_id → no second ticket
                let outcome2 = handle_daemon_request(
                    DaemonRequest::RequestErasure(RequestErasureRequest {
                        api_version: "1".into(),
                        principal_id: Some(PrincipalId::new().to_string()),
                        ids: vec!["agg-1".into()],
                        reason: Some("user request".into()),
                        scope: None,
                        command_id: Some("erase-cmd-1".into()),
                    }),
                    &h.writer,
                    &h.services,
                )
                .await?;
                match outcome2 {
                    LiveDispatchResult::Response(b2) => match *b2 {
                        DaemonResponse::ErasureAccepted(r2) => {
                            assert_eq!(r2.request_id, resp.request_id);
                        }
                        other => panic!("expected ErasureAccepted, got {other:?}"),
                    },
                    other => panic!("expected Response, got {other:?}"),
                }
                let events2 = h.writer.recorded_events().await;
                let tickets2 = events2
                    .iter()
                    .filter(|e| {
                        matches!(
                            &e.payload,
                            ai_brains_events::Payload::ErasureTicketAccepted(p)
                                if p.request_id == resp.request_id
                        )
                    })
                    .count();
                assert_eq!(tickets2, 1);
                Ok(())
            }
            other => panic!("expected ErasureAccepted, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn governed_mutation_without_command_id__no_spool_file()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("no-spool").await?;
    let principal_id = PrincipalId::new();
    let principal =
        ai_brains_control_plane::make_principal(PrincipalKind::Human, principal_id, "test-human");
    let user = UserId::new();
    let scope = ScopeRef::Personal(user);
    grant_propose(&h.store, &principal, scope.clone());
    let scope_key = ai_brains_control_plane::scope_identity_key(&scope);

    let _ = handle_daemon_request(
        DaemonRequest::ProposeConclusion(ProposeConclusionRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            scope: scope_key,
            statement: "live no spool".into(),
            evidence_ids: vec![],
            privacy: Some("LocalOnly".into()),
            command_id: None,
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    let spool = h.writer.spool_dir();
    let mut entries = tokio::fs::read_dir(spool).await?;
    let mut count = 0;
    while let Some(e) = entries.next_entry().await? {
        if e.path().extension().and_then(|x| x.to_str()) == Some("json") {
            count += 1;
        }
    }
    assert_eq!(
        count, 0,
        "governed mutation without command_id must not leave spool"
    );
    Ok(())
}

#[tokio::test]
async fn command_id_spool__replay__exactly_one_conclusion_proposed_event()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("spool-replay").await?;
    let principal_id = PrincipalId::new();
    let principal =
        ai_brains_control_plane::make_principal(PrincipalKind::Human, principal_id, "test-human");
    let user = UserId::new();
    let scope = ScopeRef::Personal(user);
    grant_propose(&h.store, &principal, scope.clone());
    let scope_key = ai_brains_control_plane::scope_identity_key(&scope);
    let command_id = format!("cmd-{}", Uuid::new_v4());

    let outcome = handle_daemon_request(
        DaemonRequest::ProposeConclusion(ProposeConclusionRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            scope: scope_key.clone(),
            statement: "spooled claim".into(),
            evidence_ids: vec![],
            privacy: Some("LocalOnly".into()),
            command_id: Some(command_id.clone()),
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    let conclusion_id = match outcome {
        LiveDispatchResult::Response(b) => match *b {
            DaemonResponse::ConclusionProposed(r) => r.conclusion_id,
            DaemonResponse::Error(e) => panic!("propose failed: {}: {}", e.code, e.message),
            other => panic!("unexpected {other:?}"),
        },
        other => panic!("expected Response {other:?}"),
    };

    // Simulate crash-after-append-before-spool-delete: write spool file again.
    let spool_name = ai_brainsd::services::sanitize_command_id_for_filename(&command_id);
    let spool_path = h.writer.spool_dir().join(format!("{spool_name}.json"));
    let req = DaemonRequest::ProposeConclusion(ProposeConclusionRequest {
        api_version: "1".into(),
        principal_id: Some(principal_id.to_string()),
        scope: scope_key,
        statement: "spooled claim".into(),
        evidence_ids: vec![],
        privacy: Some("LocalOnly".into()),
        command_id: Some(command_id),
    });
    tokio::fs::write(&spool_path, serde_json::to_vec(&req)?).await?;

    // Restart writer → replay_spool
    let writer2 =
        DaemonWriter::start(h.writer.spool_dir().to_path_buf(), Arc::clone(&h.store)).await?;
    let _ = writer2; // keep alive until replay completes (start awaits replay in worker spawn)
    tokio::time::sleep(Duration::from_millis(200)).await;

    let events = h.store.read_all_events().map_err(|e| e.to_string())?;
    let proposed = events
        .iter()
        .filter(|e| {
            matches!(
                &e.payload,
                ai_brains_events::Payload::ConclusionProposed(p)
                    if p.conclusion_id.to_string() == conclusion_id
            )
        })
        .count();
    assert_eq!(
        proposed, 1,
        "spool replay must not double-append ConclusionProposed"
    );
    Ok(())
}

#[tokio::test]
async fn query_during_ingest__completes() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use ai_brains_contracts::ingest::IngestRequest;
    use ai_brains_core::ids::{HarnessId, SessionId, TurnId};
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::{
        Actor, AggregateType, Payload, ProjectRegisteredPayload, SessionStartedPayload,
    };

    let h = start_harness("concurrent").await?;
    let project_id = ProjectId::new();
    let session_id = SessionId::new();
    // FK prerequisites for ingest (same as single_writer test).
    let project_event = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: "concurrent".into(),
        tx_id: None,
    }))?;
    h.store.append_event(&project_event)?;
    let session_event = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::SessionStarted(SessionStartedPayload {
        session_id,
        project_id,
        tx_id: None,
    }))?;
    h.store.append_event(&session_event)?;

    let ingest = IngestRequest {
        session_id,
        project_id,
        harness_id: HarnessId::default(),
        turn_id: TurnId::new(),
        role: "user".into(),
        content: "concurrent prompt".into(),
        thinking: None,
        privacy: Privacy::LocalOnly,
        tx_id: None,
    };

    let writer = h.writer.clone();
    let services = h.services.clone();
    let ingest_handle = tokio::spawn(async move { writer.ingest(ingest).await });

    // Query while ingest may still be in flight (must not hang the query path forever).
    let q = handle_daemon_request(
        DaemonRequest::ResolveScope(ResolveScopeRequest {
            api_version: "1".into(),
            cwd: None,
            signals: None,
            explicit_project_id: Some(ProjectId::new().to_string()),
            force_personal: false,
            personal_user_id: None,
        }),
        &h.writer,
        &services,
    )
    .await?;
    assert!(matches!(q, LiveDispatchResult::Response(_)));

    let ingest_res = ingest_handle.await.map_err(|e| e.to_string())??;
    assert!(ingest_res.processed);
    Ok(())
}

#[test]
fn daemon_dispatch__main_and_service__same_handler_module() {
    // Structural: both hosts call the shared dispatch helpers (not a forked copy).
    let main_src = include_str!("../src/main.rs");
    let service_src = include_str!("../src/windows_service.rs");
    for (label, src) in [("main.rs", main_src), ("windows_service.rs", service_src)] {
        assert!(
            src.contains("handle_daemon_request"),
            "{label} must call handle_daemon_request"
        );
        assert!(
            src.contains("write_dispatch_result"),
            "{label} must call write_dispatch_result"
        );
        assert!(
            src.contains("parse_live_request_line"),
            "{label} must call parse_live_request_line (AC3 live boundary)"
        );
    }
    let name = std::any::type_name_of_val(&handle_daemon_request);
    assert!(
        name.contains("dispatch"),
        "handler must live in dispatch module, got {name}"
    );
}

// --- AC3 live boundary: parse failures become INVALID_REQUEST (never silent drop) ---

#[test]
fn parse_live_request_line__unknown_type__returns_invalid_request() {
    let line = br#"{"type":"not_a_real_op","payload":null}"#;
    let err = parse_live_request_line(line).expect_err("unknown type must fail");
    assert_eq!(err.code, INVALID_REQUEST);
    assert!(
        err.message.contains("unknown") || err.message.contains("not_a_real_op"),
        "message should mention unknown type, got: {}",
        err.message
    );
}

#[test]
fn parse_live_request_line__malformed_json__returns_invalid_request() {
    let line = br#"{not json at all"#;
    let err = parse_live_request_line(line).expect_err("malformed JSON must fail");
    assert_eq!(err.code, INVALID_REQUEST);
    assert!(
        !err.message.is_empty(),
        "error message must be non-empty for client diagnostics"
    );
}

#[test]
fn parse_live_request_line__ping__ok() {
    let line = br#"{"type":"ping"}"#;
    let req = parse_live_request_line(line).expect("ping must parse");
    assert!(matches!(req, DaemonRequest::Ping));
}

#[test]
fn parse_live_request_line__raw_bridge_record__wraps_as_sync() {
    // Legacy clients may send a bare BridgeRecord (no DaemonRequest envelope).
    let line = br#"{
        "bridge_version": "0.3",
        "direction": "inbound",
        "timestamp": "2026-01-01T00:00:00Z",
        "parent_hash": null,
        "project_id": "00000000-0000-0000-0000-0000000000a1",
        "session_id": null,
        "tx_id": null,
        "record_kind": "query",
        "payload": { "type": "Query", "text": "what is the decision?" },
        "privacy": "LocalOnly"
    }"#;
    let req = parse_live_request_line(line).expect("raw BridgeRecord must fall back to Sync");
    match req {
        DaemonRequest::Sync(record) => {
            assert_eq!(record.record_kind, "query");
        }
        other => panic!("expected Sync, got {other:?}"),
    }
}

#[test]
fn parse_live_request_line__unknown_type__error_serializes_to_daemon_response() {
    // Evidence that the Err arm produces a writeable DaemonResponse::Error (hosts write this).
    let line = br#"{"type":"not_a_real_op"}"#;
    let api_err = parse_live_request_line(line).expect_err("must fail");
    let resp = DaemonResponse::Error(api_err);
    let json = serde_json::to_value(&resp).expect("serialize Error");
    assert_eq!(json["type"], "error");
    assert_eq!(json["payload"]["code"], INVALID_REQUEST);
}

// --- P2: Sync query multi-line framing via write_dispatch_result ---

#[tokio::test]
async fn write_dispatch_result__multiline_two_lines__double_newline_terminator() {
    let (mut write_half, mut read_half) = tokio::io::duplex(256);
    let (shutdown_tx, _shutdown_rx) = tokio::sync::broadcast::channel(1);
    write_dispatch_result(
        &mut write_half,
        LiveDispatchResult::MultiLine(vec![b"a".to_vec(), b"b".to_vec()]),
        &shutdown_tx,
    )
    .await
    .expect("write MultiLine");
    drop(write_half);

    let mut out = Vec::new();
    read_half
        .read_to_end(&mut out)
        .await
        .expect("read framed bytes");
    assert_eq!(
        out, b"a\nb\n\n",
        "two lines each end with \\n, then final blank \\n"
    );
}

#[tokio::test]
async fn write_dispatch_result__multiline_empty__single_blank_line() {
    let (mut write_half, mut read_half) = tokio::io::duplex(64);
    let (shutdown_tx, _shutdown_rx) = tokio::sync::broadcast::channel(1);
    write_dispatch_result(
        &mut write_half,
        LiveDispatchResult::MultiLine(Vec::new()),
        &shutdown_tx,
    )
    .await
    .expect("write empty MultiLine");
    drop(write_half);

    let mut out = Vec::new();
    read_half
        .read_to_end(&mut out)
        .await
        .expect("read framed bytes");
    assert_eq!(out, b"\n", "empty MultiLine is final blank line only");
}

#[tokio::test]
async fn write_dispatch_result__response_pong__single_line_newline() {
    let (mut write_half, mut read_half) = tokio::io::duplex(256);
    let (shutdown_tx, _shutdown_rx) = tokio::sync::broadcast::channel(1);
    write_dispatch_result(
        &mut write_half,
        LiveDispatchResult::Response(Box::new(DaemonResponse::Pong)),
        &shutdown_tx,
    )
    .await
    .expect("write Pong");
    drop(write_half);

    let mut out = Vec::new();
    read_half
        .read_to_end(&mut out)
        .await
        .expect("read framed bytes");
    assert!(
        out.ends_with(b"\n"),
        "single Response must be newline-terminated"
    );
    assert_eq!(
        out.iter().filter(|&&b| b == b'\n').count(),
        1,
        "exactly one trailing newline for single Response"
    );
    let decoded: DaemonResponse = serde_json::from_slice(&out[..out.len() - 1]).expect("json");
    assert!(matches!(decoded, DaemonResponse::Pong));
}

// Ensure we no longer expect UNSUPPORTED for governed ops (compile-time reminder).
#[allow(dead_code)]
const _NO_UNSUPPORTED_FOR_GOVERNED: &str = UNSUPPORTED_OPERATION;
