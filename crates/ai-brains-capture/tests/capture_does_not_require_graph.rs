mod common;

#[test]
fn capture_does_not_require_graph() -> Result<(), Box<dyn std::error::Error>> {
    let service = common::service();
    let mut sink = common::sink();
    let request = common::ingest_request("assistant", "works without graph");

    let outcome = service.ingest_request(request, common::context(), &mut sink)?;

    assert_eq!(outcome.events.len(), 1);
    Ok(())
}
