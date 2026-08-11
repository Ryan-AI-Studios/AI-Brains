//! Deterministic Project Briefing builder (T152 Phase C).
//!
//! Selection order: **policy → lifecycle/freshness/valid-time → authority → relevance**.
//! No LLM. Stale/Disputed/Rejected never appear as current authority.

use ai_brains_contracts::briefings::{
    BriefingClaimDto, BriefingConstraintDto, BriefingScopeDto, BriefingWarningDto, BudgetReportDto,
    FreshnessSummaryDto, LedgerfulSectionDto, ProjectBriefingPacket,
};
use ai_brains_contracts::knowledge::EvidenceHandle;
use ai_brains_contracts::offset_to_utc;
use ai_brains_core::briefing::{
    AuthoritativeClaimKind, AuthoritativeClaimRef, validate_authoritative_claims,
};
use ai_brains_core::ids::{BriefingId, PrincipalId};
use ai_brains_core::principal::Principal;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_events::payload::BriefingGeneratedPayload;
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::projections::briefing::briefing_cache_key;
use time::format_description::well_known::Rfc3339;

use crate::briefings::budget::{BudgetConfig, apply_budget};
use crate::errors::{ControlPlaneError, Result};
use crate::ports::{Clock, EventWriter, GovernedQueryStore, PolicyContext, PolicyEvaluator};
use crate::scope_resolver::{
    ResolvedScope, ScopeConfidence, ScopeIdentityStore, ScopeResolveInput, is_authoritative,
    resolve_scope,
};
use crate::sources::{build_event, scope_identity_key};

/// Policy version label embedded in briefing cache keys (bump when evaluator semantics change).
pub const BRIEFING_POLICY_VERSION: &str = "DefaultPolicyEvaluator-v1";

/// Inputs for a project briefing generation.
#[derive(Debug, Clone)]
pub struct ProjectBriefingRequest {
    pub principal: Principal,
    pub resolve: ScopeResolveInput,
    pub budget: BudgetConfig,
    pub privacy: Privacy,
    /// When true, do not emit `BriefingGenerated`.
    pub dry_run: bool,
    /// Optional pre-assigned briefing id (tests).
    pub briefing_id: Option<BriefingId>,
    /// Optional ledgerful section (caller-supplied; degraded if None).
    pub ledgerful: Option<LedgerfulSectionDto>,
}

/// Build a deterministic Project Briefing packet.
///
/// - Uses T151 `resolve_scope` + `is_authoritative` (#20).
/// - Low/Ambiguous non-authoritative scopes: empty current authority + warnings.
/// - Unauthorized principal: denied / empty sections.
/// - Current lists: Approved decisions; Active/Confirmed conclusions only.
/// - Stale/Disputed/Rejected → warnings only.
pub fn build_project_briefing<W, Q, C, P, I>(
    writer: Option<&W>,
    query: &Q,
    clock: &C,
    policy: &P,
    identity: &I,
    req: ProjectBriefingRequest,
) -> Result<ProjectBriefingPacket>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
    I: ScopeIdentityStore,
{
    let briefing_id = req.briefing_id.unwrap_or_default();
    let now = clock.now()?;
    let resolved = resolve_scope(&req.resolve, identity)?;
    let scope_dto = scope_dto_from_resolved(&resolved);
    let scope_key = scope_identity_key(&resolved.scope);

    // Cache lookup (version vector advances on conclusion/decision/conflict/grant change).
    // Principal is folded into the scope segment so cross-principal keys never collide;
    // grant epoch inside the version vector forces miss on issue/revoke (T152-R2-01).
    // Skipped for dry_run so tests/preflight never write or serve cache side-effects.
    let version_vector =
        query.epistemic_version_vector(&scope_key, &req.principal.id.to_string())?;
    let cache_scope = format!("{scope_key}|principal:{}", req.principal.id);
    let cache_key = briefing_cache_key(
        "Project",
        &cache_scope,
        BRIEFING_POLICY_VERSION,
        &version_vector,
        req.budget.max_words as u64,
    );
    if !req.dry_run
        && let Some((packet_json, expires)) = query.get_briefing_cache(&cache_key)?
    {
        let expired = match expires.as_deref() {
            Some(exp) => match time::OffsetDateTime::parse(exp, &Rfc3339) {
                Ok(exp_ts) => exp_ts <= now,
                Err(_) => true, // unparseable expiry → treat as miss
            },
            None => false,
        };
        if !expired {
            match serde_json::from_str::<ProjectBriefingPacket>(&packet_json) {
                Ok(mut packet) => {
                    // T152-P1-01: never serve a High-authority cache hit when the
                    // current resolution is non-authoritative (Low/Ambiguous/sentinel).
                    // Scope identity alone is not enough — confidence can change while
                    // the scope_key (and thus cache key) stays the same.
                    if is_authoritative(&resolved)
                        && cache_hit_still_authorized(policy, &req, &resolved.scope, &packet)?
                    {
                        // T152-FRESH-P1-01: re-filter valid-time on hit so a claim that
                        // was current when cached is not served after valid_until.
                        refilter_cached_packet_valid_time(query, &mut packet, now)?;
                        packet.generated_at = Some(offset_to_utc(now));
                        return Ok(packet);
                    }
                    // Non-authoritative resolution or policy no longer justifies
                    // cached authority → rebuild (empty/low-authority path).
                }
                Err(e) => {
                    return Err(ControlPlaneError::Query(format!(
                        "corrupt briefing cache packet_json for key {cache_key}: {e}"
                    )));
                }
            }
        }
    }

    // #20 — non-authoritative scope: refuse high-authority injection.
    if !is_authoritative(&resolved) {
        let mut packet = ProjectBriefingPacket {
            api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
            briefing_id: briefing_id.to_string(),
            kind: "Project".to_string(),
            scope: scope_dto,
            handoff: None,
            decisions: Vec::new(),
            conclusions: Vec::new(),
            constraints: Vec::new(),
            warnings: vec![BriefingWarningDto {
                kind: "low_confidence".into(),
                message: "Scope is not authoritative; high-authority claims withheld (#20)".into(),
                subject_id: None,
                subject_kind: None,
            }],
            freshness: empty_freshness(),
            ledgerful: req.ledgerful.or_else(|| {
                Some(LedgerfulSectionDto {
                    hotspots: Vec::new(),
                    impact_notes: Vec::new(),
                    degraded: true,
                })
            }),
            evidence_handles: Vec::new(),
            budget: BudgetReportDto {
                max_words: req.budget.max_words,
                used_words: 0,
                truncated_sections: Vec::new(),
                more_available: false,
            },
            generated_at: Some(offset_to_utc(now)),
            denied: false,
            denial_reason: None,
        };
        // Carry resolver warnings.
        for w in &resolved.warnings {
            packet.warnings.push(BriefingWarningDto {
                kind: "other".into(),
                message: w.clone(),
                subject_id: None,
                subject_kind: None,
            });
        }
        // T227 F8: non-authoritative is !denied with empty authority.
        packet.warnings.push(BriefingWarningDto {
            kind: "empty_authority".into(),
            message: "No current Approved decisions or Active/Confirmed conclusions at this scope"
                .into(),
            subject_id: None,
            subject_kind: None,
        });
        apply_budget(&mut packet, req.budget);
        maybe_emit_briefing(writer, req.dry_run, &packet, req.privacy)?;
        return Ok(packet);
    }

    // Personal scope is not a Project packet (caller should use personal briefing).
    if matches!(resolved.scope, ScopeRef::Personal(_)) {
        return Ok(ProjectBriefingPacket::empty_denied(
            briefing_id.to_string(),
            scope_dto,
            "Project briefing refuses Personal scope; use Personal Continuity Briefing",
        ));
    }

    let policy_ctx = PolicyContext::default_for_privacy(req.privacy);
    let can_read_decisions = policy.allow(
        req.principal.id,
        GrantCapability::ReadDecisions,
        &resolved.scope,
        &policy_ctx,
    )?;
    let can_read_conclusions = policy.allow(
        req.principal.id,
        GrantCapability::ReadConclusions,
        &resolved.scope,
        &policy_ctx,
    )?;

    if !can_read_decisions && !can_read_conclusions {
        // F7: empty_denied already seeds kind=denied — do not push a second denied warning.
        let mut packet = ProjectBriefingPacket::empty_denied(
            briefing_id.to_string(),
            scope_dto,
            "ReadDecisions/ReadConclusions denied for principal at scope",
        );
        packet.generated_at = Some(offset_to_utc(now));
        apply_budget(&mut packet, req.budget);
        maybe_emit_briefing(writer, req.dry_run, &packet, req.privacy)?;
        return Ok(packet);
    }

    let mut decisions = Vec::new();
    let mut conclusions = Vec::new();
    let mut warnings = Vec::new();
    let mut evidence_handles: Vec<EvidenceHandle> = Vec::new();
    let mut auth_claims: Vec<AuthoritativeClaimRef> = Vec::new();
    let mut stale_count = 0u32;
    let mut current_count = 0u32;

    // --- Decisions: Approved only for current authority; valid-time window must cover now ---
    if can_read_decisions {
        let rows = query.list_decisions(Some(&scope_key), Some("Approved"))?;
        for row in rows {
            // T152-P1-02: exclude future / expired decisions from current authority.
            if !decision_valid_at(&row, now) {
                warnings.push(BriefingWarningDto {
                    kind: "out_of_valid_time".into(),
                    message: format!(
                        "Decision {} outside valid-time window at briefing time; omitted from current authority",
                        row.id
                    ),
                    subject_id: Some(row.id.to_string()),
                    subject_kind: Some("Decision".into()),
                });
                continue;
            }
            let mut handles = evidence_handles_for_decision(query, row.id)?;
            // Decision handle itself counts when linked conclusions exist without evidence.
            if handles.is_empty() {
                let cids = query.conclusion_ids_for_decision(row.id)?;
                for cid in cids {
                    handles.push(EvidenceHandle {
                        evidence_id: format!("decision-support:conclusion:{cid}"),
                        cite_label: Some("decision_support".into()),
                    });
                }
            }
            if handles.is_empty() {
                // Still require a handle: use decision id as synthetic authority handle.
                handles.push(EvidenceHandle {
                    evidence_id: format!("decision:{}", row.id),
                    cite_label: Some("decision".into()),
                });
            }
            auth_claims.push(AuthoritativeClaimRef {
                kind: AuthoritativeClaimKind::Decision,
                id: row.id.to_string(),
                evidence_handles: handles.iter().map(|h| h.evidence_id.clone()).collect(),
            });
            for h in &handles {
                push_unique_handle(&mut evidence_handles, h.clone());
            }
            decisions.push(BriefingClaimDto {
                id: row.id.to_string(),
                kind: "Decision".into(),
                statement: row.statement,
                state: row.state,
                evidence_handles: handles,
                title: Some(row.title),
            });
            current_count = current_count.saturating_add(1);
        }
    } else {
        warnings.push(BriefingWarningDto {
            kind: "denied".into(),
            message: "ReadDecisions denied; decisions section empty".into(),
            subject_id: None,
            subject_kind: Some("Decision".into()),
        });
    }

    // --- Conclusions: Active/Confirmed current; Stale/Disputed/Rejected → warnings ---
    // T152-P1-02: valid_from/valid_until must cover `now` for current authority.
    if can_read_conclusions {
        for state in ["Active", "Confirmed"] {
            let rows = query.list_conclusions_by_scope_state(Some(&scope_key), Some(state))?;
            for row in rows {
                if !conclusion_valid_at(&row, now) {
                    warnings.push(BriefingWarningDto {
                        kind: "out_of_valid_time".into(),
                        message: format!(
                            "Conclusion {} outside valid-time window at briefing time; omitted from current authority",
                            row.id
                        ),
                        subject_id: Some(row.id.to_string()),
                        subject_kind: Some("Conclusion".into()),
                    });
                    continue;
                }
                let handles = evidence_handles_for_conclusion(query, row.id)?;
                let handle_dtos = if handles.is_empty() {
                    // Unsupported conclusions should not be current authority without handles.
                    warnings.push(BriefingWarningDto {
                        kind: "other".into(),
                        message: format!(
                            "Conclusion {} lacks evidence handles; omitted from current authority",
                            row.id
                        ),
                        subject_id: Some(row.id.to_string()),
                        subject_kind: Some("Conclusion".into()),
                    });
                    continue;
                } else {
                    handles
                };
                auth_claims.push(AuthoritativeClaimRef {
                    kind: AuthoritativeClaimKind::Conclusion,
                    id: row.id.to_string(),
                    evidence_handles: handle_dtos.iter().map(|h| h.evidence_id.clone()).collect(),
                });
                for h in &handle_dtos {
                    push_unique_handle(&mut evidence_handles, h.clone());
                }
                conclusions.push(BriefingClaimDto {
                    id: row.id.to_string(),
                    kind: "Conclusion".into(),
                    statement: row.statement,
                    state: row.state,
                    evidence_handles: handle_dtos,
                    title: None,
                });
                current_count = current_count.saturating_add(1);
            }
        }

        for state in ["Stale", "Disputed", "Rejected"] {
            let rows = query.list_conclusions_by_scope_state(Some(&scope_key), Some(state))?;
            for row in rows {
                if state == "Stale" {
                    stale_count = stale_count.saturating_add(1);
                }
                warnings.push(BriefingWarningDto {
                    kind: state.to_ascii_lowercase(),
                    message: format!(
                        "Conclusion {} is {state} (not current authority): {}",
                        row.id, row.statement
                    ),
                    subject_id: Some(row.id.to_string()),
                    subject_kind: Some("Conclusion".into()),
                });
            }
        }
    } else {
        warnings.push(BriefingWarningDto {
            kind: "denied".into(),
            message: "ReadConclusions denied; conclusions section empty".into(),
            subject_id: None,
            subject_kind: Some("Conclusion".into()),
        });
    }

    // Open claim conflicts in this scope → warnings only when the principal can
    // read *both* referenced claim kinds (T152-FRESH-P1-03). Otherwise the
    // explanation can leak conclusion text to a decisions-only principal.
    for conflict in query.list_open_claim_conflicts()? {
        if conflict.scope != scope_key {
            continue;
        }
        if !can_read_conflict_claim_kind(
            &conflict.claim_a_kind,
            can_read_decisions,
            can_read_conclusions,
        ) || !can_read_conflict_claim_kind(
            &conflict.claim_b_kind,
            can_read_decisions,
            can_read_conclusions,
        ) {
            continue;
        }
        warnings.push(BriefingWarningDto {
            kind: "open_conflict".into(),
            message: format!(
                "Open claim conflict {}: {}",
                conflict.id, conflict.explanation
            ),
            subject_id: Some(conflict.id.to_string()),
            subject_kind: Some("ClaimConflict".into()),
        });
    }

    // Validate authoritative claims have handles.
    validate_authoritative_claims(&auth_claims).map_err(|e| {
        ControlPlaneError::InvalidPayload(format!("briefing authority validation: {e}"))
    })?;

    // Constraints: tagged conclusions with CONSTRAINT-style statements (best-effort).
    let constraints = extract_constraints(&conclusions);

    let freshness = FreshnessSummaryDto {
        total_sources: current_count.saturating_add(stale_count),
        fresh_count: current_count,
        stale_count,
        unavailable_count: 0,
        worst_state: if stale_count > 0 {
            "Stale".into()
        } else if current_count > 0 {
            "Fresh".into()
        } else {
            "Unknown".into()
        },
    };

    let mut packet = ProjectBriefingPacket {
        api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
        briefing_id: briefing_id.to_string(),
        kind: "Project".to_string(),
        scope: scope_dto,
        handoff: None,
        decisions,
        conclusions,
        constraints,
        warnings,
        freshness,
        ledgerful: req.ledgerful.or_else(|| {
            Some(LedgerfulSectionDto {
                hotspots: Vec::new(),
                impact_notes: Vec::new(),
                degraded: true,
            })
        }),
        evidence_handles,
        budget: BudgetReportDto {
            max_words: req.budget.max_words,
            used_words: 0,
            truncated_sections: Vec::new(),
            more_available: false,
        },
        generated_at: Some(offset_to_utc(now)),
        denied: false,
        denial_reason: None,
    };

    // T227 F8/F27: empty_authority only when allowed and both authority sections empty.
    if packet.decisions.is_empty() && packet.conclusions.is_empty() {
        packet.warnings.push(BriefingWarningDto {
            kind: "empty_authority".into(),
            message: "No current Approved decisions or Active/Confirmed conclusions at this scope"
                .into(),
            subject_id: None,
            subject_kind: None,
        });
    }

    apply_budget(&mut packet, req.budget);

    // T152-FRESH-P1-04: envelope privacy is strictest of request + included claims.
    let emit_privacy = derived_briefing_privacy(req.privacy, query, &packet)?;
    maybe_emit_briefing(writer, req.dry_run, &packet, emit_privacy)?;
    if !req.dry_run {
        put_briefing_cache_if_enabled(
            query,
            &cache_key,
            &cache_scope,
            &version_vector,
            &packet,
            now,
        )?;
    }
    Ok(packet)
}

/// Whether a cached packet may still be served under current policy grants.
///
/// Returns false when:
/// - principal no longer has any read grant and the packet is not a denied shell, or
/// - packet carries decisions/conclusions the principal can no longer read, or
/// - packet is denied but principal now has read grants (must rebuild for authority).
fn cache_hit_still_authorized<P: PolicyEvaluator>(
    policy: &P,
    req: &ProjectBriefingRequest,
    scope: &ScopeRef,
    packet: &ProjectBriefingPacket,
) -> Result<bool> {
    let policy_ctx = PolicyContext::default_for_privacy(req.privacy);
    let can_read_decisions = policy.allow(
        req.principal.id,
        GrantCapability::ReadDecisions,
        scope,
        &policy_ctx,
    )?;
    let can_read_conclusions = policy.allow(
        req.principal.id,
        GrantCapability::ReadConclusions,
        scope,
        &policy_ctx,
    )?;

    if packet.denied {
        // Denied cache only valid while still fully denied.
        return Ok(!can_read_decisions && !can_read_conclusions);
    }

    // Authorized packet: never serve authority sections the principal can no longer read.
    if !can_read_decisions && !can_read_conclusions {
        return Ok(false);
    }
    if !packet.decisions.is_empty() && !can_read_decisions {
        return Ok(false);
    }
    if !packet.conclusions.is_empty() && !can_read_conclusions {
        return Ok(false);
    }
    Ok(true)
}

fn put_briefing_cache_if_enabled<Q: GovernedQueryStore>(
    query: &Q,
    cache_key: &str,
    cache_scope: &str,
    version_vector: &str,
    packet: &ProjectBriefingPacket,
    now: time::OffsetDateTime,
) -> Result<()> {
    let packet_json = serde_json::to_string(packet)
        .map_err(|e| ControlPlaneError::Query(format!("serialize briefing for cache: {e}")))?;
    let generated_at = now
        .format(&Rfc3339)
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
    query.put_briefing_cache(
        cache_key,
        "Project",
        cache_scope,
        BRIEFING_POLICY_VERSION,
        version_vector,
        packet.budget.max_words as u64,
        &packet_json,
        &generated_at,
        None,
    )?;
    Ok(())
}

fn maybe_emit_briefing<W: EventWriter>(
    writer: Option<&W>,
    dry_run: bool,
    packet: &ProjectBriefingPacket,
    privacy: Privacy,
) -> Result<()> {
    if dry_run {
        return Ok(());
    }
    let Some(writer) = writer else {
        return Ok(());
    };
    let briefing_id = BriefingId::from_uuid(
        uuid::Uuid::parse_str(&packet.briefing_id)
            .map_err(|e| ControlPlaneError::InvalidPayload(e.to_string()))?,
    );
    let evidence_ids = packet
        .evidence_handles
        .iter()
        .filter_map(|h| {
            // Only real UUID evidence ids are recorded on the event.
            uuid::Uuid::parse_str(&h.evidence_id)
                .ok()
                .map(ai_brains_core::ids::EvidenceId::from_uuid)
        })
        .collect();
    let event = build_event(
        AggregateType::Briefing,
        briefing_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::BriefingGenerated(BriefingGeneratedPayload {
            briefing_id,
            kind: packet.kind.clone(),
            evidence_ids,
            query_trace_id: None,
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(())
}

fn scope_dto_from_resolved(resolved: &ResolvedScope) -> BriefingScopeDto {
    BriefingScopeDto {
        scope_key: scope_identity_key(&resolved.scope),
        confidence: confidence_label(resolved.confidence).to_string(),
        warnings: resolved.warnings.clone(),
        alternatives: resolved
            .alternatives
            .iter()
            .map(scope_identity_key)
            .collect(),
        authoritative: is_authoritative(resolved),
    }
}

fn confidence_label(c: ScopeConfidence) -> &'static str {
    match c {
        ScopeConfidence::Ambiguous => "Ambiguous",
        ScopeConfidence::Low => "Low",
        ScopeConfidence::Medium => "Medium",
        ScopeConfidence::High => "High",
    }
}

fn empty_freshness() -> FreshnessSummaryDto {
    FreshnessSummaryDto {
        total_sources: 0,
        fresh_count: 0,
        stale_count: 0,
        unavailable_count: 0,
        worst_state: "Unknown".into(),
    }
}

fn evidence_handles_for_conclusion<Q: GovernedQueryStore>(
    query: &Q,
    id: ai_brains_core::ids::ConclusionId,
) -> Result<Vec<EvidenceHandle>> {
    let ids = query.evidence_ids_for_conclusion(id)?;
    Ok(ids
        .into_iter()
        .map(|e| EvidenceHandle {
            evidence_id: e.to_string(),
            cite_label: None,
        })
        .collect())
}

fn evidence_handles_for_decision<Q: GovernedQueryStore>(
    query: &Q,
    id: ai_brains_core::ids::DecisionId,
) -> Result<Vec<EvidenceHandle>> {
    let ids = query.evidence_ids_for_decision(id)?;
    Ok(ids
        .into_iter()
        .map(|e| EvidenceHandle {
            evidence_id: e.to_string(),
            cite_label: None,
        })
        .collect())
}

fn push_unique_handle(out: &mut Vec<EvidenceHandle>, handle: EvidenceHandle) {
    if !out.iter().any(|h| h.evidence_id == handle.evidence_id) {
        out.push(handle);
    }
}

fn extract_constraints(conclusions: &[BriefingClaimDto]) -> Vec<BriefingConstraintDto> {
    conclusions
        .iter()
        .filter(|c| {
            let s = c.statement.to_ascii_uppercase();
            s.contains("CONSTRAINT:") || s.contains("INVARIANT:")
        })
        .map(|c| BriefingConstraintDto {
            id: c.id.clone(),
            statement: c.statement.clone(),
            evidence_handles: c.evidence_handles.clone(),
        })
        .collect()
}

/// Whether a conclusion's valid-time window covers `at` (valid_from ≤ at < valid_until|∞).
fn conclusion_valid_at(row: &crate::ports::ConclusionRow, at: time::OffsetDateTime) -> bool {
    row.valid_from <= at && row.valid_until.map(|u| u > at).unwrap_or(true)
}

/// Whether a decision's valid-time window covers `at`.
///
/// Missing `valid_from` is treated as always-started (legacy rows).
fn decision_valid_at(row: &crate::ports::DecisionRow, at: time::OffsetDateTime) -> bool {
    let from_ok = row.valid_from.map(|vf| vf <= at).unwrap_or(true);
    let until_ok = row.valid_until.map(|u| u > at).unwrap_or(true);
    from_ok && until_ok
}

/// Re-apply valid-time (and still-current lifecycle) on a cache hit without full rebuild.
///
/// Drops decisions/conclusions that are outside the window at `now` or no longer
/// Approved / Active|Confirmed, and rebuilds constraints / evidence handles / freshness.
fn refilter_cached_packet_valid_time<Q: GovernedQueryStore>(
    query: &Q,
    packet: &mut ProjectBriefingPacket,
    now: time::OffsetDateTime,
) -> Result<()> {
    if packet.denied {
        return Ok(());
    }

    let mut kept_decisions = Vec::with_capacity(packet.decisions.len());
    for d in std::mem::take(&mut packet.decisions) {
        let Ok(uuid) = uuid::Uuid::parse_str(&d.id) else {
            packet.warnings.push(BriefingWarningDto {
                kind: "out_of_valid_time".into(),
                message: format!(
                    "Cached decision {} has unparseable id; dropped on cache revalidation",
                    d.id
                ),
                subject_id: Some(d.id.clone()),
                subject_kind: Some("Decision".into()),
            });
            continue;
        };
        let id = ai_brains_core::ids::DecisionId::from_uuid(uuid);
        match query.get_decision(id)? {
            Some(row) if row.state == "Approved" && decision_valid_at(&row, now) => {
                kept_decisions.push(d);
            }
            Some(_) | None => {
                packet.warnings.push(BriefingWarningDto {
                    kind: "out_of_valid_time".into(),
                    message: format!(
                        "Decision {} outside valid-time or no longer current; dropped from cached packet",
                        d.id
                    ),
                    subject_id: Some(d.id.clone()),
                    subject_kind: Some("Decision".into()),
                });
            }
        }
    }
    packet.decisions = kept_decisions;

    let mut kept_conclusions = Vec::with_capacity(packet.conclusions.len());
    for c in std::mem::take(&mut packet.conclusions) {
        let Ok(uuid) = uuid::Uuid::parse_str(&c.id) else {
            packet.warnings.push(BriefingWarningDto {
                kind: "out_of_valid_time".into(),
                message: format!(
                    "Cached conclusion {} has unparseable id; dropped on cache revalidation",
                    c.id
                ),
                subject_id: Some(c.id.clone()),
                subject_kind: Some("Conclusion".into()),
            });
            continue;
        };
        let id = ai_brains_core::ids::ConclusionId::from_uuid(uuid);
        match query.get_conclusion(id)? {
            Some(row)
                if (row.state == "Active" || row.state == "Confirmed")
                    && conclusion_valid_at(&row, now) =>
            {
                kept_conclusions.push(c);
            }
            Some(_) | None => {
                packet.warnings.push(BriefingWarningDto {
                    kind: "out_of_valid_time".into(),
                    message: format!(
                        "Conclusion {} outside valid-time or no longer current; dropped from cached packet",
                        c.id
                    ),
                    subject_id: Some(c.id.clone()),
                    subject_kind: Some("Conclusion".into()),
                });
            }
        }
    }
    packet.conclusions = kept_conclusions;

    packet.constraints = extract_constraints(&packet.conclusions);

    // Rebuild evidence handles from remaining claims only.
    let mut evidence_handles: Vec<EvidenceHandle> = Vec::new();
    for d in &packet.decisions {
        for h in &d.evidence_handles {
            push_unique_handle(&mut evidence_handles, h.clone());
        }
    }
    for c in &packet.conclusions {
        for h in &c.evidence_handles {
            push_unique_handle(&mut evidence_handles, h.clone());
        }
    }
    packet.evidence_handles = evidence_handles;

    let current_count = (packet.decisions.len() + packet.conclusions.len()) as u32;
    let stale_count = packet.warnings.iter().filter(|w| w.kind == "stale").count() as u32;
    packet.freshness = FreshnessSummaryDto {
        total_sources: current_count.saturating_add(stale_count),
        fresh_count: current_count,
        stale_count,
        unavailable_count: 0,
        worst_state: if stale_count > 0 {
            "Stale".into()
        } else if current_count > 0 {
            "Fresh".into()
        } else {
            "Unknown".into()
        },
    };

    Ok(())
}

/// Whether the principal may see a conflict arm of the given claim kind.
fn can_read_conflict_claim_kind(
    kind: &str,
    can_read_decisions: bool,
    can_read_conclusions: bool,
) -> bool {
    match kind {
        k if k.eq_ignore_ascii_case("Decision") => can_read_decisions,
        k if k.eq_ignore_ascii_case("Conclusion") => can_read_conclusions,
        // Unknown kinds: withhold to avoid leaking opaque explanation text.
        _ => false,
    }
}

/// Strictest privacy of request + every included claim/evidence source (T152-FRESH3-P1-01).
///
/// Combines:
/// - current conclusion rows in the packet
/// - stale/disputed/rejected (and other) conclusions whose text appears in warnings
/// - decisions: no privacy column; combine privacy of linked evidence and supporting conclusions
/// - evidence_projection privacy for real UUID handles on the packet
fn derived_briefing_privacy<Q: GovernedQueryStore>(
    request: Privacy,
    query: &Q,
    packet: &ProjectBriefingPacket,
) -> Result<Privacy> {
    let mut privacy = request;

    for c in &packet.conclusions {
        privacy = combine_conclusion_privacy(privacy, query, &c.id)?;
    }

    // Warning subject conclusions (Stale/Disputed/Rejected statement text is included).
    for w in &packet.warnings {
        if w.subject_kind
            .as_deref()
            .is_some_and(|k| k.eq_ignore_ascii_case("Conclusion"))
            && let Some(ref sid) = w.subject_id
        {
            privacy = combine_conclusion_privacy(privacy, query, sid)?;
        }
        if w.subject_kind
            .as_deref()
            .is_some_and(|k| k.eq_ignore_ascii_case("Decision"))
            && let Some(ref sid) = w.subject_id
        {
            privacy = combine_decision_privacy(privacy, query, sid)?;
        }
    }

    for d in &packet.decisions {
        privacy = combine_decision_privacy(privacy, query, &d.id)?;
    }

    for h in &packet.evidence_handles {
        privacy = combine_evidence_handle_privacy(privacy, query, &h.evidence_id)?;
    }

    Ok(privacy)
}

fn combine_conclusion_privacy<Q: GovernedQueryStore>(
    privacy: Privacy,
    query: &Q,
    id_str: &str,
) -> Result<Privacy> {
    let Ok(uuid) = uuid::Uuid::parse_str(id_str) else {
        return Ok(privacy);
    };
    let id = ai_brains_core::ids::ConclusionId::from_uuid(uuid);
    if let Some(row) = query.get_conclusion(id)? {
        return Ok(privacy.combine(parse_stored_privacy(&row.privacy)));
    }
    Ok(privacy)
}

fn combine_decision_privacy<Q: GovernedQueryStore>(
    privacy: Privacy,
    query: &Q,
    id_str: &str,
) -> Result<Privacy> {
    let Ok(uuid) = uuid::Uuid::parse_str(id_str) else {
        return Ok(privacy);
    };
    let id = ai_brains_core::ids::DecisionId::from_uuid(uuid);
    let mut privacy = privacy;
    // Decision rows have no privacy column — inherit from linked evidence + support conclusions.
    for eid in query.evidence_ids_for_decision(id)? {
        if let Some(raw) = query.evidence_privacy(eid)? {
            privacy = privacy.combine(parse_stored_privacy(&raw));
        }
    }
    for cid in query.conclusion_ids_for_decision(id)? {
        if let Some(row) = query.get_conclusion(cid)? {
            privacy = privacy.combine(parse_stored_privacy(&row.privacy));
        }
    }
    Ok(privacy)
}

fn combine_evidence_handle_privacy<Q: GovernedQueryStore>(
    privacy: Privacy,
    query: &Q,
    evidence_id: &str,
) -> Result<Privacy> {
    let Ok(uuid) = uuid::Uuid::parse_str(evidence_id) else {
        // Synthetic handles (decision:, decision-support:) have no evidence row.
        return Ok(privacy);
    };
    let id = ai_brains_core::ids::EvidenceId::from_uuid(uuid);
    if let Some(raw) = query.evidence_privacy(id)? {
        return Ok(privacy.combine(parse_stored_privacy(&raw)));
    }
    Ok(privacy)
}

/// Parse privacy stored on projection rows (JSON-serialized or bare label).
pub(crate) fn parse_stored_privacy(raw: &str) -> Privacy {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Privacy::LocalOnly;
    }
    if let Ok(p) = serde_json::from_str::<Privacy>(trimmed) {
        return p;
    }
    // Bare label fallback (grant-style / unquoted).
    match trimmed {
        "CloudOk" | "Public" => Privacy::CloudOk,
        "LocalOnly" | "ProjectLocal" => Privacy::LocalOnly,
        "NeverInject" | "Private" => Privacy::NeverInject,
        "Sealed" => Privacy::Sealed,
        _ => Privacy::LocalOnly,
    }
}

/// Resolve project id from principal + explicit override without cwd (tests/helpers).
pub fn resolved_repository_high(project_id: ai_brains_core::ids::ProjectId) -> ResolvedScope {
    ResolvedScope {
        scope: ScopeRef::Repository(project_id),
        confidence: ScopeConfidence::High,
        evidence: vec![],
        warnings: Vec::new(),
        alternatives: Vec::new(),
    }
}

/// Helper for tests: principal id access.
pub fn principal_id(p: &Principal) -> PrincipalId {
    p.id
}
