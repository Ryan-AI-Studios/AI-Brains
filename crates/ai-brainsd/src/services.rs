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
//! All governed reads and mutations that touch projections or append events check
//! capabilities (`ReadEvidence`, `ReadConclusions`, `Erase`, propose/resolve via CP).
//!
//! # Principal resolution
//!
//! 1. Wire `principal_id` if parseable as UUID:
//!    - well-known System principal UUID → System (`daemon-system`) for kind parity
//!      with CLI `cli_principal` (T160 Codex P1)
//!    - any other UUID → Human (`daemon-human`)
//! 2. Else env `AI_BRAINS_DAEMON_PRINCIPAL_ID` if UUID → Human (legacy clients that
//!    omit wire `principal_id`)
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
//! Governed mutations spool **only** when `command_id` is present. Filename is
//! `{op}_{sanitized_command_id}.json` (op-scoped to avoid cross-op collision).
//! Without `command_id`, process live with no durable spool.
//! Terminal domain outcomes delete spool; retriable infra (`EventAppend` /
//! `Query` / `Clock`) keep spool for restart replay.
//!
//! # Erasure honesty
//!
//! `accepted`/`queued` is returned only after a durable `ErasureTicketAccepted`
//! event is appended (or found after policy allow). Response **does not** claim
//! content-envelope wipe (P8 residual). Requires `GrantCapability::Erase` on the
//! request scope **before** the idempotent ticket short-circuit.
//!
//! # Governed briefing (T152-R1-07 / T159)
//!
//! Daemon project/personal briefing is **always** the governed control-plane path.
//! `governed_briefing: Some(false)` is rejected with `INVALID_PAYLOAD`; `None` or
//! `Some(true)` proceeds.

use ai_brains_contracts::briefings::{
    EvidenceListItemDto, EvidenceListResponse, InspectEvidenceRequest, ListEvidenceRequest,
    PersonalBriefingRequest as WirePersonalBriefing, PersonalBriefingResponse,
    ProjectBriefingRequest as WireProjectBriefing, ProjectBriefingResponse, QueryKnowledgeRequest,
    truncate_evidence_list_summary,
};
use ai_brains_contracts::erasure::{
    ERASURE_TICKET_NO_WIPE_WARNING, ErasureAcceptedResponse, RequestErasureRequest,
    WipeContentEnvelopeRequest,
};
use ai_brains_contracts::knowledge::{
    ConclusionProposedResponse, DecisionProposedResponse,
    ProposeConclusionRequest as WireProposeConclusion,
    ProposeDecisionRequest as WireProposeDecision,
};
use ai_brains_contracts::offset_to_utc;
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::review::{
    ListReviewItemsRequest, ResolveReviewItemRequest, ReviewItemDto, ReviewQueueResponse,
    ReviewResolvedResponse,
};
use ai_brains_contracts::scopes::{ResolveScopeRequest, ScopeEvidenceDto, ScopeResolvedResponse};
use ai_brains_contracts::sources::{InspectSourceRequest, ListSourcesRequest, SourceListResponse};
use ai_brains_control_plane::{
    BudgetConfig, ControlPlaneError, EventWriter, ExpandHandleRequest, GovernedQueryStore,
    PersonalBriefingRequest as CpPersonalBriefing, PolicyContext, PolicyEvaluator,
    ProgressiveQueryRequest, ProjectBriefingRequest as CpProjectBriefing,
    ProposeConclusionRequest as CpProposeConclusion, ProposeDecisionRequest as CpProposeDecision,
    ResolvedScope, ScopeConfidence, ScopeResolveInput, StoreContentEnvelopeWipe, StoreEventWriter,
    StorePorts, SystemClock, WipeContentEnvelopeCommand, build_personal_briefing,
    build_project_briefing, clamp_list_limit, expand_handle, is_authoritative,
    list_open_review_items_for_scope, make_principal, parse_content_key_id, parse_scope_key,
    progressive_query, propose_conclusion, propose_decision, resolve_review_item, resolve_scope,
    scope_identity_key, source_row_to_dto, tombstone_id_from_command, wipe_content_envelope,
};
use ai_brains_core::ids::{
    ConclusionId, DecisionId, EvidenceId, PrincipalId, ProjectId, ReviewItemId, SourceId, UserId,
};
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
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

// NS_* and id_from_command live in ai-brains-control-plane (T160 shared derivation).
// Re-export so existing daemon call sites / tests keep a stable path.
pub use ai_brains_control_plane::{
    NS_PROPOSE_CONCLUSION, NS_PROPOSE_DECISION, NS_REQUEST_ERASURE, NS_WIPE_CONTENT_ENVELOPE,
    id_from_command,
};

// Review resolve idempotency is review_item_id + status based in control-plane
// (not a command_id-derived domain id). Spool still keys by command_id when set.

/// Warning text for ticket erasure responses (E3 — ticket ≠ CE wipe).
/// Prefer contracts constant; re-export for daemon/tests stability.
pub const ERASURE_CE_WIPE_WARNING: &str = ERASURE_TICKET_NO_WIPE_WARNING;

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
        match resolve_scope(&input, &identity) {
            Ok(resolved) => Ok(DaemonResponse::ScopeResolved(map_resolved_scope(&resolved))),
            Err(e) => Ok(map_control_plane_error(e)),
        }
    }

    pub fn project_briefing(&self, req: WireProjectBriefing) -> Result<DaemonResponse, BoxError> {
        // Daemon briefing is always governed (T152-R1-07 / T159).
        // Honor `governed_briefing`: None | Some(true) proceed; Some(false) rejected.
        if req.governed_briefing == Some(false) {
            return Ok(DaemonResponse::Error(ApiError::new(
                "INVALID_PAYLOAD",
                "daemon briefing is always governed; omit governed_briefing or set true",
            )));
        }
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
        match build_project_briefing(
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
        ) {
            Ok(packet) => Ok(DaemonResponse::ProjectBriefing(
                ProjectBriefingResponse::new(packet),
            )),
            Err(e) => Ok(map_control_plane_error(e)),
        }
    }

    pub fn personal_briefing(&self, req: WirePersonalBriefing) -> Result<DaemonResponse, BoxError> {
        // Daemon briefing is always governed (T152-R1-07 / T159).
        // Honor `governed_briefing`: None | Some(true) proceed; Some(false) rejected.
        if req.governed_briefing == Some(false) {
            return Ok(DaemonResponse::Error(ApiError::new(
                "INVALID_PAYLOAD",
                "daemon briefing is always governed; omit governed_briefing or set true",
            )));
        }
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
        match build_personal_briefing(
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
        ) {
            Ok(packet) => Ok(DaemonResponse::PersonalBriefing(
                PersonalBriefingResponse::new(packet),
            )),
            Err(e) => Ok(map_control_plane_error(e)),
        }
    }

    pub fn query_knowledge(&self, req: QueryKnowledgeRequest) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let clock = SystemClock;
        let policy = ports.production_policy();
        let principal = resolve_principal(req.principal_id.as_deref());
        let scope = match req.scope.as_deref() {
            Some(s) => match parse_scope_key(s) {
                Ok(sc) => sc,
                Err(e) => return Ok(map_control_plane_error(e)),
            },
            None => {
                return Ok(DaemonResponse::Error(ApiError::new(
                    "INVALID_PAYLOAD",
                    "query_knowledge requires scope",
                )));
            }
        };
        let event_store = ports.store();
        match progressive_query(
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
        ) {
            Ok(resp) => Ok(DaemonResponse::QueryKnowledge(resp)),
            Err(e) => Ok(map_control_plane_error(e)),
        }
    }

    pub fn inspect_evidence(
        &self,
        req: InspectEvidenceRequest,
    ) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let policy = ports.production_policy();
        let principal = resolve_principal(req.principal_id.as_deref());
        let scope = match req.scope.as_deref() {
            Some(s) => match parse_scope_key(s) {
                Ok(sc) => sc,
                Err(e) => return Ok(map_control_plane_error(e)),
            },
            None => {
                return Ok(DaemonResponse::Error(ApiError::new(
                    "INVALID_PAYLOAD",
                    "inspect_evidence requires scope",
                )));
            }
        };
        let event_store = ports.store();
        match expand_handle(
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
        ) {
            Ok(preview) => Ok(DaemonResponse::EvidencePreview(preview)),
            Err(e) => Ok(map_control_plane_error(e)),
        }
    }

    pub fn inspect_source(&self, req: InspectSourceRequest) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let principal = resolve_principal(req.principal_id.as_deref());
        let scope = match req.scope.as_deref() {
            Some(s) => match parse_scope_key(s) {
                Ok(sc) => sc,
                Err(e) => return Ok(map_control_plane_error(e)),
            },
            None => {
                return Ok(DaemonResponse::Error(ApiError::new(
                    "INVALID_PAYLOAD",
                    "inspect_source requires scope",
                )));
            }
        };
        let policy = ports.production_policy();
        let policy_ctx = PolicyContext::default_for_privacy(Privacy::LocalOnly);
        match policy.allow(
            principal.id,
            GrantCapability::ReadEvidence,
            &scope,
            &policy_ctx,
        ) {
            Ok(true) => {}
            Ok(false) => {
                return Ok(map_control_plane_error(ControlPlaneError::PolicyDenied(
                    "ReadEvidence denied for inspect_source".into(),
                )));
            }
            Err(e) => return Ok(map_control_plane_error(e)),
        }
        let source_id = match SourceId::from_str(&req.id) {
            Ok(id) => id,
            Err(_) => {
                return Ok(DaemonResponse::Error(ApiError::new(
                    "INVALID_PAYLOAD",
                    format!("invalid source id: {}", req.id),
                )));
            }
        };
        let expected_scope = scope_identity_key(&scope);
        let ports = self.ports();
        match ports.query.get_source(source_id) {
            Ok(Some(row)) if row.scope == expected_scope => {
                Ok(DaemonResponse::Source(source_row_to_dto(&row)))
            }
            // Missing, empty legacy scope on non-empty request, or other-scope:
            // NOT_FOUND (anti-enumeration — do not leak existence across scopes).
            Ok(Some(_)) | Ok(None) => Ok(DaemonResponse::Error(ApiError::new(
                "NOT_FOUND",
                format!("source {}", req.id),
            ))),
            Err(e) => Ok(map_control_plane_error(e)),
        }
    }

    pub fn list_review_items(
        &self,
        req: ListReviewItemsRequest,
    ) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let principal = resolve_principal(req.principal_id.as_deref());
        let scope = match req.scope.as_deref() {
            Some(s) => match parse_scope_key(s) {
                Ok(sc) => sc,
                Err(e) => return Ok(map_control_plane_error(e)),
            },
            None => {
                return Ok(DaemonResponse::Error(ApiError::new(
                    "INVALID_PAYLOAD",
                    "list_review_items requires scope",
                )));
            }
        };
        let policy = ports.production_policy();
        let policy_ctx = PolicyContext::default_for_privacy(Privacy::LocalOnly);
        match policy.allow(
            principal.id,
            GrantCapability::ReadConclusions,
            &scope,
            &policy_ctx,
        ) {
            Ok(true) => {}
            Ok(false) => {
                return Ok(map_control_plane_error(ControlPlaneError::PolicyDenied(
                    "ReadConclusions denied for list_review_items".into(),
                )));
            }
            Err(e) => return Ok(map_control_plane_error(e)),
        }
        let scope_key = scope_identity_key(&scope);
        // Scope isolation: shared CP filter (CLI local path uses the same helper).
        let mut items = match list_open_review_items_for_scope(&ports.query, &scope_key) {
            Ok(items) => items,
            Err(e) => return Ok(map_control_plane_error(e)),
        };
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

    pub fn list_sources(&self, req: ListSourcesRequest) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let principal = resolve_principal(req.principal_id.as_deref());
        let scope = match req.scope.as_deref() {
            Some(s) => match parse_scope_key(s) {
                Ok(sc) => sc,
                Err(e) => return Ok(map_control_plane_error(e)),
            },
            None => {
                return Ok(DaemonResponse::Error(ApiError::new(
                    "INVALID_PAYLOAD",
                    "list_sources requires scope",
                )));
            }
        };
        let policy = ports.production_policy();
        let policy_ctx = PolicyContext::default_for_privacy(Privacy::LocalOnly);
        match policy.allow(
            principal.id,
            GrantCapability::ReadEvidence,
            &scope,
            &policy_ctx,
        ) {
            Ok(true) => {}
            Ok(false) => {
                // T203 F11: new list paths attach details.hint (parity with CLI local deny).
                return Ok(policy_denied_with_hint(
                    "ReadEvidence denied for list_sources",
                ));
            }
            Err(e) => return Ok(map_control_plane_error(e)),
        }
        let page = clamp_list_limit(req.limit);
        let scope_key = scope_identity_key(&scope);
        let mut rows = match ports.query.list_sources_for_scope(&scope_key, page + 1) {
            Ok(rows) => rows,
            Err(e) => return Ok(map_control_plane_error(e)),
        };
        let more_available = rows.len() > page;
        if more_available {
            rows.truncate(page);
        }
        let items = rows.iter().map(source_row_to_dto).collect();
        Ok(DaemonResponse::SourceList(
            SourceListResponse::new(items).with_more(more_available),
        ))
    }

    pub fn list_evidence(&self, req: ListEvidenceRequest) -> Result<DaemonResponse, BoxError> {
        let ports = self.ports();
        let principal = resolve_principal(req.principal_id.as_deref());
        let scope = match req.scope.as_deref() {
            Some(s) => match parse_scope_key(s) {
                Ok(sc) => sc,
                Err(e) => return Ok(map_control_plane_error(e)),
            },
            None => {
                return Ok(DaemonResponse::Error(ApiError::new(
                    "INVALID_PAYLOAD",
                    "list_evidence requires scope",
                )));
            }
        };
        let policy = ports.production_policy();
        let policy_ctx = PolicyContext::default_for_privacy(Privacy::LocalOnly);
        match policy.allow(
            principal.id,
            GrantCapability::ReadEvidence,
            &scope,
            &policy_ctx,
        ) {
            Ok(true) => {}
            Ok(false) => {
                // T203 F11: new list paths attach details.hint (parity with CLI local deny).
                return Ok(policy_denied_with_hint(
                    "ReadEvidence denied for list_evidence",
                ));
            }
            Err(e) => return Ok(map_control_plane_error(e)),
        }
        let page = clamp_list_limit(req.limit);
        let scope_key = scope_identity_key(&scope);
        let mut rows =
            match ports
                .query
                .list_evidence_for_scope(&scope_key, req.query.as_deref(), page + 1)
            {
                Ok(rows) => rows,
                Err(e) => return Ok(map_control_plane_error(e)),
            };
        let more_available = rows.len() > page;
        if more_available {
            rows.truncate(page);
        }
        let items: Vec<EvidenceListItemDto> = rows
            .into_iter()
            .map(|r| EvidenceListItemDto {
                id: r.id.to_string(),
                summary: truncate_evidence_list_summary(&r.summary),
                status: r.status,
                source_id: r.source_id.to_string(),
                recorded_at: Some(offset_to_utc(r.recorded_at)),
            })
            .collect();
        Ok(DaemonResponse::EvidenceList(
            EvidenceListResponse::new(items).with_more(more_available),
        ))
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
        GovernedMutation::WipeContentEnvelope(req) => process_wipe_content_envelope(ports, req),
    }
}

/// Governed mutation kinds handled on the writer queue.
#[derive(Debug, Clone)]
pub enum GovernedMutation {
    ProposeConclusion(WireProposeConclusion),
    ProposeDecision(WireProposeDecision),
    ResolveReviewItem(ResolveReviewItemRequest),
    RequestErasure(RequestErasureRequest),
    WipeContentEnvelope(WipeContentEnvelopeRequest),
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
    let evidence_ids = match parse_evidence_ids_strict(&req.evidence_ids) {
        Ok(ids) => ids,
        Err(bad) => {
            return Ok(DaemonResponse::Error(ApiError::new(
                "INVALID_PAYLOAD",
                format!("invalid evidence id: {bad}"),
            )));
        }
    };
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
        Err(e) => map_mutation_control_plane_error(e),
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
    let conclusion_ids = match parse_conclusion_ids_strict(&req.conclusion_ids) {
        Ok(parsed) if parsed.is_empty() => None,
        Ok(parsed) => Some(parsed),
        Err(bad) => {
            return Ok(DaemonResponse::Error(ApiError::new(
                "INVALID_PAYLOAD",
                format!("invalid conclusion id: {bad}"),
            )));
        }
    };
    let evidence_ids = match parse_evidence_ids_strict(&req.evidence_ids) {
        Ok(parsed) if parsed.is_empty() => None,
        Ok(parsed) => Some(parsed),
        Err(bad) => {
            return Ok(DaemonResponse::Error(ApiError::new(
                "INVALID_PAYLOAD",
                format!("invalid evidence id: {bad}"),
            )));
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
        Err(e) => map_mutation_control_plane_error(e),
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
    // resolution is primary; non-empty note is appended (never overwrite resolution).
    let reason = match req.note.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
        Some(note) => format!("{} ({})", req.resolution, note),
        None => req.resolution.clone(),
    };
    // command_id is used only for spool durability (filename); CP detect-already-done
    // keys on review_item_id + status, not a command_id-derived domain id.

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
        Err(e) => map_mutation_control_plane_error(e),
    }
}

fn process_request_erasure(
    ports: &StorePorts,
    req: RequestErasureRequest,
) -> Result<DaemonResponse, BoxError> {
    // 1. Resolve principal
    let principal = resolve_principal(req.principal_id.as_deref());
    let request_id = match req.command_id.as_deref() {
        Some(cid) if !cid.trim().is_empty() => id_from_command(NS_REQUEST_ERASURE, cid).to_string(),
        _ => Uuid::new_v4().to_string(),
    };

    // 2. Require + parse scope
    let scope = match req.scope.as_deref() {
        Some(s) => match parse_scope_key(s) {
            Ok(sc) => sc,
            Err(e) => return Ok(map_control_plane_error(e)),
        },
        None => {
            return Ok(DaemonResponse::Error(ApiError::new(
                "INVALID_PAYLOAD",
                "request_erasure requires scope",
            )));
        }
    };

    // 3. Policy gate (always — before ticket short-circuit so replay without grant denies)
    let policy = ports.production_policy();
    let policy_ctx = PolicyContext::default_for_privacy(Privacy::LocalOnly);
    match policy.allow(principal.id, GrantCapability::Erase, &scope, &policy_ctx) {
        Ok(true) => {}
        Ok(false) => {
            return map_mutation_control_plane_error(ControlPlaneError::PolicyDenied(
                "Erase denied".into(),
            ));
        }
        Err(e) => return map_mutation_control_plane_error(e),
    }

    // 4. Detect-already-done (same principal+grant) or append
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

    // Append failure is retriable infra — return Err so command_id spool is retained.
    ports
        .writer
        .append_events(&[event])
        .map_err(|e| -> BoxError { e.to_string().into() })?;

    let mut resp = ErasureAcceptedResponse::new(request_id, "accepted");
    resp.warnings.push(ERASURE_CE_WIPE_WARNING.to_string());
    Ok(DaemonResponse::ErasureAccepted(resp))
}

fn process_wipe_content_envelope(
    ports: &StorePorts,
    req: WipeContentEnvelopeRequest,
) -> Result<DaemonResponse, BoxError> {
    // 1. Principal
    let principal = resolve_principal(req.principal_id.as_deref());

    // 2. Scope
    let scope = match parse_scope_key(&req.scope) {
        Ok(sc) => sc,
        Err(e) => return Ok(map_control_plane_error(e)),
    };

    // 3. Content key id
    let content_key_id = match parse_content_key_id(&req.content_key_id) {
        Ok(id) => id,
        Err(e) => return Ok(map_control_plane_error(e)),
    };

    // 4. Deterministic tombstone from command_id when present
    let tombstone_id = req
        .command_id
        .as_deref()
        .filter(|c| !c.trim().is_empty())
        .map(tombstone_id_from_command);

    let side = StoreContentEnvelopeWipe::new(ports.store());
    let policy = ports.production_policy();
    match wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &policy,
        &side,
        WipeContentEnvelopeCommand {
            principal,
            content_key_id,
            scope,
            reason: req.reason,
            tombstone_id,
            dry_run: req.dry_run,
            confirm: req.confirm,
        },
    ) {
        Ok(resp) => Ok(DaemonResponse::ContentEnvelopeWiped(resp)),
        Err(e) => map_mutation_control_plane_error(e),
    }
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
///
/// Used for queries (all CP errors) and mutation **terminal** domain outcomes.
pub fn map_control_plane_error(err: ControlPlaneError) -> DaemonResponse {
    let (code, message) = control_plane_error_parts(&err);
    DaemonResponse::Error(ApiError::new(code, message))
}

/// Stable remediation template for POLICY_DENIED `details.hint` (T201 F6 / T203 F11 / T210 F12).
///
/// Kept in-daemon (not a CLI dep) with the same wording as
/// `ai_brains_cli::governed_common::POLICY_DENIED_HINT` — keep dual-site strings in sync.
const POLICY_DENIED_HINT: &str = "ensure a grant for this capability exists; run `ai-brains policy bootstrap --scope …` (or check with `ai-brains policy show --scope …`)";

/// Build POLICY_DENIED with non-empty `details.hint` for new discovery list paths.
fn policy_denied_with_hint(message: impl Into<String>) -> DaemonResponse {
    let mut map = serde_json::Map::new();
    map.insert(
        "hint".to_string(),
        serde_json::Value::String(POLICY_DENIED_HINT.to_string()),
    );
    DaemonResponse::Error(
        ApiError::new("POLICY_DENIED", message).with_details(serde_json::Value::Object(map)),
    )
}

/// Store/clock failures that must keep the command_id spool for restart replay.
///
/// Terminal domain outcomes (`PolicyDenied`, `NotFound`, `InvalidPayload`,
/// `ApprovalRequired`, `InvalidTransition`, `UnsupportedCannotConfirm`,
/// `IdentityConflict`, `Fingerprint`) complete handling → spool deleted.
pub fn is_retriable_control_plane_error(err: &ControlPlaneError) -> bool {
    matches!(
        err,
        ControlPlaneError::EventAppend(_)
            | ControlPlaneError::Query(_)
            | ControlPlaneError::Clock(_)
    )
}

/// Mutation-path mapping: terminal domain → `Ok(Error(...))` (spool deleted);
/// retriable infra → `Err` (writer retains spool for restart replay).
pub fn map_mutation_control_plane_error(
    err: ControlPlaneError,
) -> Result<DaemonResponse, BoxError> {
    if is_retriable_control_plane_error(&err) {
        Err(err.to_string().into())
    } else {
        Ok(map_control_plane_error(err))
    }
}

fn control_plane_error_parts(err: &ControlPlaneError) -> (&'static str, String) {
    match err {
        ControlPlaneError::PolicyDenied(m) => ("POLICY_DENIED", m.clone()),
        ControlPlaneError::NotFound(m) => ("NOT_FOUND", m.clone()),
        ControlPlaneError::InvalidPayload(m) => ("INVALID_PAYLOAD", m.clone()),
        ControlPlaneError::ApprovalRequired(m) => ("APPROVAL_REQUIRED", m.clone()),
        ControlPlaneError::InvalidTransition(m) => ("INVALID_TRANSITION", m.clone()),
        ControlPlaneError::NotEnvelopeBacked(m) => ("NOT_ENVELOPE_BACKED", m.clone()),
        // Terminal domain codes without a dedicated frozen wire code → INTERNAL.
        // Retriable infra (EventAppend/Query/Clock) also map here when structured;
        // mutation path surfaces those as Err instead of this mapper.
        other => ("INTERNAL", other.to_string()),
    }
}

/// Well-known System principal UUID shared with CLI `cli_principal` (briefing.rs).
const CLI_SYSTEM_PRINCIPAL_U128: u128 = 0xA1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2;

/// Resolve principal for daemon IPC (see module docs).
pub fn resolve_principal(wire_principal_id: Option<&str>) -> Principal {
    if let Some(raw) = wire_principal_id {
        let trimmed = raw.trim();
        if let Ok(u) = Uuid::parse_str(trimmed) {
            // Preserve System kind for the well-known CLI System principal so
            // wire identity + kind match local CP (policy matrix parity).
            if u.as_u128() == CLI_SYSTEM_PRINCIPAL_U128 {
                return make_principal(
                    PrincipalKind::System,
                    PrincipalId::from_uuid(u),
                    "daemon-system",
                );
            }
            return make_principal(
                PrincipalKind::Human,
                PrincipalId::from_uuid(u),
                "daemon-human",
            );
        }
    }
    // Legacy clients that omit wire principal_id may still set daemon env.
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
        PrincipalId::from_uuid(Uuid::from_u128(CLI_SYSTEM_PRINCIPAL_U128)),
        "daemon-system",
    )
}

/// Sanitize command_id for use as a spool filename stem component.
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

/// Stable op kind for governed spool filenames (avoids cross-op command_id collision).
pub fn governed_mutation_op_kind(op: &GovernedMutation) -> &'static str {
    match op {
        GovernedMutation::ProposeConclusion(_) => "propose_conclusion",
        GovernedMutation::ProposeDecision(_) => "propose_decision",
        GovernedMutation::ResolveReviewItem(_) => "resolve_review_item",
        GovernedMutation::RequestErasure(_) => "request_erasure",
        GovernedMutation::WipeContentEnvelope(_) => "wipe_content_envelope",
    }
}

/// Spool filename stem: `{op}_{sanitized_command_id}` (caller appends `.json`).
pub fn governed_spool_stem(op_kind: &str, command_id: &str) -> String {
    format!(
        "{}_{}",
        op_kind,
        sanitize_command_id_for_filename(command_id)
    )
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

/// Strict parse: any malformed id → `Err` with the bad id string (caller maps to INVALID_PAYLOAD).
/// Empty input remains allowed (unsupported conclusion path / optional support links).
fn parse_evidence_ids_strict(ids: &[String]) -> Result<Vec<EvidenceId>, String> {
    let mut out = Vec::with_capacity(ids.len());
    for s in ids {
        match EvidenceId::from_str(s) {
            Ok(id) => out.push(id),
            Err(_) => return Err(s.clone()),
        }
    }
    Ok(out)
}

/// Strict parse for conclusion ids (same contract as evidence ids).
fn parse_conclusion_ids_strict(ids: &[String]) -> Result<Vec<ConclusionId>, String> {
    let mut out = Vec::with_capacity(ids.len());
    for s in ids {
        match ConclusionId::from_str(s) {
            Ok(id) => out.push(id),
            Err(_) => return Err(s.clone()),
        }
    }
    Ok(out)
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
    fn policy_denied_with_hint__includes_details_hint() {
        let resp = policy_denied_with_hint("ReadEvidence denied for list_sources");
        match resp {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "POLICY_DENIED");
                assert!(err.message.contains("list_sources"));
                let hint = err
                    .details
                    .as_ref()
                    .and_then(|d| d.get("hint"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                assert!(
                    !hint.is_empty() && hint.contains("bootstrap"),
                    "expected non-empty details.hint mentioning bootstrap, got {hint:?}"
                );
                assert!(
                    hint.contains("policy show") || hint.contains("policy bootstrap"),
                    "expected secondary show/bootstrap remediation, got {hint:?}"
                );
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn map_control_plane_error__not_found__code() {
        let resp = map_control_plane_error(ControlPlaneError::NotFound("missing item".into()));
        match resp {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "NOT_FOUND");
                assert!(err.message.contains("missing item"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn map_control_plane_error__invalid_payload__code() {
        let resp =
            map_control_plane_error(ControlPlaneError::InvalidPayload("bad scope key".into()));
        match resp {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "INVALID_PAYLOAD");
                assert!(err.message.contains("bad scope"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn map_control_plane_error__approval_required__code() {
        let resp =
            map_control_plane_error(ControlPlaneError::ApprovalRequired("needs review".into()));
        match resp {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "APPROVAL_REQUIRED");
                assert!(err.message.contains("needs review"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn map_control_plane_error__invalid_transition__code() {
        let resp = map_control_plane_error(ControlPlaneError::InvalidTransition(
            "already resolved".into(),
        ));
        match resp {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "INVALID_TRANSITION");
                assert!(err.message.contains("already resolved"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn map_control_plane_error__event_append__internal_code() {
        let resp = map_control_plane_error(ControlPlaneError::EventAppend("disk full".into()));
        match resp {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "INTERNAL");
                assert!(err.message.contains("disk full") || err.message.contains("append"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn is_retriable_control_plane_error__infra_vs_terminal() {
        assert!(is_retriable_control_plane_error(
            &ControlPlaneError::EventAppend("x".into())
        ));
        assert!(is_retriable_control_plane_error(&ControlPlaneError::Query(
            "x".into()
        )));
        assert!(is_retriable_control_plane_error(&ControlPlaneError::Clock(
            "x".into()
        )));
        assert!(!is_retriable_control_plane_error(
            &ControlPlaneError::PolicyDenied("x".into())
        ));
        assert!(!is_retriable_control_plane_error(
            &ControlPlaneError::NotFound("x".into())
        ));
        assert!(!is_retriable_control_plane_error(
            &ControlPlaneError::InvalidPayload("x".into())
        ));
        assert!(!is_retriable_control_plane_error(
            &ControlPlaneError::ApprovalRequired("x".into())
        ));
        assert!(!is_retriable_control_plane_error(
            &ControlPlaneError::InvalidTransition("x".into())
        ));
        assert!(!is_retriable_control_plane_error(
            &ControlPlaneError::UnsupportedCannotConfirm("x".into())
        ));
        assert!(!is_retriable_control_plane_error(
            &ControlPlaneError::IdentityConflict("x".into())
        ));
        assert!(!is_retriable_control_plane_error(
            &ControlPlaneError::Fingerprint("x".into())
        ));
        // T165: NotEnvelopeBacked is terminal (spool deleted) and maps NOT_ENVELOPE_BACKED.
        assert!(!is_retriable_control_plane_error(
            &ControlPlaneError::NotEnvelopeBacked("legacy memory".into())
        ));
        match map_control_plane_error(ControlPlaneError::NotEnvelopeBacked("legacy".into())) {
            DaemonResponse::Error(err) => {
                assert_eq!(err.code, "NOT_ENVELOPE_BACKED");
                assert!(err.message.contains("legacy"));
            }
            other => panic!("expected NOT_ENVELOPE_BACKED Error, got {other:?}"),
        }
        let mapped =
            map_mutation_control_plane_error(ControlPlaneError::NotEnvelopeBacked("x".into()));
        assert!(
            mapped.is_ok(),
            "terminal NotEnvelopeBacked must Ok(Error) so spool is deleted"
        );
    }

    #[test]
    fn map_mutation_control_plane_error__retriable_infra__returns_err() {
        let result =
            map_mutation_control_plane_error(ControlPlaneError::EventAppend("simulated".into()));
        assert!(
            result.is_err(),
            "EventAppend must be Err so process_governed_on_writer retains spool"
        );
        let result = map_mutation_control_plane_error(ControlPlaneError::Query("q".into()));
        assert!(result.is_err());
        let result = map_mutation_control_plane_error(ControlPlaneError::Clock("c".into()));
        assert!(result.is_err());
    }

    #[test]
    fn map_mutation_control_plane_error__terminal_domain__returns_ok_error() {
        let result =
            map_mutation_control_plane_error(ControlPlaneError::PolicyDenied("denied".into()));
        match result {
            Ok(DaemonResponse::Error(err)) => assert_eq!(err.code, "POLICY_DENIED"),
            other => panic!("expected Ok(Error POLICY_DENIED), got {other:?}"),
        }
        let result =
            map_mutation_control_plane_error(ControlPlaneError::Fingerprint("fp fail".into()));
        match result {
            Ok(DaemonResponse::Error(err)) => assert_eq!(err.code, "INTERNAL"),
            other => panic!("expected Ok(Error INTERNAL), got {other:?}"),
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

    #[test]
    fn governed_spool_stem__includes_op_kind() {
        let stem = governed_spool_stem("propose_conclusion", "cmd-1");
        assert_eq!(stem, "propose_conclusion_cmd-1");
        let stem2 = governed_spool_stem("request_erasure", "cmd-1");
        assert_eq!(stem2, "request_erasure_cmd-1");
        assert_ne!(
            stem, stem2,
            "same command_id different ops must not collide"
        );
    }

    #[test]
    fn parse_evidence_ids_strict__malformed__returns_bad_id() {
        let bad = vec!["not-a-uuid".to_string()];
        match parse_evidence_ids_strict(&bad) {
            Err(s) => assert_eq!(s, "not-a-uuid"),
            Ok(_) => panic!("expected Err with bad id"),
        }
    }

    #[test]
    fn parse_evidence_ids_strict__empty__ok() {
        let empty: Vec<String> = Vec::new();
        let parsed = parse_evidence_ids_strict(&empty).expect("empty allowed");
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_conclusion_ids_strict__malformed__returns_bad_id() {
        let bad = vec!["bad-conclusion".to_string()];
        match parse_conclusion_ids_strict(&bad) {
            Err(s) => assert_eq!(s, "bad-conclusion"),
            Ok(_) => panic!("expected Err with bad id"),
        }
    }

    #[test]
    fn resolve_principal__wire_system_uuid__system_kind() {
        let system_uuid = Uuid::from_u128(CLI_SYSTEM_PRINCIPAL_U128);
        let p = resolve_principal(Some(&system_uuid.to_string()));
        assert!(matches!(p.kind, PrincipalKind::System));
        assert_eq!(p.id.to_string(), system_uuid.to_string());
        assert_eq!(p.display_name, "daemon-system");
    }

    #[test]
    fn resolve_principal__wire_other_uuid__human_kind() {
        let id = Uuid::parse_str("11111111-2222-3333-4444-555555555555").expect("fixture uuid");
        let p = resolve_principal(Some(&id.to_string()));
        assert!(matches!(p.kind, PrincipalKind::Human));
        assert_eq!(p.id.to_string(), id.to_string());
        assert_eq!(p.display_name, "daemon-human");
    }

    #[test]
    fn resolve_principal__no_wire_with_env__human_from_env() {
        use ai_brains_core::temp_env::TempEnv;
        let env_id = Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").expect("fixture uuid");
        let _guard = TempEnv::set("AI_BRAINS_DAEMON_PRINCIPAL_ID", env_id.to_string());
        let p = resolve_principal(None);
        assert!(matches!(p.kind, PrincipalKind::Human));
        assert_eq!(p.id.to_string(), env_id.to_string());
        assert_eq!(p.display_name, "daemon-env-human");
    }

    #[test]
    fn resolve_principal__no_wire_no_env__system_default() {
        use ai_brains_core::temp_env::TempEnv;
        let _guard = TempEnv::remove("AI_BRAINS_DAEMON_PRINCIPAL_ID");
        let p = resolve_principal(None);
        let system_uuid = Uuid::from_u128(CLI_SYSTEM_PRINCIPAL_U128);
        assert!(matches!(p.kind, PrincipalKind::System));
        assert_eq!(p.id.to_string(), system_uuid.to_string());
        assert_eq!(p.display_name, "daemon-system");
    }

    #[test]
    fn resolve_principal__wire_present_overrides_env() {
        use ai_brains_core::temp_env::TempEnv;
        let env_id = Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").expect("env fixture");
        let wire_id =
            Uuid::parse_str("11111111-2222-3333-4444-555555555555").expect("wire fixture");
        let _guard = TempEnv::set("AI_BRAINS_DAEMON_PRINCIPAL_ID", env_id.to_string());
        let p = resolve_principal(Some(&wire_id.to_string()));
        assert_eq!(
            p.id.to_string(),
            wire_id.to_string(),
            "wire principal_id must win over AI_BRAINS_DAEMON_PRINCIPAL_ID"
        );
        assert!(matches!(p.kind, PrincipalKind::Human));
    }
}
