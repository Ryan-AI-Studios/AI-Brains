mod common;

use ai_brains_capture::SessionStopStatus;
use ai_brains_events::{EventKind, Payload};

#[test]
fn session_failed_appends_status() -> Result<(), Box<dyn std::error::Error>> {
    let service = common::service();
    let mut sink = common::sink();
    let command = common::stop_command(SessionStopStatus::Failed);

    let outcome = service.stop_session(command, common::context(), &mut sink)?;

    assert_eq!(outcome.events[0].event_type, EventKind::SessionFailed);
    match &outcome.events[0].payload {
        Payload::SessionFailed(payload) => assert_eq!(payload.reason, "failure"),
        payload => panic!("unexpected payload: {payload:?}"),
    }
    Ok(())
}
