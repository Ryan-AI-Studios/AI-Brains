//! Repository identity + path-alias projections (T151 / T254).
//!
//! Materialized from [`RepositoryIdentityRegistered`], [`RepositoryPathAliasAdded`],
//! and owner-scoped [`RepositoryPathAliasRemoved`] so `rebuild_projections`
//! rehydrates durable scope-resolution keys. Other-owner Added does not steal.

use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct RepositoryIdentityProjection;

impl Projection for RepositoryIdentityProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::RepositoryIdentityRegistered(p) => {
                // Force rebind: clear other projects holding this remote hash first.
                if p.force
                    && let Some(hash) = p.remote_url_hash.as_ref()
                    && !hash.is_empty()
                {
                    tx.execute(
                        "UPDATE repository_identity_projection
                         SET remote_url_hash = NULL
                         WHERE remote_url_hash = ? AND project_id != ?",
                        rusqlite::params![hash, p.project_id.to_string()],
                    )?;
                }

                // Upsert identity row; preserve existing hash/ledgerful when omitted.
                tx.execute(
                    "INSERT INTO repository_identity_projection (
                        project_id, remote_url_hash, ledgerful_project_id, last_verified_at
                     ) VALUES (?, ?, ?, ?)
                     ON CONFLICT(project_id) DO UPDATE SET
                        remote_url_hash = COALESCE(excluded.remote_url_hash, repository_identity_projection.remote_url_hash),
                        ledgerful_project_id = COALESCE(excluded.ledgerful_project_id, repository_identity_projection.ledgerful_project_id),
                        last_verified_at = excluded.last_verified_at",
                    rusqlite::params![
                        p.project_id.to_string(),
                        p.remote_url_hash,
                        p.ledgerful_project_id,
                        occurred_at,
                    ],
                )?;
            }
            Payload::RepositoryPathAliasAdded(p) => {
                tx.execute(
                    "INSERT INTO repository_path_alias_projection (normalized_path, project_id)
                     VALUES (?, ?)
                     ON CONFLICT(normalized_path) DO UPDATE SET project_id = excluded.project_id
                     WHERE repository_path_alias_projection.project_id = excluded.project_id",
                    rusqlite::params![p.normalized_path, p.project_id.to_string()],
                )?;
            }
            Payload::RepositoryPathAliasRemoved(p) => {
                tx.execute(
                    "DELETE FROM repository_path_alias_projection
                     WHERE normalized_path = ? AND project_id = ?",
                    rusqlite::params![p.normalized_path, p.project_id.to_string()],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
