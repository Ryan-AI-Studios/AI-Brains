use crate::{ModelError, ModelProvider, Result};
use ai_brains_core::privacy::Privacy;

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

    pub fn select_provider(&self, privacy: &Privacy) -> Result<&dyn ModelProvider> {
        for provider in &self.providers {
            if privacy == &Privacy::LocalOnly && !provider.is_local() {
                continue;
            }
            return Ok(provider.as_ref());
        }
        Err(ModelError::PrivacyViolation(format!(
            "No suitable provider found for privacy level {:?}",
            privacy
        )))
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
