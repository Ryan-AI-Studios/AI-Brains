mod common;

use ai_brains_events::{EventKind, Payload};

#[test]
fn session_start_appends_event() -> Result<(), Box<dyn std::error::Error>> {
    let service = common::service();
    let mut sink = common::sink();
    let command = common::start_command();

    let outcome = service.start_session(command.clone(), common::context(), &mut sink)?;

    assert_eq!(outcome.events.len(), 1);
    assert_eq!(outcome.events[0].event_type, EventKind::SessionStarted);
    match &outcome.events[0].payload {
        Payload::SessionStarted(payload) => assert_eq!(payload.project_id, command.project_id),
        payload => panic!("unexpected payload: {payload:?}"),
    }
    Ok(())
}
