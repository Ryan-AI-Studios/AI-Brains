//! Claim conflict open/resolve + time-valid selection (T150 Phase F).

use ai_brains_core::ids::ConflictId;
use ai_brains_core::principal::Principal;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::GrantCapability;
use ai_brains_events::payload::{ClaimConflictOpenedPayload, ClaimConflictResolvedPayload};
use ai_brains_events::{Actor, AggregateType, Payload};
use time::OffsetDateTime;

use crate::errors::{ControlPlaneError, Result};
use crate::ports::{
    ClaimConflictRow, ConclusionRow, EventWriter, GovernedQueryStore, PolicyContext,
    PolicyEvaluator,
};
use crate::sources::{build_event, parse_scope_key};

#[derive(Debug, Clone)]
pub struct OpenClaimConflictRequest {
    pub claim_a_kind: String,
    pub claim_a_id: String,
    pub claim_b_kind: String,
    pub claim_b_id: String,
    pub scope: String,
    pub explanation: String,
    pub privacy: Privacy,
    pub valid_from: Option<OffsetDateTime>,
    pub valid_until: Option<OffsetDateTime>,
    pub conflict_id: Option<ConflictId>,
}

/// Open a claim conflict. Requires `ProposeConclusion` on the conflict scope.
pub fn open_claim_conflict<W, P>(
    writer: &W,
    policy: &P,
    principal: &Principal,
    req: OpenClaimConflictRequest,
) -> Result<ConflictId>
where
    W: EventWriter,
    P: PolicyEvaluator,
{
    if req.explanation.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "conflict explanation must be non-empty".into(),
        ));
    }
    let scope = parse_scope_key(&req.scope)?;
    let policy_ctx = PolicyContext::default_for_privacy(req.privacy);
    if !policy.allow(
        principal.id,
        GrantCapability::ProposeConclusion,
        &scope,
        &policy_ctx,
    )? {
        return Err(ControlPlaneError::PolicyDenied(
            "ProposeConclusion denied for open claim conflict".into(),
        ));
    }

    let conflict_id = req.conflict_id.unwrap_or_default();
    let event = build_event(
        AggregateType::Conflict,
        conflict_id.as_uuid(),
        Actor::System,
        req.privacy,
        Payload::ClaimConflictOpened(ClaimConflictOpenedPayload {
            conflict_id,
            claim_a_kind: req.claim_a_kind,
            claim_a_id: req.claim_a_id,
            claim_b_kind: req.claim_b_kind,
            claim_b_id: req.claim_b_id,
            scope: req.scope,
            explanation: req.explanation,
            valid_from: req.valid_from,
            valid_until: req.valid_until,
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(conflict_id)
}

/// Resolve a claim conflict. Requires `ApproveConclusion` on the conflict scope.
pub fn resolve_claim_conflict<W, Q, P>(
    writer: &W,
    query: &Q,
    policy: &P,
    principal: &Principal,
    conflict_id: ConflictId,
    resolution: &str,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    P: PolicyEvaluator,
{
    if resolution.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "conflict resolution must be non-empty".into(),
        ));
    }
    let row = query
        .get_claim_conflict(conflict_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("claim_conflict {conflict_id}")))?;
    if row.status != "Open" {
        return Err(ControlPlaneError::InvalidTransition(format!(
            "conflict {conflict_id} is {}",
            row.status
        )));
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
            "ApproveConclusion denied for resolve claim conflict".into(),
        ));
    }

    let event = build_event(
        AggregateType::Conflict,
        conflict_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::ClaimConflictResolved(ClaimConflictResolvedPayload {
            conflict_id,
            resolution: resolution.to_string(),
            resolved_by: principal.id,
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(())
}

/// Select conclusions valid at domain time `at` (uses valid_from/until only).
pub fn select_conclusions_valid_at<Q>(
    query: &Q,
    scope: &str,
    statement: Option<&str>,
    at: OffsetDateTime,
) -> Result<Vec<ConclusionRow>>
where
    Q: GovernedQueryStore,
{
    query.conclusions_valid_at(scope, statement, at)
}

/// Prefer repository-scoped claim over workspace when both match statement and context is repo.
pub fn resolve_scope_preference(
    candidates: &[ConclusionRow],
    context_scope: &str,
) -> Option<ConclusionRow> {
    if candidates.is_empty() {
        return None;
    }
    if context_scope.starts_with("Repository:")
        && let Some(c) = candidates
            .iter()
            .find(|c| c.scope.starts_with("Repository:"))
    {
        return Some(c.clone());
    }
    if context_scope.starts_with("Workspace:")
        && let Some(c) = candidates
            .iter()
            .find(|c| c.scope.starts_with("Workspace:"))
    {
        return Some(c.clone());
    }
    candidates.first().cloned()
}

/// Decision (approved) beats agent candidate conclusion; candidate remains listed.
pub fn prefer_decision_over_candidate(
    decisions: &[crate::ports::DecisionRow],
    conclusions: &[ConclusionRow],
) -> (Option<crate::ports::DecisionRow>, Vec<ConclusionRow>) {
    let approved = decisions.iter().find(|d| d.state == "Approved").cloned();
    (approved, conclusions.to_vec())
}

/// Equal-authority **incompatible** claims under the same scope.
///
/// Detects conflict when statements **differ** (opposing claims), both share the
/// same authority band (Candidate/Active/Confirmed), and neither is terminal.
/// Does **not** merge prose — callers open a claim_conflict with two claim ids.
/// Identical statements are not treated as an incompatibility conflict.
pub fn equal_authority_conflict(
    a: &ConclusionRow,
    b: &ConclusionRow,
) -> Option<(String, String, String)> {
    if a.id == b.id || a.scope != b.scope {
        return None;
    }
    if a.statement == b.statement {
        return None;
    }
    if a.state == "Superseded"
        || b.state == "Superseded"
        || a.state == "Rejected"
        || b.state == "Rejected"
    {
        return None;
    }
    if a.protected_category != b.protected_category || a.unsupported != b.unsupported {
        return None;
    }
    // Same authority band (both candidates or both active/confirmed).
    if a.state != b.state {
        return None;
    }
    if !matches!(a.state.as_str(), "Candidate" | "Active" | "Confirmed") {
        return None;
    }
    Some((
        a.id.to_string(),
        b.id.to_string(),
        format!("equal-authority incompatible claims in scope {}", a.scope),
    ))
}

/// Current successor for superseded claim (projection superseded_by).
pub fn current_successor(row: &ConclusionRow) -> Option<&str> {
    row.superseded_by.as_deref()
}

pub fn open_conflicts_snapshot<Q>(query: &Q) -> Result<Vec<ClaimConflictRow>>
where
    Q: GovernedQueryStore,
{
    query.list_open_claim_conflicts()
}
