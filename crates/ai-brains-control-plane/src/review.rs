//! Review item commands (T150 Phase E).

use ai_brains_core::ids::ReviewItemId;
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_events::payload::ReviewItemResolvedPayload;
use ai_brains_events::{Actor, AggregateType, Payload};

use crate::errors::{ControlPlaneError, Result};
use crate::ports::{EventWriter, GovernedQueryStore, PolicyContext, PolicyEvaluator};
use crate::sources::build_event;

/// Resolve an open review item.
///
/// Policy: human gate (`PrincipalKind::Human`) + `ApproveDecision` on the caller's
/// scope — same posture as decision approval. Agents hard-deny Approve* via
/// [`crate::DefaultPolicyEvaluator`]. Review items do not carry a first-class scope
/// column, so the caller supplies the governing scope for grant lookup.
#[allow(clippy::too_many_arguments)]
pub fn resolve_review_item<W, Q, P>(
    writer: &W,
    query: &Q,
    policy: &P,
    principal: &Principal,
    review_item_id: ReviewItemId,
    reason: &str,
    privacy: Privacy,
    scope: ScopeRef,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    P: PolicyEvaluator,
{
    if reason.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "resolution reason must be non-empty".into(),
        ));
    }

    if !matches!(principal.kind, PrincipalKind::Human) {
        return Err(ControlPlaneError::ApprovalRequired(
            "review resolve requires human principal (not Agent)".into(),
        ));
    }

    let row = query
        .get_review_item(review_item_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("review_item {review_item_id}")))?;

    if row.status != "Open" {
        return Err(ControlPlaneError::InvalidTransition(format!(
            "review item {review_item_id} is {}",
            row.status
        )));
    }

    let policy_ctx = PolicyContext::default_for_privacy(privacy);
    if !policy.allow(
        principal.id,
        GrantCapability::ApproveDecision,
        &scope,
        &policy_ctx,
    )? {
        return Err(ControlPlaneError::PolicyDenied(
            "ApproveDecision denied for review resolve".into(),
        ));
    }

    let event = build_event(
        AggregateType::ReviewItem,
        review_item_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::ReviewItemResolved(ReviewItemResolvedPayload {
            review_item_id,
            resolution: reason.to_string(),
            resolved_by: principal.id,
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(())
}
