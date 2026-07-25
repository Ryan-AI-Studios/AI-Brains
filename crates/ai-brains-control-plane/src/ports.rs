use ai_brains_core::ids::{ConclusionId, DecisionId, PrincipalId, SourceId, SourceVersionId};
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_core::source::SourceKind;
use ai_brains_events::Envelope;
use time::OffsetDateTime;

use crate::errors::Result;

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
    ) -> Result<bool>;
}
