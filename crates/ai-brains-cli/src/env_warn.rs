//! T223 — local `.env` project-context override emit policy (pure helpers).
//!
//! Collapse multi-key overrides into one line, demote session-only overrides to
//! debug, and honor `AI_BRAINS_QUIET_ENV_WARN`. Precedence / force-set stay in
//! `apply_local_project_context_env` (main.rs); this module only formats and
//! classifies collected `(key, old_shell_value)` pairs.

/// Process env key for quiet suppress (shell or project `.env` at apply time).
pub const QUIET_ENV_WARN_KEY: &str = "AI_BRAINS_QUIET_ENV_WARN";

/// Force-set project context keys (stable order for collect + format).
pub const PROJECT_ID_KEY: &str = "AI_BRAINS_PROJECT_ID";
pub const SESSION_ID_KEY: &str = "AI_BRAINS_SESSION_ID";

/// Emit target for a non-empty override set after classify (quiet applied at call site).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvOverrideEmit {
    /// Collapsed body without `Warning:` prefix (session-only, quiet, or non-warn command).
    Debug(String),
    /// Full stderr line including `Warning: ` prefix (project differs, not quiet).
    Stderr(String),
}

/// Truthy SOOT for `AI_BRAINS_QUIET_ENV_WARN`: `1` / `true` / `yes` (trim, case-insensitive).
///
/// CLI-local copy (product has several independent parsers); consolidation to core is
/// a soft residual (T223 F18).
pub fn quiet_env_warn_truthy(raw: Option<&str>) -> bool {
    let Some(s) = raw else {
        return false;
    };
    let t = s.trim();
    t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
}

/// Body shared by stderr and debug (no `Warning:` prefix).
///
/// SOOT: `local .env overrides inherited shell: AI_BRAINS_PROJECT_ID (was {old})[, …].`
/// Caller must pass keys in stable order (PROJECT then SESSION when both).
pub fn format_override_body(overrides: &[(&str, &str)]) -> String {
    let mut parts = Vec::with_capacity(overrides.len());
    for (key, old) in overrides {
        parts.push(format!("{key} (was {old})"));
    }
    format!(
        "local .env overrides inherited shell: {}.",
        parts.join(", ")
    )
}

/// Classify collected overrides for default warn policy.
///
/// - empty → `None`
/// - session-only (`AI_BRAINS_SESSION_ID` sole key) → `Debug(body)`
/// - otherwise (project differs, with or without session) → `Stderr("Warning: " + body)`
///
/// Quiet / `!warn_on_override` are applied at the call site (force Debug when non-empty).
pub fn classify_env_overrides(overrides: &[(&str, &str)]) -> Option<EnvOverrideEmit> {
    if overrides.is_empty() {
        return None;
    }
    let body = format_override_body(overrides);
    let session_only = overrides.len() == 1 && overrides[0].0 == SESSION_ID_KEY;
    if session_only {
        Some(EnvOverrideEmit::Debug(body))
    } else {
        Some(EnvOverrideEmit::Stderr(format!("Warning: {body}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD_PROJECT: &str = "77777777-7777-7777-7777-777777777777";
    const OLD_SESSION: &str = "66666666-6666-6666-6666-666666666666";

    #[test]
    #[allow(non_snake_case)]
    fn classify_env_overrides__both_keys__stderr_project_then_session() {
        let overrides = [(PROJECT_ID_KEY, OLD_PROJECT), (SESSION_ID_KEY, OLD_SESSION)];
        let emit = classify_env_overrides(&overrides);
        let expected = format!(
            "Warning: local .env overrides inherited shell: {PROJECT_ID_KEY} (was {OLD_PROJECT}), {SESSION_ID_KEY} (was {OLD_SESSION})."
        );
        assert_eq!(emit, Some(EnvOverrideEmit::Stderr(expected)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_env_overrides__project_only__stderr_project() {
        let overrides = [(PROJECT_ID_KEY, OLD_PROJECT)];
        let emit = classify_env_overrides(&overrides);
        let expected = format!(
            "Warning: local .env overrides inherited shell: {PROJECT_ID_KEY} (was {OLD_PROJECT})."
        );
        assert_eq!(emit, Some(EnvOverrideEmit::Stderr(expected)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_env_overrides__session_only__debug_session() {
        let overrides = [(SESSION_ID_KEY, OLD_SESSION)];
        let emit = classify_env_overrides(&overrides);
        let expected =
            format!("local .env overrides inherited shell: {SESSION_ID_KEY} (was {OLD_SESSION}).");
        assert_eq!(emit, Some(EnvOverrideEmit::Debug(expected)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_env_overrides__empty__none() {
        assert_eq!(classify_env_overrides(&[]), None);
    }

    #[test]
    #[allow(non_snake_case)]
    fn quiet_env_warn_truthy__matrix() {
        assert!(quiet_env_warn_truthy(Some("1")));
        assert!(quiet_env_warn_truthy(Some("true")));
        assert!(quiet_env_warn_truthy(Some("TRUE")));
        assert!(quiet_env_warn_truthy(Some(" yes ")));
        assert!(!quiet_env_warn_truthy(Some("0")));
        assert!(!quiet_env_warn_truthy(Some("no")));
        assert!(!quiet_env_warn_truthy(Some("")));
        assert!(!quiet_env_warn_truthy(None));
        assert!(!quiet_env_warn_truthy(Some("false")));
        assert!(!quiet_env_warn_truthy(Some("2")));
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_override_body__both_keys__collapsed_debug_soot() {
        let overrides = [(PROJECT_ID_KEY, OLD_PROJECT), (SESSION_ID_KEY, OLD_SESSION)];
        let body = format_override_body(&overrides);
        let expected = format!(
            "local .env overrides inherited shell: {PROJECT_ID_KEY} (was {OLD_PROJECT}), {SESSION_ID_KEY} (was {OLD_SESSION})."
        );
        assert_eq!(body, expected);
        // One collapsed line — no newlines, no Warning prefix.
        assert!(!body.contains('\n'));
        assert!(!body.starts_with("Warning:"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_override_body__never_legacy_per_key_template() {
        let overrides = [(PROJECT_ID_KEY, OLD_PROJECT), (SESSION_ID_KEY, OLD_SESSION)];
        let body = format_override_body(&overrides);
        let stderr = match classify_env_overrides(&overrides) {
            Some(EnvOverrideEmit::Stderr(s)) => s,
            other => panic!("expected Stderr, got {other:?}"),
        };
        let legacy = "local .env AI_BRAINS_PROJECT_ID overrides inherited shell value";
        assert!(
            !body.contains(legacy),
            "debug body must not use legacy dual template; got: {body}"
        );
        assert!(
            !stderr.contains(legacy),
            "stderr must not use legacy dual template; got: {stderr}"
        );
        // Session key legacy shape also gone.
        let legacy_session = "local .env AI_BRAINS_SESSION_ID overrides inherited shell value";
        assert!(!body.contains(legacy_session));
        assert!(!stderr.contains(legacy_session));
    }
}
