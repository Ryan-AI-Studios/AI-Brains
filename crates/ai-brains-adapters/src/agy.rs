use crate::errors::Result;
use crate::message_only::{IngestableTurn, filter_agy_simple_turn};
use ai_brains_core::ids::{SessionId, TurnId};
use serde::Deserialize;
use std::path::Path;
use uuid::Uuid;

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

pub fn generate_deterministic_turn_id(session_id: &SessionId, index: usize) -> TurnId {
    TurnId::from_uuid(Uuid::new_v5(
        &session_id.as_uuid(),
        format!("agy-turn-{}", index).as_bytes(),
    ))
}
