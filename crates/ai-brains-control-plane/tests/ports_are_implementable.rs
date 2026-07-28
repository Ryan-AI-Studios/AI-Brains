#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_control_plane::{
    ClaimConflictRow, Clock, ConclusionRow, ControlPlaneError, DecisionRow, EventWriter,
    Fingerprinter, GovernedQueryStore, PolicyEvaluator, Result, ReviewItemRow, SourceRow,
    StaleFact,
};
use ai_brains_core::ids::{
    ConclusionId, ConflictId, DecisionId, EvidenceId, PrincipalId, ReviewItemId, SourceId,
    SourceVersionId,
};
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_core::source::SourceKind;
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
    fn find_source(
        &self,
        _scope: &str,
        _kind: &SourceKind,
        _locator: Option<&str>,
        _display_name: &str,
    ) -> Result<Option<SourceId>> {
        Ok(None)
    }
    fn get_source(&self, _source_id: SourceId) -> Result<Option<SourceRow>> {
        Ok(None)
    }
    fn latest_source_version(
        &self,
        _source_id: SourceId,
    ) -> Result<Option<(SourceVersionId, String)>> {
        Ok(None)
    }
    fn conclusions_depending_on_source(&self, _source_id: SourceId) -> Result<Vec<ConclusionId>> {
        Ok(Vec::new())
    }
    fn decisions_depending_on_source(&self, _source_id: SourceId) -> Result<Vec<DecisionId>> {
        Ok(Vec::new())
    }
    fn is_conclusion_stale(&self, _conclusion_id: ConclusionId) -> Result<bool> {
        Ok(false)
    }
    fn latest_stale_fact(&self, _conclusion_id: ConclusionId) -> Result<Option<StaleFact>> {
        Ok(None)
    }
    fn source_version_count(&self, _source_id: SourceId) -> Result<u64> {
        Ok(0)
    }
    fn evidence_count_for_source(&self, _source_id: SourceId) -> Result<u64> {
        Ok(0)
    }
    fn get_conclusion(&self, _conclusion_id: ConclusionId) -> Result<Option<ConclusionRow>> {
        Ok(None)
    }
    fn list_conclusions_by_scope_state(
        &self,
        _scope: Option<&str>,
        _state: Option<&str>,
    ) -> Result<Vec<ConclusionRow>> {
        Ok(Vec::new())
    }
    fn get_decision(&self, _decision_id: DecisionId) -> Result<Option<DecisionRow>> {
        Ok(None)
    }
    fn list_decisions(
        &self,
        _scope: Option<&str>,
        _state: Option<&str>,
    ) -> Result<Vec<DecisionRow>> {
        Ok(Vec::new())
    }
    fn list_open_review_items(&self) -> Result<Vec<ReviewItemRow>> {
        Ok(Vec::new())
    }
    fn get_review_item(&self, _review_item_id: ReviewItemId) -> Result<Option<ReviewItemRow>> {
        Ok(None)
    }
    fn list_open_claim_conflicts(&self) -> Result<Vec<ClaimConflictRow>> {
        Ok(Vec::new())
    }
    fn get_claim_conflict(&self, _conflict_id: ConflictId) -> Result<Option<ClaimConflictRow>> {
        Ok(None)
    }
    fn conclusions_valid_at(
        &self,
        _scope: &str,
        _statement: Option<&str>,
        _at: OffsetDateTime,
    ) -> Result<Vec<ConclusionRow>> {
        Ok(Vec::new())
    }
    fn evidence_ids_for_conclusion(&self, _conclusion_id: ConclusionId) -> Result<Vec<EvidenceId>> {
        Ok(Vec::new())
    }
    fn evidence_ids_for_decision(&self, _decision_id: DecisionId) -> Result<Vec<EvidenceId>> {
        Ok(Vec::new())
    }
    fn evidence_privacy(&self, _evidence_id: EvidenceId) -> Result<Option<String>> {
        Ok(None)
    }
    fn conclusion_ids_for_decision(&self, _decision_id: DecisionId) -> Result<Vec<ConclusionId>> {
        Ok(Vec::new())
    }
    fn epistemic_version_vector(&self, _scope: &str, _principal_id: &str) -> Result<String> {
        Ok(String::new())
    }
    fn get_briefing_cache(&self, _cache_key: &str) -> Result<Option<(String, Option<String>)>> {
        Ok(None)
    }
    fn put_briefing_cache(
        &self,
        _cache_key: &str,
        _briefing_type: &str,
        _scope_key: &str,
        _policy_version: &str,
        _source_version_vector: &str,
        _budget: u64,
        _packet_json: &str,
        _generated_at: &str,
        _expires: Option<&str>,
    ) -> Result<()> {
        Ok(())
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
        _ctx: &ai_brains_control_plane::PolicyContext,
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
    assert!(
        q.find_source("", &SourceKind::File, None, "x")
            .expect("find")
            .is_none()
    );
    assert!(
        q.latest_source_version(SourceId::new())
            .expect("latest")
            .is_none()
    );
    assert!(
        q.conclusions_depending_on_source(SourceId::new())
            .expect("deps")
            .is_empty()
    );
    assert!(
        q.decisions_depending_on_source(SourceId::new())
            .expect("deps")
            .is_empty()
    );
    assert!(!q.is_conclusion_stale(ConclusionId::new()).expect("stale"));
    assert!(
        q.latest_stale_fact(ConclusionId::new())
            .expect("fact")
            .is_none()
    );
    assert_eq!(q.source_version_count(SourceId::new()).expect("count"), 0);
    assert_eq!(
        q.evidence_count_for_source(SourceId::new()).expect("count"),
        0
    );

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
            &ai_brains_control_plane::PolicyContext::unspecified(),
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
