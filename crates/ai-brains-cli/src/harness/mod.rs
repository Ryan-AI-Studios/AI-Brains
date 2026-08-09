//! Harness detect, wiring, prefs, AGY map, and install (T235).
//!
//! Pure fs/env probes — no models, embeddings, or graph (capture independence).

pub mod agy_map;
pub mod detect;
pub mod fs_util;
pub mod install;
pub mod prefs;
pub mod wiring;

// Re-exports used by commands / doctor / preflight (production paths).
pub use detect::{HARNESS_ORDER, HarnessId, parse_harness_id, resolve_home};
pub use install::{
    InstallOutcome, UninstallOutcome, f34_map_contract_summary, install_agy, install_grok,
    install_opencode, install_pending, install_pending_summary, uninstall_agy, uninstall_grok,
    uninstall_opencode, uninstall_pending,
};
pub use prefs::{load_prefs, save_prefs};
pub use wiring::{HarnessStatus, WiringStatus, collect_status_report, wiring_status_label};

/// Pure F34 map — linked for production so the Stop→payload contract is not test-only.
///
/// The managed PowerShell wrapper reimplements the same rules; this function is the
/// unit-tested SOOT (AC16). Callers may use it for future non-PS backends.
#[inline]
pub fn map_agy_stop_to_hook_payload(
    stop: &serde_json::Value,
) -> Result<agy_map::AgyHookPayload, agy_map::MapSkip> {
    agy_map::map_agy_stop_to_hook_payload(stop)
}
