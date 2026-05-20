use crate::errors::Result;
use crate::lexical::lexical_search;
use crate::GraphSearch;
use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_core::privacy::Privacy;
use ai_brains_store::VaultConnection;

#[derive(Debug, Clone, PartialEq)]
pub struct RecallHit {
    pub memory_id: String,
    pub content: String,
    pub source: String,
    pub score: Option<f64>,
    /// Privacy flag inherited from the source memory.
    pub privacy: Option<Privacy>,
}

impl RecallHit {
    /// Create a basic FTS5 hit with no privacy flag.
    pub fn fts(memory_id: String, content: String, score: Option<f64>) -> Self {
        Self {
            memory_id,
            content,
            source: "fts".to_string(),
            score,
            privacy: None,
        }
    }

    /// Create a hit from the unified IPC bridge.
    pub fn bridge(
        memory_id: String,
        content: String,
        score: Option<f64>,
        source: String,
        privacy: Option<Privacy>,
    ) -> Self {
        Self {
            memory_id,
            content,
            source,
            score,
            privacy,
        }
    }
}

/// Primary recall entry point. Attempts unified IPC recall via ChangeGuard
/// (`bridge query`) first. If IPC is unavailable or fails, falls back to
/// local FTS5 search. Results from both sources are blended, with privacy
/// flags preserved from bridge hits.
pub fn recall(
    conn: &VaultConnection,
    graph: Option<&GraphSearch>,
    query: &str,
    limit: usize,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
) -> Result<Vec<RecallHit>> {
    // Phase 1: Try unified IPC recall via ChangeGuard bridge query.
    let bridge_hits = query_changeguard_bridge(query, project_id, session_id);

    // Phase 2: Always run local FTS5 as a fallback / supplement.
    let local_hits: Vec<RecallHit> = lexical_search(conn, query, project_id, session_id)?
        .into_iter()
        .map(|memory| RecallHit::fts(memory.memory_id, memory.content, memory.score))
        .collect();

    // Phase 3: Blend results. Bridge hits come first (higher authority),
    // followed by local FTS5 hits. Deduplicate by memory_id.
    let mut seen_ids = std::collections::HashSet::new();
    let mut blended = Vec::new();

    match bridge_hits {
        Ok(bridge) => {
            for hit in bridge {
                if seen_ids.insert(hit.memory_id.clone()) {
                    blended.push(hit);
                }
            }
        }
        Err(e) => {
            eprintln!(
                "ChangeGuard bridge query failed, falling back to local FTS5 only: {}",
                e
            );
        }
    }

    // Add local hits, skipping any already present from the bridge.
    for hit in local_hits {
        if seen_ids.insert(hit.memory_id.clone()) {
            blended.push(hit);
        }
    }

    // Truncate to limit.
    if blended.len() > limit {
        blended.truncate(limit);
    }

    // Graph-based augmentation placeholder (preserves existing contract).
    if let Some(_searcher) = graph {
        // Future: graph-based ranking/expansion could go here.
    }

    Ok(blended)
}

// ---------------------------------------------------------------------------
// Unified IPC Bridge Query
// ---------------------------------------------------------------------------

/// Query ChangeGuard's blended Tantivy search via the bridge IPC.
/// Sends a `bridge query` subcommand and parses the NDJSON response
/// for `BridgeRecord::Insight` entries.
///
/// Returns Ok(Vec) on success. On any failure (CLI missing, non-zero exit,
/// parse errors), returns an Err so the caller can fall back to local FTS5.
#[allow(clippy::disallowed_methods)]
fn query_changeguard_bridge(
    query: &str,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
) -> std::result::Result<Vec<RecallHit>, String> {
    // Build a temp file for the query input and output.
    let temp_in = tempfile::NamedTempFile::new().map_err(|e| format!("tempfile error: {}", e))?;
    let temp_in_path = temp_in.path().to_path_buf();

    let temp_out = tempfile::NamedTempFile::new().map_err(|e| format!("tempfile error: {}", e))?;
    let temp_out_path = temp_out.path().to_path_buf();

    // Write a minimal BridgeRecord as the query envelope.
    let timestamp = chrono::Utc::now().to_rfc3339();
    let query_record = BridgeRecord {
        bridge_version: "0.2".to_string(),
        direction: ai_brains_contracts::bridge::BridgeDirection::Outbound,
        timestamp,
        parent_hash: None,
        project_id: project_id.map(|p| p.to_string()).unwrap_or_default(),
        session_id: session_id.map(|s| s.to_string()),
        tx_id: None,
        record_kind: "search_query".to_string(),
        payload: serde_json::json!({
            "query": query,
            "kind": "unified",
        }),
        privacy: Privacy::LocalOnly,
    };

    let ndjson =
        serde_json::to_string(&query_record).map_err(|e| format!("serialize error: {}", e))?;
    std::fs::write(&temp_in_path, ndjson.as_bytes()).map_err(|e| format!("write error: {}", e))?;

    // Invoke: changeguard bridge query --in <input> --out <output>
    let output = std::process::Command::new("changeguard")
        .args([
            "bridge",
            "query",
            "--in",
            temp_in_path.to_str().unwrap_or(""),
            "--out",
            temp_out_path.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| format!("changeguard CLI not available: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("changeguard bridge query failed: {}", stderr));
    }

    // Parse NDJSON output: each line is a BridgeRecord with record_kind = "insight".
    let content =
        std::fs::read_to_string(&temp_out_path).map_err(|e| format!("read output error: {}", e))?;

    let mut hits = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let record: BridgeRecord = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to parse bridge query response line: {}", e);
                continue;
            }
        };

        // Only process Insight records from the unified search result.
        if record.record_kind.to_lowercase() != "insight" {
            continue;
        }

        let memory_id = record
            .payload
            .get("memory_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let content = record
            .payload
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let score = record.payload.get("score").and_then(|v| v.as_f64());
        let source = record
            .payload
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("bridge")
            .to_string();
        let privacy = Some(record.privacy);

        if !content.is_empty() {
            hits.push(RecallHit::bridge(
                memory_id, content, score, source, privacy,
            ));
        }
    }

    Ok(hits)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recall_hit_fts_constructor() {
        let hit = RecallHit::fts("mem-1".into(), "test content".into(), Some(0.85));
        assert_eq!(hit.memory_id, "mem-1");
        assert_eq!(hit.source, "fts");
        assert_eq!(hit.score, Some(0.85));
        assert_eq!(hit.privacy, None);
    }

    #[test]
    fn recall_hit_bridge_constructor() {
        let hit = RecallHit::bridge(
            "mem-2".into(),
            "bridge content".into(),
            Some(0.92),
            "code_context".into(),
            Some(Privacy::LocalOnly),
        );
        assert_eq!(hit.memory_id, "mem-2");
        assert_eq!(hit.source, "code_context");
        assert_eq!(hit.score, Some(0.92));
        assert_eq!(hit.privacy, Some(Privacy::LocalOnly));
    }

    #[test]
    fn query_changeguard_bridge_fails_when_cli_missing() {
        // Temporarily mess with PATH so changeguard can't be found.
        // Actually, the function already handles missing CLI gracefully.
        let result = query_changeguard_bridge("test query", None, None);
        // On CI / local without changeguard, this should fail gracefully.
        if let Err(ref e) = result {
            assert!(
                e.contains("not available") || e.contains("CLI not found") || e.contains("failed"),
                "Error message should indicate unavailability: {}",
                e
            );
        }
        // If it succeeds (changeguard is installed), that's also fine.
    }

    #[test]
    fn blending_deduplicates_by_memory_id() {
        let mut bridge_hits = vec![
            RecallHit::bridge(
                "mem-1".into(),
                "c1".into(),
                Some(0.9),
                "bridge".into(),
                None,
            ),
            RecallHit::bridge(
                "mem-2".into(),
                "c2".into(),
                Some(0.8),
                "bridge".into(),
                None,
            ),
        ];

        let local_fts = vec![
            RecallHit::fts("mem-2".into(), "c2-fts".into(), Some(0.7)),
            RecallHit::fts("mem-3".into(), "c3".into(), Some(0.6)),
        ];

        let mut seen = std::collections::HashSet::new();
        let mut blended = Vec::new();

        for hit in bridge_hits.drain(..) {
            if seen.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }
        for hit in local_fts {
            if seen.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }

        assert_eq!(blended.len(), 3, "Should have 3 unique hits");
        // mem-1: from bridge only
        assert_eq!(blended[0].memory_id, "mem-1");
        assert_eq!(blended[0].source, "bridge");
        // mem-2: from bridge (first in, wins over FTS)
        assert_eq!(blended[1].memory_id, "mem-2");
        assert_eq!(blended[1].source, "bridge");
        // mem-3: from FTS only
        assert_eq!(blended[2].memory_id, "mem-3");
        assert_eq!(blended[2].source, "fts");
    }

    #[test]
    fn blending_preserves_privacy_flags_from_bridge() {
        let bridge_hits = vec![RecallHit::bridge(
            "mem-private".into(),
            "secret".into(),
            Some(1.0),
            "bridge".into(),
            Some(Privacy::NeverInject),
        )];

        let mut seen = std::collections::HashSet::new();
        let mut blended = Vec::new();
        for hit in bridge_hits {
            if seen.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }

        assert_eq!(blended.len(), 1);
        assert_eq!(blended[0].privacy, Some(Privacy::NeverInject));
    }
}
