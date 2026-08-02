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
//! - **Path safety (T190):** vault list/observe use capability Dir walk +
//!   component nofollow open (ADR-0021). Soft-canonicalize remains non-claim.
//! - **Reserved stems:** blanket Windows device-name refuse (see `vault_fs`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_path::{CapOpenError, open_ambient_vault_dir, open_dir_component_nofollow};
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

        let is_obsidian_vault = detect_obsidian_vault(&root)?;

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
///
/// `NotFound` → `false` (plain markdown root). Other I/O errors propagate so a
/// vault is not silently misclassified when the marker is inaccessible.
pub fn is_obsidian_vault(root: &Path) -> Result<bool, ConnectorError> {
    detect_obsidian_vault(root)
}

fn detect_obsidian_vault(root: &Path) -> Result<bool, ConnectorError> {
    let marker = root.join(".obsidian");
    match std::fs::symlink_metadata(&marker) {
        Ok(m) => Ok(m.is_dir()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(ConnectorError::Internal {
            detail: format!("obsidian marker metadata {}: {e}", marker.display()),
        }),
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
        SourceKind::LegacyAiBrains => "LegacyAiBrains".into(),
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

        // Ambient open of trusted root once; descent uses Dir handles only (F21/F22).
        let root_dir = open_ambient_vault_dir(&self.root).map_err(map_cap_open_err)?;

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
            &root_dir,
            "",
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

fn map_cap_open_err(e: CapOpenError) -> ConnectorError {
    map_vault_err(match e {
        CapOpenError::PathEscape(s) => VaultFsError::PathEscape(s),
        CapOpenError::ReparseRefused(s) => VaultFsError::ReparseRefused(s),
        CapOpenError::Oversized { size, max_bytes } => VaultFsError::Oversized { size, max_bytes },
        CapOpenError::NotFound(s) => VaultFsError::NotFound(s),
        CapOpenError::NotAFile(s) | CapOpenError::NotADir(s) | CapOpenError::Io(s) => {
            VaultFsError::Io(s)
        }
    })
}

/// True when an `Io` message is ENOTDIR / "not a directory" (Unix `O_DIRECTORY` on a file).
///
/// Used by `walk_vault` so non-directory entries can fall through to the file path
/// without treating permission-denied or other real I/O as silent skip.
fn is_enotdir_message(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("not a directory")
        || lower.contains("enotdir")
        || lower.contains("is a file")
}

/// Recursive directory walk via cap-std [`Dir`] handles; **no** `std::fs::read_dir`.
///
/// Stops when `handles.len() == max_files`. Symlink/reparse entries are skipped
/// (list); observe/read uses nofollow open and fails closed.
#[allow(clippy::too_many_arguments)]
fn walk_vault(
    vault_root: &Path,
    current: &cap_std::fs::Dir,
    rel_prefix: &str,
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

    // Dir::entries on the open handle — never ambient std::fs::read_dir (F22).
    let names = ai_brains_path::list_entry_names(current).map_err(map_cap_open_err)?;

    for name in names {
        if handles.len() >= max_files {
            *truncated = true;
            return Ok(());
        }

        // Skip reserved stems on any component name.
        if is_reserved_windows_stem(&name) {
            continue;
        }

        let child_rel = if rel_prefix.is_empty() {
            name.clone()
        } else {
            format!("{rel_prefix}/{name}")
        };

        // Probe type via nofollow open: dir first, then file. Reparse → skip.
        // Real I/O errors (permission denied, etc.) MUST surface — do not silent-skip
        // unreadable subtrees (Codex P2-01). Only ENOTDIR / typed NotADir fall through.
        let try_as_file = match open_dir_component_nofollow(current, &name) {
            Ok(child_dir) => {
                if is_ignored_dir_name(&name) {
                    continue;
                }
                walk_vault(
                    vault_root,
                    &child_dir,
                    &child_rel,
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
            Err(CapOpenError::ReparseRefused(_)) => {
                // Symlink/junction/reparse: skip for list (observe will refuse).
                continue;
            }
            Err(CapOpenError::NotADir(_)) | Err(CapOpenError::NotAFile(_)) => true,
            Err(CapOpenError::NotFound(_)) => {
                // Race: entry vanished during walk.
                continue;
            }
            Err(CapOpenError::Io(ref msg)) if is_enotdir_message(msg) => {
                // Unix O_DIRECTORY on a regular file → ENOTDIR as Io.
                true
            }
            Err(e) => {
                // Permission denied / other real open failures on dirs: surface.
                return Err(map_cap_open_err(e));
            }
        };

        if !try_as_file || !is_markdown_file(&name) {
            continue;
        }

        match ai_brains_path::open_file_component_nofollow(current, &name) {
            Ok(file) => {
                let meta = file.metadata().map_err(|e| ConnectorError::Internal {
                    detail: format!("metadata {child_rel}: {e}"),
                })?;
                if !meta.is_file() || meta.is_symlink() {
                    continue;
                }
                if meta.len() > max_file_bytes {
                    continue;
                }
            }
            Err(CapOpenError::ReparseRefused(_))
            | Err(CapOpenError::NotFound(_))
            | Err(CapOpenError::NotAFile(_)) => continue,
            Err(e) => return Err(map_cap_open_err(e)),
        }

        let locator = normalize_locator(&child_rel);
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
    use ai_brains_core::ids::UserId;
    use ai_brains_core::scope::ScopeRef;
    use uuid::Uuid;

    fn test_scope() -> ScopeRef {
        ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(42)))
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
    fn walk_vault__regular_notes__listed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        std::fs::create_dir_all(&notes).expect("mkdir");
        std::fs::write(notes.join("a.md"), b"# a").expect("write");
        std::fs::write(notes.join("b.md"), b"# b").expect("write");

        let root_dir = open_ambient_vault_dir(dir.path()).expect("open root");
        let scope = test_scope();
        let mut handles = Vec::new();
        let mut truncated = false;
        walk_vault(
            dir.path(),
            &root_dir,
            "",
            0,
            32,
            1_048_576,
            10_000,
            &scope,
            &mut handles,
            &mut truncated,
        )
        .expect("walk");
        assert!(!truncated);
        let locs: Vec<_> = handles.iter().map(|h| h.locator.as_str()).collect();
        assert!(locs.contains(&"notes/a.md"), "{locs:?}");
        assert!(locs.contains(&"notes/b.md"), "{locs:?}");
    }

    #[test]
    fn walk_vault__intermediate_symlink__refused() {
        let dir = tempfile::tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        std::fs::create_dir_all(&notes).expect("mkdir");
        std::fs::write(notes.join("ok.md"), b"# ok").expect("write");

        let outside = tempfile::tempdir().expect("outside");
        std::fs::write(outside.path().join("secret.md"), b"SECRET").expect("outside");
        let link = notes.join("evil");
        let created = {
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(outside.path(), &link).is_ok()
            }
            #[cfg(windows)]
            {
                std::os::windows::fs::symlink_dir(outside.path(), &link).is_ok()
            }
            #[cfg(not(any(unix, windows)))]
            {
                false
            }
        };
        if !created {
            eprintln!(
                "soft-skip: could not create dir symlink/junction (privilege missing). \
                 Intermediate reparse walk refuse covered when privilege available."
            );
            return;
        }

        let root_dir = open_ambient_vault_dir(dir.path()).expect("open root");
        let scope = test_scope();
        let mut handles = Vec::new();
        let mut truncated = false;
        walk_vault(
            dir.path(),
            &root_dir,
            "",
            0,
            32,
            1_048_576,
            10_000,
            &scope,
            &mut handles,
            &mut truncated,
        )
        .expect("walk must not error; symlink dirs are skipped");
        let locs: Vec<_> = handles.iter().map(|h| h.locator.as_str()).collect();
        assert!(locs.contains(&"notes/ok.md"), "{locs:?}");
        assert!(
            !locs.iter().any(|l| l.contains("evil") || l.contains("secret")),
            "must not list through intermediate symlink: {locs:?}"
        );
    }

    #[test]
    fn walk_vault__no_std_fs_read_dir() {
        // Behavioral gate: walk succeeds solely via Dir handles. A source search
        // for `std::fs::read_dir` inside walk_vault is the static complement
        // (F22 / AC12) — this test proves the new path functions end-to-end.
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("root.md"), b"# r").expect("write");
        let root_dir = open_ambient_vault_dir(dir.path()).expect("open");
        let mut handles = Vec::new();
        let mut truncated = false;
        let scope = test_scope();
        walk_vault(
            dir.path(),
            &root_dir,
            "",
            0,
            8,
            4096,
            100,
            &scope,
            &mut handles,
            &mut truncated,
        )
        .expect("walk");
        assert_eq!(handles.len(), 1);
        assert_eq!(handles[0].locator, "root.md");
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
