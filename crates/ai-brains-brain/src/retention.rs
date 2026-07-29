//! Nightly raw-turn projection cleanup (legacy path).
//!
//! Class-based plan/apply lives in `ai-brains-control-plane::class_based_retention`.
//! This service **only** runs stream-A raw-turn projection delete and **never**
//! performs CE bulk unless explicitly documented elsewhere with opt-in (R7).

use ai_brains_store::QueryStore;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct RetentionService {
    query_store: Arc<dyn QueryStore>,
    retention_days: i64,
}

impl RetentionService {
    pub fn new(query_store: Arc<dyn QueryStore>, retention_days: i64) -> Self {
        Self {
            query_store,
            retention_days,
        }
    }

    /// Resolve raw-turn horizon from env `AI_BRAINS_RETENTION_RAW_TURN_DAYS`, else `default_days`.
    pub fn days_from_env(default_days: i64) -> i64 {
        std::env::var("AI_BRAINS_RETENTION_RAW_TURN_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|d| *d > 0)
            .unwrap_or(default_days)
    }

    /// R7: nightly must not auto-execute CE bulk without opt-in.
    ///
    /// True only when `AI_BRAINS_RETENTION_APPLY_CE=1` (or true/yes) or
    /// `AI_BRAINS_RETENTION_APPLY_CE_ON_NIGHTLY` is set similarly.
    pub fn apply_ce_on_nightly_from_env() -> bool {
        env_truthy("AI_BRAINS_RETENTION_APPLY_CE")
            || env_truthy("AI_BRAINS_RETENTION_APPLY_CE_ON_NIGHTLY")
    }

    /// Projection-only raw turn cleanup. Never calls CE wipe (R2/R7).
    pub async fn run_cleanup(&self) -> Result<usize, Box<dyn std::error::Error>> {
        if Self::apply_ce_on_nightly_from_env() {
            // Opt-in flag is recognized but nightly still does not run CE in this
            // service — class CE apply is operator-driven (`retention apply --confirm`).
            warn!(
                "AI_BRAINS_RETENTION_APPLY_CE is set; nightly still runs projection-only raw-turn cleanup (use `ai-brains retention apply --confirm` for class CE)"
            );
        } else {
            info!(
                "Nightly CE bulk disabled (default; set AI_BRAINS_RETENTION_APPLY_CE=1 only as operator opt-in documentation — apply remains confirm-gated CLI)"
            );
        }

        info!(
            "Starting raw turn retention cleanup ({} days, projection_delete only)...",
            self.retention_days
        );

        let cutoff = Utc::now() - Duration::days(self.retention_days);
        match self.query_store.delete_old_turns(cutoff) {
            Ok(count) => {
                info!(
                    "Cleaned up {} expired turns (not cryptographic erasure).",
                    count
                );
                Ok(count)
            }
            Err(e) => {
                error!("Retention cleanup failed: {}", e);
                Err(e.into())
            }
        }
    }
}

fn env_truthy(key: &str) -> bool {
    match std::env::var(key) {
        Ok(s) => {
            let t = s.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn nightly_default__no_ce_without_opt_in() {
        // Default path: without env, CE must be false (R7).
        // We cannot safely mutate process-global env in parallel tests without TempEnv;
        // assert the pure default helper semantics via apply_ce_on_nightly_from_env when unset.
        // If CI sets the opt-in, this still documents the intended default.
        let enabled = RetentionService::apply_ce_on_nightly_from_env();
        if std::env::var("AI_BRAINS_RETENTION_APPLY_CE").is_err()
            && std::env::var("AI_BRAINS_RETENTION_APPLY_CE_ON_NIGHTLY").is_err()
        {
            assert!(!enabled, "R7: nightly CE must default off when env unset");
        }
    }

    #[test]
    fn days_from_env__default_when_unset() {
        if std::env::var("AI_BRAINS_RETENTION_RAW_TURN_DAYS").is_err() {
            assert_eq!(RetentionService::days_from_env(90), 90);
        }
    }
}
