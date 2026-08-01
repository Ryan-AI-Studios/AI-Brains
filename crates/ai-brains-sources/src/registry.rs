//! In-process connector registry (T153).
//!
//! Binds a stable [`PrincipalId`] (UUID v5 from connector id) on register.
//! Production register refuses non-[`SandboxMode::TrustedBuiltin`] sandboxes
//! per **ADR-0019** (Trusted Built-ins First; two-layer L9 defense with
//! serde fail-closed on unknown sandbox strings).
//! No vault / projection persistence.

use std::collections::BTreeMap;

use ai_brains_core::ids::PrincipalId;
use thiserror::Error;
use uuid::Uuid;

use crate::connector::Connector;
use crate::manifest::{ConnectorManifest, ManifestError, SandboxMode, validate_manifest};

/// UUID v5 namespace for AI-Brains connector principal binding.
///
/// Uses the standard OID namespace so principal ids are stable across runs given
/// the same connector id string. Documented fixed namespace for connectors.
pub const CONNECTOR_PRINCIPAL_NAMESPACE: Uuid = Uuid::NAMESPACE_OID;

/// Name prefix folded into the v5 name for clarity in offline inspection.
const CONNECTOR_PRINCIPAL_NAME_PREFIX: &str = "ai-brains.connector.";

/// Derive a stable [`PrincipalId`] for a connector id string.
pub fn principal_id_for_connector(connector_id: &str) -> PrincipalId {
    let name = format!("{CONNECTOR_PRINCIPAL_NAME_PREFIX}{connector_id}");
    PrincipalId::from_uuid(Uuid::new_v5(
        &CONNECTOR_PRINCIPAL_NAMESPACE,
        name.as_bytes(),
    ))
}

/// Registry errors.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RegistryError {
    #[error("connector id already registered: {0}")]
    DuplicateId(String),

    #[error("sandbox mode not allowed for production registry: only TrustedBuiltin")]
    SandboxNotAllowed,

    #[error("invalid connector manifest: {0}")]
    InvalidManifest(#[from] ManifestError),
}

struct Entry {
    connector: Box<dyn Connector>,
    /// Manifest copy with `principal_id` bound after register.
    bound_manifest: ConnectorManifest,
}

/// In-process registry: register, get by id, list manifests (sorted by id).
#[derive(Default)]
pub struct InProcessConnectorRegistry {
    entries: BTreeMap<String, Entry>,
}

impl InProcessConnectorRegistry {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Register a connector.
    ///
    /// - Validates the manifest (`schema_version == 1`, non-empty kinds, …).
    /// - Refuses non-`TrustedBuiltin` sandbox (**ADR-0019** L1 / L9 layer 2).
    /// - Binds a stable UUID v5 [`PrincipalId`] on the **registry-owned**
    ///   bound manifest (`get_manifest` / `list_manifests` only).
    /// - Does **not** mutate `connector.manifest()`; trait objects stay as
    ///   provided (`principal_id` typically `None` until a future overlay).
    /// - Fails on duplicate id.
    pub fn register(&mut self, connector: Box<dyn Connector>) -> Result<(), RegistryError> {
        let manifest = connector.manifest();
        validate_manifest(manifest)?;
        if manifest.sandbox != SandboxMode::TrustedBuiltin {
            return Err(RegistryError::SandboxNotAllowed);
        }
        let id = manifest.id.clone();
        if self.entries.contains_key(&id) {
            return Err(RegistryError::DuplicateId(id));
        }
        let principal_id = principal_id_for_connector(&id);
        let mut bound = manifest.clone();
        bound.principal_id = Some(principal_id);
        self.entries.insert(
            id,
            Entry {
                connector,
                bound_manifest: bound,
            },
        );
        Ok(())
    }

    /// Resolve a registered connector by id.
    ///
    /// For policy principal binding, use [`Self::get_manifest`] (or
    /// [`Self::list_manifests`]) — **not** `get(...).manifest().principal_id`,
    /// which reflects the implementor's pre-register view.
    pub fn get(&self, id: &str) -> Option<&dyn Connector> {
        self.entries.get(id).map(|e| e.connector.as_ref())
    }

    /// Bound manifest (includes `principal_id`) for a registered connector.
    ///
    /// This is the authoritative registered view for Connector-kind policy.
    pub fn get_manifest(&self, id: &str) -> Option<&ConnectorManifest> {
        self.entries.get(id).map(|e| &e.bound_manifest)
    }

    /// All bound manifests, sorted by connector id (BTreeMap order).
    ///
    /// Each entry includes the registry-bound `principal_id`.
    pub fn list_manifests(&self) -> Vec<&ConnectorManifest> {
        self.entries.values().map(|e| &e.bound_manifest).collect()
    }

    /// Number of registered connectors.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod unit_tests {
    use super::*;
    use ai_brains_core::source::SourceKind;

    use crate::connector::{
        Connector, ConnectorContext, ConnectorError, ObservePayload, Preview, SourceHandle,
        WriteProposal, WriteProposalInput,
    };
    use crate::manifest::{
        ConnectorManifest, ConnectorOperations, ConnectorTrustLabel, CredentialDeclaration,
        FreshnessMechanism, MANIFEST_SCHEMA_VERSION, ScopeClass,
    };

    /// Minimal connector with a fixed manifest for registry unit tests.
    struct FakeConnector {
        manifest: ConnectorManifest,
    }

    impl Connector for FakeConnector {
        fn manifest(&self) -> &ConnectorManifest {
            &self.manifest
        }

        fn list(&self, _ctx: &ConnectorContext) -> Result<Vec<SourceHandle>, ConnectorError> {
            Err(ConnectorError::OperationNotSupported { operation: "list" })
        }

        fn observe(
            &self,
            _ctx: &ConnectorContext,
            _handle: &SourceHandle,
        ) -> Result<ObservePayload, ConnectorError> {
            Err(ConnectorError::OperationNotSupported {
                operation: "observe",
            })
        }

        fn preview(
            &self,
            _ctx: &ConnectorContext,
            _handle: &SourceHandle,
        ) -> Result<Preview, ConnectorError> {
            Err(ConnectorError::OperationNotSupported {
                operation: "preview",
            })
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

    fn fake_manifest(sandbox: SandboxMode) -> ConnectorManifest {
        ConnectorManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: "test.untrusted.placeholder".into(),
            display_name: "Test Untrusted Placeholder".into(),
            connector_version: "0.0.0".into(),
            source_kinds: vec![SourceKind::File],
            operations: ConnectorOperations::default(),
            scope_affinity: vec![ScopeClass::Personal],
            freshness: FreshnessMechanism::Fingerprint,
            credentials: CredentialDeclaration::None,
            sandbox,
            default_trust: ConnectorTrustLabel::LocalOnly,
            principal_id: None,
        }
    }

    /// Layer 2 / T153 R1-06: registry refuses non-TrustedBuiltin via test-only construct.
    #[test]
    fn register__TestUntrustedPlaceholder_sandbox__SandboxNotAllowed() {
        let connector = FakeConnector {
            manifest: fake_manifest(SandboxMode::TestUntrustedPlaceholder),
        };
        let mut registry = InProcessConnectorRegistry::new();
        let err = match registry.register(Box::new(connector)) {
            Err(e) => e,
            Ok(()) => panic!("non-TrustedBuiltin must be refused"),
        };
        assert_eq!(err, RegistryError::SandboxNotAllowed);
        assert!(registry.is_empty());
    }

    #[test]
    fn register__TrustedBuiltin_sandbox__ok() {
        let connector = FakeConnector {
            manifest: fake_manifest(SandboxMode::TrustedBuiltin),
        };
        let mut registry = InProcessConnectorRegistry::new();
        assert_eq!(
            registry.register(Box::new(connector)),
            Ok(()),
            "TrustedBuiltin must register"
        );
        assert_eq!(registry.len(), 1);
        let bound = match registry.get_manifest("test.untrusted.placeholder") {
            Some(m) => m,
            None => panic!("bound manifest present"),
        };
        assert!(bound.principal_id.is_some());
        assert_eq!(bound.sandbox, SandboxMode::TrustedBuiltin);
    }
}
