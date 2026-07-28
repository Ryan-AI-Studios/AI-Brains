//! `ai-brainsd` library: single-writer vault + named-pipe dispatch (T158/T159).
//!
//! # CQRS (T159)
//!
//! - **Mutations** (Ingest/Sync + governed propose/resolve/erasure) go through
//!   [`DaemonWriter`]'s bounded mpsc queue (single consumer).
//! - **Queries** (legacy Sync query + governed scope/briefings/knowledge) use
//!   shared vault reads off the write queue via [`services::GovernedServices`].
//!
//! # Spool
//!
//! - Legacy Ingest/Sync: always uuid-named spool files.
//! - Governed mutations: spool **only** when `command_id` is present (filename
//!   sanitized from command_id). Replay hits control-plane detect-already-done.

use ai_brains_capture::{CaptureContext, CaptureService, CaptureSink};
use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_contracts::ingest::{IngestRequest, IngestResponse};
use ai_brains_control_plane::StorePorts;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_events::Envelope;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub mod dispatch;
pub mod instance_guard;
pub mod pipe_error;
pub mod services;

#[cfg(windows)]
pub mod pipe_security;

#[cfg(windows)]
pub mod windows_service;

use services::{GovernedMutation, process_governed_mutation, sanitize_command_id_for_filename};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

struct DaemonStoreSink {
    store: Arc<dyn EventStore>,
    last_error: Option<String>,
}

impl CaptureSink for DaemonStoreSink {
    fn append(&mut self, envelope: Envelope) {
        if let Err(e) = self.store.append_event(&envelope) {
            self.last_error = Some(e.to_string());
        }
    }
}

enum WriterMessage {
    Ingest {
        request: IngestRequest,
        spool_path: PathBuf,
        reply: oneshot::Sender<Result<IngestResponse, BoxError>>,
    },
    Sync {
        record: BridgeRecord,
        spool_path: PathBuf,
        reply: oneshot::Sender<Result<(), BoxError>>,
    },
    Governed {
        op: GovernedMutation,
        spool_path: Option<PathBuf>,
        reply: oneshot::Sender<Result<DaemonResponse, BoxError>>,
    },
}

/// Single-writer facade for the daemon vault (mutations + legacy query helpers).
#[derive(Clone)]
pub struct DaemonWriter {
    sender: mpsc::Sender<WriterMessage>,
    event_store: Arc<SqliteEventStore>,
    spool_dir: PathBuf,
}

impl DaemonWriter {
    pub async fn start(
        spool_dir: PathBuf,
        event_store: Arc<SqliteEventStore>,
    ) -> Result<Self, BoxError> {
        fs::create_dir_all(&spool_dir).await?;

        let (sender, mut receiver) = mpsc::channel(64);
        let worker_store_arc = Arc::clone(&event_store);
        let worker_spool_dir = spool_dir.clone();

        tokio::spawn(async move {
            let service = CaptureService::new();
            let worker_dyn = Arc::clone(&worker_store_arc) as Arc<dyn EventStore>;
            // StorePorts for governed mutations on the writer task only.
            let ports = StorePorts::from_store(SqliteEventStore::new(
                worker_store_arc.connection().clone(),
            ));

            if let Err(e) = replay_spool(&worker_spool_dir, &worker_dyn, &service, &ports).await {
                eprintln!("Failed to replay spool on daemon startup: {}", e);
            }

            while let Some(message) = receiver.recv().await {
                match message {
                    WriterMessage::Ingest {
                        request,
                        spool_path,
                        reply,
                    } => {
                        let result =
                            process_ingest(&service, &worker_dyn, request, Some(spool_path)).await;
                        let _ = reply.send(result);
                    }
                    WriterMessage::Sync {
                        record,
                        spool_path,
                        reply,
                    } => {
                        let result =
                            process_sync(&service, &worker_dyn, record, Some(spool_path)).await;
                        let _ = reply.send(result);
                    }
                    WriterMessage::Governed {
                        op,
                        spool_path,
                        reply,
                    } => {
                        let result = process_governed_on_writer(&ports, op, spool_path).await;
                        let _ = reply.send(result);
                    }
                }
            }
        });

        Ok(Self {
            sender,
            event_store,
            spool_dir,
        })
    }

    pub async fn ingest(&self, request: IngestRequest) -> Result<IngestResponse, BoxError> {
        let spool_path = self.spool_dir.join(format!("{}.json", Uuid::new_v4()));
        let payload = serde_json::to_vec(&DaemonRequest::Ingest(request.clone()))?;
        fs::write(&spool_path, payload).await?;

        let (reply_tx, reply_rx) = oneshot::channel();
        self.sender
            .send(WriterMessage::Ingest {
                request,
                spool_path,
                reply: reply_tx,
            })
            .await
            .map_err(|_| "daemon queue closed")?;

        reply_rx.await.map_err(|_| -> BoxError {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "daemon worker dropped",
            ))
        })?
    }

    pub async fn sync(&self, record: BridgeRecord) -> Result<(), BoxError> {
        let spool_path = self.spool_dir.join(format!("{}.json", Uuid::new_v4()));
        let payload = serde_json::to_vec(&DaemonRequest::Sync(record.clone()))?;
        fs::write(&spool_path, payload).await?;

        let (reply_tx, reply_rx) = oneshot::channel();
        self.sender
            .send(WriterMessage::Sync {
                record,
                spool_path,
                reply: reply_tx,
            })
            .await
            .map_err(|_| "daemon queue closed")?;

        reply_rx.await.map_err(|_| -> BoxError {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "daemon worker dropped",
            ))
        })?
    }

    /// Enqueue a governed mutation on the single-writer task.
    ///
    /// When `command_id` is present on the request, spools the full
    /// [`DaemonRequest`] JSON first (at-least-once). Deletes spool on success.
    pub async fn enqueue_governed(
        &self,
        daemon_request: DaemonRequest,
        op: GovernedMutation,
        command_id: Option<&str>,
    ) -> Result<DaemonResponse, BoxError> {
        let spool_path = if let Some(cid) = command_id.filter(|c| !c.trim().is_empty()) {
            let name = sanitize_command_id_for_filename(cid);
            let path = self.spool_dir.join(format!("{name}.json"));
            let payload = serde_json::to_vec(&daemon_request)?;
            fs::write(&path, payload).await?;
            Some(path)
        } else {
            None
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        self.sender
            .send(WriterMessage::Governed {
                op,
                spool_path,
                reply: reply_tx,
            })
            .await
            .map_err(|_| "daemon queue closed")?;

        reply_rx.await.map_err(|_| -> BoxError {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "daemon worker dropped",
            ))
        })?
    }

    pub async fn recorded_events(&self) -> Vec<Envelope> {
        self.event_store.read_all_events().unwrap_or_else(|e| {
            eprintln!("Failed to read events from event store: {}", e);
            Vec::new()
        })
    }

    pub async fn query_memories(
        &self,
        query: &str,
        project_id: Option<ai_brains_core::ids::ProjectId>,
        session_id: Option<ai_brains_core::ids::SessionId>,
    ) -> Result<Vec<ai_brains_retrieval::RecallHit>, BoxError> {
        let conn = self.event_store.connection();
        let graph_search: Option<&ai_brains_retrieval::GraphSearch> = None;
        let hits = ai_brains_retrieval::recall(
            conn,
            graph_search,
            query,
            5,
            ai_brains_retrieval::RecallOptions {
                project_id,
                session_id,
                semantic: false,
                graph_boost: 0.0,
                graph_hop_depth: 0,
                no_bridge: true,
                quiet: true,
            },
        )?;
        Ok(hits)
    }

    pub fn spool_dir(&self) -> &Path {
        &self.spool_dir
    }

    pub fn event_store(&self) -> &Arc<SqliteEventStore> {
        &self.event_store
    }
}

async fn process_governed_on_writer(
    ports: &StorePorts,
    op: GovernedMutation,
    spool_path: Option<PathBuf>,
) -> Result<DaemonResponse, BoxError> {
    // Blocking CP/store work on the writer task (single-threaded consumer).
    let result = process_governed_mutation(ports, op);
    match &result {
        Ok(DaemonResponse::Error(_)) => {
            // Domain error: keep spool for retry when command_id was set.
            // (Client may re-send; leaving spool is safer than losing the command.)
            // Actually for POLICY_DENIED etc. leaving spool forever is bad.
            // Spec: delete on success. Policy denial is a terminal success of the
            // "process" attempt from spool's perspective for live path.
            // For spool replay of permanent denials, replaying forever is wrong.
            // Delete spool on any completed handling (Ok with any DaemonResponse).
            if let Some(path) = spool_path {
                let _ = fs::remove_file(path).await;
            }
        }
        Ok(_) => {
            if let Some(path) = spool_path {
                let _ = fs::remove_file(path).await;
            }
        }
        Err(_) => {
            // Infra failure — leave spool for restart replay.
        }
    }
    result
}

async fn replay_spool(
    spool_dir: &Path,
    store: &Arc<dyn EventStore>,
    service: &CaptureService,
    ports: &StorePorts,
) -> Result<(), BoxError> {
    let mut entries = fs::read_dir(spool_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path).await?;
        let request: DaemonRequest = serde_json::from_str(&content)?;
        match request {
            DaemonRequest::Ingest(ingest) => {
                process_ingest(service, store, ingest, Some(path)).await?;
            }
            DaemonRequest::Sync(record) => {
                process_sync(service, store, record, Some(path)).await?;
            }
            DaemonRequest::ProposeConclusion(req) => {
                let op = GovernedMutation::ProposeConclusion(req);
                let _ = process_governed_on_writer(ports, op, Some(path)).await?;
            }
            DaemonRequest::ProposeDecision(req) => {
                let op = GovernedMutation::ProposeDecision(req);
                let _ = process_governed_on_writer(ports, op, Some(path)).await?;
            }
            DaemonRequest::ResolveReviewItem(req) => {
                let op = GovernedMutation::ResolveReviewItem(req);
                let _ = process_governed_on_writer(ports, op, Some(path)).await?;
            }
            DaemonRequest::RequestErasure(req) => {
                let op = GovernedMutation::RequestErasure(req);
                let _ = process_governed_on_writer(ports, op, Some(path)).await?;
            }
            // Queries / control — not durable spool work. Drop without panic.
            DaemonRequest::Ping
            | DaemonRequest::Shutdown
            | DaemonRequest::ResolveScope(_)
            | DaemonRequest::ProjectBriefing(_)
            | DaemonRequest::PersonalBriefing(_)
            | DaemonRequest::QueryKnowledge(_)
            | DaemonRequest::InspectEvidence(_)
            | DaemonRequest::InspectSource(_)
            | DaemonRequest::ListReviewItems(_) => {
                let _ = fs::remove_file(path).await;
            }
        }
    }
    Ok(())
}

async fn process_ingest(
    service: &CaptureService,
    store: &Arc<dyn EventStore>,
    request: IngestRequest,
    spool_path: Option<PathBuf>,
) -> Result<IngestResponse, BoxError> {
    let mut sink = DaemonStoreSink {
        store: Arc::clone(store),
        last_error: None,
    };
    let outcome = service.ingest_request(request, CaptureContext::default(), &mut sink)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to persist ingested turn: {}", err).into());
    }

    if let Some(path) = spool_path {
        let _ = fs::remove_file(path).await;
    }

    let event_id = outcome
        .primary_event()
        .map(|e| e.event_id.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    Ok(IngestResponse {
        event_id,
        processed: true,
    })
}

async fn process_sync(
    service: &CaptureService,
    store: &Arc<dyn EventStore>,
    record: BridgeRecord,
    spool_path: Option<PathBuf>,
) -> Result<(), BoxError> {
    if record.direction != ai_brains_contracts::bridge::BridgeDirection::Inbound {
        if let Some(path) = spool_path {
            let _ = fs::remove_file(path).await;
        }
        return Ok(());
    }

    // Lineage Verification
    let expected_last: Option<String> = store
        .get_sync_state("last_inbound_hash")
        .map_err(|e| format!("Store error: {}", e))?;

    if let Some(actual_parent) = &record.parent_hash {
        if let Some(expected_last_hash) = &expected_last {
            if actual_parent != expected_last_hash {
                let msg = format!(
                    "Lineage verification failed: parent_hash mismatch. Expected {}, got {}",
                    expected_last_hash, actual_parent
                );
                if let Some(path) = spool_path {
                    let _ = fs::remove_file(path).await;
                }
                return Err(msg.into());
            }
        } else {
            let msg = format!(
                "Bridge record rejected: non-null parent_hash {} but state has no previous inbound hash",
                actual_parent
            );
            if let Some(path) = spool_path {
                let _ = fs::remove_file(path).await;
            }
            return Err(msg.into());
        }
    }

    // Hash computation for updating state
    use sha2::{Digest, Sha256};
    let json_for_hash = serde_json::to_string(&record).unwrap_or_else(|e| {
        eprintln!("Failed to serialize BridgeRecord for hash: {}", e);
        String::new()
    });
    let mut hasher = Sha256::new();
    hasher.update(json_for_hash.as_bytes());
    let hash_hex = hex::encode(hasher.finalize());

    store
        .set_sync_state("last_inbound_hash", &hash_hex)
        .map_err(|e| format!("Store error: {}", e))?;

    let mut sink = DaemonStoreSink {
        store: Arc::clone(store),
        last_error: None,
    };

    let role = match record.record_kind.to_lowercase().as_str() {
        "user" | "prompt" => "user",
        _ => "assistant",
    };

    let content = record.formatted_payload();

    // Parse string IDs from interchange format into typed IDs.
    use std::str::FromStr;
    let project_id = ai_brains_core::ids::ProjectId::from_str(&record.project_id)
        .unwrap_or_else(|_| ai_brains_core::ids::ProjectId::new());
    let session_id = match &record.session_id {
        Some(s) => ai_brains_core::ids::SessionId::from_str(s)
            .unwrap_or_else(|_| ai_brains_core::ids::SessionId::new()),
        None => ai_brains_core::ids::SessionId::new(),
    };
    let tx_id = record
        .tx_id
        .clone()
        .map(ai_brains_core::ids::TransactionId::new);

    let session_privacy = store
        .get_session_privacy(&session_id.to_string())
        .map_err(|e| format!("Store error: {}", e))?
        .unwrap_or(record.privacy);
    let combined_privacy = record.privacy.combine(session_privacy);

    // Handle specific structured payloads
    if record.record_kind == "verify_outcome"
        && let Ok(outcome) = serde_json::from_value::<ai_brains_events::VerifyOutcomeRecordedPayload>(
            record.payload_value(),
        )
    {
        let event = ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::System,
            uuid::Uuid::new_v4(),
            ai_brains_events::Actor::System,
            combined_privacy,
        )
        .build(ai_brains_events::Payload::VerifyOutcomeRecorded(outcome))
        .map_err(|e| format!("Event build error: {}", e))?;

        store
            .append_event(&event)
            .map_err(|e| format!("Event append error: {}", e))?;

        if let Some(path) = spool_path {
            let _ = fs::remove_file(path).await;
        }
        return Ok(());
    }

    let request = IngestRequest {
        session_id,
        project_id,
        harness_id: ai_brains_core::ids::HarnessId::default(),
        turn_id: ai_brains_core::ids::TurnId::new(),
        role: role.to_string(),
        content,
        thinking: None,
        privacy: combined_privacy,
        tx_id,
    };

    service.ingest_request(request, CaptureContext::default(), &mut sink)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to persist synced record: {}", err).into());
    }

    if let Some(path) = spool_path {
        let _ = fs::remove_file(path).await;
    }

    Ok(())
}
