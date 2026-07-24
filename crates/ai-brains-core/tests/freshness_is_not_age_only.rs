#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_core::freshness::{FreshnessInputs, FreshnessState, evaluate_freshness};

#[test]
fn old_but_source_aligned__current_or_revalidation_due_not_stale() {
    let old_aligned = FreshnessInputs {
        age_secs: 365 * 24 * 3600,
        source_aligned: true,
        source_unavailable: false,
        revalidation_age_secs: u64::MAX, // never due
    };
    assert_eq!(evaluate_freshness(old_aligned), FreshnessState::Current);

    let old_due = FreshnessInputs {
        age_secs: 90 * 24 * 3600,
        source_aligned: true,
        source_unavailable: false,
        revalidation_age_secs: 30 * 24 * 3600,
    };
    assert_eq!(evaluate_freshness(old_due), FreshnessState::RevalidationDue);
}

#[test]
fn young_but_source_changed__stale() {
    let young_misaligned = FreshnessInputs {
        age_secs: 60,
        source_aligned: false,
        source_unavailable: false,
        revalidation_age_secs: 30 * 24 * 3600,
    };
    assert_eq!(
        evaluate_freshness(young_misaligned),
        FreshnessState::Stale,
        "misalignment forces Stale regardless of recency"
    );
}

#[test]
fn source_unavailable__maps_to_source_unavailable() {
    let inputs = FreshnessInputs {
        age_secs: 10,
        source_aligned: true,
        source_unavailable: true,
        revalidation_age_secs: 1000,
    };
    assert_eq!(
        evaluate_freshness(inputs),
        FreshnessState::SourceUnavailable
    );
}
