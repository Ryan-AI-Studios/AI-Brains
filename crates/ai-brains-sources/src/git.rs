//! Git repository connector (`builtin.git`) — T155 / P6.3.
//!
//! # Design locks
//!
//! - **I/O:** only via [`ai_brains_git::collect_metadata`] (CLI git; no libgit2).
//! - **Identity:** `{scope_key}|GitRepository|{normalized_root_or_common_dir}`.
//! - **Content:** [`crate::canonicalize_git_metadata`] bytes; fingerprint via
//!   [`crate::fingerprint_git_metadata`].
//! - **Write-back:** [`Connector::propose_write`] is unsupported.
//! - **Sandbox / trust:** TrustedBuiltin / LocalOnly; credentials PathAccess.
//!
//! # Unavailability (anti-#22)
//!
//! | Situation | Behavior |
//! |-----------|----------|
//! | Path not a git repo | `list` → `Ok([])` **and** [`Self::last_unavailable_reason`] =
//!   `"not_a_repository"`; `observe` on any handle → `Err` |
//! | Timeout / CommandFailed / Io | Prefer `Err(ConnectorError::Internal{…})` from
//!   `list`/`observe`; also set [`Self::last_unavailable_reason`] |
//! | Multi-remote, no origin | Observe succeeds; `remote_url_hash` may be `None` |
//!
//! Callers **must** check [`Self::last_unavailable_reason`] and
//! [`Self::last_list_truncated`] immediately after every `list`. Treating bare
//! `Ok([])` as “no evidence, nothing wrong” reintroduces deferred #22.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_git::{GitError, GitMetadata, collect_metadata};

use crate::canonicalize_git_metadata;
use crate::connector::{
    Connector, ConnectorContext, ConnectorError, ObservePayload, Preview, SourceHandle,
    WriteProposal, WriteProposalInput,
};
use crate::manifest::{
    ConnectorManifest, ConnectorOperations, ConnectorTrustLabel, CredentialDeclaration,
    FreshnessMechanism, MANIFEST_SCHEMA_VERSION, SandboxMode, ScopeClass,
};

/// Stable connector id.
pub const GIT_CONNECTOR_ID: &str = "builtin.git";

/// Default cap on git-related handles returned by `list` (repo root is one handle).
pub const DEFAULT_GIT_MAX_HANDLES: usize = 16;

/// Default collect timeout in milliseconds (matches [`ai_brains_git::DEFAULT_GIT_TIMEOUT_MS`]).
///
/// Documented on options for callers; production `collect_metadata` already applies
/// this deadline inside `ai-brains-git`. A future `collect_metadata_timeout` may
/// plumb [`GitConnectorOptions::collect_timeout_ms`] through without changing the
/// connector public shape.
pub const DEFAULT_GIT_COLLECT_TIMEOUT_MS: u64 = 5_000;

/// Soft-empty reason when the configured root is not inside a git repository.
pub const REASON_NOT_A_REPOSITORY: &str = "not_a_repository";

/// Options for [`GitConnector`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitConnectorOptions {
    /// Preferred collect deadline in ms (documented; git crate default is 5000).
    ///
    /// Currently informational until `ai-brains-git` exposes a timeout-parameterized
    /// collect API. Production collect already uses the 5s default.
    pub collect_timeout_ms: u64,
    /// Max handles from `list` (default [`DEFAULT_GIT_MAX_HANDLES`]).
    pub max_handles: usize,
}

impl Default for GitConnectorOptions {
    fn default() -> Self {
        Self {
            collect_timeout_ms: DEFAULT_GIT_COLLECT_TIMEOUT_MS,
            max_handles: DEFAULT_GIT_MAX_HANDLES,
        }
    }
}

/// Built-in git repository connector.
///
/// Open any directory. When the path is inside a git work tree, `list` emits one
/// [`SourceKind::GitRepository`] handle. When it is not a repository, `list`
/// returns an empty vec **and** sets [`Self::last_unavailable_reason`].
pub struct GitConnector {
    root: PathBuf,
    options: GitConnectorOptions,
    manifest: ConnectorManifest,
    last_list_truncated: AtomicBool,
    last_unavailable_reason: Mutex<Option<String>>,
}

impl GitConnector {
    /// Open a repository / working-tree root. Root must exist and be a directory.
    ///
    /// # Errors
    ///
    /// Returns [`ConnectorError::Internal`] when root is missing or not a directory.
    pub fn open(
        root: impl AsRef<Path>,
        options: GitConnectorOptions,
    ) -> Result<Self, ConnectorError> {
        let root = root.as_ref();
        let meta = std::fs::symlink_metadata(root).map_err(|e| ConnectorError::Internal {
            detail: format!("open git root {}: {e}", root.display()),
        })?;
        if !meta.is_dir() {
            return Err(ConnectorError::Internal {
                detail: format!("git root is not a directory: {}", root.display()),
            });
        }

        // Prefer absolute/canonical root for stable identity.
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let manifest = ConnectorManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: GIT_CONNECTOR_ID.into(),
            display_name: "Git Repository".into(),
            connector_version: "0.1.0".into(),
            source_kinds: vec![SourceKind::GitRepository],
            operations: ConnectorOperations {
                list: true,
                observe: true,
                preview: true,
                propose_write: false,
            },
            scope_affinity: vec![
                ScopeClass::Personal,
                ScopeClass::Repository,
                ScopeClass::Workspace,
            ],
            freshness: FreshnessMechanism::Fingerprint,
            credentials: CredentialDeclaration::PathAccess,
            sandbox: SandboxMode::TrustedBuiltin,
            default_trust: ConnectorTrustLabel::LocalOnly,
            principal_id: None,
        };

        Ok(Self {
            root,
            options,
            manifest,
            last_list_truncated: AtomicBool::new(false),
            last_unavailable_reason: Mutex::new(None),
        })
    }

    /// Whether the last successful `list()` hit `max_handles` and stopped early.
    pub fn last_list_truncated(&self) -> bool {
        self.last_list_truncated.load(Ordering::Relaxed)
    }

    /// Soft-empty / hard-failure reason from the last `list` (or observe failure).
    ///
    /// Callers **must** check this after every `list`. `None` means healthy
    /// (including a legitimate empty list only when max_handles is 0 with a
    /// live repo — rare). Soft empty not-a-repo always sets a reason.
    pub fn last_unavailable_reason(&self) -> Option<String> {
        self.last_unavailable_reason
            .lock()
            .map(|g| g.clone())
            .unwrap_or(None)
    }

    /// Configured root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Options used at open.
    pub fn options(&self) -> &GitConnectorOptions {
        &self.options
    }

    fn clear_side_channels(&self) {
        self.last_list_truncated.store(false, Ordering::Relaxed);
        if let Ok(mut g) = self.last_unavailable_reason.lock() {
            *g = None;
        }
    }

    fn set_unavailable(&self, reason: impl Into<String>) {
        if let Ok(mut g) = self.last_unavailable_reason.lock() {
            *g = Some(reason.into());
        }
    }

    fn set_truncated(&self, truncated: bool) {
        self.last_list_truncated.store(truncated, Ordering::Relaxed);
    }

    /// Collect metadata; map git hard failures to connector errors.
    fn collect(&self) -> Result<GitMetadata, ConnectorError> {
        // `collect_timeout_ms` is documented on options; production collect uses
        // the git crate default (5s) until a timeout-parameterized API exists.
        let _timeout_ms = self.options.collect_timeout_ms;
        match collect_metadata(&self.root) {
            Ok(meta) => Ok(meta),
            Err(e) => {
                let detail = git_error_reason(&e);
                self.set_unavailable(detail.clone());
                Err(map_git_error(e))
            }
        }
    }

    fn identity_and_locator(&self, scope: &ScopeRef, meta: &GitMetadata) -> (String, String) {
        let path_for_id = meta
            .common_dir
            .as_ref()
            .or(meta.root.as_ref())
            .map(|p| p.as_path())
            .unwrap_or(self.root.as_path());
        let locator = normalize_repo_locator(path_for_id);
        let identity = make_git_identity(scope, &locator);
        (identity, locator)
    }

    fn build_handle(&self, scope: &ScopeRef, meta: &GitMetadata) -> SourceHandle {
        let (identity, locator) = self.identity_and_locator(scope, meta);
        SourceHandle {
            identity,
            kind: SourceKind::GitRepository,
            locator,
        }
    }
}

/// Map a [`GitError`] into [`ConnectorError::Internal`] (hard-failure path).
///
/// Exported for unit tests that cannot force a live hang without wall clock.
pub fn map_git_error(err: GitError) -> ConnectorError {
    ConnectorError::Internal {
        detail: git_error_reason(&err),
    }
}

fn git_error_reason(err: &GitError) -> String {
    match err {
        GitError::Timeout {
            command,
            elapsed_ms,
        } => format!("timeout:{command}:{elapsed_ms}"),
        GitError::CommandFailed { command, message } => {
            format!("command_failed:{command}:{message}")
        }
        GitError::Utf8(e) => format!("utf8:{e}"),
        GitError::Io(e) => format!("io:{e}"),
        GitError::NonAbsolutePath(p) => format!("non_absolute:{}", p.display()),
        GitError::DiffstatParse(s) => format!("diffstat_parse:{s}"),
    }
}

fn normalize_repo_locator(path: &Path) -> String {
    let raw = path.to_string_lossy();
    ai_brains_path::normalize_for_location_compare(&raw)
}

fn scope_key(scope: &ScopeRef) -> String {
    match scope {
        ScopeRef::Personal(id) => format!("Personal:{id}"),
        ScopeRef::Repository(id) => format!("Repository:{id}"),
        ScopeRef::Workspace(id) => format!("Workspace:{id}"),
    }
}

fn make_git_identity(scope: &ScopeRef, locator: &str) -> String {
    format!("{}|GitRepository|{locator}", scope_key(scope))
}

fn preview_from_metadata(meta: &GitMetadata) -> Preview {
    let branch = meta.branch.as_deref().unwrap_or("(detached)");
    let commit_short = meta
        .commit
        .as_deref()
        .map(|c| if c.len() > 12 { &c[..12] } else { c })
        .unwrap_or("(none)");
    let dirty = if meta.is_dirty { "dirty" } else { "clean" };
    let remote = if meta.remote_url_hash.is_some() {
        "remote_hash=present"
    } else {
        "remote_hash=absent"
    };
    // Never include raw remote URLs or credentials.
    let text = format!("branch={branch} commit={commit_short} status={dirty} {remote}");
    Preview {
        text,
        line_start: Some(1),
        line_end: Some(1),
    }
}

impl Connector for GitConnector {
    fn manifest(&self) -> &ConnectorManifest {
        &self.manifest
    }

    fn list(&self, ctx: &ConnectorContext) -> Result<Vec<SourceHandle>, ConnectorError> {
        if !self.manifest.operations.list {
            return Err(ConnectorError::OperationNotSupported { operation: "list" });
        }

        self.clear_side_channels();

        let meta = self.collect()?;
        if !meta.is_repository() {
            self.set_unavailable(REASON_NOT_A_REPOSITORY);
            return Ok(Vec::new());
        }

        let handle = self.build_handle(&ctx.scope, &meta);
        let mut handles = vec![handle];

        let max = self.options.max_handles;
        if handles.len() > max {
            handles.truncate(max);
            self.set_truncated(true);
        } else if max == 0 {
            handles.clear();
            self.set_truncated(true);
        }

        Ok(handles)
    }

    fn observe(
        &self,
        ctx: &ConnectorContext,
        handle: &SourceHandle,
    ) -> Result<ObservePayload, ConnectorError> {
        if !self.manifest.operations.observe {
            return Err(ConnectorError::OperationNotSupported {
                operation: "observe",
            });
        }
        if handle.kind != SourceKind::GitRepository {
            return Err(ConnectorError::UndeclaredSourceKind {
                kind: format!("{:?}", handle.kind),
            });
        }

        let meta = self.collect()?;
        if !meta.is_repository() {
            self.set_unavailable(REASON_NOT_A_REPOSITORY);
            return Err(ConnectorError::Internal {
                detail: REASON_NOT_A_REPOSITORY.into(),
            });
        }

        let built = self.build_handle(&ctx.scope, &meta);
        // Locator mismatch (stale handle from another root) → not found.
        if !handle.locator.is_empty() && handle.locator != built.locator {
            // Still allow observe when caller used empty or root-relative locator
            // equal to our root normalization alternate form — prefer exact match.
            let alt = normalize_repo_locator(&self.root);
            if handle.locator != alt && handle.locator != built.locator {
                return Err(ConnectorError::HandleNotFound {
                    locator: handle.locator.clone(),
                });
            }
        }

        let content = canonicalize_git_metadata(&meta);
        Ok(ObservePayload {
            handle: built.clone(),
            content,
            identity: built.identity,
        })
    }

    fn preview(
        &self,
        ctx: &ConnectorContext,
        handle: &SourceHandle,
    ) -> Result<Preview, ConnectorError> {
        if !self.manifest.operations.preview {
            return Err(ConnectorError::OperationNotSupported {
                operation: "preview",
            });
        }
        if handle.kind != SourceKind::GitRepository {
            return Err(ConnectorError::UndeclaredSourceKind {
                kind: format!("{:?}", handle.kind),
            });
        }

        // Use observe path for consistency of unavailability signaling, but
        // build human summary from metadata directly (no secrets).
        let meta = self.collect()?;
        if !meta.is_repository() {
            self.set_unavailable(REASON_NOT_A_REPOSITORY);
            return Err(ConnectorError::Internal {
                detail: REASON_NOT_A_REPOSITORY.into(),
            });
        }

        // Validate handle kind/locator via observe identity path.
        let _ = self.observe(ctx, handle)?;
        Ok(preview_from_metadata(&meta))
    }

    fn propose_write(
        &self,
        _ctx: &ConnectorContext,
        _proposal: &WriteProposalInput,
    ) -> Result<WriteProposal, ConnectorError> {
        Err(ConnectorError::OperationNotSupported {
            operation: "propose_write",
        })
    }
}

// --- ListSideChannels for refresh_bounded ---

impl crate::refresh::ListSideChannels for GitConnector {
    fn last_list_truncated(&self) -> bool {
        GitConnector::last_list_truncated(self)
    }

    fn last_unavailable_reason(&self) -> Option<String> {
        GitConnector::last_unavailable_reason(self)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod unit_tests {
    use super::*;

    #[test]
    fn map_git_error__timeout__internal_with_timeout_prefix() {
        let err = GitError::Timeout {
            command: "git status".into(),
            elapsed_ms: 5000,
        };
        let mapped = map_git_error(err);
        match mapped {
            ConnectorError::Internal { detail } => {
                assert!(detail.starts_with("timeout:"), "{detail}");
                assert!(detail.contains("git status"), "{detail}");
                assert!(detail.contains("5000"), "{detail}");
            }
            other => panic!("expected Internal, got {other:?}"),
        }
    }

    #[test]
    fn map_git_error__command_failed__internal_with_command_failed_prefix() {
        let err = GitError::CommandFailed {
            command: "git rev-parse HEAD".into(),
            message: "fatal: bad object".into(),
        };
        let mapped = map_git_error(err);
        match mapped {
            ConnectorError::Internal { detail } => {
                assert!(detail.starts_with("command_failed:"), "{detail}");
                assert!(detail.contains("fatal: bad object"), "{detail}");
            }
            other => panic!("expected Internal, got {other:?}"),
        }
    }

    #[test]
    fn preview_from_metadata__no_raw_remote_url() {
        let meta = GitMetadata {
            branch: Some("main".into()),
            commit: Some("abcdef0123456789".into()),
            remote_url_hash: Some("deadbeef".into()),
            is_dirty: false,
            ..GitMetadata::default()
        };
        let preview = preview_from_metadata(&meta);
        assert!(!preview.text.contains("http"));
        assert!(!preview.text.contains("git@"));
        assert!(preview.text.contains("remote_hash=present"));
        assert!(preview.text.contains("branch=main"));
        assert!(preview.text.contains("commit=abcdef012345"));
        assert!(preview.text.contains("status=clean"));
    }
}
