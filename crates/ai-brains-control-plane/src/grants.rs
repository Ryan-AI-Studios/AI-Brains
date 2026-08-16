//! Principal / grant / workspace / repository-identity commands (T151 Phase E).
//!
//! All state changes append via [`EventWriter`]. Repository identity and path
//! aliases are event-sourced (`RepositoryIdentityRegistered` /
//! `RepositoryPathAliasAdded` / `RepositoryPathAliasRemoved`) so
//! `rebuild_projections` rehydrates them.

use ai_brains_core::ids::{GrantId, PrincipalId, ProjectId, WorkspaceId};
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_core::source::SourceKind;
use ai_brains_events::payload::{
    PrincipalRegisteredPayload, RepositoryIdentityRegisteredPayload,
    RepositoryJoinedWorkspacePayload, RepositoryPathAliasAddedPayload,
    RepositoryPathAliasRemovedPayload, ScopeGrantIssuedPayload, ScopeGrantRevokedPayload,
    WorkspaceRegisteredPayload,
};
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_git::hash_remote_url;
use ai_brains_path::normalize_for_location_compare;

use crate::errors::{ControlPlaneError, Result};
use crate::ports::{Clock, EventWriter};
use crate::scope_resolver::ScopeIdentityStore;
use crate::sources::build_event;

/// Register (or refresh) a principal shell identity.
pub fn register_principal<W: EventWriter, C: Clock>(
    writer: &W,
    clock: &C,
    principal: &Principal,
) -> Result<()> {
    let _now = clock.now()?;
    let kind = principal_kind_label(&principal.kind);
    let event = build_event(
        AggregateType::Principal,
        principal.id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
        Payload::PrincipalRegistered(PrincipalRegisteredPayload {
            principal_id: principal.id,
            kind,
            display_name: principal.display_name.clone(),
            bound_source_kinds: principal.bound_source_kinds.clone(),
            bound_capabilities: principal.bound_capabilities.clone(),
        }),
    )?;
    writer.append_events(&[event])
}

/// Issue a scope grant for a principal. Returns the new grant id.
///
/// `privacy` is persisted on the grant event/projection and participates in
/// policy `strictest_wins` / cloud-route blocking. Prefer
/// [`Privacy::LocalOnly`] unless a broader route is intentionally granted.
pub fn issue_grant<W: EventWriter, C: Clock>(
    writer: &W,
    clock: &C,
    principal_id: PrincipalId,
    scope: ScopeRef,
    capability: GrantCapability,
    privacy: Privacy,
) -> Result<GrantId> {
    let _now = clock.now()?;
    let grant_id = GrantId::new();
    let event = build_event(
        AggregateType::Grant,
        grant_id.as_uuid(),
        Actor::System,
        privacy,
        Payload::ScopeGrantIssued(ScopeGrantIssuedPayload {
            grant_id,
            principal_id,
            scope,
            capability,
            privacy,
        }),
    )?;
    writer.append_events(&[event])?;
    Ok(grant_id)
}

/// Revoke a previously issued grant.
pub fn revoke_grant<W: EventWriter, C: Clock>(
    writer: &W,
    clock: &C,
    grant_id: GrantId,
    reason: impl Into<String>,
) -> Result<()> {
    let _now = clock.now()?;
    let event = build_event(
        AggregateType::Grant,
        grant_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
        Payload::ScopeGrantRevoked(ScopeGrantRevokedPayload {
            grant_id,
            reason: reason.into(),
        }),
    )?;
    writer.append_events(&[event])
}

/// Register a workspace aggregate.
pub fn register_workspace<W: EventWriter, C: Clock>(
    writer: &W,
    clock: &C,
    workspace_id: WorkspaceId,
    name: impl Into<String>,
) -> Result<()> {
    let _now = clock.now()?;
    let event = build_event(
        AggregateType::Workspace,
        workspace_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
        Payload::WorkspaceRegistered(WorkspaceRegisteredPayload {
            workspace_id,
            name: name.into(),
        }),
    )?;
    writer.append_events(&[event])
}

/// Join a repository project into a workspace.
pub fn join_repository<W: EventWriter, C: Clock>(
    writer: &W,
    clock: &C,
    workspace_id: WorkspaceId,
    project_id: ProjectId,
) -> Result<()> {
    let _now = clock.now()?;
    let event = build_event(
        AggregateType::Workspace,
        workspace_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
        Payload::RepositoryJoinedWorkspace(RepositoryJoinedWorkspacePayload {
            workspace_id,
            project_id,
        }),
    )?;
    writer.append_events(&[event])
}

/// Upsert repository identity by **normalized** remote URL hash (event-sourced).
///
/// If `remote_url` is provided, it is hashed via [`hash_remote_url`] (A0 normalize).
/// Empty normalized remotes are rejected (no shared bogus hash).
/// If `remote_url_hash` is provided, it is used as-is (must already be normalized hash).
///
/// When the hash already maps to a different `project_id` and `force` is false → error.
/// When `force` is true, the event carries `force` so the projection clears the prior binding.
pub fn upsert_repository_identity<W: EventWriter, I: ScopeIdentityStore>(
    writer: &W,
    identity: &I,
    project_id: ProjectId,
    remote_url_or_hash: RemoteIdentityKey,
    force: bool,
) -> Result<()> {
    let hash = match remote_url_or_hash {
        RemoteIdentityKey::RawUrl(url) => {
            let Some(h) = hash_remote_url(&url) else {
                return Err(ControlPlaneError::InvalidPayload(
                    "remote url normalized to empty; refuse identity key".into(),
                ));
            };
            h
        }
        RemoteIdentityKey::NormalizedHash(h) => {
            if h.trim().is_empty() {
                return Err(ControlPlaneError::InvalidPayload(
                    "remote_url_hash must be non-empty".into(),
                ));
            }
            h
        }
    };

    if let Some(existing_id) = identity.find_by_remote_hash(&hash)?
        && existing_id != project_id
        && !force
    {
        return Err(ControlPlaneError::IdentityConflict(format!(
            "remote hash already bound to project {existing_id}; refuse dual identity without force"
        )));
    }

    let event = build_event(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
        Payload::RepositoryIdentityRegistered(RepositoryIdentityRegisteredPayload {
            project_id,
            remote_url_hash: Some(hash),
            ledgerful_project_id: None,
            force,
        }),
    )?;
    writer.append_events(&[event])
}

/// Bind a ledgerful project id string to a repository identity (event-sourced).
pub fn set_repository_ledgerful_id<W: EventWriter>(
    writer: &W,
    project_id: ProjectId,
    ledgerful_project_id: &str,
) -> Result<()> {
    if ledgerful_project_id.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "ledgerful_project_id must be non-empty".into(),
        ));
    }
    let event = build_event(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
        Payload::RepositoryIdentityRegistered(RepositoryIdentityRegisteredPayload {
            project_id,
            remote_url_hash: None,
            ledgerful_project_id: Some(ledgerful_project_id.to_string()),
            force: false,
        }),
    )?;
    writer.append_events(&[event])
}

/// Register a normalized path alias for a project (Windows + WSL forms are both OK).
pub fn register_path_alias<W: EventWriter>(
    writer: &W,
    path: &str,
    project_id: ProjectId,
) -> Result<()> {
    let normalized = normalize_for_location_compare(path);
    if normalized.is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "path alias normalized to empty".into(),
        ));
    }
    let event = build_event(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
        Payload::RepositoryPathAliasAdded(RepositoryPathAliasAddedPayload {
            project_id,
            normalized_path: normalized,
        }),
    )?;
    writer.append_events(&[event])
}

/// Unregister a normalized path alias for a project (Windows + WSL forms are both OK).
///
/// Appends compensating `RepositoryPathAliasRemoved`. Projection delete is owner-scoped.
pub fn unregister_path_alias<W: EventWriter>(
    writer: &W,
    path: &str,
    project_id: ProjectId,
) -> Result<()> {
    let normalized = normalize_for_location_compare(path);
    if normalized.is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "path alias normalized to empty".into(),
        ));
    }
    let event = build_event(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
        Payload::RepositoryPathAliasRemoved(RepositoryPathAliasRemovedPayload {
            project_id,
            normalized_path: normalized,
        }),
    )?;
    writer.append_events(&[event])
}

/// Input for identity upsert: raw remote URL (will normalize+hash) or precomputed hash.
#[derive(Debug, Clone)]
pub enum RemoteIdentityKey {
    RawUrl(String),
    NormalizedHash(String),
}

/// PascalCase wire labels for [`PrincipalKind`] (event + `principal_projection.kind`).
///
/// Round-trip: `principal_kind_label` → event/projection → [`parse_principal_kind`].
fn principal_kind_label(kind: &PrincipalKind) -> String {
    match kind {
        PrincipalKind::Human => "Human".into(),
        PrincipalKind::Agent => "Agent".into(),
        PrincipalKind::Connector => "Connector".into(),
        PrincipalKind::System => "System".into(),
        PrincipalKind::Service => "Service".into(),
        PrincipalKind::Other(s) => format!("Other:{s}"),
    }
}

/// Parse a stored principal kind string into [`PrincipalKind`].
///
/// Accepts PascalCase labels (`Human`, `Agent`, `Connector`, `System`,
/// `Service`) and `Other:{label}`. Unknown values become `Other(raw)`.
pub(crate) fn parse_principal_kind(label: &str) -> PrincipalKind {
    match label {
        "Human" => PrincipalKind::Human,
        "Agent" => PrincipalKind::Agent,
        "Connector" => PrincipalKind::Connector,
        "System" => PrincipalKind::System,
        "Service" => PrincipalKind::Service,
        other if let Some(rest) = other.strip_prefix("Other:") => {
            PrincipalKind::Other(rest.to_string())
        }
        other => PrincipalKind::Other(other.to_string()),
    }
}

pub(crate) fn parse_capability(label: &str) -> Result<GrantCapability> {
    let quoted = format!("\"{label}\"");
    serde_json::from_str(&quoted)
        .map_err(|e| ControlPlaneError::InvalidPayload(format!("unknown capability {label}: {e}")))
}

pub(crate) fn parse_privacy(label: &str) -> Privacy {
    match label {
        "CloudOk" | "Public" => Privacy::CloudOk,
        "LocalOnly" | "ProjectLocal" => Privacy::LocalOnly,
        "NeverInject" | "Private" => Privacy::NeverInject,
        "Sealed" => Privacy::Sealed,
        _ => Privacy::LocalOnly,
    }
}

pub(crate) fn parse_source_kinds_json(json: &str) -> Result<Vec<SourceKind>> {
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(json)
        .map_err(|e| ControlPlaneError::Query(format!("bound_source_kinds: {e}")))
}

pub(crate) fn parse_capabilities_json(json: &str) -> Result<Vec<GrantCapability>> {
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(json)
        .map_err(|e| ControlPlaneError::Query(format!("bound_capabilities: {e}")))
}
