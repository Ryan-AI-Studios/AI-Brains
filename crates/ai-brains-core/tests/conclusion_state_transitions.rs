#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_core::conclusion::{ApprovalAuthority, ConclusionState, RevalidationProof};
use ai_brains_core::errors::Error;
use ai_brains_core::ids::PrincipalId;

#[test]
fn candidate_to_active__without_approval__allowed() {
    let next = ConclusionState::Candidate
        .transition(ConclusionState::Active, None, None)
        .expect("Candidate → Active is non-protected");
    assert_eq!(next, ConclusionState::Active);
}

#[test]
fn candidate_to_confirmed__without_approval__rejected() {
    let err = ConclusionState::Candidate
        .transition(ConclusionState::Confirmed, None, None)
        .expect_err("Confirmed requires approval");
    assert!(matches!(err, Error::ApprovalRequired { .. }));
}

#[test]
fn candidate_to_confirmed__with_approval__allowed() {
    let authority = ApprovalAuthority {
        principal_id: PrincipalId::new(),
    };
    let next = ConclusionState::Candidate
        .transition(ConclusionState::Confirmed, Some(authority), None)
        .expect("approval present");
    assert_eq!(next, ConclusionState::Confirmed);
}

#[test]
fn stale_to_active__without_revalidation__rejected() {
    let err = ConclusionState::Stale
        .transition(ConclusionState::Active, None, None)
        .expect_err("Stale → Active requires revalidation");
    assert!(matches!(err, Error::RevalidationRequired { .. }));
}

#[test]
fn stale_to_active__with_revalidation__allowed() {
    let proof = RevalidationProof {
        token: "revalidated-v2".to_string(),
    };
    let next = ConclusionState::Stale
        .transition(ConclusionState::Active, None, Some(&proof))
        .expect("revalidation present");
    assert_eq!(next, ConclusionState::Active);
}
