#![allow(clippy::disallowed_methods)]

use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, SessionId, TurnId};
use ai_brains_core::privacy::Privacy;
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

fn request(content: &str) -> IngestRequest {
    IngestRequest {
        session_id: SessionId::new(),
        project_id: ai_brains_core::ids::ProjectId::new(),
        harness_id: HarnessId::new(),
        turn_id: TurnId::new(),
        role: "user".to_string(),
        content: content.to_string(),
        privacy: Privacy::CloudOk,
        thinking: None,
    }
}

#[tokio::test]
async fn single_writer_serializes_parallel_ingest(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let spool_dir = unique_spool_dir("parallel");
    let writer = DaemonWriter::start(spool_dir.clone()).await?;

    let a = writer.clone();
    let b = writer.clone();
    let c = writer.clone();

    let first = tokio::spawn(async move { a.ingest(request("one")).await });
    let second = tokio::spawn(async move { b.ingest(request("two")).await });
    let third = tokio::spawn(async move { c.ingest(request("three")).await });

    first.await??;
    second.await??;
    third.await??;

    let events = writer.recorded_events().await;
    assert_eq!(events.len(), 3);

    let _ = tokio::fs::remove_dir_all(spool_dir).await;
    Ok(())
}
