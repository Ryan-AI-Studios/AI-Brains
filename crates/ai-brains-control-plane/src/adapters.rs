//! Concrete adapters binding control-plane ports to `ai-brains-store` / `ai-brains-sources`.

use ai_brains_core::ids::{
    ConclusionId, ConflictId, DecisionId, PrincipalId, ReviewItemId, SourceId, SourceVersionId,
};
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
    ClaimConflictRow, Clock, ConclusionRow, DecisionRow, EventWriter, Fingerprinter,
    GovernedQueryStore, PolicyEvaluator, ReviewItemRow, StaleFact,
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

fn map_conclusion_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConclusionRow> {
    let id_s: String = row.get(0)?;
    let valid_from_s: String = row.get(6)?;
    let valid_until_s: Option<String> = row.get(7)?;
    let recorded_at_s: String = row.get(8)?;
    let updated_at_s: String = row.get(9)?;
    let unsupported_i: i64 = row.get(13)?;
    Ok(ConclusionRow {
        id: ConclusionId::from_uuid(
            Uuid::parse_str(&id_s)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        ),
        state: row.get(1)?,
        statement: row.get(2)?,
        scope: row.get(3)?,
        privacy: row.get(4)?,
        proposer: row.get(5)?,
        valid_from: OffsetDateTime::parse(
            &valid_from_s,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        valid_until: match valid_until_s {
            Some(s) => Some(
                OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            ),
            None => None,
        },
        recorded_at: OffsetDateTime::parse(
            &recorded_at_s,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        updated_at: OffsetDateTime::parse(
            &updated_at_s,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        supersedes: row.get(10)?,
        superseded_by: row.get(11)?,
        protected_category: row.get(12)?,
        unsupported: unsupported_i != 0,
    })
}

const CONCLUSION_SELECT: &str = "SELECT conclusion_id, state, statement, scope, privacy, proposer,
    valid_from, valid_until, recorded_at, updated_at, supersedes, superseded_by,
    protected_category, unsupported
 FROM conclusion_projection";

fn map_decision_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DecisionRow> {
    let id_s: String = row.get(0)?;
    let valid_from_s: Option<String> = row.get(8)?;
    let valid_until_s: Option<String> = row.get(9)?;
    let recorded_at_s: String = row.get(10)?;
    let updated_at_s: String = row.get(11)?;
    Ok(DecisionRow {
        id: DecisionId::from_uuid(
            Uuid::parse_str(&id_s)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        ),
        state: row.get(1)?,
        title: row.get(2)?,
        statement: row.get(3)?,
        scope: row.get(4)?,
        proposer: row.get(5)?,
        approver: row.get(6)?,
        proposal_event_id: row.get(7)?,
        valid_from: match valid_from_s {
            Some(s) => Some(
                OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            ),
            None => None,
        },
        valid_until: match valid_until_s {
            Some(s) => Some(
                OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            ),
            None => None,
        },
        recorded_at: OffsetDateTime::parse(
            &recorded_at_s,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        updated_at: OffsetDateTime::parse(
            &updated_at_s,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        superseded_by: row.get(12)?,
    })
}

const DECISION_SELECT: &str = "SELECT decision_id, state, title, statement, scope, proposer,
    approver, proposal_event_id, valid_from, valid_until, recorded_at, updated_at, superseded_by
 FROM decision_projection";

fn map_review_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReviewItemRow> {
    let id_s: String = row.get(0)?;
    let recorded_at_s: String = row.get(12)?;
    let updated_at_s: String = row.get(13)?;
    Ok(ReviewItemRow {
        id: ReviewItemId::from_uuid(
            Uuid::parse_str(&id_s)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        ),
        subject_kind: row.get(1)?,
        subject_id: row.get(2)?,
        criticality: row.get(3)?,
        status: row.get(4)?,
        opened_by: row.get(5)?,
        subject: row.get(6)?,
        resolution: row.get(7)?,
        resolved_by: row.get(8)?,
        related_conclusion_id: row.get(9)?,
        related_decision_id: row.get(10)?,
        related_source_id: row.get(11)?,
        recorded_at: OffsetDateTime::parse(
            &recorded_at_s,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        updated_at: OffsetDateTime::parse(
            &updated_at_s,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
    })
}

fn map_conflict_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClaimConflictRow> {
    let id_s: String = row.get(0)?;
    let valid_from_s: Option<String> = row.get(7)?;
    let valid_until_s: Option<String> = row.get(8)?;
    let recorded_at_s: String = row.get(11)?;
    let updated_at_s: String = row.get(12)?;
    Ok(ClaimConflictRow {
        id: ConflictId::from_uuid(
            Uuid::parse_str(&id_s)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        ),
        claim_a_kind: row.get(1)?,
        claim_a_id: row.get(2)?,
        claim_b_kind: row.get(3)?,
        claim_b_id: row.get(4)?,
        status: row.get(5)?,
        scope: row.get(6)?,
        valid_from: match valid_from_s {
            Some(s) => Some(
                OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            ),
            None => None,
        },
        valid_until: match valid_until_s {
            Some(s) => Some(
                OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            ),
            None => None,
        },
        explanation: row.get(9)?,
        resolution: row.get(10)?,
        recorded_at: OffsetDateTime::parse(
            &recorded_at_s,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        updated_at: OffsetDateTime::parse(
            &updated_at_s,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
    })
}

impl GovernedQueryStore for StoreGovernedQuery {
    fn has_conclusion(&self, conclusion_id: ConclusionId) -> Result<bool> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        // Prefer epistemic projection; fall back to dependency edges for pre-projection facts.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM conclusion_projection WHERE conclusion_id = ?",
                [conclusion_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        if count > 0 {
            return Ok(true);
        }
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
                "SELECT COUNT(*) FROM decision_projection WHERE decision_id = ?",
                [decision_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        if count > 0 {
            return Ok(true);
        }
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

    fn get_conclusion(&self, conclusion_id: ConclusionId) -> Result<Option<ConclusionRow>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let sql = format!("{CONCLUSION_SELECT} WHERE conclusion_id = ?");
        match conn.query_row(&sql, [conclusion_id.to_string()], map_conclusion_row) {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ControlPlaneError::Query(e.to_string())),
        }
    }

    fn list_conclusions_by_scope_state(
        &self,
        scope: Option<&str>,
        state: Option<&str>,
    ) -> Result<Vec<ConclusionRow>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut sql = format!("{CONCLUSION_SELECT} WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        if let Some(s) = scope {
            sql.push_str(" AND scope = ?");
            params.push(Box::new(s.to_string()));
        }
        if let Some(st) = state {
            sql.push_str(" AND state = ?");
            params.push(Box::new(st.to_string()));
        }
        sql.push_str(" ORDER BY valid_from ASC, conclusion_id ASC");
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let rows = stmt
            .query_map(param_refs.as_slice(), map_conclusion_row)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| ControlPlaneError::Query(e.to_string()))?);
        }
        Ok(out)
    }

    fn get_decision(&self, decision_id: DecisionId) -> Result<Option<DecisionRow>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let sql = format!("{DECISION_SELECT} WHERE decision_id = ?");
        match conn.query_row(&sql, [decision_id.to_string()], map_decision_row) {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ControlPlaneError::Query(e.to_string())),
        }
    }

    fn list_decisions(&self, scope: Option<&str>, state: Option<&str>) -> Result<Vec<DecisionRow>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut sql = format!("{DECISION_SELECT} WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        if let Some(s) = scope {
            sql.push_str(" AND scope = ?");
            params.push(Box::new(s.to_string()));
        }
        if let Some(st) = state {
            sql.push_str(" AND state = ?");
            params.push(Box::new(st.to_string()));
        }
        sql.push_str(" ORDER BY recorded_at ASC, decision_id ASC");
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let rows = stmt
            .query_map(param_refs.as_slice(), map_decision_row)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| ControlPlaneError::Query(e.to_string()))?);
        }
        Ok(out)
    }

    fn list_open_review_items(&self) -> Result<Vec<ReviewItemRow>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT review_item_id, subject_kind, subject_id, criticality, status,
                        opened_by, subject, resolution, resolved_by,
                        related_conclusion_id, related_decision_id, related_source_id,
                        recorded_at, updated_at
                 FROM review_item_projection
                 WHERE status = 'Open'
                 ORDER BY recorded_at ASC, review_item_id ASC",
            )
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let rows = stmt
            .query_map([], map_review_row)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| ControlPlaneError::Query(e.to_string()))?);
        }
        Ok(out)
    }

    fn get_review_item(&self, review_item_id: ReviewItemId) -> Result<Option<ReviewItemRow>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        match conn.query_row(
            "SELECT review_item_id, subject_kind, subject_id, criticality, status,
                    opened_by, subject, resolution, resolved_by,
                    related_conclusion_id, related_decision_id, related_source_id,
                    recorded_at, updated_at
             FROM review_item_projection WHERE review_item_id = ?",
            [review_item_id.to_string()],
            map_review_row,
        ) {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ControlPlaneError::Query(e.to_string())),
        }
    }

    fn list_open_claim_conflicts(&self) -> Result<Vec<ClaimConflictRow>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT conflict_id, claim_a_kind, claim_a_id, claim_b_kind, claim_b_id,
                        status, scope, valid_from, valid_until, explanation, resolution,
                        recorded_at, updated_at
                 FROM claim_conflict_projection
                 WHERE status = 'Open'
                 ORDER BY recorded_at ASC, conflict_id ASC",
            )
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let rows = stmt
            .query_map([], map_conflict_row)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| ControlPlaneError::Query(e.to_string()))?);
        }
        Ok(out)
    }

    fn get_claim_conflict(&self, conflict_id: ConflictId) -> Result<Option<ClaimConflictRow>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        match conn.query_row(
            "SELECT conflict_id, claim_a_kind, claim_a_id, claim_b_kind, claim_b_id,
                    status, scope, valid_from, valid_until, explanation, resolution,
                    recorded_at, updated_at
             FROM claim_conflict_projection WHERE conflict_id = ?",
            [conflict_id.to_string()],
            map_conflict_row,
        ) {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ControlPlaneError::Query(e.to_string())),
        }
    }

    fn conclusions_valid_at(
        &self,
        scope: &str,
        statement: Option<&str>,
        at: OffsetDateTime,
    ) -> Result<Vec<ConclusionRow>> {
        let at_s = at
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        // valid_from <= at AND (valid_until IS NULL OR valid_until > at)
        // Does NOT use recorded_at / occurred_at (bitemporal: domain valid time).
        let mut sql = format!(
            "{CONCLUSION_SELECT}
             WHERE scope = ?
               AND valid_from <= ?
               AND (valid_until IS NULL OR valid_until > ?)
               AND state NOT IN ('Superseded', 'Rejected')"
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        params.push(Box::new(scope.to_string()));
        params.push(Box::new(at_s.clone()));
        params.push(Box::new(at_s));
        if let Some(stmt_text) = statement {
            sql.push_str(" AND statement = ?");
            params.push(Box::new(stmt_text.to_string()));
        }
        sql.push_str(" ORDER BY valid_from ASC, conclusion_id ASC");
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let rows = stmt
            .query_map(param_refs.as_slice(), map_conclusion_row)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| ControlPlaneError::Query(e.to_string()))?);
        }
        Ok(out)
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
