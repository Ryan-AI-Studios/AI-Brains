//! Shared helpers for governed CLI commands (T160).
//!
//! Thin adapter utilities only — no domain logic. Path policy, emit, exit codes,
//! principal resolution, and ResolvedScope → wire DTO mapping live here.

use crate::commands::briefing::cli_principal;
use crate::daemon_client::{DaemonClient, DaemonClientError};
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::scopes::{ScopeEvidenceDto, ScopeResolvedResponse};
use ai_brains_control_plane::{
    ControlPlaneError, ResolvedScope, ScopeConfidence, is_authoritative, make_principal,
    scope_identity_key,
};
use ai_brains_core::ids::PrincipalId;
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_daemon_api::DaemonResponse;
use serde::Serialize;
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Exit codes (frozen T160)
// ---------------------------------------------------------------------------

// Frozen exit-code surface (T160). Some codes are reserved for clap/scripts.
#[allow(dead_code)]
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INTERNAL: i32 = 1;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_POLICY_DENIED: i32 = 3;
pub const EXIT_NOT_FOUND: i32 = 4;
pub const EXIT_DAEMON_UNAVAILABLE: i32 = 5;
pub const EXIT_INVALID_PAYLOAD: i32 = 6;
/// Trust hard-gate failure (T169 evaluate governed). Distinct from EXIT_INTERNAL (tool broke).
pub const EXIT_HARD_GATE_FAILED: i32 = 7;

/// Structured code for build-feature unavailable (T198 graph stub; T200 install honesty).
pub const FEATURE_UNAVAILABLE: &str = "FEATURE_UNAVAILABLE";

/// Exit code for feature-unavailable paths (clap-style usage = [`EXIT_USAGE`] = 2).
pub fn exit_code_feature_unavailable() -> i32 {
    EXIT_USAGE
}

/// Map a structured API error code to a CLI exit code.
pub fn exit_code_for_api_error(err: &ApiError) -> i32 {
    match err.code.as_str() {
        "POLICY_DENIED" => EXIT_POLICY_DENIED,
        "NOT_FOUND" => EXIT_NOT_FOUND,
        "DAEMON_UNAVAILABLE" => EXIT_DAEMON_UNAVAILABLE,
        "INVALID_PAYLOAD" | "NOT_ENVELOPE_BACKED" => EXIT_INVALID_PAYLOAD,
        "APPROVAL_REQUIRED" => EXIT_POLICY_DENIED,
        // Path / live-vault refusals (migrate, shadow reuse) → EXIT_INTERNAL (1).
        "PATH_REFUSED" => EXIT_INTERNAL,
        // Evaluate trust gates failed (harness worked; product blocked).
        "HARD_GATE_FAILED" => EXIT_HARD_GATE_FAILED,
        // Optional feature not in this binary (T198/T200).
        "FEATURE_UNAVAILABLE" => EXIT_USAGE,
        _ => EXIT_INTERNAL,
    }
}

/// Map control-plane errors to exit codes + ApiError.
pub fn api_error_from_cp(err: &ControlPlaneError) -> ApiError {
    let (code, message) = match err {
        ControlPlaneError::PolicyDenied(m) => ("POLICY_DENIED", m.clone()),
        ControlPlaneError::NotFound(m) => ("NOT_FOUND", m.clone()),
        ControlPlaneError::InvalidPayload(m) => ("INVALID_PAYLOAD", m.clone()),
        ControlPlaneError::ApprovalRequired(m) => ("APPROVAL_REQUIRED", m.clone()),
        ControlPlaneError::InvalidTransition(m) => ("INVALID_TRANSITION", m.clone()),
        ControlPlaneError::NotEnvelopeBacked(m) => ("NOT_ENVELOPE_BACKED", m.clone()),
        other => ("INTERNAL", other.to_string()),
    };
    ApiError::new(code, message)
}

#[allow(dead_code)]
pub fn exit_code_for_cp(err: &ControlPlaneError) -> i32 {
    exit_code_for_api_error(&api_error_from_cp(err))
}

// ---------------------------------------------------------------------------
// Governed CLI error (main maps exit_code)
// ---------------------------------------------------------------------------

/// Error type for governed commands that already emitted stdout/stderr.
#[derive(Debug)]
pub struct GovernedCliError {
    pub exit_code: i32,
    pub message: String,
    pub emitted: bool,
}

impl GovernedCliError {
    pub fn emitted(exit_code: i32, message: impl Into<String>) -> Self {
        Self {
            exit_code,
            message: message.into(),
            emitted: true,
        }
    }
}

impl fmt::Display for GovernedCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GovernedCliError {}

pub type GovernedResult = Result<(), Box<dyn std::error::Error>>;

// ---------------------------------------------------------------------------
// Emit helpers
// ---------------------------------------------------------------------------

/// Emit machine-clean JSON on stdout only.
pub fn emit_json(value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    let s = serde_json::to_string_pretty(value)?;
    println!("{s}");
    Ok(())
}

/// Emit human text on stdout.
pub fn emit_human(text: &str) {
    println!("{text}");
}

/// Emit structured error for json mode (stdout) or human mode (stderr).
pub fn emit_error(format: OutputFormat, err: &ApiError) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => {
            // Scripts parse one stream: JSON error envelope on stdout.
            let s = serde_json::to_string_pretty(err)?;
            println!("{s}");
        }
        OutputFormat::Human | OutputFormat::Markdown => {
            eprintln!("{}: {}", err.code, err.message);
        }
    }
    Ok(())
}

/// Emit error and return a [`GovernedCliError`] with the mapped exit code.
pub fn fail_api(format: OutputFormat, err: ApiError) -> GovernedResult {
    let code = exit_code_for_api_error(&err);
    let msg = format!("{}: {}", err.code, err.message);
    let _ = emit_error(format, &err);
    Err(Box::new(GovernedCliError::emitted(code, msg)))
}

pub fn fail_cp(format: OutputFormat, err: ControlPlaneError) -> GovernedResult {
    fail_api(format, api_error_from_cp(&err))
}

pub fn fail_daemon_response_error(format: OutputFormat, err: ApiError) -> GovernedResult {
    fail_api(format, err)
}

// ---------------------------------------------------------------------------
// Output format
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Human,
    Markdown,
}

impl OutputFormat {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
            Some("human") | Some("text") | Some("pretty") => Self::Human,
            Some("markdown") | Some("md") => Self::Markdown,
            _ => Self::Json, // default for new governed commands
        }
    }
}

// ---------------------------------------------------------------------------
// Path policy
// ---------------------------------------------------------------------------

/// Flags controlling daemon vs local control-plane path selection.
#[derive(Debug, Clone, Copy, Default)]
pub struct PathFlags {
    /// Force in-process control-plane (propose/review only).
    pub local: bool,
    /// Prefer daemon when available.
    pub daemon: bool,
    /// Require daemon; fail with exit 5 if unavailable.
    pub require_daemon: bool,
}

/// Outcome of path selection before any mutation send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathDecision {
    Local { note: Option<String> },
    Daemon,
}

/// Errors from path policy (pre-send classification only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathPolicyError {
    DaemonUnavailable,
    LocalForbidden { reason: String },
    AmbiguousDaemon { message: String },
}

impl fmt::Display for PathPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DaemonUnavailable => write!(f, "daemon unavailable"),
            Self::LocalForbidden { reason } => write!(f, "local path forbidden: {reason}"),
            Self::AmbiguousDaemon { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for PathPolicyError {}

impl PathPolicyError {
    pub fn to_api_error(&self) -> ApiError {
        match self {
            Self::DaemonUnavailable => ApiError::new(
                "DAEMON_UNAVAILABLE",
                "daemon is not available; required for this command",
            ),
            Self::LocalForbidden { reason } => ApiError::new("INVALID_PAYLOAD", reason.clone()),
            Self::AmbiguousDaemon { message } => ApiError::new("INTERNAL", message.clone()),
        }
    }
}

/// Choose path for **read** commands: local default; `--daemon` / `--require-daemon` force pipe.
pub async fn choose_read_path(flags: PathFlags) -> Result<PathDecision, PathPolicyError> {
    if flags.local && (flags.daemon || flags.require_daemon) {
        return Err(PathPolicyError::LocalForbidden {
            reason: "cannot combine --local with --daemon/--require-daemon".into(),
        });
    }
    if flags.local {
        return Ok(PathDecision::Local { note: None });
    }
    if flags.daemon || flags.require_daemon {
        let client = DaemonClient::new();
        if client.probe(Duration::from_millis(50)).await {
            return Ok(PathDecision::Daemon);
        }
        return Err(PathPolicyError::DaemonUnavailable);
    }
    // Default: local CP
    Ok(PathDecision::Local { note: None })
}

/// Choose path for **propose/review** mutations.
///
/// - `--local` → local always
/// - daemon preferred when probe succeeds (or `--daemon` / `--require-daemon`)
/// - pre-send daemon down → local OK with stderr note unless require-daemon
pub async fn choose_mutation_path(flags: PathFlags) -> Result<PathDecision, PathPolicyError> {
    if flags.local && (flags.daemon || flags.require_daemon) {
        return Err(PathPolicyError::LocalForbidden {
            reason: "cannot combine --local with --daemon/--require-daemon".into(),
        });
    }
    if flags.local {
        return Ok(PathDecision::Local { note: None });
    }

    let client = DaemonClient::new();
    let up = client.probe(Duration::from_millis(50)).await;
    if up {
        return Ok(PathDecision::Daemon);
    }
    if flags.require_daemon || flags.daemon {
        return Err(PathPolicyError::DaemonUnavailable);
    }
    Ok(PathDecision::Local {
        note: Some(
            "daemon not reachable before send; using local control-plane (stderr note)".into(),
        ),
    })
}

/// Erasure is **always** daemon-required; reject `--local`.
pub async fn choose_erasure_path(flags: PathFlags) -> Result<PathDecision, PathPolicyError> {
    if flags.local {
        return Err(PathPolicyError::LocalForbidden {
            reason: "erasure is daemon-only; --local is not supported".into(),
        });
    }
    let client = DaemonClient::new();
    if client.probe(Duration::from_millis(50)).await {
        return Ok(PathDecision::Daemon);
    }
    Err(PathPolicyError::DaemonUnavailable)
}

/// Classify a post-attempt daemon error for mutations.
///
/// After send + ambiguous failure: **no** local fallback.
pub fn classify_daemon_mutation_error(err: &DaemonClientError) -> PathPolicyError {
    if err.is_ambiguous() {
        PathPolicyError::AmbiguousDaemon {
            message: format!("outcome unknown; retry same --command-id on daemon ({err})"),
        }
    } else if err.is_pre_send_unavailable() {
        PathPolicyError::DaemonUnavailable
    } else {
        PathPolicyError::AmbiguousDaemon {
            message: format!("daemon error after interaction: {err}"),
        }
    }
}

/// Handle path policy error → emit + GovernedCliError.
pub fn fail_path(format: OutputFormat, err: PathPolicyError) -> GovernedResult {
    fail_api(format, err.to_api_error())
}

// ---------------------------------------------------------------------------
// Principal / command_id
// ---------------------------------------------------------------------------

/// Resolve CLI principal: optional `--principal-id`, else env / system principal.
pub fn resolve_principal(principal_id: Option<&str>) -> Principal {
    if let Some(raw) = principal_id {
        let trimmed = raw.trim();
        if let Ok(u) = Uuid::parse_str(trimmed) {
            return make_principal(PrincipalKind::Human, PrincipalId::from_uuid(u), "cli-human");
        }
    }
    cli_principal()
}

/// Wire principal id string for daemon requests.
///
/// Always sends the resolved principal id (including the default System principal)
/// so the daemon cannot diverge via `AI_BRAINS_DAEMON_PRINCIPAL_ID` when the CLI
/// already selected an identity (T160 Codex P1 / principal parity).
pub fn principal_id_wire(principal: &Principal) -> Option<String> {
    Some(principal.id.to_string())
}

/// Auto-generate a command_id UUID when the user omitted one.
pub fn ensure_command_id(command_id: Option<&str>) -> String {
    match command_id.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => s.to_string(),
        None => Uuid::new_v4().to_string(),
    }
}

// ---------------------------------------------------------------------------
// Scope mapping
// ---------------------------------------------------------------------------

/// Map control-plane [`ResolvedScope`] → wire [`ScopeResolvedResponse`] (parity with daemon).
pub fn map_resolved_scope(resolved: &ResolvedScope) -> ScopeResolvedResponse {
    let confidence = confidence_name(resolved.confidence);
    let authoritative = is_authoritative(resolved)
        && !matches!(
            resolved.confidence,
            ScopeConfidence::Low | ScopeConfidence::Ambiguous
        );
    ScopeResolvedResponse {
        api_version: ai_brains_contracts::scopes::API_VERSION.to_string(),
        scope: scope_identity_key(&resolved.scope),
        confidence: confidence.to_string(),
        authoritative,
        evidence: resolved
            .evidence
            .iter()
            .map(|e| ScopeEvidenceDto {
                signal: e.signal.clone(),
                detail: e.detail.clone(),
            })
            .collect(),
        warnings: resolved.warnings.clone(),
        alternatives: resolved
            .alternatives
            .iter()
            .map(scope_identity_key)
            .collect(),
    }
}

fn confidence_name(c: ScopeConfidence) -> &'static str {
    match c {
        ScopeConfidence::High => "High",
        ScopeConfidence::Medium => "Medium",
        ScopeConfidence::Low => "Low",
        ScopeConfidence::Ambiguous => "Ambiguous",
    }
}

// ---------------------------------------------------------------------------
// Daemon response helpers
// ---------------------------------------------------------------------------

/// If response is Error, fail with mapped exit code; else Ok(response).
pub fn expect_daemon_ok(
    format: OutputFormat,
    resp: DaemonResponse,
) -> Result<DaemonResponse, Box<dyn std::error::Error>> {
    match resp {
        DaemonResponse::Error(err) => {
            fail_daemon_response_error(format, err).map(|_| unreachable!())
        }
        other => Ok(other),
    }
}

/// Print human-friendly scope resolution (always surfaces authoritative / warnings / alternatives).
pub fn emit_scope_human(resp: &ScopeResolvedResponse) {
    let auth = if resp.authoritative {
        "authoritative"
    } else {
        "NOT authoritative"
    };
    println!("scope: {}", resp.scope);
    println!("confidence: {} ({auth})", resp.confidence);
    if !resp.warnings.is_empty() {
        println!("warnings:");
        for w in &resp.warnings {
            println!("  - {w}");
        }
    }
    if !resp.alternatives.is_empty() {
        println!("alternatives:");
        for a in &resp.alternatives {
            println!("  - {a}");
        }
    }
    if !resp.evidence.is_empty() {
        println!("evidence:");
        for e in &resp.evidence {
            println!("  - {}: {}", e.signal, e.detail);
        }
    }
    if !resp.authoritative {
        println!("note: non-authoritative resolution — do not treat as full grant (scope #20)");
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_for_api_error__policy_denied__3() {
        let err = ApiError::new("POLICY_DENIED", "no grant");
        assert_eq!(exit_code_for_api_error(&err), EXIT_POLICY_DENIED);
    }

    #[test]
    fn exit_code_for_api_error__not_found__4() {
        let err = ApiError::new("NOT_FOUND", "missing");
        assert_eq!(exit_code_for_api_error(&err), EXIT_NOT_FOUND);
    }

    #[test]
    fn exit_code_for_api_error__invalid_payload__6() {
        let err = ApiError::new("INVALID_PAYLOAD", "bad");
        assert_eq!(exit_code_for_api_error(&err), EXIT_INVALID_PAYLOAD);
    }

    #[test]
    fn exit_code_feature_unavailable__returns_exit_usage_2() {
        assert_eq!(exit_code_feature_unavailable(), EXIT_USAGE);
        assert_eq!(exit_code_feature_unavailable(), 2);
    }

    #[test]
    fn exit_code_for_api_error__feature_unavailable__2() {
        let err = ApiError::new(FEATURE_UNAVAILABLE, "graph not in this build");
        assert_eq!(exit_code_for_api_error(&err), EXIT_USAGE);
        assert_eq!(exit_code_for_api_error(&err), 2);
    }

    #[test]
    fn cli_mutation_ambiguous_daemon__no_silent_local_fallback() {
        let err = DaemonClientError::Timeout { request_sent: true };
        let classified = classify_daemon_mutation_error(&err);
        match &classified {
            PathPolicyError::AmbiguousDaemon { message } => {
                assert!(
                    message.contains("outcome unknown") || message.contains("command-id"),
                    "message={message}"
                );
            }
            other => panic!("expected AmbiguousDaemon, got {other:?}"),
        }
        // Must not map to a soft local-ok path.
        assert_eq!(
            exit_code_for_api_error(&classified.to_api_error()),
            EXIT_INTERNAL
        );
    }

    #[test]
    fn ensure_command_id__omitted__generates_uuid() {
        let id = ensure_command_id(None);
        assert!(Uuid::parse_str(&id).is_ok());
    }

    #[test]
    fn ensure_command_id__provided__preserved() {
        assert_eq!(ensure_command_id(Some("my-cmd")), "my-cmd");
    }

    #[test]
    fn map_resolved_scope__low__authoritative_false() {
        use ai_brains_control_plane::ResolutionEvidence;
        use ai_brains_core::ids::ProjectId;
        use ai_brains_core::scope::ScopeRef;

        let resolved = ResolvedScope {
            scope: ScopeRef::Repository(ProjectId::from_uuid(Uuid::nil())),
            confidence: ScopeConfidence::Low,
            evidence: vec![ResolutionEvidence {
                signal: "cwd".into(),
                detail: "heuristic".into(),
            }],
            warnings: vec!["cwd-only".into()],
            alternatives: Vec::new(),
        };
        let wire = map_resolved_scope(&resolved);
        assert!(!wire.authoritative);
        assert_eq!(wire.confidence, "Low");
        assert_eq!(wire.warnings.len(), 1);
        assert_eq!(wire.evidence.len(), 1);
    }

    #[test]
    fn principal_id_wire__default_system__always_some_system_uuid() {
        let system = make_principal(
            PrincipalKind::System,
            PrincipalId::from_uuid(Uuid::from_u128(
                0xA1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2,
            )),
            "cli-system",
        );
        let wire = principal_id_wire(&system);
        assert_eq!(wire.as_deref(), Some(system.id.to_string().as_str()));
        assert!(
            matches!(system.kind, PrincipalKind::System),
            "fixture must be System principal"
        );
    }

    #[test]
    fn principal_id_wire__explicit_human__that_uuid() {
        let id = Uuid::parse_str("11111111-2222-3333-4444-555555555555").expect("fixture uuid");
        let principal = make_principal(
            PrincipalKind::Human,
            PrincipalId::from_uuid(id),
            "cli-human",
        );
        let wire = principal_id_wire(&principal);
        assert_eq!(wire.as_deref(), Some(id.to_string().as_str()));
    }
}
