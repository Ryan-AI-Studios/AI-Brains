//! Connector capability manifest (schema v1).
//!
//! Built-in source readers declare identity, supported kinds/ops, scope affinity,
//! freshness, credential *declarations* (never secrets), sandbox mode, and
//! default trust. See track T153.

use ai_brains_core::ids::PrincipalId;
use ai_brains_core::source::SourceKind;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Only schema version accepted by this crate.
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Wire / in-process capability declaration for a connector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorManifest {
    /// Must be [`MANIFEST_SCHEMA_VERSION`] (`1`). Higher versions are rejected.
    pub schema_version: u32,
    /// Stable connector id (e.g. `builtin.mock`).
    pub id: String,
    pub display_name: String,
    /// Connector implementation version (not the schema version).
    pub connector_version: String,
    pub source_kinds: Vec<SourceKind>,
    pub operations: ConnectorOperations,
    pub scope_affinity: Vec<ScopeClass>,
    pub freshness: FreshnessMechanism,
    pub credentials: CredentialDeclaration,
    pub sandbox: SandboxMode,
    pub default_trust: ConnectorTrustLabel,
    /// Bound when registered in [`crate::InProcessConnectorRegistry`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<PrincipalId>,
}

/// Supported connector operations (flags must match runtime behavior).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConnectorOperations {
    pub list: bool,
    pub observe: bool,
    pub preview: bool,
    pub propose_write: bool,
}

/// Scope classes a connector may touch (affinity declaration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ScopeClass {
    Personal,
    Repository,
    Workspace,
}

/// How change is detected for freshness.
///
/// v1 only supports fingerprint-based detection (existing sources algorithms).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FreshnessMechanism {
    Fingerprint,
}

/// Declared credential *requirements* — never stores secrets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum CredentialDeclaration {
    /// No credentials required.
    None,
    /// Filesystem path access only (local vault / repo roots).
    PathAccess,
}

/// Sandbox posture for connector execution.
///
/// Production registries accept only [`SandboxMode::TrustedBuiltin`].
/// WASI / subprocess host modes are reserved per **ADR-0019** (Trusted
/// Built-ins First) — not product/serde variants until a host lands.
///
/// Two-layer defense (ADR-0019 L9): (1) serde fails closed on unknown
/// `sandbox` strings; (2) registry refuses non-`TrustedBuiltin`
/// (`RegistryError::SandboxNotAllowed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub enum SandboxMode {
    /// In-process built-in; still subject to policy (T151).
    TrustedBuiltin,

    /// Test-only constructible mode for registry denial coverage (T182 / T153 R1-06).
    /// Not a production host variant. Never serialize as a real product mode.
    #[cfg(test)]
    #[doc(hidden)]
    TestUntrustedPlaceholder,
}

/// Trust default declared by the connector (DTO parity with control-plane
/// `ConnectorTrust`: `LocalOnly` / `CloudOk` / `Unknown`).
///
/// Kept in `ai-brains-sources` so sources does not depend on control-plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ConnectorTrustLabel {
    LocalOnly,
    CloudOk,
    Unknown,
}

impl ConnectorTrustLabel {
    /// String label matching control-plane `ConnectorTrust` variant names.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalOnly => "LocalOnly",
            Self::CloudOk => "CloudOk",
            Self::Unknown => "Unknown",
        }
    }
}

/// Manifest parse / validation errors.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ManifestError {
    #[error("unsupported manifest schema_version {found}; only {expected} is accepted")]
    UnsupportedSchemaVersion { found: u32, expected: u32 },

    #[error("manifest source_kinds must be non-empty")]
    EmptySourceKinds,

    #[error("manifest id must be non-empty")]
    EmptyId,

    #[error("manifest display_name must be non-empty")]
    EmptyDisplayName,

    #[error("manifest connector_version must be non-empty")]
    EmptyConnectorVersion,

    #[error("manifest JSON error: {0}")]
    Json(String),
}

/// Validate a deserialized [`ConnectorManifest`].
///
/// Rejects `schema_version != 1`, empty identity fields, and empty `source_kinds`.
pub fn validate_manifest(manifest: &ConnectorManifest) -> Result<(), ManifestError> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(ManifestError::UnsupportedSchemaVersion {
            found: manifest.schema_version,
            expected: MANIFEST_SCHEMA_VERSION,
        });
    }
    if manifest.id.trim().is_empty() {
        return Err(ManifestError::EmptyId);
    }
    if manifest.display_name.trim().is_empty() {
        return Err(ManifestError::EmptyDisplayName);
    }
    if manifest.connector_version.trim().is_empty() {
        return Err(ManifestError::EmptyConnectorVersion);
    }
    if manifest.source_kinds.is_empty() {
        return Err(ManifestError::EmptySourceKinds);
    }
    Ok(())
}

/// Deserialize and validate a schema_v1 manifest from JSON bytes.
pub fn parse_manifest_json(bytes: &[u8]) -> Result<ConnectorManifest, ManifestError> {
    let manifest: ConnectorManifest =
        serde_json::from_slice(bytes).map_err(|e| ManifestError::Json(e.to_string()))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

/// Deserialize and validate a schema_v1 manifest from a JSON string.
pub fn parse_manifest_str(s: &str) -> Result<ConnectorManifest, ManifestError> {
    parse_manifest_json(s.as_bytes())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod unit_tests {
    use super::*;

    #[test]
    fn validate_manifest__schema_v1__ok() {
        let m = ConnectorManifest {
            schema_version: 1,
            id: "builtin.mock".into(),
            display_name: "Mock".into(),
            connector_version: "0.1.0".into(),
            source_kinds: vec![SourceKind::File],
            operations: ConnectorOperations {
                list: true,
                observe: true,
                preview: false,
                propose_write: false,
            },
            scope_affinity: vec![ScopeClass::Personal],
            freshness: FreshnessMechanism::Fingerprint,
            credentials: CredentialDeclaration::None,
            sandbox: SandboxMode::TrustedBuiltin,
            default_trust: ConnectorTrustLabel::LocalOnly,
            principal_id: None,
        };
        assert_eq!(validate_manifest(&m), Ok(()));
    }

    #[test]
    fn validate_manifest__schema_v99__rejected() {
        let mut m = ConnectorManifest {
            schema_version: 99,
            id: "x".into(),
            display_name: "X".into(),
            connector_version: "0.0.1".into(),
            source_kinds: vec![SourceKind::File],
            operations: ConnectorOperations::default(),
            scope_affinity: vec![],
            freshness: FreshnessMechanism::Fingerprint,
            credentials: CredentialDeclaration::None,
            sandbox: SandboxMode::TrustedBuiltin,
            default_trust: ConnectorTrustLabel::Unknown,
            principal_id: None,
        };
        assert!(matches!(
            validate_manifest(&m),
            Err(ManifestError::UnsupportedSchemaVersion {
                found: 99,
                expected: 1
            })
        ));
        m.schema_version = 1;
        m.source_kinds.clear();
        assert_eq!(validate_manifest(&m), Err(ManifestError::EmptySourceKinds));
    }

    /// Layer 1 (ADR-0019 L9): unknown sandbox strings fail at serde/parse.
    /// Must **not** surface as `RegistryError::SandboxNotAllowed` (never reaches registry).
    fn minimal_manifest_json_with_sandbox(sandbox: &str) -> String {
        format!(
            r#"{{
              "schema_version": 1,
              "id": "builtin.mock",
              "display_name": "Mock",
              "connector_version": "0.1.0",
              "source_kinds": ["File"],
              "operations": {{
                "list": true,
                "observe": true,
                "preview": false,
                "propose_write": false
              }},
              "scope_affinity": ["Personal"],
              "freshness": "Fingerprint",
              "credentials": "None",
              "sandbox": "{sandbox}",
              "default_trust": "LocalOnly"
            }}"#
        )
    }

    #[test]
    fn parse_manifest_str__sandbox_Subprocess__serde_fail_closed() {
        let err = match parse_manifest_str(&minimal_manifest_json_with_sandbox("Subprocess")) {
            Err(e) => e,
            Ok(_) => panic!("unknown sandbox must fail at serde/parse"),
        };
        assert!(
            matches!(err, ManifestError::Json(_)),
            "expected ManifestError::Json for unknown sandbox, got {err:?}"
        );
        let msg = err.to_string();
        assert!(
            !msg.contains("SandboxNotAllowed"),
            "unknown sandbox must not claim SandboxNotAllowed (never reaches registry): {msg}"
        );
    }

    #[test]
    fn parse_manifest_str__sandbox_UntrustedExternal__serde_fail_closed() {
        let err = match parse_manifest_str(&minimal_manifest_json_with_sandbox("UntrustedExternal"))
        {
            Err(e) => e,
            Ok(_) => panic!("unknown sandbox must fail at serde/parse"),
        };
        assert!(
            matches!(err, ManifestError::Json(_)),
            "expected ManifestError::Json for unknown sandbox, got {err:?}"
        );
        let msg = err.to_string();
        assert!(
            !msg.contains("SandboxNotAllowed"),
            "unknown sandbox must not claim SandboxNotAllowed (never reaches registry): {msg}"
        );
    }
}
