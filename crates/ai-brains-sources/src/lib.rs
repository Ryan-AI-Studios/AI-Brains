//! Deterministic source fingerprints, content normalization, and the
//! connector port + capability manifest (T149 / T153).
//!
//! Digest algorithm is **SHA-256** only (plus authoritative string forms for
//! Ledgerful bridge hashes and External ETag/revision).
//!
//! Common format: `v{NORMALIZER_VERSION}:{sha256_hex}`.
//! File fingerprints fold canonical source identity into the preimage.
//!
//! Git I/O goes exclusively through [`ai_brains_git::collect_metadata`] — this
//! crate never shells out to `git` and never hashes `.git` wholesale.
//!
//! # Connectors (T153)
//!
//! Sync [`Connector`] trait + versioned [`ConnectorManifest`] (`schema_version = 1`).
//! Production [`InProcessConnectorRegistry`] accepts only
//! [`SandboxMode::TrustedBuiltin`]. Policy still gates observe (T151);
//! this crate does **not** depend on the control-plane.

mod connector;
mod fingerprint;
mod git_fingerprint;
mod manifest;
mod mock;
mod normalization;
mod registry;

pub use connector::{
    Connector, ConnectorContext, ConnectorError, ObservePayload, Preview, SourceHandle,
    WriteProposal, WriteProposalInput,
};
pub use fingerprint::{
    Sha256Fingerprinter, file_fingerprint_preimage, fingerprint_bytes, fingerprint_external,
    fingerprint_file_with_identity, fingerprint_ledgerful,
};
pub use git_fingerprint::{
    canonicalize_git_metadata, fingerprint_git_metadata, fingerprint_git_path,
};
pub use manifest::{
    ConnectorManifest, ConnectorOperations, ConnectorTrustLabel, CredentialDeclaration,
    FreshnessMechanism, MANIFEST_SCHEMA_VERSION, ManifestError, SandboxMode, ScopeClass,
    parse_manifest_json, parse_manifest_str, validate_manifest,
};
pub use mock::{MOCK_CONNECTOR_ID, MockConnector, MockSource};
pub use normalization::{NORMALIZER_VERSION, normalize_file_bytes, normalize_utf8_text};
pub use registry::{
    CONNECTOR_PRINCIPAL_NAMESPACE, InProcessConnectorRegistry, RegistryError,
    principal_id_for_connector,
};

pub use fingerprint::SourcesError;
pub type Result<T> = std::result::Result<T, SourcesError>;
