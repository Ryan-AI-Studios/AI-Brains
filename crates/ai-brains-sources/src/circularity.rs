//! Circularity classification for external memory items (T156 / P6.4).
//!
//! # Invariants
//!
//! - [`classify_circularity`] **never** returns [`CircularityClass::Independent`].
//!   Unlabeled external content that does not match markers or the outbound index
//!   is **`Unknown`** (fail closed). Paraphrases of control-plane writes defeat
//!   literal markers and fingerprint identity — treating “no evidence of echo”
//!   as independence would allow self-amplifying claims.
//! - [`CircularityClass::Independent`] is assigned only via an explicit trusted
//!   construction path ([`meta_with_assert_independent`] when callers pass
//!   `assert_independent: true` from a trusted surface — e.g. connector
//!   `trust_assert_independent` + fixture field), never by the classifier and
//!   never from untrusted path-export JSON alone.
//! - [`OutboundIndex`] is **empty in production v1**. Rule 2 (fingerprint /
//!   origin event match against the index) is real in tests when fixtures seed
//!   the index, but is **not** a live production second defense layer until a
//!   later track records outbound exports. Production defense is: rule 1
//!   (markers when present) + rule 3 fail-closed `Unknown` +
//!   [`may_count_as_independent_support`] true only for `Independent`.
//!
//! # Detection rules (v1)
//!
//! | Rule | Condition | Class |
//! |------|-----------|--------|
//! | 1 | Origin markers present (`origin_event_id` / `origin_source_id` /
//!   `origin_marker` non-empty, or known payload keys) | `EchoOfControlPlane` |
//! | 2 | Fingerprint or `origin_event_id` matches [`OutboundIndex`] | `EchoOfControlPlane` |
//! | 3 | Else | `Unknown` — **never** `Independent` |

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Schema version for [`ExternalItemMeta`] JSON.
pub const EXTERNAL_ITEM_META_SCHEMA_VERSION: u32 = 1;

/// Known origin-marker keys accepted in external payloads (any one is enough).
pub const ORIGIN_MARKER_KEYS: &[&str] = &[
    "ai_brains_event_id",
    "origin_event_id",
    "ai_brains_source_id",
    "origin_source_id",
];

/// Circularity class for an external memory item relative to this control plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum CircularityClass {
    /// Explicitly attested as independent (trusted construction only).
    Independent,
    /// Item originated from (or echoes) this control plane.
    EchoOfControlPlane,
    /// No markers and no index match — fail closed; not independent support.
    Unknown,
}

/// Versioned metadata attached to external memory observations.
///
/// Embedded in Hermes/Honcho observe content JSON (`schema_version` = 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalItemMeta {
    /// Always [`EXTERNAL_ITEM_META_SCHEMA_VERSION`] for v1 payloads.
    pub schema_version: u32,
    /// Provider id string (`"hermes"` / `"honcho"`).
    pub provider: String,
    /// Provider-native item id (session id, confirmed-item id, …).
    pub provider_item_id: String,
    /// Optional recorded-at timestamp (provider string; opaque to classifier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<String>,
    /// Origin AI-Brains event id when re-exported by the external tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_event_id: Option<String>,
    /// Origin AI-Brains source id when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_source_id: Option<String>,
    /// Stable outbound marker stamped on writes (future).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_marker: Option<String>,
    /// Classification result (or explicit Independent from trusted path).
    pub circularity: CircularityClass,
    /// When true on trusted import, construction may set Independent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assert_independent: Option<bool>,
}

/// Seedable set of fingerprints and/or origin event ids known to have left
/// this control plane.
///
/// **Production v1:** leave empty. There is no outbound export recorder in the
/// read-only T156 adapters, so rule 2 is not a live production second layer.
/// Tests seed this index to prove rule 2 classification.
#[derive(Debug, Clone, Default)]
pub struct OutboundIndex {
    fingerprints: HashSet<String>,
    origin_event_ids: HashSet<String>,
}

impl OutboundIndex {
    /// Empty index (production default).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Whether this index contains no entries.
    pub fn is_empty(&self) -> bool {
        self.fingerprints.is_empty() && self.origin_event_ids.is_empty()
    }

    /// Insert a content fingerprint that was previously written outbound.
    pub fn insert_fingerprint(&mut self, fingerprint: impl Into<String>) {
        self.fingerprints.insert(fingerprint.into());
    }

    /// Insert an origin event id known to have left the control plane.
    pub fn insert_origin_event_id(&mut self, event_id: impl Into<String>) {
        self.origin_event_ids.insert(event_id.into());
    }

    /// True if `fingerprint` is registered.
    pub fn contains_fingerprint(&self, fingerprint: &str) -> bool {
        self.fingerprints.contains(fingerprint)
    }

    /// True if `event_id` is registered.
    pub fn contains_origin_event_id(&self, event_id: &str) -> bool {
        self.origin_event_ids.contains(event_id)
    }
}

/// Classify circularity for an external item.
///
/// # Invariant
///
/// **Never** returns [`CircularityClass::Independent`]. Independent is only
/// assigned via [`meta_with_assert_independent`] (or equivalent trusted
/// construction). Paraphrased re-exports without markers fail closed as
/// [`CircularityClass::Unknown`].
///
/// # Note on OutboundIndex
///
/// In production v1 the index is empty, so rule 2 does not fire unless a later
/// track seeds outbound accounting. Prefer
/// [`classify_circularity_with_fingerprint`] when a content fingerprint is
/// available so rule 2 can match seeded outbound fingerprints in tests.
pub fn classify_circularity(meta: &ExternalItemMeta, outbound: &OutboundIndex) -> CircularityClass {
    classify_circularity_with_fingerprint(meta, outbound, None)
}

/// Like [`classify_circularity`], also applying rule 2 against an optional
/// content fingerprint (and treating `provider_item_id` as a fingerprint key
/// when present in the index — convenient for unit tests).
pub fn classify_circularity_with_fingerprint(
    meta: &ExternalItemMeta,
    outbound: &OutboundIndex,
    content_fingerprint: Option<&str>,
) -> CircularityClass {
    // Rule 1: origin markers on the meta fields.
    if non_empty_opt(meta.origin_event_id.as_deref())
        || non_empty_opt(meta.origin_source_id.as_deref())
        || non_empty_opt(meta.origin_marker.as_deref())
    {
        return CircularityClass::EchoOfControlPlane;
    }

    // Rule 2: outbound index match (test / future; empty in prod v1).
    // origin_event_id non-empty already returns Echo via rule 1; still check
    // the index for fingerprint keys and for event ids when only the index
    // (not the meta field) carries the signal in future call shapes.
    if let Some(fp) = content_fingerprint {
        let t = fp.trim();
        if !t.is_empty() && outbound.contains_fingerprint(t) {
            return CircularityClass::EchoOfControlPlane;
        }
    }
    if outbound.contains_fingerprint(&meta.provider_item_id) {
        return CircularityClass::EchoOfControlPlane;
    }
    if let Some(eid) = meta.origin_event_id.as_deref() {
        let t = eid.trim();
        if !t.is_empty() && outbound.contains_origin_event_id(t) {
            return CircularityClass::EchoOfControlPlane;
        }
    }

    // Rule 3: fail closed — never Independent.
    CircularityClass::Unknown
}

/// Classify using meta fields plus optional raw JSON payload (marker keys).
///
/// Payload keys in [`ORIGIN_MARKER_KEYS`] with non-empty string values count
/// as rule-1 markers even when meta fields are unset.
pub fn classify_circularity_with_payload(
    meta: &ExternalItemMeta,
    outbound: &OutboundIndex,
    payload: Option<&serde_json::Value>,
) -> CircularityClass {
    if let Some(value) = payload
        && payload_has_origin_markers(value)
    {
        return CircularityClass::EchoOfControlPlane;
    }
    classify_circularity(meta, outbound)
}

/// Whether this class may count as independent corroborating evidence.
///
/// Only [`CircularityClass::Independent`] returns `true`. Echo and Unknown
/// fail closed.
pub fn may_count_as_independent_support(class: CircularityClass) -> bool {
    matches!(class, CircularityClass::Independent)
}

/// Inputs for trusted / classified [`ExternalItemMeta`] construction.
#[derive(Debug, Clone, Default)]
pub struct ExternalItemMetaInput {
    pub provider: String,
    pub provider_item_id: String,
    pub origin_event_id: Option<String>,
    pub origin_source_id: Option<String>,
    pub origin_marker: Option<String>,
    pub recorded_at: Option<String>,
    /// When true, construction sets Independent without classify.
    pub assert_independent: bool,
}

/// Build meta with optional trusted Independent assertion.
///
/// When `input.assert_independent` is true, sets
/// [`CircularityClass::Independent`] without running the classifier. When
/// false, runs [`classify_circularity`] (which never returns Independent).
pub fn meta_with_assert_independent(
    input: ExternalItemMetaInput,
    outbound: &OutboundIndex,
) -> ExternalItemMeta {
    let assert_independent = input.assert_independent;
    let mut meta = ExternalItemMeta {
        schema_version: EXTERNAL_ITEM_META_SCHEMA_VERSION,
        provider: input.provider,
        provider_item_id: input.provider_item_id,
        recorded_at: input.recorded_at,
        origin_event_id: input.origin_event_id,
        origin_source_id: input.origin_source_id,
        origin_marker: input.origin_marker,
        circularity: CircularityClass::Unknown,
        assert_independent: if assert_independent { Some(true) } else { None },
    };

    if assert_independent {
        meta.circularity = CircularityClass::Independent;
    } else {
        meta.circularity = classify_circularity(&meta, outbound);
    }
    meta
}

/// Filter metas to those that may count as independent support.
pub fn filter_independent_support(metas: &[ExternalItemMeta]) -> Vec<&ExternalItemMeta> {
    metas
        .iter()
        .filter(|m| may_count_as_independent_support(m.circularity))
        .collect()
}

/// Filter by circularity class list (accept only listed classes).
pub fn filter_by_circularity_classes<'a>(
    metas: &'a [ExternalItemMeta],
    accept: &[CircularityClass],
) -> Vec<&'a ExternalItemMeta> {
    metas
        .iter()
        .filter(|m| accept.contains(&m.circularity))
        .collect()
}

/// Extract non-empty origin marker strings from a JSON value (object keys).
///
/// Walks top-level object keys listed in [`ORIGIN_MARKER_KEYS`] and nested
/// `external_item_meta` / `meta` objects when present.
pub fn extract_origin_markers_from_value(value: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    collect_markers(value, &mut out);
    out
}

/// Extract origin markers from raw UTF-8 JSON bytes. Non-JSON → empty.
pub fn extract_origin_markers_from_bytes(bytes: &[u8]) -> Vec<String> {
    match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(v) => extract_origin_markers_from_value(&v),
        Err(_) => Vec::new(),
    }
}

/// True when JSON payload contains any known non-empty origin marker key.
pub fn payload_has_origin_markers(value: &serde_json::Value) -> bool {
    !extract_origin_markers_from_value(value).is_empty()
}

fn non_empty_opt(s: Option<&str>) -> bool {
    s.map(|v| !v.trim().is_empty()).unwrap_or(false)
}

fn collect_markers(value: &serde_json::Value, out: &mut Vec<String>) {
    let Some(obj) = value.as_object() else {
        return;
    };
    for key in ORIGIN_MARKER_KEYS {
        if let Some(v) = obj.get(*key) {
            push_marker_value(v, out);
        }
    }
    // Nested meta objects commonly used by connectors.
    for nest in ["external_item_meta", "meta", "origin"] {
        if let Some(nested) = obj.get(nest) {
            collect_markers(nested, out);
        }
    }
}

fn push_marker_value(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::String(s) if !s.trim().is_empty() => {
            out.push(s.trim().to_string());
        }
        serde_json::Value::Number(n) => {
            out.push(n.to_string());
        }
        serde_json::Value::Bool(b) => {
            // Boolean markers are not treated as origin lineage.
            let _ = b;
        }
        _ => {}
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    fn bare_meta(provider_item_id: &str) -> ExternalItemMeta {
        ExternalItemMeta {
            schema_version: EXTERNAL_ITEM_META_SCHEMA_VERSION,
            provider: "hermes".into(),
            provider_item_id: provider_item_id.into(),
            recorded_at: None,
            origin_event_id: None,
            origin_source_id: None,
            origin_marker: None,
            circularity: CircularityClass::Unknown,
            assert_independent: None,
        }
    }

    #[test]
    fn circularity__origin_event_id_present__echo() {
        let mut meta = bare_meta("sess-1");
        meta.origin_event_id = Some("evt-abc".into());
        let class = classify_circularity(&meta, &OutboundIndex::empty());
        assert_eq!(class, CircularityClass::EchoOfControlPlane);
    }

    #[test]
    fn circularity__ai_brains_marker_key__echo() {
        let meta = bare_meta("sess-2");
        let payload = serde_json::json!({
            "summary": "hello",
            "ai_brains_event_id": "evt-from-payload"
        });
        let class =
            classify_circularity_with_payload(&meta, &OutboundIndex::empty(), Some(&payload));
        assert_eq!(class, CircularityClass::EchoOfControlPlane);
    }

    #[test]
    fn circularity__no_markers_no_index_match__unknown() {
        let meta = bare_meta("sess-3");
        let class = classify_circularity(&meta, &OutboundIndex::empty());
        assert_eq!(class, CircularityClass::Unknown);
        assert_ne!(class, CircularityClass::Independent);
    }

    #[test]
    fn circularity__no_markers_no_index_match__not_independent_support() {
        let meta = bare_meta("sess-4");
        let class = classify_circularity(&meta, &OutboundIndex::empty());
        assert!(!may_count_as_independent_support(class));
    }

    #[test]
    fn circularity__may_count_independent__only_independent_true() {
        assert!(may_count_as_independent_support(
            CircularityClass::Independent
        ));
        assert!(!may_count_as_independent_support(
            CircularityClass::EchoOfControlPlane
        ));
        assert!(!may_count_as_independent_support(CircularityClass::Unknown));
    }

    #[test]
    fn circularity__echo__cannot_be_independent_support() {
        assert!(!may_count_as_independent_support(
            CircularityClass::EchoOfControlPlane
        ));
    }

    #[test]
    fn circularity__unknown__cannot_be_independent_support() {
        assert!(!may_count_as_independent_support(CircularityClass::Unknown));
    }

    fn make_meta(
        provider: &str,
        id: &str,
        origin_event_id: Option<&str>,
        assert_independent: bool,
        outbound: &OutboundIndex,
    ) -> ExternalItemMeta {
        meta_with_assert_independent(
            ExternalItemMetaInput {
                provider: provider.into(),
                provider_item_id: id.into(),
                origin_event_id: origin_event_id.map(str::to_string),
                origin_source_id: None,
                origin_marker: None,
                recorded_at: None,
                assert_independent,
            },
            outbound,
        )
    }

    #[test]
    fn circularity__assert_independent_fixture__may_support() {
        let meta = make_meta(
            "hermes",
            "sess-trusted",
            None,
            true,
            &OutboundIndex::empty(),
        );
        assert_eq!(meta.circularity, CircularityClass::Independent);
        assert!(may_count_as_independent_support(meta.circularity));
        assert_eq!(meta.assert_independent, Some(true));
    }

    #[test]
    fn circularity__outbound_index_match__echo() {
        let mut outbound = OutboundIndex::empty();
        outbound.insert_fingerprint("fp-known-out");
        let meta = bare_meta("fp-known-out");
        let class = classify_circularity(&meta, &outbound);
        assert_eq!(class, CircularityClass::EchoOfControlPlane);
    }

    #[test]
    fn circularity__classify_never_returns_independent() {
        // Exhaustive-ish: markers, no markers, empty index, seeded index miss.
        let cases = [
            bare_meta("a"),
            {
                let mut m = bare_meta("b");
                m.origin_event_id = Some("e".into());
                m
            },
            {
                let mut m = bare_meta("c");
                m.origin_source_id = Some("s".into());
                m
            },
            {
                let mut m = bare_meta("d");
                m.origin_marker = Some("m".into());
                m
            },
        ];
        let outbound = OutboundIndex::empty();
        for meta in &cases {
            let class = classify_circularity(meta, &outbound);
            assert_ne!(
                class,
                CircularityClass::Independent,
                "classify must never return Independent for {:?}",
                meta.provider_item_id
            );
        }
        // Payload markers path.
        let payload = serde_json::json!({"ai_brains_source_id": "src-1"});
        let class = classify_circularity_with_payload(&bare_meta("x"), &outbound, Some(&payload));
        assert_ne!(class, CircularityClass::Independent);
    }

    #[test]
    fn independent_support_gate__echo_evidence_ids_rejected() {
        let metas = vec![make_meta(
            "honcho",
            "item-echo",
            Some("evt-1"),
            false,
            &OutboundIndex::empty(),
        )];
        assert_eq!(metas[0].circularity, CircularityClass::EchoOfControlPlane);
        let accepted = filter_independent_support(&metas);
        assert!(accepted.is_empty());
    }

    #[test]
    fn independent_support_gate__unknown_ids_rejected() {
        let metas = vec![make_meta(
            "honcho",
            "item-unk",
            None,
            false,
            &OutboundIndex::empty(),
        )];
        assert_eq!(metas[0].circularity, CircularityClass::Unknown);
        let accepted = filter_independent_support(&metas);
        assert!(accepted.is_empty());
    }

    #[test]
    fn independent_support_gate__independent_ids_accepted() {
        let metas = vec![make_meta(
            "honcho",
            "item-ok",
            None,
            true,
            &OutboundIndex::empty(),
        )];
        let accepted = filter_independent_support(&metas);
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].provider_item_id, "item-ok");
    }

    #[test]
    fn independent_support_gate__mixed_set__filters_echo_and_unknown() {
        let outbound = OutboundIndex::empty();
        let metas = vec![
            make_meta("hermes", "echo-1", Some("e1"), false, &outbound),
            make_meta("hermes", "unk-1", None, false, &outbound),
            make_meta("hermes", "ind-1", None, true, &outbound),
            make_meta("honcho", "ind-2", None, true, &outbound),
        ];
        let accepted = filter_independent_support(&metas);
        let ids: Vec<&str> = accepted
            .iter()
            .map(|m| m.provider_item_id.as_str())
            .collect();
        assert_eq!(ids, vec!["ind-1", "ind-2"]);
    }

    #[test]
    fn extract_markers__nested_meta_object() {
        let v = serde_json::json!({
            "body": "x",
            "external_item_meta": {
                "origin_event_id": " nested-evt "
            }
        });
        let markers = extract_origin_markers_from_value(&v);
        assert_eq!(markers, vec!["nested-evt"]);
        assert!(payload_has_origin_markers(&v));
    }
}
