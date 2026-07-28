//! Default deny-by-default policy matrix (T151 Phase D).
//!
//! Production code **must** use [`DefaultPolicyEvaluator`] (via
//! [`crate::StorePorts::production_policy`] / [`crate::StorePorts::policy_evaluator`]).
//! [`crate::AllowAllPolicy`] is a **test-only** helper and must not be used as the
//! production default.
//!
//! Matrix summary (deny by default):
//! - unknown principal → deny
//! - Agent: **only** Read* + Propose* with grant; hard-deny Approve*, Export, Erase
//! - Human: granted scope capabilities; privacy/route still enforced
//! - Connector: **only** `ReadEvidence` (observe/capture) when source_kind is bound + grant;
//!   hard-deny Propose*/Export/Erase/Approve*
//! - System: empty `bound_capabilities` → grant alone; non-empty bound →
//!   capability must be in bound **and** have a grant (least privilege)
//! - Service / Other: deny unless grant rows say otherwise
//! - LocalOnly (or stricter) + Cloud route → deny even if grant allows
//! - `connector_trust = LocalOnly` + Cloud route → deny (same posture as privacy LocalOnly)
//!
//! Denials are logged with reason codes only — never claim/statement text.
//!
//! ## Model / cloud reason codes (shared vocabulary with core)
//!
//! | Code | Meaning |
//! |------|---------|
//! | `privacy_route_mismatch` | Local-strict privacy (LocalOnly\|NeverInject\|Sealed) + cloud route/provider |
//! | `cloud_extraction_disabled` | `AI_BRAINS_ALLOW_CLOUD_EXTRACTION` off (models registry) |
//! | `no_local_provider` | Local required but none viable (models registry) |
//! | `connector_trust_route_mismatch` | Connector trust LocalOnly + Cloud route |
//!
//! See [`ai_brains_core::model_provenance::reason`] and [`reason`] in this module.
//! Local-strict definition: [`ai_brains_core::privacy::privacy_is_local_strict`].
//!
//! ## Grant reduction
//! Active grants are unique per `(principal, scope, capability)` (partial unique index),
//! so capability-narrowing via [`reduce_grants`] / [`strictest_wins`] is not needed on
//! the allow path. Privacy combine still walks all grants for the scope when evaluating
//! cloud-route blocks. [`reduce_grants`] remains for callers that combine multi-capability
//! grant lists outside the unique-active constraint.

use ai_brains_core::ids::PrincipalId;
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::privacy::{Privacy, privacy_is_local_strict};
use ai_brains_core::scope::{GrantCapability, ScopeGrant, ScopeRef, strictest_wins};

use crate::errors::Result;
use crate::ports::{ConnectorTrust, PolicyContext, PolicyEvaluator, ProcessingRoute};
use crate::sources::scope_identity_key;

/// One policy decision audit row (no content bodies).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDecisionEntry {
    pub principal_id: PrincipalId,
    pub capability: GrantCapability,
    pub scope_key: String,
    pub allowed: bool,
    pub reason_code: String,
    pub privacy: Option<Privacy>,
}

/// Store surface for principal + grant reads and decision logging.
pub trait GrantPrincipalStore {
    fn get_principal(&self, id: PrincipalId) -> Result<Option<Principal>>;
    fn active_grants(&self, principal: PrincipalId, scope: &ScopeRef) -> Result<Vec<ScopeGrant>>;
    /// Log a policy decision without claim/statement text.
    fn log_policy_decision(&self, entry: PolicyDecisionEntry) -> Result<()>;
}

/// Production policy evaluator: SQL grants + pure principal matrix.
pub struct DefaultPolicyEvaluator<S: GrantPrincipalStore> {
    store: S,
}

impl<S: GrantPrincipalStore> DefaultPolicyEvaluator<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    fn decide(
        &self,
        principal_id: PrincipalId,
        capability: GrantCapability,
        scope: &ScopeRef,
        ctx: &PolicyContext,
    ) -> Result<(bool, String)> {
        // Privacy/route gate applies to everyone (including unknown).
        if privacy_blocks_cloud_route(ctx) {
            return Ok((false, reason::PRIVACY_ROUTE_MISMATCH.into()));
        }

        // Connector trust LocalOnly must not ride a Cloud processing route.
        if connector_trust_blocks_cloud_route(ctx) {
            return Ok((false, reason::CONNECTOR_TRUST_ROUTE_MISMATCH.into()));
        }

        let Some(principal) = self.store.get_principal(principal_id)? else {
            return Ok((false, "unknown_principal".into()));
        };

        let grants = self.store.active_grants(principal_id, scope)?;
        let combined_privacy = combine_grant_privacy(&grants, ctx.privacy);

        // Grant privacy may further restrict cloud routing.
        if privacy_is_local_strict(combined_privacy)
            && matches!(ctx.route, Some(ProcessingRoute::Cloud))
        {
            return Ok((false, reason::PRIVACY_ROUTE_MISMATCH.into()));
        }

        let allowed = match principal.kind {
            PrincipalKind::Agent => evaluate_agent(&principal, capability, &grants),
            PrincipalKind::Human => evaluate_human(capability, &grants),
            PrincipalKind::Connector => evaluate_connector(&principal, capability, &grants, ctx),
            PrincipalKind::System => evaluate_system(&principal, capability, &grants),
            PrincipalKind::Service => evaluate_grant_only(capability, &grants),
            PrincipalKind::Other(_) => {
                // Unknown taxonomy — deny unless grant rows say otherwise.
                evaluate_grant_only(capability, &grants)
            }
        };

        let reason = if allowed {
            "allowed".to_string()
        } else {
            match principal.kind {
                PrincipalKind::Agent if is_approve_capability(capability) => {
                    "agent_cannot_approve".into()
                }
                PrincipalKind::Agent if !is_agent_allowed_capability(capability) => {
                    "agent_cap_not_read_or_propose".into()
                }
                PrincipalKind::Connector if !is_connector_observe_capability(capability) => {
                    "connector_cap_not_observe".into()
                }
                PrincipalKind::Connector if !source_kind_bound(&principal, ctx) => {
                    "connector_source_kind_unbound".into()
                }
                PrincipalKind::System
                    if principal.bound_capabilities.is_empty()
                        || !principal.bound_capabilities.contains(&capability) =>
                {
                    "system_cap_not_bound".into()
                }
                _ if !has_capability_grant(capability, &grants) => "missing_grant".into(),
                _ => "denied".into(),
            }
        };

        Ok((allowed, reason))
    }
}

impl<S: GrantPrincipalStore> PolicyEvaluator for DefaultPolicyEvaluator<S> {
    fn allow(
        &self,
        principal: PrincipalId,
        capability: GrantCapability,
        scope: &ScopeRef,
        ctx: &PolicyContext,
    ) -> Result<bool> {
        let (allowed, reason_code) = self.decide(principal, capability, scope, ctx)?;
        // Log denials and (for audit) allows with reason codes only — never claim text.
        let entry = PolicyDecisionEntry {
            principal_id: principal,
            capability,
            scope_key: scope_identity_key(scope),
            allowed,
            reason_code,
            privacy: Some(ctx.privacy),
        };
        self.store.log_policy_decision(entry)?;
        Ok(allowed)
    }
}

fn privacy_blocks_cloud_route(ctx: &PolicyContext) -> bool {
    privacy_is_local_strict(ctx.privacy) && matches!(ctx.route, Some(ProcessingRoute::Cloud))
}

/// LocalOnly connector trust cannot use a Cloud processing route (R1-F7).
fn connector_trust_blocks_cloud_route(ctx: &PolicyContext) -> bool {
    matches!(ctx.connector_trust, Some(ConnectorTrust::LocalOnly))
        && matches!(ctx.route, Some(ProcessingRoute::Cloud))
}

fn combine_grant_privacy(grants: &[ScopeGrant], content_privacy: Privacy) -> Privacy {
    let mut p = content_privacy;
    for g in grants {
        p = p.combine(g.privacy);
    }
    p
}

fn is_approve_capability(capability: GrantCapability) -> bool {
    matches!(
        capability,
        GrantCapability::ApproveConclusion | GrantCapability::ApproveDecision
    )
}

/// Connector observe/capture class: only ReadEvidence (spec §3.5).
fn is_connector_observe_capability(capability: GrantCapability) -> bool {
    matches!(capability, GrantCapability::ReadEvidence)
}

/// Agent matrix: Read* and Propose* only (never Approve*/Export/Erase).
fn is_agent_allowed_capability(capability: GrantCapability) -> bool {
    matches!(
        capability,
        GrantCapability::ReadEvidence
            | GrantCapability::ReadConclusions
            | GrantCapability::ReadDecisions
            | GrantCapability::ProposeConclusion
            | GrantCapability::ProposeDecision
    )
}

fn has_capability_grant(capability: GrantCapability, grants: &[ScopeGrant]) -> bool {
    grants.iter().any(|g| g.capability == capability)
}

/// Reduce grants with [`strictest_wins`] when multiple apply (same scope assumed).
///
/// Not used by the default allow path (unique active grant per capability); available
/// for multi-grant privacy/capability folding by callers.
pub fn reduce_grants(grants: &[ScopeGrant]) -> Option<ScopeGrant> {
    grants.iter().cloned().reduce(|a, b| strictest_wins(&a, &b))
}

fn evaluate_agent(
    _principal: &Principal,
    capability: GrantCapability,
    grants: &[ScopeGrant],
) -> bool {
    // Hard-deny anything outside Read*/Propose* (Approve*, Export, Erase, …).
    if !is_agent_allowed_capability(capability) {
        return false;
    }
    has_capability_grant(capability, grants)
}

fn evaluate_human(capability: GrantCapability, grants: &[ScopeGrant]) -> bool {
    has_capability_grant(capability, grants)
}

fn evaluate_connector(
    principal: &Principal,
    capability: GrantCapability,
    grants: &[ScopeGrant],
    ctx: &PolicyContext,
) -> bool {
    // Spec §3.5: Connector | Only observe/capture for bound_source_kinds + grant; no Approve*.
    // Observe class is ReadEvidence only (not Propose/Export/other reads).
    if !is_connector_observe_capability(capability) {
        return false;
    }
    if !source_kind_bound(principal, ctx) {
        return false;
    }
    has_capability_grant(capability, grants)
}

fn source_kind_bound(principal: &Principal, ctx: &PolicyContext) -> bool {
    let Some(kind) = ctx.source_kind.as_ref() else {
        // Observe checks require an explicit source kind in context.
        return false;
    };
    if principal.bound_source_kinds.is_empty() {
        return false;
    }
    principal.bound_source_kinds.iter().any(|k| k == kind)
}

fn evaluate_system(
    principal: &Principal,
    capability: GrantCapability,
    grants: &[ScopeGrant],
) -> bool {
    let granted = has_capability_grant(capability, grants);
    // Empty bound = deny-all except explicit grants (grant alone OK).
    // Non-empty bound = least privilege: capability ∈ bound ∧ grant present.
    if principal.bound_capabilities.is_empty() {
        return granted;
    }
    principal.bound_capabilities.contains(&capability) && granted
}

fn evaluate_grant_only(capability: GrantCapability, grants: &[ScopeGrant]) -> bool {
    has_capability_grant(capability, grants)
}

/// Reason-code constants for tests and callers.
///
/// `PRIVACY_ROUTE_MISMATCH` is shared with [`ai_brains_core::model_provenance::reason`].
pub mod reason {
    pub const UNKNOWN_PRINCIPAL: &str = "unknown_principal";
    pub const MISSING_GRANT: &str = "missing_grant";
    pub const AGENT_CANNOT_APPROVE: &str = "agent_cannot_approve";
    pub const AGENT_CAP_NOT_READ_OR_PROPOSE: &str = "agent_cap_not_read_or_propose";
    pub const CONNECTOR_SOURCE_KIND_UNBOUND: &str = "connector_source_kind_unbound";
    pub const CONNECTOR_CAP_NOT_OBSERVE: &str = "connector_cap_not_observe";
    pub const SYSTEM_CAP_NOT_BOUND: &str = "system_cap_not_bound";
    /// Same string as [`ai_brains_core::model_provenance::reason::PRIVACY_ROUTE_MISMATCH`].
    pub const PRIVACY_ROUTE_MISMATCH: &str =
        ai_brains_core::model_provenance::reason::PRIVACY_ROUTE_MISMATCH;
    pub const CONNECTOR_TRUST_ROUTE_MISMATCH: &str = "connector_trust_route_mismatch";
    /// Models-registry code (documented for cross-crate alignment; not used by policy matrix).
    pub const CLOUD_EXTRACTION_DISABLED: &str =
        ai_brains_core::model_provenance::reason::CLOUD_EXTRACTION_DISABLED;
    /// Models-registry code (documented for cross-crate alignment; not used by policy matrix).
    pub const NO_LOCAL_PROVIDER: &str = ai_brains_core::model_provenance::reason::NO_LOCAL_PROVIDER;
    pub const ALLOWED: &str = "allowed";
}
