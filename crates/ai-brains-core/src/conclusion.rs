use crate::errors::{Error, Result};
use crate::ids::PrincipalId;
use serde::{Deserialize, Serialize};

/// Epistemic state of a conclusion (ADR-0011).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ConclusionState {
    Candidate,
    Active,
    Confirmed,
    Stale,
    Disputed,
    Superseded,
    Rejected,
}

/// Human (or authorized principal) approval for gated transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApprovalAuthority {
    pub principal_id: PrincipalId,
}

/// Proof that source material was revalidated before leaving Stale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevalidationProof {
    pub token: String,
}

impl ConclusionState {
    /// Pure transition rules. Gated paths require authority/proof arguments.
    pub fn transition(
        &self,
        to: ConclusionState,
        approval: Option<ApprovalAuthority>,
        revalidation: Option<&RevalidationProof>,
    ) -> Result<ConclusionState> {
        match (*self, to) {
            // Non-protected activation path
            (ConclusionState::Candidate, ConclusionState::Active) => Ok(to),
            // Confirmed requires human approval
            (ConclusionState::Candidate, ConclusionState::Confirmed)
            | (ConclusionState::Active, ConclusionState::Confirmed) => {
                if approval.is_some() {
                    Ok(to)
                } else {
                    Err(Error::ApprovalRequired {
                        transition: format!("{self:?} -> {to:?}"),
                    })
                }
            }
            // Stale → Active requires revalidation proof
            (ConclusionState::Stale, ConclusionState::Active) => {
                if revalidation.is_some() {
                    Ok(to)
                } else {
                    Err(Error::RevalidationRequired {
                        from: format!("{self:?}"),
                        to: format!("{to:?}"),
                    })
                }
            }
            (ConclusionState::Active, ConclusionState::Stale)
            | (ConclusionState::Confirmed, ConclusionState::Stale)
            | (ConclusionState::Candidate, ConclusionState::Rejected)
            | (ConclusionState::Active, ConclusionState::Rejected)
            | (ConclusionState::Candidate, ConclusionState::Disputed)
            | (ConclusionState::Active, ConclusionState::Disputed)
            | (ConclusionState::Confirmed, ConclusionState::Disputed)
            | (ConclusionState::Active, ConclusionState::Superseded)
            | (ConclusionState::Confirmed, ConclusionState::Superseded)
            | (ConclusionState::Stale, ConclusionState::Superseded)
            | (ConclusionState::Disputed, ConclusionState::Superseded)
            | (ConclusionState::Disputed, ConclusionState::Rejected) => Ok(to),
            _ => Err(Error::InvalidStatusTransition {
                from: format!("{self:?}"),
                to: format!("{to:?}"),
            }),
        }
    }
}
