use crate::ids::PrincipalId;
use crate::scope::GrantCapability;
use crate::source::SourceKind;
use serde::{Deserialize, Serialize};

/// Kind of principal that may act on governed memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PrincipalKind {
    Human,
    Agent,
    Connector,
    System,
    /// Legacy kind retained for historical payloads and fixtures.
    Service,
    Other(String),
}

/// Shell identity for event/DTO layers (not a full IAM model).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Principal {
    pub id: PrincipalId,
    pub kind: PrincipalKind,
    pub display_name: String,
    /// Source kinds this principal is bound to.
    ///
    /// Empty means **no** source-kind bindings (deny-by-default for Connector observe
    /// checks). This is not an "unrestricted shell".
    #[serde(default)]
    pub bound_source_kinds: Vec<SourceKind>,
    /// Capabilities this principal is bound to (System matrix).
    ///
    /// Empty means **no** bound capabilities (System allows only via explicit grants).
    /// Non-empty: capability must be in this set **and** have a grant (least privilege).
    /// This is not an "unrestricted shell".
    #[serde(default)]
    pub bound_capabilities: Vec<GrantCapability>,
}
