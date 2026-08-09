mod adapter;
pub mod agy;
mod antigravity;
mod capability;
mod claude;
mod codex;
mod config_patch;
mod errors;
mod gemini;
pub mod grok;
mod hook_output;
mod install;
pub mod message_only;
mod neutral_event;
mod opencode;
mod wrapper;

pub use adapter::{AdapterKind, adapter_capability};
pub use agy::{
    AGY_UNBOUND_ALIAS, AGY_UNBOUND_DISPLAY_NAME, TranscriptIngestTurn, agy_env_fallback_allowed,
    agy_source_meta_key, generate_deterministic_turn_id, generate_turn_id_for_ingest,
    normalize_agy_project_hash, parse_transcript_for_ingest, path_derived_display_name,
    prefer_full_transcript_path,
};
pub use antigravity::{
    AgyBindKind, AntigravityFormat, AntigravityImportOptions, AntigravityImportStats,
    AntigravitySessionSource, AntigravityStep, AntigravityTurn, antigravity_capability,
    discover_sessions, discover_sessions_from_home, extract_turns, import_antigravity_sessions,
    load_agy_history_index, load_agy_history_index_from_home, manual_import_instructions,
    parse_overview_file, parse_project_chat_file, print_import_stats, resolve_agy_project,
    session_id_from_path, strip_user_xml_tags,
};
pub use capability::{AdapterCapability, CapabilityLevel};
pub use claude::parse_claude_stop_payload;
pub use config_patch::apply_idempotent_patch;
pub use errors::{AdapterError, Result};
pub use grok::{
    GROK_HARNESS_UUID, GROK_UNBOUND_ALIAS, GROK_UNBOUND_DISPLAY_NAME, GrokBindKind,
    GrokImportOptions, GrokImportStats, GrokSessionSource, append_grok_turns,
    discover_grok_sessions, generate_grok_turn_id, grok_capability, grok_env_fallback_allowed,
    grok_source_meta_key, import_grok_sessions, is_subagent_session, normalize_grok_project_hash,
    parse_chat_history_file, percent_decode_component, percent_encode_path_component,
    print_grok_import_stats, resolve_chat_history_path, resolve_grok_home, resolve_grok_project,
};
pub use hook_output::render_hook_output;
pub use install::install_scope;
pub use message_only::{
    AntigravityStepInput, DropReason, IngestRole, IngestableTurn, classify_antigravity_step,
    extract_text_from_json_content, extract_user_text, filter_agy_simple_lines,
    filter_agy_simple_turn, filter_antigravity_steps, filter_grok_history_lines,
    filter_grok_history_record, filter_opencode_message, filter_opencode_message_lines,
    filter_opencode_messages, filter_turn, filter_turn_with_ts,
};
pub use neutral_event::NeutralEvent;
pub use wrapper::wrapper_command;
