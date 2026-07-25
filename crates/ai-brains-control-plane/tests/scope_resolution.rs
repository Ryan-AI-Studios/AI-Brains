#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ai_brains_control_plane::{
    ResolutionEvidence, ResolvedScope, Result, ScopeConfidence, ScopeIdentityStore,
    ScopeResolveInput, is_authoritative, resolve_scope,
};
use ai_brains_core::ids::ProjectId;
use ai_brains_core::scope::ScopeRef;
use ai_brains_git::{GitMetadata, hash_remote_url};
use ai_brains_path::normalize_for_location_compare;
use uuid::Uuid;

/// In-memory identity store for pure resolver tests.
#[derive(Default)]
struct StubIdentityStore {
    by_remote_hash: HashMap<String, ProjectId>,
    by_path_alias: HashMap<String, ProjectId>,
    by_ledgerful: HashMap<String, ProjectId>,
}

impl StubIdentityStore {
    fn register_remote(&mut self, hash: &str, project_id: ProjectId) {
        self.by_remote_hash.insert(hash.to_string(), project_id);
    }

    fn register_path(&mut self, path: &str, project_id: ProjectId) {
        let n = normalize_for_location_compare(path);
        self.by_path_alias.insert(n, project_id);
    }
}

impl ScopeIdentityStore for StubIdentityStore {
    fn find_by_remote_hash(&self, hash: &str) -> Result<Option<ProjectId>> {
        Ok(self.by_remote_hash.get(hash).copied())
    }

    fn find_by_path_alias(&self, normalized_path: &str) -> Result<Option<ProjectId>> {
        Ok(self.by_path_alias.get(normalized_path).copied())
    }

    fn find_by_common_dir_alias(&self, path: &str) -> Result<Option<ProjectId>> {
        self.find_by_path_alias(path)
    }

    fn find_by_ledgerful_id(&self, id: &str) -> Result<Option<ProjectId>> {
        Ok(self.by_ledgerful.get(id).copied())
    }
}

fn project(n: u128) -> ProjectId {
    ProjectId::from_uuid(Uuid::from_u128(n))
}

fn git_with_remote_and_common(hash: &str, common: impl AsRef<Path>) -> GitMetadata {
    GitMetadata {
        root: Some(common.as_ref().to_path_buf()),
        remote_url_hash: Some(hash.to_string()),
        common_dir: Some(common.as_ref().to_path_buf()),
        ..GitMetadata::default()
    }
}

#[test]
fn resolve_scope__explicit_project_id__high_confidence_repository() {
    let store = StubIdentityStore::default();
    let pid = project(1);
    let input = ScopeResolveInput {
        cwd: PathBuf::from(r"C:\dev\unused"),
        explicit_project_id: Some(pid),
        force_personal: false,
        personal_user_id: None,
        git_metadata: None,
    };
    let resolved = resolve_scope(&input, &store).expect("resolve");
    assert_eq!(resolved.scope, ScopeRef::Repository(pid));
    assert_eq!(resolved.confidence, ScopeConfidence::High);
    assert!(
        resolved
            .evidence
            .iter()
            .any(|e| e.signal == "explicit_project_id")
    );
    assert!(resolved.alternatives.is_empty());
    assert!(is_authoritative(&resolved));
}

#[test]
fn resolve_scope__ssh_and_https_same_logical_remote__same_project_id() {
    let mut store = StubIdentityStore::default();
    let pid = project(42);
    // A0: normalize makes SSH and HTTPS share one hash.
    let https = "https://github.com/org/repo.git";
    let ssh = "git@github.com:org/repo.git";
    let hash = hash_remote_url(https).unwrap();
    assert_eq!(hash, hash_remote_url(ssh).unwrap());
    store.register_remote(&hash, pid);

    let for_https = ScopeResolveInput {
        cwd: PathBuf::from(r"C:\work\repo"),
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
        git_metadata: Some(git_with_remote_and_common(&hash, r"C:\work\repo\.git")),
    };
    let for_ssh = ScopeResolveInput {
        cwd: PathBuf::from("/mnt/c/work/repo"),
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
        git_metadata: Some(git_with_remote_and_common(&hash, r"C:\work\repo\.git")),
    };

    let a = resolve_scope(&for_https, &store).expect("https");
    let b = resolve_scope(&for_ssh, &store).expect("ssh");
    assert_eq!(a.scope, ScopeRef::Repository(pid));
    assert_eq!(b.scope, ScopeRef::Repository(pid));
    assert_eq!(a.confidence, ScopeConfidence::Medium);
    assert_eq!(b.confidence, ScopeConfidence::Medium);
    assert!(
        a.evidence
            .iter()
            .any(|e| e.signal == "normalized_remote_hash")
    );
}

#[test]
fn resolve_scope__wsl_and_windows_path_aliases__same_scope() {
    let mut store = StubIdentityStore::default();
    let pid = project(7);
    // Both forms normalize to the same key via ai-brains-path.
    let win = r"C:\Dev\Project";
    let wsl = "/mnt/c/Dev/Project";
    store.register_path(win, pid);
    store.register_path(wsl, pid);

    let win_input = ScopeResolveInput::new(win);
    let wsl_input = ScopeResolveInput::new(wsl);

    let a = resolve_scope(&win_input, &store).expect("win");
    let b = resolve_scope(&wsl_input, &store).expect("wsl");
    assert_eq!(a.scope, ScopeRef::Repository(pid));
    assert_eq!(b.scope, ScopeRef::Repository(pid));
}

#[test]
fn resolve_scope__two_worktrees_same_common_dir__same_candidate() {
    let mut store = StubIdentityStore::default();
    let pid = project(9);
    let common = r"C:\repos\main\.git";
    let hash = hash_remote_url("https://example.com/org/repo.git").unwrap();
    store.register_remote(&hash, pid);
    store.register_path(common, pid);

    let main_cwd = PathBuf::from(r"C:\repos\main");
    let wt_cwd = PathBuf::from(r"C:\repos\worktree-feature");
    let meta_main = git_with_remote_and_common(&hash, common);
    let meta_wt = git_with_remote_and_common(&hash, common);

    let a = resolve_scope(
        &ScopeResolveInput {
            cwd: main_cwd,
            explicit_project_id: None,
            force_personal: false,
            personal_user_id: None,
            git_metadata: Some(meta_main),
        },
        &store,
    )
    .expect("main");
    let b = resolve_scope(
        &ScopeResolveInput {
            cwd: wt_cwd,
            explicit_project_id: None,
            force_personal: false,
            personal_user_id: None,
            git_metadata: Some(meta_wt),
        },
        &store,
    )
    .expect("wt");
    assert_eq!(a.scope, ScopeRef::Repository(pid));
    assert_eq!(b.scope, ScopeRef::Repository(pid));
}

#[test]
fn resolve_scope__two_projects_same_confidence__ambiguous_with_alternatives() {
    let mut store = StubIdentityStore::default();
    let p_a = project(10);
    let p_b = project(11);
    let hash_a = hash_remote_url("https://example.com/a.git").unwrap();
    let hash_b = hash_remote_url("https://example.com/b.git").unwrap();
    store.register_remote(&hash_a, p_a);
    store.register_remote(&hash_b, p_b);

    // Same Medium tier: remote hash → A and common_dir alias → B.
    let common_for_b = r"C:\other\b\.git";
    store.register_path(common_for_b, p_b);

    let meta = GitMetadata {
        root: Some(PathBuf::from(r"C:\work\a")),
        remote_url_hash: Some(hash_a),
        common_dir: Some(PathBuf::from(common_for_b)),
        ..GitMetadata::default()
    };
    let input = ScopeResolveInput {
        cwd: PathBuf::from(r"C:\work\a"),
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
        git_metadata: Some(meta),
    };
    let resolved = resolve_scope(&input, &store).expect("resolve");
    assert_eq!(resolved.confidence, ScopeConfidence::Ambiguous);
    assert!(!resolved.alternatives.is_empty());
    // Primary + alternatives cover both projects.
    let mut ids = vec![match resolved.scope {
        ScopeRef::Repository(id) => id,
        other => panic!("expected repository, got {other:?}"),
    }];
    for alt in &resolved.alternatives {
        if let ScopeRef::Repository(id) = alt {
            ids.push(*id);
        }
    }
    assert!(ids.contains(&p_a));
    assert!(ids.contains(&p_b));
}

#[test]
fn resolve_scope__no_auto_personal_without_force() {
    let store = StubIdentityStore::default();
    let input = ScopeResolveInput::new(r"C:\tmp\nowhere-unregistered");
    let resolved = resolve_scope(&input, &store).expect("resolve");
    assert!(
        !matches!(resolved.scope, ScopeRef::Personal(_)),
        "must not auto-select Personal: {:?}",
        resolved.scope
    );
    assert!(!is_authoritative(&resolved));
}

#[test]
fn resolve_scope__force_personal__high_personal() {
    let store = StubIdentityStore::default();
    let user = ai_brains_core::ids::UserId::from_uuid(Uuid::from_u128(99));
    let input = ScopeResolveInput {
        cwd: PathBuf::from(r"C:\tmp\nowhere"),
        explicit_project_id: None,
        force_personal: true,
        personal_user_id: Some(user),
        git_metadata: None,
    };
    let resolved = resolve_scope(&input, &store).expect("resolve");
    assert_eq!(resolved.scope, ScopeRef::Personal(user));
    assert_eq!(resolved.confidence, ScopeConfidence::High);
}

#[test]
fn resolve_scope__missing_git_only_cwd_path_alias__low_with_warning() {
    let mut store = StubIdentityStore::default();
    let pid = project(3);
    let cwd = r"C:\Dev\OnlyCwd";
    store.register_path(cwd, pid);

    let input = ScopeResolveInput {
        cwd: PathBuf::from(cwd),
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
        git_metadata: None,
    };
    let resolved = resolve_scope(&input, &store).expect("resolve");
    assert_eq!(resolved.scope, ScopeRef::Repository(pid));
    assert_eq!(resolved.confidence, ScopeConfidence::Low);
    assert!(
        !resolved.warnings.is_empty(),
        "expected warning for cwd-only resolution"
    );
}

#[test]
fn resolve_scope__missing_git_only_cwd_no_alias__low_warning_not_personal() {
    let store = StubIdentityStore::default();
    let input = ScopeResolveInput {
        cwd: PathBuf::from(r"C:\Dev\NoAlias"),
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
        git_metadata: None,
    };
    let resolved = resolve_scope(&input, &store).expect("resolve");
    assert_eq!(resolved.confidence, ScopeConfidence::Low);
    assert!(!resolved.warnings.is_empty());
    assert!(!matches!(resolved.scope, ScopeRef::Personal(_)));
    assert!(!is_authoritative(&resolved));
}

#[test]
fn resolve_scope__second_resolve_same_remote__does_not_invent_second_identity() {
    let mut store = StubIdentityStore::default();
    let pid = project(55);
    let hash = hash_remote_url("https://github.com/org/once.git").unwrap();
    // Registered once only — resolver never creates rows.
    store.register_remote(&hash, pid);

    let meta = git_with_remote_and_common(&hash, r"C:\once\.git");
    let input = ScopeResolveInput {
        cwd: PathBuf::from(r"C:\once"),
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
        git_metadata: Some(meta.clone()),
    };
    let first = resolve_scope(&input, &store).expect("first");
    let second = resolve_scope(&input, &store).expect("second");
    assert_eq!(first.scope, second.scope);
    assert_eq!(first.scope, ScopeRef::Repository(pid));
    // Store still has a single mapping.
    assert_eq!(store.by_remote_hash.len(), 1);
    assert_eq!(store.find_by_remote_hash(&hash).unwrap(), Some(pid));
}

#[test]
fn resolve_scope__evidence_fields_are_structured() {
    let mut store = StubIdentityStore::default();
    let pid = project(1);
    let hash = hash_remote_url("https://example.com/x.git").unwrap();
    store.register_remote(&hash, pid);
    let input = ScopeResolveInput {
        cwd: PathBuf::from(r"C:\x"),
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
        git_metadata: Some(git_with_remote_and_common(&hash, r"C:\x\.git")),
    };
    let ResolvedScope { evidence, .. } = resolve_scope(&input, &store).unwrap();
    assert!(
        evidence
            .iter()
            .any(|e: &ResolutionEvidence| !e.signal.is_empty())
    );
}

/// Production path: no pre-injected GitMetadata — resolver collects from cwd.
#[test]
fn resolve_scope__real_git_repo_collects_metadata_by_remote_hash() {
    use std::process::Command;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-brains-scope-resolve-{nanos}"));
    std::fs::create_dir_all(&root).expect("temp dir");

    let run = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    };
    run(&["init"]);
    run(&["config", "user.name", "AI Brains Test"]);
    run(&["config", "user.email", "tests@example.com"]);
    run(&[
        "remote",
        "add",
        "origin",
        "https://github.com/org/scope-collect.git",
    ]);

    let hash = hash_remote_url("https://github.com/org/scope-collect.git").unwrap();
    let pid = project(99);
    let mut store = StubIdentityStore::default();
    store.register_remote(&hash, pid);

    // No git_metadata injection — production collection path.
    let input = ScopeResolveInput::new(&root);
    let resolved = resolve_scope(&input, &store).expect("resolve via collected git");
    assert_eq!(resolved.scope, ScopeRef::Repository(pid));
    assert_eq!(resolved.confidence, ScopeConfidence::Medium);
    assert!(
        resolved
            .evidence
            .iter()
            .any(|e| e.signal == "normalized_remote_hash" && e.detail == hash),
        "expected remote-hash evidence, got {:?}",
        resolved.evidence
    );
    assert!(is_authoritative(&resolved));

    let _ = std::fs::remove_dir_all(&root);
}
