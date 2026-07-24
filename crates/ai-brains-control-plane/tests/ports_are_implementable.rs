#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_control_plane::{
    Clock, ControlPlaneError, EventWriter, Fingerprinter, GovernedQueryStore, PolicyEvaluator,
    Result,
};
use ai_brains_core::ids::{ConclusionId, DecisionId, PrincipalId};
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_events::Envelope;
use time::OffsetDateTime;

struct DummyWriter;
impl EventWriter for DummyWriter {
    fn append_events(&self, _events: &[Envelope]) -> Result<()> {
        Ok(())
    }
}

struct DummyQuery;
impl GovernedQueryStore for DummyQuery {
    fn has_conclusion(&self, _conclusion_id: ConclusionId) -> Result<bool> {
        Ok(false)
    }
    fn has_decision(&self, _decision_id: DecisionId) -> Result<bool> {
        Ok(false)
    }
}

struct DummyClock;
impl Clock for DummyClock {
    fn now(&self) -> Result<OffsetDateTime> {
        OffsetDateTime::from_unix_timestamp(0).map_err(|e| ControlPlaneError::Clock(e.to_string()))
    }
}

struct DummyFingerprinter;
impl Fingerprinter for DummyFingerprinter {
    fn fingerprint(&self, content: &[u8]) -> Result<String> {
        Ok(format!("len:{}", content.len()))
    }
}

struct DummyPolicy;
impl PolicyEvaluator for DummyPolicy {
    fn allow(
        &self,
        _principal: PrincipalId,
        _capability: GrantCapability,
        _scope: &ScopeRef,
    ) -> Result<bool> {
        Ok(false)
    }
}

#[test]
fn dummy_ports__implementable_without_panic() {
    let w = DummyWriter;
    w.append_events(&[]).expect("writer ok");

    let q = DummyQuery;
    assert!(!q.has_conclusion(ConclusionId::new()).expect("query"));
    assert!(!q.has_decision(DecisionId::new()).expect("query"));

    let c = DummyClock;
    let _ = c.now().expect("clock");

    let f = DummyFingerprinter;
    assert_eq!(f.fingerprint(b"hi").expect("fp"), "len:2");

    let p = DummyPolicy;
    assert!(
        !p.allow(
            PrincipalId::new(),
            GrantCapability::ReadEvidence,
            &ScopeRef::Personal(ai_brains_core::ids::UserId::new()),
        )
        .expect("policy")
    );
}

#[test]
fn governed_query_store__decision_id_is_not_memory_id_at_api() {
    // Compile-time dual-model proof: GovernedQueryStore::has_decision takes DecisionId.
    // The following would fail to compile if uncommented:
    // let m = ai_brains_core::ids::MemoryId::new();
    // let _ = DummyQuery.has_decision(m);
    let d = DecisionId::new();
    assert!(!DummyQuery.has_decision(d).expect("typed decision query"));
}
