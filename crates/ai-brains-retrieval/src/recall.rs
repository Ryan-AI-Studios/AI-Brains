use crate::GraphSearch;
use crate::errors::Result;
use crate::fts_utils::sanitize_fts_query;
use crate::hybrid::{candidate_depth, fuse_local_and_semantic, rrf_k};
use crate::lexical::{LexicalSearchOptions, lexical_search, substring_fallback};
use crate::ranking::ScoreKind;
use crate::semantic::classify_embedding_error;
use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_contracts::recall::EmbeddingStatusDto;
use ai_brains_core::is_contentless_query;
use ai_brains_core::privacy::Privacy;
use ai_brains_store::VaultConnection;

#[derive(Debug, Clone, Copy, Default)]
pub struct RecallOptions {
    pub project_id: Option<ai_brains_core::ids::ProjectId>,
    pub session_id: Option<ai_brains_core::ids::SessionId>,
    pub semantic: bool,
    pub graph_boost: f64,
    pub graph_hop_depth: usize,
    /// When true, suppress non-fatal warnings (e.g. bridge-failed notices
    /// when the cwd is not a git repository).
    pub quiet: bool,
    /// When true, skip the Ledgerful bridge query entirely and use only
    /// vault FTS5 + semantic search.
    pub no_bridge: bool,
    /// Optional one-shot override for the semantic cosine floor (soft F32).
    /// When set, overrides env / default for this recall only.
    pub min_semantic_score: Option<f64>,
    /// When true, keep T70 code-symbol stubs in the mix (CLI `--symbols`).
    /// Default **false**: exclude stubs from the candidate set (T260 F1).
    pub include_symbols: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecallHit {
    pub memory_id: String,
    pub content: String,
    pub source: String,
    pub score: Option<f64>,
    /// Privacy flag inherited from the source memory.
    pub privacy: Option<Privacy>,
    /// Session ID of the source memory, if any.
    pub session_id: Option<String>,
    /// Memory projection `updated_at` when known (FTS/substring/graph); bridge leaves None (F16).
    pub updated_at: Option<String>,
    /// True when this hit is a Plan-class DECISION demoted by ranking (T211 F11).
    pub is_plan_demoted: bool,
    /// How [`score`](Self::score) enters pin re-rank (T215 F6).
    pub score_kind: ScoreKind,
    /// Pre-fuse cosine similarity when known (T218 F4). RRF writes rank-score into
    /// [`score`](Self::score); this field preserves dense-arm sim for display / honesty.
    pub cosine: Option<f64>,
}

/// Stored score for a graph-neighbor hit given the parent's score and kind (T215 F-01 / F13).
///
/// `graph_boost` is historically a small additive on FTS/BM25-scale scores (~0.1).
/// For [`ScoreKind::HigherIsBetter`] (RRF / cosine), divide by [`RELEVANCE_SCALE`] so
/// the composite effective score gains only `+graph_boost` after scaling — not
/// `+graph_boost * RELEVANCE_SCALE` (which would swamp rank-1 RRF parents).
///
/// - [`ScoreKind::Bm25LowerBetter`]: `parent + graph_boost` (historical FTS path).
/// - [`ScoreKind::HigherIsBetter`]: `parent + graph_boost / RELEVANCE_SCALE`.
/// - [`ScoreKind::BridgeHigherIsBetter`]: `parent + graph_boost` (bridge base is raw unscaled).
pub fn graph_neighbor_stored_score(
    parent_score: Option<f64>,
    parent_kind: ScoreKind,
    graph_boost: f64,
) -> Option<f64> {
    use crate::ranking::RELEVANCE_SCALE;
    let base = parent_score.unwrap_or(0.0);
    let delta = match parent_kind {
        ScoreKind::HigherIsBetter => graph_boost / RELEVANCE_SCALE,
        ScoreKind::Bm25LowerBetter | ScoreKind::BridgeHigherIsBetter => graph_boost,
    };
    Some(base + delta)
}

impl RecallHit {
    /// Create a basic FTS5 hit with no privacy flag.
    pub fn fts(
        memory_id: String,
        content: String,
        score: Option<f64>,
        session_id: Option<String>,
        updated_at: Option<String>,
    ) -> Self {
        Self {
            memory_id,
            content,
            source: "fts".to_string(),
            score,
            privacy: None,
            session_id,
            updated_at,
            is_plan_demoted: false,
            score_kind: ScoreKind::Bm25LowerBetter,
            cosine: None,
        }
    }

    /// Create a hit from the substring LIKE fallback.
    pub fn substring(
        memory_id: String,
        content: String,
        session_id: Option<String>,
        updated_at: Option<String>,
    ) -> Self {
        Self {
            memory_id,
            content,
            source: "substring".to_string(),
            score: None,
            privacy: None,
            session_id,
            updated_at,
            is_plan_demoted: false,
            score_kind: ScoreKind::Bm25LowerBetter,
            cosine: None,
        }
    }

    /// Create a hit added via graph neighbor expansion.
    ///
    /// `score_kind` must be inherited from the parent hit (T215 F13 / AC16).
    pub fn graph(
        memory_id: String,
        content: String,
        score: Option<f64>,
        session_id: Option<String>,
        updated_at: Option<String>,
        score_kind: ScoreKind,
    ) -> Self {
        Self {
            memory_id,
            content,
            source: "graph".to_string(),
            score,
            privacy: None,
            session_id,
            updated_at,
            is_plan_demoted: false,
            score_kind,
            cosine: None,
        }
    }

    /// Create a hit from the unified IPC bridge.
    ///
    /// Bridge timestamps are not memory `updated_at` — leave None (F16).
    /// Score kind is [`ScoreKind::BridgeHigherIsBetter`] (M1 / F6).
    pub fn bridge(
        memory_id: String,
        content: String,
        score: Option<f64>,
        source: String,
        privacy: Option<Privacy>,
        session_id: Option<String>,
    ) -> Self {
        Self {
            memory_id,
            content,
            source,
            score,
            privacy,
            session_id,
            updated_at: None,
            is_plan_demoted: false,
            score_kind: ScoreKind::BridgeHigherIsBetter,
            cosine: None,
        }
    }

    /// Create a hit from dense embedding cosine similarity (T215 F42).
    ///
    /// Score kind is [`ScoreKind::HigherIsBetter`]; `score` is raw cosine in \[0, 1\].
    /// Pre-fuse cosine is also stored on [`RecallHit::cosine`] (T218 F4).
    pub fn semantic(
        memory_id: String,
        content: String,
        score: Option<f64>,
        privacy: Option<Privacy>,
        session_id: Option<String>,
        updated_at: Option<String>,
    ) -> Self {
        Self {
            memory_id,
            content,
            source: "semantic".to_string(),
            score,
            privacy,
            session_id,
            updated_at,
            is_plan_demoted: false,
            score_kind: ScoreKind::HigherIsBetter,
            cosine: score,
        }
    }
}

/// Full recall outcome including optional embedding status (T202).
#[derive(Debug, Clone)]
pub struct RecallOutcome {
    pub hits: Vec<RecallHit>,
    /// Set when `options.semantic` was true; `None` when semantic was not requested.
    pub embedding: Option<EmbeddingStatusDto>,
    /// When semantic requested: count of semantic hits that passed the **0.55
    /// hybrid-arm** cosine floor (**before** the dual semantic-only floor and
    /// before RRF; T218 F11/L3). `None` when semantic was not requested. Used
    /// for pretty F11 honesty (`ok` + zero post-threshold + FTS non-empty).
    pub semantic_post_threshold_count: Option<usize>,
}

/// Primary recall entry point. Attempts unified IPC recall via Ledgerful
/// (`bridge query`) first. If IPC is unavailable or fails, falls back to
/// local FTS5 search. Results from both sources are blended, with privacy
/// flags preserved from bridge hits.
///
/// When `semantic` is true, also queries via embedding-based semantic search
/// and blends those results alongside bridge and FTS5 hits.
///
/// Thin wrapper around [`recall_full`] for callers that only need hits.
pub fn recall(
    conn: &VaultConnection,
    graph: Option<&GraphSearch>,
    query: &str,
    limit: usize,
    options: RecallOptions,
) -> Result<Vec<RecallHit>> {
    Ok(recall_full(conn, graph, query, limit, options)?.hits)
}

/// Primary recall entry point with embedding status (T202).
///
/// Semantic backend failure does **not** abort whole recall (F3): FTS/bridge
/// results still return and `embedding` carries a closed status string.
pub fn recall_full(
    conn: &VaultConnection,
    graph: Option<&GraphSearch>,
    query: &str,
    limit: usize,
    options: RecallOptions,
) -> Result<RecallOutcome> {
    let project_id = options.project_id;
    let session_id = options.session_id;

    // T261 F7: 0-contentful is T207 empty — no bridge / LIKE / embed / graph.
    if is_contentless_query(query) {
        return Ok(RecallOutcome {
            hits: Vec::new(),
            embedding: if options.semantic {
                Some(EmbeddingStatusDto {
                    status: "skipped".to_string(),
                    endpoint: None,
                    detail: Some("contentless_query".to_string()),
                })
            } else {
                None
            },
            semantic_post_threshold_count: if options.semantic { Some(0) } else { None },
        });
    }

    // Sanitize for bridge only (T217 F10). Lexical builds MATCH from raw query
    // so OR rescue is not double-sanitized.
    let sanitized = sanitize_fts_query(query);

    // Phase 1: Try unified IPC recall via Ledgerful bridge query.
    // Cap bridge results at ceil(limit/2) to guarantee vault memories surface.
    let bridge_cap = limit.div_ceil(2);
    let bridge_hits = if options.no_bridge {
        Ok(Vec::new())
    } else {
        query_ledgerful_bridge(&sanitized, project_id, session_id)
    };

    // F9/T217: candidate depth bounds FTS MATCH LIMIT and semantic pool.
    let depth = candidate_depth(limit);
    let exclude_stubs = !options.include_symbols;

    // Phase 2: Local FTS5 with multi-token rescue ladder (T217; rescue opt-in).
    let mut local_hits: Vec<RecallHit> = lexical_search(
        conn,
        query,
        project_id,
        session_id,
        LexicalSearchOptions {
            rescue: true,
            limit: depth,
            exclude_symbol_stubs: exclude_stubs,
        },
    )?
    .into_iter()
    .map(|memory| {
        RecallHit::fts(
            memory.memory_id,
            memory.content,
            memory.score,
            memory.session_id,
            memory.updated_at,
        )
    })
    .collect();
    if exclude_stubs {
        crate::symbol_stub::retain_non_symbol_stubs(&mut local_hits);
    }

    // Phase 2b: If FTS ladder returned nothing, try a substring LIKE scan (T105).
    // Limited to small project scopes to avoid expensive full-table scans.
    if local_hits.is_empty() {
        let fallback =
            substring_fallback(conn, query, project_id, session_id, limit, exclude_stubs)?;
        if !fallback.is_empty() {
            local_hits = fallback
                .into_iter()
                .map(|memory| {
                    RecallHit::substring(
                        memory.memory_id,
                        memory.content,
                        memory.session_id,
                        memory.updated_at,
                    )
                })
                .collect();
            if exclude_stubs {
                crate::symbol_stub::retain_non_symbol_stubs(&mut local_hits);
            }
        }
    }

    // Phase 3: Semantic search when requested (soft-fail; structured status).
    let (semantic_hits, embedding_status, semantic_post_threshold_count): (
        Vec<RecallHit>,
        Option<EmbeddingStatusDto>,
        Option<usize>,
    ) = if options.semantic {
        match crate::semantic::semantic_search(
            conn,
            query,
            depth,
            project_id,
            session_id,
            options.min_semantic_score,
            exclude_stubs,
        ) {
            Ok(outcome) => {
                // F27: warn only for real embed soft-fails (not empty store).
                if matches!(outcome.embedding.status.as_str(), "unreachable" | "error") {
                    tracing::warn!(
                        status = %outcome.embedding.status,
                        detail = ?outcome.embedding.detail,
                        endpoint = ?outcome.embedding.endpoint,
                        "Semantic search soft-failed; continuing with lexical results"
                    );
                }
                let mut hits = outcome.hits;
                if exclude_stubs {
                    crate::symbol_stub::retain_non_symbol_stubs(&mut hits);
                }
                let n = hits.len();
                (hits, Some(outcome.embedding), Some(n))
            }
            Err(e) => {
                // Unexpected DB/store path: still soft-fail whole semantic arm.
                tracing::warn!(
                    error = %e,
                    "Semantic search failed, continuing with lexical results"
                );
                (Vec::new(), Some(classify_embedding_error(&e)), Some(0))
            }
        }
    } else {
        (Vec::new(), None, None)
    };

    #[cfg(not(feature = "graph"))]
    let _ = (graph, options.graph_boost, options.graph_hop_depth);

    // Phase 4: Blend (T215 F14).
    // When semantic: RRF(fts, semantic) → merge bridge (cap; bridge wins id) → graph → rerank.
    // When !semantic: bridge → FTS → graph → rerank (no RRF); ScoreKind still correct.
    let mut seen_ids = std::collections::HashSet::new();
    let mut blended = Vec::new();

    let push_bridge = |blended: &mut Vec<RecallHit>,
                       seen: &mut std::collections::HashSet<String>| {
        match &bridge_hits {
            Ok(bridge) => {
                for hit in bridge.iter().take(bridge_cap) {
                    if seen.insert(hit.memory_id.clone()) {
                        blended.push(hit.clone());
                    }
                }
            }
            Err(e) => {
                if !options.quiet {
                    eprintln!(
                        "Ledgerful bridge query failed, falling back to local FTS5 only: {}",
                        e
                    );
                }
            }
        }
    };

    if options.semantic {
        // F37/F41 + dual floor: production SOOT is fuse_local_and_semantic
        // (apply_dual_semantic_floor + FTS-only RRF + substring outside).
        // semantic_post_threshold_count is already post-0.55 / pre-dual-floor.
        let fused_local = fuse_local_and_semantic(
            &local_hits,
            semantic_hits,
            options.min_semantic_score,
            depth,
            rrf_k(),
        );

        // Bridge first (wins on id), then fused FTS/semantic + substring rest.
        push_bridge(&mut blended, &mut seen_ids);
        for hit in fused_local {
            if seen_ids.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }
    } else {
        // !semantic: bridge → FTS/substring (no RRF).
        push_bridge(&mut blended, &mut seen_ids);
        for hit in local_hits {
            if seen_ids.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }
    }

    // T260 F8 / §5.3: drop stub-shaped bridge Insights *before* they can seed
    // graph neighbors. Retain again after graph for stub-shaped neighbors.
    if exclude_stubs {
        crate::symbol_stub::retain_non_symbol_stubs(&mut blended);
    }

    // Graph-based neighbor expansion: for each current hit, fetch 1-hop
    // neighbors and add unseen ones with a boosted score. After RRF+bridge (F13).
    #[cfg(feature = "graph")]
    if options.graph_hop_depth >= 1
        && let Some(searcher) = graph
    {
        let mut graph_hits: Vec<RecallHit> = Vec::new();
        // Snapshot existing hits (id, score, score_kind) for parent inheritance.
        let existing: Vec<(String, Option<f64>, ScoreKind)> = blended
            .iter()
            .map(|h| (h.memory_id.clone(), h.score, h.score_kind))
            .collect();

        for (parent_id, parent_score, parent_kind) in existing {
            let neighbors = match searcher.get_neighbors(&parent_id) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Graph neighbor lookup failed for {}: {}", parent_id, e);
                    continue;
                }
            };
            for neighbor in neighbors {
                if !seen_ids.contains(&neighbor.external_id) {
                    // Fetch content + updated_at from memory_projection (F16/F38).
                    let row_opt: Option<(String, Option<String>)> = {
                        let db = conn.lock().ok();
                        db.and_then(|c| {
                            c.query_row(
                                "SELECT content, updated_at FROM memory_projection WHERE memory_id = ?1",
                                rusqlite::params![neighbor.external_id],
                                |row| {
                                    Ok((
                                        row.get::<_, String>(0)?,
                                        row.get::<_, Option<String>>(1)?,
                                    ))
                                },
                            )
                            .ok()
                        })
                    };
                    if let Some((content, updated_at)) = row_opt {
                        // F13 / F-01: boost in composite/effective space for HigherIsBetter
                        // parents so graph_boost (~0.1 BM25-era) is not RELEVANCE_SCALE'd.
                        let boost_score = graph_neighbor_stored_score(
                            parent_score,
                            parent_kind,
                            options.graph_boost,
                        );
                        seen_ids.insert(neighbor.external_id.clone());
                        graph_hits.push(RecallHit::graph(
                            neighbor.external_id,
                            content,
                            boost_score,
                            None,
                            updated_at,
                            parent_kind,
                        ));
                    }
                }
            }
        }
        blended.extend(graph_hits);
    }

    if exclude_stubs {
        crate::symbol_stub::retain_non_symbol_stubs(&mut blended);
    }

    // T211: pin-type + recency composite re-rank (F8). Single post-blend entry
    // point (F40) — ScoreKind-aware (T215). Truncate after. T260 F3: stub
    // content-dedupe runs after this sort, never before.
    crate::ranking::rerank_hits(&mut blended);
    crate::symbol_stub::dedupe_symbol_stubs(&mut blended);

    if blended.len() > limit {
        blended.truncate(limit);
    }

    Ok(RecallOutcome {
        hits: blended,
        embedding: embedding_status,
        semantic_post_threshold_count,
    })
}

// ---------------------------------------------------------------------------
// Unified IPC Bridge Query
// ---------------------------------------------------------------------------

/// Query Ledgerful's Tantivy search via `ledgerful search --json`.
/// Parses the NDJSON response for `BridgeRecord::Insight` entries.
///
/// Returns Ok(Vec) on success. On any failure (CLI missing, non-zero exit,
/// parse errors), returns an Err so the caller can fall back to local FTS5.
fn bridge_search_args(query: &str) -> Vec<&str> {
    vec!["search", "--auto-index", "--json", query]
}

#[allow(clippy::disallowed_methods)]
fn query_ledgerful_bridge(
    query: &str,
    _project_id: Option<ai_brains_core::ids::ProjectId>,
    _session_id: Option<ai_brains_core::ids::SessionId>,
) -> std::result::Result<Vec<RecallHit>, String> {
    let output = std::process::Command::new("ledgerful")
        .args(bridge_search_args(query))
        .output()
        .map_err(|e| format!("ledgerful CLI not available: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ledgerful search failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut hits = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Defensive: stdout may contain tracing/log lines mixed with NDJSON.
        // Skip anything that does not look like a JSON object or array.
        if !line.starts_with('{') && !line.starts_with('[') {
            continue;
        }

        let record: BridgeRecord = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_e) => {
                // Silently skip malformed JSON lines mixed into stdout.
                continue;
            }
        };

        let record_kind = record.record_kind.to_lowercase();
        if record_kind != "insight" && record_kind != "bm25_match" {
            continue;
        }

        let payload = record.payload_value();
        let memory_id = payload
            .get("memory_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let content = payload
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let score = payload.get("relevance").and_then(|v| v.as_f64());
        let source = payload
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("bridge")
            .to_string();
        let privacy = Some(record.privacy);
        let session_id = record.session_id.clone().filter(|s| !s.is_empty());

        if !content.is_empty() {
            hits.push(RecallHit::bridge(
                memory_id, content, score, source, privacy, session_id,
            ));
        }
    }

    Ok(hits)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // test-only expect/unwrap OK
mod tests {
    use super::*;
    use crate::ranking::{PinKind, RELEVANCE_SCALE, StalenessClass, effective_score};

    /// F-01: HigherIsBetter graph boost must not swamp parent RRF after RELEVANCE_SCALE.
    #[test]
    #[allow(non_snake_case)]
    fn graph_neighbor__higher_is_better_boost_does_not_swamp_parent__f01() {
        let graph_boost = 0.1;
        // Rank-1 RRF alone under k=60.
        let parent_raw = 1.0 / 61.0;
        let parent_score = Some(parent_raw);
        let neighbor_score =
            graph_neighbor_stored_score(parent_score, ScoreKind::HigherIsBetter, graph_boost)
                .expect("score");

        // Stored: parent + boost/SCALE — not parent + raw boost.
        let expected_stored = parent_raw + graph_boost / RELEVANCE_SCALE;
        assert!(
            (neighbor_score - expected_stored).abs() < 1e-12,
            "neighbor_score={neighbor_score} expected={expected_stored}"
        );

        let parent_eff = effective_score(
            parent_score,
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            ScoreKind::HigherIsBetter,
        );
        let neighbor_eff = effective_score(
            Some(neighbor_score),
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            ScoreKind::HigherIsBetter,
        );
        // Effective neighbor is only ~graph_boost above parent (~0.1), not ~50+.
        let delta = neighbor_eff - parent_eff;
        assert!(
            (delta - graph_boost).abs() < 1e-9,
            "effective delta={delta} must be ~{graph_boost}, not swamp parent"
        );
        assert!(
            delta < 1.0,
            "graph neighbor must not leapfrog by RELEVANCE_SCALE*boost; delta={delta}"
        );

        // Legacy bug path: raw +0.1 stored under HigherIsBetter → delta ≈ 50.
        let buggy_eff = effective_score(
            Some(parent_raw + graph_boost),
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            ScoreKind::HigherIsBetter,
        );
        assert!(
            buggy_eff - parent_eff > 40.0,
            "sanity: unscaled boost would swamp (got {})",
            buggy_eff - parent_eff
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn graph_neighbor_stored_score__bm25_and_bridge_add_raw_boost() {
        let boost = 0.1;
        let bm25 = graph_neighbor_stored_score(Some(-2.0), ScoreKind::Bm25LowerBetter, boost);
        assert_eq!(bm25, Some(-1.9));
        let bridge =
            graph_neighbor_stored_score(Some(18.0), ScoreKind::BridgeHigherIsBetter, boost);
        assert_eq!(bridge, Some(18.1));
    }

    #[test]
    fn recall_hit_fts_constructor() {
        let hit = RecallHit::fts(
            "mem-1".into(),
            "test content".into(),
            Some(0.85),
            None,
            None,
        );
        assert_eq!(hit.memory_id, "mem-1");
        assert_eq!(hit.source, "fts");
        assert_eq!(hit.score, Some(0.85));
        assert_eq!(hit.privacy, None);
        assert_eq!(hit.session_id, None);
        assert_eq!(hit.updated_at, None);
        assert!(!hit.is_plan_demoted);
        assert_eq!(hit.score_kind, ScoreKind::Bm25LowerBetter);
    }

    #[test]
    fn recall_hit_fts_constructor_with_session_id() {
        let hit = RecallHit::fts(
            "mem-1".into(),
            "test content".into(),
            Some(0.85),
            Some("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string()),
            None,
        );
        assert_eq!(
            hit.session_id,
            Some("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string())
        );
    }

    #[test]
    fn recall_hit_bridge_constructor() {
        let hit = RecallHit::bridge(
            "mem-2".into(),
            "bridge content".into(),
            Some(0.92),
            "code_context".into(),
            Some(Privacy::LocalOnly),
            None,
        );
        assert_eq!(hit.memory_id, "mem-2");
        assert_eq!(hit.source, "code_context");
        assert_eq!(hit.score, Some(0.92));
        assert_eq!(hit.privacy, Some(Privacy::LocalOnly));
        assert_eq!(hit.session_id, None);
        assert_eq!(hit.score_kind, ScoreKind::BridgeHigherIsBetter);
    }

    #[test]
    fn recall_hit_semantic_constructor() {
        let hit = RecallHit::semantic(
            "mem-3".into(),
            "sem content".into(),
            Some(0.77),
            Some(Privacy::LocalOnly),
            Some("sess".into()),
            Some("2026-01-01T00:00:00Z".into()),
        );
        assert_eq!(hit.source, "semantic");
        assert_eq!(hit.score, Some(0.77));
        assert_eq!(hit.score_kind, ScoreKind::HigherIsBetter);
        assert_eq!(hit.updated_at.as_deref(), Some("2026-01-01T00:00:00Z"));
        // T218 F4: pre-fuse cosine mirrors score on semantic constructor.
        assert_eq!(hit.cosine, Some(0.77));
    }

    #[test]
    fn recall_hit_fts_constructor_has_no_cosine() {
        let hit = RecallHit::fts("mem-1".into(), "test".into(), Some(-2.0), None, None);
        assert_eq!(hit.cosine, None);
    }

    #[test]
    fn recall_hit_bridge_constructor_with_session_id() {
        let hit = RecallHit::bridge(
            "mem-2".into(),
            "bridge content".into(),
            Some(0.92),
            "code_context".into(),
            Some(Privacy::LocalOnly),
            Some("session-123".into()),
        );
        assert_eq!(hit.session_id, Some("session-123".to_string()));
    }

    #[test]
    #[allow(non_snake_case)]
    fn bridge_session_id_normalization__empty_string_becomes_none() {
        let raw = Some("".to_string());
        let normalized = raw.filter(|s| !s.is_empty());
        assert_eq!(normalized, None);
    }

    #[test]
    fn bridge_search_args_includes_auto_index_flag_and_query() {
        let args = bridge_search_args("some query");
        assert_eq!(args, vec!["search", "--auto-index", "--json", "some query"]);
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
                None,
            ),
            RecallHit::bridge(
                "mem-2".into(),
                "c2".into(),
                Some(0.8),
                "bridge".into(),
                None,
                None,
            ),
        ];

        let local_fts = vec![
            RecallHit::fts("mem-2".into(), "c2-fts".into(), Some(0.7), None, None),
            RecallHit::fts("mem-3".into(), "c3".into(), Some(0.6), None, None),
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
    fn bridge_cap_leaves_room_for_vault_hits() {
        // With limit=4, bridge cap = ceil(4/2) = 2.
        // Even if bridge returns 5 hits, only 2 make it; the other 2 slots go to vault FTS.
        let bridge_cap = 4_usize.div_ceil(2);
        assert_eq!(bridge_cap, 2);

        let many_bridge: Vec<RecallHit> = (0..5)
            .map(|i| {
                RecallHit::bridge(
                    format!("bridge-{}", i),
                    format!("content {}", i),
                    Some(0.9 - i as f64 * 0.05),
                    "bridge".into(),
                    None,
                    None,
                )
            })
            .collect();

        let vault_hits: Vec<RecallHit> = (0..3)
            .map(|i| {
                RecallHit::fts(
                    format!("vault-{}", i),
                    format!("vault {}", i),
                    Some(0.5),
                    None,
                    None,
                )
            })
            .collect();

        let mut seen = std::collections::HashSet::new();
        let mut blended = Vec::new();

        let mut capped = many_bridge;
        capped.truncate(bridge_cap);
        for hit in capped {
            if seen.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }
        for hit in vault_hits {
            if seen.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }
        blended.truncate(4);

        let vault_count = blended.iter().filter(|h| h.source == "fts").count();
        assert!(
            vault_count >= 1,
            "At least one vault memory must appear; got {}",
            vault_count
        );
        assert_eq!(blended.len(), 4);
    }

    #[test]
    fn blending_preserves_privacy_flags_from_bridge() {
        let bridge_hits = vec![RecallHit::bridge(
            "mem-private".into(),
            "secret".into(),
            Some(1.0),
            "bridge".into(),
            Some(Privacy::NeverInject),
            None,
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
