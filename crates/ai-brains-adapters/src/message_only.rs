//! Shared message-only capture contract (T234).
//!
//! Capture Privacy SOOT: keep **user prompt text** and **final assistant text** only.
//! Drop tool calls/results, thinking/reasoning, system chrome, and harness metadata
//! wrappers (extract inner user text when present).
//!
//! Pure string/JSON logic — no models, embeddings, graph, or network.

use serde_json::Value;

/// Role allowed after message-only normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestRole {
    User,
    Assistant,
}

impl IngestRole {
    /// Canonical role string for ingest (`user` / `assistant`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

/// A turn ready for vault ingest after message-only filtering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestableTurn {
    pub role: IngestRole,
    pub content: String,
    pub source_ts: Option<String>,
}

/// Why a candidate turn was dropped (tests / debug; not required on happy path).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropReason {
    DisallowedRole,
    EmptyContent,
    ToolStep,
    ThinkingOrReasoning,
    SystemChrome,
    NoExtractableText,
}

// ---------------------------------------------------------------------------
// Core entry points
// ---------------------------------------------------------------------------

/// Filter a simple role+content pair (agy `{role,content}` JSONL and generic paths).
///
/// User content runs through [`extract_user_text`] (XML / `<user_query>` chrome).
/// Assistant content is trimmed only (multipart extraction is harness-level via
/// [`extract_text_from_json_content`]).
pub fn filter_turn(role: &str, content: &str) -> Option<IngestableTurn> {
    filter_turn_with_ts(role, content, None)
}

/// Same as [`filter_turn`] with optional source timestamp.
pub fn filter_turn_with_ts(
    role: &str,
    content: &str,
    source_ts: Option<String>,
) -> Option<IngestableTurn> {
    let ingest_role = match role.trim().to_ascii_lowercase().as_str() {
        "user" => IngestRole::User,
        "assistant" => IngestRole::Assistant,
        "system" | "tool" | "function" | "reasoning" | "thinking" => return None,
        _ => return None,
    };

    let cleaned = match ingest_role {
        IngestRole::User => extract_user_text(content),
        IngestRole::Assistant => content.trim().to_string(),
    };

    if cleaned.is_empty() {
        return None;
    }

    // F15: drop sole tool JSON payloads masquerading as assistant/user text.
    // Legitimate prose that merely *mentions* tool_calls is kept (not a substring ban).
    if is_sole_tool_json_payload(&cleaned) {
        return None;
    }

    Some(IngestableTurn {
        role: ingest_role,
        content: cleaned,
        source_ts,
    })
}

/// True when `content` is a JSON object whose sole purpose is a tool payload (F15).
fn is_sole_tool_json_payload(content: &str) -> bool {
    let trimmed = content.trim();
    if !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return false;
    }
    let Ok(Value::Object(map)) = serde_json::from_str::<Value>(trimmed) else {
        return false;
    };
    if map.is_empty() {
        return false;
    }

    let has_tool_key = map.contains_key("tool_calls")
        || map.contains_key("tool_result")
        || map.contains_key("tool_use")
        || map.contains_key("function_call")
        || map.contains_key("backend_tool_call");

    if !has_tool_key {
        // type-tagged tool records without nested keys
        if let Some(t) = map.get("type").and_then(|v| v.as_str()) {
            return matches!(
                t,
                "tool_result"
                    | "tool_call"
                    | "tool_use"
                    | "function_call"
                    | "backend_tool_call"
                    | "function"
            );
        }
        return false;
    }

    // If a human-facing text/content string is also present and non-empty, keep
    // (interleaved payloads should use multipart extractors, not this sole-payload path).
    let has_human_text = ["content", "text", "message"].iter().any(|k| {
        map.get(*k)
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.trim().is_empty())
    });
    !has_human_text
}

/// Filter an already-parsed agy simple turn (`{role, content, timestamp?}`).
pub fn filter_agy_simple_turn(
    role: &str,
    content: &str,
    timestamp: Option<String>,
) -> Option<IngestableTurn> {
    filter_turn_with_ts(role, content, timestamp)
}

/// Filter lines of simple `{role, content}` JSONL (malformed lines skipped).
pub fn filter_agy_simple_lines(jsonl: &str) -> Vec<IngestableTurn> {
    let mut out = Vec::new();
    for line in jsonl.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(role) = value.get("role").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(content) = value.get("content").and_then(|v| v.as_str()) else {
            // content may be non-string; try multipart extract for assistant/user
            let extracted = value
                .get("content")
                .and_then(extract_text_from_json_content);
            let Some(text) = extracted else {
                continue;
            };
            if let Some(turn) = filter_turn_with_ts(
                role,
                &text,
                value
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            ) {
                out.push(turn);
            }
            continue;
        };
        if let Some(turn) = filter_turn_with_ts(
            role,
            content,
            value
                .get("timestamp")
                .and_then(|v| v.as_str())
                .map(str::to_string),
        ) {
            out.push(turn);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Antigravity / AGY step classification (F7)
// ---------------------------------------------------------------------------

/// Classify a single Antigravity/AGY step by strict `(source, step_type)`.
///
/// Drops VIEW_FILE / RUN_COMMAND / TOOL_OUTPUT / SYSTEM / CHECKPOINT **regardless
/// of content**. Keeps USER_EXPLICIT/USER_INPUT and non-empty PLANNER_RESPONSE text.
pub fn classify_antigravity_step(
    source: &str,
    step_type: &str,
    content: Option<&str>,
    _tool_calls: &[Value],
    created_at: Option<&str>,
) -> Option<IngestableTurn> {
    match (source, step_type) {
        ("USER_EXPLICIT", "USER_INPUT") => {
            let raw = content.unwrap_or("");
            let cleaned = extract_user_text(raw);
            if cleaned.is_empty() {
                return None;
            }
            Some(IngestableTurn {
                role: IngestRole::User,
                content: cleaned,
                source_ts: created_at.map(str::to_string),
            })
        }
        ("MODEL", "PLANNER_RESPONSE") => {
            // Visible text only; tool-only (empty / sole tool JSON) dropped. Thinking never stored.
            let text = content.map(str::trim).unwrap_or("");
            if text.is_empty() || is_sole_tool_json_payload(text) {
                return None;
            }
            Some(IngestableTurn {
                role: IngestRole::Assistant,
                content: text.to_string(),
                source_ts: created_at.map(str::to_string),
            })
        }
        // Drop regardless of content (F7 / AC16): tool steps, chrome, other types
        ("MODEL", "VIEW_FILE")
        | ("MODEL", "RUN_COMMAND")
        | ("MODEL", "TOOL_OUTPUT")
        | (_, "VIEW_FILE")
        | (_, "RUN_COMMAND")
        | (_, "TOOL_OUTPUT")
        | (_, "SYSTEM")
        | (_, "CHECKPOINT")
        | ("SYSTEM", _) => None,
        _ => None,
    }
}

/// Borrowed Antigravity/AGY step fields for batch filter (avoids coupling to serde types).
#[derive(Debug, Clone, Copy)]
pub struct AntigravityStepInput<'a> {
    pub source: &'a str,
    pub step_type: &'a str,
    pub content: Option<&'a str>,
    pub tool_calls: &'a [Value],
    pub created_at: Option<&'a str>,
}

/// Filter a slice of Antigravity steps into ingestable turns (order preserved).
pub fn filter_antigravity_steps(steps: &[AntigravityStepInput<'_>]) -> Vec<IngestableTurn> {
    steps
        .iter()
        .filter_map(|s| {
            classify_antigravity_step(s.source, s.step_type, s.content, s.tool_calls, s.created_at)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Grok chat_history (F8 / F10 / F37)
// ---------------------------------------------------------------------------

/// Filter one Grok `chat_history.jsonl` record (`type` + `content` string|array).
pub fn filter_grok_history_record(record: &Value) -> Option<IngestableTurn> {
    let type_str = record
        .get("type")
        .and_then(|v| v.as_str())
        .or_else(|| record.get("role").and_then(|v| v.as_str()))?;

    match type_str {
        "user" => {
            // Synthetic chrome: empty after extract_user_text → drop (F8; fuller taxonomy soft F24).
            let text = record
                .get("content")
                .and_then(extract_text_from_json_content)
                .map(|s| extract_user_text(&s))
                .unwrap_or_default();
            if text.is_empty() {
                return None;
            }
            let ts = record
                .get("timestamp")
                .or_else(|| record.get("created_at"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            Some(IngestableTurn {
                role: IngestRole::User,
                content: text,
                source_ts: ts,
            })
        }
        "assistant" => {
            let text = record
                .get("content")
                .and_then(extract_text_from_json_content)
                .unwrap_or_default();
            let text = text.trim().to_string();
            if text.is_empty() || is_sole_tool_json_payload(&text) {
                return None;
            }
            let ts = record
                .get("timestamp")
                .or_else(|| record.get("created_at"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            Some(IngestableTurn {
                role: IngestRole::Assistant,
                content: text,
                source_ts: ts,
            })
        }
        "reasoning" | "tool_result" | "backend_tool_call" | "system" | "function" | "tool" => None,
        _ => None,
    }
}

/// Filter Grok history JSONL text (malformed lines skipped).
pub fn filter_grok_history_lines(jsonl: &str) -> Vec<IngestableTurn> {
    let mut out = Vec::new();
    for line in jsonl.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(turn) = filter_grok_history_record(&value) {
            out.push(turn);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// OpenCode-export-like messages (F9 — fixture-ready for T238)
// ---------------------------------------------------------------------------

/// Filter one OpenCode-export-like message object.
pub fn filter_opencode_message(record: &Value) -> Option<IngestableTurn> {
    let role = record
        .get("role")
        .and_then(|v| v.as_str())
        .or_else(|| record.get("type").and_then(|v| v.as_str()))?;

    match role {
        "user" | "assistant" => {
            let text = if let Some(parts) = record.get("parts").and_then(|v| v.as_array()) {
                extract_text_from_parts(parts)
            } else {
                record
                    .get("content")
                    .and_then(extract_text_from_json_content)
            }?;
            let text = if role == "user" {
                extract_user_text(&text)
            } else {
                text.trim().to_string()
            };
            if text.is_empty() || is_sole_tool_json_payload(&text) {
                return None;
            }
            let ingest_role = if role == "user" {
                IngestRole::User
            } else {
                IngestRole::Assistant
            };
            Some(IngestableTurn {
                role: ingest_role,
                content: text,
                source_ts: record
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            })
        }
        "system" | "tool" | "function" | "reasoning" | "thinking" => None,
        _ => None,
    }
}

/// Filter OpenCode-export-like messages array or JSONL.
pub fn filter_opencode_messages(records: &[Value]) -> Vec<IngestableTurn> {
    records.iter().filter_map(filter_opencode_message).collect()
}

/// Filter OpenCode-like JSONL (one message per line).
pub fn filter_opencode_message_lines(jsonl: &str) -> Vec<IngestableTurn> {
    let mut out = Vec::new();
    for line in jsonl.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(turn) = filter_opencode_message(&value) {
            out.push(turn);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Text extraction helpers (F10 / F11 / F37 / F43)
// ---------------------------------------------------------------------------

/// Extract user-visible text from harness chrome.
///
/// - Strips `<ADDITIONAL_METADATA>` / `<USER_SETTINGS_CHANGE>` blocks
/// - Prefers body inside `<USER_REQUEST>…</USER_REQUEST>`
/// - Prefers body inside `<user_query>…</user_query>`
/// - Char-boundary safe (ASCII tag `find` only; never mid-scalar slices)
pub fn extract_user_text(raw: &str) -> String {
    let mut result = raw.to_string();
    result = strip_xml_block(&result, "ADDITIONAL_METADATA");
    result = strip_xml_block(&result, "USER_SETTINGS_CHANGE");

    if let Some(extracted) = extract_xml_content(&result, "USER_REQUEST") {
        result = extracted;
    }
    if let Some(extracted) = extract_xml_content(&result, "user_query") {
        result = extracted;
    }

    // Drop pure system-reminder wrappers with no remaining user text
    let trimmed = result.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // If only synthetic harness reminder chrome remains (no user_query extracted)
    if is_system_reminder_only(trimmed) {
        return String::new();
    }

    trimmed.to_string()
}

/// Extract text from `content` that is either a string or an array of parts.
///
/// Keeps `type == "text"` (or bare string parts / objects with `text` without
/// tool-ish type). Drops image / tool_use / tool_call / thinking parts.
pub fn extract_text_from_json_content(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() { None } else { Some(s.clone()) }
        }
        Value::Array(parts) => extract_text_from_parts(parts),
        Value::Object(map) => {
            // Single part-shaped object
            if let Some(t) = map.get("text").and_then(|v| v.as_str()) {
                let part_type = map.get("type").and_then(|v| v.as_str()).unwrap_or("text");
                if is_tool_or_thinking_part_type(part_type) {
                    return None;
                }
                let t = t.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn extract_text_from_parts(parts: &[Value]) -> Option<String> {
    let mut chunks: Vec<String> = Vec::new();
    for part in parts {
        match part {
            Value::String(s) => {
                let t = s.trim();
                if !t.is_empty() {
                    chunks.push(s.clone());
                }
            }
            Value::Object(map) => {
                let part_type = map.get("type").and_then(|v| v.as_str());
                if let Some(pt) = part_type
                    && is_tool_or_thinking_part_type(pt)
                {
                    continue;
                }
                // tool-shaped without text type
                if part_type.is_none()
                    && (map.contains_key("name") || map.contains_key("arguments"))
                    && !map.contains_key("text")
                {
                    continue;
                }
                if let Some(t) = map.get("text").and_then(|v| v.as_str()) {
                    let t = t.trim();
                    if !t.is_empty() {
                        chunks.push(t.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    if chunks.is_empty() {
        None
    } else {
        Some(chunks.join("\n"))
    }
}

fn is_tool_or_thinking_part_type(part_type: &str) -> bool {
    matches!(
        part_type,
        "tool_use"
            | "tool_call"
            | "tool_result"
            | "function_call"
            | "function"
            | "image"
            | "image_url"
            | "thinking"
            | "reasoning"
            | "redacted_thinking"
            | "backend_tool_call"
    )
}

/// True only for pure harness system-reminder chrome (no real user prose).
fn is_system_reminder_only(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    // Prefer structured tags; avoid dropping user prose that merely *mentions* reminders.
    if lower.starts_with("<system_reminder") || lower.starts_with("<system-reminder") {
        return !lower.contains("<user_query") && !lower.contains("<user_request");
    }
    false
}

/// Remove `<TAG>…</TAG>` blocks (ASCII tag names; char-boundary safe via `find`).
fn strip_xml_block(content: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");

    let mut result = String::new();
    let mut remaining = content;
    let mut in_block = false;
    let mut depth: i32 = 0;

    while let Some(pos) = if in_block {
        remaining.find(&close).or_else(|| remaining.find(&open))
    } else {
        remaining.find(&open)
    } {
        if !in_block {
            result.push_str(&remaining[..pos]);
            remaining = &remaining[pos + open.len()..];
            in_block = true;
            depth = 1;
        } else if remaining[pos..].starts_with(&close) {
            depth -= 1;
            remaining = &remaining[pos + close.len()..];
            if depth == 0 {
                in_block = false;
            }
        } else {
            depth += 1;
            remaining = &remaining[pos + open.len()..];
        }
    }

    if !in_block {
        result.push_str(remaining);
    }

    result
}

/// Extract inner text of first `<TAG>…</TAG>` (ASCII tags; char-boundary safe).
fn extract_xml_content(content: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");

    let start = content.find(&open)?;
    let after_open = start + open.len();
    let end = content[after_open..].find(&close)? + after_open;
    if end < after_open {
        return None;
    }

    Some(content[after_open..end].trim().to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    // Project test naming: function_or_feature__condition__expected_result
    #![allow(non_snake_case)]

    use super::*;
    use serde_json::json;

    #[test]
    fn filter_turn__user_and_assistant__kept() {
        let u = filter_turn("user", "hello").expect("user");
        assert_eq!(u.role, IngestRole::User);
        assert_eq!(u.content, "hello");
        let a = filter_turn("assistant", "hi").expect("assistant");
        assert_eq!(a.role, IngestRole::Assistant);
        assert_eq!(a.content, "hi");
    }

    #[test]
    fn filter_turn__system_tool_reasoning__dropped() {
        assert!(filter_turn("system", "x").is_none());
        assert!(filter_turn("tool", "x").is_none());
        assert!(filter_turn("reasoning", "x").is_none());
        assert!(filter_turn("function", "x").is_none());
        assert!(filter_turn("unknown", "x").is_none());
    }

    #[test]
    fn filter_turn__empty_after_strip__dropped() {
        assert!(filter_turn("user", "   ").is_none());
        assert!(filter_turn("assistant", "").is_none());
        assert!(
            filter_turn(
                "user",
                "<ADDITIONAL_METADATA>\nmeta\n</ADDITIONAL_METADATA>"
            )
            .is_none()
        );
    }

    #[test]
    fn filter_turn__assistant_sole_tool_json_payload__dropped() {
        // F15 / Codex P1 — tool JSON as sole assistant content must not ingest
        assert!(filter_turn("assistant", r#"{"tool_calls":[{"name":"view_file"}]}"#).is_none());
        assert!(filter_turn("assistant", r#"{"tool_result":"stdout dump"}"#).is_none());
        assert!(filter_turn("assistant", r#"{"type":"tool_result","output":"x"}"#).is_none());
        // Legitimate prose mentioning tools stays
        let kept = filter_turn(
            "assistant",
            "We should avoid storing tool_calls in the vault.",
        )
        .expect("prose");
        assert!(kept.content.contains("tool_calls"));
    }

    #[test]
    fn harness_paths__assistant_sole_tool_json__dropped() {
        // Guard applied on AGY / Grok / OpenCode assistant constructors (Codex r2)
        assert!(
            classify_antigravity_step(
                "MODEL",
                "PLANNER_RESPONSE",
                Some(r#"{"tool_calls":[{"name":"x"}]}"#),
                &[],
                None,
            )
            .is_none()
        );
        let grok_str = json!({"type":"assistant","content":"{\"tool_calls\":[]}"});
        assert!(filter_grok_history_record(&grok_str).is_none());
        let oc = json!({"role":"assistant","content":"{\"tool_result\":\"x\"}"});
        assert!(filter_opencode_message(&oc).is_none());
    }

    #[test]
    fn extract_user_text__user_request_kept_metadata_dropped() {
        let input = "<USER_REQUEST>\ndo the ai brains preflight\n</USER_REQUEST>\n<ADDITIONAL_METADATA>\nThe current local time is: 2026-05-01\n</ADDITIONAL_METADATA>";
        assert_eq!(extract_user_text(input), "do the ai brains preflight");
    }

    #[test]
    fn extract_user_text__emoji_user_request__no_panic_correct_inner() {
        // AC15 / F43 — multibyte inside USER_REQUEST
        let input = "<USER_REQUEST>\nhello 🎯 world 日本語\n</USER_REQUEST>";
        assert_eq!(extract_user_text(input), "hello 🎯 world 日本語");
    }

    #[test]
    fn extract_user_text__emoji_user_query__no_panic_correct_inner() {
        // AC15 / F43 — multibyte inside user_query
        let input = "noise <user_query>\nship 🚀 日本語\n</user_query>";
        assert_eq!(extract_user_text(input), "ship 🚀 日本語");
    }

    #[test]
    fn extract_user_text__user_query_inner() {
        let input = "noise <user_query>\nfix the path\n</user_query> trailer";
        assert_eq!(extract_user_text(input), "fix the path");
    }

    #[test]
    fn classify_antigravity__thinking_field_not_in_content() {
        // AC2 — thinking must never enter kept content (even if present on Value path)
        let a =
            classify_antigravity_step("MODEL", "PLANNER_RESPONSE", Some("visible only"), &[], None)
                .expect("assistant");
        assert_eq!(a.content, "visible only");
        assert!(!a.content.contains("hidden"));
    }

    #[test]
    fn classify_antigravity__view_file_and_run_command_with_content__dropped() {
        // AC16
        let view = classify_antigravity_step(
            "MODEL",
            "VIEW_FILE",
            Some("secret file contents"),
            &[],
            None,
        );
        assert!(view.is_none());
        let run =
            classify_antigravity_step("MODEL", "RUN_COMMAND", Some("stdout of command"), &[], None);
        assert!(run.is_none());
    }

    #[test]
    fn classify_antigravity__user_and_planner_text__kept() {
        let u = classify_antigravity_step(
            "USER_EXPLICIT",
            "USER_INPUT",
            Some("<USER_REQUEST>\nhello\n</USER_REQUEST>"),
            &[],
            Some("2026-05-01T00:00:00Z"),
        )
        .expect("user");
        assert_eq!(u.content, "hello");
        let a = classify_antigravity_step(
            "MODEL",
            "PLANNER_RESPONSE",
            Some("Here is the answer."),
            &[],
            None,
        )
        .expect("assistant");
        assert_eq!(a.content, "Here is the answer.");
    }

    #[test]
    fn classify_antigravity__tool_only_planner__dropped() {
        let tool_only = classify_antigravity_step(
            "MODEL",
            "PLANNER_RESPONSE",
            None,
            &[json!({"name": "view_file"})],
            None,
        );
        assert!(tool_only.is_none());
        let empty = classify_antigravity_step("MODEL", "PLANNER_RESPONSE", Some("  "), &[], None);
        assert!(empty.is_none());
    }

    #[test]
    fn extract_text_from_json_content__array_text_parts_drop_tools() {
        // AC6 / AC7
        let value = json!([
            {"type": "text", "text": "part one"},
            {"type": "tool_call", "name": "run", "arguments": "{}"},
            {"type": "text", "text": "part two"},
            {"type": "thinking", "text": "hidden"},
            {"type": "image", "url": "http://x"}
        ]);
        let text = extract_text_from_json_content(&value).expect("text");
        assert_eq!(text, "part one\npart two");
        assert!(!text.contains("hidden"));
        assert!(!text.contains("run"));
    }

    #[test]
    fn extract_text_from_json_content__tool_only_parts__none() {
        let value = json!([
            {"type": "tool_use", "name": "x", "input": {}},
            {"type": "tool_call", "name": "y"}
        ]);
        assert!(extract_text_from_json_content(&value).is_none());
    }

    #[test]
    fn filter_grok_history__drops_reasoning_tools_system() {
        let jsonl = r#"
{"type":"user","content":"hello"}
{"type":"reasoning","content":"chain of thought"}
{"type":"tool_result","content":"tool out"}
{"type":"backend_tool_call","content":"{}"}
{"type":"system","content":"chrome"}
{"type":"assistant","content":"final answer"}
"#;
        let turns = filter_grok_history_lines(jsonl);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, IngestRole::User);
        assert_eq!(turns[0].content, "hello");
        assert_eq!(turns[1].role, IngestRole::Assistant);
        assert_eq!(turns[1].content, "final answer");
    }

    #[test]
    fn filter_grok_history__user_array_with_user_query() {
        let record = json!({
            "type": "user",
            "content": [
                {"type": "text", "text": "<user_query>\nfix CI\n</user_query>"},
                {"type": "image", "url": "http://x"}
            ]
        });
        let turn = filter_grok_history_record(&record).expect("user");
        assert_eq!(turn.content, "fix CI");
    }

    #[test]
    fn filter_opencode_messages__user_assistant_only() {
        let records = vec![
            json!({"role": "user", "content": "q"}),
            json!({"role": "tool", "content": "out"}),
            json!({"role": "assistant", "parts": [
                {"type": "text", "text": "a"},
                {"type": "tool_call", "name": "x"}
            ]}),
            json!({"role": "system", "content": "s"}),
        ];
        let turns = filter_opencode_messages(&records);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].content, "q");
        assert_eq!(turns[1].content, "a");
    }

    #[test]
    fn filter_agy_simple_lines__drops_system() {
        let jsonl = r#"
{"role":"user","content":"hello","timestamp":"t1"}
{"role":"system","content":"internal"}
{"role":"assistant","content":"hi"}
"#;
        let turns = filter_agy_simple_lines(jsonl);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, IngestRole::User);
        assert_eq!(turns[1].role, IngestRole::Assistant);
    }
}
