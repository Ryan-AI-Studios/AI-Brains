//! Scope resolution from explicit ids, git identity, ledgerful, and path aliases (T151 Phase C).
//!
//! Signal priority (highest first):
//! 1. Explicit [`ProjectId`] → High
//! 2. Git: normalized remote hash + common-dir aliases → Medium
//! 3. Ledgerful project id from path → Medium
//! 4. Registered path alias → Medium (when paired with stronger signals) / Low alone
//! 5. Cwd heuristic only → Low + warning
//!
//! Resolver **only resolves** — it never creates repository identities (see grants / Phase E).
//! Personal is never selected unless [`ScopeResolveInput::force_personal`] is set.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ai_brains_core::ids::{ProjectId, UserId};
use ai_brains_core::scope::ScopeRef;
use ai_brains_git::GitMetadata;
use ai_brains_path::{
    extract_project_id_from_ledgerful, find_ledgerful_dir, normalize_for_location_compare,
};

use crate::errors::{ControlPlaneError, Result};

/// Confidence of a resolved scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScopeConfidence {
    /// Multiple equally strong candidates; inspect [`ResolvedScope::alternatives`].
    Ambiguous,
    /// Weak / cwd-only heuristic.
    Low,
    /// Git, ledgerful, or registered path alias match.
    Medium,
    /// Explicit project id / forced personal override.
    High,
}

/// One signal that contributed to resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionEvidence {
    /// e.g. `explicit_project_id`, `normalized_remote_hash`, `common_dir`, `path_alias`, `cwd`
    pub signal: String,
    pub detail: String,
}

/// Result of resolving the active scope for a working context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedScope {
    pub scope: ScopeRef,
    pub confidence: ScopeConfidence,
    pub evidence: Vec<ResolutionEvidence>,
    pub warnings: Vec<String>,
    pub alternatives: Vec<ScopeRef>,
}

/// Inputs for [`resolve_scope`].
#[derive(Debug, Clone)]
pub struct ScopeResolveInput {
    pub cwd: PathBuf,
    pub explicit_project_id: Option<ProjectId>,
    /// Only select Personal when this is true (grant/override). Never auto-Personal.
    pub force_personal: bool,
    /// Optional personal user id used when `force_personal` is set.
    pub personal_user_id: Option<UserId>,
    /// Precomputed git metadata (tests / callers that already collected).
    pub git_metadata: Option<GitMetadata>,
}

impl ScopeResolveInput {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self {
            cwd: cwd.into(),
            explicit_project_id: None,
            force_personal: false,
            personal_user_id: None,
            git_metadata: None,
        }
    }
}

/// Query surface for repository identity lookups (pure resolver dependency).
pub trait ScopeIdentityStore {
    fn find_by_remote_hash(&self, hash: &str) -> Result<Option<ProjectId>>;
    fn find_by_path_alias(&self, normalized_path: &str) -> Result<Option<ProjectId>>;
    /// Path-alias lookup for a git common-dir (worktree shared identity).
    fn find_by_common_dir_alias(&self, path: &str) -> Result<Option<ProjectId>>;
    fn find_by_ledgerful_id(&self, id: &str) -> Result<Option<ProjectId>>;
}

#[derive(Debug, Clone)]
struct Candidate {
    project_id: ProjectId,
    confidence: ScopeConfidence,
    evidence: ResolutionEvidence,
}

/// Resolve the governed scope for `input` using identity projections + git metadata.
///
/// When [`ScopeResolveInput::git_metadata`] is `None`, metadata is collected from
/// [`ScopeResolveInput::cwd`] via [`ai_brains_git::collect_metadata`]. Collection
/// errors become soft warnings (resolution continues on path/ledgerful signals).
///
/// Does not register identities. Same normalized remote always maps to the single registered
/// project (if any) — dual registration is rejected at upsert time (Phase E).
pub fn resolve_scope<S: ScopeIdentityStore>(
    input: &ScopeResolveInput,
    store: &S,
) -> Result<ResolvedScope> {
    // 1. Explicit ProjectId → High
    if let Some(project_id) = input.explicit_project_id {
        return Ok(ResolvedScope {
            scope: ScopeRef::Repository(project_id),
            confidence: ScopeConfidence::High,
            evidence: vec![ResolutionEvidence {
                signal: "explicit_project_id".into(),
                detail: project_id.to_string(),
            }],
            warnings: Vec::new(),
            alternatives: Vec::new(),
        });
    }

    let mut candidates: Vec<Candidate> = Vec::new();
    let mut free_evidence: Vec<ResolutionEvidence> = Vec::new();
    let mut early_warnings: Vec<String> = Vec::new();

    // Collect git metadata when not pre-injected (production path).
    let collected_git: Option<GitMetadata> = if input.git_metadata.is_some() {
        None
    } else {
        match ai_brains_git::collect_metadata(&input.cwd) {
            Ok(meta) => Some(meta),
            Err(e) => {
                early_warnings.push(format!("git metadata collection failed: {e}"));
                None
            }
        }
    };
    let meta = input.git_metadata.as_ref().or(collected_git.as_ref());
    let had_git_metadata = meta.is_some();

    // 2. Git: normalized remote hash + common_dir
    if let Some(meta) = meta {
        if let Some(hash) = meta.remote_url_hash.as_deref()
            && !hash.is_empty()
        {
            if let Some(project_id) = store.find_by_remote_hash(hash)? {
                candidates.push(Candidate {
                    project_id,
                    confidence: ScopeConfidence::Medium,
                    evidence: ResolutionEvidence {
                        signal: "normalized_remote_hash".into(),
                        detail: hash.to_string(),
                    },
                });
            }
        } else if !meta.remote_names.is_empty() {
            // Multi-remote / no selected hash: surface remote names as evidence.
            free_evidence.push(ResolutionEvidence {
                signal: "remote_names".into(),
                detail: meta.remote_names.join(","),
            });
            early_warnings.push(format!(
                "git remotes present but no authoritative remote hash selected: {}",
                meta.remote_names.join(", ")
            ));
        }
        if let Some(common_dir) = meta.common_dir.as_ref() {
            let normalized = normalize_path_signal(common_dir);
            if let Some(project_id) = store.find_by_common_dir_alias(&normalized)? {
                candidates.push(Candidate {
                    project_id,
                    confidence: ScopeConfidence::Medium,
                    evidence: ResolutionEvidence {
                        signal: "common_dir".into(),
                        detail: normalized.clone(),
                    },
                });
            }
            // Also try path-alias table for common-dir (aliases may store either form).
            if let Some(project_id) = store.find_by_path_alias(&normalized)? {
                candidates.push(Candidate {
                    project_id,
                    confidence: ScopeConfidence::Medium,
                    evidence: ResolutionEvidence {
                        signal: "common_dir".into(),
                        detail: format!("path_alias:{normalized}"),
                    },
                });
            }
        }
    }

    // 3. Ledgerful project id from path
    if let Some(ledgerful_dir) = find_ledgerful_dir(&input.cwd)
        && let Some(ledgerful_id) = extract_project_id_from_ledgerful(&ledgerful_dir)
        && let Some(project_id) = store.find_by_ledgerful_id(&ledgerful_id)?
    {
        candidates.push(Candidate {
            project_id,
            confidence: ScopeConfidence::Medium,
            evidence: ResolutionEvidence {
                signal: "ledgerful_project_id".into(),
                detail: ledgerful_id,
            },
        });
    }

    // 4–5. Registered path alias for cwd (Medium if git also hit same project elsewhere;
    // Low when this is the only signal — handled at finish).
    let cwd_normalized = normalize_path_signal(&input.cwd);
    if let Some(project_id) = store.find_by_path_alias(&cwd_normalized)? {
        let only_path = candidates.is_empty();
        candidates.push(Candidate {
            project_id,
            confidence: if only_path {
                ScopeConfidence::Low
            } else {
                ScopeConfidence::Medium
            },
            evidence: ResolutionEvidence {
                signal: if only_path {
                    "cwd".into()
                } else {
                    "path_alias".into()
                },
                detail: cwd_normalized.clone(),
            },
        });
    }

    if candidates.is_empty() {
        if input.force_personal {
            let user = input.personal_user_id.unwrap_or_default();
            return Ok(ResolvedScope {
                scope: ScopeRef::Personal(user),
                confidence: ScopeConfidence::High,
                evidence: vec![ResolutionEvidence {
                    signal: "force_personal".into(),
                    detail: user.to_string(),
                }],
                warnings: early_warnings,
                alternatives: Vec::new(),
            });
        }
        // No auto-Personal. Surface Low + warning with no trusted repository scope.
        let mut evidence = free_evidence;
        evidence.push(ResolutionEvidence {
            signal: "cwd".into(),
            detail: cwd_normalized,
        });
        let mut warnings = early_warnings;
        warnings.push(
            "scope unresolved: missing git identity and no registered path alias; Personal not auto-selected"
                .into(),
        );
        return Ok(ResolvedScope {
            // Sentinel: nil project id with Low confidence means "unresolved, not Personal".
            scope: ScopeRef::Repository(ProjectId::from_uuid(uuid::Uuid::nil())),
            confidence: ScopeConfidence::Low,
            evidence,
            warnings,
            alternatives: Vec::new(),
        });
    }

    finish_candidates(candidates, free_evidence, early_warnings, had_git_metadata)
}

fn finish_candidates(
    candidates: Vec<Candidate>,
    free_evidence: Vec<ResolutionEvidence>,
    mut early_warnings: Vec<String>,
    had_git_metadata: bool,
) -> Result<ResolvedScope> {
    // Highest confidence tier among hits.
    let max_conf = candidates
        .iter()
        .map(|c| c.confidence)
        .max()
        .ok_or_else(|| ControlPlaneError::InvalidPayload("empty candidates".into()))?;

    // Group project ids at the top tier (stable order).
    let mut by_project: BTreeMap<String, (ProjectId, Vec<ResolutionEvidence>)> = BTreeMap::new();
    for c in candidates.into_iter().filter(|c| c.confidence == max_conf) {
        let entry = by_project
            .entry(c.project_id.to_string())
            .or_insert_with(|| (c.project_id, Vec::new()));
        entry.1.push(c.evidence);
    }

    let mut projects: Vec<(ProjectId, Vec<ResolutionEvidence>)> =
        by_project.into_values().collect();
    projects.sort_by_key(|a| a.0.to_string());

    if projects.len() > 1 {
        let primary = projects[0].0;
        let mut evidence = projects[0].1.clone();
        evidence.extend(free_evidence);
        let alternatives: Vec<ScopeRef> = projects
            .iter()
            .skip(1)
            .map(|(id, _)| ScopeRef::Repository(*id))
            .collect();
        for (_id, ev) in projects.iter().skip(1) {
            evidence.extend(ev.iter().cloned());
        }
        early_warnings
            .push("multiple projects matched at the same confidence; inspect alternatives".into());
        return Ok(ResolvedScope {
            scope: ScopeRef::Repository(primary),
            confidence: ScopeConfidence::Ambiguous,
            evidence,
            warnings: early_warnings,
            alternatives,
        });
    }

    let (project_id, mut evidence) = projects
        .into_iter()
        .next()
        .ok_or_else(|| ControlPlaneError::InvalidPayload("empty project group".into()))?;
    evidence.extend(free_evidence);

    if max_conf == ScopeConfidence::Low {
        early_warnings.push(
            "resolved via cwd/path alias only; confidence is low — prefer git remote or explicit project id"
                .into(),
        );
    }
    if !had_git_metadata
        && evidence
            .iter()
            .all(|e| e.signal == "cwd" || e.signal == "path_alias")
        && !early_warnings.iter().any(|w: &String| w.contains("cwd"))
    {
        early_warnings.push("missing git metadata; resolved from path signals only".into());
    }

    Ok(ResolvedScope {
        scope: ScopeRef::Repository(project_id),
        confidence: max_conf,
        evidence,
        warnings: early_warnings,
        alternatives: Vec::new(),
    })
}

fn normalize_path_signal(path: &Path) -> String {
    normalize_for_location_compare(&path.to_string_lossy())
}

/// Whether a resolved scope is a trusted (non-sentinel) hit.
pub fn is_authoritative(resolved: &ResolvedScope) -> bool {
    if resolved.confidence == ScopeConfidence::Ambiguous {
        return false;
    }
    if resolved.confidence == ScopeConfidence::Low
        && resolved
            .warnings
            .iter()
            .any(|w| w.contains("Personal not auto-selected"))
    {
        return false;
    }
    match &resolved.scope {
        ScopeRef::Repository(id) => id.as_uuid() != uuid::Uuid::nil(),
        ScopeRef::Workspace(_) | ScopeRef::Personal(_) => true,
    }
}
