use crate::{ModelError, ModelProvider, Result};
use ai_brains_core::model_provenance::{cloud_route_allowed, reason};
use ai_brains_core::privacy::Privacy;

/// Env flag: allow non-local model providers when privacy is CloudOk.
/// Values `1` / `true` / `TRUE` / `yes` (case-insensitive for true/yes) enable.
/// **Default: false** (cloud extraction disabled).
pub const ALLOW_CLOUD_EXTRACTION_ENV: &str = "AI_BRAINS_ALLOW_CLOUD_EXTRACTION";

pub struct ProviderRegistry {
    providers: Vec<Box<dyn ModelProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn ModelProvider>) {
        self.providers.push(provider);
    }

    /// Select a provider for the given privacy level.
    ///
    /// - Filters via [`cloud_route_allowed`] (local-strict privacy + allow_cloud flag).
    /// - **Local-first:** among viable providers, prefer `is_local() == true`, then
    ///   registration order within each locality group (ADR 0012).
    /// - Errors use stable reason codes only (no Privacy Debug / secrets).
    pub fn select_provider(&self, privacy: &Privacy) -> Result<&dyn ModelProvider> {
        let allow_cloud = allow_cloud_extraction_from_env();

        let mut viable_local: Vec<&dyn ModelProvider> = Vec::new();
        let mut viable_remote: Vec<&dyn ModelProvider> = Vec::new();
        let mut saw_privacy_mismatch = false;
        let mut saw_cloud_disabled = false;

        for provider in &self.providers {
            let is_local = provider.is_local();
            match cloud_route_allowed(*privacy, is_local, allow_cloud) {
                Ok(()) => {
                    if is_local {
                        viable_local.push(provider.as_ref());
                    } else {
                        viable_remote.push(provider.as_ref());
                    }
                }
                Err(denial) => {
                    if denial.reason_code == reason::PRIVACY_ROUTE_MISMATCH {
                        saw_privacy_mismatch = true;
                    } else if denial.reason_code == reason::CLOUD_EXTRACTION_DISABLED {
                        saw_cloud_disabled = true;
                    }
                }
            }
        }

        if let Some(p) = viable_local.first() {
            return Ok(*p);
        }
        if let Some(p) = viable_remote.first() {
            return Ok(*p);
        }

        // None viable — map to stable reason codes (no secrets / Privacy Debug dumps).
        // Prefer privacy_route_mismatch when local-strict privacy filtered remotes;
        // no_local_provider only when the registry is empty (nothing registered at all).
        let reason_code = if self.providers.is_empty() {
            reason::NO_LOCAL_PROVIDER
        } else if saw_privacy_mismatch {
            reason::PRIVACY_ROUTE_MISMATCH
        } else if saw_cloud_disabled {
            reason::CLOUD_EXTRACTION_DISABLED
        } else {
            reason::NO_LOCAL_PROVIDER
        };

        Err(ModelError::PrivacyViolation(reason_code.to_string()))
    }
}

/// Parse `AI_BRAINS_ALLOW_CLOUD_EXTRACTION`. Default **false**.
pub fn allow_cloud_extraction_from_env() -> bool {
    match std::env::var(ALLOW_CLOUD_EXTRACTION_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
