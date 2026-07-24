#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_core::decision::DecisionState;
use ai_brains_core::errors::Error;
use ai_brains_core::ids::PrincipalId;

#[test]
fn proposed_to_approved__without_approver__rejected() {
    let err = DecisionState::Proposed
        .transition(DecisionState::Approved, None)
        .expect_err("silent auto-approve is illegal");
    assert!(matches!(err, Error::ApprovalRequired { .. }));
}

#[test]
fn proposed_to_approved__with_approver__allowed() {
    let approver = PrincipalId::new();
    let next = DecisionState::Proposed
        .transition(DecisionState::Approved, Some(approver))
        .expect("approver present");
    assert_eq!(next, DecisionState::Approved);
}

#[test]
fn approved_to_revoked__allowed() {
    let next = DecisionState::Approved
        .transition(DecisionState::Revoked, None)
        .expect("revoke does not require re-approval");
    assert_eq!(next, DecisionState::Revoked);
}
