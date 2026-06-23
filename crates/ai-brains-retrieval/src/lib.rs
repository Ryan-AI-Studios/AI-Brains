mod ansi;
mod errors;
mod fts_utils;
mod lexical;
mod preflight;
mod privacy_filter;
mod recall;
mod semantic;
mod sessions;
mod word_budget;

pub use ansi::strip_ansi;
pub use errors::{Result, RetrievalError};
pub use fts_utils::sanitize_fts_query;
pub use lexical::{lexical_search, substring_fallback, RetrievalMemory};
pub use preflight::{build_preflight, PreflightContext};
pub use recall::{recall, RecallHit, RecallOptions};
pub use sessions::active_sessions;

#[cfg(not(feature = "graph"))]
pub struct MockGraphSearch;

#[cfg(feature = "graph")]
pub use ai_brains_graph::queries::GraphSearch;

#[cfg(not(feature = "graph"))]
pub type GraphSearch = MockGraphSearch;
