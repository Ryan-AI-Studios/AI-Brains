use crate::errors::Result;
use crate::message_only::{IngestableTurn, filter_agy_simple_turn};
use ai_brains_core::ids::{SessionId, TurnId};
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Stable AGY unbound project alias (hook + batch share this SOOT).
pub const AGY_UNBOUND_ALIAS: &str = "agy-unbound";

/// Display name for the shared unbound AGY project (F12).
pub const AGY_UNBOUND_DISPLAY_NAME: &str = "(unbound AGY)";

/// A single line from the agy JSONL transcript (raw parse; not yet message-only filtered).
#[derive(Debug, Clone, Deserialize)]
pub struct AgyTranscriptLine {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}

pub struct AgyTurn {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}

/// Message-only turn ready for hook or batch ingest (shared parse SOOT).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptIngestTurn {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
    /// Original step_index when parsed from step-shaped SOOT (for turn-id preference).
    pub step_index: Option<u32>,
}

/// Parse agy `{role,content}` JSONL without filtering (raw lines).
/// Malformed lines are skipped (F26/F41 fail-open), matching antigravity overview style.
/// Prefer [`filter_agy_turns`] / [`parse_agy_transcript_message_only`] for ingest.
pub fn parse_agy_transcript(path: &Path) -> Result<Vec<AgyTurn>> {
    let content = std::fs::read_to_string(path)?;

    let mut turns = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Ok(transcript_line) = serde_json::from_str::<AgyTranscriptLine>(line) else {
            // F41: skip bad JSONL lines; do not fail whole file
            continue;
        };

        turns.push(AgyTurn {
            role: transcript_line.role,
            content: transcript_line.content,
            timestamp: transcript_line.timestamp,
        });
    }

    Ok(turns)
}

/// Apply message-only SOOT to raw agy turns (user/assistant text only; drop system/tool).
pub fn filter_agy_turns(turns: &[AgyTurn]) -> Vec<AgyTurn> {
    turns
        .iter()
        .filter_map(|t| {
            filter_agy_simple_turn(&t.role, &t.content, t.timestamp.clone()).map(|ing| AgyTurn {
                role: ing.role.as_str().to_string(),
                content: ing.content,
                timestamp: ing.source_ts,
            })
        })
        .collect()
}

/// Parse + message-only filter in one step (ingest SOOT for hooks / importers).
pub fn parse_agy_transcript_message_only(path: &Path) -> Result<Vec<AgyTurn>> {
    let raw = parse_agy_transcript(path)?;
    Ok(filter_agy_turns(&raw))
}

/// Map message-only turns back to agy role/content (for callers that already hold SOOT turns).
pub fn ingestable_to_agy(turns: &[IngestableTurn]) -> Vec<AgyTurn> {
    turns
        .iter()
        .map(|t| AgyTurn {
            role: t.role.as_str().to_string(),
            content: t.content.clone(),
            timestamp: t.source_ts.clone(),
        })
        .collect()
}

/// Deterministic turn id SOOT (F2 / AC19): `v5(session, "turn-{i}")`.
///
/// Prefer [`generate_turn_id_for_ingest`] when a step_index may be present.
pub fn generate_deterministic_turn_id(session_id: &SessionId, index: usize) -> TurnId {
    generate_turn_id_for_ingest(session_id, index, None)
}

/// Turn-id SOOT shared by hook and batch (F2).
///
/// - When `step_index` is `Some`, name is `turn-{step_index}`.
/// - Else name is `turn-{sequential_index}` (retire legacy `agy-turn-{i}`).
pub fn generate_turn_id_for_ingest(
    session_id: &SessionId,
    sequential_index: usize,
    step_index: Option<u32>,
) -> TurnId {
    let name = match step_index {
        Some(si) => format!("turn-{si}"),
        None => format!("turn-{sequential_index}"),
    };
    TurnId::from_uuid(Uuid::new_v5(&session_id.as_uuid(), name.as_bytes()))
}

/// Prefer `transcript_full.jsonl` sibling when ingest path is `transcript.jsonl` (F29 / AC21).
pub fn prefer_full_transcript_path(path: &Path) -> PathBuf {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name.eq_ignore_ascii_case("transcript.jsonl") {
        let sibling = path.with_file_name("transcript_full.jsonl");
        if sibling.is_file() {
            return sibling;
        }
    }
    path.to_path_buf()
}

/// True when a JSON object looks step-shaped (F1): has `step_index` or (`source`+`type`).
pub fn is_step_shaped_object(value: &Value) -> bool {
    let Some(obj) = value.as_object() else {
        return false;
    };
    if obj.contains_key("step_index") {
        return true;
    }
    obj.contains_key("source") && obj.contains_key("type")
}

/// Shared transcript parser for hook + batch (F1/F2/F29).
///
/// - Prefers sibling `transcript_full.jsonl` when present (F29).
/// - Detects step-shaped vs legacy `{role,content}` from first non-empty JSONL object.
/// - Message-only SOOT; fail-open per line (F41).
pub fn parse_transcript_for_ingest(path: &Path) -> Result<Vec<TranscriptIngestTurn>> {
    let content_path = prefer_full_transcript_path(path);
    let content = std::fs::read_to_string(&content_path)?;

    let mut first_obj: Option<Value> = None;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            first_obj = Some(v);
            break;
        }
        // Malformed first lines skipped when searching for shape (F41).
    }

    let step_shaped = first_obj.as_ref().is_some_and(is_step_shaped_object);

    if step_shaped {
        parse_step_shaped_for_ingest(&content)
    } else {
        parse_legacy_role_content_for_ingest(&content)
    }
}

fn parse_step_shaped_for_ingest(content: &str) -> Result<Vec<TranscriptIngestTurn>> {
    let mut turns = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(step) = serde_json::from_str::<crate::antigravity::AntigravityStep>(line) else {
            continue; // F41
        };
        // Skip completely empty step shells (no source/type and zero step_index with no content)
        if step.source.is_empty() && step.step_type.is_empty() && step.content.is_none() {
            continue;
        }
        if let Some(ing) = crate::message_only::classify_antigravity_step(
            step.source.as_str(),
            step.step_type.as_str(),
            step.content.as_deref(),
            &step.tool_calls,
            step.created_at.as_deref(),
        ) {
            turns.push(TranscriptIngestTurn {
                role: ing.role.as_str().to_string(),
                content: ing.content,
                timestamp: ing.source_ts,
                step_index: Some(step.step_index),
            });
        }
    }
    Ok(turns)
}

fn parse_legacy_role_content_for_ingest(content: &str) -> Result<Vec<TranscriptIngestTurn>> {
    let mut turns = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(transcript_line) = serde_json::from_str::<AgyTranscriptLine>(line) else {
            continue;
        };
        if let Some(ing) = filter_agy_simple_turn(
            &transcript_line.role,
            &transcript_line.content,
            transcript_line.timestamp,
        ) {
            turns.push(TranscriptIngestTurn {
                role: ing.role.as_str().to_string(),
                content: ing.content,
                timestamp: ing.source_ts,
                step_index: None,
            });
        }
    }
    Ok(turns)
}

/// Normalize a project hash / workspace string for alias keys (F3/F13).
///
/// Empty / `agy-unbound` → unbound alias. Path-like inputs are path-normalized;
/// non-path tokens keep their trimmed form when normalize fails.
pub fn normalize_agy_project_hash(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(AGY_UNBOUND_ALIAS) {
        return AGY_UNBOUND_ALIAS.to_string();
    }
    match ai_brains_path::normalize_project_path(trimmed) {
        Ok(p) => p.canonical().to_string(),
        Err(_) => trimmed.to_string(),
    }
}

/// Whether F3 allows `AI_BRAINS_PROJECT_ID` env fallback for this hash.
pub fn agy_env_fallback_allowed(project_hash: &str) -> bool {
    let t = project_hash.trim();
    t.is_empty() || t.eq_ignore_ascii_case(AGY_UNBOUND_ALIAS)
}

/// Display name for a path-derived project (basename of normalized path).
pub fn path_derived_display_name(normalized_path: &str) -> String {
    Path::new(normalized_path)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(normalized_path)
        .to_string()
}

/// Stable source_meta key keyed by normalized path (F30 / AC22).
///
/// Format: `source_meta:agy:{sha256_hex(normalized_path)}` so dual-root brains
/// with the same conversationId do not clobber each other.
pub fn agy_source_meta_key(path: &Path) -> String {
    let key_material = match ai_brains_path::normalize_project_path(&path.to_string_lossy()) {
        Ok(p) => p.canonical().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    };
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(key_material.as_bytes());
    format!("source_meta:agy:{}", hex::encode(hasher.finalize()))
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parse_transcript_for_ingest__step_shaped__user_assistant_only() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("transcript.jsonl");
        let body = r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\nhello\n</USER_REQUEST>","tool_calls":[]}
{"step_index":4,"source":"MODEL","type":"PLANNER_RESPONSE","content":"Here is the answer.","tool_calls":[]}
"#;
        std::fs::write(&path, body).unwrap();
        let turns = parse_transcript_for_ingest(&path).unwrap();
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[0].content, "hello");
        assert_eq!(turns[0].step_index, Some(0));
        assert_eq!(turns[1].role, "assistant");
        assert_eq!(turns[1].content, "Here is the answer.");
        assert_eq!(turns[1].step_index, Some(4));
    }

    #[test]
    fn parse_transcript_for_ingest__role_content_legacy__kept() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("legacy.jsonl");
        let body = r#"{"role":"user","content":"hi"}
{"role":"assistant","content":"yo"}
{"role":"system","content":"dropme"}
"#;
        std::fs::write(&path, body).unwrap();
        let turns = parse_transcript_for_ingest(&path).unwrap();
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[1].role, "assistant");
        assert!(turns[0].step_index.is_none());
    }

    #[test]
    fn parse_transcript_for_ingest__view_file_thinking__dropped() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("tools.jsonl");
        let body = r#"{"step_index":1,"source":"MODEL","type":"VIEW_FILE","content":"secret","tool_calls":[]}
{"step_index":2,"source":"MODEL","type":"RUN_COMMAND","content":"stdout","tool_calls":[]}
{"step_index":3,"source":"MODEL","type":"THINKING","content":"chain of thought","tool_calls":[]}
{"step_index":4,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\nkeep me\n</USER_REQUEST>","tool_calls":[]}
"#;
        std::fs::write(&path, body).unwrap();
        let turns = parse_transcript_for_ingest(&path).unwrap();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].content, "keep me");
        assert!(!turns.iter().any(|t| t.content.contains("secret")));
        assert!(!turns.iter().any(|t| t.content.contains("chain of thought")));
    }

    #[test]
    fn turn_id__hook_and_batch__identical_for_fixture() {
        let session =
            SessionId::from_uuid(Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap());
        let dir = tempdir().unwrap();
        let path = dir.path().join("transcript.jsonl");
        let body = r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\na\n</USER_REQUEST>","tool_calls":[]}
{"step_index":4,"source":"MODEL","type":"PLANNER_RESPONSE","content":"b","tool_calls":[]}
"#;
        std::fs::write(&path, body).unwrap();
        let turns = parse_transcript_for_ingest(&path).unwrap();
        // Simulate hook and batch both walking the same shared parse result.
        let hook_ids: Vec<_> = turns
            .iter()
            .enumerate()
            .map(|(i, t)| generate_turn_id_for_ingest(&session, i, t.step_index))
            .collect();
        let batch_ids: Vec<_> = turns
            .iter()
            .enumerate()
            .map(|(i, t)| generate_turn_id_for_ingest(&session, i, t.step_index))
            .collect();
        assert_eq!(hook_ids, batch_ids);
        // step_index preferred over sequential
        assert_eq!(
            hook_ids[0],
            generate_turn_id_for_ingest(&session, 0, Some(0))
        );
        assert_eq!(
            hook_ids[1],
            generate_turn_id_for_ingest(&session, 1, Some(4))
        );
        assert_ne!(
            generate_deterministic_turn_id(&session, 0),
            // legacy namespace retired
            TurnId::from_uuid(Uuid::new_v5(&session.as_uuid(), b"agy-turn-0"))
        );
    }

    #[test]
    fn parse_transcript__prefers_transcript_full__when_present() {
        let dir = tempdir().unwrap();
        let logs = dir.path().join("logs");
        std::fs::create_dir_all(&logs).unwrap();
        let truncated = logs.join("transcript.jsonl");
        let full = logs.join("transcript_full.jsonl");
        std::fs::write(
            &truncated,
            r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\nshort\n</USER_REQUEST>","truncated_fields":["content"],"tool_calls":[]}
"#,
        )
        .unwrap();
        std::fs::write(
            &full,
            r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\nthis is the full longer user request text\n</USER_REQUEST>","tool_calls":[]}
{"step_index":4,"source":"MODEL","type":"PLANNER_RESPONSE","content":"full assistant answer body here","tool_calls":[]}
"#,
        )
        .unwrap();
        let turns = parse_transcript_for_ingest(&truncated).unwrap();
        assert_eq!(turns.len(), 2);
        assert!(turns[0].content.contains("full longer"));
        assert!(turns[1].content.contains("full assistant"));
        assert!(!turns[0].content.eq("short"));
    }

    #[test]
    fn source_meta__dual_root__no_clobber() {
        let a = Path::new(r"C:\Users\x\.gemini\antigravity\brain\sid\logs\transcript.jsonl");
        let b = Path::new(r"C:\Users\x\.gemini\antigravity-cli\brain\sid\logs\transcript.jsonl");
        let ka = agy_source_meta_key(a);
        let kb = agy_source_meta_key(b);
        assert!(ka.starts_with("source_meta:agy:"));
        assert!(kb.starts_with("source_meta:agy:"));
        assert_ne!(ka, kb, "dual-root same session_id must not share meta key");
        // Case variants of same path normalize to same key
        let c = Path::new(r"c:\users\x\.gemini\antigravity-cli\brain\sid\logs\transcript.jsonl");
        assert_eq!(kb, agy_source_meta_key(c));
    }

    #[test]
    fn normalize_agy_project_hash__unbound_and_case() {
        assert_eq!(normalize_agy_project_hash(""), AGY_UNBOUND_ALIAS);
        assert_eq!(normalize_agy_project_hash("agy-unbound"), AGY_UNBOUND_ALIAS);
        let a = normalize_agy_project_hash(r"C:\dev\Dedupe");
        let b = normalize_agy_project_hash(r"c:\dev\dedupe");
        assert_eq!(a, b);
    }

    #[test]
    fn agy_env_fallback_allowed__only_unbound() {
        assert!(agy_env_fallback_allowed(""));
        assert!(agy_env_fallback_allowed("agy-unbound"));
        assert!(!agy_env_fallback_allowed(r"C:\dev\proj"));
        assert!(!agy_env_fallback_allowed("abc123hash"));
    }

    #[test]
    fn prefer_full__absent_keeps_truncated() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("transcript.jsonl");
        std::fs::write(&path, "{}").unwrap();
        assert_eq!(prefer_full_transcript_path(&path), path);
    }
}
