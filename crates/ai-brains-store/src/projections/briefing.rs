//! Briefing cache + query trace projections (T152).

use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct BriefingProjection;

impl Projection for BriefingProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::BriefingGenerated(p) => {
                // Cache row is optional; BriefingGenerated only carries ids.
                // Insert a lightweight placeholder keyed by briefing_id so rebuild
                // rehydrates that a briefing was generated. Full packet_json is
                // written by the briefing service cache path when available.
                let cache_key = format!("event:{}", p.briefing_id);
                let evidence_ids: Vec<String> =
                    p.evidence_ids.iter().map(|e| e.to_string()).collect();
                let mut map = serde_json::Map::new();
                map.insert(
                    "briefing_id".into(),
                    serde_json::Value::String(p.briefing_id.to_string()),
                );
                map.insert("kind".into(), serde_json::Value::String(p.kind.clone()));
                map.insert(
                    "evidence_ids".into(),
                    serde_json::Value::Array(
                        evidence_ids
                            .into_iter()
                            .map(serde_json::Value::String)
                            .collect(),
                    ),
                );
                map.insert(
                    "query_trace_id".into(),
                    match p.query_trace_id {
                        Some(id) => serde_json::Value::String(id.to_string()),
                        None => serde_json::Value::Null,
                    },
                );
                let packet_json = serde_json::to_string(&serde_json::Value::Object(map))
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
                tx.execute(
                    "INSERT INTO briefing_cache_projection (
                        cache_key, briefing_type, scope_key, policy_version,
                        source_version_vector, budget, packet_json, generated_at, expires
                     ) VALUES (?, ?, '', '', '', 0, ?, ?, NULL)
                     ON CONFLICT(cache_key) DO UPDATE SET
                        briefing_type = excluded.briefing_type,
                        packet_json = excluded.packet_json,
                        generated_at = excluded.generated_at",
                    rusqlite::params![cache_key, p.kind, packet_json, occurred_at],
                )?;
            }
            Payload::QueryTraceRecorded(p) => {
                let handles = serde_json::to_string(
                    &p.evidence_ids
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>(),
                )
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
                // Prefer ranking_json from the event; legacy events without the field
                // deserialize as empty and rehydrate as `{}`.
                let ranking = if p.ranking_json.trim().is_empty() {
                    "{}"
                } else {
                    p.ranking_json.as_str()
                };
                tx.execute(
                    "INSERT INTO query_trace_projection (
                        trace_id, scope, principal, query, applied_policy,
                        ranking_json, result_handles_json, freshness_summary,
                        conflict_summary, recorded_at
                     ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(trace_id) DO UPDATE SET
                        scope = excluded.scope,
                        principal = excluded.principal,
                        query = excluded.query,
                        applied_policy = excluded.applied_policy,
                        ranking_json = excluded.ranking_json,
                        result_handles_json = excluded.result_handles_json,
                        freshness_summary = excluded.freshness_summary,
                        conflict_summary = excluded.conflict_summary,
                        recorded_at = excluded.recorded_at",
                    rusqlite::params![
                        p.query_trace_id.to_string(),
                        p.scope,
                        p.principal_id,
                        p.query_text,
                        p.applied_policy,
                        ranking,
                        handles,
                        p.freshness_summary,
                        p.conflict_summary,
                        occurred_at,
                    ],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Build a stable cache key for briefing lookups.
///
/// Format: `{briefing_type}|{scope_key}|{policy_version}|{source_version_vector}|{budget}`
pub fn briefing_cache_key(
    briefing_type: &str,
    scope_key: &str,
    policy_version: &str,
    source_version_vector: &str,
    budget: u64,
) -> String {
    format!("{briefing_type}|{scope_key}|{policy_version}|{source_version_vector}|{budget}")
}
