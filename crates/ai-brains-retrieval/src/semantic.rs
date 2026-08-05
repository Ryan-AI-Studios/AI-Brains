use crate::errors::{Result, RetrievalError};
use crate::privacy_filter::is_injectable_privacy;
use crate::recall::RecallHit;
use ai_brains_contracts::recall::EmbeddingStatusDto;
use ai_brains_core::privacy::Privacy;
use ai_brains_models::ModelProvider;
use ai_brains_store::VaultConnection;
use rusqlite::params_from_iter;

type EmbeddedMemory = (String, String, Privacy, Option<String>, Vec<u8>);

/// Default embedding endpoint (llama.cpp OpenAI-compat style; aligned with nightly).
pub const DEFAULT_EMBEDDING_URL: &str = "http://127.0.0.1:8083";

/// Default embedding model name (aligned with nightly / docs).
pub const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text-v1.5";

/// Outcome of a semantic search attempt, always including embedding status (T202).
#[derive(Debug, Clone)]
pub struct SemanticOutcome {
    pub hits: Vec<RecallHit>,
    pub embedding: EmbeddingStatusDto,
}

/// Resolve the raw embedding endpoint from env (may include path/query/userinfo).
/// Prefer [`public_endpoint_label`] for any operator-visible surface (F26).
pub fn embedding_endpoint() -> String {
    std::env::var("AI_BRAINS_EMBEDDING_URL").unwrap_or_else(|_| DEFAULT_EMBEDDING_URL.to_string())
}

/// Sanitize endpoint to `scheme://host[:port]` only — strip userinfo, path, query, fragment.
///
/// Never returns credentials or vault paths. Unparseable input → `invalid-endpoint`.
pub fn public_endpoint_label(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return "invalid-endpoint".to_string();
    }
    // Manual parse without new deps: scheme://[userinfo@]host[:port][/path][?query][#frag]
    let Some((scheme, rest)) = raw.split_once("://") else {
        return "invalid-endpoint".to_string();
    };
    if scheme.is_empty()
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    {
        return "invalid-endpoint".to_string();
    }
    // Drop userinfo if present.
    let after_auth = match rest.split_once('@') {
        Some((_userinfo, hostport_path)) => hostport_path,
        None => rest,
    };
    // Host[:port] only — stop at path/query/fragment.
    let hostport = after_auth
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(after_auth)
        .trim();
    if hostport.is_empty() {
        return "invalid-endpoint".to_string();
    }
    // Reject obviously non-host shapes (spaces).
    if hostport.chars().any(|c| c.is_whitespace()) {
        return "invalid-endpoint".to_string();
    }
    format!("{scheme}://{hostport}")
}

/// Resolve the embedding model name from env (F5).
pub fn embedding_model() -> String {
    std::env::var("AI_BRAINS_EMBEDDING_MODEL")
        .unwrap_or_else(|_| DEFAULT_EMBEDDING_MODEL.to_string())
}

/// Classify a typed model error (F4 / M6 / F26) — never copy raw provider body into `detail`.
pub fn classify_model_error(
    err: &ai_brains_models::ModelError,
    endpoint_label: &str,
) -> EmbeddingStatusDto {
    use ai_brains_models::ModelError;
    let (status, detail) = match err {
        ModelError::Network(_) => ("unreachable", "network"),
        ModelError::Timeout => ("unreachable", "timeout"),
        ModelError::Provider(_) => ("error", "provider"),
        ModelError::PrivacyViolation(_) => ("error", "privacy"),
        ModelError::Unknown(_) => ("error", "unknown"),
    };
    EmbeddingStatusDto {
        status: status.to_string(),
        endpoint: Some(endpoint_label.to_string()),
        detail: Some(detail.to_string()),
    }
}

/// Classify a retrieval/embedding error into a closed embedding status (F4 / F35).
///
/// Prefer [`classify_model_error`] at the model boundary. This path is for thread
/// panic / runtime failures and residual string-shaped `RetrievalError::Model`.
///
/// Mapping:
/// - transport/network/timeout → `unreachable`
/// - provider / privacy / parse / panic → `error`
///
/// Provider-class strings win over transport keywords that may appear in bodies
/// (e.g. HTTP 500 body containing the word "timeout").
pub fn classify_embedding_error(err: &RetrievalError) -> EmbeddingStatusDto {
    let msg = err.to_string();
    let lower = msg.to_ascii_lowercase();
    let endpoint = public_endpoint_label(&embedding_endpoint());
    let (status, detail) = if is_error_class_message(&lower) {
        ("error", stable_detail_from_message(&lower))
    } else if is_unreachable_message(&lower) {
        ("unreachable", stable_detail_from_message(&lower))
    } else {
        ("error", "unknown")
    };
    EmbeddingStatusDto {
        status: status.to_string(),
        endpoint: Some(endpoint),
        detail: Some(detail.to_string()),
    }
}

/// Provider / parse / panic class — checked **before** transport keywords (P1-02).
fn is_error_class_message(lower: &str) -> bool {
    lower.contains("provider error")
        || lower.contains("privacy violation")
        || lower.contains("thread panicked")
        || lower.contains("runtime creation failed")
        || lower.contains("missing data")
        || lower.contains("returned ")
        || lower.contains("parse")
        || lower.contains("deserialize")
}

fn is_unreachable_message(lower: &str) -> bool {
    // Transport class (M6 / F4) — only after provider-class excluded.
    if lower.contains("timeout") || lower.contains("timed out") {
        return true;
    }
    if lower.contains("network error") {
        return true;
    }
    lower.contains("connection refused")
        || lower.contains("connect error")
        || lower.contains("failed to connect")
        || lower.contains("could not connect")
        || lower.contains("connection reset")
        || lower.contains("network is unreachable")
        || lower.contains("no route to host")
        || lower.contains("name or service not known")
        || lower.contains("no such host")
        || lower.contains("dns")
        || lower.contains("error trying to connect")
        || lower.contains("error sending request")
}

/// Stable non-sensitive detail codes only (F26) — never raw error bodies.
fn stable_detail_from_message(lower: &str) -> &'static str {
    if lower.contains("thread panicked") {
        "thread_panic"
    } else if lower.contains("runtime creation failed") {
        "runtime"
    } else if lower.contains("provider error") || lower.contains("returned ") {
        "provider"
    } else if lower.contains("privacy") {
        "privacy"
    } else if lower.contains("timeout") || lower.contains("timed out") {
        "timeout"
    } else if lower.contains("network")
        || lower.contains("connection")
        || lower.contains("dns")
        || lower.contains("error sending request")
    {
        "network"
    } else {
        "unknown"
    }
}

/// Derive status after a successful embed request against stored rows (F4 / M7).
///
/// - zero rows or all undecodable → `no_stored_embeddings`
/// - ≥1 decodable row scored → `ok` (even when similarity yields zero hits)
pub fn status_after_embed_ok(
    total_rows: usize,
    decodable_rows: usize,
    endpoint: Option<String>,
) -> EmbeddingStatusDto {
    if total_rows == 0 {
        EmbeddingStatusDto {
            status: "no_stored_embeddings".to_string(),
            endpoint,
            detail: Some("zero_rows".to_string()),
        }
    } else if decodable_rows == 0 {
        EmbeddingStatusDto {
            status: "no_stored_embeddings".to_string(),
            endpoint,
            detail: Some("all_rows_undecodable".to_string()),
        }
    } else {
        EmbeddingStatusDto {
            status: "ok".to_string(),
            endpoint,
            detail: None,
        }
    }
}

/// Perform semantic search over pinned memories with non-null embeddings.
///
/// Fetches an embedding for the query via LlamaCppProvider, then computes
/// cosine similarity against stored embedding BLOBs.
///
/// Embed backend failures return `Ok` with empty hits and a classified status
/// (soft-fail; F3). Database errors still surface as `Err`.
pub fn semantic_search(
    conn: &VaultConnection,
    query: &str,
    limit: usize,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
) -> Result<SemanticOutcome> {
    let endpoint_label = public_endpoint_label(&embedding_endpoint());

    let query_embedding = match fetch_embedding(query) {
        Ok(v) => v,
        Err(status) => {
            return Ok(SemanticOutcome {
                hits: Vec::new(),
                embedding: status,
            });
        }
    };

    let memories = fetch_pinned_embeddings(conn, project_id, session_id)?;
    let total_rows = memories.len();
    let mut decodable_rows = 0usize;
    let mut scored: Vec<(f64, RecallHit)> = Vec::new();

    for (memory_id, content, privacy, session_id, embedding_bytes) in memories {
        let Some(emb) = bytes_to_f32_vec(&embedding_bytes) else {
            continue;
        };
        decodable_rows = decodable_rows.saturating_add(1);
        let Some(sim) = cosine_similarity(&query_embedding, &emb) else {
            continue;
        };
        scored.push((
            sim,
            RecallHit {
                memory_id,
                content,
                source: "semantic".to_string(),
                score: Some(sim),
                privacy: Some(privacy),
                session_id,
                // Semantic arm does not SELECT memory updated_at (F16: None OK).
                updated_at: None,
                is_plan_demoted: false,
            },
        ));
    }

    let embedding = status_after_embed_ok(total_rows, decodable_rows, Some(endpoint_label));

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    Ok(SemanticOutcome {
        hits: scored.into_iter().map(|(_, hit)| hit).collect(),
        embedding,
    })
}

/// Fetch query embedding. On model/transport failure returns classified status
/// (never raw provider body / credentials). Runtime/thread failures → `error`.
fn fetch_embedding(text: &str) -> std::result::Result<Vec<f32>, EmbeddingStatusDto> {
    let text = text.to_string();
    let endpoint_raw = embedding_endpoint();
    let endpoint_label = public_endpoint_label(&endpoint_raw);
    let model = embedding_model();
    let endpoint_for_provider = endpoint_raw;
    let label_for_thread = endpoint_label.clone();
    let handle = std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => {
                return Err(EmbeddingStatusDto {
                    status: "error".to_string(),
                    endpoint: Some(label_for_thread.clone()),
                    detail: Some("runtime".to_string()),
                });
            }
        };
        let provider =
            ai_brains_models::llama_cpp::LlamaCppProvider::new(endpoint_for_provider, model);
        let req = ai_brains_models::EmbeddingRequest { text };
        match rt.block_on(provider.embed(req)) {
            Ok(res) => Ok(res.vector),
            Err(e) => Err(classify_model_error(&e, &label_for_thread)),
        }
    });

    match handle.join() {
        Ok(inner) => inner,
        Err(_) => Err(EmbeddingStatusDto {
            status: "error".to_string(),
            endpoint: Some(endpoint_label),
            detail: Some("thread_panic".to_string()),
        }),
    }
}

fn fetch_pinned_embeddings(
    conn: &VaultConnection,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
) -> Result<Vec<EmbeddedMemory>> {
    let conn = conn.lock()?;

    let mut sql = "SELECT mp.memory_id, mp.content, mp.privacy, mp.session_id, mp.embedding
        FROM memory_projection mp
        LEFT JOIN session_projection sp ON mp.session_id = sp.session_id
        WHERE mp.status = 'pinned' AND mp.embedding IS NOT NULL"
        .to_string();

    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(sid) = session_id {
        sql.push_str(" AND mp.session_id = ?");
        params.push(sid.to_string().into());
    }

    if let Some(pid) = project_id {
        sql.push_str(" AND (sp.project_id = ? OR mp.project_id = ?)");
        let pid_str = pid.to_string();
        params.push(pid_str.clone().into());
        params.push(pid_str.into());
    }

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(params))?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        let memory_id: String = row.get(0)?;
        let content: String = row.get(1)?;
        let privacy_str: String = row.get(2)?;
        let session_id: Option<String> = row.get(3)?;
        let embedding: Vec<u8> = row.get(4)?;

        if !is_injectable_privacy(&privacy_str) {
            continue;
        }

        let privacy: Privacy = serde_json::from_str(&privacy_str).unwrap_or(Privacy::LocalOnly);
        results.push((memory_id, content, privacy, session_id, embedding));
    }

    Ok(results)
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Option<Vec<f32>> {
    if !bytes.len().is_multiple_of(4) {
        return None;
    }
    let mut vec = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        let arr: [u8; 4] = chunk.try_into().ok()?;
        vec.push(f32::from_le_bytes(arr));
    }
    Some(vec)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f64> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let mut dot = 0.0f64;
    let mut norm_a = 0.0f64;
    let mut norm_b = 0.0f64;
    for i in 0..a.len() {
        let av = a[i] as f64;
        let bv = b[i] as f64;
        dot += av * bv;
        norm_a += av * av;
        norm_b += bv * bv;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        return None;
    }
    Some(dot / denom)
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;

    #[test]
    fn classify_model_error__network__unreachable() {
        let err = ai_brains_models::ModelError::Network("connection refused".into());
        let status = classify_model_error(&err, "http://127.0.0.1:8083");
        assert_eq!(status.status, "unreachable");
        assert_eq!(status.detail.as_deref(), Some("network"));
    }

    #[test]
    fn classify_model_error__timeout__unreachable() {
        let err = ai_brains_models::ModelError::Timeout;
        let status = classify_model_error(&err, "http://127.0.0.1:8083");
        assert_eq!(status.status, "unreachable");
        assert_eq!(status.detail.as_deref(), Some("timeout"));
    }

    #[test]
    fn classify_model_error__provider_body_mentions_timeout__still_error() {
        // P1-02: typed Provider must not be misclassified by body keywords.
        let err = ai_brains_models::ModelError::Provider(
            "returned 500: connection refused timeout in upstream".into(),
        );
        let status = classify_model_error(&err, "http://127.0.0.1:8083");
        assert_eq!(status.status, "error");
        assert_eq!(status.detail.as_deref(), Some("provider"));
        assert!(
            !status
                .detail
                .as_deref()
                .unwrap_or("")
                .contains("connection"),
            "detail must not echo body: {:?}",
            status.detail
        );
    }

    #[test]
    fn classify_embedding_error__connection_refused__unreachable() {
        let err = RetrievalError::Model(
            "embedding request failed: Network error: error trying to connect: \
             tcp connect error: Connection refused (os error 10061)"
                .into(),
        );
        let status = classify_embedding_error(&err);
        assert_eq!(
            status.status, "unreachable",
            "connection refused must map to unreachable; detail={:?}",
            status.detail
        );
        assert_eq!(status.detail.as_deref(), Some("network"));
    }

    #[test]
    fn classify_embedding_error__timeout__unreachable() {
        let err = RetrievalError::Model("embedding request failed: Timeout".into());
        let status = classify_embedding_error(&err);
        assert_eq!(status.status, "unreachable");
        assert_eq!(status.detail.as_deref(), Some("timeout"));
    }

    #[test]
    fn classify_embedding_error__dns_failure__unreachable() {
        let err = RetrievalError::Model(
            "embedding request failed: Network error: dns error: no such host".into(),
        );
        let status = classify_embedding_error(&err);
        assert_eq!(status.status, "unreachable");
    }

    #[test]
    fn classify_embedding_error__error_sending_request__unreachable() {
        // Windows/reqwest often surfaces this without "connection refused".
        let err = RetrievalError::Model(
            "embedding request failed: Network error: error sending request for url \
             (http://127.0.0.1:1/v1/embeddings)"
                .into(),
        );
        let status = classify_embedding_error(&err);
        assert_eq!(status.status, "unreachable");
    }

    #[test]
    fn classify_embedding_error__provider_body_timeout_keyword__error_not_unreachable() {
        let err = RetrievalError::Model(
            "embedding request failed: Provider error: llama.cpp returned 500: timeout upstream"
                .into(),
        );
        let status = classify_embedding_error(&err);
        assert_eq!(
            status.status, "error",
            "provider-class must win over timeout keyword in body; detail={:?}",
            status.detail
        );
        assert_eq!(status.detail.as_deref(), Some("provider"));
    }

    #[test]
    fn classify_embedding_error__http_non_2xx__error() {
        let err = RetrievalError::Model(
            "embedding request failed: Provider error: llama.cpp (embeddings) returned 500: boom"
                .into(),
        );
        let status = classify_embedding_error(&err);
        assert_eq!(
            status.status, "error",
            "HTTP provider errors must map to error; detail={:?}",
            status.detail
        );
    }

    #[test]
    fn classify_embedding_error__body_parse__error() {
        let err = RetrievalError::Model(
            "embedding request failed: Provider error: Missing data[0].embedding field".into(),
        );
        let status = classify_embedding_error(&err);
        assert_eq!(status.status, "error");
    }

    #[test]
    fn classify_embedding_error__thread_panic__error() {
        let err = RetrievalError::Model("embedding thread panicked: Any { .. }".into());
        let status = classify_embedding_error(&err);
        assert_eq!(status.status, "error");
        assert_eq!(status.detail.as_deref(), Some("thread_panic"));
    }

    #[test]
    fn public_endpoint_label__strips_userinfo_path_query() {
        let raw = "https://user:s3cret@embed.example:8443/v1/embeddings?api_key=abc#frag";
        assert_eq!(public_endpoint_label(raw), "https://embed.example:8443");
    }

    #[test]
    fn public_endpoint_label__default_host_port() {
        assert_eq!(
            public_endpoint_label("http://127.0.0.1:8083"),
            "http://127.0.0.1:8083"
        );
    }

    #[test]
    fn public_endpoint_label__invalid__sentinel() {
        assert_eq!(public_endpoint_label("not a url"), "invalid-endpoint");
        assert_eq!(public_endpoint_label(""), "invalid-endpoint");
    }

    #[test]
    fn status_after_embed_ok__zero_rows__no_stored_embeddings() {
        let s = status_after_embed_ok(0, 0, Some("http://127.0.0.1:8083".into()));
        assert_eq!(s.status, "no_stored_embeddings");
        assert_eq!(s.detail.as_deref(), Some("zero_rows"));
    }

    #[test]
    fn status_after_embed_ok__all_undecodable__no_stored_embeddings() {
        let s = status_after_embed_ok(3, 0, Some("http://127.0.0.1:8083".into()));
        assert_eq!(s.status, "no_stored_embeddings");
        assert_eq!(s.detail.as_deref(), Some("all_rows_undecodable"));
    }

    #[test]
    fn status_after_embed_ok__decodable_rows__ok() {
        let s = status_after_embed_ok(5, 2, Some("http://127.0.0.1:8083".into()));
        assert_eq!(s.status, "ok");
        assert!(s.detail.is_none());
    }

    #[test]
    fn embedding_model__env_override__honored() {
        let _g = TempEnv::set("AI_BRAINS_EMBEDDING_MODEL", "custom-embed-model");
        assert_eq!(embedding_model(), "custom-embed-model");
    }

    #[test]
    fn embedding_model__unset__default_nomic() {
        let _g = TempEnv::remove("AI_BRAINS_EMBEDDING_MODEL");
        assert_eq!(embedding_model(), DEFAULT_EMBEDDING_MODEL);
    }

    #[test]
    fn embedding_endpoint__unset__default_8083() {
        let _g = TempEnv::remove("AI_BRAINS_EMBEDDING_URL");
        assert_eq!(embedding_endpoint(), DEFAULT_EMBEDDING_URL);
    }

    #[test]
    fn bytes_to_f32_vec__odd_length__none() {
        assert!(bytes_to_f32_vec(&[1, 2, 3]).is_none());
    }

    #[test]
    fn bytes_to_f32_vec__valid_le__decodes() {
        let bytes = 1.0f32.to_le_bytes();
        let v = bytes_to_f32_vec(&bytes).expect("valid");
        assert_eq!(v.len(), 1);
        assert!((v[0] - 1.0).abs() < f32::EPSILON);
    }
}
