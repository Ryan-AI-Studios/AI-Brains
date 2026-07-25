//! Conclusion lifecycle commands (T150 Phase D).

use ai_brains_core::conclusion::{ApprovalAuthority, ConclusionState};
use ai_brains_core::ids::{ConclusionId, EvidenceId, PrincipalId};
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::protected_category::ProtectedCategory;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_events::payload::{
    ConclusionActivatedPayload, ConclusionConfirmedPayload, ConclusionProposedPayload,
    ConclusionRejectedPayload, ConclusionSupersededPayload,
};
use ai_brains_events::{Actor, AggregateType, Payload};
use time::OffsetDateTime;

use crate::errors::{ControlPlaneError, Result};
use crate::ports::{Clock, EventWriter, GovernedQueryStore, PolicyEvaluator};
use crate::sources::{build_event, ensure_valid_time_interval, scope_identity_key};

#[derive(Debug, Clone)]
pub struct ProposeConclusionRequest {
    pub principal: Principal,
    pub scope: ScopeRef,
    pub statement: String,
    pub evidence_ids: Vec<EvidenceId>,
    pub privacy: Privacy,
    pub valid_from: Option<OffsetDateTime>,
    pub valid_until: Option<OffsetDateTime>,
    pub protected_category: Option<ProtectedCategory>,
    /// Optional pre-assigned id (tests); otherwise generated.
    pub conclusion_id: Option<ConclusionId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposeConclusionResult {
    pub conclusion_id: ConclusionId,
    pub unsupported: bool,
}

pub fn propose_conclusion<W, Q, C, P>(
    writer: &W,
    _query: &Q,
    clock: &C,
    policy: &P,
    req: ProposeConclusionRequest,
) -> Result<ProposeConclusionResult>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
{
    if !policy.allow(
        req.principal.id,
        GrantCapability::ProposeConclusion,
        &req.scope,
    )? {
        return Err(ControlPlaneError::PolicyDenied(
            "ProposeConclusion denied".into(),
        ));
    }
    if req.statement.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "statement must be non-empty".into(),
        ));
    }

    let now = clock.now()?;
    let conclusion_id = req.conclusion_id.unwrap_or_default();
    let unsupported = req.evidence_ids.is_empty();
    let valid_from_ts = req.valid_from.unwrap_or(now);
    ensure_valid_time_interval(valid_from_ts, req.valid_until)?;
    let valid_from = Some(valid_from_ts);
    let scope_key = scope_identity_key(&req.scope);
    let protected = req.protected_category.map(|c| c.as_str().to_string());

    let event = build_event(
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        Actor::System,
        req.privacy,
        Payload::ConclusionProposed(ConclusionProposedPayload {
            conclusion_id,
            statement: req.statement,
            evidence_ids: req.evidence_ids,
            proposer: req.principal.id,
            valid_from,
            valid_until: req.valid_until,
            scope: scope_key,
            protected_category: protected,
            unsupported,
            model_provenance: None,
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(ProposeConclusionResult {
        conclusion_id,
        unsupported,
    })
}

pub fn activate_conclusion<W, Q, C>(
    writer: &W,
    query: &Q,
    _clock: &C,
    principal: &Principal,
    conclusion_id: ConclusionId,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
{
    let row = query
        .get_conclusion(conclusion_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("conclusion {conclusion_id}")))?;

    if row.protected_category.is_some() {
        return Err(ControlPlaneError::PolicyDenied(
            "protected conclusion cannot be activated without confirmation path".into(),
        ));
    }

    let state = parse_conclusion_state(&row.state)?;
    state
        .transition(ConclusionState::Active, None, None)
        .map_err(|e| ControlPlaneError::InvalidTransition(e.to_string()))?;

    // Agent may activate non-protected candidates.
    let _ = principal;

    let event = build_event(
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::ConclusionActivated(ConclusionActivatedPayload { conclusion_id }),
    )?;
    writer.append_events(&[event])?;
    Ok(())
}

pub fn confirm_conclusion<W, Q, C>(
    writer: &W,
    query: &Q,
    clock: &C,
    principal: &Principal,
    conclusion_id: ConclusionId,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
{
    let row = query
        .get_conclusion(conclusion_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("conclusion {conclusion_id}")))?;

    if row.unsupported {
        return Err(ControlPlaneError::UnsupportedCannotConfirm(
            conclusion_id.to_string(),
        ));
    }

    if row.protected_category.is_some() && !is_human(principal) {
        return Err(ControlPlaneError::ApprovalRequired(
            "protected conclusion confirm requires human principal".into(),
        ));
    }

    if !is_human(principal) && row.protected_category.is_none() {
        // Non-protected confirm still requires human approval authority per domain rules.
        return Err(ControlPlaneError::ApprovalRequired(
            "confirm requires human ApprovalAuthority".into(),
        ));
    }

    let state = parse_conclusion_state(&row.state)?;
    let approval = ApprovalAuthority {
        principal_id: principal.id,
    };
    state
        .transition(ConclusionState::Confirmed, Some(approval), None)
        .map_err(|e| ControlPlaneError::InvalidTransition(e.to_string()))?;

    let now = clock.now()?;
    let event = build_event(
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::ConclusionConfirmed(ConclusionConfirmedPayload {
            conclusion_id,
            approver: principal.id,
            confirmed_at: now,
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(())
}

/// Alias for confirm (approve_conclusion → Confirmed).
pub fn approve_conclusion<W, Q, C>(
    writer: &W,
    query: &Q,
    clock: &C,
    principal: &Principal,
    conclusion_id: ConclusionId,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
{
    confirm_conclusion(writer, query, clock, principal, conclusion_id, privacy)
}

pub fn reject_conclusion<W, Q, C>(
    writer: &W,
    query: &Q,
    _clock: &C,
    principal: &Principal,
    conclusion_id: ConclusionId,
    reason: &str,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
{
    if reason.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "reject reason must be non-empty".into(),
        ));
    }
    let row = query
        .get_conclusion(conclusion_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("conclusion {conclusion_id}")))?;
    let state = parse_conclusion_state(&row.state)?;
    state
        .transition(ConclusionState::Rejected, None, None)
        .map_err(|e| ControlPlaneError::InvalidTransition(e.to_string()))?;

    let event = build_event(
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::ConclusionRejected(ConclusionRejectedPayload {
            conclusion_id,
            rejector: principal.id,
            reason: reason.to_string(),
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(())
}

/// Correct: propose successor + supersede predecessor in a single batch.
///
/// Old state must allow transition to Superseded (Active/Confirmed/Stale/Disputed).
/// Candidates must be activated (or confirmed) first. Propose policy is enforced.
#[allow(clippy::too_many_arguments)]
pub fn correct_conclusion<W, Q, C, P>(
    writer: &W,
    query: &Q,
    clock: &C,
    policy: &P,
    principal: &Principal,
    old_conclusion_id: ConclusionId,
    new_statement: String,
    evidence_ids: Vec<EvidenceId>,
    reason: &str,
    privacy: Privacy,
) -> Result<ConclusionId>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
{
    if reason.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "correction reason must be non-empty".into(),
        ));
    }
    if new_statement.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "corrected statement must be non-empty".into(),
        ));
    }
    let old = query
        .get_conclusion(old_conclusion_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("conclusion {old_conclusion_id}")))?;

    let old_state = parse_conclusion_state(&old.state)?;
    // Domain allows Superseded only from Active/Confirmed/Stale/Disputed — not Candidate/Rejected.
    old_state
        .transition(ConclusionState::Superseded, None, None)
        .map_err(|e| ControlPlaneError::InvalidTransition(e.to_string()))?;

    let scope = parse_scope_key(&old.scope)?;
    if !policy.allow(principal.id, GrantCapability::ProposeConclusion, &scope)? {
        return Err(ControlPlaneError::PolicyDenied(
            "ProposeConclusion denied".into(),
        ));
    }

    let now = clock.now()?;
    ensure_valid_time_interval(now, old.valid_until)?;
    let new_id = ConclusionId::new();
    let unsupported = evidence_ids.is_empty();
    let propose = build_event(
        AggregateType::Conclusion,
        new_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::ConclusionProposed(ConclusionProposedPayload {
            conclusion_id: new_id,
            statement: new_statement,
            evidence_ids,
            proposer: principal.id,
            valid_from: Some(now),
            valid_until: old.valid_until,
            scope: old.scope.clone(),
            protected_category: old.protected_category.clone(),
            unsupported,
            model_provenance: None,
        }),
    )?;
    let supersede = build_event(
        AggregateType::Conclusion,
        old_conclusion_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::ConclusionSuperseded(ConclusionSupersededPayload {
            conclusion_id: old_conclusion_id,
            superseded_by: new_id,
            reason: reason.to_string(),
        }),
    )?;
    writer.append_events(&[propose, supersede])?;
    Ok(new_id)
}

fn is_human(principal: &Principal) -> bool {
    matches!(principal.kind, PrincipalKind::Human)
}

fn parse_conclusion_state(s: &str) -> Result<ConclusionState> {
    match s {
        "Candidate" => Ok(ConclusionState::Candidate),
        "Active" => Ok(ConclusionState::Active),
        "Confirmed" => Ok(ConclusionState::Confirmed),
        "Stale" => Ok(ConclusionState::Stale),
        "Disputed" => Ok(ConclusionState::Disputed),
        "Superseded" => Ok(ConclusionState::Superseded),
        "Rejected" => Ok(ConclusionState::Rejected),
        other => Err(ControlPlaneError::InvalidTransition(format!(
            "unknown conclusion state {other}"
        ))),
    }
}

/// Rehydrate [`ScopeRef`] from the stored scope identity key.
fn parse_scope_key(key: &str) -> Result<ScopeRef> {
    use ai_brains_core::ids::{ProjectId, UserId, WorkspaceId};
    use uuid::Uuid;

    let parse_uuid = |rest: &str, kind: &str| -> Result<Uuid> {
        Uuid::parse_str(rest).map_err(|e| {
            ControlPlaneError::InvalidPayload(format!("invalid {kind} id in scope key: {e}"))
        })
    };

    if let Some(rest) = key.strip_prefix("Repository:") {
        Ok(ScopeRef::Repository(ProjectId::from_uuid(parse_uuid(
            rest,
            "Repository",
        )?)))
    } else if let Some(rest) = key.strip_prefix("Workspace:") {
        Ok(ScopeRef::Workspace(WorkspaceId::from_uuid(parse_uuid(
            rest,
            "Workspace",
        )?)))
    } else if let Some(rest) = key.strip_prefix("Personal:") {
        Ok(ScopeRef::Personal(UserId::from_uuid(parse_uuid(
            rest, "Personal",
        )?)))
    } else {
        Err(ControlPlaneError::InvalidPayload(format!(
            "unparseable scope key: {key}"
        )))
    }
}

/// Helper for tests / callers constructing a principal.
pub fn principal(kind: PrincipalKind, id: PrincipalId, name: &str) -> Principal {
    Principal {
        id,
        kind,
        display_name: name.to_string(),
    }
}
