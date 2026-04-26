use ai_brains_capture::{CaptureContext, CaptureService, MemorySink};
use ai_brains_contracts::ingest::{IngestRequest, IngestResponse};
use ai_brains_daemon_api::DaemonRequest;
use ai_brains_events::Envelope;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{mpsc, oneshot, Mutex};
use uuid::Uuid;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

enum WriterMessage {
    Ingest {
        request: IngestRequest,
        spool_path: PathBuf,
        reply: oneshot::Sender<Result<IngestResponse, BoxError>>,
    },
}

#[derive(Clone)]
pub struct DaemonWriter {
    sender: mpsc::Sender<WriterMessage>,
    events: Arc<Mutex<Vec<Envelope>>>,
    spool_dir: PathBuf,
}

impl DaemonWriter {
    pub async fn start(spool_dir: PathBuf) -> Result<Self, BoxError> {
        fs::create_dir_all(&spool_dir).await?;

        let (sender, mut receiver) = mpsc::channel(64);
        let events = Arc::new(Mutex::new(Vec::new()));
        let worker_events = Arc::clone(&events);
        let worker_spool_dir = spool_dir.clone();

        tokio::spawn(async move {
            let service = CaptureService::new();
            replay_spool(&worker_spool_dir, &worker_events, &service)
                .await
                .ok();

            while let Some(message) = receiver.recv().await {
                match message {
                    WriterMessage::Ingest {
                        request,
                        spool_path,
                        reply,
                    } => {
                        let result =
                            process_ingest(&service, &worker_events, request, Some(spool_path))
                                .await;
                        let _ = reply.send(result);
                    }
                }
            }
        });

        Ok(Self {
            sender,
            events,
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

    pub async fn recorded_events(&self) -> Vec<Envelope> {
        self.events.lock().await.clone()
    }

    pub fn spool_dir(&self) -> &Path {
        &self.spool_dir
    }
}

async fn replay_spool(
    spool_dir: &Path,
    events: &Arc<Mutex<Vec<Envelope>>>,
    service: &CaptureService,
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
                process_ingest(service, events, ingest, Some(path)).await?;
            }
        }
    }
    Ok(())
}

async fn process_ingest(
    service: &CaptureService,
    events: &Arc<Mutex<Vec<Envelope>>>,
    request: IngestRequest,
    spool_path: Option<PathBuf>,
) -> Result<IngestResponse, BoxError> {
    let mut sink = MemorySink::default();
    let outcome = service.ingest_request(request, CaptureContext::default(), &mut sink)?;
    events.lock().await.extend(outcome.events.clone());
    if let Some(path) = spool_path {
        let _ = fs::remove_file(path).await;
    }

    Ok(IngestResponse {
        event_id: outcome.events[0].event_id.to_string(),
        processed: true,
    })
}
