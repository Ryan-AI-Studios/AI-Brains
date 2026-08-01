//! Shared hermetic vault + grant helpers for evaluation seeds.

use std::path::PathBuf;

use ai_brains_core::ids::{ConclusionId, DecisionId, EvidenceId, PrincipalId, ProjectId, UserId};
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_crypto::DataKey;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use tempfile::NamedTempFile;
use uuid::Uuid;

use crate::adapters::{StorePorts, SystemClock};
use crate::errors::{ControlPlaneError, Result};
use crate::grants::{issue_grant, register_principal};
use crate::scope_resolver::ScopeResolveInput;
use crate::{
    ProposeConclusionRequest, ProposeDecisionRequest, activate_conclusion, approve_decision,
    make_principal, propose_conclusion, propose_decision,
};

/// Fixed DNS-style namespace for deterministic evaluation ids.
pub const EVAL_NS: &str = "ai-brains.evaluation.t169";

pub fn stable_uuid(label: &str) -> Uuid {
    let ns = Uuid::new_v5(&Uuid::NAMESPACE_DNS, EVAL_NS.as_bytes());
    Uuid::new_v5(&ns, label.as_bytes())
}

pub fn stable_project(label: &str) -> ProjectId {
    ProjectId::from_uuid(stable_uuid(&format!("project:{label}")))
}

pub fn stable_user(label: &str) -> UserId {
    UserId::from_uuid(stable_uuid(&format!("user:{label}")))
}

pub fn stable_principal_id(label: &str) -> PrincipalId {
    PrincipalId::from_uuid(stable_uuid(&format!("principal:{label}")))
}

pub fn stable_decision_id(label: &str) -> DecisionId {
    DecisionId::from_uuid(stable_uuid(&format!("decision:{label}")))
}

pub fn stable_conclusion_id(label: &str) -> ConclusionId {
    ConclusionId::from_uuid(stable_uuid(&format!("conclusion:{label}")))
}

pub fn stable_evidence_id(label: &str) -> EvidenceId {
    EvidenceId::from_uuid(stable_uuid(&format!("evidence:{label}")))
}

/// Open a fresh SQLCipher vault in a NamedTempFile (hermetic; E1/E25).
pub fn open_hermetic_ports() -> Result<(NamedTempFile, StorePorts)> {
    let temp_file = NamedTempFile::new().map_err(|e| {
        ControlPlaneError::Query(format!("tempfile for evaluation vault failed: {e}"))
    })?;
    let db_path = temp_file.path().to_str().ok_or_else(|| {
        ControlPlaneError::Query("evaluation vault path is not valid UTF-8".into())
    })?;
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key)
        .map_err(|e| ControlPlaneError::Query(format!("open evaluation vault: {e}")))?;
    conn.migrate()
        .map_err(|e| ControlPlaneError::Query(format!("migrate evaluation vault: {e}")))?;
    Ok((
        temp_file,
        StorePorts::from_store(SqliteEventStore::new(conn)),
    ))
}

pub fn agent(label: &str) -> Principal {
    make_principal(
        PrincipalKind::Agent,
        stable_principal_id(&format!("agent:{label}")),
        label,
    )
}

pub fn human(label: &str) -> Principal {
    make_principal(
        PrincipalKind::Human,
        stable_principal_id(&format!("human:{label}")),
        label,
    )
}

pub fn grant_read_write(ports: &StorePorts, principal: PrincipalId, scope: ScopeRef) -> Result<()> {
    let clock = SystemClock;
    for cap in [
        GrantCapability::ReadEvidence,
        GrantCapability::ReadConclusions,
        GrantCapability::ReadDecisions,
        GrantCapability::ProposeConclusion,
        GrantCapability::ProposeDecision,
        GrantCapability::ApproveDecision,
        GrantCapability::ApproveConclusion,
        GrantCapability::Erase,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            principal,
            scope.clone(),
            cap,
            Privacy::LocalOnly,
        )?;
    }
    Ok(())
}

pub fn register(ports: &StorePorts, p: &Principal) -> Result<()> {
    register_principal(&ports.writer, &SystemClock, p)
}

pub fn resolve_for_project(project_id: ProjectId) -> ScopeResolveInput {
    ScopeResolveInput {
        cwd: PathBuf::from("."),
        explicit_project_id: Some(project_id),
        force_personal: false,
        personal_user_id: None,
        git_metadata: None,
    }
}

/// Propose + approve a decision with one evidence handle (stable uuid-v5 ids).
pub fn seed_approved_decision(
    ports: &StorePorts,
    principal: &Principal,
    scope: ScopeRef,
    title: &str,
    statement: &str,
    id_label: &str,
) -> Result<String> {
    let policy = ports.production_policy();
    let clock = SystemClock;
    let decision_id = stable_decision_id(id_label);
    let evidence_id = stable_evidence_id(id_label);
    let dec = propose_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeDecisionRequest {
            principal: principal.clone(),
            scope,
            title: title.into(),
            statement: statement.into(),
            conclusion_ids: None,
            evidence_ids: Some(vec![evidence_id]),
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: Some(decision_id),
        },
    )?;
    approve_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        principal,
        dec.decision_id,
        Privacy::LocalOnly,
    )?;
    Ok(dec.decision_id.to_string())
}

/// Propose + activate a conclusion with evidence (stable uuid-v5 ids).
pub fn seed_active_conclusion(
    ports: &StorePorts,
    principal: &Principal,
    scope: ScopeRef,
    statement: &str,
    id_label: &str,
) -> Result<String> {
    let policy = ports.production_policy();
    let clock = SystemClock;
    let conclusion_id = stable_conclusion_id(id_label);
    let evidence_id = stable_evidence_id(id_label);
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: principal.clone(),
            scope,
            statement: statement.into(),
            evidence_ids: vec![evidence_id],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: Some(conclusion_id),
        },
    )?;
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        principal,
        res.conclusion_id,
        Privacy::LocalOnly,
    )?;
    Ok(res.conclusion_id.to_string())
}
