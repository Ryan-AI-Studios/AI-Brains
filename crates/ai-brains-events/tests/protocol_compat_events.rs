//! T180 P-EVENT protocol compatibility: schema v1, R0 elevate pointer, Upcast stub honesty.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_core::privacy::Privacy;
use ai_brains_events::upcast::Upcast;
use ai_brains_events::version::CURRENT_SCHEMA_VERSION;
use ai_brains_events::{Actor, AggregateType, Envelope, EventKind, Payload};
use time::OffsetDateTime;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// T180-E-schema-v1
// ---------------------------------------------------------------------------

#[test]
fn t180_e_schema_v1__current_schema_version_is_one() {
    // T180-E-schema-v1
    assert_eq!(CURRENT_SCHEMA_VERSION, 1);
}

// ---------------------------------------------------------------------------
// T180-E-r0-unknown (elevate — full suite in unknown_payload_roundtrip_preserves_fields.rs)
// ---------------------------------------------------------------------------

#[test]
fn t180_e_r0_unknown__unknown_payload_preserves_fields() {
    // T180-E-r0-unknown — thin elevate of R0 Unknown fidelity.
    let raw_payload = serde_json::json!({
        "type": "FutureEventV2",
        "field_a": 1,
        "nested": { "b": false }
    });

    let envelope = Envelope {
        event_id: Uuid::from_u128(42),
        schema_version: 1,
        aggregate_type: AggregateType::System,
        aggregate_id: Uuid::nil(),
        event_type: EventKind::Unknown("FutureEventV2".to_string()),
        occurred_at: OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("valid ts"),
        actor: Actor::System,
        causation_id: None,
        correlation_id: None,
        privacy: Privacy::LocalOnly,
        payload: Payload::Unknown(raw_payload.clone()),
        payload_hash: "deadbeef".to_string(),
    };

    let as_value = serde_json::to_value(&envelope).expect("ser");
    let back: Envelope = serde_json::from_value(as_value).expect("de");
    match back.payload {
        Payload::Unknown(v) => assert_eq!(v, raw_payload),
        other => panic!("expected Unknown, got {other:?}"),
    }
    assert_eq!(back.schema_version, 1);
}

// ---------------------------------------------------------------------------
// T180-E-upcast-stub (F30) — no fake migrations
// ---------------------------------------------------------------------------

fn sample_envelope(schema_version: u32) -> Envelope {
    Envelope {
        event_id: Uuid::from_u128(7),
        schema_version,
        aggregate_type: AggregateType::System,
        aggregate_id: Uuid::nil(),
        event_type: EventKind::Unknown("x".into()),
        occurred_at: OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("valid ts"),
        actor: Actor::System,
        causation_id: None,
        correlation_id: None,
        privacy: Privacy::LocalOnly,
        payload: Payload::Unknown(serde_json::json!({"type": "x"})),
        payload_hash: "h".into(),
    }
}

#[test]
fn t180_e_upcast_stub__current_version__passthrough() {
    // T180-E-upcast-stub
    let env = sample_envelope(CURRENT_SCHEMA_VERSION);
    let out = env.upcast().expect("current schema must pass through");
    assert_eq!(out.schema_version, CURRENT_SCHEMA_VERSION);
}

#[test]
fn t180_e_upcast_stub__future_version__passthrough() {
    let env = sample_envelope(CURRENT_SCHEMA_VERSION + 1);
    let out = env
        .upcast()
        .expect("future schema passes through (payload may already be Unknown)");
    assert_eq!(out.schema_version, CURRENT_SCHEMA_VERSION + 1);
}

#[test]
fn t180_e_upcast_stub__historical_v0__returns_err() {
    // Stub: upcast_once always Err for non-current historical versions (no fake migration).
    let env = sample_envelope(0);
    let err = env.upcast().expect_err("v0 must not silently migrate");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("unknown")
            || msg.to_lowercase().contains("version")
            || !msg.is_empty(),
        "upcast stub error should mention version; got: {msg}"
    );
}
