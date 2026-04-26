mod common;

use ai_brains_events::{EventKind, Payload};

#[test]
fn assistant_final_appends_event() -> Result<(), Box<dyn std::error::Error>> {
    let service = common::service();
    let mut sink = common::sink();
    let request = common::ingest_request("assistant", "final answer");

    let outcome = service.ingest_request(request, common::context(), &mut sink)?;

    assert_eq!(
        outcome.events[0].event_type,
        EventKind::AssistantFinalRecorded
    );
    match &outcome.events[0].payload {
        Payload::AssistantFinalRecorded(payload) => assert_eq!(payload.content, "final answer"),
        payload => panic!("unexpected payload: {payload:?}"),
    }
    Ok(())
}
