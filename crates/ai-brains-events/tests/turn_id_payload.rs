//! T262 AC1 — additive `turn_id` on capture payloads.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_core::ids::{SessionId, TurnId};
use ai_brains_events::{AssistantFinalRecordedPayload, UserPromptRecordedPayload};

const SESSION: &str = "11111111-1111-1111-1111-111111111111";
const TURN: &str = "22222222-2222-2222-2222-222222222222";

#[test]
fn user_prompt_recorded_payload__missing_turn_id_key__deserializes_none__ac1() {
    let json = format!(r#"{{"session_id":"{SESSION}","content":"hi"}}"#);
    let p: UserPromptRecordedPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        p.session_id,
        SessionId::from_uuid(uuid::Uuid::parse_str(SESSION).unwrap())
    );
    assert_eq!(p.content, "hi");
    assert_eq!(p.turn_id, None);
}

#[test]
fn user_prompt_recorded_payload__turn_id_present__round_trip__ac1() {
    let json = format!(r#"{{"session_id":"{SESSION}","content":"hi","turn_id":"{TURN}"}}"#);
    let p: UserPromptRecordedPayload = serde_json::from_str(&json).expect("deserialize");
    let expected = TurnId::from_uuid(uuid::Uuid::parse_str(TURN).unwrap());
    assert_eq!(p.turn_id, Some(expected));
    let again: UserPromptRecordedPayload =
        serde_json::from_value(serde_json::to_value(&p).expect("ser")).expect("de");
    assert_eq!(again.turn_id, Some(expected));
}

#[test]
fn assistant_final_recorded_payload__missing_turn_id_key__deserializes_none__ac1() {
    let json = format!(r#"{{"session_id":"{SESSION}","content":"ok"}}"#);
    let p: AssistantFinalRecordedPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(p.turn_id, None);
}

#[test]
fn assistant_final_recorded_payload__turn_id_present__round_trip__ac1() {
    let json = format!(r#"{{"session_id":"{SESSION}","content":"ok","turn_id":"{TURN}"}}"#);
    let p: AssistantFinalRecordedPayload = serde_json::from_str(&json).expect("deserialize");
    let expected = TurnId::from_uuid(uuid::Uuid::parse_str(TURN).unwrap());
    assert_eq!(p.turn_id, Some(expected));
    let again: AssistantFinalRecordedPayload =
        serde_json::from_value(serde_json::to_value(&p).expect("ser")).expect("de");
    assert_eq!(again.turn_id, Some(expected));
}
