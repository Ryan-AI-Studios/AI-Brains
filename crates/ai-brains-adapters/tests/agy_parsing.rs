#[cfg(test)]
mod tests {
    #![allow(non_snake_case)] // function_or_feature__condition__expected_result

    use ai_brains_adapters::agy::{
        filter_agy_turns, generate_deterministic_turn_id, parse_agy_transcript,
        parse_agy_transcript_message_only,
    };
    use ai_brains_core::ids::SessionId;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[allow(clippy::disallowed_methods)]
    fn test_parse_agy_transcript() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "{{\"role\": \"user\", \"content\": \"hello\", \"timestamp\": \"2026-05-24T12:00:00Z\"}}"
        )
        .unwrap();
        writeln!(
            file,
            "{{\"role\": \"assistant\", \"content\": \"hi\", \"timestamp\": \"2026-05-24T12:00:01Z\"}}"
        )
        .unwrap();
        writeln!(
            file,
            "{{\"role\": \"system\", \"content\": \"internal\", \"timestamp\": \"2026-05-24T12:00:02Z\"}}"
        )
        .unwrap();

        // Raw parse keeps all roles (filter is a separate SOOT step).
        let turns = parse_agy_transcript(file.path()).unwrap();
        assert_eq!(turns.len(), 3);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[0].content, "hello");
        assert_eq!(turns[1].role, "assistant");
        assert_eq!(turns[1].content, "hi");
        assert_eq!(turns[2].role, "system");
        assert_eq!(turns[2].content, "internal");

        // Message-only SOOT drops system (AC14).
        let filtered = filter_agy_turns(&turns);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].role, "user");
        assert_eq!(filtered[1].role, "assistant");

        let only = parse_agy_transcript_message_only(file.path()).unwrap();
        assert_eq!(only.len(), 2);
    }

    #[test]
    fn test_deterministic_turn_id() {
        let session_id = SessionId::new();
        let id1 = generate_deterministic_turn_id(&session_id, 0);
        let id2 = generate_deterministic_turn_id(&session_id, 0);
        let id3 = generate_deterministic_turn_id(&session_id, 1);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    #[allow(clippy::disallowed_methods)]
    fn parse_agy_transcript__malformed_line__skipped_not_fatal() {
        // F26/F41 fail-open
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{{\"role\": \"user\", \"content\": \"hello\"}}").unwrap();
        writeln!(file, "NOT VALID JSON").unwrap();
        writeln!(file, "{{\"role\": \"assistant\", \"content\": \"hi\"}}").unwrap();
        writeln!(
            file,
            "{{\"role\": \"assistant\", \"content\": \"{{\\\"tool_calls\\\":[]}}\"}}"
        )
        .unwrap();

        let raw = parse_agy_transcript(file.path()).unwrap();
        assert_eq!(raw.len(), 3); // malformed skipped; tool JSON string still raw

        let filtered = filter_agy_turns(&raw);
        assert_eq!(filtered.len(), 2); // tool sole payload dropped by F15
        assert_eq!(filtered[0].content, "hello");
        assert_eq!(filtered[1].content, "hi");
    }
}
