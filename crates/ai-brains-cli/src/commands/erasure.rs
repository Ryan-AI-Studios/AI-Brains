//! `ai-brains erasure request` — daemon-only erasure ticket (T160).
//!
//! Never claims content-envelope wipe (P8 residual). Rejects `--local`.
//! Exit 5 when daemon is unavailable.

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, emit_human, emit_json, ensure_command_id, expect_daemon_ok,
    fail_path, principal_id_wire, resolve_principal,
};
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::erasure::{ErasureAcceptedResponse, RequestErasureRequest};
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};

/// CE wipe honesty warning (must appear in JSON warnings; never claim wipe).
pub const ERASURE_CE_WIPE_WARNING: &str =
    "content-envelope wipe not performed (P8 residual); ticket accepted only";

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
            emit_erasure(format, &wire)
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

fn emit_erasure(
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
            emit_human("note: this does not perform content-envelope wipe (P8 residual)");
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
}
