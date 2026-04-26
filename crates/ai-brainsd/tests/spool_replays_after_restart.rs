#![allow(clippy::disallowed_methods)]

use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, SessionId, TurnId};
use ai_brains_core::privacy::Privacy;
use ai_brains_daemon_api::DaemonRequest;
use ai_brainsd::DaemonWriter;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn unique_spool_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    std::env::temp_dir().join(format!("ai-brainsd-{name}-{nanos}"))
}

#[tokio::test]
async fn spool_replays_after_restart() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let spool_dir = unique_spool_dir("replay");
    tokio::fs::create_dir_all(&spool_dir).await?;

    let request = IngestRequest {
        session_id: SessionId::new(),
        harness_id: HarnessId::new(),
        turn_id: TurnId::new(),
        role: "user".to_string(),
        content: "replay me".to_string(),
        privacy: Privacy::CloudOk,
        thinking: None,
    };
    let serialized = serde_json::to_string(&DaemonRequest::Ingest(request))?;
    tokio::fs::write(spool_dir.join("pending.json"), serialized).await?;

    let writer = DaemonWriter::start(spool_dir.clone()).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    let events = writer.recorded_events().await;
    assert_eq!(events.len(), 1);
    assert!(!spool_dir.join("pending.json").exists());

    let _ = tokio::fs::remove_dir_all(spool_dir).await;
    Ok(())
}
