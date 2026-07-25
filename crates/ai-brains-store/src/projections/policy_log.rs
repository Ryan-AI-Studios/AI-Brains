//! Policy decision log (T151).
//!
//! Materializes [`Payload::PolicyDecisionRecorded`] into `policy_decision_log`
//! so `rebuild_projections` rehydrates audit rows from the event log.
//! Rows hold reason codes only — never claim/statement text.

use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct PolicyLogProjection;

impl Projection for PolicyLogProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let Payload::PolicyDecisionRecorded(p) = &envelope.payload else {
            return Ok(());
        };

        let recorded_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
        let capability = capability_label(&p.capability)?;
        let privacy = p.privacy.map(privacy_label);

        tx.execute(
            "INSERT INTO policy_decision_log (
                principal_id, capability, scope_key, allowed, reason_code, privacy, recorded_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                p.principal_id.to_string(),
                capability,
                p.scope_key,
                if p.allowed { 1i64 } else { 0i64 },
                p.reason_code,
                privacy,
                recorded_at,
            ],
        )?;
        Ok(())
    }
}

fn capability_label(capability: &ai_brains_core::scope::GrantCapability) -> Result<String> {
    let json = serde_json::to_string(capability)
        .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
    Ok(json.trim_matches('"').to_string())
}

fn privacy_label(privacy: Privacy) -> String {
    match privacy {
        Privacy::CloudOk => "CloudOk".to_string(),
        Privacy::LocalOnly => "LocalOnly".to_string(),
        Privacy::NeverInject => "NeverInject".to_string(),
        Privacy::Sealed => "Sealed".to_string(),
    }
}
