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
