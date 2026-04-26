use ai_brains_core::ids::*;

#[test]
fn test_project_id_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let id = ProjectId::new();
    let serialized = serde_json::to_string(&id)?;
    let deserialized: ProjectId = serde_json::from_str(&serialized)?;
    assert_eq!(id, deserialized);
    Ok(())
}

#[test]
fn test_session_id_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let id = SessionId::new();
    let serialized = serde_json::to_string(&id)?;
    let deserialized: SessionId = serde_json::from_str(&serialized)?;
    assert_eq!(id, deserialized);
    Ok(())
}
