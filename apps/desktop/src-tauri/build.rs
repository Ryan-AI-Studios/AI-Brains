fn main() {
    let attrs = tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&["ping", "get_daemon_connection_info"]),
    );
    if let Err(e) = tauri_build::try_build(attrs) {
        eprintln!("tauri-build failed: {e}");
        std::process::exit(1);
    }
}
