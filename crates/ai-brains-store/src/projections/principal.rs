//! Principal registry projection (T151).
//!
//! # Kind wire format
//!
//! `kind` is stored as `TEXT` using the PascalCase label of
//! [`ai_brains_core::principal::PrincipalKind`]:
//! `Human`, `Agent`, `Connector`, `System`, `Service`, or `Other:{label}`.
//! Control-plane `parse_principal_kind` reverses this for domain reads.
//! Unknown historical strings map to `Other(raw)` so legacy rows remain loadable.

use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct PrincipalProjection;

impl Projection for PrincipalProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        if let Payload::PrincipalRegistered(p) = &envelope.payload {
            let bound_source_kinds = serde_json::to_string(&p.bound_source_kinds)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
            let bound_capabilities = serde_json::to_string(&p.bound_capabilities)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
            tx.execute(
                "INSERT INTO principal_projection (
                    principal_id, kind, display_name,
                    bound_source_kinds, bound_capabilities,
                    recorded_at, updated_at
                 ) VALUES (?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(principal_id) DO UPDATE SET
                    kind = excluded.kind,
                    display_name = excluded.display_name,
                    bound_source_kinds = excluded.bound_source_kinds,
                    bound_capabilities = excluded.bound_capabilities,
                    updated_at = excluded.updated_at",
                rusqlite::params![
                    p.principal_id.to_string(),
                    p.kind,
                    p.display_name,
                    bound_source_kinds,
                    bound_capabilities,
                    occurred_at,
                    occurred_at,
                ],
            )?;
        }
        Ok(())
    }
}
