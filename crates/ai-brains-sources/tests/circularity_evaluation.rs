#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! T169 scenario 10 — circular external write-back hard gates (sources crate).

use ai_brains_sources::{
    CircularityClass, ExternalItemMeta, OutboundIndex, classify_circularity,
    may_count_as_independent_support,
};

fn unlabeled(provider: &str, item: &str) -> ExternalItemMeta {
    ExternalItemMeta {
        schema_version: 1,
        provider: provider.into(),
        provider_item_id: item.into(),
        recorded_at: None,
        origin_event_id: None,
        origin_source_id: None,
        origin_marker: None,
        circularity: CircularityClass::Unknown,
        assert_independent: None,
    }
}

fn echo(provider: &str, item: &str, origin_event: &str) -> ExternalItemMeta {
    ExternalItemMeta {
        schema_version: 1,
        provider: provider.into(),
        provider_item_id: item.into(),
        recorded_at: None,
        origin_event_id: Some(origin_event.into()),
        origin_source_id: None,
        origin_marker: None,
        circularity: CircularityClass::Unknown,
        assert_independent: None,
    }
}

#[test]
fn metric_independent_support__false_positive__counts() {
    let outbound = OutboundIndex::empty();

    let unmarked = unlabeled("hermes", "session-1");
    let class = classify_circularity(&unmarked, &outbound);
    assert_ne!(class, CircularityClass::Independent);
    assert!(
        !may_count_as_independent_support(class),
        "unlabeled must not count as independent support"
    );

    let echo_item = echo("honcho", "item-2", "evt-origin-1");
    let echo_class = classify_circularity(&echo_item, &outbound);
    assert_eq!(echo_class, CircularityClass::EchoOfControlPlane);
    assert!(
        !may_count_as_independent_support(echo_class),
        "echo must not count as independent support"
    );
}

#[test]
fn scenario__circular_external_writeback__hard_gates_pass() {
    let outbound = OutboundIndex::empty();

    // Hard: Echo / unlabeled → not Independent; may_count_as_independent_support false.
    let cases = [
        unlabeled("hermes", "u1"),
        unlabeled("honcho", "u2"),
        echo("hermes", "e1", "evt-1"),
        echo("honcho", "e2", "evt-2"),
    ];

    let mut false_positives = 0u64;
    for meta in &cases {
        let class = classify_circularity(meta, &outbound);
        if class == CircularityClass::Independent {
            false_positives += 1;
        }
        if may_count_as_independent_support(class) {
            false_positives += 1;
        }
    }
    assert_eq!(
        false_positives, 0,
        "independent_support_false_positive must be 0"
    );
}
