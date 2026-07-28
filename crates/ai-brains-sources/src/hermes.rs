//! Hermes session-summary connector (`builtin.hermes`) — T156 / P6.4.
//!
//! # Design locks
//!
//! - **Source:** in-memory fixture summaries, or NDJSON/JSONL export directory.
//! - **Identity:** `{scope_key}|HermesSession|{session_id}`.
//! - **Content:** versioned JSON body + [`crate::ExternalItemMeta`] (circularity
//!   from [`crate::classify_circularity`], or Independent only when a **trusted**
//!   construction path honors `assert_independent: true` — see
//!   [`HermesConnector::trust_assert_independent`]).
//! - **Write-back:** [`Connector::propose_write`] is unsupported.
//! - **Sandbox / trust:** TrustedBuiltin / LocalOnly; credentials PathAccess.
//! - **Feature flag:** `AI_BRAINS_CONNECTOR_HERMES` default **off** (missing /
//!   `0` / `false` → disabled). Production wiring should pass
//!   [`is_env_connector_enabled`]; tests use explicit [`HermesConnector::from_fixture`]
//!   (`enabled = true`) or [`HermesConnector::with_enabled`].
//! - **assert_independent trust:** [`HermesConnector::from_fixture`] sets
//!   `trust_assert_independent = true`. Path loaders
//!   ([`HermesConnector::from_path`]) keep it **false** and ignore the field
//!   (never Independent from untrusted export JSON). Use
//!   [`HermesConnector::with_trust_assert_independent`] for operator-attested
//!   imports.
//! - **OutboundIndex:** empty in production v1 (see [`crate::OutboundIndex`]).
//!   Connector rule 2 matches `provider_item_id` / `origin_event_id` against
//!   the index (not a full observe-body content fingerprint).
//!
//! # Privacy
//!
//! If the fixture/export carries a parseable [`Privacy`] value, that label is
//! used. If privacy is **absent or unparseable** → [`Privacy::Sealed`]. Ambient
//! caller privacy is **not** inherited as the item label. Invalid privacy
//! strings do **not** fail item deserialize (lenient field deserializer).
//!
//! # Unavailability (anti-#22)
//!
//! | Situation | Behavior |
//! |-----------|----------|
//! | Connector disabled | `list` → `Ok([])` + reason `"connector disabled"` |
//! | Healthy empty fixture | `list` → `Ok([])`; reason `None` |
//! | Hard missing / unreadable path | Use [`HermesConnector::unavailable`]; soft empty + reason; `observe` → `Err` |
//! | Path load timeout (`timeout_ms`) | `list` → soft empty + reason containing timeout; `observe` → `Err` |
//! | Invalid JSONL line | `list`/`load` → `Err(Internal)` (not silent empty) |
//!
//! Callers **must** check [`Self::last_unavailable_reason`] and
//! [`Self::last_list_truncated`] immediately after every `list`.

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
use crate::manifest::{
    ConnectorManifest, ConnectorOperations, ConnectorTrustLabel, CredentialDeclaration,
    FreshnessMechanism, MANIFEST_SCHEMA_VERSION, SandboxMode, ScopeClass,
};

/// Stable connector id.
pub const HERMES_CONNECTOR_ID: &str = "builtin.hermes";

/// Default cap on session handles returned by `list`.
pub const DEFAULT_HERMES_MAX_HANDLES: usize = 256;

/// Default timeout budget (ms) for path export loads.
///
/// Path loads honor [`HermesConnectorOptions::timeout_ms`] via a wall-clock
/// deadline on a load worker thread. Future HTTP clients should use the same
/// budget.
pub const DEFAULT_HERMES_TIMEOUT_MS: u64 = 5_000;

/// Env flag that enables the Hermes connector in production wiring.
pub const ENV_HERMES_CONNECTOR: &str = "AI_BRAINS_CONNECTOR_HERMES";

/// Soft-empty reason when the connector is feature-flag disabled.
pub const REASON_CONNECTOR_DISABLED: &str = "connector disabled";

/// Default preview character budget for summary text.
const DEFAULT_PREVIEW_CHARS: usize = 512;

/// Options for [`HermesConnector`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesConnectorOptions {
    /// Max handles from `list` (default [`DEFAULT_HERMES_MAX_HANDLES`]).
    pub max_handles: usize,
    /// Path-load wall-clock deadline in ms (default [`DEFAULT_HERMES_TIMEOUT_MS`]).
    ///
    /// Applied by [`HermesConnector`] path mode around [`load_hermes_export_dir`]
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

impl Default for HermesConnectorOptions {
    fn default() -> Self {
        Self {
            max_handles: DEFAULT_HERMES_MAX_HANDLES,
            timeout_ms: DEFAULT_HERMES_TIMEOUT_MS,
        }
    }
}

/// Hermes session summary fixture DTO (schema_version = 1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HermesSessionSummary {
    pub schema_version: u32,
    pub session_id: String,
    pub summary_text: String,
    #[serde(default)]
    pub source_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurred_at: Option<String>,
    /// Optional privacy; missing/unparseable → Sealed on observe.
    ///
    /// Unparseable strings deserialize as `None` (not a hard DTO error) so the
    /// item still loads and observe labels it [`Privacy::Sealed`].
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

/// Where Hermes summaries are loaded from.
#[derive(Debug, Clone)]
pub enum HermesSource {
    /// In-memory fixture (tests / offline).
    Fixture(Vec<HermesSessionSummary>),
    /// Directory of `*.jsonl` / `*.ndjson` export files.
    Path(PathBuf),
}

/// Built-in Hermes connector (read-only, fixture-first).
pub struct HermesConnector {
    source: HermesSource,
    options: HermesConnectorOptions,
    /// When false, `list` returns soft-empty with reason (contracted).
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
    /// Lazily loaded path items (populated on first list when Path source).
    path_cache: Mutex<Option<Result<Vec<HermesSessionSummary>, String>>>,
}

impl HermesConnector {
    /// Fixture mode for tests; **enabled = true** and
    /// **trust_assert_independent = true** (trusted construction path).
    pub fn from_fixture(items: Vec<HermesSessionSummary>, options: HermesConnectorOptions) -> Self {
        Self::new_inner(
            HermesSource::Fixture(items),
            options,
            true,
            true,
            OutboundIndex::empty(),
            None,
        )
    }

    /// Fixture mode with explicit enabled flag (and optional outbound index).
    ///
    /// Sets `trust_assert_independent = true` (fixture is a trusted path).
    pub fn from_fixture_with_enabled(
        items: Vec<HermesSessionSummary>,
        options: HermesConnectorOptions,
        enabled: bool,
    ) -> Self {
        Self::new_inner(
            HermesSource::Fixture(items),
            options,
            enabled,
            true,
            OutboundIndex::empty(),
            None,
        )
    }

    /// Fixture mode that respects `AI_BRAINS_CONNECTOR_HERMES` env (default off).
    pub fn from_fixture_env(
        items: Vec<HermesSessionSummary>,
        options: HermesConnectorOptions,
    ) -> Self {
        Self::from_fixture_with_enabled(
            items,
            options,
            is_env_connector_enabled(ENV_HERMES_CONNECTOR),
        )
    }

    /// Override enabled flag on an existing connector configuration.
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

    /// Path export directory (Phase E). Loads `*.jsonl` / `*.ndjson` on list.
    ///
    /// `trust_assert_independent` is **false**: path JSON cannot self-attest
    /// Independent via `assert_independent`.
    pub fn from_path(dir: impl Into<PathBuf>, options: HermesConnectorOptions) -> Self {
        Self::new_inner(
            HermesSource::Path(dir.into()),
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
        options: HermesConnectorOptions,
        enabled: bool,
    ) -> Self {
        Self::new_inner(
            HermesSource::Path(dir.into()),
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
            HermesSource::Fixture(Vec::new()),
            HermesConnectorOptions::default(),
            true,
            false,
            OutboundIndex::empty(),
            Some(reason.into()),
        )
    }

    fn new_inner(
        source: HermesSource,
        options: HermesConnectorOptions,
        enabled: bool,
        trust_assert_independent: bool,
        outbound_index: OutboundIndex,
        unavailable: Option<String>,
    ) -> Self {
        let manifest = ConnectorManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: HERMES_CONNECTOR_ID.into(),
            display_name: "Hermes".into(),
            connector_version: "0.1.0".into(),
            source_kinds: vec![SourceKind::HermesSession],
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
    pub fn options(&self) -> &HermesConnectorOptions {
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

    fn items_snapshot(&self) -> Result<Vec<HermesSessionSummary>, ConnectorError> {
        if self.store_unavailable {
            let reason = self
                .unavailable_reason_fixed
                .clone()
                .unwrap_or_else(|| "unavailable".into());
            return Err(ConnectorError::Internal { detail: reason });
        }

        match &self.source {
            HermesSource::Fixture(items) => Ok(items.clone()),
            HermesSource::Path(dir) => self.load_path_cached(dir),
        }
    }

    fn load_path_cached(&self, dir: &Path) -> Result<Vec<HermesSessionSummary>, ConnectorError> {
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
        let loaded = run_path_io_with_timeout(timeout_ms, move || load_hermes_export_dir(&dir));
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

    fn handle_for(&self, scope: &ScopeRef, item: &HermesSessionSummary) -> SourceHandle {
        let identity = make_hermes_identity(scope, &item.session_id);
        SourceHandle {
            identity,
            kind: SourceKind::HermesSession,
            locator: item.session_id.clone(),
        }
    }

    fn find_item(&self, handle: &SourceHandle) -> Result<HermesSessionSummary, ConnectorError> {
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
            .find(|i| i.session_id == handle.locator)
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
    ///   observe body for rule 2 (meta is embedded in content; body fingerprint
    ///   would be circular). Seed tests with `provider_item_id` fingerprints or
    ///   origin event ids. Production index is empty (see
    ///   [`crate::OutboundIndex`]).
    /// - **Independent:** only when `self.trust_assert_independent` **and**
    ///   `item.assert_independent == true`.
    fn build_meta(&self, item: &HermesSessionSummary) -> ExternalItemMeta {
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
                provider: "hermes".into(),
                provider_item_id: item.session_id.clone(),
                origin_event_id,
                origin_source_id,
                origin_marker: item.origin_marker.clone(),
                recorded_at: item.occurred_at.clone(),
                assert_independent,
            },
            &self.outbound_index,
        )
    }

    fn build_content_json(
        &self,
        item: &HermesSessionSummary,
        meta: &ExternalItemMeta,
    ) -> Result<Vec<u8>, ConnectorError> {
        #[derive(Serialize)]
        struct HermesObserveContent<'a> {
            schema_version: u32,
            session_id: &'a str,
            summary_text: &'a str,
            source_ids: &'a [String],
            #[serde(skip_serializing_if = "Option::is_none")]
            occurred_at: Option<&'a str>,
            privacy: Privacy,
            external_item_meta: &'a ExternalItemMeta,
        }

        let body = HermesObserveContent {
            schema_version: item.schema_version,
            session_id: &item.session_id,
            summary_text: &item.summary_text,
            source_ids: &item.source_ids,
            occurred_at: item.occurred_at.as_deref(),
            privacy: resolve_item_privacy(item.privacy),
            external_item_meta: meta,
        };
        serde_json::to_vec(&body).map_err(|e| ConnectorError::Internal {
            detail: format!("hermes content serialize: {e}"),
        })
    }
}

/// True when env var is `1`, `true`, or `yes` (case-insensitive). Missing / other → false.
pub fn is_env_connector_enabled(var: &str) -> bool {
    match std::env::var(var) {
        Ok(v) => {
            let t = v.trim();
            t.eq_ignore_ascii_case("1")
                || t.eq_ignore_ascii_case("true")
                || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

/// Resolve external-item privacy: present parseable → use; else Sealed.
pub fn resolve_item_privacy(privacy: Option<Privacy>) -> Privacy {
    privacy.unwrap_or(Privacy::Sealed)
}

/// Lenient optional [`Privacy`] deserializer for external DTOs.
///
/// Known Privacy values parse normally; unknown/invalid values become `None`
/// (caller then applies [`resolve_item_privacy`] → Sealed). Does not fail the
/// whole item deserialize.
pub fn deserialize_optional_privacy<'de, D>(deserializer: D) -> Result<Option<Privacy>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(v) => Ok(serde_json::from_value::<Privacy>(v).ok()),
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

fn scope_key(scope: &ScopeRef) -> String {
    match scope {
        ScopeRef::Personal(id) => format!("Personal:{id}"),
        ScopeRef::Repository(id) => format!("Repository:{id}"),
        ScopeRef::Workspace(id) => format!("Workspace:{id}"),
    }
}

/// Identity formula: `{scope}|HermesSession|{session_id}`.
pub fn make_hermes_identity(scope: &ScopeRef, session_id: &str) -> String {
    format!("{}|HermesSession|{session_id}", scope_key(scope))
}

/// Run path (or other blocking) IO under a wall-clock deadline.
///
/// Spawns `work` on a dedicated thread and waits up to `timeout_ms` for its
/// result. Used by Hermes/Honcho path loaders so `timeout_ms` is enforced.
///
/// - **`timeout_ms == 0`:** returns timeout immediately **without** starting
///   work (deterministic test / misconfig fail-closed).
/// - **Timeout residual:** the worker thread is **not** killed after the
///   deadline. Pure filesystem IO on Windows cannot be cancelled safely; an
///   orphaned thread may continue until the OS unblocks. Callers still observe
///   a timely error. Same residual class as git Job Object limits for wedged
///   subprocesses that ignore kill signals.
pub(crate) fn run_path_io_with_timeout<T, F>(timeout_ms: u64, work: F) -> Result<T, ConnectorError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ConnectorError> + Send + 'static,
{
    if timeout_ms == 0 {
        return Err(ConnectorError::Internal {
            detail: "path load timeout (timeout_ms=0)".into(),
        });
    }

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(work());
    });

    match rx.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(ConnectorError::Internal {
            detail: format!("path load timed out after {timeout_ms}ms"),
        }),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err(ConnectorError::Internal {
            detail: "path load worker disconnected".into(),
        }),
    }
}

/// Load all `*.jsonl` / `*.ndjson` lines from a directory into summaries.
///
/// Invalid JSON on a non-empty line → error (not silent skip).
///
/// Callers that need a wall-clock bound should wrap via the path-IO timeout
/// helper (Hermes/Honcho path connectors do this automatically).
pub fn load_hermes_export_dir(dir: &Path) -> Result<Vec<HermesSessionSummary>, ConnectorError> {
    if !dir.exists() {
        return Err(ConnectorError::Internal {
            detail: format!("hermes export path missing: {}", dir.display()),
        });
    }
    if !dir.is_dir() {
        return Err(ConnectorError::Internal {
            detail: format!("hermes export path not a directory: {}", dir.display()),
        });
    }

    let entries = std::fs::read_dir(dir).map_err(|e| ConnectorError::Internal {
        detail: format!("hermes export read_dir: {e}"),
    })?;

    let mut files: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| ConnectorError::Internal {
            detail: format!("hermes export dir entry: {e}"),
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext == "jsonl" || ext == "ndjson" {
            files.push(path);
        }
    }
    files.sort();

    let mut items = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).map_err(|e| ConnectorError::Internal {
            detail: format!("hermes export read {}: {e}", path.display()),
        })?;
        for (line_no, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let item: HermesSessionSummary =
                serde_json::from_str(trimmed).map_err(|e| ConnectorError::Internal {
                    detail: format!(
                        "hermes export invalid JSON {}:{}: {e}",
                        path.display(),
                        line_no + 1
                    ),
                })?;
            items.push(item);
        }
    }
    Ok(items)
}

fn preview_from_summary(item: &HermesSessionSummary) -> Preview {
    let text = if item.summary_text.chars().count() > DEFAULT_PREVIEW_CHARS {
        let truncated: String = item
            .summary_text
            .chars()
            .take(DEFAULT_PREVIEW_CHARS)
            .collect();
        format!("{truncated}…")
    } else {
        item.summary_text.clone()
    };
    Preview {
        text,
        line_start: Some(1),
        line_end: Some(1),
    }
}

impl Connector for HermesConnector {
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
                // Path hard failures: soft-empty list + side-channel (anti-#22),
                // matching unavailable pattern for missing/unreadable stores.
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
        if handle.kind != SourceKind::HermesSession {
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
        if handle.kind != SourceKind::HermesSession {
            return Err(ConnectorError::UndeclaredSourceKind {
                kind: format!("{:?}", handle.kind),
            });
        }

        let item = self.find_item(handle)?;
        let _ = self.handle_for(&ctx.scope, &item);
        Ok(preview_from_summary(&item))
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

impl crate::refresh::ListSideChannels for HermesConnector {
    fn last_list_truncated(&self) -> bool {
        HermesConnector::last_list_truncated(self)
    }

    fn last_unavailable_reason(&self) -> Option<String> {
        HermesConnector::last_unavailable_reason(self)
    }
}
