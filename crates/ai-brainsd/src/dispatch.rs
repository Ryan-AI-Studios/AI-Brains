//! Shared live daemon request dispatch (T158/T159).
//!
//! Both the interactive named-pipe loop ([`crate`] main binary) and the Windows
//! service host call [`handle_daemon_request`] so protocol variants get one
//! handler site (do not re-copy arms into `main.rs` / `windows_service.rs`).
//!
//! # CQRS (T159)
//!
//! - Mutations: [`DaemonWriter`] mpsc (single writer)
//! - Queries: [`GovernedServices`] off-queue StorePorts reads
//!
//! # Policy
//!
//! Production policy only (via control-plane services). Errors map to stable
//! `ApiError.code` values (`POLICY_DENIED`, `NOT_FOUND`, …).

use ai_brains_contracts::bridge::{BridgeDirection, BridgePayload, BridgeRecord};
use ai_brains_contracts::response::ApiError;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use std::str::FromStr;

use crate::DaemonWriter;
use crate::services::{GovernedMutation, GovernedServices};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Stable error code when a live IPC line cannot be parsed as a request.
///
/// Used by both pipe hosts so clients never hang on silent drop (T158 AC3).
pub const INVALID_REQUEST: &str = "INVALID_REQUEST";

/// Outcome of handling one live `DaemonRequest`.
///
/// Callers own the transport write (newline framing).
#[derive(Debug)]
pub enum LiveDispatchResult {
    /// Single JSON [`DaemonResponse`] — caller serializes and writes one line + `\n`.
    /// Boxed to keep the enum small (`DaemonResponse` is a large externally tagged union).
    Response(Box<DaemonResponse>),
    /// Shutdown signal — caller triggers `shutdown_tx`; no response body is written today.
    Shutdown,
    /// Sync query path: already-serialized JSON **lines without trailing newlines**.
    /// Caller writes each line + `\n`, then a final blank line (`\n`) as today.
    MultiLine(Vec<Vec<u8>>),
}

/// Parse one line of live IPC (without trailing newline).
///
/// 1. Try `DaemonRequest` (tagged `type` / `payload`).
/// 2. On failure, try raw legacy [`BridgeRecord`] → `DaemonRequest::Sync`.
/// 3. On both failures, return [`INVALID_REQUEST`] — **never silent-drop**.
///
/// Callers MUST write `DaemonResponse::Error` for the `Err` arm so clients do not hang.
pub fn parse_live_request_line(line: &[u8]) -> Result<DaemonRequest, ApiError> {
    match serde_json::from_slice::<DaemonRequest>(line) {
        Ok(req) => Ok(req),
        Err(daemon_err) => match serde_json::from_slice::<BridgeRecord>(line) {
            Ok(record) => Ok(DaemonRequest::Sync(record)),
            Err(bridge_err) => {
                let daemon_msg = daemon_err.to_string();
                let message = if daemon_msg.contains("unknown variant") {
                    format!("unknown request type: {daemon_msg}")
                } else {
                    format!(
                        "failed to parse DaemonRequest or BridgeRecord: {daemon_msg}; bridge: {bridge_err}"
                    )
                };
                Err(ApiError::new(INVALID_REQUEST, message))
            }
        },
    }
}

/// Handle one deserialized live request.
///
/// Legacy: Ping/Shutdown/Ingest/Sync as before.
/// Governed queries: off-queue via `services`.
/// Governed mutations: single-writer queue via `writer`.
pub async fn handle_daemon_request(
    request: DaemonRequest,
    writer: &DaemonWriter,
    services: &GovernedServices,
) -> Result<LiveDispatchResult, BoxError> {
    match request {
        DaemonRequest::Ping => Ok(LiveDispatchResult::Response(Box::new(DaemonResponse::Pong))),
        DaemonRequest::Shutdown => {
            tracing::info!("Shutdown request received via IPC.");
            Ok(LiveDispatchResult::Shutdown)
        }
        DaemonRequest::Ingest(req) => {
            let resp = writer.ingest(req).await?;
            Ok(LiveDispatchResult::Response(Box::new(
                DaemonResponse::Ingest(resp),
            )))
        }
        DaemonRequest::Sync(record) => dispatch_sync(record, writer).await,

        // --- Governed queries (off-queue) ---
        DaemonRequest::ResolveScope(req) => {
            let resp = services.resolve_scope(req)?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }
        DaemonRequest::ProjectBriefing(req) => {
            let resp = services.project_briefing(req)?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }
        DaemonRequest::PersonalBriefing(req) => {
            let resp = services.personal_briefing(req)?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }
        DaemonRequest::QueryKnowledge(req) => {
            let resp = services.query_knowledge(req)?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }
        DaemonRequest::InspectEvidence(req) => {
            let resp = services.inspect_evidence(req)?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }
        DaemonRequest::InspectSource(req) => {
            let resp = services.inspect_source(req)?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }
        DaemonRequest::ListReviewItems(req) => {
            let resp = services.list_review_items(req)?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }

        // --- Governed mutations (writer queue) ---
        DaemonRequest::ProposeConclusion(req) => {
            let command_id = req.command_id.clone();
            let daemon_req = DaemonRequest::ProposeConclusion(req.clone());
            let resp = writer
                .enqueue_governed(
                    daemon_req,
                    GovernedMutation::ProposeConclusion(req),
                    command_id.as_deref(),
                )
                .await?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }
        DaemonRequest::ProposeDecision(req) => {
            let command_id = req.command_id.clone();
            let daemon_req = DaemonRequest::ProposeDecision(req.clone());
            let resp = writer
                .enqueue_governed(
                    daemon_req,
                    GovernedMutation::ProposeDecision(req),
                    command_id.as_deref(),
                )
                .await?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }
        DaemonRequest::ResolveReviewItem(req) => {
            let command_id = req.command_id.clone();
            let daemon_req = DaemonRequest::ResolveReviewItem(req.clone());
            let resp = writer
                .enqueue_governed(
                    daemon_req,
                    GovernedMutation::ResolveReviewItem(req),
                    command_id.as_deref(),
                )
                .await?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }
        DaemonRequest::RequestErasure(req) => {
            let command_id = req.command_id.clone();
            let daemon_req = DaemonRequest::RequestErasure(req.clone());
            let resp = writer
                .enqueue_governed(
                    daemon_req,
                    GovernedMutation::RequestErasure(req),
                    command_id.as_deref(),
                )
                .await?;
            Ok(LiveDispatchResult::Response(Box::new(resp)))
        }
    }
}

async fn dispatch_sync(
    record: BridgeRecord,
    writer: &DaemonWriter,
) -> Result<LiveDispatchResult, BoxError> {
    if record.record_kind == "query" {
        let payload = record.payload_value();
        let query_text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("");

        // T112: pass IDs through as Option so the daemon defaults to unscoped search.
        let project_id = ai_brains_core::ids::ProjectId::from_str(&record.project_id).ok();
        let session_id = record
            .session_id
            .as_ref()
            .and_then(|s| ai_brains_core::ids::SessionId::from_str(s).ok());

        let hits = writer
            .query_memories(query_text, project_id, session_id)
            .await?;
        let timestamp = chrono::Utc::now();
        let mut lines = Vec::with_capacity(hits.len());

        for h in hits {
            let payload = BridgePayload::Insight {
                type_field: "Insight".to_string(),
                memory_id: h.memory_id,
                relevance: h.score.unwrap_or(1.0),
                content: h.content,
            };

            let resp_record = BridgeRecord {
                bridge_version: "0.3".to_string(),
                direction: BridgeDirection::Outbound,
                timestamp,
                parent_hash: None,
                project_id: record.project_id.clone(),
                session_id: record.session_id.clone(),
                tx_id: None,
                record_kind: "insight".to_string(),
                payload,
                privacy: ai_brains_core::privacy::Privacy::LocalOnly,
            };

            let bytes = serde_json::to_vec(&resp_record)?;
            lines.push(bytes);
        }

        Ok(LiveDispatchResult::MultiLine(lines))
    } else {
        writer.sync(record).await?;
        Ok(LiveDispatchResult::Response(Box::new(
            DaemonResponse::Sync { success: true },
        )))
    }
}

/// Write a [`LiveDispatchResult`] to an async stream (shared by main + service).
pub async fn write_dispatch_result<S>(
    server: &mut S,
    result: LiveDispatchResult,
    shutdown_tx: &tokio::sync::broadcast::Sender<()>,
) -> Result<(), BoxError>
where
    S: tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::AsyncWriteExt;

    match result {
        LiveDispatchResult::Response(resp) => {
            let mut payload = serde_json::to_vec(&resp)?;
            payload.push(b'\n');
            server.write_all(&payload).await?;
            Ok(())
        }
        LiveDispatchResult::Shutdown => {
            let _ = shutdown_tx.send(());
            Ok(())
        }
        LiveDispatchResult::MultiLine(lines) => {
            for mut line in lines {
                line.push(b'\n');
                server.write_all(&line).await?;
            }
            server.write_all(b"\n").await?;
            Ok(())
        }
    }
}
