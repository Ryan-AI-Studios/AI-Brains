//! T319 — vault `memory_id` vs governed handle namespace honesty (CLI overlay).
//!
//! After `expand_handle` / source miss, probe `memory_exists` and name the other
//! namespace. Do **not** grow `governed_common.rs`. Do **not** edit control-plane
//! `expand_handle`. Do **not** import `graph::vault_memory_present` (feature-gated).

use crate::commands::governed_common::{LIST_RECALL_QUERY, UNKNOWN_HANDLE_PREVIEW};

/// F6 — human/JSON preview when the UUID exists as a vault memory, not a handle.
pub(crate) const WRONG_NAMESPACE_PREVIEW: &str =
    "This UUID is a vault memory_id, not a governed handle.";

/// F1 — map `memory_exists` Result without `?` (copy of graph helper; graph is cfg-gated).
pub(crate) fn namespace_memory_present<E: std::fmt::Display>(result: Result<bool, E>) -> bool {
    match result {
        Ok(true) => true,
        Ok(false) => false,
        Err(err) => {
            tracing::warn!(error = %err, "memory_exists failed; treating id as unknown");
            false
        }
    }
}

/// F6 human next line (`next: ai-brains recall "…"`).
pub(crate) fn wrong_namespace_next_line() -> String {
    format!("next: ai-brains recall \"{LIST_RECALL_QUERY}\"")
}

/// F6 JSON `next_step` (no `next:` prefix; no `(Pinned: N)`).
pub(crate) fn wrong_namespace_json_next() -> String {
    format!("ai-brains recall \"{LIST_RECALL_QUERY}\"")
}

/// F4 / AC4 — source `details.hint` one string.
pub(crate) fn wrong_namespace_source_hint() -> String {
    format!("{WRONG_NAMESPACE_PREVIEW} {}", wrong_namespace_next_line())
}

/// Overlay Unknown expand/evidence JSON after `to_value`.
///
/// - `memory_exists=true` → F6 preview + insert `next_step` (replace T263, do not stack).
/// - `memory_exists=false` → T263 `Handle not found.` when preview empty; no `next_step`.
/// - Non-Unknown kinds → unchanged.
pub(crate) fn apply_unknown_handle_overlay(value: &mut serde_json::Value, memory_exists: bool) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    if obj.get("kind").and_then(|k| k.as_str()) != Some("Unknown") {
        return;
    }
    if memory_exists {
        obj.insert(
            "preview".to_string(),
            serde_json::Value::String(WRONG_NAMESPACE_PREVIEW.to_string()),
        );
        obj.insert(
            "next_step".to_string(),
            serde_json::Value::String(wrong_namespace_json_next()),
        );
        return;
    }
    // Unknown-unknown: fill empty preview with T263 const; never insert next_step.
    let empty = obj
        .get("preview")
        .and_then(|p| p.as_str())
        .is_none_or(|s| s.is_empty());
    if empty {
        obj.insert(
            "preview".to_string(),
            serde_json::Value::String(UNKNOWN_HANDLE_PREVIEW.to_string()),
        );
    }
    obj.remove("next_step");
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use serde_json::json;

    /// AC1 — memory EXISTS → F6 preview + JSON next_step.
    #[test]
    fn apply_unknown_handle_overlay__memory_exists__preview_and_next_step() {
        let mut v = json!({
            "api_version": "1",
            "handle_id": "431f6505-50d7-5176-8cda-f8ba2534fe14",
            "kind": "Unknown",
            "preview": "",
            "truncated": false
        });
        apply_unknown_handle_overlay(&mut v, true);
        assert_eq!(
            v["preview"].as_str(),
            Some(WRONG_NAMESPACE_PREVIEW),
            "AC1 preview F6; got {v}"
        );
        assert_eq!(
            v["next_step"].as_str(),
            Some(wrong_namespace_json_next().as_str()),
            "AC1 next_step F6 JSON; got {v}"
        );
        assert_eq!(v["kind"].as_str(), Some("Unknown"), "kind stays Unknown");
    }

    /// AC2 — unknown-unknown → Handle not found.; no next_step.
    #[test]
    fn apply_unknown_handle_overlay__unknown_unknown__handle_not_found_no_next() {
        let mut v = json!({
            "api_version": "1",
            "handle_id": "00000000-0000-0000-0000-000000000000",
            "kind": "Unknown",
            "preview": "",
            "truncated": false
        });
        apply_unknown_handle_overlay(&mut v, false);
        assert_eq!(
            v["preview"].as_str(),
            Some(UNKNOWN_HANDLE_PREVIEW),
            "AC2 T263 preview; got {v}"
        );
        assert!(
            v.get("next_step").is_none(),
            "AC2 must not insert next_step; got {v}"
        );
    }

    /// AC3 — found-kind fixture + Denied unchanged (OpenCode m3 / F23).
    #[test]
    fn apply_unknown_handle_overlay__non_unknown__unchanged() {
        let mut evidence = json!({
            "api_version": "1",
            "handle_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "kind": "Evidence",
            "preview": "seeded evidence body",
            "truncated": false
        });
        let before = evidence.clone();
        apply_unknown_handle_overlay(&mut evidence, true);
        assert_eq!(evidence, before, "AC3 Evidence unchanged; got {evidence}");
        assert!(
            evidence.get("next_step").is_none(),
            "AC3 Evidence must not gain next_step"
        );

        // Live CP may emit `Evidence:Active` (status suffix); overlay keys on exact Unknown only.
        let mut evidence_active = json!({
            "api_version": "1",
            "handle_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "kind": "Evidence:Active",
            "preview": "seeded evidence body",
            "truncated": false
        });
        let before_active = evidence_active.clone();
        apply_unknown_handle_overlay(&mut evidence_active, true);
        assert_eq!(
            evidence_active, before_active,
            "AC3 Evidence:Active unchanged; got {evidence_active}"
        );

        let mut denied = json!({
            "api_version": "1",
            "handle_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
            "kind": "Denied",
            "preview": "",
            "truncated": false
        });
        let before_denied = denied.clone();
        apply_unknown_handle_overlay(&mut denied, true);
        assert_eq!(denied, before_denied, "AC3 Denied unchanged; got {denied}");
        assert!(
            denied.get("next_step").is_none(),
            "AC3 Denied must not gain next_step"
        );
    }

    /// AC4 — source hint is exact `{F6 preview} {F6 next line}`.
    #[test]
    fn wrong_namespace_source_hint__contains_preview_and_next() {
        let hint = wrong_namespace_source_hint();
        let expected = format!("{WRONG_NAMESPACE_PREVIEW} {}", wrong_namespace_next_line());
        assert_eq!(hint, expected, "AC4 exact source hint");
    }

    #[test]
    fn namespace_memory_present__err__false() {
        assert!(!namespace_memory_present(Err::<bool, &str>("locked")));
        assert!(namespace_memory_present(Ok::<bool, &str>(true)));
        assert!(!namespace_memory_present(Ok::<bool, &str>(false)));
    }

    /// F6 replace — pre-filled Handle not found. must not stack with F6.
    #[test]
    fn apply_unknown_handle_overlay__memory_exists__replaces_handle_not_found() {
        let mut v = json!({
            "kind": "Unknown",
            "preview": "Handle not found."
        });
        apply_unknown_handle_overlay(&mut v, true);
        assert_eq!(v["preview"].as_str(), Some(WRONG_NAMESPACE_PREVIEW));
        assert!(
            !v["preview"]
                .as_str()
                .unwrap_or("")
                .contains("Handle not found."),
            "must replace, not stack; got {v}"
        );
    }
}
