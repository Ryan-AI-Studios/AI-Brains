//! Soft-fail policy for git CLI helpers (T155 R1-01/R1-02).
//!
//! [`SoftFailPolicy::Soft`] preserves legacy degrade-to-empty behavior used by
//! the scope resolver and other non-connector callers of [`crate::collect_metadata`].
//! [`SoftFailPolicy::Strict`] propagates hard failures so the Git connector can
//! surface timeout / missing binary / non-not-a-repo command failures as `Err`.

use crate::command::DEFAULT_GIT_TIMEOUT;
use crate::errors::GitError;
use std::time::Duration;

/// Whether git helper failures soft-map to empty/default or propagate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftFailPolicy {
    /// Map many failures to empty/default (legacy / scope resolver).
    Soft,
    /// Propagate Timeout, Io, and non-not-a-repo CommandFailed.
    Strict,
}

/// Timeout + soft-fail policy for a collect / helper call chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitRunOptions {
    /// Per-spawn deadline passed to [`crate::run_git_timeout`].
    pub timeout: Duration,
    /// Soft vs strict failure mapping.
    pub policy: SoftFailPolicy,
}

impl Default for GitRunOptions {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_GIT_TIMEOUT,
            policy: SoftFailPolicy::Soft,
        }
    }
}

impl GitRunOptions {
    /// Soft policy with the default timeout (legacy collect).
    pub fn soft() -> Self {
        Self::default()
    }

    /// Strict policy with the default timeout (Git connector collect).
    pub fn strict() -> Self {
        Self {
            timeout: DEFAULT_GIT_TIMEOUT,
            policy: SoftFailPolicy::Strict,
        }
    }

    /// Soft policy with an explicit timeout.
    pub fn soft_with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            policy: SoftFailPolicy::Soft,
        }
    }

    /// Strict policy with an explicit timeout.
    pub fn strict_with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            policy: SoftFailPolicy::Strict,
        }
    }
}

/// True when a [`GitError::CommandFailed`] stderr looks like a genuine
/// "not a git repository" outcome (case-insensitive substring match).
pub fn is_not_a_git_repository_error(err: &GitError) -> bool {
    match err {
        GitError::CommandFailed { message, .. } => message
            .to_ascii_lowercase()
            .contains("not a git repository"),
        _ => false,
    }
}

/// Whether `discover_root` should map this error to `Ok(None)` under `policy`.
///
/// - **Soft:** map everything except [`GitError::Timeout`] (timeout stays hard).
/// - **Strict:** map only genuine not-a-repository [`GitError::CommandFailed`].
pub fn discover_error_is_soft(err: &GitError, policy: SoftFailPolicy) -> bool {
    match policy {
        SoftFailPolicy::Soft => !matches!(err, GitError::Timeout { .. }),
        SoftFailPolicy::Strict => is_not_a_git_repository_error(err),
    }
}

/// Soft-map a helper `Result` under Soft; propagate under Strict.
pub(crate) fn or_soft_default<T>(
    result: Result<T, GitError>,
    policy: SoftFailPolicy,
    default: T,
) -> Result<T, GitError> {
    match (result, policy) {
        (Ok(v), _) => Ok(v),
        (Err(_), SoftFailPolicy::Soft) => Ok(default),
        (Err(e), SoftFailPolicy::Strict) => Err(e),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn timeout_err() -> GitError {
        GitError::Timeout {
            command: "git rev-parse --show-toplevel".into(),
            elapsed_ms: 5000,
        }
    }

    fn not_a_repo_err() -> GitError {
        GitError::CommandFailed {
            command: "git rev-parse --show-toplevel".into(),
            message: "fatal: not a git repository (or any of the parent directories): .git".into(),
        }
    }

    fn other_command_failed() -> GitError {
        GitError::CommandFailed {
            command: "git status".into(),
            message: "fatal: bad object".into(),
        }
    }

    fn io_err() -> GitError {
        GitError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "program not found",
        ))
    }

    #[test]
    fn classify_git_error__timeout__is_hard() {
        let err = timeout_err();
        assert!(!is_not_a_git_repository_error(&err));
        assert!(!discover_error_is_soft(&err, SoftFailPolicy::Soft));
        assert!(!discover_error_is_soft(&err, SoftFailPolicy::Strict));
    }

    #[test]
    fn classify_git_error__not_a_repo_message__is_soft() {
        let err = not_a_repo_err();
        assert!(is_not_a_git_repository_error(&err));
        assert!(discover_error_is_soft(&err, SoftFailPolicy::Soft));
        assert!(discover_error_is_soft(&err, SoftFailPolicy::Strict));
    }

    #[test]
    fn soft_fail_policy__strict__propagates_timeout() {
        let err = timeout_err();
        assert!(!discover_error_is_soft(&err, SoftFailPolicy::Strict));
        let mapped: Result<Option<()>, _> = or_soft_default(Err(err), SoftFailPolicy::Strict, None);
        assert!(matches!(mapped, Err(GitError::Timeout { .. })));
    }

    #[test]
    fn soft_fail_policy__strict__not_a_repo_command_failed__ok_none() {
        let err = not_a_repo_err();
        assert!(discover_error_is_soft(&err, SoftFailPolicy::Strict));
    }

    #[test]
    fn soft_fail_policy__soft__command_failed__ok_none() {
        let err = other_command_failed();
        assert!(discover_error_is_soft(&err, SoftFailPolicy::Soft));
        // Soft mid-helper still maps CommandFailed to default.
        let mapped = or_soft_default(Err(err), SoftFailPolicy::Soft, 0u32);
        match mapped {
            Ok(v) => assert_eq!(v, 0),
            Err(e) => panic!("expected soft default, got {e}"),
        }
    }

    #[test]
    fn soft_fail_policy__strict__io__propagates() {
        let err = io_err();
        assert!(!discover_error_is_soft(&err, SoftFailPolicy::Strict));
        let mapped: Result<(), _> = or_soft_default(Err(err), SoftFailPolicy::Strict, ());
        assert!(matches!(mapped, Err(GitError::Io(_))));
    }

    #[test]
    fn soft_fail_policy__strict__other_command_failed__not_soft_for_discover() {
        let err = other_command_failed();
        assert!(!is_not_a_git_repository_error(&err));
        assert!(!discover_error_is_soft(&err, SoftFailPolicy::Strict));
    }

    #[test]
    fn is_not_a_git_repository_error__case_insensitive() {
        let err = GitError::CommandFailed {
            command: "git rev-parse --show-toplevel".into(),
            message: "FATAL: NOT A GIT REPOSITORY".into(),
        };
        assert!(is_not_a_git_repository_error(&err));
    }
}
