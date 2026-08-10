use ai_brains_core::ids::SessionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallQuery {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallResult {
    pub memory_id: String,
    pub content: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// T211 soft F26: content-heuristic staleness (`"plan"` when plan-demoted). Omitted when None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staleness: Option<String>,
    /// T218 F5: how to interpret `score` on the wire.
    /// Closed set: `"bm25"` | `"rrf"` | `"bridge"` only (never `"cosine"` / `"hybrid"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_kind: Option<String>,
    /// T218 F4/F5: pre-fuse cosine when known (separate from score_kind; not a kind).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cosine: Option<f64>,
}

/// Status of the embedding backend for a semantic recall attempt (T202).
///
/// Closed status set: `ok` | `unreachable` | `error` | `no_stored_embeddings` | `skipped`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmbeddingStatusDto {
    /// Closed set: ok | unreachable | error | no_stored_embeddings | skipped
    pub status: String,
    /// Host URL only (no secrets); omitted when not useful.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Soft detail (e.g. `zero_rows`, `all_rows_undecodable`); never secrets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallResponse {
    pub results: Vec<RecallResult>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "effective_session_id"
    )]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// Present when `--semantic` was requested (T202 honesty).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<EmbeddingStatusDto>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::disallowed_methods)]
    #[allow(non_snake_case)]
    fn recall_response__serializes_with_session_id() {
        let resp = RecallResponse {
            results: vec![],
            session_id: Some("test-session".to_string()),
            hint: None,
            embedding: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("session_id"));
        assert!(json.contains("test-session"));
    }

    #[test]
    #[allow(clippy::disallowed_methods)]
    #[allow(non_snake_case)]
    fn recall_response__omits_none_session_id() {
        let resp = RecallResponse {
            results: vec![],
            session_id: None,
            hint: None,
            embedding: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("session_id"));
    }

    #[test]
    #[allow(clippy::disallowed_methods)]
    #[allow(non_snake_case)]
    fn recall_response__omits_none_embedding() {
        let resp = RecallResponse {
            results: vec![],
            session_id: None,
            hint: None,
            embedding: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(
            !json.contains("embedding"),
            "embedding must be omitted when None; got {json}"
        );
    }

    #[test]
    #[allow(clippy::disallowed_methods)]
    #[allow(non_snake_case)]
    fn recall_response__serializes_embedding_status() {
        let resp = RecallResponse {
            results: vec![],
            session_id: None,
            hint: None,
            embedding: Some(EmbeddingStatusDto {
                status: "unreachable".to_string(),
                endpoint: Some("http://127.0.0.1:8083".to_string()),
                detail: Some("connection refused".to_string()),
            }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"embedding\""));
        assert!(json.contains("\"status\":\"unreachable\""));
        assert!(json.contains("127.0.0.1:8083"));
        assert!(json.contains("connection refused"));
    }

    #[test]
    #[allow(clippy::disallowed_methods)]
    #[allow(non_snake_case)]
    fn embedding_status_dto__omits_none_optional_fields() {
        let dto = EmbeddingStatusDto {
            status: "ok".to_string(),
            endpoint: None,
            detail: None,
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(!json.contains("endpoint"));
        assert!(!json.contains("detail"));
        assert!(json.contains("\"status\":\"ok\""));
    }

    /// AC5: score_kind wire closed set; optional cosine; omit when None.
    #[test]
    #[allow(clippy::disallowed_methods)]
    #[allow(non_snake_case)]
    fn recall_result__score_kind_and_cosine_additive__ac5() {
        let with = RecallResult {
            memory_id: "m1".into(),
            content: "c".into(),
            source: "hybrid".into(),
            score: Some(0.016393), // raw RRF — not rescaled
            session_id: None,
            staleness: None,
            score_kind: Some("rrf".into()),
            cosine: Some(0.72),
        };
        let json = serde_json::to_string(&with).unwrap();
        assert!(json.contains("\"score_kind\":\"rrf\""));
        assert!(json.contains("\"cosine\":0.72") || json.contains("\"cosine\":0.720"));
        // AC6: score remains RRF machine value (not fake 0–1 confidence).
        assert!(json.contains("0.016393") || json.contains("\"score\":0.016"));
        assert!(!json.contains("\"score_kind\":\"cosine\""));
        assert!(!json.contains("\"score_kind\":\"hybrid\""));

        // Wire kinds only bm25|rrf|bridge
        for kind in ["bm25", "rrf", "bridge"] {
            let r = RecallResult {
                memory_id: "m".into(),
                content: "c".into(),
                source: "fts".into(),
                score: Some(1.0),
                session_id: None,
                staleness: None,
                score_kind: Some(kind.into()),
                cosine: None,
            };
            let j = serde_json::to_string(&r).unwrap();
            assert!(j.contains(&format!("\"score_kind\":\"{kind}\"")));
            assert!(
                !j.contains("\"cosine\""),
                "cosine omitted when None; got {j}"
            );
        }

        // Deserializes without new fields (N−1).
        let legacy = r#"{"memory_id":"x","content":"y","source":"fts"}"#;
        let parsed: RecallResult = serde_json::from_str(legacy).unwrap();
        assert!(parsed.score_kind.is_none());
        assert!(parsed.cosine.is_none());
    }

    /// AC6: score is not rescaled to fake 0–1 confidence when score_kind is rrf.
    #[test]
    #[allow(clippy::disallowed_methods)]
    #[allow(non_snake_case)]
    fn recall_result__rrf_score_not_rescaled__ac6() {
        let rrf_raw = 1.0 / 61.0;
        let r = RecallResult {
            memory_id: "m".into(),
            content: "c".into(),
            source: "semantic".into(),
            score: Some(rrf_raw),
            session_id: None,
            staleness: None,
            score_kind: Some("rrf".into()),
            cosine: Some(0.81),
        };
        let v: serde_json::Value = serde_json::to_value(&r).unwrap();
        let score = v["score"].as_f64().unwrap();
        assert!(
            (score - rrf_raw).abs() < 1e-12,
            "JSON score must stay raw RRF ({rrf_raw}), got {score}"
        );
        assert!(score < 0.05, "RRF rank-1 alone is ~0.016, not a cosine");
        assert_eq!(v["score_kind"], "rrf");
        assert!((v["cosine"].as_f64().unwrap() - 0.81).abs() < 1e-12);
    }
}
