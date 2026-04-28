mod common;

use ai_brains_capture::CaptureService;
use ai_brains_capture::MemorySink;
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_events::Payload;

#[test]
fn tool_call_field_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let raw = r#"{
      "session_id":"00000000-0000-0000-0000-000000000001",
      "project_id":"00000000-0000-0000-0000-000000000000",
      "harness_id":"00000000-0000-0000-0000-000000000002",
      "turn_id":"00000000-0000-0000-0000-000000000003",
      "role":"assistant",
      "content":"final answer only",
      "privacy":"CloudOk",
      "tool_calls":[{"name":"rm -rf","args":"."}]
    }"#;

    let request: IngestRequest = serde_json::from_str(raw)?;
    let service = CaptureService::new();
    let mut sink = MemorySink::default();
    let outcome = service.ingest_request(request, common::context(), &mut sink)?;

    match &outcome.events[0].payload {
        Payload::AssistantFinalRecorded(payload) => {
            assert_eq!(payload.content, "final answer only")
        }
        payload => panic!("unexpected payload: {payload:?}"),
    }
    Ok(())
}
