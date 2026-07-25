//! Workspace and workspace↔repository membership projections (T151).

use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct WorkspaceProjection;

impl Projection for WorkspaceProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::WorkspaceRegistered(p) => {
                // Payload has no privacy field; default CloudOk ("ok" shared boundary).
                let privacy = privacy_label(Privacy::CloudOk);
                tx.execute(
                    "INSERT INTO workspace_projection (
                        workspace_id, name, privacy, recorded_at, updated_at
                     ) VALUES (?, ?, ?, ?, ?)
                     ON CONFLICT(workspace_id) DO UPDATE SET
                        name = excluded.name,
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.workspace_id.to_string(),
                        p.name,
                        privacy,
                        occurred_at,
                        occurred_at,
                    ],
                )?;
            }
            Payload::RepositoryJoinedWorkspace(p) => {
                // Ensure parent workspace row exists for FK (membership-only path).
                tx.execute(
                    "INSERT INTO workspace_projection (
                        workspace_id, name, privacy, recorded_at, updated_at
                     ) VALUES (?, '', ?, ?, ?)
                     ON CONFLICT(workspace_id) DO NOTHING",
                    rusqlite::params![
                        p.workspace_id.to_string(),
                        privacy_label(Privacy::CloudOk),
                        occurred_at,
                        occurred_at,
                    ],
                )?;
                tx.execute(
                    "INSERT INTO workspace_repository_projection (workspace_id, project_id)
                     VALUES (?, ?)
                     ON CONFLICT(workspace_id, project_id) DO NOTHING",
                    rusqlite::params![p.workspace_id.to_string(), p.project_id.to_string()],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}

fn privacy_label(privacy: Privacy) -> String {
    match privacy {
        Privacy::CloudOk => "CloudOk".to_string(),
        Privacy::LocalOnly => "LocalOnly".to_string(),
        Privacy::NeverInject => "NeverInject".to_string(),
        Privacy::Sealed => "Sealed".to_string(),
    }
}
