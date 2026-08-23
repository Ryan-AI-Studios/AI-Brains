//! Shared helpers for governed CLI commands (T160).
//!
//! Thin adapter utilities only — no domain logic. Path policy, emit, exit codes,
//! principal resolution, and ResolvedScope → wire DTO mapping live here.

use crate::commands::briefing::cli_principal;
use crate::daemon_client::{DaemonClient, DaemonClientError};
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::scopes::{ScopeEvidenceDto, ScopeResolvedResponse};
use ai_brains_control_plane::{
    ControlPlaneError, ResolvedScope, ScopeConfidence, ScopeIdentityStore, ScopeResolveInput,
    is_authoritative, make_principal, resolve_scope, scope_identity_key,
};
use ai_brains_core::ids::{PrincipalId, ProjectId};
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_daemon_api::DaemonResponse;
use serde::Serialize;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
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

/// INSTALL primary reinstall SOOT (T200 F27 / T222 F27 / T232 F5) — single source for graph-off
/// stubs, doctor `graph_feature` remediation, and graph_density warn remediations when graph-off.
pub const GRAPH_REINSTALL_SOOT: &str =
    "cargo install --path crates/ai-brains-cli --locked --features graph";

/// Stable F6 remediation template for POLICY_DENIED `details.hint` (T201 / T210).
///
/// Dual-site SOOT with `ai_brainsd::services::POLICY_DENIED_HINT` — keep wording in sync (T280 F1).
pub const POLICY_DENIED_HINT: &str = "ensure a grant for this capability exists; run `ai-brains policy bootstrap --dry-run` then `ai-brains policy bootstrap` (omit --scope when project context is authoritative)";

// CLI-only progressive→recall fallback (T243 F13). Not dual-site.
pub const PROGRESSIVE_RECALL_FALLBACK: &str = "Ungoverned vault search: ai-brains recall \"…\"";

/// Default copy-paste recall needle for granted-empty lists (T290 F5).
pub const LIST_RECALL_QUERY: &str = "what did we decide";

/// Collapse ASCII whitespace, replace `"`, cap 80 chars (T290 F6). Empty → [`LIST_RECALL_QUERY`].
pub fn sanitize_recall_query(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return LIST_RECALL_QUERY.to_string();
    }
    let mut collapsed = String::with_capacity(trimmed.len());
    let mut prev_space = false;
    for c in trimmed.chars() {
        if c.is_ascii_whitespace() {
            if !prev_space {
                collapsed.push(' ');
                prev_space = true;
            }
        } else if c == '$' || c == '`' {
            // Drop PowerShell interpolators so copy-paste `recall "…"` is not executable.
            prev_space = false;
        } else {
            prev_space = false;
            collapsed.push(if c == '"' { '\'' } else { c });
        }
    }
    if collapsed.is_empty() {
        return LIST_RECALL_QUERY.to_string();
    }
    collapsed.chars().take(80).collect()
}

/// Granted-empty `next_step` (T290 F7). `recall_query` None → [`LIST_RECALL_QUERY`].
pub fn format_authorized_empty_next(pin_count: Option<u64>, recall_query: Option<&str>) -> String {
    let needle = match recall_query {
        Some(q) => sanitize_recall_query(q),
        None => LIST_RECALL_QUERY.to_string(),
    };
    match pin_count {
        Some(n) => {
            format!("Ungoverned vault search: ai-brains recall \"{needle}\" (Pinned: {n})")
        }
        None => format!("Ungoverned vault search: ai-brains recall \"{needle}\""),
    }
}

/// CLI overlay for authorized-empty discovery lists (T263 F8 / T290 F3).
///
/// When JSON has an empty `items` array and is not a deny/error envelope, set
/// `next_step` (if absent) via [`format_authorized_empty_next`]. Does not change DTOs.
pub fn apply_authorized_empty_list_next(value: &mut serde_json::Value, pin_count: Option<u64>) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    if obj.get("denied").and_then(|d| d.as_bool()) == Some(true) {
        return;
    }
    if obj.get("code").and_then(|c| c.as_str()) == Some("POLICY_DENIED") {
        return;
    }
    let empty_items = obj
        .get("items")
        .and_then(|i| i.as_array())
        .is_some_and(|a| a.is_empty());
    if !empty_items {
        return;
    }
    if obj.contains_key("next_step") {
        return;
    }
    obj.insert(
        "next_step".to_string(),
        serde_json::Value::String(format_authorized_empty_next(pin_count, None)),
    );
}

/// Expand Unknown preview SOOT (T263 F7) — CLI overlay on existing `preview` string.
pub const UNKNOWN_HANDLE_PREVIEW: &str = "Handle not found.";

/// Discovery-class capability labels (T210 bootstrap / T241 probe) — Read* only.
pub const DISCOVERY_CAP_LABELS: [&str; 3] = ["ReadEvidence", "ReadConclusions", "ReadDecisions"];

/// Full capability catalog for `policy check` fail_usage + after_help (T241 F6b).
/// Discovery trio first, then remaining stable order.
pub const CAPABILITY_CATALOG: &[&str] = &[
    "ReadEvidence (discovery)",
    "ReadConclusions (discovery)",
    "ReadDecisions (discovery)",
    "ApproveConclusion",
    "ApproveDecision",
    "Erase",
    "Export",
    "ProposeConclusion",
    "ProposeDecision",
];

/// Short bootstrap SOOT (T241 F14) — show human, preflight, briefing denial_hint.
pub const POLICY_BOOTSTRAP_SOOT_SHORT: &str =
    "next: run `ai-brains policy bootstrap --dry-run` then `ai-brains policy bootstrap`";

/// Long bootstrap SOOT (T241 F14) — doctor remediation only.
pub const POLICY_BOOTSTRAP_SOOT_LONG: &str = "next: run `ai-brains policy bootstrap --dry-run` then `ai-brains policy bootstrap` (omit --scope when project context is authoritative)";

/// Usage message when `--capability` is omitted on `policy check` (T241 F6/F30).
pub fn capability_required_usage_message() -> String {
    let mut lines = Vec::with_capacity(1 + CAPABILITY_CATALOG.len());
    lines.push("--capability is required. Valid capabilities:".to_string());
    for cap in CAPABILITY_CATALOG {
        lines.push(format!("  {cap}"));
    }
    lines.join("\n")
}

/// Count unique discovery labels present among applied grants (T241 F1/F31).
pub fn discovery_active_count<'a, I>(capabilities: I) -> usize
where
    I: IntoIterator<Item = &'a str>,
{
    let mut present = std::collections::BTreeSet::new();
    for cap in capabilities {
        for label in &DISCOVERY_CAP_LABELS {
            if cap.eq_ignore_ascii_case(label) {
                present.insert(*label);
            }
        }
    }
    present.len()
}

/// Build `{"hint": …}` details without `serde_json::json!` (disallowed unwrap in production).
pub fn policy_denied_hint_details() -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "hint".to_string(),
        serde_json::Value::String(POLICY_DENIED_HINT.to_string()),
    );
    serde_json::Value::Object(map)
}

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
        // Domain state-machine refusal (known CP code; not silent catch-all).
        "INVALID_TRANSITION" => EXIT_INTERNAL,
        // Evaluate trust gates failed (harness worked; product blocked).
        "HARD_GATE_FAILED" => EXIT_HARD_GATE_FAILED,
        // Optional feature not in this binary (T198/T200).
        code if code == FEATURE_UNAVAILABLE => exit_code_feature_unavailable(),
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
    crate::commands::identity_warn::print_json_stdout(value)
}

/// Emit human text on stdout.
pub fn emit_human(text: &str) {
    println!("{text}");
}

/// Emit structured error for json mode (stdout) or human mode (stderr).
///
/// T221 F5: Human/Markdown also print `details.hint` on a following stderr line
/// when it is a non-empty string (bootstrap remediation for POLICY_DENIED).
pub fn emit_error(format: OutputFormat, err: &ApiError) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => {
            // Scripts parse one stream: JSON error envelope on stdout.
            crate::commands::identity_warn::print_json_stdout(err)?;
        }
        OutputFormat::Human | OutputFormat::Markdown => {
            eprintln!("{}: {}", err.code, err.message);
            if let Some(hint) = api_error_hint(err) {
                eprintln!("{hint}");
            }
        }
    }
    Ok(())
}

/// Extract non-empty `details.hint` string from an [`ApiError`] (T221 F5 / tests).
pub(crate) fn api_error_hint(err: &ApiError) -> Option<&str> {
    err.details
        .as_ref()
        .and_then(|d| d.get("hint"))
        .and_then(|h| h.as_str())
        .filter(|s| !s.is_empty())
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

/// Emit a usage-class failure on stderr and return exit **2** (T202 F11).
///
/// Prefer this for missing required operator inputs that are not clap-required
/// (e.g. progressive project id after env bind). `handle_cli_result` already
/// downcasts [`GovernedCliError`] to the exit code.
pub fn fail_usage(msg: impl Into<String>) -> GovernedResult {
    let message = msg.into();
    eprintln!("{message}");
    Err(Box::new(GovernedCliError::emitted(EXIT_USAGE, message)))
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
// Scope mapping + soft-resolve (T203)
// ---------------------------------------------------------------------------

/// Resolve a scope identity key for list/show CLI paths (T203 F6).
///
/// Order: explicit `--scope` wins → else soft-resolve from cwd / `AI_BRAINS_PROJECT_ID`
/// when the result is authoritative and non-empty → else `Err` usage message for
/// [`fail_usage`] at the call site (exit **2**, never exit **6**).
pub fn resolve_scope_key_for_cli(
    explicit: Option<&str>,
    identity: &impl ScopeIdentityStore,
) -> Result<String, String> {
    if let Some(s) = explicit.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(s.to_string());
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let explicit_project_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .and_then(|s| ProjectId::from_str(&s).ok());

    let input = ScopeResolveInput {
        cwd,
        explicit_project_id,
        force_personal: false,
        personal_user_id: None,
        git_metadata: None,
    };

    let resolved = resolve_scope(&input, identity).map_err(|e| {
        format!(
            "scope resolve failed while soft-filling omitted --scope: {e}\n\
             Provide an explicit scope, for example:\n\
               --scope Repository:<uuid>\n\
             Or run: ai-brains scope resolve\n\
             Non-authoritative context is not filled silently."
        )
    })?;

    let key = scope_identity_key(&resolved.scope);
    let authoritative = is_authoritative(&resolved)
        && !matches!(
            resolved.confidence,
            ScopeConfidence::Low | ScopeConfidence::Ambiguous
        )
        && !key.is_empty();

    if authoritative {
        return Ok(key);
    }

    Err(soft_resolve_usage_message(&resolved, &key))
}

fn soft_resolve_usage_message(resolved: &ResolvedScope, key: &str) -> String {
    let suggested = if key.is_empty() {
        "Repository:<uuid>".to_string()
    } else {
        key.to_string()
    };
    let alt_hint = if resolved.alternatives.is_empty() {
        String::new()
    } else {
        let alts: Vec<String> = resolved
            .alternatives
            .iter()
            .map(scope_identity_key)
            .collect();
        format!("\nAlternatives: {}", alts.join(", "))
    };
    let confidence = confidence_name(resolved.confidence);
    format!(
        "missing --scope and context is not authoritative (confidence: {confidence})\n\
         Provide an explicit scope, for example:\n\
           --scope {suggested}\n\
         Or run: ai-brains scope resolve\n\
         Non-authoritative context is not filled silently.\
         {alt_hint}"
    )
}

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

/// Format human-friendly scope resolution (T160 field order + T249 next-step).
pub(crate) fn format_scope_human(resp: &ScopeResolvedResponse) -> String {
    let auth = if resp.authoritative {
        "authoritative"
    } else {
        "NOT authoritative"
    };
    let mut lines = Vec::new();
    lines.push(format!("scope: {}", resp.scope));
    lines.push(format!("confidence: {} ({auth})", resp.confidence));
    if !resp.warnings.is_empty() {
        lines.push("warnings:".into());
        for w in &resp.warnings {
            lines.push(format!("  - {w}"));
        }
    }
    if !resp.alternatives.is_empty() {
        lines.push("alternatives:".into());
        for a in &resp.alternatives {
            lines.push(format!("  - {a}"));
        }
    }
    if !resp.evidence.is_empty() {
        lines.push("evidence:".into());
        for e in &resp.evidence {
            lines.push(format!("  - {}: {}", e.signal, e.detail));
        }
    }
    if !resp.authoritative {
        lines.push(
            "note: non-authoritative resolution — do not treat as full grant (scope #20)".into(),
        );
        lines.push("next: ai-brains project whoami".into());
    }
    format!("{}\n", lines.join("\n"))
}

/// Print human-friendly scope resolution (always surfaces authoritative / warnings / alternatives).
pub fn emit_scope_human(resp: &ScopeResolvedResponse) {
    print!("{}", format_scope_human(resp));
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn exit_code_for_api_error__policy_denied__3() {
        let err = ApiError::new("POLICY_DENIED", "no grant");
        assert_eq!(exit_code_for_api_error(&err), EXIT_POLICY_DENIED);
    }

    #[test]
    fn apply_authorized_empty_list_next__empty_items__sets_recall() {
        // T263 AC7 / F8
        let mut value = serde_json::json!({
            "items": [],
            "more_available": false,
        });
        apply_authorized_empty_list_next(&mut value, None);
        let step = value["next_step"].as_str().unwrap_or("");
        assert!(
            !step.is_empty() && step.contains("recall"),
            "authorized-empty list must set next_step with recall; got {value}"
        );
        assert_eq!(
            value["items"].as_array().map(Vec::len),
            Some(0),
            "items must stay empty; got {value}"
        );
    }

    #[test]
    fn apply_authorized_empty_list_next__nonempty_or_denied__omits_next_step() {
        let mut nonempty = serde_json::json!({"items": [{"id": "1"}]});
        apply_authorized_empty_list_next(&mut nonempty, Some(12));
        assert!(
            nonempty.get("next_step").is_none(),
            "non-empty items must not get next_step; got {nonempty}"
        );
        let mut denied = serde_json::json!({
            "code": "POLICY_DENIED",
            "message": "no grant",
            "items": [],
        });
        apply_authorized_empty_list_next(&mut denied, Some(12));
        assert!(
            denied.get("next_step").is_none(),
            "denied envelope must not get authorized-empty next_step; got {denied}"
        );
        let mut denied_flag = serde_json::json!({"denied": true, "items": []});
        apply_authorized_empty_list_next(&mut denied_flag, Some(12));
        assert!(
            denied_flag.get("next_step").is_none(),
            "denied:true must not get authorized-empty next_step; got {denied_flag}"
        );
    }

    /// T290 AC14 — overlay gate rstest: denied / nonempty skip; empty apply.
    #[rstest]
    #[case(serde_json::json!({"items": [{"id": "1"}]}), true)]
    #[case(serde_json::json!({"denied": true, "items": []}), true)]
    #[case(serde_json::json!({"code": "POLICY_DENIED", "items": []}), true)]
    #[case(serde_json::json!({"items": []}), false)]
    fn apply_authorized_empty_list_next__overlay_gate__denied_nonempty_empty(
        #[case] mut value: serde_json::Value,
        #[case] omit: bool,
    ) {
        apply_authorized_empty_list_next(&mut value, Some(3));
        if omit {
            assert!(
                value.get("next_step").is_none(),
                "overlay must skip; got {value}"
            );
        } else {
            let step = value["next_step"].as_str().unwrap_or("");
            assert!(
                step.contains("recall") && step.contains("(Pinned: 3)"),
                "empty overlay must apply; got {value}"
            );
        }
    }

    /// T290 AC15 — deny-stderr ellipsis const frozen.
    #[test]
    fn progressive_recall_fallback__exact__ellipsis_unchanged() {
        assert_eq!(
            PROGRESSIVE_RECALL_FALLBACK,
            "Ungoverned vault search: ai-brains recall \"…\""
        );
    }

    /// T290 AC13 — list/progressive DTOs stay unaugmented (no T288 keys).
    #[test]
    fn list_and_progressive_dtos__serde__no_vault_pin_count() {
        use ai_brains_contracts::briefings::{EvidenceListResponse, ProgressiveQueryResponse};
        use ai_brains_contracts::review::ReviewQueueResponse;
        use ai_brains_contracts::sources::SourceListResponse;
        let packets = [
            serde_json::to_value(EvidenceListResponse::new(Vec::new())).expect("evidence"),
            serde_json::to_value(SourceListResponse::new(Vec::new())).expect("source"),
            serde_json::to_value(ReviewQueueResponse::new(Vec::new())).expect("review"),
            serde_json::to_value(ProgressiveQueryResponse::new(
                Vec::new(),
                "scope",
                "policy",
                "trace",
                false,
            ))
            .expect("progressive"),
        ];
        for v in packets {
            assert!(
                v.get("vault_pin_count").is_none() && v.get("vault_pin_previews").is_none(),
                "DTO must not grow T288 keys; got {v}"
            );
        }
    }

    /// T290 AC1 — exact F7 shape; single line; no U+2026.
    #[test]
    fn format_authorized_empty_next__with_count__includes_pinned_and_copy_paste() {
        let with_count = format_authorized_empty_next(Some(12), None);
        assert_eq!(
            with_count,
            "Ungoverned vault search: ai-brains recall \"what did we decide\" (Pinned: 12)"
        );
        assert!(
            !with_count.contains('\n') && !with_count.contains('…'),
            "formatter must be one line without U+2026; got {with_count}"
        );
        let without = format_authorized_empty_next(None, None);
        assert_eq!(
            without,
            "Ungoverned vault search: ai-brains recall \"what did we decide\""
        );
        assert!(
            !without.contains('\n') && !without.contains('…'),
            "formatter must be one line without U+2026; got {without}"
        );
    }

    /// T290 AC4 / AC14 — sanitize cases (tab, newline, quotes, empty, 80-cap).
    #[rstest]
    #[case("  foo\nbar  ", "foo bar")]
    #[case("foo\tbar", "foo bar")]
    #[case("say \"hi\"", "say 'hi'")]
    #[case("echo $(hi)", "echo (hi)")]
    #[case("say `whoami`", "say whoami")]
    #[case("", "what did we decide")]
    #[case("   ", "what did we decide")]
    fn sanitize_recall_query__cases__expected_needle(#[case] raw: &str, #[case] expected: &str) {
        assert_eq!(sanitize_recall_query(raw), expected, "raw={raw:?}");
    }

    #[test]
    fn sanitize_recall_query__eighty_one_a__truncates_to_eighty() {
        let raw = "a".repeat(81);
        let got = sanitize_recall_query(&raw);
        assert_eq!(got.len(), 80, "got={got}");
        assert_eq!(got, "a".repeat(80));
    }

    #[test]
    fn format_authorized_empty_next__newline_query__single_line() {
        let step = format_authorized_empty_next(Some(0), Some("  foo\nbar  "));
        assert!(
            !step.contains('\n'),
            "formatter must stay one line after newline query; got {step}"
        );
    }

    #[test]
    fn format_authorized_empty_next__powershell_interpolators__stripped() {
        let step = format_authorized_empty_next(None, Some("echo $(Get-Process) `whoami`"));
        assert!(
            !step.contains('$') && !step.contains('`'),
            "copy-paste next_step must not keep PowerShell interpolators; got {step}"
        );
        assert!(
            step.contains("echo (Get-Process) whoami"),
            "needle must keep the rest of the query; got {step}"
        );
    }

    /// T280 F1 / F27 / AC1 — deny HINT omits required `--scope …` (U+2026).
    #[test]
    fn policy_denied_hint__wording__omits_required_scope() {
        const F1: &str = "ensure a grant for this capability exists; run `ai-brains policy bootstrap --dry-run` then `ai-brains policy bootstrap` (omit --scope when project context is authoritative)";
        assert_eq!(POLICY_DENIED_HINT, F1);
        assert_eq!(POLICY_DENIED_HINT.len(), 172);
        assert!(
            !POLICY_DENIED_HINT.contains("--scope …"),
            "HINT must not require --scope ellipsis; got {POLICY_DENIED_HINT}"
        );
        assert!(
            POLICY_DENIED_HINT.contains("omit --scope")
                && POLICY_DENIED_HINT.contains("policy bootstrap")
                && POLICY_DENIED_HINT.contains("--dry-run"),
            "HINT must name dry-run bootstrap and omit-scope; got {POLICY_DENIED_HINT}"
        );
    }

    /// AC4 / F5 — human deny path surfaces details.hint for bootstrap remediation.
    #[test]
    fn emit_error__human_with_details_hint__hint_extractable() {
        let err =
            ApiError::new("POLICY_DENIED", "no grant").with_details(policy_denied_hint_details());
        let hint = api_error_hint(&err).expect("hint present");
        assert!(
            hint.contains("policy bootstrap"),
            "hint must mention bootstrap; got {hint}"
        );
        assert_eq!(hint, POLICY_DENIED_HINT);
        // Human path prints CODE then hint (emit_error implementation); unit locks extract + SOOT.
    }

    #[test]
    fn emit_error__human_without_hint__no_hint() {
        let err = ApiError::new("POLICY_DENIED", "no grant");
        assert!(api_error_hint(&err).is_none());
    }

    /// AC12 / F33 — ControlPlaneError::PolicyDenied maps to exit 3 via fail_cp path.
    #[test]
    fn exit_code_for_cp__policy_denied__3() {
        let err = ControlPlaneError::PolicyDenied("capability denied".into());
        assert_eq!(exit_code_for_cp(&err), EXIT_POLICY_DENIED);
        let api = api_error_from_cp(&err);
        assert_eq!(api.code, "POLICY_DENIED");
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
    fn exit_code_for_api_error__invalid_transition__1() {
        let err = ApiError::new("INVALID_TRANSITION", "already resolved");
        assert_eq!(exit_code_for_api_error(&err), EXIT_INTERNAL);
        assert_eq!(exit_code_for_api_error(&err), 1);
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
    fn format_scope_human__authoritative__has_scope_no_next() {
        let mut resp = ScopeResolvedResponse::new(
            "Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "High",
            true,
        );
        resp.evidence.push(ScopeEvidenceDto {
            signal: "explicit_project_id".into(),
            detail: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
        });
        let out = format_scope_human(&resp);
        assert!(out.contains("scope:"), "got:\n{out}");
        assert!(
            out.contains(" (authoritative)"),
            "authoritative label missing:\n{out}"
        );
        assert!(
            !out.contains("NOT authoritative"),
            "authoritative must not say NOT:\n{out}"
        );
        assert!(out.contains("evidence:"), "got:\n{out}");
        assert!(
            out.contains("explicit_project_id"),
            "evidence signal missing:\n{out}"
        );
        assert!(
            !out.contains("next:"),
            "authoritative must omit next:\n{out}"
        );
    }

    #[test]
    fn format_scope_human__non_authoritative__note_and_whoami_next() {
        let mut resp = ScopeResolvedResponse::new("", "Low", false);
        resp.evidence.push(ScopeEvidenceDto {
            signal: "cwd".into(),
            detail: "heuristic".into(),
        });
        let out = format_scope_human(&resp);
        assert!(
            out.contains("scope:"),
            "empty scope still prints scope:\n{out}"
        );
        assert!(out.contains("NOT authoritative"), "got:\n{out}");
        assert!(
            out.contains(
                "note: non-authoritative resolution — do not treat as full grant (scope #20)"
            ),
            "T160 note missing:\n{out}"
        );
        let last = out
            .lines()
            .rev()
            .find(|l| !l.is_empty())
            .expect("non-empty line");
        assert_eq!(last, "next: ai-brains project whoami");
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
