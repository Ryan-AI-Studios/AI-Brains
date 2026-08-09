//! T234 fixture matrix: AGY steps, Grok history, OpenCode-export-like messages.
//! T238: nested live export + synthetic chrome matrix.
#![allow(non_snake_case)] // function_or_feature__condition__expected_result
#![allow(clippy::disallowed_methods)]

use ai_brains_adapters::message_only::{
    IngestRole, classify_antigravity_step, filter_grok_history_lines, filter_opencode_export,
    filter_opencode_message_lines,
};
use serde_json::Value;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("message_only");
    path.push(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

fn fixture_json(name: &str) -> Value {
    serde_json::from_str(&fixture(name)).unwrap_or_else(|e| panic!("parse fixture {name}: {e}"))
}

#[test]
fn agy_fixture__tools_thinking_view_run__only_user_assistant() {
    // AC2 + AC16: VIEW_FILE / RUN_COMMAND with content never kept
    let jsonl = fixture("agy_steps.jsonl");
    let mut turns = Vec::new();
    for line in jsonl.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(step) = serde_json::from_str::<Value>(line) else {
            panic!("fixture step must be valid JSON: {line}");
        };
        let Some(source) = step["source"].as_str() else {
            panic!("fixture step missing source");
        };
        let Some(step_type) = step["type"].as_str() else {
            panic!("fixture step missing type");
        };
        let content = step.get("content").and_then(|v| v.as_str());
        let tool_calls = step
            .get("tool_calls")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let created_at = step.get("created_at").and_then(|v| v.as_str());
        if let Some(t) =
            classify_antigravity_step(source, step_type, content, &tool_calls, created_at)
        {
            turns.push(t);
        }
    }

    assert_eq!(turns.len(), 2, "expected user + final assistant only");
    assert_eq!(turns[0].role, IngestRole::User);
    assert_eq!(turns[0].content, "read the config");
    assert_eq!(turns[1].role, IngestRole::Assistant);
    assert_eq!(turns[1].content, "The config looks fine.");
    for t in &turns {
        assert!(!t.content.contains("secret file"));
        assert!(!t.content.contains("command stdout"));
        assert!(!t.content.contains("tool output"));
        assert!(!t.content.contains("checkpoint"));
    }
}

#[test]
fn grok_fixture__reasoning_tools_system__only_user_assistant() {
    // AC3 + AC6 + AC7
    let turns = filter_grok_history_lines(&fixture("grok_history.jsonl"));
    assert_eq!(turns.len(), 3);
    assert_eq!(turns[0].role, IngestRole::User);
    assert_eq!(turns[0].content, "ship the feature");
    assert_eq!(turns[1].role, IngestRole::Assistant);
    assert_eq!(turns[1].content, "Feature shipped.");
    assert_eq!(turns[2].role, IngestRole::Assistant);
    assert_eq!(turns[2].content, "Also updated docs.");
    for t in &turns {
        assert!(!t.content.contains("chain of thought"));
        assert!(!t.content.contains("command output leak"));
        assert!(!t.content.contains("run_terminal_command"));
    }
}

#[test]
fn opencode_fixture__only_user_assistant() {
    // AC2 (T234 AC4) — flat jsonl regression
    let turns = filter_opencode_message_lines(&fixture("opencode_messages.jsonl"));
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].role, IngestRole::User);
    assert_eq!(turns[0].content, "what is the status?");
    assert_eq!(turns[1].role, IngestRole::Assistant);
    assert_eq!(turns[1].content, "All green.");
}

#[test]
fn opencode_export_live__nested_part_union__only_text() {
    // AC1 / AC19 — nested live shape + full denylist
    let turns = filter_opencode_export(&fixture_json("opencode_export_live.json"));
    assert_eq!(turns.len(), 2, "{turns:?}");
    assert_eq!(turns[0].turn.role, IngestRole::User);
    assert_eq!(turns[0].turn.content, "ship OpenCode seamless ingest");
    assert_eq!(turns[0].msg_id.as_deref(), Some("msg_user_live_1"));
    assert_eq!(turns[1].turn.role, IngestRole::Assistant);
    assert_eq!(turns[1].turn.content, "OpenCode seamless ingest is ready.");
    assert_eq!(turns[1].msg_id.as_deref(), Some("msg_asst_live_1"));
    for t in &turns {
        let c = &t.turn.content;
        assert!(!c.contains("plan the steps"));
        assert!(!c.contains("never ingest"));
        assert!(!c.contains("stdout leak"));
        assert!(!c.contains("file part"));
        assert!(!c.contains("subtask"));
    }
}

#[test]
fn opencode_synthetic_chrome__zero_user_memories_real_kept() {
    // AC22
    let turns = filter_opencode_export(&fixture_json("opencode_synthetic_chrome.json"));
    assert_eq!(turns.len(), 2, "{turns:?}");
    assert_eq!(turns[0].turn.role, IngestRole::User);
    assert_eq!(turns[0].turn.content, "what is the status of T238?");
    assert_eq!(turns[1].turn.role, IngestRole::Assistant);
    assert_eq!(turns[1].turn.content, "T238 is in progress.");
    for t in &turns {
        assert!(!t.turn.content.contains("Called the Read"));
        assert!(!t.turn.content.contains("executed by the user"));
        assert!(!t.turn.content.contains("ignored synthetic"));
        assert!(!t.turn.content.contains("editor context"));
        assert!(!t.turn.content.contains("compaction"));
        assert!(!t.turn.content.contains("thinking about"));
    }
}
