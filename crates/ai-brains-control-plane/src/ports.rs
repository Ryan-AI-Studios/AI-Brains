use ai_brains_core::ids::{
    ConclusionId, ConflictId, DecisionId, EvidenceId, PrincipalId, ReviewItemId, SourceId,
    SourceVersionId,
};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_core::source::SourceKind;
use ai_brains_events::Envelope;
use time::OffsetDateTime;

use crate::errors::Result;

/// Connector trust posture for policy evaluation (stub for T151 Phase A).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorTrust {
    LocalOnly,
    CloudOk,
    Unknown,
}

/// Processing route for policy evaluation (stub for T151 Phase A).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingRoute {
    Local,
    Cloud,
}

/// Contextual inputs for [`PolicyEvaluator::allow`] beyond principal/capability/scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyContext {
    pub privacy: Privacy,
    pub connector_trust: Option<ConnectorTrust>,
    pub route: Option<ProcessingRoute>,
    pub source_kind: Option<SourceKind>,
}

impl PolicyContext {
    /// Context with only privacy set; trust/route/source unspecified.
    pub fn default_for_privacy(privacy: Privacy) -> Self {
        Self {
            privacy,
            connector_trust: None,
            route: None,
            source_kind: None,
        }
    }

    /// Fully unspecified context; privacy defaults to [`Privacy::LocalOnly`].
    pub fn unspecified() -> Self {
        Self::default_for_privacy(Privacy::LocalOnly)
    }
}

/// Append governed (and legacy) events atomically.
pub trait EventWriter {
    fn append_events(&self, events: &[Envelope]) -> Result<()>;
}

/// Active stale fact for a conclusion (event-log authority).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleFact {
    pub changed_source_version_id: Option<SourceVersionId>,
    pub unavailable_reason: Option<String>,
    pub source_id: Option<SourceId>,
}

/// Row from `conclusion_projection`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConclusionRow {
    pub id: ConclusionId,
    pub state: String,
    pub statement: String,
    pub scope: String,
    pub privacy: String,
    pub proposer: String,
    pub valid_from: OffsetDateTime,
    pub valid_until: Option<OffsetDateTime>,
    pub recorded_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub supersedes: Option<String>,
    pub superseded_by: Option<String>,
    pub protected_category: Option<String>,
    pub unsupported: bool,
}

/// Row from `decision_projection`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRow {
    pub id: DecisionId,
    pub state: String,
    pub title: String,
    pub statement: String,
    pub scope: String,
    pub proposer: String,
    pub approver: Option<String>,
    pub proposal_event_id: Option<String>,
    pub valid_from: Option<OffsetDateTime>,
    pub valid_until: Option<OffsetDateTime>,
    pub recorded_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub superseded_by: Option<String>,
}

/// Row from `review_item_projection`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewItemRow {
    pub id: ReviewItemId,
    pub subject_kind: String,
    pub subject_id: String,
    pub criticality: String,
    pub status: String,
    pub opened_by: String,
    pub subject: String,
    pub resolution: Option<String>,
    pub resolved_by: Option<String>,
    pub related_conclusion_id: Option<String>,
    pub related_decision_id: Option<String>,
    pub related_source_id: Option<String>,
    pub recorded_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Row from `claim_conflict_projection`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimConflictRow {
    pub id: ConflictId,
    pub claim_a_kind: String,
    pub claim_a_id: String,
    pub claim_b_kind: String,
    pub claim_b_id: String,
    pub status: String,
    pub scope: String,
    pub valid_from: Option<OffsetDateTime>,
    pub valid_until: Option<OffsetDateTime>,
    pub explanation: String,
    pub resolution: Option<String>,
    pub recorded_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Typed projection reads for governed memory.
///
/// Uses governed newtypes so a legacy [`ai_brains_core::ids::MemoryId`] cannot be
/// passed where a [`DecisionId`] is required (T148 dual-model boundary).
pub trait GovernedQueryStore {
    fn has_conclusion(&self, conclusion_id: ConclusionId) -> Result<bool>;
    fn has_decision(&self, decision_id: DecisionId) -> Result<bool>;

    /// Resolve source by stable identity within a scope.
    fn find_source(
        &self,
        scope: &str,
        kind: &SourceKind,
        locator: Option<&str>,
        display_name: &str,
    ) -> Result<Option<SourceId>>;

    /// Latest recorded version id + fingerprint for a source.
    fn latest_source_version(
        &self,
        source_id: SourceId,
    ) -> Result<Option<(SourceVersionId, String)>>;

    fn conclusions_depending_on_source(&self, source_id: SourceId) -> Result<Vec<ConclusionId>>;

    fn decisions_depending_on_source(&self, source_id: SourceId) -> Result<Vec<DecisionId>>;

    fn is_conclusion_stale(&self, conclusion_id: ConclusionId) -> Result<bool>;

    /// Latest active stale fact for a conclusion (None if currently activated).
    fn latest_stale_fact(&self, conclusion_id: ConclusionId) -> Result<Option<StaleFact>>;

    fn source_version_count(&self, source_id: SourceId) -> Result<u64>;

    fn evidence_count_for_source(&self, source_id: SourceId) -> Result<u64>;

    // --- T150 typed epistemic rows ---

    fn get_conclusion(&self, conclusion_id: ConclusionId) -> Result<Option<ConclusionRow>>;

    fn list_conclusions_by_scope_state(
        &self,
        scope: Option<&str>,
        state: Option<&str>,
    ) -> Result<Vec<ConclusionRow>>;

    fn get_decision(&self, decision_id: DecisionId) -> Result<Option<DecisionRow>>;

    fn list_decisions(&self, scope: Option<&str>, state: Option<&str>) -> Result<Vec<DecisionRow>>;

    fn list_open_review_items(&self) -> Result<Vec<ReviewItemRow>>;

    fn get_review_item(&self, review_item_id: ReviewItemId) -> Result<Option<ReviewItemRow>>;

    fn list_open_claim_conflicts(&self) -> Result<Vec<ClaimConflictRow>>;

    fn get_claim_conflict(&self, conflict_id: ConflictId) -> Result<Option<ClaimConflictRow>>;

    /// Conclusions whose valid-time window contains `at` (uses valid_from/until, not recorded time).
    fn conclusions_valid_at(
        &self,
        scope: &str,
        statement: Option<&str>,
        at: OffsetDateTime,
    ) -> Result<Vec<ConclusionRow>>;

    /// Evidence ids linked to a conclusion (for briefing authority handles).
    fn evidence_ids_for_conclusion(&self, conclusion_id: ConclusionId) -> Result<Vec<EvidenceId>>;

    /// Evidence ids linked to a decision via decision_support_projection.
    fn evidence_ids_for_decision(&self, decision_id: DecisionId) -> Result<Vec<EvidenceId>>;

    /// Privacy label from `evidence_projection` when a row exists (JSON or bare label).
    ///
    /// Returns `None` when the evidence id is not projected (synthetic handles, missing rows).
    fn evidence_privacy(&self, evidence_id: EvidenceId) -> Result<Option<String>>;

    /// Conclusion ids supporting a decision (also usable as authority handles).
    fn conclusion_ids_for_decision(&self, decision_id: DecisionId) -> Result<Vec<ConclusionId>>;

    /// Compact version vector for briefing cache keys.
    ///
    /// Includes epistemic row counts for `scope`, principal-scoped grant epoch
    /// (active grant count and max issued/revoked timestamps) so grant
    /// issue/revoke forces a cache miss (T152-R2-01), plus scope-linked
    /// source/evidence version counts and max timestamps (T152-P2-01).
    fn epistemic_version_vector(&self, scope: &str, principal_id: &str) -> Result<String>;

    /// Lookup a cached briefing packet by cache key.
    ///
    /// Returns `(packet_json, expires_rfc3339)` when a row exists. Callers must
    /// check expiry against the current clock.
    fn get_briefing_cache(&self, cache_key: &str) -> Result<Option<(String, Option<String>)>>;

    /// Insert or replace a full briefing packet cache row.
    #[allow(clippy::too_many_arguments)]
    fn put_briefing_cache(
        &self,
        cache_key: &str,
        briefing_type: &str,
        scope_key: &str,
        policy_version: &str,
        source_version_vector: &str,
        budget: u64,
        packet_json: &str,
        generated_at: &str,
        expires: Option<&str>,
    ) -> Result<()>;
}

/// Thin clock port (may wrap `ai_brains_core::clock` in adapters).
pub trait Clock {
    fn now(&self) -> Result<OffsetDateTime>;
}

/// Deterministic source fingerprinting.
pub trait Fingerprinter {
    fn fingerprint(&self, content: &[u8]) -> Result<String>;
}

/// Principal + capability + scope policy evaluation.
pub trait PolicyEvaluator {
    fn allow(
        &self,
        principal: PrincipalId,
        capability: GrantCapability,
        scope: &ScopeRef,
        ctx: &PolicyContext,
    ) -> Result<bool>;
}
