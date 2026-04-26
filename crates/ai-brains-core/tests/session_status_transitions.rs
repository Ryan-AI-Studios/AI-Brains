use ai_brains_core::status::*;

#[test]
fn test_session_status_transitions() -> Result<(), Box<dyn std::error::Error>> {
    let mut status = SessionStatus::Active;

    // Valid transitions
    status = status.transition(SessionStatus::Paused)?;
    assert_eq!(status, SessionStatus::Paused);

    status = status.transition(SessionStatus::Active)?;
    assert_eq!(status, SessionStatus::Active);

    status = status.transition(SessionStatus::Completed)?;
    assert_eq!(status, SessionStatus::Completed);

    // Invalid transitions
    let res = status.transition(SessionStatus::Active);
    assert!(res.is_err());
    Ok(())
}
