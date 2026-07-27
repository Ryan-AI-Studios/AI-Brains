//! Sync connector port (T153).
//!
//! # Path safety (deferred #12 partial — contract for T154+)
//!
//! Implementors that accept filesystem locators **must**:
//! - refuse Windows reparse points / symlink escape from configured roots;
//! - normalize path-bearing locators with helpers available **without** a
//!   control-plane dependency (e.g. `ai-brains-path` / crate-local helpers).
//!   Vault observe orchestration may also normalize at the control-plane
//!   boundary (`normalize_path_locator` lives there and must not be imported
//!   into this crate — that would create a sources↔control-plane cycle);
//! - never follow attacker-controlled links outside the vault root.
//!
//! Full soft-canonicalize TOCTOU hardening remains a P6 residual for T154+.
//!
//! # Write-back
//!
//! [`Connector::propose_write`] returns a proposal **artifact** only. It must
//! never create or mutate files on disk.

use ai_brains_core::ids::PrincipalId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::manifest::{ConnectorManifest, ConnectorTrustLabel};

/// Per-call context for connector operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectorContext {
    pub principal_id: Option<PrincipalId>,
    pub scope: ScopeRef,
    pub privacy: Privacy,
    pub trust: ConnectorTrustLabel,
}

/// Lightweight handle enumerating a candidate source without full content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceHandle {
    /// Stable identity key (typically scope+kind+locator) for fingerprinting.
    pub identity: String,
    pub kind: SourceKind,
    /// Locator string (path, URI, or logical key). Path-bearing locators must
    /// obey the path-safety contract in the module docs.
    pub locator: String,
}

/// Observation payload suitable for the fingerprint / observe pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservePayload {
    pub handle: SourceHandle,
    /// Raw content bytes (normalized by the fingerprint layer).
    pub content: Vec<u8>,
    /// Identity string folded into file fingerprints (`fingerprint_file_with_identity`).
    pub identity: String,
}

/// Bounded text preview with optional line anchors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preview {
    pub text: String,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
}

/// Input for a write proposal (never applied by the connector itself).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteProposalInput {
    pub handle: SourceHandle,
    /// Desired content the connector would write if approved.
    pub proposed_content: String,
    /// Optional human-readable rationale (not a secret).
    pub rationale: Option<String>,
}

/// Proposal artifact only — connectors must not mutate the filesystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteProposal {
    pub handle: SourceHandle,
    pub proposed_content: String,
    pub rationale: Option<String>,
    /// Opaque artifact id / description for review surfaces.
    pub artifact_id: String,
}

/// Errors from connector operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConnectorError {
    #[error("operation not supported by this connector: {operation}")]
    OperationNotSupported { operation: &'static str },

    #[error("source handle not found: {locator}")]
    HandleNotFound { locator: String },

    #[error("undeclared source kind for this connector: {kind:?}")]
    UndeclaredSourceKind { kind: String },

    #[error("connector error: {detail}")]
    Internal { detail: String },
}

/// Synchronous source connector port.
///
/// Core stays sync; do not add `async-trait` here.
pub trait Connector: Send + Sync {
    /// Declared capabilities; must match runtime behavior.
    fn manifest(&self) -> &ConnectorManifest;

    /// Enumerate candidate handles (if `operations.list`).
    fn list(&self, ctx: &ConnectorContext) -> Result<Vec<SourceHandle>, ConnectorError>;

    /// Load content + identity for fingerprint/observe (if `operations.observe`).
    fn observe(
        &self,
        ctx: &ConnectorContext,
        handle: &SourceHandle,
    ) -> Result<ObservePayload, ConnectorError>;

    /// Bounded preview (if `operations.preview`).
    fn preview(
        &self,
        ctx: &ConnectorContext,
        handle: &SourceHandle,
    ) -> Result<Preview, ConnectorError>;

    /// Return a write proposal artifact only; never writes FS (if `operations.propose_write`).
    fn propose_write(
        &self,
        ctx: &ConnectorContext,
        proposal: &WriteProposalInput,
    ) -> Result<WriteProposal, ConnectorError>;
}
