use ai_brains_core::ids::MemoryId;
use ai_brains_core::model_provenance::cloud_route_allowed;
use ai_brains_core::privacy::Privacy;
use ai_brains_models::registry::allow_cloud_extraction_from_env;
use ai_brains_models::{EmbeddingRequest, ModelProvider};
use ai_brains_store::QueryStore;
use std::str::FromStr;
use std::sync::Arc;

use crate::deadline::NightlyDeadline;

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

        // T229 F5: char-boundary safe truncate (never panic on multi-byte UTF-8).
        let text = truncate_for_embed(content);

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

    /// Deadline-bound keyset catch-up of pinned NULL embeddings (T338 F2).
    ///
    /// Pages `(updated_at DESC, memory_id DESC)`. After each chunk, the next page
    /// starts strictly after the last **fetched** cursor (including failures).
    /// Keeps the 50ms yield. Chunk size is `chunk_env` unless remaining time is
    /// under 60s, then 50.
    pub async fn backfill_catchup(
        &self,
        deadline: &NightlyDeadline,
        chunk_env: usize,
    ) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let mut success = 0;
        let mut failed = 0;
        let mut after: Option<(String, String)> = None;

        loop {
            if deadline.expired() {
                break;
            }
            let remaining = deadline.remaining_secs();
            let limit = if remaining < 60 { 50 } else { chunk_env };
            let cursor = after
                .as_ref()
                .map(|(updated_at, memory_id)| (updated_at.as_str(), memory_id.as_str()));
            let page = self
                .query_store
                .page_pinned_without_embeddings(limit, cursor)?;
            if page.is_empty() {
                break;
            }
            let last = page
                .last()
                .map(|row| (row.updated_at.clone(), row.memory_id.clone()));
            eprintln!(
                "[Nightly] Backfilling embeddings for {} memories (keyset page)...",
                page.len()
            );
            for row in page {
                match self.generate_and_store(&row.memory_id, &row.content).await {
                    Ok(true) => success += 1,
                    Ok(false) | Err(_) => failed += 1,
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
            after = last;
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

/// Max UTF-8 bytes sent to the embedding model (T229 F5).
const EMBED_TEXT_MAX_BYTES: usize = 4000;

/// Truncate embedding text at a valid UTF-8 char boundary.
///
/// Never panics: byte index 4000 may land mid multi-byte character (CJK/emoji).
/// Uses `str::floor_char_boundary` so the returned slice is always valid UTF-8
/// and at most [`EMBED_TEXT_MAX_BYTES`] long.
pub(crate) fn truncate_for_embed(content: &str) -> &str {
    let end = content.floor_char_boundary(EMBED_TEXT_MAX_BYTES.min(content.len()));
    &content[..end]
}

fn f32_vec_to_bytes(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for &v in vec {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    /// AC1: multi-byte content where byte 4000 is mid-character → valid UTF-8, len ≤ 4000, no panic.
    #[test]
    fn truncate_for_embed__multi_byte_at_4000__valid_utf8_no_panic() {
        // "你" is 3 bytes in UTF-8. Build a string whose length is just past 4000 mid-char.
        let cjk = "你"; // 3 bytes
        let mut s = String::new();
        while s.len() < 4000 {
            s.push_str(cjk);
        }
        // s.len() is multiple of 3 ≥ 4000. If 4000 is mid-char, floor_char_boundary retreats.
        // Force mid-char: prefix of 3999 bytes of ASCII then a 3-byte CJK so byte 4000 is mid-char.
        let mut mid = "a".repeat(3999);
        mid.push_str(cjk); // bytes 3999..4002 are the CJK char; index 4000 is mid-char
        assert!(mid.len() > 4000);
        assert!(!mid.is_char_boundary(4000));

        let out = truncate_for_embed(&mid);
        assert!(out.is_char_boundary(out.len()));
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
        assert!(out.len() <= EMBED_TEXT_MAX_BYTES);
        assert_eq!(out.len(), 3999); // retreated to before the CJK
        assert_eq!(out, "a".repeat(3999));

        // Also cover pure CJK packing past 4000 (boundary alignment).
        let out_cjk = truncate_for_embed(&s);
        assert!(out_cjk.len() <= EMBED_TEXT_MAX_BYTES);
        assert!(std::str::from_utf8(out_cjk.as_bytes()).is_ok());
        assert!(out_cjk.is_char_boundary(out_cjk.len()));
    }

    /// AC1 emoji: 4-byte char straddling the 4000 boundary.
    #[test]
    fn truncate_for_embed__emoji_straddling_4000__valid_utf8_no_panic() {
        let emoji = "😀"; // 4 bytes (U+1F600)
        assert_eq!(emoji.len(), 4);
        let mut s = "b".repeat(3998);
        s.push_str(emoji); // bytes 3998..4002; index 4000 is mid-emoji
        assert!(!s.is_char_boundary(4000));

        let out = truncate_for_embed(&s);
        assert!(out.is_char_boundary(out.len()));
        assert!(out.len() <= EMBED_TEXT_MAX_BYTES);
        assert_eq!(out.len(), 3998);
        assert_eq!(out, "b".repeat(3998));
    }

    /// AC2: short ASCII unchanged; long ASCII → exactly 4000 bytes.
    #[test]
    fn truncate_for_embed__ascii__short_unchanged_long_exactly_4000() {
        let short = "hello embed";
        assert_eq!(truncate_for_embed(short), short);

        let long = "x".repeat(5000);
        let out = truncate_for_embed(&long);
        assert_eq!(out.len(), EMBED_TEXT_MAX_BYTES);
        assert_eq!(out, "x".repeat(4000));
    }

    /// AC2 edge: exactly 4000 ASCII bytes unchanged.
    #[test]
    fn truncate_for_embed__exactly_4000_ascii__unchanged() {
        let exact = "y".repeat(4000);
        let out = truncate_for_embed(&exact);
        assert_eq!(out.len(), 4000);
        assert_eq!(out, exact);
    }

    /// F5 smart-quote (U+201C is 3 bytes) straddling byte 4000 — same panic class as CJK.
    #[test]
    fn truncate_for_embed__smart_quote_straddling_4000__valid_utf8_no_panic() {
        let quote = "\u{201C}"; // “ LEFT DOUBLE QUOTATION MARK — 3 UTF-8 bytes
        assert_eq!(quote.len(), 3);
        let mut s = "c".repeat(3999);
        s.push_str(quote);
        assert!(!s.is_char_boundary(4000));

        let out = truncate_for_embed(&s);
        assert!(out.is_char_boundary(out.len()));
        assert!(out.len() <= EMBED_TEXT_MAX_BYTES);
        assert_eq!(out.len(), 3999);
        assert_eq!(out, "c".repeat(3999));
    }

    /// AC9: sole production call site is `truncate_for_embed` in `generate_and_store`.
    /// Helper units prove the panic class; production section must not reintroduce
    /// the raw mid-UTF8 slice.
    #[test]
    fn truncate_for_embed__module_has_no_raw_byte_slice_4000() {
        let src = include_str!("embeddings.rs");
        let production = src.split("#[cfg(test)]").next().unwrap_or(src);
        assert!(
            !production.contains("&content[..4000]"),
            "raw mid-UTF8 slice residual must not return in production code"
        );
        assert!(
            production.contains("truncate_for_embed(content)"),
            "generate_and_store must call truncate_for_embed"
        );
    }
}
