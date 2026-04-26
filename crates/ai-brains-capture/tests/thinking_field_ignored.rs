mod common;

use ai_brains_events::Payload;

#[test]
fn thinking_field_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let service = common::service();
    let mut sink = common::sink();
    let mut request = common::ingest_request("assistant", "visible answer");
    request.thinking = Some("hidden chain of thought".to_string());

    let outcome = service.ingest_request(request, common::context(), &mut sink)?;

    match &outcome.events[0].payload {
        Payload::AssistantFinalRecorded(payload) => {
            assert_eq!(payload.content, "visible answer");
            assert!(!payload.content.contains("hidden chain of thought"));
        }
        payload => panic!("unexpected payload: {payload:?}"),
    }
    Ok(())
}
