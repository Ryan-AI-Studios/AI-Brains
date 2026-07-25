//! Concrete adapters binding control-plane ports to `ai-brains-store` / `ai-brains-sources`.

use ai_brains_core::ids::{ConclusionId, DecisionId, PrincipalId, SourceId, SourceVersionId};
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_core::source::SourceKind;
use ai_brains_events::{Envelope, Payload};
use ai_brains_sources::Sha256Fingerprinter;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::event_store::EventStore;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::errors::{ControlPlaneError, Result};
use crate::ports::{
    Clock, EventWriter, Fingerprinter, GovernedQueryStore, PolicyEvaluator, StaleFact,
};

/// [`EventWriter`] over a real [`SqliteEventStore`] (transactional multi-append).
pub struct StoreEventWriter {
    store: SqliteEventStore,
}

impl StoreEventWriter {
    pub fn new(store: SqliteEventStore) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &SqliteEventStore {
        &self.store
    }

    pub fn into_store(self) -> SqliteEventStore {
        self.store
    }
}

impl EventWriter for StoreEventWriter {
    fn append_events(&self, events: &[Envelope]) -> Result<()> {
        EventStore::append_events(&self.store, events)
            .map_err(|e| ControlPlaneError::EventAppend(e.to_string()))
    }
}

/// Projection + event reads for observation / invalidation workflows.
pub struct StoreGovernedQuery {
    store: SqliteEventStore,
}

impl StoreGovernedQuery {
    pub fn new(store: SqliteEventStore) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &SqliteEventStore {
        &self.store
    }
}

impl GovernedQueryStore for StoreGovernedQuery {
    fn has_conclusion(&self, conclusion_id: ConclusionId) -> Result<bool> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_dependency_projection
                 WHERE parent_type = 'Conclusion' AND parent_id = ?",
                [conclusion_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        Ok(count > 0)
    }

    fn has_decision(&self, decision_id: DecisionId) -> Result<bool> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_dependency_projection
                 WHERE parent_type = 'Decision' AND parent_id = ?",
                [decision_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        Ok(count > 0)
    }

    fn find_source(
        &self,
        scope: &str,
        kind: &SourceKind,
        locator: Option<&str>,
        display_name: &str,
    ) -> Result<Option<SourceId>> {
        let kind_json =
            serde_json::to_string(kind).map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let identity = locator.unwrap_or(display_name);
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let row: Option<String> = match conn.query_row(
            "SELECT source_id FROM source_projection
             WHERE scope = ?
               AND kind = ?
               AND COALESCE(locator, display_name) = ?",
            rusqlite::params![scope, kind_json, identity],
            |r| r.get(0),
        ) {
            Ok(v) => Some(v),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(ControlPlaneError::Query(e.to_string())),
        };

        match row {
            Some(s) => {
                let uuid =
                    Uuid::parse_str(&s).map_err(|e| ControlPlaneError::Query(e.to_string()))?;
                Ok(Some(SourceId::from_uuid(uuid)))
            }
            None => Ok(None),
        }
    }

    fn latest_source_version(
        &self,
        source_id: SourceId,
    ) -> Result<Option<(SourceVersionId, String)>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let row: Option<(String, String)> = match conn.query_row(
            "SELECT version_id, fingerprint FROM source_version_projection
             WHERE source_id = ?
             ORDER BY recorded_at DESC, version_id DESC
             LIMIT 1",
            [source_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ) {
            Ok(v) => Some(v),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(ControlPlaneError::Query(e.to_string())),
        };

        match row {
            Some((vid, fp)) => {
                let uuid =
                    Uuid::parse_str(&vid).map_err(|e| ControlPlaneError::Query(e.to_string()))?;
                Ok(Some((SourceVersionId::from_uuid(uuid), fp)))
            }
            None => Ok(None),
        }
    }

    fn conclusions_depending_on_source(&self, source_id: SourceId) -> Result<Vec<ConclusionId>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT k.parent_id
                 FROM knowledge_dependency_projection k
                 JOIN evidence_projection e ON k.evidence_id = e.evidence_id
                 WHERE k.parent_type = 'Conclusion' AND e.source_id = ?
                 ORDER BY k.parent_id",
            )
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let rows = stmt
            .query_map([source_id.to_string()], |r| r.get::<_, String>(0))
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;

        let mut out = Vec::new();
        for row in rows {
            let s = row.map_err(|e| ControlPlaneError::Query(e.to_string()))?;
            let uuid = Uuid::parse_str(&s).map_err(|e| ControlPlaneError::Query(e.to_string()))?;
            out.push(ConclusionId::from_uuid(uuid));
        }
        Ok(out)
    }

    fn decisions_depending_on_source(&self, source_id: SourceId) -> Result<Vec<DecisionId>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT k.parent_id
                 FROM knowledge_dependency_projection k
                 JOIN evidence_projection e ON k.evidence_id = e.evidence_id
                 WHERE k.parent_type = 'Decision' AND e.source_id = ?
                 ORDER BY k.parent_id",
            )
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let rows = stmt
            .query_map([source_id.to_string()], |r| r.get::<_, String>(0))
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;

        let mut out = Vec::new();
        for row in rows {
            let s = row.map_err(|e| ControlPlaneError::Query(e.to_string()))?;
            let uuid = Uuid::parse_str(&s).map_err(|e| ControlPlaneError::Query(e.to_string()))?;
            out.push(DecisionId::from_uuid(uuid));
        }
        Ok(out)
    }

    fn is_conclusion_stale(&self, conclusion_id: ConclusionId) -> Result<bool> {
        Ok(self.latest_stale_fact(conclusion_id)?.is_some())
    }

    fn latest_stale_fact(&self, conclusion_id: ConclusionId) -> Result<Option<StaleFact>> {
        // Event-log authority: last MarkedStale vs Activated for this conclusion.
        // Reads are ordered by (occurred_at, event_id) for deterministic replay.
        let events = EventStore::read_all_events(&self.store)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut fact: Option<StaleFact> = None;
        for env in events {
            match &env.payload {
                Payload::ConclusionMarkedStale(p) if p.conclusion_id == conclusion_id => {
                    fact = Some(StaleFact {
                        changed_source_version_id: p.changed_source_version_id,
                        unavailable_reason: p.unavailable_reason.clone(),
                        source_id: p.source_id,
                    });
                }
                Payload::ConclusionActivated(p) if p.conclusion_id == conclusion_id => {
                    fact = None;
                }
                _ => {}
            }
        }
        Ok(fact)
    }

    fn source_version_count(&self, source_id: SourceId) -> Result<u64> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM source_version_projection WHERE source_id = ?",
                [source_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        Ok(count as u64)
    }

    fn evidence_count_for_source(&self, source_id: SourceId) -> Result<u64> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM evidence_projection WHERE source_id = ?",
                [source_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        Ok(count as u64)
    }
}

/// System clock adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Result<OffsetDateTime> {
        Ok(ai_brains_core::clock::now())
    }
}

/// SHA-256 fingerprinter port adapter (`ai-brains-sources`).
#[derive(Debug, Default, Clone, Copy)]
pub struct Sha256FingerprinterPort {
    inner: Sha256Fingerprinter,
}

impl Sha256FingerprinterPort {
    pub fn new() -> Self {
        Self {
            inner: Sha256Fingerprinter::new(),
        }
    }
}

impl Fingerprinter for Sha256FingerprinterPort {
    fn fingerprint(&self, content: &[u8]) -> Result<String> {
        Ok(self.inner.fingerprint(content))
    }
}

/// Always-allow policy (tests / open vaults).
#[derive(Debug, Default, Clone, Copy)]
pub struct AllowAllPolicy;

impl PolicyEvaluator for AllowAllPolicy {
    fn allow(
        &self,
        _principal: PrincipalId,
        _capability: GrantCapability,
        _scope: &ScopeRef,
    ) -> Result<bool> {
        Ok(true)
    }
}

/// Always-deny policy (policy deny path tests).
#[derive(Debug, Default, Clone, Copy)]
pub struct DenyAllPolicy;

impl PolicyEvaluator for DenyAllPolicy {
    fn allow(
        &self,
        _principal: PrincipalId,
        _capability: GrantCapability,
        _scope: &ScopeRef,
    ) -> Result<bool> {
        Ok(false)
    }
}

/// Shared store handle: writer + query over the same vault connection.
pub struct StorePorts {
    pub writer: StoreEventWriter,
    pub query: StoreGovernedQuery,
}

impl StorePorts {
    pub fn from_store(store: SqliteEventStore) -> Self {
        // VaultConnection is Arc-backed and Clone; both sides share one vault.
        let query_store = SqliteEventStore::new(store.connection().clone());
        Self {
            writer: StoreEventWriter::new(store),
            query: StoreGovernedQuery::new(query_store),
        }
    }
}
