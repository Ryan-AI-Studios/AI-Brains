#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_events::Envelope;
use std::fs;
use std::path::PathBuf;

#[test]
fn legacy_v1_events_ndjson__deserializes_all_lines() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/governed-memory/legacy-v1-events.ndjson");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));

    let mut count = 0usize;
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let env: Envelope = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} deserialize failed: {e}\n{line}", i + 1));
        // Must not collapse known types into empty Unknown.
        assert!(
            !matches!(env.payload, ai_brains_events::Payload::Unknown(_)),
            "line {} unexpectedly Unknown",
            i + 1
        );
        count += 1;
    }
    assert!(count > 0, "fixture must have at least one event");
    assert_eq!(count, text.lines().filter(|l| !l.trim().is_empty()).count());
}
