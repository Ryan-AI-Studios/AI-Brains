//! Decision lifecycle commands (T150 Phase E).
//!
//! # Idempotency (T159)
//!
//! When [`ProposeDecisionRequest::decision_id`] is `Some`, it is an
//! **idempotency handle**: after payload validation and policy allow, if a
//! decision with that id already exists, [`propose_decision`] returns the
//! prior result without a second `DecisionProposed` event. Callers without
//! `ProposeDecision` still receive [`ControlPlaneError::PolicyDenied`].

use ai_brains_core::decision::DecisionState;
use ai_brains_core::ids::{ConclusionId, DecisionId, EvidenceId};
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_events::payload::{
    DecisionApprovedPayload, DecisionProposedPayload, DecisionRevokedPayload,
    DecisionSupersededPayload,
};
use ai_brains_events::{Actor, AggregateType, Payload};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::errors::{ControlPlaneError, Result};
use crate::ports::{Clock, EventWriter, GovernedQueryStore, PolicyContext, PolicyEvaluator};
use crate::sources::{
    build_event, ensure_valid_time_interval, parse_scope_key, scope_identity_key,
};

#[derive(Debug, Clone)]
pub struct ProposeDecisionRequest {
    pub principal: Principal,
    pub scope: ScopeRef,
    pub title: String,
    pub statement: String,
    pub conclusion_ids: Option<Vec<ConclusionId>>,
    pub evidence_ids: Option<Vec<EvidenceId>>,
    pub privacy: Privacy,
    pub valid_from: Option<OffsetDateTime>,
    pub valid_until: Option<OffsetDateTime>,
    pub decision_id: Option<DecisionId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposeDecisionResult {
    pub decision_id: DecisionId,
    pub proposal_event_id: Uuid,
}

pub fn propose_decision<W, Q, C, P>(
    writer: &W,
    query: &Q,
    clock: &C,
    policy: &P,
    req: ProposeDecisionRequest,
) -> Result<ProposeDecisionResult>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
{
    // 1. Payload validation
    if req.title.trim().is_empty() || req.statement.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "title and statement must be non-empty".into(),
        ));
    }

    // 2. Policy gate (always — before detect-already-done so replay without grant denies)
    let policy_ctx = PolicyContext::default_for_privacy(req.privacy);
    if !policy.allow(
        req.principal.id,
        GrantCapability::ProposeDecision,
        &req.scope,
        &policy_ctx,
    )? {
        return Err(ControlPlaneError::PolicyDenied(
            "ProposeDecision denied".into(),
        ));
    }

    // 3. Detect-already-done when a pre-assigned id is supplied (spool / client retry).
    if let Some(preassigned) = req.decision_id
        && let Some(row) = query.get_decision(preassigned)?
    {
        // Prefer stored proposal_event_id when present; nil UUID if missing/unparsable.
        let proposal_event_id = row
            .proposal_event_id
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::nil);
        return Ok(ProposeDecisionResult {
            decision_id: preassigned,
            proposal_event_id,
        });
    }

    // 4. Append
    let now = clock.now()?;
    let valid_from = req.valid_from.unwrap_or(now);
    ensure_valid_time_interval(valid_from, req.valid_until)?;
    let decision_id = req.decision_id.unwrap_or_default();
    let scope_key = scope_identity_key(&req.scope);

    let event = build_event(
        AggregateType::Decision,
        decision_id.as_uuid(),
        Actor::System,
        req.privacy,
        Payload::DecisionProposed(DecisionProposedPayload {
            decision_id,
            title: req.title,
            statement: req.statement,
            proposer: req.principal.id,
            conclusion_ids: req.conclusion_ids,
            evidence_ids: req.evidence_ids,
            valid_from: Some(valid_from),
            valid_until: req.valid_until,
            scope: scope_key,
        }),
    )?;
    let proposal_event_id = event.event_id;
    writer.append_events(&[event])?;
    Ok(ProposeDecisionResult {
        decision_id,
        proposal_event_id,
    })
}

pub fn approve_decision<W, Q, C, P>(
    writer: &W,
    query: &Q,
    clock: &C,
    policy: &P,
    principal: &Principal,
    decision_id: DecisionId,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
{
    if !matches!(principal.kind, PrincipalKind::Human) {
        return Err(ControlPlaneError::ApprovalRequired(
            "decision approval requires human principal (not Agent)".into(),
        ));
    }

    let row = query
        .get_decision(decision_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("decision {decision_id}")))?;
    let scope = parse_scope_key(&row.scope)?;
    let policy_ctx = PolicyContext::default_for_privacy(privacy);
    if !policy.allow(
        principal.id,
        GrantCapability::ApproveDecision,
        &scope,
        &policy_ctx,
    )? {
        return Err(ControlPlaneError::PolicyDenied(
            "ApproveDecision denied".into(),
        ));
    }

    let state = parse_decision_state(&row.state)?;
    state
        .transition(DecisionState::Approved, Some(principal.id))
        .map_err(|e| ControlPlaneError::InvalidTransition(e.to_string()))?;

    let proposal_event_id = match row.proposal_event_id.as_ref() {
        Some(s) if !s.trim().is_empty() => Uuid::parse_str(s).map_err(|_| {
            ControlPlaneError::InvalidPayload(format!(
                "decision {decision_id} has unparsable proposal_event_id"
            ))
        })?,
        Some(_) | None => {
            return Err(ControlPlaneError::NotFound(format!(
                "decision {decision_id} missing proposal_event_id"
            )));
        }
    };

    let now = clock.now()?;
    let event = build_event(
        AggregateType::Decision,
        decision_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::DecisionApproved(DecisionApprovedPayload {
            decision_id,
            proposal_event_id,
            approver: principal.id,
            approved_at: now,
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn supersede_decision<W, Q, C, P>(
    writer: &W,
    query: &Q,
    _clock: &C,
    policy: &P,
    principal: &Principal,
    old_decision_id: DecisionId,
    new_decision_id: DecisionId,
    reason: &str,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
{
    if reason.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "supersede reason must be non-empty".into(),
        ));
    }
    let row = query
        .get_decision(old_decision_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("decision {old_decision_id}")))?;
    let scope = parse_scope_key(&row.scope)?;
    let policy_ctx = PolicyContext::default_for_privacy(privacy);
    if !policy.allow(
        principal.id,
        GrantCapability::ProposeDecision,
        &scope,
        &policy_ctx,
    )? {
        return Err(ControlPlaneError::PolicyDenied(
            "ProposeDecision denied for supersede".into(),
        ));
    }

    let state = parse_decision_state(&row.state)?;
    state
        .transition(DecisionState::Superseded, None)
        .map_err(|e| ControlPlaneError::InvalidTransition(e.to_string()))?;

    let event = build_event(
        AggregateType::Decision,
        old_decision_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::DecisionSuperseded(DecisionSupersededPayload {
            decision_id: old_decision_id,
            superseded_by: new_decision_id,
            reason: reason.to_string(),
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn revoke_decision<W, Q, C, P>(
    writer: &W,
    query: &Q,
    _clock: &C,
    policy: &P,
    principal: &Principal,
    decision_id: DecisionId,
    reason: &str,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
{
    if reason.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "revoke reason must be non-empty".into(),
        ));
    }
    let row = query
        .get_decision(decision_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("decision {decision_id}")))?;
    let scope = parse_scope_key(&row.scope)?;
    let policy_ctx = PolicyContext::default_for_privacy(privacy);
    if !policy.allow(
        principal.id,
        GrantCapability::ApproveDecision,
        &scope,
        &policy_ctx,
    )? {
        return Err(ControlPlaneError::PolicyDenied(
            "ApproveDecision denied for revoke".into(),
        ));
    }

    let state = parse_decision_state(&row.state)?;
    state
        .transition(DecisionState::Revoked, None)
        .map_err(|e| ControlPlaneError::InvalidTransition(e.to_string()))?;

    let event = build_event(
        AggregateType::Decision,
        decision_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::DecisionRevoked(DecisionRevokedPayload {
            decision_id,
            revoker: principal.id,
            reason: reason.to_string(),
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(())
}

fn parse_decision_state(s: &str) -> Result<DecisionState> {
    match s {
        "Proposed" => Ok(DecisionState::Proposed),
        "Approved" => Ok(DecisionState::Approved),
        "Superseded" => Ok(DecisionState::Superseded),
        "Revoked" => Ok(DecisionState::Revoked),
        other => Err(ControlPlaneError::InvalidTransition(format!(
            "unknown decision state {other}"
        ))),
    }
}
