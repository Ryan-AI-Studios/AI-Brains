//! AI-Brains desktop host library (Tauri commands + WebView2 diagnostic).

mod commands;
mod webview2;

pub use commands::{
    DaemonConnectionInfo, InvokeApiError, PingResponse, daemon_connection_info_payload,
    get_daemon_connection_info, ping, ping_payload,
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
    /// CSP regression (SC4 / S7 / M24): production `csp` must stay T171-strict.
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
