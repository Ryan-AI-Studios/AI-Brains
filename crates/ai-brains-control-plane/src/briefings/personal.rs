//! Deterministic Personal Continuity Briefing (T152 Phase D).

use ai_brains_contracts::briefings::{
    AppliedGrantDto, BriefingWarningDto, BudgetReportDto, ContinuitySummaryDto,
    PersonalContinuityBriefingPacket, PersonalPreferenceDto, PersonalReviewItemDto,
};
use ai_brains_contracts::knowledge::EvidenceHandle;
use ai_brains_contracts::offset_to_utc;
use ai_brains_core::ids::{BriefingId, UserId};
use ai_brains_core::principal::Principal;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_events::payload::BriefingGeneratedPayload;
use ai_brains_events::{Actor, AggregateType, Payload};

use crate::briefings::budget::{BudgetConfig, apply_personal_budget};
use crate::errors::{ControlPlaneError, Result};
use crate::ports::{
    Clock, EventWriter, GovernedQueryStore, PolicyContext, PolicyEvaluator, ReviewItemRow,
};
use crate::sources::{build_event, scope_identity_key};

/// True when the open review item is bound to Personal scope (related conclusion/decision
/// scope_key, or free-text subject matching the personal key).
fn review_item_is_personal_related<Q: GovernedQueryStore>(
    query: &Q,
    item: &ReviewItemRow,
    personal_scope_key: &str,
) -> Result<bool> {
    if let Some(ref cid) = item.related_conclusion_id
        && let Ok(uuid) = uuid::Uuid::parse_str(cid)
    {
        let id = ai_brains_core::ids::ConclusionId::from_uuid(uuid);
        if let Some(row) = query.get_conclusion(id)? {
            return Ok(row.scope == personal_scope_key);
        }
    }
    if let Some(ref did) = item.related_decision_id
        && let Ok(uuid) = uuid::Uuid::parse_str(did)
    {
        let id = ai_brains_core::ids::DecisionId::from_uuid(uuid);
        if let Some(row) = query.get_decision(id)? {
            return Ok(row.scope == personal_scope_key);
        }
    }
    // Fallback: subject text or subject_id mentions Personal scope_key.
    if item.subject.contains(personal_scope_key) || item.subject_id.contains(personal_scope_key) {
        return Ok(true);
    }
    // Explicit personal subject kind (when stored that way).
    if item.subject_kind.eq_ignore_ascii_case("Personal") {
        return Ok(true);
    }
    Ok(false)
}

/// Inputs for a personal continuity briefing.
#[derive(Debug, Clone)]
pub struct PersonalBriefingRequest {
    pub principal: Principal,
    /// Personal user scope (must be Personal).
    pub user_id: UserId,
    pub budget: BudgetConfig,
    pub privacy: Privacy,
    pub dry_run: bool,
    pub briefing_id: Option<BriefingId>,
}

/// Build a Personal Continuity Briefing.
///
/// - Only Personal scope data.
/// - Preferences require [`GrantCapability::ReadConclusions`] on Personal scope
///   (T152-P1-04: `ReadDecisions` alone must not populate conclusions preferences).
/// - Open review items / grants listing still need at least one Personal read grant
///   (`ReadConclusions` or `ReadDecisions`).
/// - Never returns Personal content when the caller requested only Project without grant
///   (this function always requires Personal scope + grant).
pub fn build_personal_briefing<W, Q, C, P>(
    writer: Option<&W>,
    query: &Q,
    clock: &C,
    policy: &P,
    list_grants: impl Fn(Principal) -> Result<Vec<AppliedGrantDto>>,
    req: PersonalBriefingRequest,
) -> Result<PersonalContinuityBriefingPacket>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
{
    let briefing_id = req.briefing_id.unwrap_or_default();
    let now = clock.now()?;
    let scope = ScopeRef::Personal(req.user_id);
    let scope_key = scope_identity_key(&scope);
    let policy_ctx = PolicyContext::default_for_privacy(req.privacy);

    let can_read_conclusions = policy.allow(
        req.principal.id,
        GrantCapability::ReadConclusions,
        &scope,
        &policy_ctx,
    )?;
    let can_read_decisions = policy.allow(
        req.principal.id,
        GrantCapability::ReadDecisions,
        &scope,
        &policy_ctx,
    )?;
    // Entry to Personal packet surface requires some Personal read grant.
    let can_enter = can_read_conclusions || can_read_decisions;

    if !can_enter {
        // F7: empty_denied already seeds kind=denied — do not push a second denied warning.
        let mut packet = PersonalContinuityBriefingPacket::empty_denied(
            briefing_id.to_string(),
            scope_key,
            "Personal scope read denied without grant",
        );
        // T263 F4: Personal deny hint names recall (not bootstrap). Contracts leave None.
        packet.denial_hint =
            Some(super::renderer::BRIEFING_PERSONAL_DENIED_DENIAL_HINT.to_string());
        packet.generated_at = Some(offset_to_utc(now));
        apply_personal_budget(&mut packet, req.budget);
        return Ok(packet);
    }

    let grants_applied = list_grants(req.principal.clone())?;

    // Preferences: Confirmed personal-scope only (spec §3.1; T152-FRESH-P1-02).
    // Valid-time window must cover `now`. ReadConclusions required — decisions-only
    // grant must not populate conclusions-derived preferences.
    let mut preferences = Vec::new();
    let mut included_privacies: Vec<Privacy> = Vec::new();
    if can_read_conclusions {
        let rows = query.list_conclusions_by_scope_state(Some(&scope_key), Some("Confirmed"))?;
        for row in rows {
            if !personal_conclusion_valid_at(&row, now) {
                continue;
            }
            let eids = query.evidence_ids_for_conclusion(row.id)?;
            let handles: Vec<EvidenceHandle> = eids
                .into_iter()
                .map(|e| EvidenceHandle {
                    evidence_id: e.to_string(),
                    cite_label: None,
                })
                .collect();
            if handles.is_empty() {
                continue;
            }
            included_privacies.push(crate::briefings::project::parse_stored_privacy(
                &row.privacy,
            ));
            preferences.push(PersonalPreferenceDto {
                id: row.id.to_string(),
                statement: row.statement,
                evidence_handles: handles,
            });
        }
    }

    // Open review items: Personal-related only (related conclusion/decision scope, or
    // subject/scope_key binding). Project-scoped review must not bleed into Personal packets.
    // Residual: continuity summary remains empty until #18 session synthesis lands.
    let mut open_review_items = Vec::new();
    for item in query.list_open_review_items()? {
        if !review_item_is_personal_related(query, &item, &scope_key)? {
            continue;
        }
        open_review_items.push(PersonalReviewItemDto {
            id: item.id.to_string(),
            subject: item.subject,
            criticality: item.criticality,
            status: item.status,
        });
    }

    let mut packet = PersonalContinuityBriefingPacket {
        api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
        briefing_id: briefing_id.to_string(),
        kind: "Personal".to_string(),
        scope_key,
        preferences,
        continuity: ContinuitySummaryDto {
            summary: String::new(),
            thread_handles: Vec::new(),
        },
        open_review_items,
        grants_applied,
        warnings: Vec::new(),
        budget: BudgetReportDto {
            max_words: req.budget.max_words,
            used_words: 0,
            truncated_sections: Vec::new(),
            more_available: false,
        },
        generated_at: Some(offset_to_utc(now)),
        denied: false,
        denial_reason: None,
        denial_hint: None,
    };
    // T227 F9/F27: empty_continuity only when allowed and continuity summary empty.
    if packet.continuity.summary.is_empty() {
        packet.warnings.push(BriefingWarningDto {
            kind: "empty_continuity".into(),
            message:
                "Personal continuity summary is empty (session synthesis deferred; no synthetic fill)"
                    .into(),
            subject_id: None,
            subject_kind: None,
        });
    }
    apply_personal_budget(&mut packet, req.budget);

    if !req.dry_run
        && let Some(writer) = writer
    {
        let bid = BriefingId::from_uuid(
            uuid::Uuid::parse_str(&packet.briefing_id)
                .map_err(|e| ControlPlaneError::InvalidPayload(e.to_string()))?,
        );
        // T152-FRESH3-P1-01: strictest of request + included preference + evidence privacy.
        let mut emit_privacy = req.privacy;
        for p in included_privacies {
            emit_privacy = emit_privacy.combine(p);
        }
        let mut evidence_ids = Vec::new();
        let mut seen_eids = std::collections::BTreeSet::new();
        for pref in &packet.preferences {
            for h in &pref.evidence_handles {
                if let Ok(uuid) = uuid::Uuid::parse_str(&h.evidence_id) {
                    let eid = ai_brains_core::ids::EvidenceId::from_uuid(uuid);
                    if seen_eids.insert(eid.to_string()) {
                        if let Some(raw) = query.evidence_privacy(eid)? {
                            emit_privacy = emit_privacy
                                .combine(crate::briefings::project::parse_stored_privacy(&raw));
                        }
                        evidence_ids.push(eid);
                    }
                }
            }
        }
        let event = build_event(
            AggregateType::Briefing,
            bid.as_uuid(),
            Actor::System,
            emit_privacy,
            Payload::BriefingGenerated(BriefingGeneratedPayload {
                briefing_id: bid,
                kind: "Personal".into(),
                evidence_ids,
                query_trace_id: None,
            }),
        )?;
        writer.append_events(&[event])?;
    }

    Ok(packet)
}

/// Personal preference valid-time: valid_from ≤ at < valid_until|∞.
fn personal_conclusion_valid_at(
    row: &crate::ports::ConclusionRow,
    at: time::OffsetDateTime,
) -> bool {
    row.valid_from <= at && row.valid_until.map(|u| u > at).unwrap_or(true)
}
