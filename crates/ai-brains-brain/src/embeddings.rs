use ai_brains_core::ids::MemoryId;
use ai_brains_core::model_provenance::cloud_route_allowed;
use ai_brains_core::privacy::Privacy;
use ai_brains_models::registry::allow_cloud_extraction_from_env;
use ai_brains_models::{EmbeddingRequest, ModelProvider};
use ai_brains_store::QueryStore;
use std::str::FromStr;
use std::sync::Arc;

use crate::deadline::{MAX_EMBED_HTTP_BATCH, NightlyDeadline, parse_embed_http_batch_from_env};

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

        // T351 F1: chunk raw content (max 4×2048 scalars), then per-chunk T229 4000-byte cap.
        let (chunks, _truncated) = chunk_for_embed(content);
        match self.embed_chunks_to_blob(&chunks).await {
            Ok(Some(bytes)) => {
                if let Err(e) = self.query_store.store_embedding(memory_id, &bytes) {
                    tracing::warn!("Failed to store embedding for memory {}: {}", memory_id, e);
                    Ok(false)
                } else {
                    tracing::info!("Stored embedding for memory {}", memory_id);
                    Ok(true)
                }
            }
            Ok(None) => Ok(false),
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
    /// Third tuple is T351 truncations (5th+ scalar window or per-chunk 4000-byte cap).
    pub async fn backfill_recent(
        &self,
        limit: usize,
        since_days: Option<i32>,
    ) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
        let memories = self
            .query_store
            .get_memories_without_embeddings(limit, since_days)?;

        if memories.is_empty() {
            return Ok((0, 0, 0));
        }

        eprintln!(
            "[Nightly] Backfilling embeddings for {} memories...",
            memories.len()
        );

        let http_batch = parse_embed_http_batch_from_env();
        let (success, failed, truncated) = self.process_rows(memories, http_batch).await;

        eprintln!(
            "[Nightly] Embedding backfill complete: {} succeeded, {} failed.",
            success, failed
        );
        Ok((success, failed, truncated))
    }

    /// Deadline-bound keyset catch-up of pinned NULL embeddings (T338 F2).
    ///
    /// Pages `(updated_at DESC, memory_id DESC)`. After each chunk, the next page
    /// starts strictly after the last **fetched** cursor (including failures).
    /// Inner HTTP batch is [`parse_embed_http_batch_from_env`] (T343); 50ms yield
    /// is once per HTTP (including trailing flush). Chunk size is `chunk_env`
    /// unless remaining time is under 60s, then 50.
    pub async fn backfill_catchup(
        &self,
        deadline: &NightlyDeadline,
        chunk_env: usize,
    ) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
        let mut success = 0;
        let mut failed = 0;
        let mut truncated = 0;
        let mut after: Option<(String, String)> = None;
        let http_batch = parse_embed_http_batch_from_env();

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
            let rows = page
                .into_iter()
                .map(|row| (row.memory_id, row.content))
                .collect::<Vec<_>>();
            let (s, f, t) = self.process_rows(rows, http_batch).await;
            success += s;
            failed += f;
            truncated += t;
            after = last;
        }

        eprintln!(
            "[Nightly] Embedding backfill complete: {} succeeded, {} failed.",
            success, failed
        );
        Ok((success, failed, truncated))
    }

    /// Refresh stale embeddings (older than threshold)
    pub async fn refresh_stale(
        &self,
        days_threshold: i32,
        limit: usize,
    ) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
        let memories = self.query_store.get_stale_memories(days_threshold, limit)?;

        if memories.is_empty() {
            return Ok((0, 0, 0));
        }

        eprintln!(
            "[Nightly] Refreshing {} stale embeddings (>{} days old)...",
            memories.len(),
            days_threshold
        );

        for (idx, (memory_id, _)) in memories.iter().enumerate() {
            eprintln!(
                "    [Stale {}/{}] Re-embedding memory {}...",
                idx + 1,
                memories.len(),
                &memory_id[..8.min(memory_id.len())]
            );
        }

        let http_batch = parse_embed_http_batch_from_env();
        let (success, failed, truncated) = self.process_rows(memories, http_batch).await;

        eprintln!(
            "[Nightly] Stale refresh complete: {} succeeded, {} failed.",
            success, failed
        );
        Ok((success, failed, truncated))
    }

    /// Cloud-policy per memory, then pack 1..=8 texts per `embed_batch` (T343 F1/F7).
    /// Flushes a trailing 1..=7 buffer. Yields 50ms once per HTTP, including the flush.
    async fn process_rows(
        &self,
        rows: impl IntoIterator<Item = (String, String)>,
        http_batch: usize,
    ) -> (usize, usize, usize) {
        let allow_cloud = allow_cloud_extraction_from_env();
        let is_local = self.embedding_provider.is_local();
        let batch = http_batch.clamp(1, MAX_EMBED_HTTP_BATCH);
        let mut success = 0usize;
        let mut failed = 0usize;
        let mut truncated = 0usize;
        let mut buffer: Vec<(String, String)> = Vec::with_capacity(batch);

        for (memory_id, content) in rows {
            let privacy = self.resolve_memory_privacy(&memory_id);
            if let Err(denial) = cloud_route_allowed(privacy, is_local, allow_cloud) {
                tracing::warn!(
                    reason_code = %denial.reason_code,
                    memory_id = %memory_id,
                    "skipping embedding: cloud-policy gate denied"
                );
                failed += 1;
                continue;
            }
            let (chunks, chunk_trunc) = chunk_for_embed(&content);
            if chunk_trunc {
                truncated += 1;
            }
            if chunks.len() > 1 {
                if !buffer.is_empty() {
                    let (s, f) = self.flush_embed_batch(&mut buffer).await;
                    success += s;
                    failed += f;
                }
                let (s, f) = self.embed_long_memory_isolated(&memory_id, chunks).await;
                success += s;
                failed += f;
                continue;
            }
            let text = chunks.into_iter().next().unwrap_or_default();
            buffer.push((memory_id, text));
            if buffer.len() == batch {
                let (s, f) = self.flush_embed_batch(&mut buffer).await;
                success += s;
                failed += f;
            }
        }
        if !buffer.is_empty() {
            let (s, f) = self.flush_embed_batch(&mut buffer).await;
            success += s;
            failed += f;
        }
        (success, failed, truncated)
    }

    async fn flush_embed_batch(&self, buffer: &mut Vec<(String, String)>) -> (usize, usize) {
        if buffer.is_empty() {
            return (0, 0);
        }
        let ids: Vec<String> = buffer.iter().map(|(id, _)| id.clone()).collect();
        let texts: Vec<String> = buffer.iter().map(|(_, text)| text.clone()).collect();
        buffer.clear();

        let result = self.embedding_provider.embed_batch(texts).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        match result {
            Ok(vectors) => {
                if vectors.len() != ids.len() {
                    tracing::warn!(
                        expected = ids.len(),
                        got = vectors.len(),
                        "embed_batch length mismatch; failing whole batch"
                    );
                    return (0, ids.len());
                }
                let mut success = 0usize;
                let mut failed = 0usize;
                for (id, response) in ids.iter().zip(vectors) {
                    let bytes = f32_vec_to_bytes(&response.vector);
                    if let Err(e) = self.query_store.store_embedding(id, &bytes) {
                        tracing::warn!("Failed to store embedding for memory {}: {}", id, e);
                        failed += 1;
                    } else {
                        tracing::info!("Stored embedding for memory {}", id);
                        success += 1;
                    }
                }
                (success, failed)
            }
            Err(e) => {
                tracing::warn!("Failed to generate embeddings for batch: {}", e);
                (0, ids.len())
            }
        }
    }

    /// Single-chunk: `embed()`. Multi-chunk: `embed_batch` then mean-pool + L2 (T351).
    async fn embed_chunks_to_blob(
        &self,
        chunks: &[String],
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        if chunks.len() <= 1 {
            let text = chunks.first().cloned().unwrap_or_default();
            let response = self
                .embedding_provider
                .embed(EmbeddingRequest { text })
                .await?;
            return Ok(Some(f32_vec_to_bytes(&response.vector)));
        }
        let expected = chunks.len();
        let responses = self.embedding_provider.embed_batch(chunks.to_vec()).await?;
        if responses.len() != expected {
            tracing::warn!(
                expected,
                got = responses.len(),
                "embed_batch length mismatch; skipping long-memory store"
            );
            return Ok(None);
        }
        let vectors: Vec<Vec<f32>> = responses.into_iter().map(|r| r.vector).collect();
        Ok(mean_pool_l2(&vectors).map(|v| f32_vec_to_bytes(&v)))
    }

    async fn embed_long_memory_isolated(
        &self,
        memory_id: &str,
        chunks: Vec<String>,
    ) -> (usize, usize) {
        let expected = chunks.len();
        let result = self.embedding_provider.embed_batch(chunks).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        match result {
            Ok(responses) => {
                if responses.len() != expected {
                    tracing::warn!(
                        expected,
                        got = responses.len(),
                        memory_id,
                        "embed_batch length mismatch; failing isolated long memory"
                    );
                    return (0, 1);
                }
                let vectors: Vec<Vec<f32>> = responses.into_iter().map(|r| r.vector).collect();
                let Some(pooled) = mean_pool_l2(&vectors) else {
                    return (0, 1);
                };
                let bytes = f32_vec_to_bytes(&pooled);
                if let Err(e) = self.query_store.store_embedding(memory_id, &bytes) {
                    tracing::warn!("Failed to store embedding for memory {memory_id}: {e}");
                    (0, 1)
                } else {
                    (1, 0)
                }
            }
            Err(e) => {
                tracing::warn!("Failed to generate embeddings for long memory {memory_id}: {e}");
                (0, 1)
            }
        }
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

/// Max unicode scalars sent after T229 (llama.cpp embed `--ubatch-size` 2048).
const EMBED_UBATCH_MAX_CHARS: usize = 2048;

/// Prefix-truncate to [`EMBED_UBATCH_MAX_CHARS`] unicode scalars at a char boundary.
///
/// Stay-green T343 helper (production packer uses [`chunk_for_embed`]).
/// Does **not** use [`ai_brains_models::estimate_tokens`] (len/3.5 under-counts dense ASCII).
#[cfg(test)]
pub(crate) fn truncate_for_embed_ubatch(content: &str) -> &str {
    if content.chars().count() <= EMBED_UBATCH_MAX_CHARS {
        return content;
    }
    match content.char_indices().nth(EMBED_UBATCH_MAX_CHARS) {
        Some((i, _)) => &content[..i],
        None => content,
    }
}

const MAX_EMBED_CHUNKS: usize = 4;
const MEAN_POOL_MIN_NORM: f32 = 1e-12;

/// Split raw content into ≤2048-scalar windows (max 4), then per-chunk 4000-byte cap (T351 F1).
/// Second flag is true when a 5th+ window was dropped or a per-chunk byte cap trimmed a window.
pub(crate) fn chunk_for_embed(content: &str) -> (Vec<String>, bool) {
    let mut truncated = content.chars().count() > MAX_EMBED_CHUNKS * EMBED_UBATCH_MAX_CHARS;
    let mut chunks = Vec::new();
    let mut iter = content.chars();
    for _ in 0..MAX_EMBED_CHUNKS {
        let window: String = iter.by_ref().take(EMBED_UBATCH_MAX_CHARS).collect();
        if window.is_empty() {
            break;
        }
        let capped = truncate_for_embed(&window);
        if capped.len() < window.len() {
            truncated = true;
        }
        chunks.push(capped.to_string());
    }
    if chunks.is_empty() {
        chunks.push(String::new());
    }
    (chunks, truncated)
}

/// Uniform mean then L2-normalize. None on dim mismatch, non-finite, or near-zero norm.
pub(crate) fn mean_pool_l2(vectors: &[Vec<f32>]) -> Option<Vec<f32>> {
    let first = vectors.first()?;
    let dim = first.len();
    if dim == 0 {
        return None;
    }
    if vectors.iter().any(|v| v.len() != dim) {
        return None;
    }
    let n = vectors.len() as f32;
    let mut mean = vec![0.0_f32; dim];
    for v in vectors {
        for (i, &x) in v.iter().enumerate() {
            if !x.is_finite() {
                return None;
            }
            mean[i] += x;
        }
    }
    for m in &mut mean {
        *m /= n;
        if !m.is_finite() {
            return None;
        }
    }
    let norm = mean.iter().map(|x| x * x).sum::<f32>().sqrt();
    if !norm.is_finite() || norm <= MEAN_POOL_MIN_NORM {
        return None;
    }
    for m in &mut mean {
        *m /= norm;
    }
    Some(mean)
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

    /// AC9: production chunks via `chunk_for_embed` (T351) which applies T229 per chunk.
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
            production.contains("chunk_for_embed("),
            "generate_and_store / packer must call chunk_for_embed"
        );
        assert!(
            production.contains("truncate_for_embed("),
            "per-chunk T229 cap must still call truncate_for_embed"
        );
    }

    #[test]
    fn chunk_windows__max_2048_scalars() {
        let long = "x".repeat(3000);
        let (chunks, truncated) = chunk_for_embed(&long);
        assert_eq!(chunks.len(), 2);
        assert!(!truncated);
        assert_eq!(chunks[0].chars().count(), EMBED_UBATCH_MAX_CHARS);
        assert_eq!(chunks[1].chars().count(), 952);
        let cjk = "你".repeat(3000);
        let (cjk_chunks, cjk_trunc) = chunk_for_embed(&cjk);
        assert_eq!(cjk_chunks.len(), 2);
        // 2048 CJK scalars × 3 UTF-8 bytes > 4000: T229 caps each window (F5 truncated).
        assert!(cjk_trunc);
        for c in &cjk_chunks {
            assert!(c.chars().count() <= EMBED_UBATCH_MAX_CHARS);
            assert!(c.len() <= EMBED_TEXT_MAX_BYTES);
            assert!(std::str::from_utf8(c.as_bytes()).is_ok());
        }
        let over = "y".repeat(9000);
        let (over_chunks, over_trunc) = chunk_for_embed(&over);
        assert_eq!(over_chunks.len(), 4);
        assert!(over_trunc);
    }

    #[test]
    fn mean_pool_l2__dim_mismatch__none() {
        assert!(mean_pool_l2(&[vec![1.0, 0.0], vec![0.0]]).is_none());
        assert!(mean_pool_l2(&[vec![0.0, 0.0]]).is_none());
        let pooled = mean_pool_l2(&[vec![1.0, 0.0], vec![0.0, 1.0]]).expect("mean");
        let n = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((n - 1.0).abs() < 1e-5);
    }

    #[test]
    fn truncate_for_embed_ubatch__over_2048_scalars__prefix_char_boundary() {
        let long = "x".repeat(3000);
        let out = truncate_for_embed_ubatch(&long);
        assert_eq!(out.chars().count(), EMBED_UBATCH_MAX_CHARS);
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
        assert!(out.is_char_boundary(out.len()));
        assert_eq!(out, "x".repeat(2048));

        let short = "hello";
        assert_eq!(truncate_for_embed_ubatch(short), short);

        let exact = "y".repeat(2048);
        assert_eq!(truncate_for_embed_ubatch(&exact), exact);

        // CJK: 2048 scalars at a char boundary (each 你 is one scalar, 3 bytes).
        let cjk = "你".repeat(2100);
        let out_cjk = truncate_for_embed_ubatch(&cjk);
        assert_eq!(out_cjk.chars().count(), EMBED_UBATCH_MAX_CHARS);
        assert!(std::str::from_utf8(out_cjk.as_bytes()).is_ok());
        assert!(out_cjk.is_char_boundary(out_cjk.len()));
    }
}
