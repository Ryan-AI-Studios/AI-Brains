//! Scope grant projection (T151).
//!
//! Privacy is taken from [`ScopeGrantIssuedPayload::privacy`] (serde-default
//! [`Privacy::LocalOnly`] for historical events without the field).
//!
//! Scope identity key format matches control-plane `scope_identity_key`:
//! `Repository:{id}` / `Workspace:{id}` / `Personal:{id}`.

use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct GrantProjection;

impl Projection for GrantProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::ScopeGrantIssued(p) => {
                let scope_key = scope_identity_key(&p.scope);
                let capability = capability_label(&p.capability)?;
                let privacy = privacy_label(p.privacy);
                tx.execute(
                    "INSERT INTO scope_grant_projection (
                        grant_id, principal_id, scope_key, capability, privacy,
                        issued_at, revoked_at
                     ) VALUES (?, ?, ?, ?, ?, ?, NULL)
                     ON CONFLICT(grant_id) DO UPDATE SET
                        principal_id = excluded.principal_id,
                        scope_key = excluded.scope_key,
                        capability = excluded.capability,
                        privacy = excluded.privacy,
                        issued_at = excluded.issued_at,
                        revoked_at = NULL",
                    rusqlite::params![
                        p.grant_id.to_string(),
                        p.principal_id.to_string(),
                        scope_key,
                        capability,
                        privacy,
                        occurred_at,
                    ],
                )?;
            }
            Payload::ScopeGrantRevoked(p) => {
                tx.execute(
                    "UPDATE scope_grant_projection
                     SET revoked_at = ?
                     WHERE grant_id = ?",
                    rusqlite::params![occurred_at, p.grant_id.to_string()],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Stable scope identity string (same format as control-plane `scope_identity_key`).
pub fn scope_identity_key(scope: &ScopeRef) -> String {
    match scope {
        ScopeRef::Repository(id) => format!("Repository:{id}"),
        ScopeRef::Workspace(id) => format!("Workspace:{id}"),
        ScopeRef::Personal(id) => format!("Personal:{id}"),
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
