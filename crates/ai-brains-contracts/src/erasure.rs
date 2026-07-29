//! Erasure wire surface (T158 ticket + T165 cryptographic erase).
//!
//! Dual-path honesty:
//! - [`RequestErasureRequest`] / [`ErasureAcceptedResponse`] — ticket only (not CE).
//! - [`WipeContentEnvelopeRequest`] / [`ContentEnvelopeWipedResponse`] — CE for
//!   envelope-backed content only (content_key_store row required).

use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1";

/// Ticket-path honesty: wipe is a separate command (`erasure wipe`), not ticket accept.
pub const ERASURE_TICKET_NO_WIPE_WARNING: &str = "content-envelope wipe not performed; ticket accepted only (use erasure wipe for envelope-backed CE)";

/// Success-path honesty bullets for CE wipe (E10). Always include on wipe success.
pub const WIPE_HONESTY_NOT_NIST_PURGE: &str =
    "not NIST Purge/Destroy; not physical media sanitization (TRUNCATE is not Purge)";
pub const WIPE_HONESTY_PRE_ERASE_BACKUP: &str =
    "pre-erase backups, exports, and offline copies remain decryptable if restored";
pub const WIPE_HONESTY_TICKET_NOT_CE: &str =
    "erasure ticket and soft forget are not cryptographic erasure";
pub const WIPE_HONESTY_ENVELOPE_ONLY: &str =
    "cryptographic erasure applies only to envelope-backed content (content_key_store)";
pub const WIPE_HONESTY_SQLCIPHER_NOT_ITEM_CE: &str =
    "SQLCipher vault lock is not per-item cryptographic erasure";

/// Dependents skipped when no blob subject maps to a registered SourceId (E15).
pub const WIPE_WARNING_DEPENDENTS_SKIPPED: &str = "dependents_skipped_no_source_link";

/// WAL TRUNCATE was BUSY after retry; wipe still success if wrap destroyed (E16).
pub const WIPE_WARNING_WAL_PENDING_PASSIVE: &str = "wal_checkpoint_status: pending_passive";

fn default_api_version() -> String {
    API_VERSION.to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

/// Request erasure of governed records (ids + reason only — no crypto claims).
///
/// Handlers accept the request into a ticket queue; content-envelope wipe is a
/// separate command ([`WipeContentEnvelopeRequest`]).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestErasureRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// Target record / aggregate ids.
    #[serde(default)]
    pub ids: Vec<String>,
    /// Human-readable reason (no secrets).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Client command / idempotency key. When set, daemon spools and derives
    /// a deterministic ticket `request_id` (uuid v5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
}

/// Acknowledgement that an erasure request was accepted (not that wipe completed).
///
/// **E1:** `warnings: []` when none; `status` is a queue/accept state, not a crypto proof.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErasureAcceptedResponse {
    pub api_version: String,
    /// Request / ticket id for tracking.
    pub request_id: String,
    /// e.g. `accepted`, `queued`
    pub status: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl ErasureAcceptedResponse {
    pub fn new(request_id: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            request_id: request_id.into(),
            status: status.into(),
            warnings: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// T165 — Governed content-envelope wipe (cryptographic erasure)
// ---------------------------------------------------------------------------

/// Governed CE wipe for envelope-backed content only (E1).
///
/// **Execute semantics (E9):** destructive work runs only when
/// `dry_run == false` **and** `confirm == true`. Default is dry-run safe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WipeContentEnvelopeRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// Content key to cryptographically erase (required UUID string).
    pub content_key_id: String,
    /// Scope identity key (required; policy GrantCapability::Erase).
    pub scope: String,
    /// Optional ops reason (no secrets).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Client command / idempotency key. When set, daemon spools and derives
    /// deterministic tombstone_id (uuid v5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
    /// Default **true** — no wrap destroy, no events, no purge (E9).
    #[serde(default = "default_true")]
    pub dry_run: bool,
    /// Must be **true** with `dry_run: false` to execute wipe (E9).
    #[serde(default = "default_false")]
    pub confirm: bool,
}

/// Counts of derived plaintext purged for subjects under the content key (E13).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WipePurgedCounts {
    #[serde(default)]
    pub fts_rows: u64,
    #[serde(default)]
    pub embeddings: u64,
    #[serde(default)]
    pub projection_rows: u64,
}

/// Verification layer (E14): store re-query only — not a fake AEAD open_fails.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WipeVerify {
    /// Fresh re-query: wrap status destroyed / wrap material absent.
    pub wrap_absent: bool,
}

/// Validation layer (E14): derived indexes + store open path + WAL (not crypto proof).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WipeValidation {
    /// Fixture / subject plaintext has 0 FTS hits after purge (execute path).
    pub fts_clear: bool,
    /// Store cannot supply wrap material for open (not GCM tag failure).
    pub store_open_refused: bool,
    /// `"truncated"` | `"pending_passive"` | `"skipped_dry_run"` | `"skipped_already_erased"`.
    pub wal_checkpoint: String,
}

/// Response for governed CE wipe.
///
/// **E1 empty-state:** missing / non-envelope key is a structured error
/// (`NOT_ENVELOPE_BACKED`), not an empty success.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentEnvelopeWipedResponse {
    pub api_version: String,
    /// `dry_run` | `wiped` | `already_erased`
    pub status: String,
    pub content_key_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tombstone_id: Option<String>,
    pub wrap_destroyed: bool,
    /// Count of `encrypted_content_blob` rows for this key (E13).
    pub blobs_considered: u64,
    pub purged: WipePurgedCounts,
    pub dependents_marked: u64,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub verify: WipeVerify,
    pub validation: WipeValidation,
}

impl ContentEnvelopeWipedResponse {
    /// Default honesty warnings required on success paths (E10).
    pub fn honesty_warnings() -> Vec<String> {
        vec![
            WIPE_HONESTY_NOT_NIST_PURGE.to_string(),
            WIPE_HONESTY_PRE_ERASE_BACKUP.to_string(),
            WIPE_HONESTY_TICKET_NOT_CE.to_string(),
            WIPE_HONESTY_ENVELOPE_ONLY.to_string(),
            WIPE_HONESTY_SQLCIPHER_NOT_ITEM_CE.to_string(),
        ]
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn request_erasure_request__roundtrip() {
        let req = RequestErasureRequest {
            api_version: API_VERSION.to_string(),
            principal_id: Some("p1".into()),
            ids: vec!["agg-1".into()],
            reason: Some("user request".into()),
            scope: Some("Personal:00000000-0000-0000-0000-0000000000u1".into()),
            command_id: Some("erase-cmd-1".into()),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: RequestErasureRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, req);
    }

    #[test]
    fn request_erasure_request__command_id_optional() {
        let decoded: RequestErasureRequest =
            serde_json::from_str(r#"{"api_version":"1","ids":["a"]}"#).expect("deserialize");
        assert!(decoded.command_id.is_none());
    }

    #[test]
    fn wipe_content_envelope_request__defaults_dry_run_true_confirm_false() {
        let decoded: WipeContentEnvelopeRequest = serde_json::from_str(
            r#"{"content_key_id":"00000000-0000-0000-0000-000000000001","scope":"Personal:u"}"#,
        )
        .expect("deserialize");
        assert!(decoded.dry_run);
        assert!(!decoded.confirm);
        assert_eq!(decoded.api_version, API_VERSION);
    }

    #[test]
    fn wipe_content_envelope_request__roundtrip() {
        let req = WipeContentEnvelopeRequest {
            api_version: API_VERSION.to_string(),
            principal_id: Some("p1".into()),
            content_key_id: "00000000-0000-0000-0000-000000000001".into(),
            scope: "Personal:u".into(),
            reason: Some("ops".into()),
            command_id: Some("wipe-1".into()),
            dry_run: false,
            confirm: true,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: WipeContentEnvelopeRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, req);
    }

    #[test]
    fn contracts__wipe_response__e1_empty_and_warnings() {
        let mut resp = ContentEnvelopeWipedResponse {
            api_version: API_VERSION.to_string(),
            status: "wiped".into(),
            content_key_id: "00000000-0000-0000-0000-000000000001".into(),
            tombstone_id: Some("00000000-0000-0000-0000-000000000002".into()),
            wrap_destroyed: true,
            blobs_considered: 2,
            purged: WipePurgedCounts {
                fts_rows: 1,
                embeddings: 1,
                projection_rows: 0,
            },
            dependents_marked: 0,
            warnings: ContentEnvelopeWipedResponse::honesty_warnings(),
            verify: WipeVerify { wrap_absent: true },
            validation: WipeValidation {
                fts_clear: true,
                store_open_refused: true,
                wal_checkpoint: "truncated".into(),
            },
        };
        resp.warnings
            .push(WIPE_WARNING_DEPENDENTS_SKIPPED.to_string());

        let json = serde_json::to_string(&resp).expect("serialize");
        let decoded: ContentEnvelopeWipedResponse =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, resp);

        let joined = decoded.warnings.join(" ");
        assert!(
            joined.to_ascii_lowercase().contains("purge")
                || joined.to_ascii_lowercase().contains("nist"),
            "must include no-Purge honesty: {joined}"
        );
        assert!(
            joined.to_ascii_lowercase().contains("backup")
                || joined.to_ascii_lowercase().contains("offline"),
            "must include pre-erase backup honesty: {joined}"
        );
        assert!(
            joined.to_ascii_lowercase().contains("sqlcipher")
                || joined.to_ascii_lowercase().contains("vault lock"),
            "must include SQLCipher vault-lock honesty: {joined}"
        );
        // E14: no fake open_fails dual crypto signal on wire.
        assert!(
            !json.contains("open_fails"),
            "must not carry fake open_fails field: {json}"
        );
        assert!(decoded.verify.wrap_absent);
    }

    #[test]
    fn contracts__ticket_response__still_has_wipe_warning_constant() {
        assert!(
            ERASURE_TICKET_NO_WIPE_WARNING
                .to_ascii_lowercase()
                .contains("wipe")
        );
        assert!(
            !ERASURE_TICKET_NO_WIPE_WARNING
                .to_ascii_lowercase()
                .contains("wipe completed")
        );
        let mut resp = ErasureAcceptedResponse::new("ticket-1", "accepted");
        resp.warnings
            .push(ERASURE_TICKET_NO_WIPE_WARNING.to_string());
        assert!(!resp.warnings.is_empty());
    }
}
