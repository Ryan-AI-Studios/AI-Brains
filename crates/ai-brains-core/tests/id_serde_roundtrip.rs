use ai_brains_core::ids::*;

macro_rules! id_roundtrip {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let id = <$ty>::new();
            let serialized = serde_json::to_string(&id)?;
            let deserialized: $ty = serde_json::from_str(&serialized)?;
            assert_eq!(id, deserialized);
            Ok(())
        }
    };
}

id_roundtrip!(project_id_serde_roundtrip, ProjectId);
id_roundtrip!(session_id_serde_roundtrip, SessionId);
id_roundtrip!(user_id_serde_roundtrip, UserId);
id_roundtrip!(device_id_serde_roundtrip, DeviceId);
id_roundtrip!(harness_id_serde_roundtrip, HarnessId);
id_roundtrip!(turn_id_serde_roundtrip, TurnId);
id_roundtrip!(memory_id_serde_roundtrip, MemoryId);
id_roundtrip!(conflict_id_serde_roundtrip, ConflictId);
id_roundtrip!(recipe_id_serde_roundtrip, RecipeId);
id_roundtrip!(knowledge_id_serde_roundtrip, KnowledgeId);
id_roundtrip!(source_id_serde_roundtrip, SourceId);
id_roundtrip!(source_version_id_serde_roundtrip, SourceVersionId);
id_roundtrip!(evidence_id_serde_roundtrip, EvidenceId);
id_roundtrip!(conclusion_id_serde_roundtrip, ConclusionId);
id_roundtrip!(decision_id_serde_roundtrip, DecisionId);
id_roundtrip!(workspace_id_serde_roundtrip, WorkspaceId);
id_roundtrip!(principal_id_serde_roundtrip, PrincipalId);
id_roundtrip!(grant_id_serde_roundtrip, GrantId);
id_roundtrip!(review_item_id_serde_roundtrip, ReviewItemId);
id_roundtrip!(briefing_id_serde_roundtrip, BriefingId);
id_roundtrip!(query_trace_id_serde_roundtrip, QueryTraceId);
id_roundtrip!(content_key_id_serde_roundtrip, ContentKeyId);
id_roundtrip!(tombstone_id_serde_roundtrip, TombstoneId);
id_roundtrip!(replication_event_id_serde_roundtrip, ReplicationEventId);

#[test]
fn decision_id_is_not_memory_id_type() {
    // Compile-time / behavioral distinction: both are UUIDs but distinct newtypes.
    let d = DecisionId::new();
    let m = MemoryId::from_uuid(d.as_uuid());
    assert_eq!(d.as_uuid(), m.as_uuid());
    // Different types cannot be compared directly — this test documents the dual model.
}
