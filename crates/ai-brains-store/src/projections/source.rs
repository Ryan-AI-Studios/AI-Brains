use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct SourceProjection;

impl Projection for SourceProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::SourceRegistered(p) => {
                let kind_json = serde_json::to_string(&p.kind)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
                // Scope identity from payload (empty when historical / unspecified).
                let scope = p.scope.as_deref().unwrap_or("");
                tx.execute(
                    "INSERT INTO source_projection (
                        source_id, scope, kind, display_name, locator, status,
                        recorded_at, updated_at
                     ) VALUES (?, ?, ?, ?, ?, 'Active', ?, ?)
                     ON CONFLICT(source_id) DO UPDATE SET
                        scope = excluded.scope,
                        kind = excluded.kind,
                        display_name = excluded.display_name,
                        locator = excluded.locator,
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.source_id.to_string(),
                        scope,
                        kind_json,
                        p.display_name,
                        p.locator,
                        occurred_at,
                        occurred_at,
                    ],
                )?;
            }
            Payload::SourceObserved(p) => {
                let observed_at = p
                    .observed_at
                    .format(&Rfc3339)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
                tx.execute(
                    "UPDATE source_projection
                     SET last_observed_at = ?, updated_at = ?
                     WHERE source_id = ?",
                    rusqlite::params![observed_at, occurred_at, p.source_id.to_string()],
                )?;
            }
            Payload::SourceVersionRecorded(p) => {
                let recorded_at = p
                    .recorded_at
                    .format(&Rfc3339)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
                let normalizer_version = parse_normalizer_version(&p.fingerprint);
                tx.execute(
                    "INSERT INTO source_version_projection (
                        version_id, source_id, fingerprint, normalizer_version, recorded_at
                     ) VALUES (?, ?, ?, ?, ?)
                     ON CONFLICT(version_id) DO UPDATE SET
                        fingerprint = excluded.fingerprint,
                        normalizer_version = excluded.normalizer_version,
                        recorded_at = excluded.recorded_at",
                    rusqlite::params![
                        p.version_id.to_string(),
                        p.source_id.to_string(),
                        p.fingerprint,
                        normalizer_version,
                        recorded_at,
                    ],
                )?;
                tx.execute(
                    "UPDATE source_projection
                     SET updated_at = ?
                     WHERE source_id = ?",
                    rusqlite::params![occurred_at, p.source_id.to_string()],
                )?;
            }
            Payload::SourceUnavailable(p) => {
                let marked_at = p
                    .marked_at
                    .format(&Rfc3339)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
                tx.execute(
                    "UPDATE source_projection
                     SET status = 'Unavailable', updated_at = ?
                     WHERE source_id = ?",
                    rusqlite::params![marked_at, p.source_id.to_string()],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Extract normalizer version from fingerprint format `v{n}:{hex}` (default 1).
fn parse_normalizer_version(fingerprint: &str) -> i64 {
    let Some(rest) = fingerprint.strip_prefix('v') else {
        return 1;
    };
    let Some((ver, _)) = rest.split_once(':') else {
        return 1;
    };
    ver.parse::<i64>().unwrap_or(1)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::parse_normalizer_version;

    #[test]
    fn parse_normalizer_version__v_prefix_hex__extracts_version() {
        assert_eq!(parse_normalizer_version("v3:deadbeef"), 3);
        assert_eq!(parse_normalizer_version("v1:abc"), 1);
    }

    #[test]
    fn parse_normalizer_version__malformed__defaults_to_one() {
        assert_eq!(parse_normalizer_version("not-a-version"), 1);
        assert_eq!(parse_normalizer_version("vonly"), 1);
        assert_eq!(parse_normalizer_version(""), 1);
    }
}
