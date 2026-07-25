//! Governed-memory control plane: ports, adapters, and workflows (T148–T150).

pub mod adapters;
pub mod conclusions;
pub mod conflicts;
pub mod decisions;
pub mod errors;
pub mod invalidation;
pub mod ports;
pub mod review;
pub mod sources;

pub use adapters::{
    AllowAllPolicy, DenyAllPolicy, Sha256FingerprinterPort, StoreEventWriter, StoreGovernedQuery,
    StorePorts, SystemClock,
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
pub use decisions::{
    ProposeDecisionRequest, ProposeDecisionResult, approve_decision, propose_decision,
    revoke_decision, supersede_decision,
};
pub use errors::{ControlPlaneError, Result};
pub use invalidation::{
    InvalidationResult, SourceUnavailableRequest, invalidate_dependents_for_changed_source,
    mark_source_unavailable, plan_invalidation_events_for_changed_source,
    revalidate_matching_stale, try_mark_stale_payload,
};
pub use ports::{
    ClaimConflictRow, Clock, ConclusionRow, DecisionRow, EventWriter, Fingerprinter,
    GovernedQueryStore, PolicyEvaluator, ReviewItemRow, StaleFact,
};
pub use review::resolve_review_item;
pub use sources::{
    ObserveSourceRequest, ObserveSourceResult, SourceContent, normalize_path_locator,
    observe_source, scope_identity_key, source_identity_string,
};
