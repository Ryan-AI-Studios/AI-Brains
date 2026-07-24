use crate::ids::{ProjectId, UserId, WorkspaceId};
use crate::privacy::Privacy;
use serde::{Deserialize, Serialize};

/// Reference to a governed scope boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ScopeRef {
    Repository(ProjectId),
    Workspace(WorkspaceId),
    Personal(UserId),
}

/// Capability bits for scope grants (strictest-wins when combining).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum GrantCapability {
    ReadEvidence = 0,
    ReadConclusions = 1,
    ReadDecisions = 2,
    ProposeConclusion = 3,
    ApproveConclusion = 4,
    ProposeDecision = 5,
    ApproveDecision = 6,
    Export = 7,
    Erase = 8,
}

/// A principal's grant at a scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeGrant {
    pub scope: ScopeRef,
    pub capability: GrantCapability,
    pub privacy: Privacy,
}

/// When two grants apply to the same principal+scope, keep the **stricter** privacy
/// and the **narrower** (lower privilege ordinal) capability.
pub fn strictest_wins(a: &ScopeGrant, b: &ScopeGrant) -> ScopeGrant {
    let privacy = a.privacy.combine(b.privacy);
    // Lower ordinal = less powerful capability wins under "strictest".
    let capability = if a.capability <= b.capability {
        a.capability
    } else {
        b.capability
    };
    // Prefer left scope identity when equal; caller should only combine same-scope grants.
    ScopeGrant {
        scope: a.scope.clone(),
        capability,
        privacy,
    }
}
