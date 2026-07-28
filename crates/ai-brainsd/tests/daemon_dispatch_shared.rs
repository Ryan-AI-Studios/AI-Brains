//! Shared live dispatch smoke tests (T158 Phase C).
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_contracts::briefings::QueryKnowledgeRequest;
use ai_brains_contracts::erasure::RequestErasureRequest;
use ai_brains_contracts::scopes::ResolveScopeRequest;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse, UNSUPPORTED_OPERATION};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::SqliteEventStore;
use ai_brainsd::DaemonWriter;
use ai_brainsd::dispatch::{
    INVALID_REQUEST, LiveDispatchResult, handle_daemon_request, parse_live_request_line,
    write_dispatch_result,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::AsyncReadExt;

fn unique_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    std::env::temp_dir().join(format!("ai-brainsd-dispatch-{name}-{nanos}"))
}

async fn start_writer(
    name: &str,
) -> Result<DaemonWriter, Box<dyn std::error::Error + Send + Sync>> {
    let dir = unique_dir(name);
    std::fs::create_dir_all(&dir)?;
    let db_path = dir.join("vault.db");
    let key = ai_brains_crypto::SqlCipherKey::from_raw(
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
    );
    let conn = VaultConnection::open(db_path, &key)?;
    conn.migrate()?;
    let store = Arc::new(SqliteEventStore::new(conn));
    let writer = DaemonWriter::start(dir.join("spool"), store).await?;
    Ok(writer)
}

#[tokio::test]
async fn daemon_dispatch__legacy_ping__pong() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let writer = start_writer("ping").await?;
    let outcome = handle_daemon_request(DaemonRequest::Ping, &writer).await?;
    match outcome {
        LiveDispatchResult::Response(boxed) if matches!(*boxed, DaemonResponse::Pong) => Ok(()),
        other => panic!("expected Pong, got {other:?}"),
    }
}

#[tokio::test]
async fn daemon_dispatch__resolve_scope__unsupported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let writer = start_writer("resolve").await?;
    let outcome = handle_daemon_request(
        DaemonRequest::ResolveScope(ResolveScopeRequest::default()),
        &writer,
    )
    .await?;
    match outcome {
        LiveDispatchResult::Response(boxed) => match *boxed {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, UNSUPPORTED_OPERATION);
                Ok(())
            }
            other => panic!("expected UNSUPPORTED_OPERATION error, got {other:?}"),
        },
        other => panic!("expected Response, got {other:?}"),
    }
}

#[tokio::test]
async fn daemon_dispatch__governed_variants__unsupported_operation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let writer = start_writer("unsupported-stubs").await?;
    let cases = [
        DaemonRequest::ResolveScope(ResolveScopeRequest::default()),
        DaemonRequest::QueryKnowledge(QueryKnowledgeRequest {
            api_version: ai_brains_contracts::scopes::API_VERSION.to_string(),
            query: "budget".into(),
            scope: None,
            principal_id: None,
            limit: None,
        }),
        DaemonRequest::RequestErasure(RequestErasureRequest {
            api_version: ai_brains_contracts::scopes::API_VERSION.to_string(),
            principal_id: None,
            ids: vec!["agg-1".into()],
            reason: None,
            scope: None,
        }),
    ];
    for request in cases {
        let label = format!("{request:?}");
        let outcome = handle_daemon_request(request, &writer).await?;
        match outcome {
            LiveDispatchResult::Response(boxed) => match *boxed {
                DaemonResponse::Error(err) => {
                    assert_eq!(
                        err.code, UNSUPPORTED_OPERATION,
                        "expected UNSUPPORTED_OPERATION for {label}"
                    );
                }
                other => panic!("expected UNSUPPORTED_OPERATION error for {label}, got {other:?}"),
            },
            other => panic!("expected Response for {label}, got {other:?}"),
        }
    }
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
