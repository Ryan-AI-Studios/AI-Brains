mod common;

use ai_brains_capture::CaptureError;

#[test]
fn empty_prompt_rejected() {
    let service = common::service();
    let mut sink = common::sink();
    let request = common::ingest_request("user", "   ");

    let result = service.ingest_request(request, common::context(), &mut sink);

    assert!(matches!(result, Err(CaptureError::EmptyPrompt)));
}
