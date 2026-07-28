//! Review item commands (T150 Phase E).
//!
//! # Idempotency (T159)
//!
//! When a review item is already non-`Open` (resolved), [`resolve_review_item`]
//! still enforces Human + `ApproveDecision` policy, then returns `Ok(())`
//! without appending a second `ReviewItemResolved` event. Missing items still
//! return [`ControlPlaneError::NotFound`]. Callers without the grant receive
//! [`ControlPlaneError::PolicyDenied`] even on already-resolved items.
//!
//! # Scope isolation (T160)
//!
//! Open review lists must not leak vault-wide: use [`review_item_matches_scope`]
//! / [`list_open_review_items_for_scope`] so CLI local and daemon share one filter.

use ai_brains_core::ids::{ConclusionId, DecisionId, ReviewItemId, SourceId};
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_events::payload::ReviewItemResolvedPayload;
use ai_brains_events::{Actor, AggregateType, Payload};
use uuid::Uuid;

use crate::errors::{ControlPlaneError, Result};
use crate::ports::{
    EventWriter, GovernedQueryStore, PolicyContext, PolicyEvaluator, ReviewItemRow,
};
use crate::sources::build_event;

/// True when an open review item is bound to `scope_key` (related conclusion /
/// decision / source scope, or free-text subject / subject_id containing the key).
///
/// Shared by daemon list and CLI local list so path choice cannot widen the grant.
pub fn review_item_matches_scope<Q: GovernedQueryStore>(
    query: &Q,
    item: &ReviewItemRow,
    scope_key: &str,
) -> Result<bool> {
    if let Some(ref cid) = item.related_conclusion_id
        && let Ok(uuid) = Uuid::parse_str(cid)
    {
        let id = ConclusionId::from_uuid(uuid);
        if let Some(row) = query.get_conclusion(id)? {
            return Ok(row.scope == scope_key);
        }
    }
    if let Some(ref did) = item.related_decision_id
        && let Ok(uuid) = Uuid::parse_str(did)
    {
        let id = DecisionId::from_uuid(uuid);
        if let Some(row) = query.get_decision(id)? {
            return Ok(row.scope == scope_key);
        }
    }
    if let Some(ref sid) = item.related_source_id
        && let Ok(uuid) = Uuid::parse_str(sid)
    {
        let id = SourceId::from_uuid(uuid);
        if let Some(row) = query.get_source(id)? {
            return Ok(row.scope == scope_key);
        }
    }
    // Fallback: subject text or subject_id mentions the scope identity key.
    if item.subject.contains(scope_key) || item.subject_id.contains(scope_key) {
        return Ok(true);
    }
    Ok(false)
}

/// Open review items bound to `scope_key` (vault-wide open list, then filter).
pub fn list_open_review_items_for_scope<Q: GovernedQueryStore>(
    query: &Q,
    scope_key: &str,
) -> Result<Vec<ReviewItemRow>> {
    let items = query.list_open_review_items()?;
    let mut scoped = Vec::with_capacity(items.len());
    for item in items {
        if review_item_matches_scope(query, &item, scope_key)? {
            scoped.push(item);
        }
    }
    Ok(scoped)
}

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
    // Missing → NotFound (before payload/policy so probes do not leak grant state).
    let row = query
        .get_review_item(review_item_id)?
        .ok_or_else(|| ControlPlaneError::NotFound(format!("review_item {review_item_id}")))?;

    // 1. Payload validation
    if reason.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "resolution reason must be non-empty".into(),
        ));
    }

    // 2. Principal + policy gates (always — including already-resolved replay)
    if !matches!(principal.kind, PrincipalKind::Human) {
        return Err(ControlPlaneError::ApprovalRequired(
            "review resolve requires human principal (not Agent)".into(),
        ));
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

    // 3. Detect-already-done: already resolved → success without second append
    if row.status != "Open" {
        return Ok(());
    }

    // 4. Append
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
