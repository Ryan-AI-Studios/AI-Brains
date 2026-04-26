mod common;

#[test]
fn git_metadata_attached_when_available() -> Result<(), Box<dyn std::error::Error>> {
    let service = common::service();
    let mut sink = common::sink();
    let request = common::ingest_request("user", "hello");
    let context = common::git_context()?;

    let outcome = service.ingest_request(request, context, &mut sink)?;

    assert!(outcome.metadata.git_metadata.is_some());
    Ok(())
}
