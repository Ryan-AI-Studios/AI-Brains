//! Governed progressive query + retrieval traces (T152 Phase E).
//!
//! Ranking order: policy → lifecycle/freshness/valid-time → authority → relevance (optional).
//! Stale conclusions are never ranked as current truth.

use ai_brains_contracts::briefings::{
    FreshnessSummaryDto, HandlePreviewDto, ProgressiveQueryHitDto, ProgressiveQueryResponse,
    QueryTraceDto, RankingComponentsDto,
};
use ai_brains_contracts::knowledge::EvidenceHandle;
use ai_brains_contracts::offset_to_utc;
use ai_brains_core::ids::{EvidenceId, PrincipalId, QueryTraceId};
use ai_brains_core::principal::Principal;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_events::payload::QueryTraceRecordedPayload;
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::SqliteEventStore;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::errors::{ControlPlaneError, Result};
use crate::ports::{Clock, EventWriter, GovernedQueryStore, PolicyContext, PolicyEvaluator};
use crate::sources::{build_event, parse_scope_key, scope_identity_key};

/// Progressive query request.
#[derive(Debug, Clone)]
pub struct ProgressiveQueryRequest {
    pub principal: Principal,
    pub scope: ScopeRef,
    pub query: String,
    pub privacy: Privacy,
    /// Max compact results.
    pub limit: usize,
    /// When true, skip QueryTraceRecorded event.
    pub dry_run: bool,
    /// Optional valid-time anchor (default: now).
    pub at: Option<OffsetDateTime>,
}

/// Expand an evidence handle to a bounded preview.
#[derive(Debug, Clone)]
pub struct ExpandHandleRequest {
    pub principal: Principal,
    pub scope: ScopeRef,
    pub handle_id: String,
    pub privacy: Privacy,
    /// Max characters in preview body.
    pub max_chars: usize,
}

/// Run a governed progressive query.
///
/// CQRS: non-`dry_run` paths append `QueryTraceRecorded` only; the projection is
/// updated by the event apply pipeline (no direct SQL dual-write). `dry_run`
/// writes nothing.
pub fn progressive_query<W, Q, C, P>(
    writer: Option<&W>,
    query_store: &Q,
    event_store: &SqliteEventStore,
    clock: &C,
    policy: &P,
    req: ProgressiveQueryRequest,
) -> Result<ProgressiveQueryResponse>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
{
    let now = clock.now()?;
    let at = req.at.unwrap_or(now);
    let scope_key = scope_identity_key(&req.scope);
    let policy_ctx = PolicyContext::default_for_privacy(req.privacy);
    let limit = if req.limit == 0 { 16 } else { req.limit };

    let can_read = policy.allow(
        req.principal.id,
        GrantCapability::ReadConclusions,
        &req.scope,
        &policy_ctx,
    )? || policy.allow(
        req.principal.id,
        GrantCapability::ReadDecisions,
        &req.scope,
        &policy_ctx,
    )?;

    let trace_id = QueryTraceId::new();

    if !can_read {
        // Dual-site SOOT with CLI `POLICY_DENIED_HINT` (governed_common) — keep wording in sync (T221 F17).
        const POLICY_DENIED_HINT: &str = "ensure a grant for this capability exists; run `ai-brains policy bootstrap --scope …` (or check with `ai-brains policy show --scope …`)";
        let resp = ProgressiveQueryResponse {
            api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
            results: Vec::new(),
            applied_scope: scope_key.clone(),
            applied_policy: "DefaultPolicyEvaluator".into(),
            query_trace_id: trace_id.to_string(),
            more_available: false,
            freshness_summary: None,
            conflict_summary: None,
            denied: true,
            denial_reason: Some("ReadConclusions/ReadDecisions denied".into()),
            denial_hint: Some(POLICY_DENIED_HINT.to_string()),
            next_step: None,
        };
        persist_trace(
            writer,
            req.dry_run,
            &trace_id,
            &scope_key,
            req.principal.id,
            &req.query,
            "denied",
            &[],
            None,
            None,
            None,
            req.privacy,
        )?;
        return Ok(resp);
    }

    let q_lower = req.query.to_ascii_lowercase();
    let mut hits: Vec<ProgressiveQueryHitDto> = Vec::new();

    // Decisions (Approved) — highest authority.
    // T152-P1-02: exclude claims outside the requested valid-time window entirely
    // (not only de-score them).
    // T152-FRESH3-P1-01: collect decision-linked evidence privacy for QueryTrace envelope.
    let mut hit_privacies: Vec<Privacy> = Vec::new();
    if policy.allow(
        req.principal.id,
        GrantCapability::ReadDecisions,
        &req.scope,
        &policy_ctx,
    )? {
        for row in query_store.list_decisions(Some(&scope_key), Some("Approved"))? {
            if !matches_query(&row.statement, &row.title, &q_lower) {
                continue;
            }
            if !decision_in_valid_window(row.valid_from, row.valid_until, at) {
                continue;
            }
            let eids = query_store.evidence_ids_for_decision(row.id)?;
            for eid in &eids {
                if let Some(raw) = query_store.evidence_privacy(*eid)? {
                    hit_privacies.push(crate::briefings::project::parse_stored_privacy(&raw));
                }
            }
            for cid in query_store.conclusion_ids_for_decision(row.id)? {
                if let Some(crow) = query_store.get_conclusion(cid)? {
                    hit_privacies.push(crate::briefings::project::parse_stored_privacy(
                        &crow.privacy,
                    ));
                }
            }
            let handles = eids
                .iter()
                .map(|e| EvidenceHandle {
                    evidence_id: e.to_string(),
                    cite_label: None,
                })
                .collect::<Vec<_>>();
            let source_versions = source_versions_for_evidence(event_store, &eids)?;
            hits.push(ProgressiveQueryHitDto {
                id: row.id.to_string(),
                kind: "Decision".into(),
                statement: row.statement,
                state: row.state,
                evidence_handles: handles,
                source_versions,
                freshness: "Fresh".into(),
                conflict_status: None,
                ranking: RankingComponentsDto {
                    authority: 100,
                    valid_time: 50,
                    relevance: Some(relevance_score(&row.title, &q_lower)),
                },
            });
        }
    }

    // Conclusions: Active/Confirmed only as current truth (never Stale as authority).
    // Stale/Disputed counts still feed freshness_summary below.
    // T152-P1-02: future/expired conclusions are excluded from results.
    // T152-FRESH-P2: Active with zero evidence handles are not authoritative hits;
    // freshness uses stale-fact when present (defensive) rather than hard-label Fresh.
    let mut stale_in_scope = 0u32;
    let mut disputed_in_scope = 0u32;
    if policy.allow(
        req.principal.id,
        GrantCapability::ReadConclusions,
        &req.scope,
        &policy_ctx,
    )? {
        for state in ["Confirmed", "Active"] {
            let authority = if state == "Confirmed" { 90 } else { 80 };
            for row in query_store.list_conclusions_by_scope_state(Some(&scope_key), Some(state))? {
                if !matches_query(&row.statement, "", &q_lower) {
                    continue;
                }
                if !conclusion_in_valid_window(row.valid_from, row.valid_until, at) {
                    continue;
                }
                let eids = query_store.evidence_ids_for_conclusion(row.id)?;
                // Skip unsupported / zero-handle conclusions for authoritative ranking.
                if eids.is_empty() {
                    continue;
                }
                let handles = eids
                    .iter()
                    .map(|e| EvidenceHandle {
                        evidence_id: e.to_string(),
                        cite_label: None,
                    })
                    .collect::<Vec<_>>();
                let source_versions = source_versions_for_evidence(event_store, &eids)?;
                let rel = relevance_score(&row.statement, &q_lower);
                let freshness = if query_store.is_conclusion_stale(row.id)? {
                    "Stale".into()
                } else {
                    "Fresh".into()
                };
                hit_privacies.push(crate::briefings::project::parse_stored_privacy(
                    &row.privacy,
                ));
                // Include evidence projection privacy for handles on this hit.
                for eid in &eids {
                    if let Some(raw) = query_store.evidence_privacy(*eid)? {
                        hit_privacies.push(crate::briefings::project::parse_stored_privacy(&raw));
                    }
                }
                hits.push(ProgressiveQueryHitDto {
                    id: row.id.to_string(),
                    kind: "Conclusion".into(),
                    statement: row.statement,
                    state: row.state,
                    evidence_handles: handles,
                    source_versions,
                    freshness,
                    conflict_status: None,
                    ranking: RankingComponentsDto {
                        authority,
                        valid_time: 50,
                        relevance: Some(rel),
                    },
                });
            }
        }
        // Scope-level stale/disputed counts (excluded from hits, reflected in summary).
        stale_in_scope = query_store
            .list_conclusions_by_scope_state(Some(&scope_key), Some("Stale"))?
            .len() as u32;
        disputed_in_scope = query_store
            .list_conclusions_by_scope_state(Some(&scope_key), Some("Disputed"))?
            .len() as u32;
    }

    // Open conflicts summary.
    let open = query_store.list_open_claim_conflicts()?;
    let scope_conflicts: Vec<_> = open.into_iter().filter(|c| c.scope == scope_key).collect();
    let conflict_summary = if scope_conflicts.is_empty() {
        None
    } else {
        Some(format!("{} open claim conflict(s)", scope_conflicts.len()))
    };

    // Sort: authority desc, valid_time desc, relevance desc, id asc.
    hits.sort_by(|a, b| {
        b.ranking
            .authority
            .cmp(&a.ranking.authority)
            .then(b.ranking.valid_time.cmp(&a.ranking.valid_time))
            .then(
                b.ranking
                    .relevance
                    .partial_cmp(&a.ranking.relevance)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
            .then_with(|| a.id.cmp(&b.id))
    });

    let more_available = hits.len() > limit;
    hits.truncate(limit);

    let hit_values: Vec<serde_json::Value> = hits
        .iter()
        .map(|h| {
            let mut m = serde_json::Map::new();
            m.insert("id".into(), serde_json::Value::String(h.id.clone()));
            m.insert(
                "authority".into(),
                serde_json::Value::Number(h.ranking.authority.into()),
            );
            m.insert(
                "valid_time".into(),
                serde_json::Value::Number(h.ranking.valid_time.into()),
            );
            if let Some(rel) = h.ranking.relevance
                && let Some(n) = serde_json::Number::from_f64(rel)
            {
                m.insert("relevance".into(), serde_json::Value::Number(n));
            }
            serde_json::Value::Object(m)
        })
        .collect();
    let mut ranking_map = serde_json::Map::new();
    ranking_map.insert(
        "order".into(),
        serde_json::Value::Array(
            [
                "policy",
                "lifecycle",
                "valid_time",
                "authority",
                "relevance",
            ]
            .into_iter()
            .map(|s| serde_json::Value::String(s.into()))
            .collect(),
        ),
    );
    ranking_map.insert("hits".into(), serde_json::Value::Array(hit_values));
    let ranking_json = serde_json::Value::Object(ranking_map);

    let result_handles: Vec<EvidenceHandle> = hits
        .iter()
        .flat_map(|h| h.evidence_handles.clone())
        .collect();

    let fresh_count = hits.len() as u32;
    let total_sources = fresh_count
        .saturating_add(stale_in_scope)
        .saturating_add(disputed_in_scope);
    let worst_state = if stale_in_scope > 0 {
        "Stale".into()
    } else if disputed_in_scope > 0 {
        "Disputed".into()
    } else if fresh_count > 0 {
        "Fresh".into()
    } else {
        "Unknown".into()
    };
    let freshness = FreshnessSummaryDto {
        total_sources,
        fresh_count,
        stale_count: stale_in_scope,
        unavailable_count: 0,
        worst_state,
    };
    let freshness_summary_str = Some(format!(
        "total={};fresh={};stale={};disputed={}",
        freshness.total_sources, freshness.fresh_count, freshness.stale_count, disputed_in_scope
    ));

    // T152-FRESH-P1-04: QueryTraceRecorded inherits strictest included claim privacy.
    let mut emit_privacy = req.privacy;
    for p in hit_privacies {
        emit_privacy = emit_privacy.combine(p);
    }

    persist_trace(
        writer,
        req.dry_run,
        &trace_id,
        &scope_key,
        req.principal.id,
        &req.query,
        "DefaultPolicyEvaluator",
        &result_handles,
        Some(&ranking_json.to_string()),
        freshness_summary_str.as_deref(),
        conflict_summary.as_deref(),
        emit_privacy,
    )?;

    Ok(ProgressiveQueryResponse {
        api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
        results: hits,
        applied_scope: scope_key,
        applied_policy: "DefaultPolicyEvaluator".into(),
        query_trace_id: trace_id.to_string(),
        more_available,
        freshness_summary: Some(freshness),
        conflict_summary,
        denied: false,
        denial_reason: None,
        denial_hint: None,
        next_step: None,
    })
}

/// Inputs for governed query-trace retrieval (T152-P1-05).
#[derive(Debug, Clone)]
pub struct GetQueryTraceRequest {
    pub principal: Principal,
    pub privacy: Privacy,
    pub trace_id: String,
}

/// Fetch a full query trace by id from `query_trace_projection`.
///
/// Authorization (T152-P1-05):
/// - Principal on the request must match the principal recorded on the trace.
/// - Caller must hold at least one of ReadEvidence / ReadConclusions / ReadDecisions
///   on the trace's stored scope.
///
/// Cross-principal or capability denial returns `Ok(None)` (no existence leak via error).
pub fn get_query_trace<P: PolicyEvaluator>(
    store: &SqliteEventStore,
    policy: &P,
    req: GetQueryTraceRequest,
) -> Result<Option<QueryTraceDto>> {
    let conn = store
        .connection()
        .lock()
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
    let row = match conn.query_row(
        "SELECT trace_id, scope, principal, query, applied_policy, ranking_json,
                result_handles_json, freshness_summary, conflict_summary, recorded_at
         FROM query_trace_projection WHERE trace_id = ?",
        [&req.trace_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
            ))
        },
    ) {
        Ok(r) => r,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(ControlPlaneError::Query(e.to_string())),
    };
    // Drop the connection lock before policy evaluation / further work.
    drop(conn);

    let (
        query_trace_id,
        scope,
        principal,
        query,
        applied_policy,
        ranking_s,
        handles_s,
        freshness_summary,
        conflict_summary,
        recorded_s,
    ) = row;

    // Principal match (strict): cross-principal denial.
    if principal != req.principal.id.to_string() {
        return Ok(None);
    }

    // Scope + capability: require a read grant on the trace's scope.
    let scope_ref = parse_scope_key(&scope)?;
    let policy_ctx = PolicyContext::default_for_privacy(req.privacy);
    let can_read = policy.allow(
        req.principal.id,
        GrantCapability::ReadEvidence,
        &scope_ref,
        &policy_ctx,
    )? || policy.allow(
        req.principal.id,
        GrantCapability::ReadConclusions,
        &scope_ref,
        &policy_ctx,
    )? || policy.allow(
        req.principal.id,
        GrantCapability::ReadDecisions,
        &scope_ref,
        &policy_ctx,
    )?;
    if !can_read {
        return Ok(None);
    }

    let ranking_json: serde_json::Value = serde_json::from_str(&ranking_s).map_err(|e| {
        ControlPlaneError::Query(format!(
            "corrupt ranking_json on query_trace {query_trace_id}: {e}"
        ))
    })?;
    let handle_ids: Vec<String> = serde_json::from_str(&handles_s).map_err(|e| {
        ControlPlaneError::Query(format!(
            "corrupt result_handles_json on query_trace {query_trace_id}: {e}"
        ))
    })?;
    let result_handles = handle_ids
        .into_iter()
        .map(|evidence_id| EvidenceHandle {
            evidence_id,
            cite_label: None,
        })
        .collect();
    // T152-P2-03: corrupt recorded_at must error (not silent None).
    let recorded_at =
        OffsetDateTime::parse(&recorded_s, &time::format_description::well_known::Rfc3339)
            .map(offset_to_utc)
            .map_err(|e| {
                ControlPlaneError::Query(format!(
                    "corrupt recorded_at on query_trace {query_trace_id}: {e}"
                ))
            })?;

    Ok(Some(QueryTraceDto {
        api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
        query_trace_id,
        scope,
        principal,
        query,
        applied_policy,
        ranking_json,
        result_handles,
        freshness_summary,
        conflict_summary,
        recorded_at: Some(recorded_at),
    }))
}

/// Expand a handle to a bounded preview (no full raw dump by default).
///
/// T152-P1-03:
/// 1. Resolve the handle's owning scope first.
/// 2. Deny cross-scope expansion (requested scope must match owning scope).
/// 3. Enforce kind-specific capability (ReadEvidence / ReadConclusions / ReadDecisions).
pub fn expand_handle<Q, P>(
    query_store: &Q,
    event_store: &SqliteEventStore,
    policy: &P,
    req: ExpandHandleRequest,
) -> Result<HandlePreviewDto>
where
    Q: GovernedQueryStore,
    P: PolicyEvaluator,
{
    let max_chars = if req.max_chars == 0 {
        512
    } else {
        req.max_chars
    };
    let denied = || HandlePreviewDto {
        api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
        handle_id: req.handle_id.clone(),
        kind: "Denied".into(),
        preview: String::new(),
        truncated: false,
        source_version_id: None,
    };
    let req_scope_key = scope_identity_key(&req.scope);
    let policy_ctx = PolicyContext::default_for_privacy(req.privacy);

    // --- Resolve handle kind + owning scope before any preview content ---

    // 1) Evidence: join source_projection for owning scope.
    {
        let conn = event_store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        if let Ok((summary, status, source_version_id, source_scope)) = conn.query_row(
            "SELECT e.summary, e.status, e.source_version_id, s.scope
             FROM evidence_projection e
             JOIN source_projection s ON s.source_id = e.source_id
             WHERE e.evidence_id = ?",
            [&req.handle_id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        ) {
            drop(conn);
            if source_scope != req_scope_key {
                return Ok(denied());
            }
            if !policy.allow(
                req.principal.id,
                GrantCapability::ReadEvidence,
                &req.scope,
                &policy_ctx,
            )? {
                return Ok(denied());
            }
            let (preview, truncated) = truncate_chars(&summary, max_chars);
            return Ok(HandlePreviewDto {
                api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
                handle_id: req.handle_id,
                kind: format!("Evidence:{status}"),
                preview,
                truncated,
                source_version_id,
            });
        }
    }

    // 2) Conclusion / Decision by UUID handle.
    if let Ok(uuid) = Uuid::parse_str(&req.handle_id) {
        let cid = ai_brains_core::ids::ConclusionId::from_uuid(uuid);
        if let Some(row) = query_store.get_conclusion(cid)? {
            if row.scope != req_scope_key {
                return Ok(denied());
            }
            if !policy.allow(
                req.principal.id,
                GrantCapability::ReadConclusions,
                &req.scope,
                &policy_ctx,
            )? {
                return Ok(denied());
            }
            let (preview, truncated) = truncate_chars(&row.statement, max_chars);
            return Ok(HandlePreviewDto {
                api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
                handle_id: req.handle_id,
                kind: format!("Conclusion:{}", row.state),
                preview,
                truncated,
                source_version_id: None,
            });
        }
        let did = ai_brains_core::ids::DecisionId::from_uuid(uuid);
        if let Some(row) = query_store.get_decision(did)? {
            if row.scope != req_scope_key {
                return Ok(denied());
            }
            if !policy.allow(
                req.principal.id,
                GrantCapability::ReadDecisions,
                &req.scope,
                &policy_ctx,
            )? {
                return Ok(denied());
            }
            let body = format!("{}: {}", row.title, row.statement);
            let (preview, truncated) = truncate_chars(&body, max_chars);
            return Ok(HandlePreviewDto {
                api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
                handle_id: req.handle_id,
                kind: format!("Decision:{}", row.state),
                preview,
                truncated,
                source_version_id: None,
            });
        }
    }

    Ok(HandlePreviewDto {
        api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
        handle_id: req.handle_id,
        kind: "Unknown".into(),
        preview: String::new(),
        truncated: false,
        source_version_id: None,
    })
}

fn conclusion_in_valid_window(
    valid_from: OffsetDateTime,
    valid_until: Option<OffsetDateTime>,
    at: OffsetDateTime,
) -> bool {
    valid_from <= at && valid_until.map(|u| u > at).unwrap_or(true)
}

fn decision_in_valid_window(
    valid_from: Option<OffsetDateTime>,
    valid_until: Option<OffsetDateTime>,
    at: OffsetDateTime,
) -> bool {
    let from_ok = valid_from.map(|vf| vf <= at).unwrap_or(true);
    let until_ok = valid_until.map(|u| u > at).unwrap_or(true);
    from_ok && until_ok
}

fn matches_query(statement: &str, title: &str, q_lower: &str) -> bool {
    if q_lower.trim().is_empty() {
        return true;
    }
    let hay = format!("{statement} {title}").to_ascii_lowercase();
    q_lower
        .split_whitespace()
        .any(|tok| !tok.is_empty() && hay.contains(tok))
}

fn relevance_score(text: &str, q_lower: &str) -> f64 {
    if q_lower.trim().is_empty() {
        return 0.0;
    }
    let hay = text.to_ascii_lowercase();
    let mut hits = 0u32;
    let mut total = 0u32;
    for tok in q_lower.split_whitespace() {
        if tok.is_empty() {
            continue;
        }
        total += 1;
        if hay.contains(tok) {
            hits += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        f64::from(hits) / f64::from(total)
    }
}

fn truncate_chars(s: &str, max: usize) -> (String, bool) {
    if s.chars().count() <= max {
        return (s.to_string(), false);
    }
    let preview: String = s.chars().take(max).collect();
    (preview, true)
}

/// Collect distinct source_version ids linked from `evidence_projection` for the given evidence ids.
fn source_versions_for_evidence(
    store: &SqliteEventStore,
    evidence_ids: &[EvidenceId],
) -> Result<Vec<String>> {
    if evidence_ids.is_empty() {
        return Ok(Vec::new());
    }
    let conn = store
        .connection()
        .lock()
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for eid in evidence_ids {
        let version: Option<String> = match conn.query_row(
            "SELECT source_version_id FROM evidence_projection WHERE evidence_id = ?",
            [eid.to_string()],
            |r| r.get::<_, Option<String>>(0),
        ) {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(ControlPlaneError::Query(e.to_string())),
        };
        if let Some(v) = version
            && !v.is_empty()
            && seen.insert(v.clone())
        {
            out.push(v);
        }
    }
    Ok(out)
}

/// Append `QueryTraceRecorded` (projection rehydrates via the event apply path).
///
/// `dry_run == true` writes nothing (no projection, no event).
#[allow(clippy::too_many_arguments)]
fn persist_trace<W: EventWriter>(
    writer: Option<&W>,
    dry_run: bool,
    trace_id: &QueryTraceId,
    scope_key: &str,
    principal: PrincipalId,
    query: &str,
    applied_policy: &str,
    handles: &[EvidenceHandle],
    ranking_json: Option<&str>,
    freshness_summary: Option<&str>,
    conflict_summary: Option<&str>,
    privacy: Privacy,
) -> Result<()> {
    if dry_run {
        return Ok(());
    }
    let Some(writer) = writer else {
        return Ok(());
    };
    let evidence_ids: Vec<EvidenceId> = handles
        .iter()
        .filter_map(|h| {
            Uuid::parse_str(&h.evidence_id)
                .ok()
                .map(EvidenceId::from_uuid)
        })
        .collect();
    let ranking = ranking_json.unwrap_or("{}").to_string();
    let event = build_event(
        AggregateType::QueryTrace,
        trace_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::QueryTraceRecorded(QueryTraceRecordedPayload {
            query_trace_id: *trace_id,
            query_text: query.to_string(),
            evidence_ids,
            scope: scope_key.to_string(),
            principal_id: principal.to_string(),
            applied_policy: applied_policy.to_string(),
            ranking_json: ranking,
            freshness_summary: freshness_summary.map(str::to_string),
            conflict_summary: conflict_summary.map(str::to_string),
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(())
}
