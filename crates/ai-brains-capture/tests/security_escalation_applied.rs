mod common;

use ai_brains_core::privacy::Privacy;

#[test]
fn security_escalation_applied() -> Result<(), Box<dyn std::error::Error>> {
    let service = common::service();
    let mut sink = common::sink();
    let request = common::ingest_request("user", "Authorization: Bearer abcdefghijklmnopQRST1234");

    let outcome = service.ingest_request(request, common::context(), &mut sink)?;

    assert_eq!(outcome.effective_privacy, Privacy::LocalOnly);
    assert_eq!(outcome.events[0].privacy, Privacy::LocalOnly);
    Ok(())
}
