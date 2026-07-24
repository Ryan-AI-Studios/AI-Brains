use serde::{Deserialize, Serialize};

/// Freshness is **not** recency alone: source alignment drives staleness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FreshnessState {
    Current,
    RevalidationDue,
    Stale,
    SourceUnavailable,
    Unknown,
}

/// Inputs for evaluating freshness independent of wall-clock age alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreshnessInputs {
    /// Age in seconds (informational; does not alone force Stale).
    pub age_secs: u64,
    /// True when the derived item still matches the current source version fingerprint.
    pub source_aligned: bool,
    /// True when the source cannot be reached / has been marked unavailable.
    pub source_unavailable: bool,
    /// Policy threshold for suggesting revalidation without marking fully Stale.
    pub revalidation_age_secs: u64,
}

/// Compute freshness: old-but-aligned stays Current (or RevalidationDue by age);
/// young-but-misaligned becomes Stale / RevalidationDue.
pub fn evaluate_freshness(inputs: FreshnessInputs) -> FreshnessState {
    if inputs.source_unavailable {
        return FreshnessState::SourceUnavailable;
    }
    if !inputs.source_aligned {
        // Misalignment is staleness regardless of age.
        return FreshnessState::Stale;
    }
    if inputs.age_secs >= inputs.revalidation_age_secs {
        return FreshnessState::RevalidationDue;
    }
    FreshnessState::Current
}
