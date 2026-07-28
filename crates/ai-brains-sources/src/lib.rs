//! Deterministic source fingerprints, content normalization, the connector
//! port + capability manifest (T149 / T153), Markdown / Obsidian vault
//! connector (T154), Git / Ledgerful connectors + bounded refresh (T155), and
//! Hermes / Honcho external-memory read adapters + circularity (T156).
//!
//! Digest algorithm is **SHA-256** only (plus authoritative string forms for
//! Ledgerful bridge hashes and External ETag/revision).
//!
//! Common format: `v{NORMALIZER_VERSION}:{sha256_hex}`.
//! File fingerprints fold canonical source identity into the preimage.
//!
//! Git connector I/O goes through
//! [`ai_brains_git::collect_metadata_strict_with_timeout`]; soft
//! [`ai_brains_git::collect_metadata`] remains for fingerprint helpers / legacy.
//! This crate never shells out to `git` directly and never hashes `.git` wholesale.
//!
//! # Connectors (T153 / T154 / T155 / T156)
//!
//! Sync [`Connector`] trait + versioned [`ConnectorManifest`] (`schema_version = 1`).
//! Production [`InProcessConnectorRegistry`] accepts only
//! [`SandboxMode::TrustedBuiltin`]. Policy still gates observe (T151);
//! this crate does **not** depend on the control-plane.
//!
//! Built-in path-bearing connectors:
//! - [`MarkdownObsidianConnector`] (`builtin.obsidian`)
//! - [`GitConnector`] (`builtin.git`)
//! - [`LedgerfulConnector`] (`builtin.ledgerful`)
//! - [`HermesConnector`] (`builtin.hermes`) — fixture/export read; flag default off
//! - [`HonchoConnector`] (`builtin.honcho`) — fixture/export read; **no AGPL SDK**; flag default off
//!
//! Circularity: [`classify_circularity`] never returns Independent; unmarked
//! external content is Unknown and cannot pass
//! [`may_count_as_independent_support`]. [`OutboundIndex`] is empty in
//! production v1.
//!
//! Bounded multi-connector helper: [`refresh_bounded`].

mod circularity;
mod connector;
mod fingerprint;
mod git;
mod git_fingerprint;
mod hermes;
mod honcho;
mod ledgerful;
mod manifest;
mod markdown;
mod mock;
mod normalization;
mod obsidian;
mod refresh;
mod registry;
mod vault_fs;

pub use circularity::{
    CircularityClass, EXTERNAL_ITEM_META_SCHEMA_VERSION, ExternalItemMeta, ExternalItemMetaInput,
    ORIGIN_MARKER_KEYS, OutboundIndex, classify_circularity, classify_circularity_with_fingerprint,
    classify_circularity_with_payload, extract_origin_markers_from_bytes,
    extract_origin_markers_from_value, filter_by_circularity_classes, filter_independent_support,
    may_count_as_independent_support, meta_with_assert_independent, payload_has_origin_markers,
};
pub use connector::{
    Connector, ConnectorContext, ConnectorError, ObservePayload, Preview, SourceHandle,
    WriteProposal, WriteProposalInput,
};
pub use fingerprint::{
    Sha256Fingerprinter, file_fingerprint_preimage, fingerprint_bytes, fingerprint_external,
    fingerprint_file_with_identity, fingerprint_ledgerful,
};
pub use git::{
    DEFAULT_GIT_COLLECT_CACHE_TTL_MS, DEFAULT_GIT_COLLECT_TIMEOUT_MS, DEFAULT_GIT_MAX_HANDLES,
    GIT_CONNECTOR_ID, GitConnector, GitConnectorOptions, REASON_NOT_A_REPOSITORY, map_git_error,
};
pub use git_fingerprint::{
    canonicalize_git_metadata, fingerprint_git_metadata, fingerprint_git_path,
};
pub use hermes::{
    DEFAULT_HERMES_MAX_HANDLES, DEFAULT_HERMES_TIMEOUT_MS, ENV_HERMES_CONNECTOR,
    HERMES_CONNECTOR_ID, HermesConnector, HermesConnectorOptions, HermesSessionSummary,
    HermesSource, REASON_CONNECTOR_DISABLED, deserialize_optional_privacy,
    is_env_connector_enabled, load_hermes_export_dir, make_hermes_identity, resolve_item_privacy,
};
pub use honcho::{
    DEFAULT_HONCHO_MAX_HANDLES, DEFAULT_HONCHO_TIMEOUT_MS, ENV_HONCHO_CONNECTOR,
    HONCHO_CONNECTOR_ID, HonchoConfirmedItem, HonchoConnector, HonchoConnectorOptions,
    HonchoSource, load_honcho_export_dir, make_honcho_identity,
};
pub use ledgerful::{
    DEFAULT_LEDGERFUL_MAX_RECORDS, LEDGERFUL_CONNECTOR_ID, LedgerfulConnector,
    LedgerfulConnectorOptions, LedgerfulSource, record_locator, serialize_bridge_record,
    stable_record_key,
};
pub use manifest::{
    ConnectorManifest, ConnectorOperations, ConnectorTrustLabel, CredentialDeclaration,
    FreshnessMechanism, MANIFEST_SCHEMA_VERSION, ManifestError, SandboxMode, ScopeClass,
    parse_manifest_json, parse_manifest_str, validate_manifest,
};
pub use markdown::{preview_from_markdown, split_frontmatter};
pub use mock::{MOCK_CONNECTOR_ID, MockConnector, MockSource};
pub use normalization::{NORMALIZER_VERSION, normalize_file_bytes, normalize_utf8_text};
pub use obsidian::{
    DEFAULT_MAX_DEPTH, DEFAULT_MAX_FILE_BYTES, DEFAULT_MAX_FILES, DEFAULT_PREVIEW_CHARS,
    MarkdownObsidianConnector, OBSIDIAN_CONNECTOR_ID, VaultOptions, is_obsidian_vault,
};
pub use refresh::{
    DEFAULT_REFRESH_DEADLINE, DEFAULT_REFRESH_DEADLINE_MS, ListSideChannels, ObservedItem,
    RefreshFailure, RefreshReport, RefreshTarget, refresh_bounded,
};
pub use registry::{
    CONNECTOR_PRINCIPAL_NAMESPACE, InProcessConnectorRegistry, RegistryError,
    principal_id_for_connector,
};
pub use vault_fs::{
    VaultFsError, is_reserved_windows_stem, normalize_locator, read_file_under_root,
    refuse_reparse_along_path, refuse_reparse_path, resolve_under_root,
};

pub use fingerprint::SourcesError;
pub type Result<T> = std::result::Result<T, SourcesError>;
