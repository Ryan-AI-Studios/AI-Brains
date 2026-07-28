//! Conclusion lifecycle commands (T150 Phase D).
//!
//! # Idempotency (T159)
//!
//! When [`ProposeConclusionRequest::conclusion_id`] is `Some`, it is an
//! **idempotency handle**: if a conclusion with that id already exists in the
//! projection, [`propose_conclusion`] returns the prior result **without**
//! appending a second `ConclusionProposed` event. When `None`, a new id is
//! generated and append proceeds as usual (first-wins semantics for T159 when
//! the same pre-assigned id is reused with different statements).

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
use crate::ports::{Clock, EventWriter, GovernedQueryStore, PolicyContext, PolicyEvaluator};
use crate::sources::{
    build_event, ensure_valid_time_interval, parse_scope_key, scope_identity_key,
};

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
    query: &Q,
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
    // Detect-already-done when a pre-assigned id is supplied (spool / client retry).
    if let Some(preassigned) = req.conclusion_id
        && let Some(row) = query.get_conclusion(preassigned)?
    {
        return Ok(ProposeConclusionResult {
            conclusion_id: preassigned,
            unsupported: row.unsupported,
        });
    }

    let policy_ctx = PolicyContext::default_for_privacy(req.privacy);
    if !policy.allow(
        req.principal.id,
        GrantCapability::ProposeConclusion,
        &req.scope,
        &policy_ctx,
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

pub fn activate_conclusion<W, Q, C, P>(
    writer: &W,
    query: &Q,
    _clock: &C,
    policy: &P,
    principal: &Principal,
    conclusion_id: ConclusionId,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
{
    let row = query
        .get_conclusion(conclusion_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("conclusion {conclusion_id}")))?;

    if row.protected_category.is_some() {
        return Err(ControlPlaneError::PolicyDenied(
            "protected conclusion cannot be activated without confirmation path".into(),
        ));
    }

    let scope = parse_scope_key(&row.scope)?;
    let policy_ctx = PolicyContext::default_for_privacy(privacy);
    // Agents may activate non-protected candidates when granted ProposeConclusion.
    if !policy.allow(
        principal.id,
        GrantCapability::ProposeConclusion,
        &scope,
        &policy_ctx,
    )? {
        return Err(ControlPlaneError::PolicyDenied(
            "ProposeConclusion denied for activate".into(),
        ));
    }

    let state = parse_conclusion_state(&row.state)?;
    state
        .transition(ConclusionState::Active, None, None)
        .map_err(|e| ControlPlaneError::InvalidTransition(e.to_string()))?;

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

pub fn confirm_conclusion<W, Q, C, P>(
    writer: &W,
    query: &Q,
    clock: &C,
    policy: &P,
    principal: &Principal,
    conclusion_id: ConclusionId,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
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

    let scope = parse_scope_key(&row.scope)?;
    let policy_ctx = PolicyContext::default_for_privacy(privacy);
    if !policy.allow(
        principal.id,
        GrantCapability::ApproveConclusion,
        &scope,
        &policy_ctx,
    )? {
        return Err(ControlPlaneError::PolicyDenied(
            "ApproveConclusion denied".into(),
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
pub fn approve_conclusion<W, Q, C, P>(
    writer: &W,
    query: &Q,
    clock: &C,
    policy: &P,
    principal: &Principal,
    conclusion_id: ConclusionId,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
{
    confirm_conclusion(
        writer,
        query,
        clock,
        policy,
        principal,
        conclusion_id,
        privacy,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn reject_conclusion<W, Q, C, P>(
    writer: &W,
    query: &Q,
    _clock: &C,
    policy: &P,
    principal: &Principal,
    conclusion_id: ConclusionId,
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
            "reject reason must be non-empty".into(),
        ));
    }
    let row = query
        .get_conclusion(conclusion_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("conclusion {conclusion_id}")))?;
    let scope = parse_scope_key(&row.scope)?;
    let policy_ctx = PolicyContext::default_for_privacy(privacy);
    // Reject is an approval-band action (hard-deny for Agent via matrix).
    if !policy.allow(
        principal.id,
        GrantCapability::ApproveConclusion,
        &scope,
        &policy_ctx,
    )? {
        return Err(ControlPlaneError::PolicyDenied(
            "ApproveConclusion denied for reject".into(),
        ));
    }

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
    let policy_ctx = PolicyContext::default_for_privacy(privacy);
    if !policy.allow(
        principal.id,
        GrantCapability::ProposeConclusion,
        &scope,
        &policy_ctx,
    )? {
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

/// Helper for tests / callers constructing a principal.
pub fn principal(kind: PrincipalKind, id: PrincipalId, name: &str) -> Principal {
    Principal {
        id,
        kind,
        display_name: name.to_string(),
        bound_source_kinds: Vec::new(),
        bound_capabilities: Vec::new(),
    }
}
