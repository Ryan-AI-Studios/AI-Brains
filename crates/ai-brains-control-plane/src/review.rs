//! Review item commands (T150 Phase E).

use ai_brains_core::ids::ReviewItemId;
use ai_brains_core::principal::Principal;
use ai_brains_core::privacy::Privacy;
use ai_brains_events::payload::ReviewItemResolvedPayload;
use ai_brains_events::{Actor, AggregateType, Payload};

use crate::errors::{ControlPlaneError, Result};
use crate::ports::{EventWriter, GovernedQueryStore};
use crate::sources::build_event;

pub fn resolve_review_item<W, Q>(
    writer: &W,
    query: &Q,
    principal: &Principal,
    review_item_id: ReviewItemId,
    reason: &str,
    privacy: Privacy,
) -> Result<()>
where
    W: EventWriter,
    Q: GovernedQueryStore,
{
    if reason.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "resolution reason must be non-empty".into(),
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
