#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_events::Payload;

#[test]
fn unknown_future_payload__deserializes_to_unknown_with_fields() {
    let input = serde_json::json!({
        "type": "TotallyFutureEvent",
        "foo": 1,
        "bar": "x"
    });

    let payload: Payload =
        serde_json::from_value(input.clone()).expect("unknown payload must deserialize");

    match &payload {
        Payload::Unknown(v) => {
            assert_eq!(v.get("foo").and_then(|x| x.as_i64()), Some(1));
            assert_eq!(v.get("bar").and_then(|x| x.as_str()), Some("x"));
            assert_eq!(
                v.get("type").and_then(|x| x.as_str()),
                Some("TotallyFutureEvent")
            );
        }
        other => panic!("expected Payload::Unknown, got {other:?}"),
    }

    let round1 = serde_json::to_value(&payload).expect("serialize unknown");
    assert_eq!(round1, input, "re-serialized Value must equal input");

    let again: Payload = serde_json::from_value(round1.clone()).expect("re-deserialize");
    let round2 = serde_json::to_value(&again).expect("second serialize");
    assert_eq!(round2, input);
}
