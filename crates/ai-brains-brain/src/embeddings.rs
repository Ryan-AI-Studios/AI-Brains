use ai_brains_core::ids::MemoryId;
use ai_brains_core::model_provenance::cloud_route_allowed;
use ai_brains_core::privacy::Privacy;
use ai_brains_models::registry::allow_cloud_extraction_from_env;
use ai_brains_models::{EmbeddingRequest, ModelProvider};
use ai_brains_store::QueryStore;
use std::str::FromStr;
use std::sync::Arc;

/// Service for generating and storing embeddings for vault memories.
pub struct EmbeddingService {
    query_store: Arc<dyn QueryStore>,
    embedding_provider: Arc<dyn ModelProvider>,
}

impl EmbeddingService {
    pub fn new(
        query_store: Arc<dyn QueryStore>,
        embedding_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            query_store,
            embedding_provider,
        }
    }

    /// Resolve memory privacy for routing. Fail closed to [`Privacy::Sealed`]
    /// (strictest) when the id cannot be parsed, lookup errors, or privacy is unknown.
    fn resolve_memory_privacy(&self, memory_id: &str) -> Privacy {
        let Ok(id) = MemoryId::from_str(memory_id) else {
            return Privacy::Sealed;
        };
        match self.query_store.get_memory_privacy(&id) {
            Ok(Some(p)) => p,
            Ok(None) | Err(_) => Privacy::Sealed,
        }
    }

    /// Generate and store embedding for a single memory.
    ///
    /// Non-fatal for model/store failures and policy denials: never propagates those as Err.
    /// Returns `Ok(true)` when an embedding was stored; `Ok(false)` when skipped
    /// (cloud-policy gate) or soft-failed (model/store error). Hard setup errors still Err.
    ///
    /// Cloud-policy: before any `embed()` call, privacy is resolved (fail-closed Sealed)
    /// and gated via [`cloud_route_allowed`]. On denial the model is not called and
    /// nothing is stored; backfill treats this as failed/skipped (Ok(false)).
    pub async fn generate_and_store(
        &self,
        memory_id: &str,
        content: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let privacy = self.resolve_memory_privacy(memory_id);
        let allow_cloud = allow_cloud_extraction_from_env();
        if let Err(denial) =
            cloud_route_allowed(privacy, self.embedding_provider.is_local(), allow_cloud)
        {
            // reason_code only — never prompt body / Privacy Debug / API keys
            tracing::warn!(
                reason_code = %denial.reason_code,
                memory_id = %memory_id,
                "skipping embedding: cloud-policy gate denied"
            );
            // Policy skip: no model call, no store. Count as failed/skipped in backfill.
            return Ok(false);
        }

        let text = if content.len() > 4000 {
            &content[..4000]
        } else {
            content
        };

        let request = EmbeddingRequest {
            text: text.to_string(),
        };

        match self.embedding_provider.embed(request).await {
            Ok(response) => {
                let bytes = f32_vec_to_bytes(&response.vector);
                if let Err(e) = self.query_store.store_embedding(memory_id, &bytes) {
                    tracing::warn!("Failed to store embedding for memory {}: {}", memory_id, e);
                    Ok(false)
                } else {
                    tracing::info!("Stored embedding for memory {}", memory_id);
                    Ok(true)
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to generate embedding for memory {}: {}",
                    memory_id,
                    e
                );
                Ok(false)
            }
        }
    }

    /// Backfill embeddings for recent memories without embeddings.
    ///
    /// Policy-denied memories count as `failed` (no model call). Soft model/store
    /// failures also increment `failed`. Only successful stores increment `success`.
    pub async fn backfill_recent(
        &self,
        limit: usize,
        since_days: Option<i32>,
    ) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let memories = self
            .query_store
            .get_memories_without_embeddings(limit, since_days)?;

        if memories.is_empty() {
            return Ok((0, 0));
        }

        eprintln!(
            "[Nightly] Backfilling embeddings for {} memories...",
            memories.len()
        );

        let mut success = 0;
        let mut failed = 0;

        for (memory_id, content) in memories {
            match self.generate_and_store(&memory_id, &content).await {
                Ok(true) => success += 1,
                Ok(false) | Err(_) => failed += 1,
            }

            // Brief yield to avoid overwhelming the embedding server
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        eprintln!(
            "[Nightly] Embedding backfill complete: {} succeeded, {} failed.",
            success, failed
        );
        Ok((success, failed))
    }

    /// Refresh stale embeddings (older than threshold)
    pub async fn refresh_stale(
        &self,
        days_threshold: i32,
        limit: usize,
    ) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let memories = self.query_store.get_stale_memories(days_threshold, limit)?;

        if memories.is_empty() {
            return Ok((0, 0));
        }

        eprintln!(
            "[Nightly] Refreshing {} stale embeddings (>{} days old)...",
            memories.len(),
            days_threshold
        );

        let mut success = 0;
        let mut failed = 0;

        for (idx, (memory_id, content)) in memories.iter().enumerate() {
            eprintln!(
                "    [Stale {}/{}] Re-embedding memory {}...",
                idx + 1,
                memories.len(),
                &memory_id[..8.min(memory_id.len())]
            );

            match self.generate_and_store(memory_id, content).await {
                Ok(true) => success += 1,
                Ok(false) | Err(_) => failed += 1,
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        eprintln!(
            "[Nightly] Stale refresh complete: {} succeeded, {} failed.",
            success, failed
        );
        Ok((success, failed))
    }
}

fn f32_vec_to_bytes(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for &v in vec {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}
