//! `ai-brains erasure` — ticket (T160) + CE wipe (T165).
//!
//! Dual path:
//! - `erasure request` — ticket only; never claims content-envelope wipe (E3).
//! - `erasure wipe` — CE for envelope-backed content; daemon-required; dry-run default (E8/E9).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, emit_human, emit_json, ensure_command_id, expect_daemon_ok,
    fail_path, principal_id_wire, resolve_principal,
};
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::erasure::{
    ContentEnvelopeWipedResponse, ERASURE_TICKET_NO_WIPE_WARNING, ErasureAcceptedResponse,
    RequestErasureRequest, WipeContentEnvelopeRequest,
};
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};

/// Ticket-path CE honesty warning (E3). Keep available for tests/regressions.
pub const ERASURE_CE_WIPE_WARNING: &str = ERASURE_TICKET_NO_WIPE_WARNING;

pub struct RequestOptions {
    pub ids: Vec<String>,
    pub reason: Option<String>,
    pub scope: Option<String>,
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
        scope: options.scope.clone(),
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

    // E9: execute only with --confirm; otherwise dry-run (default-safe).
    let (dry_run, confirm) = if options.confirm {
        (false, true)
    } else {
        (true, false)
    };
    let _ = options.dry_run; // clap documents plan-only; confirm is the execute gate

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
            emit_human(&format!(
                "content-envelope wipe {} for key {} (wrap_destroyed={}, blobs={})",
                resp.status, resp.content_key_id, resp.wrap_destroyed, resp.blobs_considered
            ));
            if let Some(tid) = &resp.tombstone_id {
                emit_human(&format!("tombstone_id={tid}"));
            }
            emit_human(&format!(
                "verify.wrap_absent={} validation.fts_clear={} wal={}",
                resp.verify.wrap_absent, resp.validation.fts_clear, resp.validation.wal_checkpoint
            ));
            for w in &resp.warnings {
                emit_human(&format!("warning: {w}"));
            }
            Ok(())
        }
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
        assert!(joined.contains("purge") || joined.contains("nist"));
        assert!(joined.contains("backup") || joined.contains("offline"));
        assert!(!joined.contains("nist purge completed"));
    }
}
