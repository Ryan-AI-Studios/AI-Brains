use crate::ids::{BriefingId, EvidenceId};
use serde::{Deserialize, Serialize};

/// Briefing kind shell (full packet builder is a later phase).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BriefingKind {
    Preflight,
    Session,
    Review,
    Other(String),
}

/// Handle-oriented briefing summary — evidence by id, not prose-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BriefingHandle {
    pub id: BriefingId,
    pub kind: BriefingKind,
    pub evidence_ids: Vec<EvidenceId>,
}
