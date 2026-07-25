//! Source observation workflow (T149 Phase D).
//!
//! 1. Principal + scope + policy (`ReadEvidence`)
//! 2. Normalize path locators (File/Git); resolve/register source
//! 3. Fingerprint content (file+identity, git metadata, ledgerful/external)
//! 4. Unchanged → `SourceObserved` only (`changed = false`)
//! 5. Changed → **single** transactional append: version + evidence (+ register if new)
//!    + dependent invalidation events (`ConclusionMarkedStale` / `ReviewItemOpened`)
//! 6. Return ids + `changed`

use std::path::{Path, PathBuf};

use ai_brains_core::ids::{EvidenceId, PrincipalId, SourceId, SourceVersionId, UserId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_core::source::SourceKind;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    EvidenceRecordedPayload, SourceObservedPayload, SourceRegisteredPayload,
    SourceVersionRecordedPayload,
};
use ai_brains_events::{Actor, AggregateType, Envelope, Payload};
use ai_brains_path::normalize_for_location_compare;
use ai_brains_sources::{
    fingerprint_external, fingerprint_file_with_identity, fingerprint_git_path,
    fingerprint_ledgerful,
};

use crate::errors::{ControlPlaneError, Result};
use crate::invalidation::{plan_invalidation_events_for_changed_source, revalidate_matching_stale};
use crate::ports::{
    Clock, EventWriter, Fingerprinter, GovernedQueryStore, PolicyContext, PolicyEvaluator,
};

/// How content is supplied for fingerprinting.
#[derive(Debug, Clone)]
pub enum SourceContent {
    /// Raw file/markdown/external payload bytes (normalized before hash for File kinds).
    Bytes(Vec<u8>),
    /// Filesystem path to a git repository root (uses `ai-brains-sources` git path).
    GitPath(PathBuf),
}

/// Request to observe (register + fingerprint) a source.
#[derive(Debug, Clone)]
pub struct ObserveSourceRequest {
    pub principal: PrincipalId,
    pub scope: ScopeRef,
    pub kind: SourceKind,
    pub display_name: String,
    pub locator: Option<String>,
    pub content: SourceContent,
    pub privacy: Privacy,
    /// When true (default), a fingerprint change triggers dependent invalidation.
    pub run_invalidation: bool,
}

impl Default for ObserveSourceRequest {
    fn default() -> Self {
        Self {
            principal: PrincipalId::new(),
            scope: ScopeRef::Personal(UserId::new()),
            kind: SourceKind::File,
            display_name: String::new(),
            locator: None,
            content: SourceContent::Bytes(Vec::new()),
            privacy: Privacy::LocalOnly,
            run_invalidation: true,
        }
    }
}

/// Result of a successful observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObserveSourceResult {
    pub source_id: SourceId,
    pub version_id: Option<SourceVersionId>,
    pub evidence_id: Option<EvidenceId>,
    pub fingerprint: String,
    pub changed: bool,
}

/// Observe a source: register if needed, fingerprint, record version/evidence on change.
pub fn observe_source<W, Q, C, F, P>(
    writer: &W,
    query: &Q,
    clock: &C,
    fingerprinter: &F,
    policy: &P,
    mut req: ObserveSourceRequest,
) -> Result<ObserveSourceResult>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    F: Fingerprinter,
    P: PolicyEvaluator,
{
    // 1. Policy
    let policy_ctx = PolicyContext {
        privacy: req.privacy,
        connector_trust: None,
        route: None,
        source_kind: Some(req.kind.clone()),
    };
    let allowed = policy.allow(
        req.principal,
        GrantCapability::ReadEvidence,
        &req.scope,
        &policy_ctx,
    )?;
    if !allowed {
        return Err(ControlPlaneError::PolicyDenied(
            "ReadEvidence denied for principal/scope".to_string(),
        ));
    }

    let now = clock.now()?;
    let scope_key = scope_identity_key(&req.scope);
    let actor = Actor::System;

    // Path-normalize File / Git / Obsidian locators before lookup/register/fingerprint.
    req.locator = normalize_locator_for_kind(&req.kind, req.locator.take());
    if let SourceContent::GitPath(path) = &req.content {
        let normalized = normalize_path_locator(&path.to_string_lossy());
        req.content = SourceContent::GitPath(PathBuf::from(normalized));
    }

    // 2. Resolve / register
    let existing = query.find_source(
        &scope_key,
        &req.kind,
        req.locator.as_deref(),
        &req.display_name,
    )?;
    let (source_id, is_new) = match existing {
        Some(id) => (id, false),
        None => (SourceId::new(), true),
    };

    // 3. Fingerprint (includes canonical source identity for file-like kinds)
    let identity = source_identity_string(
        &scope_key,
        &req.kind,
        req.locator.as_deref(),
        &req.display_name,
    );
    let fingerprint = compute_fingerprint(fingerprinter, &req.kind, &req.content, &identity)?;

    // 4. Compare to latest
    let latest = if is_new {
        None
    } else {
        query.latest_source_version(source_id)?
    };

    let unchanged = latest.as_ref().is_some_and(|(_, fp)| fp == &fingerprint);

    if unchanged {
        // Observation metadata only — no new version/evidence.
        let observed = build_event(
            AggregateType::Source,
            source_id.as_uuid(),
            actor,
            req.privacy,
            Payload::SourceObserved(SourceObservedPayload {
                source_id,
                observed_at: now,
                note: Some("unchanged".into()),
            }),
        )?;
        writer.append_events(&[observed])?;

        // Revalidation: same fingerprint clears matching stale conclusions only
        // when the latest stale fact's version/reason matches this source.
        let _cleared = revalidate_matching_stale(
            writer,
            query,
            clock,
            source_id,
            &fingerprint,
            req.principal,
            req.privacy,
        )?;

        return Ok(ObserveSourceResult {
            source_id,
            version_id: latest.map(|(v, _)| v),
            evidence_id: None,
            fingerprint,
            changed: false,
        });
    }

    // 5. Changed (or first observation) — single TX: version + evidence (+ register)
    //    + dependent invalidation events (T149-F1 / R4).
    let version_id = SourceVersionId::new();
    let evidence_id = EvidenceId::new();
    let mut batch: Vec<Envelope> = Vec::new();

    if is_new {
        batch.push(build_event(
            AggregateType::Source,
            source_id.as_uuid(),
            actor.clone(),
            req.privacy,
            Payload::SourceRegistered(SourceRegisteredPayload {
                source_id,
                kind: req.kind.clone(),
                display_name: req.display_name.clone(),
                locator: req.locator.clone(),
                scope: Some(scope_key.clone()),
            }),
        )?);
    }

    batch.push(build_event(
        AggregateType::Source,
        source_id.as_uuid(),
        actor.clone(),
        req.privacy,
        Payload::SourceVersionRecorded(SourceVersionRecordedPayload {
            source_id,
            version_id,
            fingerprint: fingerprint.clone(),
            recorded_at: now,
        }),
    )?);

    batch.push(build_event(
        AggregateType::Evidence,
        evidence_id.as_uuid(),
        actor.clone(),
        req.privacy,
        Payload::EvidenceRecorded(EvidenceRecordedPayload {
            evidence_id,
            source_id,
            source_version_id: Some(version_id),
            fingerprint: Some(fingerprint.clone()),
            model_provenance: None,
            summary: format!("Observed {}", req.display_name),
        }),
    )?);

    batch.push(build_event(
        AggregateType::Source,
        source_id.as_uuid(),
        actor,
        req.privacy,
        Payload::SourceObserved(SourceObservedPayload {
            source_id,
            observed_at: now,
            note: Some("changed".into()),
        }),
    )?);

    // Dependents of prior evidence: plan stale/review into the same batch.
    // First version has no dependents yet; subsequent changes mark them stale.
    // Always emit MarkedStale for the new version (even if already stale) so the
    // invalidation queue records Processed audit rows for this version.
    // When `run_invalidation` is false, no queue rows are created (projection no
    // longer enqueues Pending on SourceVersionRecorded — Codex round2 P2-4).
    if req.run_invalidation && !is_new {
        let (_inv, inv_events) = plan_invalidation_events_for_changed_source(
            query,
            source_id,
            version_id,
            req.principal,
            req.privacy,
        )?;
        batch.extend(inv_events);
    }

    writer.append_events(&batch)?;

    Ok(ObserveSourceResult {
        source_id,
        version_id: Some(version_id),
        evidence_id: Some(evidence_id),
        fingerprint,
        changed: true,
    })
}

fn compute_fingerprint<F: Fingerprinter>(
    fingerprinter: &F,
    kind: &SourceKind,
    content: &SourceContent,
    identity: &str,
) -> Result<String> {
    match kind {
        SourceKind::GitRepository => match content {
            SourceContent::GitPath(path) => fingerprint_git_path(Path::new(path))
                .map_err(|e| ControlPlaneError::Fingerprint(e.to_string())),
            // Canonical bytes already prepared by caller (e.g. tests).
            SourceContent::Bytes(bytes) => fingerprinter.fingerprint(bytes),
        },
        SourceKind::Ledgerful => {
            let bytes = content_as_bytes(content);
            fingerprint_ledgerful(identity, &bytes)
                .map_err(|e| ControlPlaneError::Fingerprint(e.to_string()))
        }
        SourceKind::Manual
        | SourceKind::HermesSession
        | SourceKind::Honcho
        | SourceKind::Other(_) => {
            let bytes = content_as_bytes(content);
            fingerprint_external(identity, &bytes)
                .map_err(|e| ControlPlaneError::Fingerprint(e.to_string()))
        }
        SourceKind::File | SourceKind::ObsidianVault => {
            let bytes = content_as_bytes(content);
            fingerprint_file_with_identity(identity, &bytes)
                .map_err(|e| ControlPlaneError::Fingerprint(e.to_string()))
        }
    }
}

fn content_as_bytes(content: &SourceContent) -> Vec<u8> {
    match content {
        SourceContent::Bytes(b) => b.clone(),
        SourceContent::GitPath(path) => path.to_string_lossy().into_owned().into_bytes(),
    }
}

/// Stable identity string folded into file/external/ledgerful fingerprints.
///
/// Format: `{scope}|{kind}|{locator_or_display_name}`.
pub fn source_identity_string(
    scope_key: &str,
    kind: &SourceKind,
    locator: Option<&str>,
    display_name: &str,
) -> String {
    let kind_label = match kind {
        SourceKind::GitRepository => "GitRepository".to_string(),
        SourceKind::File => "File".to_string(),
        SourceKind::ObsidianVault => "ObsidianVault".to_string(),
        SourceKind::Ledgerful => "Ledgerful".to_string(),
        SourceKind::HermesSession => "HermesSession".to_string(),
        SourceKind::Honcho => "Honcho".to_string(),
        SourceKind::Manual => "Manual".to_string(),
        SourceKind::Other(s) => format!("Other({s})"),
    };
    let loc = locator.unwrap_or(display_name);
    format!("{scope_key}|{kind_label}|{loc}")
}

/// Normalize path-like locators for File / Git / Obsidian kinds.
fn normalize_locator_for_kind(kind: &SourceKind, locator: Option<String>) -> Option<String> {
    let loc = locator?;
    match kind {
        SourceKind::File | SourceKind::GitRepository | SourceKind::ObsidianVault => {
            Some(normalize_path_locator(&loc))
        }
        _ => Some(loc),
    }
}

/// Best-effort path normalization (Windows drive case, slash, UNC, WSL forms).
///
/// Uses [`normalize_for_location_compare`] so missing paths still get stable
/// drive-case / separator normalization.
pub fn normalize_path_locator(locator: &str) -> String {
    normalize_for_location_compare(locator)
}

/// Stable scope identity string stored on `source_projection.scope` and used for
/// unique (scope, kind, locator) resolution.
///
/// Format: `Repository:{project_id}` / `Workspace:{id}` / `Personal:{user_id}`.
pub fn scope_identity_key(scope: &ScopeRef) -> String {
    match scope {
        ScopeRef::Repository(id) => format!("Repository:{id}"),
        ScopeRef::Workspace(id) => format!("Workspace:{id}"),
        ScopeRef::Personal(id) => format!("Personal:{id}"),
    }
}

/// Rehydrate [`ScopeRef`] from a stored scope identity key.
pub fn parse_scope_key(key: &str) -> Result<ScopeRef> {
    use ai_brains_core::ids::{ProjectId, UserId, WorkspaceId};
    use uuid::Uuid;

    let parse_uuid = |rest: &str, kind: &str| -> Result<Uuid> {
        Uuid::parse_str(rest).map_err(|e| {
            ControlPlaneError::InvalidPayload(format!("invalid {kind} id in scope key: {e}"))
        })
    };

    if let Some(rest) = key.strip_prefix("Repository:") {
        Ok(ScopeRef::Repository(ProjectId::from_uuid(parse_uuid(
            rest,
            "Repository",
        )?)))
    } else if let Some(rest) = key.strip_prefix("Workspace:") {
        Ok(ScopeRef::Workspace(WorkspaceId::from_uuid(parse_uuid(
            rest,
            "Workspace",
        )?)))
    } else if let Some(rest) = key.strip_prefix("Personal:") {
        Ok(ScopeRef::Personal(UserId::from_uuid(parse_uuid(
            rest, "Personal",
        )?)))
    } else {
        Err(ControlPlaneError::InvalidPayload(format!(
            "unparseable scope key: {key}"
        )))
    }
}

pub(crate) fn build_event(
    aggregate_type: AggregateType,
    aggregate_id: uuid::Uuid,
    actor: Actor,
    privacy: Privacy,
    payload: Payload,
) -> Result<Envelope> {
    EventBuilder::new(aggregate_type, aggregate_id, actor, privacy)
        .build(payload)
        .map_err(|e| ControlPlaneError::EventAppend(e.to_string()))
}

/// Reject closed valid-time windows whose end is not strictly after start.
pub(crate) fn ensure_valid_time_interval(
    valid_from: time::OffsetDateTime,
    valid_until: Option<time::OffsetDateTime>,
) -> Result<()> {
    if let Some(until) = valid_until
        && until <= valid_from
    {
        return Err(ControlPlaneError::InvalidPayload(
            "valid_until must be strictly after valid_from".into(),
        ));
    }
    Ok(())
}
