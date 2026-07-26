//! Typed briefings (T152) — domain kinds, section budgets, and claim validation.
//!
//! Full API packet shapes live in `ai-brains-contracts`. This module owns
//! selection/validation rules shared by the control-plane briefing service.

use crate::ids::{BriefingId, EvidenceId};
use serde::{Deserialize, Serialize};

/// Briefing kind (packet builders use Project / Personal; shell keeps Preflight etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BriefingKind {
    Preflight,
    Session,
    Review,
    Project,
    Personal,
    Other(String),
}

/// Handle-oriented briefing summary — evidence by id, not prose-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BriefingHandle {
    pub id: BriefingId,
    pub kind: BriefingKind,
    pub evidence_ids: Vec<EvidenceId>,
}

/// Named sections that appear in Project or Personal briefing packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BriefingSection {
    Scope,
    Handoff,
    Decisions,
    Conclusions,
    Constraints,
    Warnings,
    Freshness,
    Ledgerful,
    EvidenceHandles,
    Budget,
    Preferences,
    Continuity,
    OpenReviewItems,
    GrantsApplied,
}

impl BriefingSection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scope => "scope",
            Self::Handoff => "handoff",
            Self::Decisions => "decisions",
            Self::Conclusions => "conclusions",
            Self::Constraints => "constraints",
            Self::Warnings => "warnings",
            Self::Freshness => "freshness",
            Self::Ledgerful => "ledgerful",
            Self::EvidenceHandles => "evidence_handles",
            Self::Budget => "budget",
            Self::Preferences => "preferences",
            Self::Continuity => "continuity",
            Self::OpenReviewItems => "open_review_items",
            Self::GrantsApplied => "grants_applied",
        }
    }
}

/// Budget metering for a generated briefing packet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetReport {
    /// Caller-supplied word budget (0 = unlimited for metering only).
    pub max_words: usize,
    pub used_words: usize,
    /// Sections dropped or truncated to stay under budget (stable order).
    pub truncated_sections: Vec<BriefingSection>,
    pub more_available: bool,
}

impl BudgetReport {
    pub fn unlimited(used_words: usize) -> Self {
        Self {
            max_words: 0,
            used_words,
            truncated_sections: Vec::new(),
            more_available: false,
        }
    }
}

/// Kind of authoritative claim that must carry evidence/decision handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AuthoritativeClaimKind {
    Decision,
    Conclusion,
}

/// A claim listed under current authority (Approved decision / Active|Confirmed conclusion).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoritativeClaimRef {
    pub kind: AuthoritativeClaimKind,
    /// Stable id string (decision_id or conclusion_id).
    pub id: String,
    /// At least one evidence or decision support handle is required.
    pub evidence_handles: Vec<String>,
}

/// Validation failure when an authoritative claim lacks evidence handles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimEvidenceError {
    pub kind: AuthoritativeClaimKind,
    pub id: String,
}

impl std::fmt::Display for ClaimEvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "authoritative {:?} claim {} has no evidence_handles",
            self.kind, self.id
        )
    }
}

impl std::error::Error for ClaimEvidenceError {}

/// Every authoritative decision/conclusion entry must have ≥1 evidence/decision handle.
pub fn validate_authoritative_claims(
    claims: &[AuthoritativeClaimRef],
) -> Result<(), ClaimEvidenceError> {
    for claim in claims {
        if claim.evidence_handles.is_empty() {
            return Err(ClaimEvidenceError {
                kind: claim.kind,
                id: claim.id.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn validate_authoritative_claims__with_handles__ok() {
        let claims = vec![AuthoritativeClaimRef {
            kind: AuthoritativeClaimKind::Decision,
            id: "d1".into(),
            evidence_handles: vec!["e1".into()],
        }];
        assert!(validate_authoritative_claims(&claims).is_ok());
    }

    #[test]
    fn validate_authoritative_claims__missing_handles__fails() {
        let claims = vec![AuthoritativeClaimRef {
            kind: AuthoritativeClaimKind::Conclusion,
            id: "c1".into(),
            evidence_handles: vec![],
        }];
        let err = validate_authoritative_claims(&claims).expect_err("must fail");
        assert_eq!(err.id, "c1");
        assert_eq!(err.kind, AuthoritativeClaimKind::Conclusion);
    }

    #[test]
    fn briefing_section__as_str__snake_case() {
        assert_eq!(
            BriefingSection::OpenReviewItems.as_str(),
            "open_review_items"
        );
    }
}
