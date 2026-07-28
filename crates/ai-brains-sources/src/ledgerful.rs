//! Ledgerful bridge-record connector (`builtin.ledgerful`) — T155 / P6.3.
//!
//! # Design locks
//!
//! - **Source:** in-memory [`BridgeRecord`] set (required v1 mode).
//! - **Identity:**
//!   `{scope_key}|Ledgerful|{project_id}|{record_kind}|{stable_record_key}`
//!   where `stable_record_key` is `tx_id` if `Some`, else SHA-256 of canonical
//!   payload JSON bytes.
//! - **Content:** canonical JSON of the full [`BridgeRecord`] (privacy field
//!   preserved). Fingerprint via [`crate::fingerprint_ledgerful`] (authoritative
//!   `ledgerful:{hash}` when `parent_hash` / bridge hash fields are present).
//! - **Write-back:** [`Connector::propose_write`] is unsupported.
//! - **Sandbox / trust:** TrustedBuiltin / LocalOnly; credentials PathAccess.
//!
//! # Unavailability (anti-#22)
//!
//! | Situation | Behavior |
//! |-----------|----------|
//! | Healthy empty record set | `list` → `Ok([])`; [`Self::last_unavailable_reason`] is `None` |
//! | Missing / unreadable store | Use [`LedgerfulConnector::unavailable`]; `list` → `Ok([])`
//!   **with** reason set; `observe` → `Err` |
//!
//! Callers **must** check [`Self::last_unavailable_reason`] and
//! [`Self::last_list_truncated`] immediately after every `list`.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use sha2::{Digest, Sha256};

use crate::connector::{
    Connector, ConnectorContext, ConnectorError, ObservePayload, Preview, SourceHandle,
    WriteProposal, WriteProposalInput,
};
use crate::manifest::{
    ConnectorManifest, ConnectorOperations, ConnectorTrustLabel, CredentialDeclaration,
    FreshnessMechanism, MANIFEST_SCHEMA_VERSION, SandboxMode, ScopeClass,
};

/// Stable connector id.
pub const LEDGERFUL_CONNECTOR_ID: &str = "builtin.ledgerful";

/// Default cap on bridge-record handles returned by `list`.
pub const DEFAULT_LEDGERFUL_MAX_RECORDS: usize = 256;

/// Options for [`LedgerfulConnector`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerfulConnectorOptions {
    /// Max records / handles from `list` (default [`DEFAULT_LEDGERFUL_MAX_RECORDS`]).
    pub max_records: usize,
}

impl Default for LedgerfulConnectorOptions {
    fn default() -> Self {
        Self {
            max_records: DEFAULT_LEDGERFUL_MAX_RECORDS,
        }
    }
}

/// Where bridge records are loaded from (v1: in-memory only).
#[derive(Debug, Clone)]
pub enum LedgerfulSource {
    /// Configured in-memory records (may be empty and healthy).
    InMemory(Vec<BridgeRecord>),
}

/// Built-in Ledgerful connector.
///
/// Construct with [`LedgerfulConnector::from_records`] for a healthy store
/// (including empty), or [`LedgerfulConnector::unavailable`] when the store is
/// missing / unreadable.
pub struct LedgerfulConnector {
    source: LedgerfulSource,
    options: LedgerfulConnectorOptions,
    manifest: ConnectorManifest,
    /// True when constructed via [`Self::unavailable`].
    store_unavailable: bool,
    unavailable_reason_fixed: Option<String>,
    last_list_truncated: AtomicBool,
    last_unavailable_reason: Mutex<Option<String>>,
}

impl LedgerfulConnector {
    /// Healthy connector from an in-memory record set (may be empty).
    ///
    /// Empty configured sets clear [`Self::last_unavailable_reason`] on list —
    /// that is legitimate “no records”, not a missing store.
    pub fn from_records(records: Vec<BridgeRecord>, options: LedgerfulConnectorOptions) -> Self {
        Self::new_inner(LedgerfulSource::InMemory(records), options, None)
    }

    /// Explicit unavailable store (missing / unreadable).
    ///
    /// `list` returns empty with `last_unavailable_reason` set; `observe` errs.
    pub fn unavailable(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self::new_inner(
            LedgerfulSource::InMemory(Vec::new()),
            LedgerfulConnectorOptions::default(),
            Some(reason),
        )
    }

    /// Unavailable with custom options (still empty records).
    pub fn unavailable_with_options(
        reason: impl Into<String>,
        options: LedgerfulConnectorOptions,
    ) -> Self {
        Self::new_inner(
            LedgerfulSource::InMemory(Vec::new()),
            options,
            Some(reason.into()),
        )
    }

    fn new_inner(
        source: LedgerfulSource,
        options: LedgerfulConnectorOptions,
        unavailable: Option<String>,
    ) -> Self {
        let manifest = ConnectorManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: LEDGERFUL_CONNECTOR_ID.into(),
            display_name: "Ledgerful".into(),
            connector_version: "0.1.0".into(),
            source_kinds: vec![SourceKind::Ledgerful],
            operations: ConnectorOperations {
                list: true,
                observe: true,
                preview: true,
                propose_write: false,
            },
            scope_affinity: vec![
                ScopeClass::Personal,
                ScopeClass::Repository,
                ScopeClass::Workspace,
            ],
            freshness: FreshnessMechanism::Fingerprint,
            credentials: CredentialDeclaration::PathAccess,
            sandbox: SandboxMode::TrustedBuiltin,
            default_trust: ConnectorTrustLabel::LocalOnly,
            principal_id: None,
        };

        Self {
            source,
            options,
            manifest,
            store_unavailable: unavailable.is_some(),
            unavailable_reason_fixed: unavailable,
            last_list_truncated: AtomicBool::new(false),
            last_unavailable_reason: Mutex::new(None),
        }
    }

    /// Whether the last successful `list()` hit `max_records` and stopped early.
    pub fn last_list_truncated(&self) -> bool {
        self.last_list_truncated.load(Ordering::Relaxed)
    }

    /// Soft-empty / hard-failure reason from the last `list` (or observe failure).
    ///
    /// Callers **must** check this after every `list`.
    pub fn last_unavailable_reason(&self) -> Option<String> {
        // Side-channel is non-secret status only; recover the guard on poison
        // so a prior panic-in-lock does not hide the last reason.
        let g = self
            .last_unavailable_reason
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        g.clone()
    }

    /// Configured options.
    pub fn options(&self) -> &LedgerfulConnectorOptions {
        &self.options
    }

    /// True when constructed as an unavailable store.
    pub fn is_store_unavailable(&self) -> bool {
        self.store_unavailable
    }

    fn clear_side_channels(&self) {
        self.last_list_truncated.store(false, Ordering::Relaxed);
        let mut g = self
            .last_unavailable_reason
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        *g = None;
    }

    fn set_unavailable(&self, reason: impl Into<String>) {
        let mut g = self
            .last_unavailable_reason
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        *g = Some(reason.into());
    }

    fn set_truncated(&self, truncated: bool) {
        self.last_list_truncated.store(truncated, Ordering::Relaxed);
    }

    fn records(&self) -> &[BridgeRecord] {
        match &self.source {
            LedgerfulSource::InMemory(v) => v.as_slice(),
        }
    }

    fn handle_for(&self, scope: &ScopeRef, record: &BridgeRecord) -> SourceHandle {
        let stable_key = stable_record_key(record);
        let locator = record_locator(record);
        let identity =
            make_ledgerful_identity(scope, &record.project_id, &record.record_kind, &stable_key);
        SourceHandle {
            identity,
            kind: SourceKind::Ledgerful,
            locator,
        }
    }

    fn find_record(&self, handle: &SourceHandle) -> Result<&BridgeRecord, ConnectorError> {
        if self.store_unavailable {
            let reason = self
                .unavailable_reason_fixed
                .clone()
                .unwrap_or_else(|| "unavailable".into());
            self.set_unavailable(reason.clone());
            return Err(ConnectorError::Internal { detail: reason });
        }

        self.records()
            .iter()
            .find(|r| record_locator(r) == handle.locator)
            .ok_or_else(|| ConnectorError::HandleNotFound {
                locator: handle.locator.clone(),
            })
    }
}

/// Locator fragment (no scope): `{project_id}|{record_kind}|{stable_key}`.
pub fn record_locator(record: &BridgeRecord) -> String {
    let key = stable_record_key(record);
    format!("{}|{}|{}", record.project_id, record.record_kind, key)
}

/// Stable record key: `tx_id` if present, else SHA-256 of canonical payload bytes.
pub fn stable_record_key(record: &BridgeRecord) -> String {
    if let Some(tx) = record.tx_id.as_ref() {
        let t = tx.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    let payload_bytes = canonical_payload_bytes(record);
    let mut hasher = Sha256::new();
    hasher.update(&payload_bytes);
    hex::encode(hasher.finalize())
}

/// Serialize a bridge record to canonical JSON content bytes for observe.
///
/// Prefer serde_json serialization of the full record so `parent_hash` and
/// privacy are preserved for [`crate::fingerprint_ledgerful`].
pub fn serialize_bridge_record(record: &BridgeRecord) -> Result<Vec<u8>, ConnectorError> {
    let value = serde_json::to_value(record).map_err(|e| ConnectorError::Internal {
        detail: format!("bridge record serialize: {e}"),
    })?;
    let canonical = canonicalize_json_value(&value);
    serde_json::to_vec(&canonical).map_err(|e| ConnectorError::Internal {
        detail: format!("bridge record canonical serialize: {e}"),
    })
}

fn canonical_payload_bytes(record: &BridgeRecord) -> Vec<u8> {
    let value = serde_json::to_value(&record.payload).unwrap_or(serde_json::Value::Null);
    let canonical = canonicalize_json_value(&value);
    serde_json::to_vec(&canonical).unwrap_or_default()
}

fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = serde_json::Map::with_capacity(map.len());
            for k in keys {
                if let Some(v) = map.get(k) {
                    out.insert(k.clone(), canonicalize_json_value(v));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(canonicalize_json_value).collect())
        }
        other => other.clone(),
    }
}

fn scope_key(scope: &ScopeRef) -> String {
    match scope {
        ScopeRef::Personal(id) => format!("Personal:{id}"),
        ScopeRef::Repository(id) => format!("Repository:{id}"),
        ScopeRef::Workspace(id) => format!("Workspace:{id}"),
    }
}

fn make_ledgerful_identity(
    scope: &ScopeRef,
    project_id: &str,
    record_kind: &str,
    stable_key: &str,
) -> String {
    format!(
        "{}|Ledgerful|{}|{}|{}",
        scope_key(scope),
        project_id,
        record_kind,
        stable_key
    )
}

fn preview_from_record(record: &BridgeRecord) -> Preview {
    let privacy = format!("{:?}", record.privacy);
    let text = format!(
        "kind={} project={} privacy={} direction={:?} tx_id={}",
        record.record_kind,
        record.project_id,
        privacy,
        record.direction,
        record.tx_id.as_deref().unwrap_or("(none)"),
    );
    Preview {
        text,
        line_start: Some(1),
        line_end: Some(1),
    }
}

impl Connector for LedgerfulConnector {
    fn manifest(&self) -> &ConnectorManifest {
        &self.manifest
    }

    fn list(&self, ctx: &ConnectorContext) -> Result<Vec<SourceHandle>, ConnectorError> {
        if !self.manifest.operations.list {
            return Err(ConnectorError::OperationNotSupported { operation: "list" });
        }

        self.clear_side_channels();

        if self.store_unavailable {
            let reason = self
                .unavailable_reason_fixed
                .clone()
                .unwrap_or_else(|| "unavailable".into());
            self.set_unavailable(reason);
            return Ok(Vec::new());
        }

        let max = self.options.max_records;
        let all = self.records();
        let truncated = all.len() > max;
        let take = all.len().min(max);

        let mut handles: Vec<SourceHandle> = all[..take]
            .iter()
            .map(|r| self.handle_for(&ctx.scope, r))
            .collect();

        // Deterministic order by locator.
        handles.sort_by(|a, b| a.locator.cmp(&b.locator));

        if truncated {
            self.set_truncated(true);
        }

        Ok(handles)
    }

    fn observe(
        &self,
        ctx: &ConnectorContext,
        handle: &SourceHandle,
    ) -> Result<ObservePayload, ConnectorError> {
        if !self.manifest.operations.observe {
            return Err(ConnectorError::OperationNotSupported {
                operation: "observe",
            });
        }
        if handle.kind != SourceKind::Ledgerful {
            return Err(ConnectorError::UndeclaredSourceKind {
                kind: format!("{:?}", handle.kind),
            });
        }

        let record = self.find_record(handle)?;
        let built = self.handle_for(&ctx.scope, record);
        let content = serialize_bridge_record(record)?;

        Ok(ObservePayload {
            handle: built.clone(),
            content,
            identity: built.identity,
        })
    }

    fn preview(
        &self,
        ctx: &ConnectorContext,
        handle: &SourceHandle,
    ) -> Result<Preview, ConnectorError> {
        if !self.manifest.operations.preview {
            return Err(ConnectorError::OperationNotSupported {
                operation: "preview",
            });
        }
        if handle.kind != SourceKind::Ledgerful {
            return Err(ConnectorError::UndeclaredSourceKind {
                kind: format!("{:?}", handle.kind),
            });
        }

        let record = self.find_record(handle)?;
        // Touch ctx/scope via identity rebuild for consistency.
        let _ = self.handle_for(&ctx.scope, record);
        Ok(preview_from_record(record))
    }

    fn propose_write(
        &self,
        _ctx: &ConnectorContext,
        _proposal: &WriteProposalInput,
    ) -> Result<WriteProposal, ConnectorError> {
        Err(ConnectorError::OperationNotSupported {
            operation: "propose_write",
        })
    }
}

impl crate::refresh::ListSideChannels for LedgerfulConnector {
    fn last_list_truncated(&self) -> bool {
        LedgerfulConnector::last_list_truncated(self)
    }

    fn last_unavailable_reason(&self) -> Option<String> {
        LedgerfulConnector::last_unavailable_reason(self)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod unit_tests {
    use super::*;

    fn sample_record(tx: Option<&str>, parent: Option<&str>) -> BridgeRecord {
        let tx_json = match tx {
            Some(t) => format!("\"{t}\""),
            None => "null".into(),
        };
        let parent_json = match parent {
            Some(p) => format!("\"{p}\""),
            None => "null".into(),
        };
        let json = format!(
            r#"{{
                "bridge_version":"1.0",
                "direction":"inbound",
                "timestamp":"2026-05-19T00:00:00Z",
                "parent_hash":{parent_json},
                "project_id":"proj-1",
                "session_id":null,
                "tx_id":{tx_json},
                "record_kind":"prompt",
                "payload":{{"text":"hello"}},
                "privacy":"LocalOnly"
            }}"#
        );
        serde_json::from_str(&json).expect("sample BridgeRecord")
    }

    #[test]
    fn stable_record_key__tx_id_preferred() {
        let r = sample_record(Some("tx-abc"), Some("hash1"));
        assert_eq!(stable_record_key(&r), "tx-abc");
    }

    #[test]
    fn stable_record_key__no_tx__sha256_of_payload() {
        let r = sample_record(None, None);
        let key = stable_record_key(&r);
        assert_eq!(key.len(), 64);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
        // Stable across calls.
        assert_eq!(key, stable_record_key(&r));
    }
}
