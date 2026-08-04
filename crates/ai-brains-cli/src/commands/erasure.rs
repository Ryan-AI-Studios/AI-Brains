//! `ai-brains erasure` — ticket (T160) + CE wipe (T165).
//!
//! Dual path:
//! - `erasure request` — ticket only; never claims content-envelope wipe (E3).
//! - `erasure wipe` — CE for envelope-backed content; daemon-required; dry-run default (E8/E9).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, emit_human, emit_json, ensure_command_id, expect_daemon_ok,
    fail_api, fail_path, principal_id_wire, resolve_principal,
};
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::erasure::{
    ContentEnvelopeWipedResponse, ERASURE_TICKET_NO_WIPE_WARNING, ErasureAcceptedResponse,
    RequestErasureRequest, WipeContentEnvelopeRequest,
};
use ai_brains_contracts::response::ApiError;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};

/// Ticket-path CE honesty warning (E3). Keep available for tests/regressions.
pub const ERASURE_CE_WIPE_WARNING: &str = ERASURE_TICKET_NO_WIPE_WARNING;

pub struct RequestOptions {
    pub ids: Vec<String>,
    pub reason: Option<String>,
    /// Required by clap (T201 F4); wire DTO still Option for non-CLI callers.
    pub scope: String,
    pub format: Option<String>,
    pub principal_id: Option<String>,
    pub command_id: Option<String>,
    pub local: bool,
    pub daemon: bool,
    /// Accepted for clap symmetry; erasure always requires the daemon.
    #[allow(dead_code)]
    pub require_daemon: bool,
}

pub struct WipeOptions {
    pub content_key_id: String,
    pub scope: String,
    pub reason: Option<String>,
    pub format: Option<String>,
    pub principal_id: Option<String>,
    pub command_id: Option<String>,
    /// When true, force dry-run (default path). When false with `--confirm`, execute.
    pub dry_run: bool,
    pub confirm: bool,
    pub local: bool,
    pub daemon: bool,
    #[allow(dead_code)]
    pub require_daemon: bool,
}

/// `ai-brains erasure request --id <ids...> [--reason] [--scope]`
pub async fn run_request(options: RequestOptions) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());
    let flags = governed_common::PathFlags {
        local: options.local,
        daemon: options.daemon,
        require_daemon: true, // always required for erasure
    };
    let path = match governed_common::choose_erasure_path(flags).await {
        Ok(p) => p,
        Err(e) => return fail_path(format, e),
    };

    let PathDecision::Daemon = path else {
        return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
    };

    let command_id = ensure_command_id(options.command_id.as_deref());
    let principal = resolve_principal(options.principal_id.as_deref());
    let client = DaemonClient::new();
    let req = DaemonRequest::RequestErasure(RequestErasureRequest {
        api_version: ai_brains_contracts::erasure::API_VERSION.to_string(),
        principal_id: principal_id_wire(&principal),
        ids: options.ids.clone(),
        reason: options.reason.clone(),
        // Wire DTO keeps Option for HTTP/IPC; CLI always sends scope after F4.
        scope: Some(options.scope.clone()),
        command_id: Some(command_id.clone()),
    });

    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(e) => {
            return fail_path(format, governed_common::classify_daemon_mutation_error(&e));
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    match resp {
        DaemonResponse::ErasureAccepted(mut wire) => {
            ensure_wipe_warning(&mut wire);
            if !wire.warnings.iter().any(|w| w.contains("command_id=")) {
                wire.warnings.push(format!("command_id={command_id}"));
            }
            emit_erasure_ticket(format, &wire)
        }
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}

/// `ai-brains erasure wipe --content-key-id --scope [--dry-run|--confirm]`
pub async fn run_wipe(options: WipeOptions) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());

    // E9: validate dry-run/confirm AND semantics before daemon probe so dual-flag
    // conflicts always surface as INVALID_PAYLOAD (not DAEMON_UNAVAILABLE).
    let (dry_run, confirm) = match resolve_wipe_execute_flags(options.dry_run, options.confirm) {
        Ok(pair) => pair,
        Err(msg) => {
            return fail_api(format, ApiError::new("INVALID_PAYLOAD", msg));
        }
    };

    let flags = governed_common::PathFlags {
        local: options.local,
        daemon: options.daemon,
        require_daemon: true,
    };
    let path = match governed_common::choose_erasure_path(flags).await {
        Ok(p) => p,
        Err(e) => return fail_path(format, e),
    };

    let PathDecision::Daemon = path else {
        return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
    };

    let command_id = ensure_command_id(options.command_id.as_deref());
    let principal = resolve_principal(options.principal_id.as_deref());
    let client = DaemonClient::new();
    let req = DaemonRequest::WipeContentEnvelope(WipeContentEnvelopeRequest {
        api_version: ai_brains_contracts::erasure::API_VERSION.to_string(),
        principal_id: principal_id_wire(&principal),
        content_key_id: options.content_key_id.clone(),
        scope: options.scope.clone(),
        reason: options.reason.clone(),
        command_id: Some(command_id.clone()),
        dry_run,
        confirm,
    });

    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(e) => {
            return fail_path(format, governed_common::classify_daemon_mutation_error(&e));
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    match resp {
        DaemonResponse::ContentEnvelopeWiped(mut wire) => {
            ensure_wipe_honesty_warnings(&mut wire);
            if !wire.warnings.iter().any(|w| w.contains("command_id=")) {
                wire.warnings.push(format!("command_id={command_id}"));
            }
            emit_wipe(format, &wire)
        }
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}

/// Map CLI `--dry-run` / `--confirm` flags to wire `(dry_run, confirm)`.
///
/// Contracts E9 AND semantics:
/// - default / `--dry-run` alone → plan only (`dry_run=true`, `confirm=false`)
/// - `--confirm` alone → execute (`dry_run=false`, `confirm=true`)
/// - both set → refuse (INVALID_PAYLOAD); never destroy under dry-run
pub fn resolve_wipe_execute_flags(
    dry_run_flag: bool,
    confirm_flag: bool,
) -> Result<(bool, bool), String> {
    if dry_run_flag && confirm_flag {
        return Err(
            "cannot combine --dry-run and --confirm; dry-run is plan-only (omit --confirm to plan, or pass --confirm alone to execute)"
                .into(),
        );
    }
    if confirm_flag {
        Ok((false, true))
    } else {
        // Default when neither flag is set, and explicit --dry-run.
        Ok((true, false))
    }
}

fn ensure_wipe_warning(resp: &mut ErasureAcceptedResponse) {
    let has = resp.warnings.iter().any(|w| {
        let lower = w.to_ascii_lowercase();
        lower.contains("wipe") || lower.contains("content-envelope")
    });
    if !has {
        resp.warnings.push(ERASURE_CE_WIPE_WARNING.to_string());
    }
}

fn ensure_wipe_honesty_warnings(resp: &mut ContentEnvelopeWipedResponse) {
    for w in ContentEnvelopeWipedResponse::honesty_warnings() {
        if !resp.warnings.iter().any(|existing| existing == &w) {
            resp.warnings.push(w);
        }
    }
}

fn emit_erasure_ticket(
    format: OutputFormat,
    resp: &ErasureAcceptedResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => emit_json(resp),
        OutputFormat::Human | OutputFormat::Markdown => {
            emit_human(&format!(
                "erasure ticket {} ({})",
                resp.request_id, resp.status
            ));
            for w in &resp.warnings {
                emit_human(&format!("warning: {w}"));
            }
            emit_human(
                "note: this does not perform content-envelope wipe; use `erasure wipe` for CE",
            );
            Ok(())
        }
    }
}

fn emit_wipe(
    format: OutputFormat,
    resp: &ContentEnvelopeWipedResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => emit_json(resp),
        OutputFormat::Human | OutputFormat::Markdown => {
            // Human path: never print technical honesty tokens "NIST" / "purged"
            // (JSON warnings may keep machine-facing constants).
            for line in format_wipe_human(resp).lines() {
                emit_human(line);
            }
            Ok(())
        }
    }
}

/// Human/Markdown wipe success text (E10 display sanitization).
///
/// Must never contain case-insensitive `"nist"` or `"purged"` as product claim
/// words. JSON warnings retain technical honesty constants for machine consumers.
pub fn format_wipe_human(resp: &ContentEnvelopeWipedResponse) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "content-envelope wipe {} for key {} (wrap_destroyed={}, blobs={})",
        resp.status, resp.content_key_id, resp.wrap_destroyed, resp.blobs_considered
    ));
    if let Some(tid) = &resp.tombstone_id {
        lines.push(format!("tombstone_id={tid}"));
    }
    lines.push(format!(
        "verify.wrap_absent={} validation.fts_clear={} wal={}",
        resp.verify.wrap_absent, resp.validation.fts_clear, resp.validation.wal_checkpoint
    ));
    // Fixed human honesty block (no NIST / purged tokens).
    lines.push("honesty: not media sanitization; WAL truncate is not secure media wipe".into());
    lines.push(
        "honesty: pre-erase backups, exports, and offline copies remain decryptable if restored"
            .into(),
    );
    lines.push("honesty: erasure ticket and soft forget are not cryptographic erasure".into());
    lines.push("honesty: cryptographic erasure applies only to envelope-backed content".into());
    lines.push("honesty: vault encryption lock is not per-item cryptographic erasure".into());
    // Map remaining technical warnings to human-safe text (skip honesty constants).
    for w in &resp.warnings {
        if let Some(human) = map_warning_for_human(w) {
            lines.push(format!("warning: {human}"));
        }
    }
    lines.join("\n")
}

/// Map a machine warning to human-safe display text, or `None` if already covered
/// by the fixed honesty block / should not be shown as raw technical text.
fn map_warning_for_human(warning: &str) -> Option<String> {
    let lower = warning.to_ascii_lowercase();
    // Honesty constants are replaced by the fixed block above.
    if lower.contains("nist")
        || lower.contains("physical media")
        || lower.contains("pre-erase backup")
        || lower.contains("ticket and soft forget")
        || lower.contains("envelope-backed content")
        || lower.contains("sqlcipher")
        || lower.contains("vault lock is not per-item")
        || (lower.contains("content_key_store")
            && lower.contains("cryptographic erasure applies only"))
    {
        return None;
    }
    if lower.contains("dependents_skipped") {
        return Some("dependents not marked stale (no registered source link)".into());
    }
    if lower.contains("pending_passive") || lower.contains("wal_checkpoint") {
        return Some("WAL checkpoint deferred; will finish via routine passive checkpoint".into());
    }
    if lower.starts_with("command_id=") {
        return Some(warning.to_string());
    }
    // Strip forbidden tokens from any other warning before display.
    let sanitized = warning
        .replace("NIST", "")
        .replace("nist", "")
        .replace("purged", "cleared")
        .replace("Purged", "Cleared");
    let trimmed = sanitized.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Unit-level response mapping for tests (no daemon).
#[cfg(test)]
pub fn map_erasure_response_for_test(mut resp: ErasureAcceptedResponse) -> ErasureAcceptedResponse {
    ensure_wipe_warning(&mut resp);
    resp
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_contracts::erasure::{WipePurgedCounts, WipeValidation, WipeVerify};

    #[test]
    fn cli_erasure_request__json__warnings_no_ce_wipe_claim() {
        let resp =
            map_erasure_response_for_test(ErasureAcceptedResponse::new("ticket-1", "accepted"));
        assert!(!resp.warnings.is_empty());
        let joined = resp.warnings.join(" ");
        assert!(
            joined.to_ascii_lowercase().contains("wipe")
                || joined.to_ascii_lowercase().contains("content-envelope"),
            "warnings must mention wipe residual: {joined}"
        );
        // Must not claim wipe completed
        assert!(
            !joined.to_ascii_lowercase().contains("wipe completed")
                && !joined.to_ascii_lowercase().contains("wiped"),
            "must not claim wipe completed: {joined}"
        );
    }

    #[test]
    fn cli_erasure_wipe__honesty_warnings_always_present() {
        let mut resp = ContentEnvelopeWipedResponse {
            api_version: ai_brains_contracts::erasure::API_VERSION.to_string(),
            status: "wiped".into(),
            content_key_id: "k".into(),
            tombstone_id: Some("t".into()),
            wrap_destroyed: true,
            blobs_considered: 1,
            purged: WipePurgedCounts::default(),
            dependents_marked: 0,
            warnings: vec![],
            verify: WipeVerify { wrap_absent: true },
            validation: WipeValidation {
                fts_clear: true,
                store_open_refused: true,
                wal_checkpoint: "truncated".into(),
            },
        };
        ensure_wipe_honesty_warnings(&mut resp);
        let joined = resp.warnings.join(" ").to_ascii_lowercase();
        // JSON/machine path may keep technical honesty constants.
        assert!(joined.contains("purge") || joined.contains("nist"));
        assert!(joined.contains("backup") || joined.contains("offline"));
        assert!(!joined.contains("nist purge completed"));
    }

    #[test]
    fn cli_erasure_wipe__human_output__no_nist_or_purged() {
        let mut resp = ContentEnvelopeWipedResponse {
            api_version: ai_brains_contracts::erasure::API_VERSION.to_string(),
            status: "wiped".into(),
            content_key_id: "00000000-0000-0000-0000-0000000000aa".into(),
            tombstone_id: Some("00000000-0000-0000-0000-0000000000bb".into()),
            wrap_destroyed: true,
            blobs_considered: 2,
            purged: WipePurgedCounts {
                fts_rows: 1,
                embeddings: 1,
                projection_rows: 1,
            },
            dependents_marked: 0,
            warnings: ContentEnvelopeWipedResponse::honesty_warnings(),
            verify: WipeVerify { wrap_absent: true },
            validation: WipeValidation {
                fts_clear: true,
                store_open_refused: true,
                wal_checkpoint: "pending_passive".into(),
            },
        };
        resp.warnings
            .push(ai_brains_contracts::erasure::WIPE_WARNING_DEPENDENTS_SKIPPED.to_string());
        resp.warnings
            .push(ai_brains_contracts::erasure::WIPE_WARNING_WAL_PENDING_PASSIVE.to_string());
        resp.warnings.push("command_id=wipe-cmd-1".into());

        let human = format_wipe_human(&resp);
        let lower = human.to_ascii_lowercase();
        assert!(
            !lower.contains("nist"),
            "human wipe text must not say NIST: {human}"
        );
        assert!(
            !lower.contains("purged"),
            "human wipe text must not say purged: {human}"
        );
        assert!(
            lower.contains("not media sanitization") || lower.contains("media wipe"),
            "must still state non-sanitization honesty: {human}"
        );
        assert!(
            lower.contains("pre-erase") || lower.contains("backup"),
            "must still state backup residual honesty: {human}"
        );
        assert!(
            lower.contains("vault encryption") || lower.contains("per-item"),
            "must state vault lock is not per-item CE (without NIST): {human}"
        );
        assert!(
            lower.contains("wrap_destroyed=true"),
            "must report wrap destroy: {human}"
        );
    }

    #[test]
    fn cli_erasure_wipe__default_flags__plan_only() {
        let (dry_run, confirm) = resolve_wipe_execute_flags(false, false).unwrap();
        assert!(dry_run);
        assert!(!confirm);
    }

    #[test]
    fn cli_erasure_wipe__confirm_only__execute_mapping() {
        let (dry_run, confirm) = resolve_wipe_execute_flags(false, true).unwrap();
        assert!(!dry_run, "confirm-only must set dry_run=false for execute");
        assert!(confirm);
    }

    #[test]
    fn cli_erasure_wipe__dry_run_only__plan_only() {
        let (dry_run, confirm) = resolve_wipe_execute_flags(true, false).unwrap();
        assert!(dry_run);
        assert!(!confirm);
    }

    #[test]
    fn cli_erasure_wipe__dry_run_and_confirm__refused() {
        let err = resolve_wipe_execute_flags(true, true).unwrap_err();
        assert!(
            err.contains("--dry-run") && err.contains("--confirm"),
            "conflict message must name both flags: {err}"
        );
        assert!(
            err.to_ascii_lowercase().contains("plan-only")
                || err.to_ascii_lowercase().contains("cannot combine"),
            "must refuse dual flags clearly: {err}"
        );
    }
}
