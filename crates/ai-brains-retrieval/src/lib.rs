mod ansi;
mod errors;
mod fts_utils;
mod hybrid;
mod lexical;
mod prefer_project;
mod preflight;
mod preflight_global;
mod preflight_safety;
mod privacy_filter;
mod ranking;
mod recall;
mod semantic;
mod session_chrome;
mod sessions;
mod symbol_stub;
mod word_budget;

pub use ai_brains_core::LEXICAL_MATCH_HARD_CAP;
pub use ansi::strip_ansi;
pub use errors::{Result, RetrievalError};
pub use fts_utils::sanitize_fts_query;
pub use hybrid::{
    RRF_K, SEMANTIC_MIN_COSINE, SEMANTIC_ONLY_MIN_COSINE, apply_dual_semantic_floor,
    candidate_depth, effective_semantic_min_cosine, effective_semantic_only_min_cosine,
    filter_by_cosine_floor, fuse_local_and_semantic, has_fts_arm, rrf_fuse, rrf_k,
    semantic_min_cosine, semantic_only_min_cosine,
};
pub use lexical::{
    LexicalSearchOptions, RetrievalMemory, index_authority_fill, lexical_search, match_limit_bound,
    substring_fallback,
};
pub use prefer_project::merge_preferred_then_global;
pub use preflight::{
    PreflightContext, build_preflight, build_preflight_with_options, first_index_decision_content,
    governed_briefing_enabled,
};
pub use preflight_safety::{SAFETY_EMPTY, keep_repo_local_hotspot};
pub use ranking::{
    LEADING_QUERY_BONUS, PinKind, RELEVANCE_SCALE, SESSION_CHROME_PENALTY, SYMBOL_PENALTY,
    ScoreKind, StalenessClass, classify_pin_kind, classify_staleness, extract_track_tokens,
    first_contentful_line, rerank_hits, rerank_hits_with_query, strip_assistant_prefix,
};
pub use recall::{
    RecallHit, RecallOptions, RecallOutcome, graph_neighbor_stored_score, merge_bridge_then_local,
    recall, recall_full,
};
pub use semantic::{
    SemanticOutcome, classify_embedding_error, classify_model_error, embedding_endpoint,
    embedding_model, public_endpoint_label, semantic_search, semantic_search_with_embedding,
    status_after_embed_ok,
};
pub use session_chrome::{
    DUMP_OTHER_CHAR_FLOOR, DUMP_OTHER_PENALTY, authority_glob_sql, bound_not_in_sql,
    dedupe_session_chrome, index_marker_glob_sql, index_pass1_glob_sql, is_authority_pin_content,
    is_session_chrome, is_verbose_other_dump, parent_seeds_graph_neighbors, prefer_authority_hits,
    safety_marker_glob_sql, tags_envelope_sql,
};
pub use sessions::active_sessions;
pub use symbol_stub::{
    SYMBOL_KINDS, dedupe_symbol_stubs, is_symbol_stub_content, retain_non_symbol_stubs,
    symbol_stub_sql_exclusion,
};
pub use word_budget::{
    content_word_count, trim_to_word_budget, trim_to_word_budget_no_sentinel, word_count,
};

#[cfg(not(feature = "graph"))]
pub struct MockGraphSearch;

#[cfg(feature = "graph")]
pub use ai_brains_graph::queries::GraphSearch;

#[cfg(not(feature = "graph"))]
pub type GraphSearch = MockGraphSearch;
