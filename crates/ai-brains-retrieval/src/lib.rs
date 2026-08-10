mod ansi;
mod errors;
mod fts_utils;
mod hybrid;
mod lexical;
mod preflight;
mod privacy_filter;
mod ranking;
mod recall;
mod semantic;
mod sessions;
mod word_budget;

pub use ai_brains_core::LEXICAL_MATCH_HARD_CAP;
pub use ansi::strip_ansi;
pub use errors::{Result, RetrievalError};
pub use fts_utils::sanitize_fts_query;
pub use hybrid::{
    RRF_K, SEMANTIC_MIN_COSINE, SEMANTIC_ONLY_MIN_COSINE, apply_dual_semantic_floor,
    candidate_depth, effective_semantic_min_cosine, effective_semantic_only_min_cosine,
    filter_by_cosine_floor, has_fts_arm, rrf_fuse, rrf_k, semantic_min_cosine,
    semantic_only_min_cosine,
};
pub use lexical::{
    LexicalSearchOptions, RetrievalMemory, lexical_search, match_limit_bound, substring_fallback,
};
pub use preflight::{
    PreflightContext, build_preflight, build_preflight_with_options, governed_briefing_enabled,
};
pub use ranking::{
    PinKind, RELEVANCE_SCALE, ScoreKind, StalenessClass, classify_pin_kind, classify_staleness,
    extract_track_tokens, rerank_hits, strip_assistant_prefix,
};
pub use recall::{
    RecallHit, RecallOptions, RecallOutcome, graph_neighbor_stored_score, recall, recall_full,
};
pub use semantic::{
    SemanticOutcome, classify_embedding_error, classify_model_error, embedding_endpoint,
    embedding_model, public_endpoint_label, semantic_search, semantic_search_with_embedding,
    status_after_embed_ok,
};
pub use sessions::active_sessions;

#[cfg(not(feature = "graph"))]
pub struct MockGraphSearch;

#[cfg(feature = "graph")]
pub use ai_brains_graph::queries::GraphSearch;

#[cfg(not(feature = "graph"))]
pub type GraphSearch = MockGraphSearch;
