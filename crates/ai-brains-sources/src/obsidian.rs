//! Markdown / Obsidian vault connector (`builtin.obsidian`) — T154 / P6.2.
//!
//! # Design locks
//!
//! - **Identity:** vault-relative path (not display title):
//!   `{scope_key}|{kind_label}|{locator}` with `/` separators.
//! - **Frontmatter:** hand-rolled `---` split only (see [`crate::markdown`]).
//! - **Ignores:** `.obsidian`, `.trash`, `.git`, `node_modules` (case-insensitive
//!   name match for Windows friendliness).
//! - **Extensions:** only `.md` / `.markdown` note files are listed.
//! - **`list()` v1 limitation:** single-shot `Vec` capped by `max_files`
//!   (default 10_000); no cursor/pagination on the T153 port. Truncation is
//!   signaled via [`MarkdownObsidianConnector::last_list_truncated`] (connector-local;
//!   does not change the `Connector` trait).
//! - **Write-back:** [`Connector::propose_write`] returns an artifact only —
//!   never mutates the filesystem.
//! - **Path safety residual:** reparse refuse + containment; residual TOCTOU
//!   without `openat`/cap-std is accepted and documented (deferred #12 slice).
//! - **Reserved stems:** blanket Windows device-name refuse (see `vault_fs`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use uuid::Uuid;

use crate::connector::{
    Connector, ConnectorContext, ConnectorError, ObservePayload, Preview, SourceHandle,
    WriteProposal, WriteProposalInput,
};
use crate::manifest::{
    ConnectorManifest, ConnectorOperations, ConnectorTrustLabel, CredentialDeclaration,
    FreshnessMechanism, MANIFEST_SCHEMA_VERSION, SandboxMode, ScopeClass,
};
use crate::markdown::preview_from_markdown;
use crate::vault_fs::{
    VaultFsError, is_reserved_windows_stem, normalize_locator, read_file_under_root,
    refuse_reparse_path, resolve_under_root,
};

/// Stable connector id.
pub const OBSIDIAN_CONNECTOR_ID: &str = "builtin.obsidian";

/// UUID v5 namespace for write-proposal artifact ids (stable, fixed).
const WRITE_PROPOSAL_NAMESPACE: Uuid = Uuid::NAMESPACE_OID;

/// Default walk / read limits.
pub const DEFAULT_MAX_FILE_BYTES: u64 = 1_048_576;
pub const DEFAULT_MAX_FILES: usize = 10_000;
pub const DEFAULT_MAX_DEPTH: usize = 32;
pub const DEFAULT_PREVIEW_CHARS: usize = 4_096;

/// Directory names skipped during vault walks (case-insensitive match).
const IGNORED_DIR_NAMES: &[&str] = &[".obsidian", ".trash", ".git", "node_modules"];

/// Vault walk / observe options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultOptions {
    pub max_file_bytes: u64,
    pub max_files: usize,
    pub max_depth: usize,
    pub preview_chars: usize,
}

impl Default for VaultOptions {
    fn default() -> Self {
        Self {
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
            max_files: DEFAULT_MAX_FILES,
            max_depth: DEFAULT_MAX_DEPTH,
            preview_chars: DEFAULT_PREVIEW_CHARS,
        }
    }
}

/// Built-in Markdown / Obsidian vault connector.
///
/// Open any directory of `.md` notes. When `root/.obsidian` is a directory the
/// root is treated as an Obsidian vault and an [`SourceKind::ObsidianVault`]
/// handle (empty locator) is included alongside note [`SourceKind::File`]
/// handles.
pub struct MarkdownObsidianConnector {
    root: PathBuf,
    options: VaultOptions,
    manifest: ConnectorManifest,
    /// Set when the last `list()` stopped early at `max_files`.
    last_list_truncated: AtomicBool,
    is_obsidian_vault: bool,
}

impl MarkdownObsidianConnector {
    /// Open a vault / notes root. Root must exist and be a directory.
    pub fn open(root: impl AsRef<Path>, options: VaultOptions) -> Result<Self, ConnectorError> {
        let root = root.as_ref();
        let meta = std::fs::symlink_metadata(root).map_err(|e| ConnectorError::Internal {
            detail: format!("open vault root {}: {e}", root.display()),
        })?;
        if !meta.is_dir() {
            return Err(ConnectorError::Internal {
                detail: format!("vault root is not a directory: {}", root.display()),
            });
        }
        refuse_reparse_path(root).map_err(map_vault_err)?;

        // Prefer absolute/canonical root for stable containment.
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let is_obsidian_vault = is_obsidian_vault(&root);

        let manifest = ConnectorManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: OBSIDIAN_CONNECTOR_ID.into(),
            display_name: "Markdown / Obsidian".into(),
            connector_version: "0.1.0".into(),
            // Spec lock: declare both kinds. Vault root handle is only emitted
            // when `.obsidian/` is present; notes are always `File`.
            source_kinds: vec![SourceKind::File, SourceKind::ObsidianVault],
            operations: ConnectorOperations {
                list: true,
                observe: true,
                preview: true,
                propose_write: true,
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
            is_obsidian_vault,
        })
    }

    /// Whether the last successful `list()` hit `max_files` and stopped early.
    pub fn last_list_truncated(&self) -> bool {
        self.last_list_truncated.load(Ordering::Relaxed)
    }

    /// Configured vault / notes root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// True when `root/.obsidian` is a directory.
    pub fn is_obsidian_vault(&self) -> bool {
        self.is_obsidian_vault
    }
}

/// True when `root/.obsidian` exists and is a directory.
pub fn is_obsidian_vault(root: &Path) -> bool {
    let marker = root.join(".obsidian");
    match std::fs::symlink_metadata(&marker) {
        Ok(m) => m.is_dir(),
        Err(_) => false,
    }
}

fn scope_key(scope: &ScopeRef) -> String {
    match scope {
        ScopeRef::Personal(id) => format!("Personal:{id}"),
        ScopeRef::Repository(id) => format!("Repository:{id}"),
        ScopeRef::Workspace(id) => format!("Workspace:{id}"),
    }
}

fn kind_label(kind: &SourceKind) -> String {
    match kind {
        SourceKind::GitRepository => "GitRepository".into(),
        SourceKind::File => "File".into(),
        SourceKind::ObsidianVault => "ObsidianVault".into(),
        SourceKind::Ledgerful => "Ledgerful".into(),
        SourceKind::HermesSession => "HermesSession".into(),
        SourceKind::Honcho => "Honcho".into(),
        SourceKind::Manual => "Manual".into(),
        SourceKind::Other(s) => format!("Other({s})"),
    }
}

fn make_identity(scope: &ScopeRef, kind: &SourceKind, locator: &str) -> String {
    format!("{}|{}|{}", scope_key(scope), kind_label(kind), locator)
}

fn ensure_kind_declared(
    manifest: &ConnectorManifest,
    kind: &SourceKind,
) -> Result<(), ConnectorError> {
    if manifest.source_kinds.iter().any(|k| k == kind) {
        Ok(())
    } else {
        Err(ConnectorError::UndeclaredSourceKind {
            kind: format!("{kind:?}"),
        })
    }
}

fn map_vault_err(err: VaultFsError) -> ConnectorError {
    match err {
        VaultFsError::NotFound(locator) => ConnectorError::HandleNotFound { locator },
        VaultFsError::Oversized { size, max_bytes } => ConnectorError::Internal {
            detail: format!("file oversized: {size} bytes exceeds max {max_bytes}"),
        },
        other => ConnectorError::Internal {
            detail: other.to_string(),
        },
    }
}

fn is_ignored_dir_name(name: &str) -> bool {
    IGNORED_DIR_NAMES
        .iter()
        .any(|n| name.eq_ignore_ascii_case(n))
}

fn is_markdown_file(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown")
}

impl Connector for MarkdownObsidianConnector {
    fn manifest(&self) -> &ConnectorManifest {
        &self.manifest
    }

    fn list(&self, ctx: &ConnectorContext) -> Result<Vec<SourceHandle>, ConnectorError> {
        if !self.manifest.operations.list {
            return Err(ConnectorError::OperationNotSupported { operation: "list" });
        }

        // Reset truncation flag each list.
        self.last_list_truncated.store(false, Ordering::Relaxed);

        refuse_reparse_path(&self.root).map_err(map_vault_err)?;

        let mut handles: Vec<SourceHandle> = Vec::new();
        let max_files = self.options.max_files;

        if self.is_obsidian_vault {
            let kind = SourceKind::ObsidianVault;
            let locator = String::new();
            handles.push(SourceHandle {
                identity: make_identity(&ctx.scope, &kind, &locator),
                kind,
                locator,
            });
        }

        let mut truncated = false;
        walk_vault(
            &self.root,
            &self.root,
            0,
            self.options.max_depth,
            self.options.max_file_bytes,
            max_files,
            &ctx.scope,
            &mut handles,
            &mut truncated,
        )?;

        if truncated {
            self.last_list_truncated.store(true, Ordering::Relaxed);
        }

        // Deterministic order by locator.
        handles.sort_by(|a, b| a.locator.cmp(&b.locator));
        // Hard cap (vault root handle + notes).
        if handles.len() > max_files {
            handles.truncate(max_files);
            self.last_list_truncated.store(true, Ordering::Relaxed);
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
        ensure_kind_declared(&self.manifest, &handle.kind)?;

        match &handle.kind {
            SourceKind::ObsidianVault => {
                // Root vault handle: empty locator, empty content marker.
                if !handle.locator.is_empty() && handle.locator != "." {
                    return Err(ConnectorError::HandleNotFound {
                        locator: handle.locator.clone(),
                    });
                }
                if !self.is_obsidian_vault {
                    return Err(ConnectorError::HandleNotFound {
                        locator: handle.locator.clone(),
                    });
                }
                refuse_reparse_path(&self.root).map_err(map_vault_err)?;
                let locator = String::new();
                let identity = make_identity(&ctx.scope, &SourceKind::ObsidianVault, &locator);
                Ok(ObservePayload {
                    handle: SourceHandle {
                        identity: identity.clone(),
                        kind: SourceKind::ObsidianVault,
                        locator,
                    },
                    content: Vec::new(),
                    identity,
                })
            }
            SourceKind::File => {
                let locator = normalize_locator(&handle.locator);
                let bytes = read_file_under_root(&self.root, &locator, self.options.max_file_bytes)
                    .map_err(map_vault_err)?;
                let identity = make_identity(&ctx.scope, &SourceKind::File, &locator);
                Ok(ObservePayload {
                    handle: SourceHandle {
                        identity: identity.clone(),
                        kind: SourceKind::File,
                        locator,
                    },
                    content: bytes,
                    identity,
                })
            }
            other => Err(ConnectorError::UndeclaredSourceKind {
                kind: format!("{other:?}"),
            }),
        }
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
        ensure_kind_declared(&self.manifest, &handle.kind)?;

        if handle.kind == SourceKind::ObsidianVault {
            return Ok(Preview {
                text: String::new(),
                line_start: Some(1),
                line_end: None,
            });
        }

        let payload = self.observe(ctx, handle)?;
        let text = String::from_utf8_lossy(&payload.content);
        Ok(preview_from_markdown(&text, self.options.preview_chars))
    }

    fn propose_write(
        &self,
        _ctx: &ConnectorContext,
        proposal: &WriteProposalInput,
    ) -> Result<WriteProposal, ConnectorError> {
        if !self.manifest.operations.propose_write {
            return Err(ConnectorError::OperationNotSupported {
                operation: "propose_write",
            });
        }
        ensure_kind_declared(&self.manifest, &proposal.handle.kind)?;

        // Artifact only — zero filesystem writes.
        let content_fp = crate::fingerprint_bytes(proposal.proposed_content.as_bytes());
        let name = format!("{}:{}", proposal.handle.locator, content_fp);
        let artifact_id = Uuid::new_v5(&WRITE_PROPOSAL_NAMESPACE, name.as_bytes()).to_string();

        Ok(WriteProposal {
            handle: proposal.handle.clone(),
            proposed_content: proposal.proposed_content.clone(),
            rationale: proposal.rationale.clone(),
            artifact_id,
        })
    }
}

/// List-walk decision for a single directory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListEntryAction {
    Proceed,
    Skip,
}

/// Classify reparse-check result for list walks.
///
/// Only confirmed reparse/symlink refusal is skipped; other filesystem errors
/// (permission denied, etc.) must surface as [`ConnectorError::Internal`].
fn list_entry_reparse_decision(
    result: Result<(), VaultFsError>,
) -> Result<ListEntryAction, ConnectorError> {
    match result {
        Ok(()) => Ok(ListEntryAction::Proceed),
        Err(VaultFsError::ReparseRefused(_)) => Ok(ListEntryAction::Skip),
        Err(other) => Err(map_vault_err(other)),
    }
}

/// Classify `symlink_metadata` for list walks.
///
/// `NotFound` is skipped (race: entry deleted during walk). All other I/O
/// errors propagate as Internal.
fn list_entry_metadata_decision(
    result: Result<std::fs::Metadata, std::io::Error>,
    path: &Path,
) -> Result<Option<std::fs::Metadata>, ConnectorError> {
    match result {
        Ok(meta) => Ok(Some(meta)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ConnectorError::Internal {
            detail: format!("symlink_metadata {}: {e}", path.display()),
        }),
    }
}

/// Classify `resolve_under_root` for list walks.
///
/// Path policy refusals (escape, reserved stem, absolute, reparse) skip the
/// entry. I/O and other unexpected errors propagate.
fn list_entry_resolve_decision(
    result: Result<PathBuf, VaultFsError>,
) -> Result<ListEntryAction, ConnectorError> {
    match result {
        Ok(_) => Ok(ListEntryAction::Proceed),
        Err(
            VaultFsError::PathEscape(_)
            | VaultFsError::ReservedStem(_)
            | VaultFsError::ReparseRefused(_)
            | VaultFsError::AbsolutePath(_)
            | VaultFsError::EmptyRelative,
        ) => Ok(ListEntryAction::Skip),
        Err(other) => Err(map_vault_err(other)),
    }
}

/// Recursive directory walk; stops when `handles.len() == max_files`.
#[allow(clippy::too_many_arguments)]
fn walk_vault(
    vault_root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    max_file_bytes: u64,
    max_files: usize,
    scope: &ScopeRef,
    handles: &mut Vec<SourceHandle>,
    truncated: &mut bool,
) -> Result<(), ConnectorError> {
    if handles.len() >= max_files {
        *truncated = true;
        return Ok(());
    }
    if depth > max_depth {
        return Ok(());
    }

    // Refuse reparse on the directory we are entering.
    refuse_reparse_path(current).map_err(map_vault_err)?;

    let rd = std::fs::read_dir(current).map_err(|e| ConnectorError::Internal {
        detail: format!("read_dir {}: {e}", current.display()),
    })?;

    // Collect entries for deterministic walk order by name.
    let mut entries: Vec<std::fs::DirEntry> = Vec::new();
    for ent in rd {
        let ent = ent.map_err(|e| ConnectorError::Internal {
            detail: format!("read_dir entry: {e}"),
        })?;
        entries.push(ent);
    }
    entries.sort_by_key(|e| e.file_name());

    for ent in entries {
        if handles.len() >= max_files {
            *truncated = true;
            return Ok(());
        }

        let name_os = ent.file_name();
        let name = name_os.to_string_lossy();
        let path = ent.path();

        // Skip reserved stems on any component name.
        if is_reserved_windows_stem(&name) {
            continue;
        }

        let file_type = match ent.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                return Err(ConnectorError::Internal {
                    detail: format!("file_type {}: {e}", path.display()),
                });
            }
        };

        // Symlink/reparse entries: refuse to follow; skip for list (observe will error).
        if file_type.is_symlink() {
            continue;
        }
        // Windows junctions may not report is_symlink; extra reparse check.
        // Only ReparseRefused is skippable; I/O errors from metadata must surface.
        match list_entry_reparse_decision(refuse_reparse_path(&path))? {
            ListEntryAction::Proceed => {}
            ListEntryAction::Skip => continue,
        }

        if file_type.is_dir() {
            if is_ignored_dir_name(&name) {
                continue;
            }
            walk_vault(
                vault_root,
                &path,
                depth + 1,
                max_depth,
                max_file_bytes,
                max_files,
                scope,
                handles,
                truncated,
            )?;
            continue;
        }

        if !file_type.is_file() {
            continue;
        }
        if !is_markdown_file(&name) {
            continue;
        }

        // Size cap: skip oversized on list. NotFound races skip; other I/O errors surface.
        let Some(meta) = list_entry_metadata_decision(std::fs::symlink_metadata(&path), &path)?
        else {
            continue;
        };
        if meta.len() > max_file_bytes {
            continue;
        }

        let rel = match path.strip_prefix(vault_root) {
            Ok(r) => r,
            Err(_) => {
                // Containment failure — skip (should not appear from a root-bounded walk).
                continue;
            }
        };
        let locator = normalize_locator(&rel.to_string_lossy());
        // Double-check resolve: policy refusals skip; I/O errors surface.
        match list_entry_resolve_decision(resolve_under_root(vault_root, &locator))? {
            ListEntryAction::Proceed => {}
            ListEntryAction::Skip => continue,
        }

        if handles.len() >= max_files {
            *truncated = true;
            return Ok(());
        }

        let kind = SourceKind::File;
        handles.push(SourceHandle {
            identity: make_identity(scope, &kind, &locator),
            kind,
            locator,
        });
    }

    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod unit_tests {
    use super::*;

    #[test]
    fn list_entry_reparse_decision__ok__proceed() {
        assert_eq!(
            list_entry_reparse_decision(Ok(())).expect("ok"),
            ListEntryAction::Proceed
        );
    }

    #[test]
    fn list_entry_reparse_decision__reparse_refused__skip() {
        let err = VaultFsError::ReparseRefused("link".into());
        assert_eq!(
            list_entry_reparse_decision(Err(err)).expect("skip"),
            ListEntryAction::Skip
        );
    }

    #[test]
    fn list_entry_reparse_decision__io_error__propagates_internal() {
        let err = VaultFsError::Io("permission denied".into());
        let mapped = list_entry_reparse_decision(Err(err)).expect_err("must surface");
        assert!(
            matches!(mapped, ConnectorError::Internal { .. }),
            "expected Internal, got {mapped:?}"
        );
        let detail = mapped.to_string().to_ascii_lowercase();
        assert!(
            detail.contains("permission denied") || detail.contains("i/o"),
            "{mapped}"
        );
    }

    #[test]
    fn list_entry_reparse_decision__not_found__propagates_not_swallow() {
        // NotFound from vault_fs is not a reparse skip; list reparse check
        // should not treat it as silent continue.
        let err = VaultFsError::NotFound("gone.md".into());
        let mapped = list_entry_reparse_decision(Err(err)).expect_err("must surface");
        assert!(
            matches!(mapped, ConnectorError::HandleNotFound { .. }),
            "expected HandleNotFound, got {mapped:?}"
        );
    }

    #[test]
    fn list_entry_metadata_decision__ok__some() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("note.md");
        std::fs::write(&file, b"hi").expect("write");
        let meta = std::fs::symlink_metadata(&file).expect("meta");
        let out = list_entry_metadata_decision(Ok(meta), &file).expect("ok");
        assert!(out.is_some());
        assert_eq!(out.expect("some").len(), 2);
    }

    #[test]
    fn list_entry_metadata_decision__not_found__none() {
        let path = Path::new("missing-during-walk.md");
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let out = list_entry_metadata_decision(Err(err), path).expect("skip race");
        assert!(out.is_none());
    }

    #[test]
    fn list_entry_metadata_decision__permission_denied__propagates_internal() {
        let path = Path::new("locked.md");
        let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let mapped = list_entry_metadata_decision(Err(err), path).expect_err("must surface");
        assert!(
            matches!(mapped, ConnectorError::Internal { ref detail } if detail.contains("symlink_metadata")),
            "expected Internal symlink_metadata, got {mapped:?}"
        );
    }

    #[test]
    fn list_entry_resolve_decision__ok__proceed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let resolved = resolve_under_root(dir.path(), "notes/a.md").expect("resolve");
        assert_eq!(
            list_entry_resolve_decision(Ok(resolved)).expect("ok"),
            ListEntryAction::Proceed
        );
    }

    #[test]
    fn list_entry_resolve_decision__path_escape__skip() {
        let err = VaultFsError::PathEscape("..".into());
        assert_eq!(
            list_entry_resolve_decision(Err(err)).expect("skip"),
            ListEntryAction::Skip
        );
    }

    #[test]
    fn list_entry_resolve_decision__reserved_stem__skip() {
        let err = VaultFsError::ReservedStem("aux.md".into());
        assert_eq!(
            list_entry_resolve_decision(Err(err)).expect("skip"),
            ListEntryAction::Skip
        );
    }

    #[test]
    fn list_entry_resolve_decision__io_error__propagates_internal() {
        let err = VaultFsError::Io("disk fault".into());
        let mapped = list_entry_resolve_decision(Err(err)).expect_err("must surface");
        assert!(
            matches!(mapped, ConnectorError::Internal { .. }),
            "expected Internal, got {mapped:?}"
        );
    }
}
