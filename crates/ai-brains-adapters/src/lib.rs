mod adapter;
pub mod agy;
mod antigravity;
mod capability;
pub mod claude;
pub mod codex;
mod config_patch;
pub mod cursor;
mod errors;
mod gemini;
pub mod grok;
mod hook_output;
mod install;
pub mod message_only;
mod neutral_event;
pub mod opencode;
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
pub use claude::{
    CLAUDE_HARNESS_UUID, CLAUDE_UNBOUND_ALIAS, CLAUDE_UNBOUND_DISPLAY_NAME, ClaudeBindKind,
    ClaudeHookMapped, ClaudeHookPayload, ClaudeImportOptions, ClaudeImportStats, ClaudeIngestTurn,
    ClaudeSessionSource, accept_claude_live_payload, append_claude_turns, claude_capability,
    claude_env_fallback_allowed, claude_source_meta_key, decode_claude_project_folder,
    discover_claude_sessions, filter_claude_jsonl_lines, filter_claude_jsonl_record,
    generate_claude_live_turn_id, generate_claude_turn_id, import_claude_sessions,
    is_claude_sidechain_path, map_claude_hook_payload, normalize_claude_project_hash,
    parse_claude_hook_payload_strict, parse_claude_jsonl_file, parse_claude_stop_payload,
    print_claude_import_stats, resolve_claude_home, resolve_claude_project, session_id_from_claude,
};
pub use codex::{
    CODEX_HARNESS_UUID, CODEX_UNBOUND_ALIAS, CODEX_UNBOUND_DISPLAY_NAME, CodexBindKind,
    CodexHookMapped, CodexHookPayload, CodexImportOptions, CodexImportStats, CodexIngestTurn,
    CodexSessionSource, accept_codex_live_payload, append_codex_turns, codex_capability,
    codex_env_fallback_allowed, codex_source_meta_key, discover_codex_sessions,
    filter_codex_rollout_lines, filter_codex_rollout_record, generate_codex_live_turn_id,
    generate_codex_turn_id, import_codex_sessions, map_codex_hook_payload,
    normalize_codex_project_hash, parse_codex_hook_payload_strict, parse_codex_rollout_file,
    print_codex_import_stats, resolve_codex_home, resolve_codex_project, session_id_from_codex,
};
pub use config_patch::apply_idempotent_patch;
pub use cursor::{
    CURSOR_HARNESS_UUID, CURSOR_UNBOUND_ALIAS, CURSOR_UNBOUND_DISPLAY_NAME, CursorBindKind,
    CursorImportOptions, CursorImportStats, CursorIngestTurn, CursorSessionSource,
    append_cursor_turns, cursor_capability, cursor_project_slug, cursor_project_slug_candidates,
    cursor_source_meta_key, discover_cursor_sessions, filter_cursor_jsonl_lines,
    filter_cursor_jsonl_record, generate_cursor_turn_id, import_cursor_sessions,
    is_cursor_sidechain_path, parse_cursor_jsonl_file, print_cursor_import_stats,
    resolve_cursor_home, resolve_cursor_project, session_id_from_cursor,
};
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
    AntigravityStepInput, DropReason, IngestRole, IngestableTurn, OpenCodeIngestTurn,
    classify_antigravity_step, extract_text_from_json_content, extract_user_text,
    filter_agy_simple_lines, filter_agy_simple_turn, filter_antigravity_steps,
    filter_grok_history_lines, filter_grok_history_record, filter_opencode_export,
    filter_opencode_message, filter_opencode_message_lines, filter_opencode_message_with_id,
    filter_opencode_messages, filter_turn, filter_turn_with_ts, normalize_opencode_export_message,
};
pub use neutral_event::NeutralEvent;
pub use opencode::{
    OPENCODE_BIN_ENV, OPENCODE_BIN_PATH_ENV, OPENCODE_EXPORT_TIMEOUT_SECS, OPENCODE_HARNESS_UUID,
    OPENCODE_LIST_DEFAULT_CAP, OPENCODE_UNBOUND_ALIAS, OPENCODE_UNBOUND_DISPLAY_NAME,
    OpenCodeBindKind, OpenCodeImportOptions, OpenCodeImportStats, OpenCodeSessionSource,
    ResolveOutcome, append_opencode_turns, export_session_via_cli, generate_opencode_turn_id,
    import_opencode_sessions, normalize_opencode_project_hash, opencode_capability,
    opencode_env_fallback_allowed, opencode_source_meta_key, parse_export_file, parse_export_json,
    print_opencode_import_stats, resolve_opencode_bin, resolve_opencode_config_dir,
    resolve_opencode_project, session_id_from_opencode,
};
pub use wrapper::wrapper_command;
