//! Scenario 8 — erased_evidence_removes_derived (in-process CE wipe, no daemon).

use std::collections::BTreeMap;

use ai_brains_core::ids::ContentKeyId;
use ai_brains_core::scope::ScopeRef;
use ai_brains_store::projections::content_envelope::{
    self, ALGORITHM_AES_256_GCM, ENVELOPE_SCHEMA_VERSION, EncryptedBlobRow,
};
use serde_json::Value;
use uuid::Uuid;

use super::SeedOutcome;
use super::common::{
    agent, grant_read_write, human, register, resolve_for_project, seed_approved_decision,
    stable_project, stable_uuid,
};
use crate::adapters::{StorePorts, SystemClock};
use crate::cryptographic_erasure::{
    StoreContentEnvelopeWipe, WipeContentEnvelopeCommand, wipe_content_envelope,
};
use crate::errors::Result;

const CREATED_AT: &str = "2026-07-29T12:00:00Z";

pub fn seed(ports: &StorePorts, params: &BTreeMap<String, Value>) -> Result<SeedOutcome> {
    let project_id = stable_project("ce-wipe");
    let scope = ScopeRef::Repository(project_id);
    let human_p = human("ce-human");
    let agent_p = agent("ce-agent");
    register(ports, &human_p)?;
    register(ports, &agent_p)?;
    grant_read_write(ports, human_p.id, scope.clone())?;
    grant_read_write(ports, agent_p.id, scope.clone())?;

    // Current authority claim (must remain / min floor).
    let dec_id = seed_approved_decision(
        ports,
        &human_p,
        scope.clone(),
        "Post-wipe authority",
        "Decision not backed by wiped envelope remains",
    )?;

    let content_key_id = ContentKeyId::from_uuid(stable_uuid("ce-wipe:key"));
    let memory_subject = format!(
        "memory-{}",
        Uuid::from_u128(0x0000_0000_0000_0000_0000_00CE_0000_0001)
    );
    let blob_id = format!(
        "blob-{}",
        Uuid::from_u128(0x0000_0000_0000_0000_0000_00CE_0000_0002)
    );

    {
        let store = ports.store();
        let conn = store
            .connection()
            .lock()
            .map_err(|e| crate::errors::ControlPlaneError::Query(e.to_string()))?;
        content_envelope::insert_content_key_wrap(
            &conn,
            &content_key_id.to_string(),
            1,
            &[0xAAu8; 12],
            &[0xBBu8; 48],
            CREATED_AT,
        )
        .map_err(|e| crate::errors::ControlPlaneError::Query(e.to_string()))?;
        let ct = vec![0xCCu8; 32];
        content_envelope::insert_encrypted_blob(
            &conn,
            &EncryptedBlobRow {
                blob_id: blob_id.clone(),
                content_key_id: content_key_id.to_string(),
                envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
                algorithm: ALGORITHM_AES_256_GCM.to_string(),
                nonce: vec![0xDDu8; 12],
                ciphertext: ct.clone(),
                content_class: None,
                subject_kind: Some("Memory".into()),
                subject_id: Some(memory_subject.clone()),
                size_bytes: ct.len() as i64,
                created_at: CREATED_AT.to_string(),
            },
        )
        .map_err(|e| crate::errors::ControlPlaneError::Query(e.to_string()))?;
    }

    let reason = params
        .get("wipe_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("evaluation CE wipe");

    let side = StoreContentEnvelopeWipe::new(ports.store());
    wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &ports.production_policy(),
        &side,
        WipeContentEnvelopeCommand {
            principal: human_p,
            content_key_id,
            scope,
            reason: Some(reason.into()),
            tombstone_id: None,
            dry_run: false,
            confirm: true,
        },
    )?;

    Ok(SeedOutcome {
        principal: agent_p,
        project_id,
        resolve: resolve_for_project(project_id),
        wiped_subject_id: Some(memory_subject),
        claim_ids: vec![dec_id],
        content_key_id: Some(content_key_id.to_string()),
        require_citations: true,
        ..SeedOutcome::default()
    })
}
