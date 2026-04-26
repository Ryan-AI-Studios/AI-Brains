mod common;

use ai_brains_events::{EventKind, Payload};

#[test]
fn user_prompt_appends_event() -> Result<(), Box<dyn std::error::Error>> {
    let service = common::service();
    let mut sink = common::sink();
    let request = common::ingest_request("user", "hello world");

    let outcome = service.ingest_request(request, common::context(), &mut sink)?;

    assert_eq!(outcome.events[0].event_type, EventKind::UserPromptRecorded);
    match &outcome.events[0].payload {
        Payload::UserPromptRecorded(payload) => assert_eq!(payload.content, "hello world"),
        payload => panic!("unexpected payload: {payload:?}"),
    }
    Ok(())
}
