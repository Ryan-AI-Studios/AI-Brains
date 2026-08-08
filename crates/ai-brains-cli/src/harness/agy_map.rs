//! Pure AGY Stop event → agy-hook payload map (F34 / F35 / AC16).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Payload matching `Docs/schemas/agy-hook-payload.json` (schema fields only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgyHookPayload {
    pub transcript_path: String,
    pub session_id: String,
    pub project_hash: String,
}

/// Soft-skip reasons for the Stop → agy-hook map (wrapper exits 0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapSkip {
    MissingTranscriptPath,
    MissingConversationId,
    InvalidSessionIdUuid,
    NotFullyIdle,
}

impl MapSkip {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingTranscriptPath => "missing transcriptPath",
            Self::MissingConversationId => "missing conversationId",
            Self::InvalidSessionIdUuid => "conversationId is not a UUID",
            Self::NotFullyIdle => "fullyIdle is false",
        }
    }
}

// as_str used by diagnostics; keep public for callers.
const _: fn(&MapSkip) -> &'static str = MapSkip::as_str;

/// Map AGY Stop JSON to agy-hook payload (F34).
///
/// - `transcriptPath` ← Stop.transcriptPath (missing → MapSkip)
/// - `sessionId` ← Stop.conversationId (must be UUID-parseable else MapSkip)
/// - `projectHash` ← first non-empty workspacePaths[] else `"agy-unbound"`
/// - `fullyIdle: false` → soft skip (F35)
pub fn map_agy_stop_to_hook_payload(stop: &Value) -> Result<AgyHookPayload, MapSkip> {
    // F35: fullyIdle false → soft skip. Missing fullyIdle is treated as ok to proceed.
    if let Some(idle) = stop.get("fullyIdle")
        && idle.as_bool() == Some(false)
    {
        return Err(MapSkip::NotFullyIdle);
    }

    let transcript_path = stop
        .get("transcriptPath")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(MapSkip::MissingTranscriptPath)?
        .to_string();

    let conversation_id = stop
        .get("conversationId")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(MapSkip::MissingConversationId)?;

    // Must be UUID-parseable
    if uuid::Uuid::parse_str(conversation_id).is_err() {
        return Err(MapSkip::InvalidSessionIdUuid);
    }

    let project_hash = first_workspace_path(stop).unwrap_or_else(|| "agy-unbound".to_string());

    Ok(AgyHookPayload {
        transcript_path,
        session_id: conversation_id.to_string(),
        project_hash,
    })
}

fn first_workspace_path(stop: &Value) -> Option<String> {
    let arr = stop.get("workspacePaths")?.as_array()?;
    for v in arr {
        if let Some(s) = v.as_str() {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn map_agy_stop__full_fixture__maps_fields() {
        // AC16
        let stop = json!({
            "conversationId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "transcriptPath": "C:\\\\tmp\\\\t.jsonl",
            "workspacePaths": ["C:\\\\dev\\\\proj"],
            "fullyIdle": true
        });
        let p = map_agy_stop_to_hook_payload(&stop).expect("map");
        assert_eq!(p.session_id, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        assert_eq!(p.transcript_path, "C:\\\\tmp\\\\t.jsonl");
        assert_eq!(p.project_hash, "C:\\\\dev\\\\proj");
    }

    #[test]
    fn map_agy_stop__empty_workspace__agy_unbound() {
        // AC16
        let stop = json!({
            "conversationId": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
            "transcriptPath": "/tmp/t.jsonl",
            "workspacePaths": [],
            "fullyIdle": true
        });
        let p = map_agy_stop_to_hook_payload(&stop).expect("map");
        assert_eq!(p.project_hash, "agy-unbound");
    }

    #[test]
    fn map_agy_stop__missing_workspace_key__agy_unbound() {
        let stop = json!({
            "conversationId": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
            "transcriptPath": "/tmp/t.jsonl",
            "fullyIdle": true
        });
        let p = map_agy_stop_to_hook_payload(&stop).expect("map");
        assert_eq!(p.project_hash, "agy-unbound");
    }

    #[test]
    fn map_agy_stop__fully_idle_false__skip() {
        let stop = json!({
            "conversationId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "transcriptPath": "/tmp/t.jsonl",
            "workspacePaths": ["x"],
            "fullyIdle": false
        });
        let err = map_agy_stop_to_hook_payload(&stop).expect_err("skip");
        assert_eq!(err, MapSkip::NotFullyIdle);
    }

    #[test]
    fn map_agy_stop__missing_transcript__skip() {
        let stop = json!({
            "conversationId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "fullyIdle": true
        });
        assert_eq!(
            map_agy_stop_to_hook_payload(&stop).unwrap_err(),
            MapSkip::MissingTranscriptPath
        );
    }

    #[test]
    fn map_agy_stop__invalid_uuid__skip() {
        let stop = json!({
            "conversationId": "not-a-uuid",
            "transcriptPath": "/tmp/t.jsonl",
            "fullyIdle": true
        });
        assert_eq!(
            map_agy_stop_to_hook_payload(&stop).unwrap_err(),
            MapSkip::InvalidSessionIdUuid
        );
    }

    #[test]
    fn map_agy_stop__payload_serde_schema_fields_only() {
        let p = AgyHookPayload {
            transcript_path: "a".into(),
            session_id: "b".into(),
            project_hash: "c".into(),
        };
        let v = serde_json::to_value(&p).expect("ser");
        let obj = v.as_object().expect("obj");
        assert_eq!(obj.len(), 3);
        assert!(obj.contains_key("transcriptPath"));
        assert!(obj.contains_key("sessionId"));
        assert!(obj.contains_key("projectHash"));
    }
}
