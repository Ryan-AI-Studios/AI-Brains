//! Governed-memory control plane: ports, adapters, and workflows (T148–T151).
//!
//! # Policy (T151)
//!
//! **Production must** construct the evaluator via
//! [`StorePorts::production_policy`] (or [`StorePorts::policy_evaluator`]),
//! which returns [`DefaultPolicyEvaluator`] over [`StoreGrantPrincipalStore`].
//! Do **not** use [`AllowAllPolicy`] outside tests — it bypasses the deny-by-default
//! matrix and is retained only for integration harnesses that exercise non-policy paths.

pub mod adapters;
pub mod briefings;
pub mod class_based_retention;
pub mod command_id;
pub mod conclusions;
pub mod conflicts;
pub mod cryptographic_erasure;
pub mod decisions;
pub mod errors;
pub mod grants;
pub mod invalidation;
pub mod legacy_import;
pub mod policy;
pub mod ports;
pub mod query;
pub mod review;
pub mod scope_resolver;
pub mod sources;

pub use adapters::{
    AllowAllPolicy, DenyAllPolicy, Sha256FingerprinterPort, StoreEventWriter, StoreGovernedQuery,
    StoreGrantPrincipalStore, StorePorts, StoreScopeIdentityStore, SystemClock,
};
pub use briefings::{
    BRIEFING_POLICY_VERSION, BudgetConfig, PersonalBriefingRequest, ProjectBriefingRequest,
    apply_budget, apply_personal_budget, build_personal_briefing, build_project_briefing,
    render_personal_markdown, render_project_json, render_project_markdown,
};
pub use class_based_retention::{
    MAX_RETENTION_HORIZON_DAYS, NS_RETENTION_APPLY, RetentionApplyCommand, RetentionConfig,
    RetentionProjectionApplyOutcome, apply_retention, apply_retention_projections,
    cascade_memory_ids_for_keys, execute_retention_projection_deletes, finalize_retention_apply,
    nightly_ce_enabled, parse_positive_horizon_days, plan_retention, prepare_retention_apply,
};
pub use command_id::{
    NS_PROPOSE_CONCLUSION, NS_PROPOSE_DECISION, NS_REQUEST_ERASURE, NS_WIPE_CONTENT_ENVELOPE,
    id_from_command,
};
pub use conclusions::{
    ProposeConclusionRequest, ProposeConclusionResult, activate_conclusion, approve_conclusion,
    confirm_conclusion, correct_conclusion, principal as make_principal, propose_conclusion,
    reject_conclusion,
};
pub use conflicts::{
    OpenClaimConflictRequest, current_successor, equal_authority_conflict, open_claim_conflict,
    open_conflicts_snapshot, prefer_decision_over_candidate, resolve_claim_conflict,
    resolve_scope_preference, select_conclusions_valid_at,
};
pub use cryptographic_erasure::{
    ContentEnvelopeWipeStore, ContentKeyStatus, StoreContentEnvelopeWipe,
    WipeContentEnvelopeCommand, parse_content_key_id, tombstone_id_from_command,
    wipe_content_envelope,
};
pub use decisions::{
    ProposeDecisionRequest, ProposeDecisionResult, approve_decision, propose_decision,
    revoke_decision, supersede_decision,
};
pub use errors::{ControlPlaneError, Result};
pub use grants::{
    RemoteIdentityKey, issue_grant, join_repository, register_path_alias, register_principal,
    register_workspace, revoke_grant, set_repository_ledgerful_id, upsert_repository_identity,
};
pub use invalidation::{
    InvalidationResult, SourceUnavailableRequest, invalidate_dependents_for_changed_source,
    mark_source_unavailable, plan_invalidation_events_for_changed_source,
    revalidate_matching_stale, try_mark_stale_payload,
};
pub use legacy_import::{
    ApplyOpts, ImportAction, ImportActionKind, ImportMechanism, ImportOpts, ImportPlan,
    ImportReport, ImportTotals, LEGACY_SOURCE_DISPLAY_NAME, LEGACY_SOURCE_NAME,
    NS_LEGACY_CONCLUSION, NS_LEGACY_DECISION, NS_LEGACY_EVIDENCE, NS_LEGACY_IMPORT_BATCH,
    NS_LEGACY_REVIEW, NS_LEGACY_SOURCE, REASON_SUPERSEDED_DUPLICATE_PIN, apply_legacy_import,
    classify_legacy, compute_plan_hash, legacy_conclusion_id, legacy_decision_id,
    legacy_evidence_id, legacy_review_id, legacy_source_id, plan_report_json,
};
pub use policy::{DefaultPolicyEvaluator, GrantPrincipalStore, PolicyDecisionEntry, reduce_grants};
pub use ports::{
    ClaimConflictRow, Clock, ConclusionRow, ConnectorTrust, DecisionRow, EventWriter,
    Fingerprinter, GovernedQueryStore, PolicyContext, PolicyEvaluator, ProcessingRoute,
    ReviewItemRow, SourceRow, StaleFact,
};
pub use query::{
    ExpandHandleRequest, GetQueryTraceRequest, ProgressiveQueryRequest, expand_handle,
    get_query_trace, progressive_query,
};
pub use review::{
    list_open_review_items_for_scope, resolve_review_item, review_item_matches_scope,
};
pub use scope_resolver::{
    ResolutionEvidence, ResolvedScope, ScopeConfidence, ScopeIdentityStore, ScopeResolveInput,
    is_authoritative, resolve_scope,
};
pub use sources::{
    ObserveSourceRequest, ObserveSourceResult, SourceContent, normalize_path_locator,
    observe_source, parse_scope_key, scope_identity_key, source_identity_string, source_row_to_dto,
};
