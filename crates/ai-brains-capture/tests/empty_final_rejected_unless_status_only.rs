mod common;

use ai_brains_capture::CaptureError;

#[test]
fn empty_final_rejected_unless_status_only() {
    let service = common::service();
    let mut sink = common::sink();
    let request = common::ingest_request("assistant", " ");

    let result = service.ingest_request(request, common::context(), &mut sink);

    assert!(matches!(result, Err(CaptureError::EmptyFinal)));
}
