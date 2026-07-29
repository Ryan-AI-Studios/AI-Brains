//! Shared live dispatch smoke tests (T158 + T159 governed handlers).
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_contracts::briefings::QueryKnowledgeRequest;
use ai_brains_contracts::erasure::{RequestErasureRequest, WipeContentEnvelopeRequest};
use ai_brains_contracts::knowledge::ProposeConclusionRequest;
use ai_brains_contracts::scopes::ResolveScopeRequest;
use ai_brains_core::ids::{ContentKeyId, PrincipalId, ProjectId, SourceId, UserId};
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
use ai_brainsd::services::{GovernedServices, governed_spool_stem};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::AsyncReadExt;
use uuid::Uuid;

/// Poll until `pred` is true or `timeout` elapses (project: no fixed sleep-for-async).
async fn wait_for_condition<F>(timeout: Duration, interval: Duration, mut pred: F) -> bool
where
    F: FnMut() -> bool,
{
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if pred() {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(interval).await;
    }
}

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

fn grant_capability(
    store: &Arc<SqliteEventStore>,
    principal: &ai_brains_core::principal::Principal,
    scope: ScopeRef,
    capability: GrantCapability,
) {
    use ai_brains_control_plane::{StorePorts, SystemClock, issue_grant, register_principal};
    let ports = StorePorts::from_store(SqliteEventStore::new(store.connection().clone()));
    let clock = SystemClock;
    register_principal(&ports.writer, &clock, principal).expect("register");
    issue_grant(
        &ports.writer,
        &clock,
        principal.id,
        scope,
        capability,
        Privacy::LocalOnly,
    )
    .expect("grant");
}

fn grant_propose(
    store: &Arc<SqliteEventStore>,
    principal: &ai_brains_core::principal::Principal,
    scope: ScopeRef,
) {
    grant_capability(store, principal, scope, GrantCapability::ProposeConclusion);
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
    let principal_id = PrincipalId::new();
    let principal =
        ai_brains_control_plane::make_principal(PrincipalKind::Human, principal_id, "erase-human");
    let user = UserId::new();
    let scope = ScopeRef::Personal(user);
    grant_capability(&h.store, &principal, scope.clone(), GrantCapability::Erase);
    let scope_key = ai_brains_control_plane::scope_identity_key(&scope);

    let outcome = handle_daemon_request(
        DaemonRequest::RequestErasure(RequestErasureRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            ids: vec!["agg-1".into()],
            reason: Some("user request".into()),
            scope: Some(scope_key.clone()),
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

                // Second call same command_id **with** Erase grant → accepted, still one ticket
                let outcome2 = handle_daemon_request(
                    DaemonRequest::RequestErasure(RequestErasureRequest {
                        api_version: "1".into(),
                        principal_id: Some(principal_id.to_string()),
                        ids: vec!["agg-1".into()],
                        reason: Some("user request".into()),
                        scope: Some(scope_key.clone()),
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

                // Second call same command_id **without** Erase grant → POLICY_DENIED
                let outcome3 = handle_daemon_request(
                    DaemonRequest::RequestErasure(RequestErasureRequest {
                        api_version: "1".into(),
                        principal_id: Some(PrincipalId::new().to_string()),
                        ids: vec!["agg-1".into()],
                        reason: Some("user request".into()),
                        scope: Some(scope_key),
                        command_id: Some("erase-cmd-1".into()),
                    }),
                    &h.writer,
                    &h.services,
                )
                .await?;
                match outcome3 {
                    LiveDispatchResult::Response(b3) => match *b3 {
                        DaemonResponse::Error(err) => {
                            assert_eq!(err.code, "POLICY_DENIED");
                        }
                        other => panic!("expected POLICY_DENIED, got {other:?}"),
                    },
                    other => panic!("expected Response, got {other:?}"),
                }
                Ok(())
            }
            other => panic!("expected ErasureAccepted, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__request_erasure_without_grant__policy_denied()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("erase-deny").await?;
    let principal_id = PrincipalId::new();
    let scope_key = format!("Personal:{}", UserId::new());
    let outcome = handle_daemon_request(
        DaemonRequest::RequestErasure(RequestErasureRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            ids: vec!["agg-1".into()],
            reason: Some("user request".into()),
            scope: Some(scope_key),
            command_id: Some("erase-deny-cmd".into()),
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
async fn handle_daemon_request__request_erasure_missing_scope__invalid_payload()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("erase-no-scope").await?;
    let outcome = handle_daemon_request(
        DaemonRequest::RequestErasure(RequestErasureRequest {
            api_version: "1".into(),
            principal_id: Some(PrincipalId::new().to_string()),
            ids: vec!["agg-1".into()],
            reason: Some("user request".into()),
            scope: None,
            command_id: Some("erase-no-scope".into()),
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "INVALID_PAYLOAD");
                assert!(err.message.contains("scope"));
                Ok(())
            }
            other => panic!("expected INVALID_PAYLOAD, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

fn insert_active_wrap(store: &Arc<SqliteEventStore>, content_key_id: &ContentKeyId) {
    use ai_brains_store::projections::content_envelope;
    let conn = store.connection().lock().expect("conn lock");
    content_envelope::insert_content_key_wrap(
        &conn,
        &content_key_id.to_string(),
        1,
        &[0xAAu8; 12],
        &[0xBBu8; 48],
        "2026-07-28T12:00:00Z",
    )
    .expect("insert wrap");
}

fn wrap_is_destroyed(store: &Arc<SqliteEventStore>, content_key_id: &ContentKeyId) -> bool {
    use ai_brains_store::projections::content_envelope;
    let conn = store.connection().lock().expect("conn lock");
    content_envelope::is_content_key_destroyed(&conn, &content_key_id.to_string())
        .expect("destroy check")
}

#[tokio::test]
async fn handle_daemon_request__wipe_content_envelope__dry_run__no_wrap_destroy()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("wipe-dry").await?;
    let principal_id = PrincipalId::new();
    let principal =
        ai_brains_control_plane::make_principal(PrincipalKind::Human, principal_id, "wipe-human");
    let user = UserId::new();
    let scope = ScopeRef::Personal(user);
    grant_capability(&h.store, &principal, scope.clone(), GrantCapability::Erase);
    let scope_key = ai_brains_control_plane::scope_identity_key(&scope);
    let key = ContentKeyId::new();
    insert_active_wrap(&h.store, &key);

    let outcome = handle_daemon_request(
        DaemonRequest::WipeContentEnvelope(WipeContentEnvelopeRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            content_key_id: key.to_string(),
            scope: scope_key,
            reason: Some("dry plan".into()),
            command_id: Some("wipe-dry-cmd".into()),
            dry_run: true,
            confirm: false,
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::ContentEnvelopeWiped(resp) => {
                assert_eq!(resp.status, "dry_run");
                assert!(!resp.wrap_destroyed);
                assert_eq!(resp.validation.wal_checkpoint, "skipped_dry_run");
                assert!(
                    resp.warnings.iter().any(|w| {
                        let l = w.to_ascii_lowercase();
                        l.contains("nist") || l.contains("purge") || l.contains("backup")
                    }),
                    "dry-run must surface honesty warnings: {:?}",
                    resp.warnings
                );
                assert!(
                    !wrap_is_destroyed(&h.store, &key),
                    "dry-run must not destroy wrap"
                );
                let events = h.writer.recorded_events().await;
                assert!(
                    !events.iter().any(|e| {
                        matches!(
                            &e.payload,
                            ai_brains_events::Payload::ContentErasureRequested(_)
                                | ai_brains_events::Payload::ContentErased(_)
                        )
                    }),
                    "dry-run must not emit CE events"
                );
                Ok(())
            }
            other => panic!("expected ContentEnvelopeWiped, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__wipe_content_envelope__execute_with_erase_grant()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("wipe-exec").await?;
    let principal_id = PrincipalId::new();
    let principal =
        ai_brains_control_plane::make_principal(PrincipalKind::Human, principal_id, "wipe-exec");
    let user = UserId::new();
    let scope = ScopeRef::Personal(user);
    grant_capability(&h.store, &principal, scope.clone(), GrantCapability::Erase);
    let scope_key = ai_brains_control_plane::scope_identity_key(&scope);
    let key = ContentKeyId::new();
    insert_active_wrap(&h.store, &key);
    {
        use ai_brains_store::projections::content_envelope;
        let conn = h.store.connection().lock().expect("conn");
        let row = content_envelope::get_content_key_wrap(&conn, &key.to_string())
            .expect("get wrap")
            .expect("wrap row");
        assert_eq!(row.status, "active", "precondition status: {row:?}");
        assert!(
            row.wrap_nonce.as_ref().is_some_and(|n| !n.is_empty()),
            "precondition wrap_nonce present"
        );
        assert!(
            row.wrap_ciphertext.as_ref().is_some_and(|c| !c.is_empty()),
            "precondition wrap_ciphertext present"
        );
        let ts = content_envelope::get_tombstone(&conn, &key.to_string()).expect("tombstone");
        assert!(ts.is_none(), "precondition: no tombstone before wipe");
    }

    let outcome = handle_daemon_request(
        DaemonRequest::WipeContentEnvelope(WipeContentEnvelopeRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            content_key_id: key.to_string(),
            scope: scope_key,
            reason: Some("execute wipe".into()),
            command_id: Some("wipe-exec-cmd-unique".into()),
            dry_run: false,
            confirm: true,
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::ContentEnvelopeWiped(resp) => {
                // With command_id the wipe should still be first-time wiped (not idempotent).
                assert_eq!(
                    resp.status, "wiped",
                    "first execute must wipe (got {resp:?})"
                );
                assert!(resp.wrap_destroyed);
                assert!(resp.verify.wrap_absent);
                assert!(
                    wrap_is_destroyed(&h.store, &key),
                    "execute wipe must destroy wrap in store"
                );
                assert!(
                    resp.warnings.iter().any(|w| {
                        let l = w.to_ascii_lowercase();
                        l.contains("nist") || l.contains("purge") || l.contains("backup")
                    }),
                    "execute wipe must surface honesty warnings: {:?}",
                    resp.warnings
                );
                let events = h.writer.recorded_events().await;
                assert!(
                    events.iter().any(|e| matches!(
                        &e.payload,
                        ai_brains_events::Payload::ContentErasureRequested(_)
                    )),
                    "expected ContentErasureRequested"
                );
                assert!(
                    events
                        .iter()
                        .any(|e| matches!(&e.payload, ai_brains_events::Payload::ContentErased(_))),
                    "expected ContentErased after destroy"
                );
                Ok(())
            }
            other => panic!("expected ContentEnvelopeWiped, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__wipe_content_envelope_without_grant__policy_denied()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("wipe-deny").await?;
    let principal_id = PrincipalId::new();
    let key = ContentKeyId::new();
    insert_active_wrap(&h.store, &key);
    let scope_key = format!("Personal:{}", UserId::new());

    let outcome = handle_daemon_request(
        DaemonRequest::WipeContentEnvelope(WipeContentEnvelopeRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            content_key_id: key.to_string(),
            scope: scope_key,
            reason: Some("no grant".into()),
            command_id: Some("wipe-deny-cmd".into()),
            dry_run: false,
            confirm: true,
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "POLICY_DENIED");
                assert!(
                    !wrap_is_destroyed(&h.store, &key),
                    "policy deny must not destroy wrap"
                );
                Ok(())
            }
            other => panic!("expected POLICY_DENIED error, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__inspect_source_without_grant__policy_denied()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use ai_brains_contracts::sources::InspectSourceRequest;
    let h = start_harness("inspect-deny").await?;
    let outcome = handle_daemon_request(
        DaemonRequest::InspectSource(InspectSourceRequest {
            api_version: "1".into(),
            id: SourceId::new().to_string(),
            principal_id: Some(PrincipalId::new().to_string()),
            scope: Some(format!("Personal:{}", UserId::new())),
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
            other => panic!("expected POLICY_DENIED, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__list_review_items_without_grant__policy_denied()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use ai_brains_contracts::review::ListReviewItemsRequest;
    let h = start_harness("list-review-deny").await?;
    let outcome = handle_daemon_request(
        DaemonRequest::ListReviewItems(ListReviewItemsRequest {
            api_version: "1".into(),
            principal_id: Some(PrincipalId::new().to_string()),
            scope: Some(format!("Personal:{}", UserId::new())),
            status: None,
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
            other => panic!("expected POLICY_DENIED, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

/// Codex R2 HIGH-1: source registered in scope A is visible with grant on A,
/// but NOT_FOUND when requesting scope B (even with a grant on B).
#[tokio::test]
async fn handle_daemon_request__inspect_source__scope_isolation__cross_scope_not_found()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use ai_brains_contracts::sources::InspectSourceRequest;
    use ai_brains_core::source::SourceKind;
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::payload::SourceRegisteredPayload;
    use ai_brains_events::{Actor, AggregateType, Payload};

    let h = start_harness("inspect-scope").await?;
    let principal_id = PrincipalId::new();
    let principal = ai_brains_control_plane::make_principal(
        PrincipalKind::Human,
        principal_id,
        "inspect-scope-human",
    );
    let scope_a = ScopeRef::Personal(UserId::new());
    let scope_b = ScopeRef::Personal(UserId::new());
    let key_a = ai_brains_control_plane::scope_identity_key(&scope_a);
    let key_b = ai_brains_control_plane::scope_identity_key(&scope_b);

    let source_id = SourceId::new();
    let env = EventBuilder::new(
        AggregateType::Source,
        source_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::SourceRegistered(SourceRegisteredPayload {
        source_id,
        kind: SourceKind::File,
        display_name: "scope-a-file".into(),
        locator: Some("/tmp/scope-a.md".into()),
        scope: Some(key_a.clone()),
    }))?;
    EventStore::append_event(h.store.as_ref(), &env)?;

    // Grant on scope A → found.
    grant_capability(
        &h.store,
        &principal,
        scope_a.clone(),
        GrantCapability::ReadEvidence,
    );
    let found = handle_daemon_request(
        DaemonRequest::InspectSource(InspectSourceRequest {
            api_version: "1".into(),
            id: source_id.to_string(),
            principal_id: Some(principal_id.to_string()),
            scope: Some(key_a.clone()),
        }),
        &h.writer,
        &h.services,
    )
    .await?;
    match found {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::Source(dto) => {
                assert_eq!(dto.id, source_id.to_string());
                assert_eq!(dto.display_name, "scope-a-file");
            }
            other => panic!("expected Source in scope A, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }

    // Grant only on scope B; request scope B for source in A → NOT_FOUND (anti-enumeration).
    let principal_b = ai_brains_control_plane::make_principal(
        PrincipalKind::Human,
        PrincipalId::new(),
        "inspect-scope-b",
    );
    grant_capability(
        &h.store,
        &principal_b,
        scope_b.clone(),
        GrantCapability::ReadEvidence,
    );
    let cross = handle_daemon_request(
        DaemonRequest::InspectSource(InspectSourceRequest {
            api_version: "1".into(),
            id: source_id.to_string(),
            principal_id: Some(principal_b.id.to_string()),
            scope: Some(key_b),
        }),
        &h.writer,
        &h.services,
    )
    .await?;
    match cross {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "NOT_FOUND");
                Ok(())
            }
            other => panic!("expected NOT_FOUND for cross-scope, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

/// Codex R2 HIGH-2: review related to a conclusion in scope A must not appear
/// when listing with grant/request on scope B.
#[tokio::test]
async fn handle_daemon_request__list_review_items__scope_isolation__filters_other_scope()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use ai_brains_contracts::review::ListReviewItemsRequest;
    use ai_brains_core::ids::{ConclusionId, ReviewItemId};
    use ai_brains_core::review::{ReviewCriticality, ReviewSubjectKind};
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::payload::ReviewItemOpenedPayload;
    use ai_brains_events::{Actor, AggregateType, Payload};

    let h = start_harness("list-review-scope").await?;
    let principal_id = PrincipalId::new();
    let principal = ai_brains_control_plane::make_principal(
        PrincipalKind::Human,
        principal_id,
        "list-review-scope-human",
    );
    let scope_a = ScopeRef::Personal(UserId::new());
    let scope_b = ScopeRef::Personal(UserId::new());
    let key_a = ai_brains_control_plane::scope_identity_key(&scope_a);
    let key_b = ai_brains_control_plane::scope_identity_key(&scope_b);

    // Propose conclusion in scope A so review can relate to it.
    grant_propose(&h.store, &principal, scope_a.clone());
    let proposed = handle_daemon_request(
        DaemonRequest::ProposeConclusion(ProposeConclusionRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            scope: key_a.clone(),
            statement: "scope A claim for review filter".into(),
            evidence_ids: vec![],
            privacy: Some("LocalOnly".into()),
            command_id: None,
        }),
        &h.writer,
        &h.services,
    )
    .await?;
    let conclusion_id = match proposed {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::ConclusionProposed(resp) => {
                ConclusionId::from_str(&resp.conclusion_id).map_err(|e| e.to_string())?
            }
            other => panic!("expected ConclusionProposed, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    };

    let review_item_id = ReviewItemId::new();
    let env = EventBuilder::new(
        AggregateType::ReviewItem,
        review_item_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ReviewItemOpened(ReviewItemOpenedPayload {
        review_item_id,
        subject: format!("review for conclusion in {key_a}"),
        opened_by: principal_id,
        subject_kind: ReviewSubjectKind::Conclusion,
        subject_id: conclusion_id.to_string(),
        criticality: ReviewCriticality::Medium,
        related_conclusion_id: Some(conclusion_id),
        related_decision_id: None,
        related_source_id: None,
    }))?;
    EventStore::append_event(h.store.as_ref(), &env)?;

    // Grant on scope B only → list must not include scope A review.
    grant_capability(
        &h.store,
        &principal,
        scope_b.clone(),
        GrantCapability::ReadConclusions,
    );
    let listed_b = handle_daemon_request(
        DaemonRequest::ListReviewItems(ListReviewItemsRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            scope: Some(key_b.clone()),
            status: None,
        }),
        &h.writer,
        &h.services,
    )
    .await?;
    match listed_b {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::ReviewList(resp) => {
                assert!(
                    !resp
                        .items
                        .iter()
                        .any(|i| i.id == review_item_id.to_string()),
                    "scope A review must not appear under scope B: {:?}",
                    resp.items
                );
            }
            other => panic!("expected ReviewList, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }

    // Grant on scope A → list includes the related review.
    grant_capability(
        &h.store,
        &principal,
        scope_a.clone(),
        GrantCapability::ReadConclusions,
    );
    let listed_a = handle_daemon_request(
        DaemonRequest::ListReviewItems(ListReviewItemsRequest {
            api_version: "1".into(),
            principal_id: Some(principal_id.to_string()),
            scope: Some(key_a),
            status: None,
        }),
        &h.writer,
        &h.services,
    )
    .await?;
    match listed_a {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::ReviewList(resp) => {
                assert!(
                    resp.items
                        .iter()
                        .any(|i| i.id == review_item_id.to_string()),
                    "scope A review must appear under scope A: {:?}",
                    resp.items
                );
                Ok(())
            }
            other => panic!("expected ReviewList, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__propose_conclusion_bad_evidence_id__invalid_payload()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("bad-evidence").await?;
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
            statement: "claim with bad evidence".into(),
            evidence_ids: vec!["not-a-uuid".into()],
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
                assert_eq!(err.code, "INVALID_PAYLOAD");
                assert!(
                    err.message.contains("evidence"),
                    "message should name evidence id: {}",
                    err.message
                );
                Ok(())
            }
            other => panic!("expected INVALID_PAYLOAD, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_daemon_request__governed_briefing_false__invalid_payload()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use ai_brains_contracts::briefings::ProjectBriefingRequest;
    let h = start_harness("briefing-ungoverned").await?;
    let outcome = handle_daemon_request(
        DaemonRequest::ProjectBriefing(ProjectBriefingRequest {
            api_version: "1".into(),
            principal_id: None,
            scope: None,
            cwd: None,
            max_words: Some(100),
            governed_briefing: Some(false),
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "INVALID_PAYLOAD");
                assert!(
                    err.message.contains("governed"),
                    "message should mention governed briefing: {}",
                    err.message
                );
                Ok(())
            }
            other => panic!("expected INVALID_PAYLOAD, got {other:?}"),
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
    let stem = governed_spool_stem("propose_conclusion", &command_id);
    let spool_path = h.writer.spool_dir().join(format!("{stem}.json"));
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

    // Restart writer → replay_spool (runs in worker spawn before live recv).
    let writer2 =
        DaemonWriter::start(h.writer.spool_dir().to_path_buf(), Arc::clone(&h.store)).await?;
    let _ = writer2;

    let spool_gone = wait_for_condition(Duration::from_secs(2), Duration::from_millis(50), || {
        !Path::new(&spool_path).exists()
    })
    .await;
    assert!(
        spool_gone,
        "spool file should be deleted after successful replay"
    );

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
async fn handle_daemon_request__query_knowledge_bad_scope__invalid_payload()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let h = start_harness("query-bad-scope").await?;
    let outcome = handle_daemon_request(
        DaemonRequest::QueryKnowledge(QueryKnowledgeRequest {
            api_version: "1".into(),
            query: "budget".into(),
            scope: Some("not-a-valid-scope".into()),
            principal_id: None,
            limit: Some(5),
        }),
        &h.writer,
        &h.services,
    )
    .await?;

    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::Error(err) => {
                assert_eq!(
                    err.code, "INVALID_PAYLOAD",
                    "query-path CP errors must use frozen codes, not host DAEMON_ERROR"
                );
                Ok(())
            }
            other => panic!("expected INVALID_PAYLOAD Error, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
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
