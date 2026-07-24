use crate::errors::{Error, Result};
use crate::ids::PrincipalId;
use serde::{Deserialize, Serialize};

/// Lifecycle state of a governed decision (distinct from legacy Memory-backed DecisionRecorded).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DecisionState {
    Proposed,
    Approved,
    Superseded,
    Revoked,
}

impl DecisionState {
    /// `Proposed → Approved` requires an approver principal (no silent auto-approve).
    pub fn transition(
        &self,
        to: DecisionState,
        approver: Option<PrincipalId>,
    ) -> Result<DecisionState> {
        match (*self, to) {
            (DecisionState::Proposed, DecisionState::Approved) => {
                if approver.is_some() {
                    Ok(to)
                } else {
                    Err(Error::ApprovalRequired {
                        transition: format!("{self:?} -> {to:?}"),
                    })
                }
            }
            (DecisionState::Proposed, DecisionState::Revoked)
            | (DecisionState::Approved, DecisionState::Superseded)
            | (DecisionState::Approved, DecisionState::Revoked)
            | (DecisionState::Proposed, DecisionState::Superseded) => Ok(to),
            _ => Err(Error::InvalidStatusTransition {
                from: format!("{self:?}"),
                to: format!("{to:?}"),
            }),
        }
    }
}
