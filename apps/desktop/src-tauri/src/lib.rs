//! AI-Brains desktop host library (Tauri commands + WebView2 diagnostic).

mod commands;
mod webview2;

pub use commands::{
    DaemonConnectionInfo, InvokeApiError, PingResponse, daemon_connection_info_payload,
    get_daemon_connection_info, open_url, ping, ping_payload, reveal_path,
};
pub use webview2::{
    WEBVIEW2_BOOTSTRAPPER_URL, WebView2Status, detect_webview2, ensure_webview2_or_exit,
};

/// Run the Tauri application (after WebView2 check in `main`).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // WebView2 is enforced in main.rs before this; double-check is cheap.
    ensure_webview2_or_exit();

    // `generate_context!` expands to code that uses expect/unwrap for compile-time
    // embedded config; clippy disallowed_methods is suppressed for that macro only.
    #[allow(clippy::disallowed_methods)]
    let context = tauri::generate_context!();

    let result = tauri::Builder::default()
        // Single-instance first (soft G), then opener dual-layer (U3).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::get_daemon_connection_info,
            commands::api::project_briefing,
            commands::api::personal_briefing,
            commands::api::query_knowledge,
            commands::api::inspect_evidence,
            commands::api::inspect_source,
            commands::api::list_review_items,
            commands::api::resolve_review_item,
            commands::api::resolve_scope,
            commands::api::request_erasure,
            commands::api::wipe_content_envelope,
            commands::api::probe_health,
            commands::open::open_url,
            commands::open::reveal_path,
        ])
        .run(context);

    if let Err(e) = result {
        eprintln!("ai-brains-desktop failed to start: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod csp_tests {
    /// CSP regression (SC4 / S7 / M24 / T173): production `csp` must stay strict.
    /// `devCsp` may relax for Vite HMR; production string must not.
    #[test]
    fn tauri_conf__csp__non_null_default_src_self_and_connect_src_ipc() {
        let conf = include_str!("../tauri.conf.json");
        let value: serde_json::Value = match serde_json::from_str(conf) {
            Ok(v) => v,
            Err(e) => panic!("tauri.conf.json must be valid JSON: {e}"),
        };

        let csp = match value.pointer("/app/security/csp") {
            Some(v) => v,
            None => panic!("app.security.csp must be present"),
        };

        assert!(
            !csp.is_null(),
            "CSP must not be null (null disables CSP protection)"
        );

        let csp_text = csp_to_text(csp);

        assert!(
            csp_text.contains("default-src") && csp_text.contains("'self'"),
            "CSP must include default-src with 'self'; got: {csp_text}"
        );
        assert!(
            csp_text.contains("connect-src") && csp_text.contains("ipc:"),
            "CSP must include connect-src with ipc:; got: {csp_text}"
        );
        assert!(
            !csp_text.contains("unsafe-eval"),
            "CSP must not allow unsafe-eval; got: {csp_text}"
        );
        assert!(
            !csp_text.contains("unsafe-inline"),
            "production CSP must not allow unsafe-inline; got: {csp_text}"
        );
        assert!(
            !csp_text.contains("localhost:1420") && !csp_text.contains("ws://localhost"),
            "production CSP must not include Vite HMR hosts; got: {csp_text}"
        );
        // Isolation iframe (U2/U5): frame-src must allow self + Tauri asset protocols.
        assert!(
            csp_text.contains("frame-src")
                && csp_text.contains("'self'")
                && csp_text.contains("customprotocol:")
                && csp_text.contains("asset:"),
            "production CSP must include frame-src 'self' customprotocol: asset:; got: {csp_text}"
        );
    }

    #[test]
    fn tauri_conf__dev_csp__may_relax_for_hmr_but_keeps_ipc() {
        let conf = include_str!("../tauri.conf.json");
        let value: serde_json::Value = match serde_json::from_str(conf) {
            Ok(v) => v,
            Err(e) => panic!("tauri.conf.json must be valid JSON: {e}"),
        };

        let Some(dev_csp) = value.pointer("/app/security/devCsp") else {
            // Optional field; if omitted, production CSP is used in dev too.
            return;
        };
        if dev_csp.is_null() {
            return;
        }
        let text = csp_to_text(dev_csp);
        assert!(
            text.contains("ipc:"),
            "devCsp must still allow ipc: for invoke; got: {text}"
        );
        assert!(
            !text.contains("unsafe-eval"),
            "devCsp must not allow unsafe-eval; got: {text}"
        );
    }

    #[test]
    fn tauri_conf__isolation_pattern__mandated() {
        let conf = include_str!("../tauri.conf.json");
        let value: serde_json::Value = match serde_json::from_str(conf) {
            Ok(v) => v,
            Err(e) => panic!("tauri.conf.json must be valid JSON: {e}"),
        };
        let use_pattern = value
            .pointer("/app/security/pattern/use")
            .and_then(|v| v.as_str());
        assert_eq!(
            use_pattern,
            Some("isolation"),
            "Isolation pattern is mandated for initial desktop release (U2)"
        );
        let dir = value
            .pointer("/app/security/pattern/options/dir")
            .and_then(|v| v.as_str());
        assert!(
            dir.is_some_and(|d| d.contains("isolation")),
            "pattern.options.dir must point at isolation app; got: {dir:?}"
        );
    }

    fn csp_to_text(csp: &serde_json::Value) -> String {
        match csp {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(map) => {
                let mut parts = Vec::new();
                for (k, v) in map {
                    let val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Array(arr) => arr
                            .iter()
                            .filter_map(|x| x.as_str().map(str::to_owned))
                            .collect::<Vec<_>>()
                            .join(" "),
                        other => other.to_string(),
                    };
                    parts.push(format!("{k} {val}"));
                }
                parts.join("; ")
            }
            other => panic!("unexpected CSP JSON shape: {other}"),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod capability_tests {
    /// SU18: never grant opener:default, allow-default-urls, or bare unscoped open-path.
    #[test]
    fn capabilities__opener__scoped_https_only_no_default_urls() {
        let raw = include_str!("../capabilities/default.json");
        let value: serde_json::Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => panic!("capabilities/default.json must be valid JSON: {e}"),
        };
        let permissions = value
            .get("permissions")
            .and_then(|p| p.as_array())
            .expect("permissions array");

        for entry in permissions {
            if let Some(s) = entry.as_str() {
                assert_ne!(
                    s, "opener:default",
                    "bare opener:default is forbidden (allows http/mailto/tel)"
                );
                assert_ne!(
                    s, "opener:allow-default-urls",
                    "opener:allow-default-urls is forbidden (allows http/mailto/tel)"
                );
                assert_ne!(
                    s, "opener:allow-open-path",
                    "bare string opener:allow-open-path is unscoped and forbidden; use object form"
                );
                assert!(
                    !s.contains("allow-default-urls"),
                    "permissions must not reference allow-default-urls; got {s}"
                );
            }
            if let Some(obj) = entry.as_object() {
                let id = obj.get("identifier").and_then(|v| v.as_str()).unwrap_or("");
                if id == "opener:allow-open-url" {
                    let allow = obj
                        .get("allow")
                        .and_then(|a| a.as_array())
                        .expect("opener:allow-open-url must have allow array");
                    assert!(!allow.is_empty(), "opener:allow-open-url must scope urls");
                    for item in allow {
                        let url = item.get("url").and_then(|u| u.as_str()).unwrap_or("");
                        assert!(
                            url.starts_with("https:"),
                            "url scope must be https-only; got {url}"
                        );
                    }
                }
                if id == "opener:allow-open-path" {
                    let allow = obj.get("allow").and_then(|a| a.as_array());
                    assert!(
                        allow.is_some_and(|a| !a.is_empty()),
                        "opener:allow-open-path must use object form with path allow list"
                    );
                }
            }
        }
    }

    #[test]
    fn package_json__must_not_include_js_opener_plugin() {
        let pkg = include_str!("../../package.json");
        assert!(
            !pkg.contains("@tauri-apps/plugin-opener"),
            "JS @tauri-apps/plugin-opener is forbidden (U20); opens go through Rust commands only"
        );
    }
}

/// D25: shared golden fixtures under `e2e/fixtures` must stay contract-shaped
/// (arrays present as `[]` not null; required keys present; error kinds honest).
#[cfg(test)]
#[allow(non_snake_case)]
mod fixture_sync_tests {
    fn parse_fixture(name: &str, raw: &str) -> serde_json::Value {
        match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => panic!("{name} must parse: {e}"),
        }
    }

    fn assert_array_key(value: &serde_json::Value, key: &str) {
        let field = value.get(key);
        assert!(field.is_some(), "{key} key must be present");
        assert!(
            !field.is_some_and(|v| v.is_null()),
            "{key} must not be null (E1)"
        );
        assert!(
            field.and_then(|v| v.as_array()).is_some(),
            "{key} must be an array"
        );
    }

    fn assert_string_key(value: &serde_json::Value, key: &str) {
        let field = value.get(key).and_then(|v| v.as_str());
        assert!(
            field.is_some_and(|s| !s.is_empty()),
            "{key} must be a non-empty string"
        );
    }

    #[test]
    fn fixture__review_empty_json__items_array_not_null() {
        let value = parse_fixture(
            "review-empty.json",
            include_str!("../../e2e/fixtures/review-empty.json"),
        );
        assert_eq!(value.get("api_version").and_then(|v| v.as_str()), Some("1"));
        assert_array_key(&value, "items");
        assert_eq!(
            value
                .get("items")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );
    }

    #[test]
    fn fixture__review_items_json__items_array_with_required_fields() {
        let value = parse_fixture(
            "review-items.json",
            include_str!("../../e2e/fixtures/review-items.json"),
        );
        assert_eq!(value.get("api_version").and_then(|v| v.as_str()), Some("1"));
        assert_array_key(&value, "items");
        let items = value
            .get("items")
            .and_then(|v| v.as_array())
            .expect("items array");
        assert!(
            !items.is_empty(),
            "review-items.json should include sample items"
        );
        let first = &items[0];
        assert_string_key(first, "id");
        assert_string_key(first, "subject");
        assert_string_key(first, "status");
    }

    #[test]
    fn fixture__offline_error_json__kind_and_message() {
        let value = parse_fixture(
            "offline-error.json",
            include_str!("../../e2e/fixtures/offline-error.json"),
        );
        assert_eq!(value.get("kind").and_then(|v| v.as_str()), Some("offline"));
        assert_string_key(&value, "message");
    }

    #[test]
    fn fixture__denied_error_json__kind_and_message() {
        let value = parse_fixture(
            "denied-error.json",
            include_str!("../../e2e/fixtures/denied-error.json"),
        );
        assert_eq!(value.get("kind").and_then(|v| v.as_str()), Some("denied"));
        assert_string_key(&value, "message");
    }

    #[test]
    fn fixture__source_https_json__required_keys_https_locator() {
        let value = parse_fixture(
            "source-https.json",
            include_str!("../../e2e/fixtures/source-https.json"),
        );
        assert_string_key(&value, "id");
        assert_string_key(&value, "kind");
        assert_string_key(&value, "display_name");
        let locator = value.get("locator").and_then(|v| v.as_str());
        assert!(
            locator.is_some_and(|s| s.starts_with("https://")),
            "source-https.json locator must be https"
        );
    }

    #[test]
    fn fixture__source_path_json__required_keys_path_locator() {
        let value = parse_fixture(
            "source-path.json",
            include_str!("../../e2e/fixtures/source-path.json"),
        );
        assert_string_key(&value, "id");
        assert_string_key(&value, "kind");
        assert_string_key(&value, "display_name");
        let locator = value.get("locator").and_then(|v| v.as_str());
        assert!(
            locator.is_some_and(|s| !s.is_empty() && !s.contains("://")),
            "source-path.json locator must be a non-URI path"
        );
    }

    #[test]
    fn fixture__source_missing_json__locator_null() {
        let value = parse_fixture(
            "source-missing.json",
            include_str!("../../e2e/fixtures/source-missing.json"),
        );
        assert_string_key(&value, "id");
        assert_string_key(&value, "kind");
        assert_string_key(&value, "display_name");
        assert!(
            value.get("locator").is_some_and(|v| v.is_null()),
            "source-missing.json locator must be null"
        );
    }

    #[test]
    fn fixture__source_http_json__http_locator_display_only() {
        let value = parse_fixture(
            "source-http.json",
            include_str!("../../e2e/fixtures/source-http.json"),
        );
        assert_string_key(&value, "id");
        let locator = value.get("locator").and_then(|v| v.as_str());
        assert!(
            locator.is_some_and(|s| s.starts_with("http://")),
            "source-http.json locator must be http (display-only, not openable)"
        );
    }

    #[test]
    fn fixture__source_no_locator_key_json__locator_key_absent() {
        let value = parse_fixture(
            "source-no-locator-key.json",
            include_str!("../../e2e/fixtures/source-no-locator-key.json"),
        );
        assert_string_key(&value, "id");
        assert_string_key(&value, "kind");
        assert!(
            !value.as_object().is_some_and(|m| m.contains_key("locator")),
            "source-no-locator-key.json must omit locator property (missing vs null)"
        );
    }

    #[test]
    fn fixture__wipe_dry_run_json__contract_shape() {
        let value = parse_fixture(
            "wipe-dry-run.json",
            include_str!("../../e2e/fixtures/wipe-dry-run.json"),
        );
        assert_eq!(value.get("api_version").and_then(|v| v.as_str()), Some("1"));
        assert_eq!(
            value.get("status").and_then(|v| v.as_str()),
            Some("dry_run")
        );
        assert_string_key(&value, "content_key_id");
        assert!(
            value.get("wrap_destroyed").and_then(|v| v.as_bool()) == Some(false),
            "dry-run wrap_destroyed must be false"
        );
        assert_array_key(&value, "warnings");
        assert!(value.get("purged").is_some_and(|v| v.is_object()));
        assert!(value.get("verify").is_some_and(|v| v.is_object()));
        assert!(value.get("validation").is_some_and(|v| v.is_object()));
    }

    #[test]
    fn fixture__wipe_execute_json__contract_shape() {
        let value = parse_fixture(
            "wipe-execute.json",
            include_str!("../../e2e/fixtures/wipe-execute.json"),
        );
        assert_eq!(value.get("api_version").and_then(|v| v.as_str()), Some("1"));
        assert_eq!(value.get("status").and_then(|v| v.as_str()), Some("wiped"));
        assert_string_key(&value, "content_key_id");
        assert!(
            value.get("wrap_destroyed").and_then(|v| v.as_bool()) == Some(true),
            "execute wrap_destroyed must be true"
        );
        assert_array_key(&value, "warnings");
        let warnings = value
            .get("warnings")
            .and_then(|v| v.as_array())
            .expect("warnings array");
        assert!(
            !warnings.is_empty(),
            "execute wipe must carry honesty warnings"
        );
        assert!(value.get("purged").is_some_and(|v| v.is_object()));
        assert!(value.get("verify").is_some_and(|v| v.is_object()));
        assert!(value.get("validation").is_some_and(|v| v.is_object()));
    }

    #[test]
    fn fixture__resolve_success_json__required_keys_and_warnings_array() {
        let value = parse_fixture(
            "resolve-success.json",
            include_str!("../../e2e/fixtures/resolve-success.json"),
        );
        assert_eq!(value.get("api_version").and_then(|v| v.as_str()), Some("1"));
        assert_string_key(&value, "id");
        assert_string_key(&value, "status");
        assert_array_key(&value, "warnings");
    }

    #[test]
    fn fixture__briefing_stale_json__freshness_stale_contract() {
        let value = parse_fixture(
            "briefing-stale.json",
            include_str!("../../e2e/fixtures/briefing-stale.json"),
        );
        assert_eq!(value.get("api_version").and_then(|v| v.as_str()), Some("1"));
        let packet = value.get("packet").expect("packet key must be present");
        assert!(packet.is_object(), "packet must be an object");
        assert_array_key(packet, "decisions");
        assert_array_key(packet, "conclusions");
        assert_array_key(packet, "constraints");
        assert_array_key(packet, "warnings");
        assert_array_key(packet, "evidence_handles");
        let freshness = packet
            .get("freshness")
            .expect("freshness key must be present");
        assert_eq!(
            freshness.get("worst_state").and_then(|v| v.as_str()),
            Some("stale")
        );
        assert!(
            freshness
                .get("stale_count")
                .and_then(|v| v.as_u64())
                .is_some_and(|n| n >= 1),
            "stale_count must be >= 1"
        );
    }
}
