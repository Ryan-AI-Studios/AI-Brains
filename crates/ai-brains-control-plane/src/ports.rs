use ai_brains_core::ids::{ConclusionId, DecisionId, PrincipalId};
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_events::Envelope;
use time::OffsetDateTime;

use crate::errors::Result;

/// Append governed (and legacy) events atomically.
pub trait EventWriter {
    fn append_events(&self, events: &[Envelope]) -> Result<()>;
}

/// Typed projection reads for governed memory (implemented in later phases).
///
/// Uses governed newtypes so a legacy [`ai_brains_core::ids::MemoryId`] cannot be
/// passed where a [`DecisionId`] is required (T148 dual-model boundary).
pub trait GovernedQueryStore {
    fn has_conclusion(&self, conclusion_id: ConclusionId) -> Result<bool>;
    fn has_decision(&self, decision_id: DecisionId) -> Result<bool>;
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
