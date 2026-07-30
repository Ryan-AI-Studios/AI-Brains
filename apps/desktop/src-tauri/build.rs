fn main() {
    let attrs =
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[
            "ping",
            "get_daemon_connection_info",
            "project_briefing",
            "personal_briefing",
            "query_knowledge",
            "inspect_evidence",
            "inspect_source",
            "list_review_items",
            "resolve_review_item",
            "resolve_scope",
            "request_erasure",
            "wipe_content_envelope",
            "probe_health",
            "open_url",
            "reveal_path",
        ]));
    if let Err(e) = tauri_build::try_build(attrs) {
        eprintln!("tauri-build failed: {e}");
        std::process::exit(1);
    }
}
