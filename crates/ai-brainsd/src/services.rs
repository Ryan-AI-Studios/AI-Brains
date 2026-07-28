//! Governed query/mutation mapping for the daemon (T159).
//!
//! # CQRS
//!
//! - **Queries** (scope, briefings, progressive query, inspect, list review) run
//!   off the writer mpsc queue via shared vault `Arc` / [`StorePorts`].
//! - **Mutations** (propose_*, resolve review, erasure ticket) are enqueued on
//!   [`crate::DaemonWriter`] and executed only on the single writer task.
//!
//! # Policy
//!
//! Always [`StorePorts::production_policy`] — never `AllowAllPolicy` in production paths.
//!
//! # Principal resolution
//!
//! 1. Wire `principal_id` if parseable as UUID → Human principal with that id
//! 2. Else env `AI_BRAINS_DAEMON_PRINCIPAL_ID` if UUID → Human
//! 3. Else CLI-compatible fixed System principal
//!
//! # Idempotency ownership
//!
//! Daemon derives deterministic domain ids from `command_id` (uuid v5 namespaces
//! below) and passes them as pre-assigned ids. **Control-plane** owns
//! detect-already-done when those ids are `Some`. Spool replay relies on that.
//!
//! # Spool
//!
//! Governed mutations spool **only** when `command_id` is present (filename
//! sanitized from the id). Without `command_id`, process live with no durable spool.
//!
//! # Erasure honesty
//!
//! `accepted`/`queued` is returned only after a durable `ErasureTicketAccepted`
//! event is appended (or found). Response **does not** claim content-envelope wipe
//! (P8 residual).

use ai_brains_contracts::briefings::{
    InspectEvidenceRequest, PersonalBriefingRequest as WirePersonalBriefing,
    PersonalBriefingResponse, ProgressiveQueryResponse,
    ProjectBriefingRequest as WireProjectBriefing, ProjectBriefingResponse, QueryKnowledgeRequest,
};
use ai_brains_contracts::erasure::{ErasureAcceptedResponse, RequestErasureRequest};
use ai_brains_contracts::knowledge::{
    ConclusionProposedResponse, DecisionProposedResponse,
    ProposeConclusionRequest as WireProposeConclusion,
    ProposeDecisionRequest as WireProposeDecision,
};
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::review::{
    ListReviewItemsRequest, ResolveReviewItemRequest, ReviewItemDto, ReviewQueueResponse,
    ReviewResolvedResponse,
};
use ai_brains_contracts::scopes::{ResolveScopeRequest, ScopeEvidenceDto, ScopeResolvedResponse};
use ai_brains_contracts::sources::{InspectSourceRequest, SourceDto};
use ai_brains_control_plane::{
    BudgetConfig, ControlPlaneError, EventWriter, ExpandHandleRequest, GovernedQueryStore,
    PersonalBriefingRequest as CpPersonalBriefing, ProgressiveQueryRequest,
    ProjectBriefingRequest as CpProjectBriefing, ProposeConclusionRequest as CpProposeConclusion,
    ProposeDecisionRequest as CpProposeDecision, ResolvedScope, ScopeConfidence, ScopeResolveInput,
    StoreEventWriter, StorePorts, SystemClock, build_personal_briefing, build_project_briefing,
    expand_handle, is_authoritative, make_principal, parse_scope_key, progressive_query,
    propose_conclusion, propose_decision, resolve_review_item, resolve_scope, scope_identity_key,
};
use ai_brains_core::ids::{
    ConclusionId, DecisionId, EvidenceId, PrincipalId, ProjectId, ReviewItemId, SourceId, UserId,
};
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_daemon_api::DaemonResponse;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::ErasureTicketAcceptedPayload;
use ai_brains_events::{Actor, AggregateType, EventKind, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::event_store::EventStore;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// UUID v5 namespace seeds (DNS-style) for command_id → domain id derivation.
pub const NS_PROPOSE_CONCLUSION: &str = "ai-brains.command.propose_conclusion";
pub const NS_PROPOSE_DECISION: &str = "ai-brains.command.propose_decision";
pub const NS_RESOLVE_REVIEW: &str = "ai-brains.command.resolve_review_item";
pub const NS_REQUEST_ERASURE: &str = "ai-brains.command.request_erasure";

/// Warning text for erasure responses (P8 residual — no CE wipe in T159).
pub const ERASURE_CE_WIPE_WARNING: &str =
    "content-envelope wipe not performed (P8 residual); ticket accepted only";

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Off-queue governed query services sharing the daemon vault.
#[derive(Clone)]
pub struct GovernedServices {
    event_store: Arc<SqliteEventStore>,
}

impl GovernedServices {
    pub fn new(event_store: Arc<SqliteEventStore>) -> Self {
        Self { event_store }
    }

    /// Build StorePorts over a vault clone (same connection Arc).
    pub fn ports(&self) -> StorePorts {
        StorePorts::from_store(SqliteEventStore::new(self.event_store.connection().clone()))
    }

    pub fn event_store(&self) -> &Arc<SqliteEventStore> {
        &self.event_store
    }

    // --- Queries (sync; caller runs on async runtime via spawn_blocking if needed) ---

    pub fn resolve_scope(&self, req: ResolveScopeRequest) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let identity = ports.identity_store();
        let cwd = req
            .cwd
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let explicit_project_id = req
            .explicit_project_id
            .as_deref()
            .and_then(|s| ProjectId::from_str(s).ok());
        let personal_user_id = req
            .personal_user_id
            .as_deref()
            .and_then(|s| UserId::from_str(s).ok());
        let input = ScopeResolveInput {
            cwd,
            explicit_project_id,
            force_personal: req.force_personal,
            personal_user_id,
            git_metadata: None,
        };
        let resolved = resolve_scope(&input, &identity).map_err(map_control_plane_error_to_box)?;
        Ok(DaemonResponse::ScopeResolved(map_resolved_scope(&resolved)))
    }

    pub fn project_briefing(&self, req: WireProjectBriefing) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let clock = SystemClock;
        let policy = ports.production_policy();
        let identity = ports.identity_store();
        let principal = resolve_principal(req.principal_id.as_deref());
        let cwd = req
            .cwd
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let explicit_project_id = req
            .scope
            .as_deref()
            .and_then(parse_repository_project_id)
            .or_else(|| {
                // Also accept bare project uuid in scope field
                req.scope
                    .as_deref()
                    .and_then(|s| ProjectId::from_str(s).ok())
            });
        let packet = build_project_briefing(
            None::<&StoreEventWriter>, // dry_run: no writer
            &ports.query,
            &clock,
            &policy,
            &identity,
            CpProjectBriefing {
                principal,
                resolve: ScopeResolveInput {
                    cwd,
                    explicit_project_id,
                    force_personal: false,
                    personal_user_id: None,
                    git_metadata: None,
                },
                budget: BudgetConfig {
                    max_words: req.max_words.unwrap_or(1500),
                    ..BudgetConfig::default()
                },
                privacy: Privacy::LocalOnly,
                dry_run: true,
                briefing_id: None,
                ledgerful: None,
            },
        )
        .map_err(map_control_plane_error_to_box)?;
        Ok(DaemonResponse::ProjectBriefing(
            ProjectBriefingResponse::new(packet),
        ))
    }

    pub fn personal_briefing(&self, req: WirePersonalBriefing) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let clock = SystemClock;
        let policy = ports.production_policy();
        let principal = resolve_principal(req.principal_id.as_deref());
        let user_id = req
            .scope
            .as_deref()
            .and_then(parse_personal_user_id)
            .unwrap_or_else(|| UserId::from_uuid(principal.id.as_uuid()));
        let grant_store = ports.grant_store();
        let personal_scope_key = scope_identity_key(&ScopeRef::Personal(user_id));
        let packet = build_personal_briefing(
            None::<&StoreEventWriter>,
            &ports.query,
            &clock,
            &policy,
            |p| {
                grant_store.list_applied_grants(
                    p.id,
                    &personal_scope_key,
                    Some(&["ReadConclusions", "ReadDecisions"]),
                )
            },
            CpPersonalBriefing {
                principal,
                user_id,
                budget: BudgetConfig {
                    max_words: req.max_words.unwrap_or(1500),
                    ..BudgetConfig::default()
                },
                privacy: Privacy::LocalOnly,
                dry_run: true,
                briefing_id: None,
            },
        )
        .map_err(map_control_plane_error_to_box)?;
        Ok(DaemonResponse::PersonalBriefing(
            PersonalBriefingResponse::new(packet),
        ))
    }

    pub fn query_knowledge(&self, req: QueryKnowledgeRequest) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let clock = SystemClock;
        let policy = ports.production_policy();
        let principal = resolve_principal(req.principal_id.as_deref());
        let scope = match req.scope.as_deref() {
            Some(s) => parse_scope_key(s).map_err(map_control_plane_error_to_box)?,
            None => {
                return Ok(DaemonResponse::Error(ApiError::new(
                    "INVALID_PAYLOAD",
                    "query_knowledge requires scope",
                )));
            }
        };
        let event_store = ports.store();
        let resp: ProgressiveQueryResponse = progressive_query(
            None::<&StoreEventWriter>, // dry_run for daemon v1
            &ports.query,
            &event_store,
            &clock,
            &policy,
            ProgressiveQueryRequest {
                principal,
                scope,
                query: req.query,
                privacy: Privacy::LocalOnly,
                limit: req.limit.unwrap_or(16),
                dry_run: true,
                at: None,
            },
        )
        .map_err(map_control_plane_error_to_box)?;
        Ok(DaemonResponse::QueryKnowledge(resp))
    }

    pub fn inspect_evidence(
        &self,
        req: InspectEvidenceRequest,
    ) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let policy = ports.production_policy();
        let principal = resolve_principal(req.principal_id.as_deref());
        let scope = match req.scope.as_deref() {
            Some(s) => parse_scope_key(s).map_err(map_control_plane_error_to_box)?,
            None => {
                return Ok(DaemonResponse::Error(ApiError::new(
                    "INVALID_PAYLOAD",
                    "inspect_evidence requires scope",
                )));
            }
        };
        let event_store = ports.store();
        let preview = expand_handle(
            &ports.query,
            &event_store,
            &policy,
            ExpandHandleRequest {
                principal,
                scope,
                handle_id: req.id,
                privacy: Privacy::LocalOnly,
                max_chars: req.max_chars.unwrap_or(512),
            },
        )
        .map_err(map_control_plane_error_to_box)?;
        Ok(DaemonResponse::EvidencePreview(preview))
    }

    pub fn inspect_source(&self, req: InspectSourceRequest) -> Result<DaemonResponse, BoxError> {
        let source_id = match SourceId::from_str(&req.id) {
            Ok(id) => id,
            Err(_) => {
                return Ok(DaemonResponse::Error(ApiError::new(
                    "INVALID_PAYLOAD",
                    format!("invalid source id: {}", req.id),
                )));
            }
        };
        match load_source_dto(self.event_store.as_ref(), source_id)? {
            Some(dto) => Ok(DaemonResponse::Source(dto)),
            None => Ok(DaemonResponse::Error(ApiError::new(
                "NOT_FOUND",
                format!("source {}", req.id),
            ))),
        }
    }

    pub fn list_review_items(
        &self,
        req: ListReviewItemsRequest,
    ) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let mut items = ports
            .query
            .list_open_review_items()
            .map_err(map_control_plane_error_to_box)?;
        if let Some(status_filter) = req.status.as_deref() {
            // Open-list API only returns Open; if filter is something else → empty.
            if !status_filter.eq_ignore_ascii_case("Open") {
                items.clear();
            }
        }
        let dtos: Vec<ReviewItemDto> = items
            .into_iter()
            .map(|r| ReviewItemDto {
                id: r.id.to_string(),
                subject: r.subject,
                status: r.status,
                opened_at: None,
            })
            .collect();
        Ok(DaemonResponse::ReviewList(ReviewQueueResponse::new(dtos)))
    }
}

// ---------------------------------------------------------------------------
// Mutation processing (writer task)
// ---------------------------------------------------------------------------

/// Run a governed mutation against control-plane / ticket append on the writer path.
pub fn process_governed_mutation(
    ports: &StorePorts,
    op: GovernedMutation,
) -> Result<DaemonResponse, BoxError> {
    match op {
        GovernedMutation::ProposeConclusion(req) => process_propose_conclusion(ports, req),
        GovernedMutation::ProposeDecision(req) => process_propose_decision(ports, req),
        GovernedMutation::ResolveReviewItem(req) => process_resolve_review(ports, req),
        GovernedMutation::RequestErasure(req) => process_request_erasure(ports, req),
    }
}

/// Governed mutation kinds handled on the writer queue.
#[derive(Debug, Clone)]
pub enum GovernedMutation {
    ProposeConclusion(WireProposeConclusion),
    ProposeDecision(WireProposeDecision),
    ResolveReviewItem(ResolveReviewItemRequest),
    RequestErasure(RequestErasureRequest),
}

fn process_propose_conclusion(
    ports: &StorePorts,
    req: WireProposeConclusion,
) -> Result<DaemonResponse, BoxError> {
    let principal = resolve_principal(req.principal_id.as_deref());
    let scope = match parse_scope_key(&req.scope) {
        Ok(s) => s,
        Err(e) => return Ok(map_control_plane_error(e)),
    };
    let privacy = parse_privacy(req.privacy.as_deref());
    let evidence_ids = parse_evidence_ids(&req.evidence_ids);
    let conclusion_id = req
        .command_id
        .as_deref()
        .map(|cid| ConclusionId::from_uuid(id_from_command(NS_PROPOSE_CONCLUSION, cid)));

    let policy = ports.production_policy();
    match propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &policy,
        CpProposeConclusion {
            principal,
            scope,
            statement: req.statement,
            evidence_ids,
            privacy,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id,
        },
    ) {
        Ok(res) => {
            let status = if res.unsupported {
                "unsupported"
            } else {
                "proposed"
            };
            Ok(DaemonResponse::ConclusionProposed(
                ConclusionProposedResponse::new(res.conclusion_id.to_string(), status),
            ))
        }
        Err(e) => Ok(map_control_plane_error(e)),
    }
}

fn process_propose_decision(
    ports: &StorePorts,
    req: WireProposeDecision,
) -> Result<DaemonResponse, BoxError> {
    let principal = resolve_principal(req.principal_id.as_deref());
    let scope = match parse_scope_key(&req.scope) {
        Ok(s) => s,
        Err(e) => return Ok(map_control_plane_error(e)),
    };
    let privacy = parse_privacy(req.privacy.as_deref());
    let decision_id = req
        .command_id
        .as_deref()
        .map(|cid| DecisionId::from_uuid(id_from_command(NS_PROPOSE_DECISION, cid)));
    let conclusion_ids = {
        let parsed: Vec<ConclusionId> = req
            .conclusion_ids
            .iter()
            .filter_map(|s| ConclusionId::from_str(s).ok())
            .collect();
        if parsed.is_empty() {
            None
        } else {
            Some(parsed)
        }
    };
    let evidence_ids = {
        let parsed = parse_evidence_ids(&req.evidence_ids);
        if parsed.is_empty() {
            None
        } else {
            Some(parsed)
        }
    };
    let title = req
        .title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| "Decision".to_string());

    let policy = ports.production_policy();
    match propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &policy,
        CpProposeDecision {
            principal,
            scope,
            title,
            statement: req.statement,
            conclusion_ids,
            evidence_ids,
            privacy,
            valid_from: None,
            valid_until: None,
            decision_id,
        },
    ) {
        Ok(res) => Ok(DaemonResponse::DecisionProposed(
            DecisionProposedResponse::new(res.decision_id.to_string(), "proposed"),
        )),
        Err(e) => Ok(map_control_plane_error(e)),
    }
}

fn process_resolve_review(
    ports: &StorePorts,
    req: ResolveReviewItemRequest,
) -> Result<DaemonResponse, BoxError> {
    let principal = resolve_principal(req.principal_id.as_deref());
    let review_item_id = match ReviewItemId::from_str(&req.id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(DaemonResponse::Error(ApiError::new(
                "INVALID_PAYLOAD",
                format!("invalid review item id: {}", req.id),
            )));
        }
    };
    let scope = match req.scope.as_deref() {
        Some(s) => match parse_scope_key(s) {
            Ok(sc) => sc,
            Err(e) => return Ok(map_control_plane_error(e)),
        },
        None => {
            return Ok(DaemonResponse::Error(ApiError::new(
                "INVALID_PAYLOAD",
                "resolve_review_item requires scope",
            )));
        }
    };
    let reason = req
        .note
        .clone()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| req.resolution.clone());
    // command_id namespace is frozen for future review-linked ids; resolution is keyed by review id.
    let _ = req
        .command_id
        .as_deref()
        .map(|cid| id_from_command(NS_RESOLVE_REVIEW, cid));

    let policy = ports.production_policy();
    match resolve_review_item(
        &ports.writer,
        &ports.query,
        &policy,
        &principal,
        review_item_id,
        &reason,
        Privacy::LocalOnly,
        scope,
    ) {
        Ok(()) => {
            let status = ports
                .query
                .get_review_item(review_item_id)
                .ok()
                .flatten()
                .map(|r| r.status)
                .unwrap_or_else(|| "Resolved".to_string());
            Ok(DaemonResponse::ReviewResolved(ReviewResolvedResponse::new(
                review_item_id.to_string(),
                status,
            )))
        }
        Err(e) => Ok(map_control_plane_error(e)),
    }
}

fn process_request_erasure(
    ports: &StorePorts,
    req: RequestErasureRequest,
) -> Result<DaemonResponse, BoxError> {
    let principal = resolve_principal(req.principal_id.as_deref());
    let request_id = match req.command_id.as_deref() {
        Some(cid) if !cid.trim().is_empty() => id_from_command(NS_REQUEST_ERASURE, cid).to_string(),
        _ => Uuid::new_v4().to_string(),
    };

    // Idempotent: scan event log for existing ticket with same request_id.
    if let Some(prior) = find_erasure_ticket(ports.store(), &request_id)? {
        let mut resp = ErasureAcceptedResponse::new(prior.request_id, "accepted");
        resp.warnings.push(ERASURE_CE_WIPE_WARNING.to_string());
        return Ok(DaemonResponse::ErasureAccepted(resp));
    }

    let aggregate_id = Uuid::parse_str(&request_id).unwrap_or_else(|_| {
        // request_id is always a uuid string from our generators above.
        id_from_command(NS_REQUEST_ERASURE, &request_id)
    });

    let event = EventBuilder::new(
        AggregateType::System,
        aggregate_id,
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ErasureTicketAccepted(
        ErasureTicketAcceptedPayload {
            request_id: request_id.clone(),
            requester: principal.id,
            target_ids: req.ids,
            reason: req.reason,
            scope: req.scope,
        },
    ))
    .map_err(|e| -> BoxError { format!("erasure ticket event build: {e}").into() })?;

    ports
        .writer
        .append_events(&[event])
        .map_err(map_control_plane_error_to_box)?;

    let mut resp = ErasureAcceptedResponse::new(request_id, "accepted");
    resp.warnings.push(ERASURE_CE_WIPE_WARNING.to_string());
    Ok(DaemonResponse::ErasureAccepted(resp))
}

fn find_erasure_ticket(
    store: SqliteEventStore,
    request_id: &str,
) -> Result<Option<ErasureTicketAcceptedPayload>, BoxError> {
    let events = store
        .read_all_events()
        .map_err(|e| -> BoxError { e.to_string().into() })?;
    for env in events {
        if env.event_type == EventKind::ErasureTicketAccepted
            && let Payload::ErasureTicketAccepted(p) = &env.payload
            && p.request_id == request_id
        {
            return Ok(Some(p.clone()));
        }
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Mapping helpers (public for unit tests)
// ---------------------------------------------------------------------------

/// Map control-plane [`ResolvedScope`] → full wire [`ScopeResolvedResponse`].
pub fn map_resolved_scope(resolved: &ResolvedScope) -> ScopeResolvedResponse {
    let confidence = confidence_name(resolved.confidence);
    let authoritative = is_authoritative(resolved);
    // Low/Ambiguous must not claim authority even if is_authoritative edge cases.
    let authoritative = authoritative
        && !matches!(
            resolved.confidence,
            ScopeConfidence::Low | ScopeConfidence::Ambiguous
        );
    ScopeResolvedResponse {
        api_version: ai_brains_contracts::scopes::API_VERSION.to_string(),
        scope: scope_identity_key(&resolved.scope),
        confidence: confidence.to_string(),
        authoritative,
        evidence: resolved
            .evidence
            .iter()
            .map(|e| ScopeEvidenceDto {
                signal: e.signal.clone(),
                detail: e.detail.clone(),
            })
            .collect(),
        warnings: resolved.warnings.clone(),
        alternatives: resolved
            .alternatives
            .iter()
            .map(scope_identity_key)
            .collect(),
    }
}

/// Map [`ControlPlaneError`] → structured [`DaemonResponse::Error`].
pub fn map_control_plane_error(err: ControlPlaneError) -> DaemonResponse {
    let (code, message) = control_plane_error_parts(&err);
    DaemonResponse::Error(ApiError::new(code, message))
}

fn control_plane_error_parts(err: &ControlPlaneError) -> (&'static str, String) {
    match err {
        ControlPlaneError::PolicyDenied(m) => ("POLICY_DENIED", m.clone()),
        ControlPlaneError::NotFound(m) => ("NOT_FOUND", m.clone()),
        ControlPlaneError::InvalidPayload(m) => ("INVALID_PAYLOAD", m.clone()),
        ControlPlaneError::ApprovalRequired(m) => ("APPROVAL_REQUIRED", m.clone()),
        ControlPlaneError::InvalidTransition(m) => ("INVALID_TRANSITION", m.clone()),
        other => ("INTERNAL", other.to_string()),
    }
}

fn map_control_plane_error_to_box(err: ControlPlaneError) -> BoxError {
    // Prefer structured responses at call sites; this path is for unexpected query failures.
    format!("{err}").into()
}

/// Resolve principal for daemon IPC (see module docs).
pub fn resolve_principal(wire_principal_id: Option<&str>) -> Principal {
    if let Some(raw) = wire_principal_id {
        let trimmed = raw.trim();
        if let Ok(u) = Uuid::parse_str(trimmed) {
            return make_principal(
                PrincipalKind::Human,
                PrincipalId::from_uuid(u),
                "daemon-human",
            );
        }
    }
    if let Ok(raw) = std::env::var("AI_BRAINS_DAEMON_PRINCIPAL_ID") {
        let trimmed = raw.trim();
        if let Ok(u) = Uuid::parse_str(trimmed) {
            return make_principal(
                PrincipalKind::Human,
                PrincipalId::from_uuid(u),
                "daemon-env-human",
            );
        }
    }
    // CLI-compatible System principal (briefing.rs cli_principal).
    make_principal(
        PrincipalKind::System,
        PrincipalId::from_uuid(Uuid::from_u128(
            0xA1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2,
        )),
        "daemon-system",
    )
}

/// Derive a deterministic UUID from a frozen DNS-style namespace + command_id.
pub fn id_from_command(namespace_name: &str, command_id: &str) -> Uuid {
    let ns = Uuid::new_v5(&Uuid::NAMESPACE_DNS, namespace_name.as_bytes());
    Uuid::new_v5(&ns, command_id.as_bytes())
}

/// Sanitize command_id for use as a spool filename stem.
pub fn sanitize_command_id_for_filename(command_id: &str) -> String {
    let mut out: String = command_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if out.is_empty() {
        out = "command".to_string();
    }
    // Cap length to avoid path issues.
    if out.len() > 120 {
        out.truncate(120);
    }
    out
}

fn confidence_name(c: ScopeConfidence) -> &'static str {
    match c {
        ScopeConfidence::High => "High",
        ScopeConfidence::Medium => "Medium",
        ScopeConfidence::Low => "Low",
        ScopeConfidence::Ambiguous => "Ambiguous",
    }
}

fn parse_privacy(raw: Option<&str>) -> Privacy {
    match raw.map(|s| s.trim()) {
        Some("CloudOk") | Some("cloud_ok") => Privacy::CloudOk,
        Some("NeverInject") | Some("never_inject") => Privacy::NeverInject,
        Some("Sealed") | Some("sealed") => Privacy::Sealed,
        _ => Privacy::LocalOnly,
    }
}

fn parse_evidence_ids(ids: &[String]) -> Vec<EvidenceId> {
    ids.iter()
        .filter_map(|s| EvidenceId::from_str(s).ok())
        .collect()
}

fn parse_repository_project_id(scope: &str) -> Option<ProjectId> {
    if let Some(rest) = scope.strip_prefix("Repository:") {
        return ProjectId::from_str(rest).ok();
    }
    None
}

fn parse_personal_user_id(scope: &str) -> Option<UserId> {
    if let Some(rest) = scope.strip_prefix("Personal:") {
        return UserId::from_str(rest).ok();
    }
    UserId::from_str(scope).ok()
}

fn load_source_dto(
    store: &SqliteEventStore,
    source_id: SourceId,
) -> Result<Option<SourceDto>, BoxError> {
    let conn = store
        .connection()
        .lock()
        .map_err(|e| -> BoxError { e.to_string().into() })?;
    let mut stmt = conn
        .prepare(
            "SELECT source_id, kind, display_name, locator, last_observed_at
             FROM source_projection WHERE source_id = ?",
        )
        .map_err(|e| -> BoxError { e.to_string().into() })?;
    let mut rows = stmt
        .query(rusqlite::params![source_id.to_string()])
        .map_err(|e| -> BoxError { e.to_string().into() })?;
    if let Some(row) = rows
        .next()
        .map_err(|e| -> BoxError { e.to_string().into() })?
    {
        let id: String = row
            .get(0)
            .map_err(|e| -> BoxError { e.to_string().into() })?;
        let kind: String = row
            .get(1)
            .map_err(|e| -> BoxError { e.to_string().into() })?;
        let display_name: String = row
            .get(2)
            .map_err(|e| -> BoxError { e.to_string().into() })?;
        let locator: Option<String> = row
            .get(3)
            .map_err(|e| -> BoxError { e.to_string().into() })?;
        let last_observed: Option<String> = row
            .get(4)
            .map_err(|e| -> BoxError { e.to_string().into() })?;
        let last_observed_at = last_observed.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
        Ok(Some(SourceDto {
            id,
            kind,
            display_name,
            locator,
            last_observed_at,
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_control_plane::ResolutionEvidence;

    #[test]
    fn map_resolved_scope__low_confidence__authoritative_false_with_warnings() {
        let resolved = ResolvedScope {
            scope: ScopeRef::Repository(ProjectId::from_uuid(Uuid::nil())),
            confidence: ScopeConfidence::Low,
            evidence: vec![ResolutionEvidence {
                signal: "cwd".into(),
                detail: "heuristic".into(),
            }],
            warnings: vec!["cwd-only resolution is not authoritative".into()],
            alternatives: Vec::new(),
        };
        let wire = map_resolved_scope(&resolved);
        assert!(!wire.authoritative);
        assert_eq!(wire.confidence, "Low");
        assert_eq!(wire.warnings.len(), 1);
        assert_eq!(wire.evidence.len(), 1);
    }

    #[test]
    fn map_control_plane_error__policy_denied__code() {
        let resp = map_control_plane_error(ControlPlaneError::PolicyDenied(
            "ProposeConclusion denied".into(),
        ));
        match resp {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "POLICY_DENIED");
                assert!(err.message.contains("ProposeConclusion"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn id_from_command__stable_for_same_command_id() {
        let a = id_from_command(NS_PROPOSE_CONCLUSION, "cmd-1");
        let b = id_from_command(NS_PROPOSE_CONCLUSION, "cmd-1");
        let c = id_from_command(NS_PROPOSE_CONCLUSION, "cmd-2");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn sanitize_command_id_for_filename__strips_unsafe() {
        let s = sanitize_command_id_for_filename("a/b:c*d");
        assert_eq!(s, "a_b_c_d");
    }
}
