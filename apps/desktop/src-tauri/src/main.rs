// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // S21: clear dialog + clean exit if WebView2 Evergreen Runtime is missing.
    ai_brains_desktop_lib::ensure_webview2_or_exit();
    ai_brains_desktop_lib::run();
}
