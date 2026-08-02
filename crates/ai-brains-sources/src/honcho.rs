//! Honcho confirmed-item connector (`builtin.honcho`) — T156 / P6.4.
//!
//! # License / AGPL posture
//!
//! The **Honcho** memory server (Plastic Labs) is AGPL-3.0. This crate **never**
//! links an AGPL SDK or the Honcho server. Adapters talk only to:
//!
//! - in-memory fixtures / exported NDJSON (CI default), or
//! - (future, optional) a user-run HTTP endpoint using **our** MIT/Apache DTOs
//!   and client — not required for T156 DoD.
//!
//! # Design locks
//!
//! - **Source:** fixture confirmed items, or NDJSON/JSONL export directory.
//! - **Identity:** `{scope_key}|Honcho|{item_id}`.
//! - **Content:** versioned JSON body + [`crate::ExternalItemMeta`].
//! - **Write-back:** [`Connector::propose_write`] is unsupported.
//! - **Sandbox / trust:** TrustedBuiltin / LocalOnly; credentials PathAccess.
//! - **Feature flag:** `AI_BRAINS_CONNECTOR_HONCHO` default **off**.
//! - **assert_independent trust:** [`HonchoConnector::from_fixture`] sets
//!   `trust_assert_independent = true`. Path loaders keep it **false** and
//!   ignore the field. Use [`HonchoConnector::with_trust_assert_independent`]
//!   for operator-attested imports.
//! - **OutboundIndex:** empty in production v1 (rule 2 is not a live production
//!   second layer; see [`crate::OutboundIndex`] and circularity rustdoc).
//!   Connector rule 2 matches `provider_item_id` / `origin_event_id` against
//!   the index (not a full observe-body content fingerprint).
//!
//! # Privacy
//!
//! Profile-class external data: absent or unparseable privacy →
//! [`Privacy::Sealed`] (strictest). Invalid privacy strings do **not** fail
//! item deserialize. Ambient scope privacy is not inherited as the item label.
//!
//! # Unavailability (anti-#22)
//!
//! Same discipline as Hermes: disabled → soft empty + `"connector disabled"`;
//! hard missing path → unavailable pattern; path load timeout → soft empty +
//! timeout reason on `list`, `Err` on `observe`; invalid JSONL → `Err`.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use serde::{Deserialize, Serialize};

use crate::circularity::{
    ExternalItemMeta, ExternalItemMetaInput, OutboundIndex, meta_with_assert_independent,
};
use crate::connector::{
    Connector, ConnectorContext, ConnectorError, ObservePayload, Preview, SourceHandle,
    WriteProposal, WriteProposalInput,
};
use crate::hermes::{
    REASON_CONNECTOR_DISABLED, deserialize_optional_privacy, is_env_connector_enabled,
    resolve_item_privacy, run_path_io_with_timeout,
};
use crate::manifest::{
    ConnectorManifest, ConnectorOperations, ConnectorTrustLabel, CredentialDeclaration,
    FreshnessMechanism, MANIFEST_SCHEMA_VERSION, SandboxMode, ScopeClass,
};

/// Stable connector id.
pub const HONCHO_CONNECTOR_ID: &str = "builtin.honcho";

/// Default cap on confirmed-item handles returned by `list`.
pub const DEFAULT_HONCHO_MAX_HANDLES: usize = 256;

/// Default timeout budget (ms) for path export loads.
///
/// Path loads honor [`HonchoConnectorOptions::timeout_ms`] via a wall-clock
/// deadline on a load worker thread. Future HTTP clients should use the same
/// budget.
pub const DEFAULT_HONCHO_TIMEOUT_MS: u64 = 5_000;

/// Env flag that enables the Honcho connector in production wiring.
pub const ENV_HONCHO_CONNECTOR: &str = "AI_BRAINS_CONNECTOR_HONCHO";

const DEFAULT_PREVIEW_CHARS: usize = 512;

/// Options for [`HonchoConnector`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HonchoConnectorOptions {
    /// Max handles from `list` (default [`DEFAULT_HONCHO_MAX_HANDLES`]).
    pub max_handles: usize,
    /// Path-load wall-clock deadline in ms (default [`DEFAULT_HONCHO_TIMEOUT_MS`]).
    ///
    /// Applied by [`HonchoConnector`] path mode around [`load_honcho_export_dir`]
    /// (worker thread + `recv_timeout`). `0` means **immediate timeout** without
    /// starting work (deterministic test / misconfig fail-closed). Fixture mode
    /// does no path IO and is unaffected.
    ///
    /// **Residual:** on timeout the load worker thread is not killed (pure FS
    /// on Windows cannot be cancelled safely); the orphaned thread may continue
    /// until the OS unblocks the IO (same class of residual as git Job Object
    /// kill limits). Callers still get a timely `Err` / soft-empty reason.
    pub timeout_ms: u64,
}

impl Default for HonchoConnectorOptions {
    fn default() -> Self {
        Self {
            max_handles: DEFAULT_HONCHO_MAX_HANDLES,
            timeout_ms: DEFAULT_HONCHO_TIMEOUT_MS,
        }
    }
}

/// Honcho confirmed item fixture DTO (schema_version = 1).
///
/// `kind` is a free string: typically `profile`, `conclusion`, or `representation`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HonchoConfirmedItem {
    pub schema_version: u32,
    pub item_id: String,
    /// profile | conclusion | representation (string, not enum — provider evolves).
    pub kind: String,
    pub statement: String,
    /// Provider timestamps as string or structured JSON.
    #[serde(default)]
    pub provider_timestamps: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<serde_json::Value>,
    /// Optional privacy; missing/unparseable → Sealed on observe.
    ///
    /// Unparseable strings deserialize as `None` (not a hard DTO error).
    #[serde(
        default,
        deserialize_with = "deserialize_optional_privacy",
        skip_serializing_if = "Option::is_none"
    )]
    pub privacy: Option<Privacy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_marker: Option<String>,
    /// Alternate rule-1 marker key (mapped into `origin_event_id` when empty).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_brains_event_id: Option<String>,
    /// Alternate rule-1 marker key (mapped into `origin_source_id` when empty).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_brains_source_id: Option<String>,
    /// Trusted construction only; Independent only when connector
    /// `trust_assert_independent` is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assert_independent: Option<bool>,
}

/// Where Honcho confirmed items are loaded from.
#[derive(Debug, Clone)]
pub enum HonchoSource {
    /// In-memory fixture (tests / offline).
    Fixture(Vec<HonchoConfirmedItem>),
    /// Directory of `*.jsonl` / `*.ndjson` export files.
    Path(PathBuf),
}

/// Built-in Honcho connector (read-only, fixture-first; no AGPL link).
pub struct HonchoConnector {
    source: HonchoSource,
    options: HonchoConnectorOptions,
    enabled: bool,
    /// When true, item `assert_independent: true` may set Independent.
    ///
    /// Default **false** for path / production construction; **true** for
    /// [`Self::from_fixture`] (trusted test/fixture path).
    trust_assert_independent: bool,
    outbound_index: OutboundIndex,
    manifest: ConnectorManifest,
    store_unavailable: bool,
    unavailable_reason_fixed: Option<String>,
    last_list_truncated: AtomicBool,
    last_unavailable_reason: Mutex<Option<String>>,
    path_cache: Mutex<Option<Result<Vec<HonchoConfirmedItem>, String>>>,
}

impl HonchoConnector {
    /// Fixture mode for tests; **enabled = true** and
    /// **trust_assert_independent = true** (trusted construction path).
    pub fn from_fixture(items: Vec<HonchoConfirmedItem>, options: HonchoConnectorOptions) -> Self {
        Self::new_inner(
            HonchoSource::Fixture(items),
            options,
            true,
            true,
            OutboundIndex::empty(),
            None,
        )
    }

    /// Fixture mode with explicit enabled flag.
    ///
    /// Sets `trust_assert_independent = true` (fixture is a trusted path).
    pub fn from_fixture_with_enabled(
        items: Vec<HonchoConfirmedItem>,
        options: HonchoConnectorOptions,
        enabled: bool,
    ) -> Self {
        Self::new_inner(
            HonchoSource::Fixture(items),
            options,
            enabled,
            true,
            OutboundIndex::empty(),
            None,
        )
    }

    /// Fixture mode that respects `AI_BRAINS_CONNECTOR_HONCHO` env (default off).
    pub fn from_fixture_env(
        items: Vec<HonchoConfirmedItem>,
        options: HonchoConnectorOptions,
    ) -> Self {
        Self::from_fixture_with_enabled(
            items,
            options,
            is_env_connector_enabled(ENV_HONCHO_CONNECTOR),
        )
    }

    /// Override enabled flag.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Whether this connector honors item `assert_independent` for Independent.
    pub fn trust_assert_independent(&self) -> bool {
        self.trust_assert_independent
    }

    /// Operator-attested import path: honor (or strip) `assert_independent`.
    ///
    /// Path loaders default to `false`. Call with `true` only when the operator
    /// attests the export may claim Independent.
    pub fn with_trust_assert_independent(mut self, trust: bool) -> Self {
        self.trust_assert_independent = trust;
        self
    }

    /// Seed outbound index (tests for circularity rule 2).
    pub fn with_outbound_index(mut self, index: OutboundIndex) -> Self {
        self.outbound_index = index;
        self
    }

    /// Path export directory (Phase E).
    ///
    /// `trust_assert_independent` is **false**: path JSON cannot self-attest
    /// Independent via `assert_independent`.
    pub fn from_path(dir: impl Into<PathBuf>, options: HonchoConnectorOptions) -> Self {
        Self::new_inner(
            HonchoSource::Path(dir.into()),
            options,
            true,
            false,
            OutboundIndex::empty(),
            None,
        )
    }

    /// Path mode with explicit enabled flag (`trust_assert_independent = false`).
    pub fn from_path_with_enabled(
        dir: impl Into<PathBuf>,
        options: HonchoConnectorOptions,
        enabled: bool,
    ) -> Self {
        Self::new_inner(
            HonchoSource::Path(dir.into()),
            options,
            enabled,
            false,
            OutboundIndex::empty(),
            None,
        )
    }

    /// Explicit unavailable store (missing / unreadable).
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::new_inner(
            HonchoSource::Fixture(Vec::new()),
            HonchoConnectorOptions::default(),
            true,
            false,
            OutboundIndex::empty(),
            Some(reason.into()),
        )
    }

    fn new_inner(
        source: HonchoSource,
        options: HonchoConnectorOptions,
        enabled: bool,
        trust_assert_independent: bool,
        outbound_index: OutboundIndex,
        unavailable: Option<String>,
    ) -> Self {
        let manifest = ConnectorManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: HONCHO_CONNECTOR_ID.into(),
            display_name: "Honcho".into(),
            connector_version: "0.1.0".into(),
            source_kinds: vec![SourceKind::Honcho],
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
            enabled,
            trust_assert_independent,
            outbound_index,
            manifest,
            store_unavailable: unavailable.is_some(),
            unavailable_reason_fixed: unavailable,
            last_list_truncated: AtomicBool::new(false),
            last_unavailable_reason: Mutex::new(None),
            path_cache: Mutex::new(None),
        }
    }

    /// Whether the last successful `list()` hit `max_handles` and stopped early.
    pub fn last_list_truncated(&self) -> bool {
        self.last_list_truncated.load(Ordering::Relaxed)
    }

    /// Soft-empty / hard-failure reason from the last `list`.
    pub fn last_unavailable_reason(&self) -> Option<String> {
        let g = self
            .last_unavailable_reason
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        g.clone()
    }

    /// Configured options.
    pub fn options(&self) -> &HonchoConnectorOptions {
        &self.options
    }

    /// True when constructed as an unavailable store.
    pub fn is_store_unavailable(&self) -> bool {
        self.store_unavailable
    }

    /// Whether the connector is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
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

    fn items_snapshot(&self) -> Result<Vec<HonchoConfirmedItem>, ConnectorError> {
        if self.store_unavailable {
            let reason = self
                .unavailable_reason_fixed
                .clone()
                .unwrap_or_else(|| "unavailable".into());
            return Err(ConnectorError::Internal { detail: reason });
        }

        match &self.source {
            HonchoSource::Fixture(items) => Ok(items.clone()),
            HonchoSource::Path(dir) => self.load_path_cached(dir),
        }
    }

    fn load_path_cached(&self, dir: &Path) -> Result<Vec<HonchoConfirmedItem>, ConnectorError> {
        {
            let guard = self.path_cache.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(cached) = guard.as_ref() {
                return match cached {
                    Ok(items) => Ok(items.clone()),
                    Err(e) => Err(ConnectorError::Internal { detail: e.clone() }),
                };
            }
        }

        let timeout_ms = self.options.timeout_ms;
        let dir = dir.to_path_buf();
        let loaded = run_path_io_with_timeout(timeout_ms, move || load_honcho_export_dir(&dir));
        let mut guard = self.path_cache.lock().unwrap_or_else(|p| p.into_inner());
        match loaded {
            Ok(items) => {
                *guard = Some(Ok(items.clone()));
                Ok(items)
            }
            Err(e) => {
                let detail = e.to_string();
                *guard = Some(Err(detail.clone()));
                Err(ConnectorError::Internal { detail })
            }
        }
    }

    fn handle_for(&self, scope: &ScopeRef, item: &HonchoConfirmedItem) -> SourceHandle {
        let identity = make_honcho_identity(scope, &item.item_id);
        SourceHandle {
            identity,
            kind: SourceKind::Honcho,
            locator: item.item_id.clone(),
        }
    }

    fn find_item(&self, handle: &SourceHandle) -> Result<HonchoConfirmedItem, ConnectorError> {
        if self.store_unavailable {
            let reason = self
                .unavailable_reason_fixed
                .clone()
                .unwrap_or_else(|| "unavailable".into());
            self.set_unavailable(reason.clone());
            return Err(ConnectorError::Internal { detail: reason });
        }
        if !self.enabled {
            self.set_unavailable(REASON_CONNECTOR_DISABLED);
            return Err(ConnectorError::Internal {
                detail: REASON_CONNECTOR_DISABLED.into(),
            });
        }

        let items = self.items_snapshot()?;
        items
            .into_iter()
            .find(|i| i.item_id == handle.locator)
            .ok_or_else(|| ConnectorError::HandleNotFound {
                locator: handle.locator.clone(),
            })
    }

    /// Build circularity meta for an observed item.
    ///
    /// - **Rule 1:** `origin_event_id` / `origin_source_id` / `origin_marker`,
    ///   with alternate payload keys `ai_brains_event_id` /
    ///   `ai_brains_source_id` mapped into origin fields when the primary
    ///   fields are empty (so Echo wins over Unknown).
    /// - **Rule 2:** [`OutboundIndex`] match on `origin_event_id` and on
    ///   `provider_item_id` as a fingerprint key (via
    ///   [`crate::classify_circularity`]). Connectors do **not** hash the full
    ///   observe body for rule 2 (meta is embedded in content). Seed tests with
    ///   `provider_item_id` fingerprints or origin event ids. Production index
    ///   is empty (see [`crate::OutboundIndex`]).
    /// - **Independent:** only when `self.trust_assert_independent` **and**
    ///   `item.assert_independent == true`.
    fn build_meta(&self, item: &HonchoConfirmedItem) -> ExternalItemMeta {
        // Prefer recorded_at from provider_timestamps when string; else None.
        let recorded_at = match &item.provider_timestamps {
            serde_json::Value::String(s) if !s.trim().is_empty() => Some(s.clone()),
            serde_json::Value::Object(map) => map
                .get("confirmed_at")
                .or_else(|| map.get("recorded_at"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            _ => None,
        };
        let origin_event_id = first_non_empty_opt(
            item.origin_event_id.as_deref(),
            item.ai_brains_event_id.as_deref(),
        );
        let origin_source_id = first_non_empty_opt(
            item.origin_source_id.as_deref(),
            item.ai_brains_source_id.as_deref(),
        );
        let assert_independent =
            self.trust_assert_independent && item.assert_independent.unwrap_or(false);
        meta_with_assert_independent(
            ExternalItemMetaInput {
                provider: "honcho".into(),
                provider_item_id: item.item_id.clone(),
                origin_event_id,
                origin_source_id,
                origin_marker: item.origin_marker.clone(),
                recorded_at,
                assert_independent,
            },
            &self.outbound_index,
        )
    }

    fn build_content_json(
        &self,
        item: &HonchoConfirmedItem,
        meta: &ExternalItemMeta,
    ) -> Result<Vec<u8>, ConnectorError> {
        #[derive(Serialize)]
        struct HonchoObserveContent<'a> {
            schema_version: u32,
            item_id: &'a str,
            kind: &'a str,
            statement: &'a str,
            provider_timestamps: &'a serde_json::Value,
            #[serde(skip_serializing_if = "Option::is_none")]
            confidence: Option<&'a serde_json::Value>,
            privacy: Privacy,
            external_item_meta: &'a ExternalItemMeta,
        }

        let body = HonchoObserveContent {
            schema_version: item.schema_version,
            item_id: &item.item_id,
            kind: &item.kind,
            statement: &item.statement,
            provider_timestamps: &item.provider_timestamps,
            confidence: item.confidence.as_ref(),
            privacy: resolve_item_privacy(item.privacy),
            external_item_meta: meta,
        };
        serde_json::to_vec(&body).map_err(|e| ConnectorError::Internal {
            detail: format!("honcho content serialize: {e}"),
        })
    }
}

fn scope_key(scope: &ScopeRef) -> String {
    match scope {
        ScopeRef::Personal(id) => format!("Personal:{id}"),
        ScopeRef::Repository(id) => format!("Repository:{id}"),
        ScopeRef::Workspace(id) => format!("Workspace:{id}"),
    }
}

/// Prefer the first non-empty trimmed string among two optional sources.
fn first_non_empty_opt(primary: Option<&str>, alternate: Option<&str>) -> Option<String> {
    for candidate in [primary, alternate].into_iter().flatten() {
        let t = candidate.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    None
}

/// Identity formula: `{scope}|Honcho|{item_id}`.
pub fn make_honcho_identity(scope: &ScopeRef, item_id: &str) -> String {
    format!("{}|Honcho|{item_id}", scope_key(scope))
}

/// Load all `*.jsonl` / `*.ndjson` lines from a directory into confirmed items.
///
/// Invalid JSON on a non-empty line → error (not silent skip).
///
/// Path I/O uses capability Dir list + per-component nofollow open (T190 / F4b).
/// Callers that need a wall-clock bound should wrap via the path-IO timeout
/// helper (Honcho path connector does this automatically).
pub fn load_honcho_export_dir(dir: &Path) -> Result<Vec<HonchoConfirmedItem>, ConnectorError> {
    let root =
        ai_brains_path::open_ambient_vault_dir(dir).map_err(|e| ConnectorError::Internal {
            detail: format!("honcho export open {}: {e}", dir.display()),
        })?;
    let names = ai_brains_path::list_entry_names(&root).map_err(|e| ConnectorError::Internal {
        detail: format!("honcho export list {}: {e}", dir.display()),
    })?;

    let mut files: Vec<String> = Vec::new();
    for name in names {
        let ext = Path::new(&name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext != "jsonl" && ext != "ndjson" {
            continue;
        }
        match ai_brains_path::open_file_component_nofollow(&root, &name) {
            Ok(_) => files.push(name),
            Err(ai_brains_path::CapOpenError::ReparseRefused(_))
            | Err(ai_brains_path::CapOpenError::NotAFile(_))
            | Err(ai_brains_path::CapOpenError::NotFound(_)) => continue,
            Err(e) => {
                return Err(ConnectorError::Internal {
                    detail: format!("honcho export open {name}: {e}"),
                });
            }
        }
    }
    files.sort();

    let mut items = Vec::new();
    for name in files {
        let bytes =
            ai_brains_path::read_file_nofollow_components(dir, &[name.as_str()], 16 * 1024 * 1024)
                .map_err(|e| ConnectorError::Internal {
                    detail: format!("honcho export read {name}: {e}"),
                })?;
        let text = String::from_utf8(bytes).map_err(|e| ConnectorError::Internal {
            detail: format!("honcho export utf-8 {name}: {e}"),
        })?;
        for (line_no, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let item: HonchoConfirmedItem =
                serde_json::from_str(trimmed).map_err(|e| ConnectorError::Internal {
                    detail: format!("honcho export invalid JSON {name}:{}: {e}", line_no + 1),
                })?;
            items.push(item);
        }
    }
    Ok(items)
}

fn preview_from_item(item: &HonchoConfirmedItem) -> Preview {
    let text = if item.statement.chars().count() > DEFAULT_PREVIEW_CHARS {
        let truncated: String = item.statement.chars().take(DEFAULT_PREVIEW_CHARS).collect();
        format!("{truncated}…")
    } else {
        item.statement.clone()
    };
    Preview {
        text,
        line_start: Some(1),
        line_end: Some(1),
    }
}

impl Connector for HonchoConnector {
    fn manifest(&self) -> &ConnectorManifest {
        &self.manifest
    }

    fn list(&self, ctx: &ConnectorContext) -> Result<Vec<SourceHandle>, ConnectorError> {
        if !self.manifest.operations.list {
            return Err(ConnectorError::OperationNotSupported { operation: "list" });
        }

        self.clear_side_channels();

        if !self.enabled {
            self.set_unavailable(REASON_CONNECTOR_DISABLED);
            return Ok(Vec::new());
        }

        if self.store_unavailable {
            let reason = self
                .unavailable_reason_fixed
                .clone()
                .unwrap_or_else(|| "unavailable".into());
            self.set_unavailable(reason);
            return Ok(Vec::new());
        }

        let items = match self.items_snapshot() {
            Ok(items) => items,
            Err(e) => {
                let detail = e.to_string();
                self.set_unavailable(detail);
                return Ok(Vec::new());
            }
        };

        let max = self.options.max_handles;
        let truncated = items.len() > max;
        let take = items.len().min(max);

        let mut handles: Vec<SourceHandle> = items[..take]
            .iter()
            .map(|i| self.handle_for(&ctx.scope, i))
            .collect();
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
        if handle.kind != SourceKind::Honcho {
            return Err(ConnectorError::UndeclaredSourceKind {
                kind: format!("{:?}", handle.kind),
            });
        }

        let item = self.find_item(handle)?;
        let built = self.handle_for(&ctx.scope, &item);
        let meta = self.build_meta(&item);
        let content = self.build_content_json(&item, &meta)?;

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
        if handle.kind != SourceKind::Honcho {
            return Err(ConnectorError::UndeclaredSourceKind {
                kind: format!("{:?}", handle.kind),
            });
        }

        let item = self.find_item(handle)?;
        let _ = self.handle_for(&ctx.scope, &item);
        Ok(preview_from_item(&item))
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

impl crate::refresh::ListSideChannels for HonchoConnector {
    fn last_list_truncated(&self) -> bool {
        HonchoConnector::last_list_truncated(self)
    }

    fn last_unavailable_reason(&self) -> Option<String> {
        HonchoConnector::last_unavailable_reason(self)
    }
}
