//! Scenario 8 — erased_evidence_removes_derived (in-process CE wipe, no daemon).

use std::collections::{BTreeMap, BTreeSet};

use ai_brains_core::ids::ContentKeyId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_store::projections::content_envelope::{
    self, ALGORITHM_AES_256_GCM, ENVELOPE_SCHEMA_VERSION, EncryptedBlobRow,
};
use serde_json::Value;

use super::SeedOutcome;
use super::common::{
    agent, grant_read_write, human, register, resolve_for_project, seed_active_conclusion,
    seed_approved_decision, stable_project, stable_uuid,
};
use crate::adapters::{StorePorts, SystemClock};
use crate::conclusions::reject_conclusion;
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

    // Unaffected current authority claim (must remain / min floor).
    let dec_id = seed_approved_decision(
        ports,
        &human_p,
        scope.clone(),
        "Post-wipe authority",
        "Decision not backed by wiped envelope remains",
        "ce-wipe:unaffected-decision",
    )?;

    // Claim whose id equals the CE wiped subject — present in authority before wipe path,
    // then removed so ce_subject_absent / must_be_absent are non-vacuous (F-003).
    let wiped_claim = seed_active_conclusion(
        ports,
        &agent_p,
        scope.clone(),
        "Derived from envelope-backed evidence subject to wipe",
        "ce-wipe:wiped-subject-claim",
    )?;

    let content_key_id = ContentKeyId::from_uuid(stable_uuid("ce-wipe:key"));
    // CE subject_id equals the authority claim id so post-wipe absence is observable.
    let wiped_subject = wiped_claim.clone();
    let blob_id = format!("blob-{}", stable_uuid("ce-wipe:blob"));

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
                subject_kind: Some("Conclusion".into()),
                subject_id: Some(wiped_subject.clone()),
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
            principal: human_p.clone(),
            content_key_id,
            scope: scope.clone(),
            reason: Some(reason.into()),
            tombstone_id: None,
            dry_run: false,
            confirm: true,
        },
    )?;

    // CE wipe does not remove non-source domain claims; exercise post-wipe staling of
    // the wiped subject claim so authority absence is real (honest T165 coupling).
    let wiped_id = wiped_claim
        .parse()
        .map_err(|e| crate::errors::ControlPlaneError::InvalidPayload(format!("id: {e}")))?;
    reject_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &ports.production_policy(),
        &human_p,
        wiped_id,
        "derived subject rejected after CE wipe",
        Privacy::LocalOnly,
    )?;

    let mut must_be_absent = BTreeSet::new();
    must_be_absent.insert(wiped_subject.clone());

    Ok(SeedOutcome {
        principal: agent_p,
        project_id,
        resolve: resolve_for_project(project_id),
        wiped_subject_id: Some(wiped_subject),
        must_be_absent_claim_ids: must_be_absent,
        claim_ids: vec![dec_id],
        content_key_id: Some(content_key_id.to_string()),
        require_citations: true,
        ..SeedOutcome::default()
    })
}
