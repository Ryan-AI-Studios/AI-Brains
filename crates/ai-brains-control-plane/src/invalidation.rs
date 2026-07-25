//! Dependency invalidation workflow (T149 Phase E).
//!
//! - Dependents only → `ConclusionMarkedStale` (requires version **or** non-empty reason)
//! - Independent conclusions untouched
//! - Decisions: **never** auto-revoked; open structured `ReviewItemOpened`
//! - Source unavailable → `SourceUnavailable` + stale + structured review by criticality
//! - Revalidation with matching fingerprint clears stale **only** when the latest
//!   stale fact's version/reason matches this source revalidation (P2-4)
//! - Observation change path batches invalidation events with version/evidence (single TX)
//! - Always emit MarkedStale for a new version (audit queue rows written Processed)

use ai_brains_core::ids::{
    ConclusionId, DecisionId, PrincipalId, ReviewItemId, SourceId, SourceVersionId,
};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::review::{ReviewCriticality, ReviewSubjectKind};
use ai_brains_events::payload::{
    ConclusionActivatedPayload, ConclusionMarkedStalePayload, ReviewItemOpenedPayload,
    SourceUnavailablePayload,
};
use ai_brains_events::{Actor, AggregateType, Envelope, EventKind, Payload};

use crate::errors::{ControlPlaneError, Result};
use crate::ports::{Clock, EventWriter, GovernedQueryStore};
use crate::sources::build_event;

/// Outcome of processing dependents after a source version change.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvalidationResult {
    pub stale_conclusions: Vec<ConclusionId>,
    pub review_items_for_decisions: Vec<DecisionId>,
}

/// Pure-build stale + review envelopes for dependents of a changed source version.
///
/// Does **not** append. Callers batch these with version/evidence for a single TX
/// (T149-F1 / R4 atomicity).
///
/// Always emits `ConclusionMarkedStale` for each dependent (even if already stale)
/// so `invalidation_queue_projection` records a Processed audit row per version.
pub fn plan_invalidation_events_for_changed_source<Q>(
    query: &Q,
    source_id: SourceId,
    changed_version_id: SourceVersionId,
    opened_by: PrincipalId,
    privacy: Privacy,
) -> Result<(InvalidationResult, Vec<Envelope>)>
where
    Q: GovernedQueryStore,
{
    let actor = Actor::System;
    let mut result = InvalidationResult::default();
    let mut batch = Vec::new();

    let conclusions = query.conclusions_depending_on_source(source_id)?;
    let decisions = query.decisions_depending_on_source(source_id)?;

    for conclusion_id in &conclusions {
        let payload = ConclusionMarkedStalePayload::try_new_with_source(
            *conclusion_id,
            Some(changed_version_id),
            None,
            Some(source_id),
        )
        .map_err(|e| ControlPlaneError::InvalidPayload(e.to_string()))?;

        batch.push(build_event(
            AggregateType::Conclusion,
            conclusion_id.as_uuid(),
            EventKind::ConclusionMarkedStale,
            actor.clone(),
            privacy,
            Payload::ConclusionMarkedStale(payload),
        )?);
        result.stale_conclusions.push(*conclusion_id);
    }

    for decision_id in &decisions {
        let review_item_id = ReviewItemId::new();
        batch.push(build_event(
            AggregateType::ReviewItem,
            review_item_id.as_uuid(),
            EventKind::ReviewItemOpened,
            actor.clone(),
            privacy,
            Payload::ReviewItemOpened(ReviewItemOpenedPayload {
                review_item_id,
                subject: format!(
                    "Supporting source changed for decision {decision_id} (version {changed_version_id})"
                ),
                opened_by,
                subject_kind: ReviewSubjectKind::Decision,
                subject_id: decision_id.to_string(),
                criticality: ReviewCriticality::High,
                related_conclusion_id: None,
                related_decision_id: Some(*decision_id),
                related_source_id: Some(source_id),
            }),
        )?);
        result.review_items_for_decisions.push(*decision_id);
    }

    Ok((result, batch))
}

/// Mark only conclusions that depend on `source_id` as stale after a new version.
/// Decisions that depend on that source get a structured review item — never revoked.
///
/// Prefer batching via [`plan_invalidation_events_for_changed_source`] when combining
/// with observation version/evidence appends.
pub fn invalidate_dependents_for_changed_source<W, Q, C>(
    writer: &W,
    query: &Q,
    _clock: &C,
    source_id: SourceId,
    changed_version_id: SourceVersionId,
    opened_by: PrincipalId,
    privacy: Privacy,
) -> Result<InvalidationResult>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
{
    let (result, batch) = plan_invalidation_events_for_changed_source(
        query,
        source_id,
        changed_version_id,
        opened_by,
        privacy,
    )?;

    if !batch.is_empty() {
        writer.append_events(&batch)?;
    }

    Ok(result)
}

/// Inputs for [`mark_source_unavailable`].
#[derive(Debug, Clone)]
pub struct SourceUnavailableRequest {
    pub source_id: SourceId,
    pub reason: String,
    pub opened_by: PrincipalId,
    pub privacy: Privacy,
    pub criticality: ReviewCriticality,
}

/// Mark a source unavailable and invalidate dependents with `unavailable_reason`.
/// Opens a structured review item on the source (and decision reviews for dependents).
pub fn mark_source_unavailable<W, Q, C>(
    writer: &W,
    query: &Q,
    clock: &C,
    req: SourceUnavailableRequest,
) -> Result<InvalidationResult>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
{
    if req.reason.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "source unavailable reason must be non-empty".to_string(),
        ));
    }

    let source_id = req.source_id;
    let reason = req.reason;
    let opened_by = req.opened_by;
    let privacy = req.privacy;
    let criticality = req.criticality;

    let now = clock.now()?;
    let actor = Actor::System;
    let mut result = InvalidationResult::default();
    let mut batch = Vec::new();

    batch.push(build_event(
        AggregateType::Source,
        source_id.as_uuid(),
        EventKind::SourceUnavailable,
        actor.clone(),
        privacy,
        Payload::SourceUnavailable(SourceUnavailablePayload {
            source_id,
            reason: reason.clone(),
            marked_at: now,
        }),
    )?);

    // Structured review on the source itself.
    let source_review_id = ReviewItemId::new();
    batch.push(build_event(
        AggregateType::ReviewItem,
        source_review_id.as_uuid(),
        EventKind::ReviewItemOpened,
        actor.clone(),
        privacy,
        Payload::ReviewItemOpened(ReviewItemOpenedPayload {
            review_item_id: source_review_id,
            subject: format!("Source unavailable: {reason}"),
            opened_by,
            subject_kind: ReviewSubjectKind::Source,
            subject_id: source_id.to_string(),
            criticality,
            related_conclusion_id: None,
            related_decision_id: None,
            related_source_id: Some(source_id),
        }),
    )?);

    let conclusions = query.conclusions_depending_on_source(source_id)?;
    let decisions = query.decisions_depending_on_source(source_id)?;

    for conclusion_id in &conclusions {
        // Encode source_id in reason for historical parsers; also set payload.source_id.
        let reason_with_source = format!("source:{source_id}: {reason}");
        let payload = ConclusionMarkedStalePayload::try_new_with_source(
            *conclusion_id,
            None,
            Some(reason_with_source),
            Some(source_id),
        )
        .map_err(|e| ControlPlaneError::InvalidPayload(e.to_string()))?;

        batch.push(build_event(
            AggregateType::Conclusion,
            conclusion_id.as_uuid(),
            EventKind::ConclusionMarkedStale,
            actor.clone(),
            privacy,
            Payload::ConclusionMarkedStale(payload),
        )?);
        result.stale_conclusions.push(*conclusion_id);
    }

    for decision_id in &decisions {
        let review_item_id = ReviewItemId::new();
        batch.push(build_event(
            AggregateType::ReviewItem,
            review_item_id.as_uuid(),
            EventKind::ReviewItemOpened,
            actor.clone(),
            privacy,
            Payload::ReviewItemOpened(ReviewItemOpenedPayload {
                review_item_id,
                subject: format!(
                    "Supporting source unavailable for decision {decision_id}: {reason}"
                ),
                opened_by,
                subject_kind: ReviewSubjectKind::Decision,
                subject_id: decision_id.to_string(),
                criticality,
                related_conclusion_id: None,
                related_decision_id: Some(*decision_id),
                related_source_id: Some(source_id),
            }),
        )?);
        result.review_items_for_decisions.push(*decision_id);
    }

    writer.append_events(&batch)?;
    Ok(result)
}

/// When a source is re-observed with a fingerprint matching its latest version,
/// clear stale conclusions that depend on this source by appending
/// `ConclusionActivated` (revalidation clear path; no dedicated revalidated event).
///
/// Clears only when the latest active stale fact matches:
/// - `changed_source_version_id` equals the source's **latest** version id, OR
/// - unavailable path for **this** source: `fact.source_id == Some(source_id)`
///   or `unavailable_reason` contains the source_id string (historical rows)
///
/// A conclusion stale due to an *older* source version (superseded by later
/// MarkedStale events) only clears when its latest fact version matches latest.
/// Unrelated-source unavailable (e.g. source A unavailable while revalidating B)
/// is not cleared.
pub fn revalidate_matching_stale<W, Q, C>(
    writer: &W,
    query: &Q,
    _clock: &C,
    source_id: SourceId,
    fingerprint: &str,
    _opened_by: PrincipalId,
    privacy: Privacy,
) -> Result<Vec<ConclusionId>>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
{
    // Only clear when fingerprint matches the latest recorded version.
    let latest = query.latest_source_version(source_id)?;
    let Some((latest_version_id, latest_fp)) = latest else {
        return Ok(Vec::new());
    };
    if latest_fp != fingerprint {
        return Ok(Vec::new());
    }

    let conclusions = query.conclusions_depending_on_source(source_id)?;
    let actor = Actor::System;
    let mut cleared = Vec::new();
    let mut batch = Vec::new();
    let source_id_str = source_id.to_string();

    for conclusion_id in conclusions {
        let Some(fact) = query.latest_stale_fact(conclusion_id)? else {
            continue;
        };

        let matches_version = fact
            .changed_source_version_id
            .is_some_and(|vid| vid == latest_version_id);
        // Unavailable-reason stale clears only when it is for this source.
        let matches_unavailable = fact.changed_source_version_id.is_none()
            && fact
                .unavailable_reason
                .as_ref()
                .is_some_and(|r| !r.trim().is_empty())
            && (fact.source_id == Some(source_id)
                || fact
                    .unavailable_reason
                    .as_ref()
                    .is_some_and(|r| r.contains(&source_id_str)));

        if !matches_version && !matches_unavailable {
            continue;
        }

        batch.push(build_event(
            AggregateType::Conclusion,
            conclusion_id.as_uuid(),
            EventKind::ConclusionActivated,
            actor.clone(),
            privacy,
            Payload::ConclusionActivated(ConclusionActivatedPayload { conclusion_id }),
        )?);
        cleared.push(conclusion_id);
    }

    if !batch.is_empty() {
        writer.append_events(&batch)?;
    }

    Ok(cleared)
}

/// Construct a validated stale payload or return InvalidPayload.
/// Used by unit tests to reassert deferred #14 from the control-plane boundary.
pub fn try_mark_stale_payload(
    conclusion_id: ConclusionId,
    changed_source_version_id: Option<SourceVersionId>,
    unavailable_reason: Option<String>,
) -> Result<ConclusionMarkedStalePayload> {
    ConclusionMarkedStalePayload::try_new(
        conclusion_id,
        changed_source_version_id,
        unavailable_reason,
    )
    .map_err(|e| ControlPlaneError::InvalidPayload(e.to_string()))
}
