//! T223/T242 — local `.env` project-context override emit policy (pure helpers).
//!
//! Collapse multi-key overrides into one line, demote session-only overrides to
//! debug, honor `AI_BRAINS_QUIET_ENV_WARN` / `AI_BRAINS_FORCE_ENV_WARN`, and
//! compute session-quiet fingerprints. Precedence / force-set stay in
//! `apply_local_project_context_env` (main.rs); marker IO is in `env_warn_session`.

use sha2::{Digest, Sha256};

/// Process env key for quiet suppress (shell or project `.env` at apply time).
pub const QUIET_ENV_WARN_KEY: &str = "AI_BRAINS_QUIET_ENV_WARN";

/// Process env key to force stderr even when a session marker already exists.
pub const FORCE_ENV_WARN_KEY: &str = "AI_BRAINS_FORCE_ENV_WARN";

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

/// Inputs for the T242 session-quiet fingerprint (order fixed — F24).
#[derive(Debug, Clone, Copy)]
pub struct EnvOverrideFingerprint<'a> {
    /// Location-normalized parent of the project `.env` path.
    pub normalized_cwd: &'a str,
    /// Pre-force shell PROJECT when it differed (empty if missing / equal).
    pub old_shell_project: Option<&'a str>,
    /// Pre-force shell SESSION when it differed (empty if missing / equal).
    pub old_shell_session: Option<&'a str>,
    /// Post-force / `.env` PROJECT value by key match (empty if key absent).
    pub new_env_project: Option<&'a str>,
    /// Post-force / `.env` SESSION value by key match (empty if key absent).
    pub new_env_session: Option<&'a str>,
}

/// Quiet / force flags for pure decide (marker claim stays at call site).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvWarnPolicy {
    pub quiet: bool,
    pub force: bool,
}

/// Shared truthy SOOT for QUIET and FORCE keys: `1` / `true` / `yes` (trim, case-insensitive).
///
/// CLI-local copy (product has several independent parsers); consolidation to core is
/// a soft residual (T223 F18 / T242 F18).
pub fn env_warn_truthy(raw: Option<&str>) -> bool {
    let Some(s) = raw else {
        return false;
    };
    let t = s.trim();
    t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
}

/// Alias kept for T223 name / external call sites (implementation is `env_warn_truthy`).
#[inline]
#[allow(dead_code)] // public SOOT alias; CLI wire uses `env_warn_truthy` for quiet+force
pub fn quiet_env_warn_truthy(raw: Option<&str>) -> bool {
    env_warn_truthy(raw)
}

/// SHA-256 hex (64 lowercase chars) of `cwd|shell_p|shell_s|env_p|env_s`.
pub fn compute_fingerprint_hex(fp: &EnvOverrideFingerprint<'_>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(fp.normalized_cwd.as_bytes());
    hasher.update(b"|");
    hasher.update(fp.old_shell_project.unwrap_or("").as_bytes());
    hasher.update(b"|");
    hasher.update(fp.old_shell_session.unwrap_or("").as_bytes());
    hasher.update(b"|");
    hasher.update(fp.new_env_project.unwrap_or("").as_bytes());
    hasher.update(b"|");
    hasher.update(fp.new_env_session.unwrap_or("").as_bytes());
    hex::encode(hasher.finalize())
}

/// Body from a full stderr line (strip `Warning: ` prefix — F31).
pub fn override_body_from_stderr_line(line: &str) -> String {
    match line.strip_prefix("Warning: ") {
        Some(body) => body.to_string(),
        None => line.to_string(),
    }
}

/// Pure classify×policy without marker.
///
/// Marker claim is applied at the call site only for `Stderr` + `!quiet` + `!force`.
/// Decision table (spec §7.1 pure part):
/// - `None` → `None`
/// - `Debug` → `Debug` (unchanged; session-only)
/// - `Stderr` + quiet → `Debug(body)` (quiet wins over force)
/// - `Stderr` + !quiet + force → `Stderr` (force; no claim required)
/// - `Stderr` + !quiet + !force → `Stderr` (candidate; caller claims marker)
pub fn decide_env_override_emit(
    classified: Option<EnvOverrideEmit>,
    policy: EnvWarnPolicy,
) -> Option<EnvOverrideEmit> {
    match classified {
        None => None,
        Some(EnvOverrideEmit::Debug(body)) => Some(EnvOverrideEmit::Debug(body)),
        Some(EnvOverrideEmit::Stderr(line)) => {
            if policy.quiet {
                Some(EnvOverrideEmit::Debug(override_body_from_stderr_line(
                    &line,
                )))
            } else {
                // force and unseen-candidate both leave Stderr for the call site.
                Some(EnvOverrideEmit::Stderr(line))
            }
        }
    }
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
/// Quiet / force / marker / `!warn_on_override` are applied at the call site.
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
    const NEW_PROJECT: &str = "99999999-9999-9999-9999-999999999999";
    const NEW_SESSION: &str = "88888888-8888-8888-8888-888888888888";

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
        // Shared parser: env_warn_truthy is the same SOOT.
        assert!(env_warn_truthy(Some("1")));
        assert!(env_warn_truthy(Some("yes")));
        assert!(!env_warn_truthy(Some("0")));
        assert!(!env_warn_truthy(None));
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

    #[test]
    #[allow(non_snake_case)]
    fn decide_env_override_emit__quiet_wins_over_force() {
        let classified = classify_env_overrides(&[(PROJECT_ID_KEY, OLD_PROJECT)]);
        let decided = decide_env_override_emit(
            classified,
            EnvWarnPolicy {
                quiet: true,
                force: true,
            },
        );
        let expected =
            format!("local .env overrides inherited shell: {PROJECT_ID_KEY} (was {OLD_PROJECT}).");
        assert_eq!(decided, Some(EnvOverrideEmit::Debug(expected)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn decide_env_override_emit__force_keeps_stderr() {
        let classified = classify_env_overrides(&[(PROJECT_ID_KEY, OLD_PROJECT)]);
        let decided = decide_env_override_emit(
            classified.clone(),
            EnvWarnPolicy {
                quiet: false,
                force: true,
            },
        );
        assert_eq!(decided, classified);
        assert!(matches!(decided, Some(EnvOverrideEmit::Stderr(_))));
    }

    #[test]
    #[allow(non_snake_case)]
    fn decide_env_override_emit__unseen_keeps_stderr() {
        let classified = classify_env_overrides(&[(PROJECT_ID_KEY, OLD_PROJECT)]);
        let decided = decide_env_override_emit(
            classified.clone(),
            EnvWarnPolicy {
                quiet: false,
                force: false,
            },
        );
        // Pure decide leaves Stderr; call site still claims marker.
        assert_eq!(decided, classified);
        assert!(matches!(decided, Some(EnvOverrideEmit::Stderr(_))));
    }

    #[test]
    #[allow(non_snake_case)]
    fn decide_env_override_emit__session_only_debug_unchanged() {
        let classified = classify_env_overrides(&[(SESSION_ID_KEY, OLD_SESSION)]);
        let decided = decide_env_override_emit(
            classified.clone(),
            EnvWarnPolicy {
                quiet: false,
                force: false,
            },
        );
        assert_eq!(decided, classified);
        assert!(matches!(decided, Some(EnvOverrideEmit::Debug(_))));
    }

    #[test]
    #[allow(non_snake_case)]
    fn compute_fingerprint_hex__stable_across_identical_inputs() {
        let fp = EnvOverrideFingerprint {
            normalized_cwd: r"c:\work\proj",
            old_shell_project: Some(OLD_PROJECT),
            old_shell_session: Some(OLD_SESSION),
            new_env_project: Some(NEW_PROJECT),
            new_env_session: Some(NEW_SESSION),
        };
        let a = compute_fingerprint_hex(&fp);
        let b = compute_fingerprint_hex(&fp);
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(a, a.to_lowercase());
    }

    #[test]
    #[allow(non_snake_case)]
    fn compute_fingerprint_hex__differs_on_cwd_or_project_change() {
        let base = EnvOverrideFingerprint {
            normalized_cwd: r"c:\work\proj",
            old_shell_project: Some(OLD_PROJECT),
            old_shell_session: Some(OLD_SESSION),
            new_env_project: Some(NEW_PROJECT),
            new_env_session: Some(NEW_SESSION),
        };
        let base_hex = compute_fingerprint_hex(&base);

        let cwd_changed = EnvOverrideFingerprint {
            normalized_cwd: r"c:\work\other",
            ..base
        };
        assert_ne!(base_hex, compute_fingerprint_hex(&cwd_changed));

        let shell_project_changed = EnvOverrideFingerprint {
            old_shell_project: Some("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
            ..base
        };
        assert_ne!(base_hex, compute_fingerprint_hex(&shell_project_changed));

        let env_project_changed = EnvOverrideFingerprint {
            new_env_project: Some("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
            ..base
        };
        assert_ne!(base_hex, compute_fingerprint_hex(&env_project_changed));
    }
}
