//! In-memory mock connector for contract tests and local fixtures (T153).
//!
//! Never writes to the filesystem. Fixture sources are held in memory.

use std::collections::HashMap;

use ai_brains_core::source::SourceKind;

use crate::connector::{
    Connector, ConnectorContext, ConnectorError, ObservePayload, Preview, SourceHandle,
    WriteProposal, WriteProposalInput,
};
use crate::manifest::{
    ConnectorManifest, ConnectorOperations, ConnectorTrustLabel, CredentialDeclaration,
    FreshnessMechanism, MANIFEST_SCHEMA_VERSION, SandboxMode, ScopeClass,
};

/// Stable mock connector id used in golden fixtures and contract tests.
pub const MOCK_CONNECTOR_ID: &str = "builtin.mock";

/// In-memory fixture entry.
#[derive(Debug, Clone)]
pub struct MockSource {
    pub handle: SourceHandle,
    pub content: Vec<u8>,
}

/// Configurable in-memory connector.
#[derive(Debug, Clone)]
pub struct MockConnector {
    manifest: ConnectorManifest,
    sources: HashMap<String, MockSource>,
}

impl MockConnector {
    /// Default mock: File kind, all ops enabled, one fixture document.
    pub fn new() -> Self {
        Self::with_operations(ConnectorOperations {
            list: true,
            observe: true,
            preview: true,
            propose_write: true,
        })
    }

    /// Mock with custom operation flags (for OperationNotSupported tests).
    pub fn with_operations(operations: ConnectorOperations) -> Self {
        let mut sources = HashMap::new();
        let handle = SourceHandle {
            identity: "Personal:mock|File|/fixture/notes.md".into(),
            kind: SourceKind::File,
            locator: "/fixture/notes.md".into(),
        };
        sources.insert(
            handle.locator.clone(),
            MockSource {
                handle: handle.clone(),
                content: b"# Mock notes\nhello from mock\n".to_vec(),
            },
        );

        let manifest = ConnectorManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: MOCK_CONNECTOR_ID.into(),
            display_name: "Mock Connector".into(),
            connector_version: "0.1.0".into(),
            source_kinds: vec![SourceKind::File],
            operations,
            scope_affinity: vec![
                ScopeClass::Personal,
                ScopeClass::Repository,
                ScopeClass::Workspace,
            ],
            freshness: FreshnessMechanism::Fingerprint,
            credentials: CredentialDeclaration::None,
            sandbox: SandboxMode::TrustedBuiltin,
            default_trust: ConnectorTrustLabel::LocalOnly,
            principal_id: None,
        };

        Self { manifest, sources }
    }

    /// Override connector id (e.g. for multi-registry tests).
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.manifest.id = id.into();
        self
    }

    /// Insert or replace an in-memory fixture source.
    ///
    /// Rejects handles whose kind is not in `manifest.source_kinds` so fixtures
    /// cannot violate the shared connector contract via the public mock API.
    pub fn insert_source(&mut self, source: MockSource) -> Result<(), ConnectorError> {
        self.ensure_kind_declared(&source.handle.kind)?;
        self.sources.insert(source.handle.locator.clone(), source);
        Ok(())
    }

    fn ensure_kind_declared(&self, kind: &SourceKind) -> Result<(), ConnectorError> {
        if self.manifest.source_kinds.iter().any(|k| k == kind) {
            Ok(())
        } else {
            Err(ConnectorError::UndeclaredSourceKind {
                kind: format!("{kind:?}"),
            })
        }
    }

    fn lookup(&self, handle: &SourceHandle) -> Result<&MockSource, ConnectorError> {
        self.sources
            .get(&handle.locator)
            .ok_or_else(|| ConnectorError::HandleNotFound {
                locator: handle.locator.clone(),
            })
    }
}

impl Default for MockConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl Connector for MockConnector {
    fn manifest(&self) -> &ConnectorManifest {
        &self.manifest
    }

    fn list(&self, _ctx: &ConnectorContext) -> Result<Vec<SourceHandle>, ConnectorError> {
        if !self.manifest.operations.list {
            return Err(ConnectorError::OperationNotSupported { operation: "list" });
        }
        let mut handles: Vec<SourceHandle> = self
            .sources
            .values()
            .filter(|s| {
                self.manifest
                    .source_kinds
                    .iter()
                    .any(|k| k == &s.handle.kind)
            })
            .map(|s| s.handle.clone())
            .collect();
        // Determinism: sort by locator.
        handles.sort_by(|a, b| a.locator.cmp(&b.locator));
        Ok(handles)
    }

    fn observe(
        &self,
        _ctx: &ConnectorContext,
        handle: &SourceHandle,
    ) -> Result<ObservePayload, ConnectorError> {
        if !self.manifest.operations.observe {
            return Err(ConnectorError::OperationNotSupported {
                operation: "observe",
            });
        }
        self.ensure_kind_declared(&handle.kind)?;
        let source = self.lookup(handle)?;
        Ok(ObservePayload {
            handle: source.handle.clone(),
            content: source.content.clone(),
            identity: source.handle.identity.clone(),
        })
    }

    fn preview(
        &self,
        _ctx: &ConnectorContext,
        handle: &SourceHandle,
    ) -> Result<Preview, ConnectorError> {
        if !self.manifest.operations.preview {
            return Err(ConnectorError::OperationNotSupported {
                operation: "preview",
            });
        }
        self.ensure_kind_declared(&handle.kind)?;
        let source = self.lookup(handle)?;
        let text = String::from_utf8_lossy(&source.content).into_owned();
        // Bound preview: first 512 chars.
        let bound: String = text.chars().take(512).collect();
        Ok(Preview {
            text: bound,
            line_start: Some(1),
            line_end: None,
        })
    }

    fn propose_write(
        &self,
        _ctx: &ConnectorContext,
        proposal: &WriteProposalInput,
    ) -> Result<WriteProposal, ConnectorError> {
        if !self.manifest.operations.propose_write {
            return Err(ConnectorError::OperationNotSupported {
                operation: "propose_write",
            });
        }
        self.ensure_kind_declared(&proposal.handle.kind)?;
        // Artifact only — no filesystem mutation, no in-memory content change.
        Ok(WriteProposal {
            handle: proposal.handle.clone(),
            proposed_content: proposal.proposed_content.clone(),
            rationale: proposal.rationale.clone(),
            artifact_id: format!(
                "mock-proposal:{}:{}",
                proposal.handle.locator,
                proposal.proposed_content.len()
            ),
        })
    }
}
