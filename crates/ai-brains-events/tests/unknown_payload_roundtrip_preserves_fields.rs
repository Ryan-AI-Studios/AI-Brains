//! T180-E-r0-unknown elevate — R0 Unknown payload fidelity (T148).
//! See also `protocol_compat_events.rs` and Docs/PROTOCOL-COMPAT.md §6.
#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_core::privacy::Privacy;
use ai_brains_events::hash::compute_payload_hash;
use ai_brains_events::{
    Actor, AggregateType, Envelope, EventKind, Payload, ProjectRegisteredPayload,
};
use time::OffsetDateTime;
use uuid::Uuid;

#[test]
fn envelope_with_unknown_payload__serialize_deserialize_preserves_fields() {
    let raw_payload = serde_json::json!({
        "type": "TotallyFutureEvent",
        "foo": 1,
        "bar": "x",
        "nested": { "a": true }
    });

    let envelope = Envelope {
        event_id: Uuid::from_u128(1),
        schema_version: 1,
        aggregate_type: AggregateType::System,
        aggregate_id: Uuid::nil(),
        event_type: EventKind::Unknown("TotallyFutureEvent".to_string()),
        occurred_at: OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("valid ts"),
        actor: Actor::System,
        causation_id: None,
        correlation_id: None,
        privacy: Privacy::LocalOnly,
        payload: Payload::Unknown(raw_payload.clone()),
        payload_hash: "deadbeef".to_string(),
    };

    let as_value = serde_json::to_value(&envelope).expect("envelope serialize");
    let back: Envelope = serde_json::from_value(as_value).expect("envelope deserialize");

    assert_eq!(
        back.event_type,
        EventKind::Unknown("TotallyFutureEvent".to_string())
    );
    match back.payload {
        Payload::Unknown(v) => {
            assert_eq!(v, raw_payload);
            assert_eq!(v.get("foo").and_then(|x| x.as_i64()), Some(1));
            assert_eq!(
                v.get("nested")
                    .and_then(|n| n.get("a"))
                    .and_then(|a| a.as_bool()),
                Some(true)
            );
        }
        other => panic!("expected Unknown, got {other:?}"),
    }
    // Pass-through hash must not be forced to recompute for Unknown-only round-trip.
    assert_eq!(back.payload_hash, "deadbeef");
}

#[test]
fn event_kind_unknown__preserves_original_tag_on_roundtrip() {
    let kind = EventKind::Unknown("TotallyFutureEvent".to_string());
    let s = serde_json::to_string(&kind).expect("serialize kind");
    assert_eq!(s, "\"TotallyFutureEvent\"");
    let back: EventKind = serde_json::from_str(&s).expect("deserialize kind");
    assert_eq!(back, EventKind::Unknown("TotallyFutureEvent".to_string()));
}

#[test]
fn known_payload__still_roundtrips_and_hashes() {
    let payload = Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id: ai_brains_core::ids::ProjectId::from_uuid(Uuid::from_u128(42)),
        name: "demo".to_string(),
        tx_id: None,
    });
    let v = serde_json::to_value(&payload).expect("ser");
    assert_eq!(
        v.get("type").and_then(|t| t.as_str()),
        Some("ProjectRegistered")
    );
    let back: Payload = serde_json::from_value(v).expect("de");
    assert_eq!(back, payload);
    let hash = compute_payload_hash(&payload).expect("hash");
    assert!(!hash.is_empty());
}
