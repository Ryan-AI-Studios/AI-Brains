//! Legacy → governed classification importer (T167 / P9.1).
//!
//! - **L1** Classify-first: `ImportOpts.dry_run` default true; apply needs `confirm`.
//! - **L2** Never opens a live vault; callers supply event stream + ports.
//! - **L3** Domain ids via uuid v5 (`id_from_command`); `plan_hash` is the determinism contract.
//! - **L4** Never emit `ConclusionConfirmed` / `DecisionApproved`.
//! - **L5** Forgotten final status excludes Evidence; cascade `unsupported` + reason codes.
//! - **L6** No content-envelope claims.
//! - **L8/L18** Raw `build_event` only — no `observe_source` / `propose_*` for bulk path.
//! - **L12** Privacy = envelope privacy only; missing → Sealed.
//! - **L15** No plaintext bodies in `plan_hash` / default reports.
//! - **L19** `default_scope`; None + no project → `missing_scope`.
//! - **L20** `LegacyImportApplied` on successful apply only.

use std::collections::{BTreeMap, HashMap, HashSet};

use ai_brains_core::ids::{
    ConclusionId, DecisionId, EvidenceId, MemoryId, PrincipalId, ProjectId, ReviewItemId, SourceId,
};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::review::{ReviewCriticality, ReviewSubjectKind};
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_events::payload::{
    ConclusionProposedPayload, DecisionProposedPayload, EvidenceRecordedPayload,
    LegacyImportAppliedPayload, ReviewItemOpenedPayload, SourceRegisteredPayload,
};
use ai_brains_events::{Actor, AggregateType, Envelope, Payload};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::command_id::id_from_command;
use crate::errors::{ControlPlaneError, Result};
use crate::ports::{Clock, EventWriter, GovernedQueryStore};
use crate::sources::{build_event, scope_identity_key};

// ---------------------------------------------------------------------------
// Stable namespaces (§5.4)
// ---------------------------------------------------------------------------

/// Namespace for vault-global legacy source `SourceId`.
pub const NS_LEGACY_SOURCE: &str = "ai-brains.legacy.source";

/// Namespace for imported `EvidenceId` (prefer memory_id input).
pub const NS_LEGACY_EVIDENCE: &str = "ai-brains.legacy.evidence";

/// Namespace for imported `ConclusionId` (synth event_id).
pub const NS_LEGACY_CONCLUSION: &str = "ai-brains.legacy.conclusion";

/// Namespace for imported `DecisionId` (DecisionRecorded event_id).
pub const NS_LEGACY_DECISION: &str = "ai-brains.legacy.decision";

/// Namespace for imported `ReviewItemId` (same decision event_id).
pub const NS_LEGACY_REVIEW: &str = "ai-brains.legacy.review";

/// Namespace for `LegacyImportApplied` aggregate id.
pub const NS_LEGACY_IMPORT_BATCH: &str = "ai-brains.legacy.import_batch";

/// Fixed name input for the vault-global legacy source.
pub const LEGACY_SOURCE_NAME: &str = "legacy-ai-brains";

/// Display name for [`SourceKind::LegacyAiBrains`].
pub const LEGACY_SOURCE_DISPLAY_NAME: &str = "Legacy AI-Brains";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Classify / plan options (L1, L19).
#[derive(Debug, Clone)]
pub struct ImportOpts {
    /// Default **true** — plan only; apply is a separate call with confirm.
    pub dry_run: bool,
    /// When true, [`plan_report_json`] may include truncated body snippets
    /// (default false / L15). Does **not** truncate apply bodies on the plan.
    pub include_truncated_summaries: bool,
    /// Fallback scope when an event has no project_id (L19).
    pub default_scope: Option<ScopeRef>,
    /// Proposer / review opener principal for applied envelopes.
    pub principal_id: PrincipalId,
    /// Optional command id for audit / batch aggregate.
    pub command_id: Option<String>,
}

impl Default for ImportOpts {
    fn default() -> Self {
        Self {
            dry_run: true,
            include_truncated_summaries: false,
            default_scope: None,
            principal_id: PrincipalId::from_uuid(Uuid::nil()),
            command_id: None,
        }
    }
}

/// Apply gate (L1).
#[derive(Debug, Clone, Default)]
pub struct ApplyOpts {
    /// Must be **true** to append any events.
    pub confirm: bool,
}

/// Kind tag for an import plan row (stable string for plan_hash).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportActionKind {
    Evidence,
    Conclusion,
    Decision,
    Review,
    Skip,
    Unresolved,
}

impl ImportActionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Evidence => "evidence",
            Self::Conclusion => "conclusion",
            Self::Decision => "decision",
            Self::Review => "review",
            Self::Skip => "skip",
            Self::Unresolved => "unresolved",
        }
    }
}

/// Mechanism for a plan row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportMechanism {
    WouldAppend,
    Skip,
}

impl ImportMechanism {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WouldAppend => "would_append",
            Self::Skip => "skip",
        }
    }
}

/// One planned classification action.
///
/// Body fields (`content`, `title`, `statement`) are always full text for apply
/// **in-process** and are **excluded** from public serde serialization (L15),
/// from [`plan_hash`](compute_plan_hash) (§6.1), and from default
/// [`plan_report_json`]. Truncation is report-only via
/// [`plan_report_json`] with `include_truncated_summaries = true`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportAction {
    pub kind: ImportActionKind,
    pub original_event_id: Uuid,
    pub derived_id: String,
    pub reason_code: String,
    pub mechanism: ImportMechanism,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported: Option<bool>,
    /// Apply body — always full content (not truncated; not in plan_hash).
    /// Skipped on serde serialize so accidental `serde_json::to_string(plan)`
    /// never leaks plaintext (L15 / Codex R1-P1). Still available in-process.
    #[serde(default, skip_serializing)]
    pub content: Option<String>,
    /// Apply-only title for decisions (not in plan_hash / not serialized).
    #[serde(default, skip_serializing)]
    pub title: Option<String>,
    /// Apply-only statement for conclusions/decisions (not in plan_hash / not serialized).
    #[serde(default, skip_serializing)]
    pub statement: Option<String>,
    /// Privacy for applied envelopes (from source envelope; L12).
    pub privacy: Privacy,
    /// Resolved scope identity key when needed for apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_key: Option<String>,
    /// Evidence ids linked to a conclusion (apply).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_ids: Vec<EvidenceId>,
    /// Related decision id for review items (apply).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_decision_id: Option<DecisionId>,
    /// Original memory id when present (provenance; L11).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_memory_id: Option<String>,
    /// Session provenance for session-summary evidence (§5.1 / L11).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Totals for a classify plan.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportTotals {
    pub evidence: u64,
    pub conclusion: u64,
    pub decision: u64,
    pub review: u64,
    pub skipped: u64,
    pub unresolved: u64,
    pub already_governed: u64,
}

/// Full classify plan (audit artifact for dry-run).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPlan {
    pub actions: Vec<ImportAction>,
    pub totals: ImportTotals,
    pub plan_hash: String,
    /// Principal used when applying (from opts).
    pub principal_id: PrincipalId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
    /// Whether classify was requested as dry_run (informational).
    pub dry_run: bool,
}

/// Result of applying a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub applied: u64,
    pub skipped: u64,
    pub already_imported: u64,
    pub unresolved: u64,
    pub source_registered: u64,
    pub plan_hash: String,
    pub legacy_import_applied: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sample_event_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Reason codes (§5.3)
// ---------------------------------------------------------------------------

pub const REASON_UNKNOWN_PAYLOAD: &str = "unknown_payload";
pub const REASON_OUT_OF_MATRIX: &str = "out_of_matrix";
pub const REASON_FORGOTTEN: &str = "forgotten";
pub const REASON_FORGOTTEN_SOURCE: &str = "forgotten_source";
pub const REASON_MISSING_SOURCE: &str = "missing_source";
pub const REASON_EMPTY_CONTENT: &str = "empty_content";
pub const REASON_ALREADY_IMPORTED: &str = "already_imported";
pub const REASON_ALREADY_GOVERNED: &str = "already_governed";
pub const REASON_MISSING_SCOPE: &str = "missing_scope";
pub const REASON_SCOPE_CONTEXT: &str = "scope_context_only";
pub const REASON_IGNORE: &str = "ignore";
pub const REASON_LEGACY_PIN: &str = "legacy_pin";
pub const REASON_LEGACY_SUMMARY: &str = "legacy_summary";
pub const REASON_LEGACY_SYNTH: &str = "legacy_synth";
pub const REASON_LEGACY_DECISION: &str = "legacy_decision";
pub const REASON_LEGACY_REVIEW: &str = "legacy_review";
/// A later `MemoryPinned` for the same `memory_id` supersedes this pin
/// (last-write wins for apply content; discarded pin gets a plan Skip row).
pub const REASON_SUPERSEDED_DUPLICATE_PIN: &str = "superseded_duplicate_pin";

// ---------------------------------------------------------------------------
// Id helpers
// ---------------------------------------------------------------------------

/// Stable vault-global `SourceId` for `LegacyAiBrains`.
pub fn legacy_source_id() -> SourceId {
    SourceId::from_uuid(id_from_command(NS_LEGACY_SOURCE, LEGACY_SOURCE_NAME))
}

/// EvidenceId: prefer memory_id UTF-8 when present; else event_id (§5.4).
pub fn legacy_evidence_id(memory_id: Option<&MemoryId>, event_id: Uuid) -> EvidenceId {
    let input = match memory_id {
        Some(mid) => mid.to_string(),
        None => event_id.to_string(),
    };
    EvidenceId::from_uuid(id_from_command(NS_LEGACY_EVIDENCE, &input))
}

pub fn legacy_conclusion_id(event_id: Uuid) -> ConclusionId {
    ConclusionId::from_uuid(id_from_command(NS_LEGACY_CONCLUSION, &event_id.to_string()))
}

pub fn legacy_decision_id(event_id: Uuid) -> DecisionId {
    DecisionId::from_uuid(id_from_command(NS_LEGACY_DECISION, &event_id.to_string()))
}

pub fn legacy_review_id(decision_event_id: Uuid) -> ReviewItemId {
    ReviewItemId::from_uuid(id_from_command(
        NS_LEGACY_REVIEW,
        &decision_event_id.to_string(),
    ))
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let dig = hasher.finalize();
    dig.iter().map(|b| format!("{b:02x}")).collect()
}

fn truncate_id(id: &str) -> String {
    const MAX: usize = 16;
    if id.chars().count() <= MAX {
        id.to_string()
    } else {
        id.chars().take(MAX).collect::<String>() + "…"
    }
}

// ---------------------------------------------------------------------------
// plan_hash (§6.1)
// ---------------------------------------------------------------------------

/// Canonical hash entry (§6.1) — no body fields.
#[derive(Debug, Serialize)]
struct PlanHashEntry {
    original_event_id: String,
    action_kind: String,
    derived_id: String,
    reason_code: String,
    mechanism: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unsupported: Option<bool>,
}

/// Canonical plan hash: sorted by `(original_event_id, action_kind)` → ActionView.
///
/// Returns [`ControlPlaneError::InvalidPayload`] if canonical serialization fails
/// (must not silently hash empty bytes).
pub fn compute_plan_hash(actions: &[ImportAction]) -> Result<String> {
    let mut map: BTreeMap<(String, String), PlanHashEntry> = BTreeMap::new();
    for a in actions {
        let key = (a.original_event_id.to_string(), a.kind.as_str().to_string());
        map.insert(
            key.clone(),
            PlanHashEntry {
                original_event_id: key.0,
                action_kind: key.1,
                derived_id: a.derived_id.clone(),
                reason_code: a.reason_code.clone(),
                mechanism: a.mechanism.as_str().to_string(),
                source_tag: a.source_tag.clone(),
                unsupported: a.unsupported,
            },
        );
    }
    // BTreeMap iteration is sorted by key; serialize as Vec of entries.
    let entries: Vec<PlanHashEntry> = map.into_values().collect();
    let bytes = serde_json::to_vec(&entries).map_err(|e| {
        ControlPlaneError::InvalidPayload(format!("plan_hash serialization failed: {e}"))
    })?;
    Ok(hex_sha256(&bytes))
}

// ---------------------------------------------------------------------------
// Classify (two-pass §5.0)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryStatus {
    Active,
    Forgotten,
}

struct PendingPin {
    event_id: Uuid,
    memory_id: MemoryId,
    content: String,
    project_id: Option<ProjectId>,
    source_tag: Option<String>,
    privacy: Privacy,
}

struct PendingSummary {
    event_id: Uuid,
    memory_id: MemoryId,
    summary: String,
    project_id: Option<ProjectId>,
    /// Provenance: original session (§5.1).
    session_id: String,
    privacy: Privacy,
}

struct PendingSynth {
    event_id: Uuid,
    memory_id: MemoryId,
    content: String,
    project_id: ProjectId,
    source_memory_ids: Vec<MemoryId>,
    privacy: Privacy,
}

struct PendingDecision {
    event_id: Uuid,
    title: String,
    decision: String,
    context: String,
    project_id: Option<ProjectId>,
    privacy: Privacy,
    /// Legacy payload decision_id (MemoryId) — never cast to DecisionId.
    legacy_memory_decision_id: MemoryId,
}

/// Classify a stream of legacy (and mixed) envelopes into a governed import plan.
///
/// Dry-run still runs both passes and computes `plan_hash` (L1 / §5.0).
pub fn classify_legacy(events: &[Envelope], opts: &ImportOpts) -> Result<ImportPlan> {
    let mut status: HashMap<MemoryId, MemoryStatus> = HashMap::new();
    let mut pins: HashMap<MemoryId, PendingPin> = HashMap::new();
    let mut summaries: Vec<PendingSummary> = Vec::new();
    let mut synths: Vec<PendingSynth> = Vec::new();
    let mut decisions: Vec<PendingDecision> = Vec::new();
    let mut actions: Vec<ImportAction> = Vec::new();
    let mut totals = ImportTotals::default();

    // --- Pass 1a: walk stream for status + collect candidates ---
    for env in events {
        let privacy = env.privacy;
        match &env.payload {
            Payload::MemoryPinned(p) => {
                // Last-write wins for pin content (matches restore/later-event
                // semantics). The discarded earlier pin is not silent-lost:
                // emit a Skip row so L11 provenance covers every original event.
                if let Some(prev) = pins.insert(
                    p.memory_id,
                    PendingPin {
                        event_id: env.event_id,
                        memory_id: p.memory_id,
                        content: p.content.clone(),
                        project_id: p.project_id,
                        source_tag: p.source_tag.clone(),
                        privacy,
                    },
                ) {
                    actions.push(ImportAction {
                        kind: ImportActionKind::Skip,
                        original_event_id: prev.event_id,
                        derived_id: String::new(),
                        reason_code: REASON_SUPERSEDED_DUPLICATE_PIN.into(),
                        mechanism: ImportMechanism::Skip,
                        source_tag: prev.source_tag,
                        unsupported: None,
                        content: None,
                        title: None,
                        statement: None,
                        privacy: prev.privacy,
                        scope_key: None,
                        evidence_ids: Vec::new(),
                        related_decision_id: None,
                        original_memory_id: Some(prev.memory_id.to_string()),
                        session_id: None,
                    });
                    totals.skipped += 1;
                }
                // Pin implies active unless later forgotten.
                status.entry(p.memory_id).or_insert(MemoryStatus::Active);
            }
            Payload::MemoryForgotten(p) => {
                status.insert(p.memory_id, MemoryStatus::Forgotten);
            }
            Payload::MemoryRestored(p) => {
                status.insert(p.memory_id, MemoryStatus::Active);
            }
            Payload::SessionSummaryCreated(p) => {
                summaries.push(PendingSummary {
                    event_id: env.event_id,
                    memory_id: p.memory_id,
                    summary: p.summary.clone(),
                    project_id: p.project_id,
                    session_id: p.session_id.to_string(),
                    privacy,
                });
                status.entry(p.memory_id).or_insert(MemoryStatus::Active);
            }
            Payload::MemorySynthesized(p) => {
                synths.push(PendingSynth {
                    event_id: env.event_id,
                    memory_id: p.memory_id,
                    content: p.content.clone(),
                    project_id: p.project_id,
                    source_memory_ids: p.source_memory_ids.clone(),
                    privacy,
                });
                status.entry(p.memory_id).or_insert(MemoryStatus::Active);
            }
            Payload::DecisionRecorded(p) => {
                decisions.push(PendingDecision {
                    event_id: env.event_id,
                    title: p.title.clone(),
                    decision: p.decision.clone(),
                    context: p.context.clone(),
                    project_id: p.project_id,
                    privacy,
                    legacy_memory_decision_id: p.decision_id,
                });
            }
            Payload::Unknown(_) => {
                actions.push(ImportAction {
                    kind: ImportActionKind::Unresolved,
                    original_event_id: env.event_id,
                    derived_id: String::new(),
                    reason_code: REASON_UNKNOWN_PAYLOAD.into(),
                    mechanism: ImportMechanism::Skip,
                    source_tag: None,
                    unsupported: None,
                    content: None,
                    title: None,
                    statement: None,
                    privacy,
                    scope_key: None,
                    evidence_ids: Vec::new(),
                    related_decision_id: None,
                    original_memory_id: None,
                    session_id: None,
                });
                totals.unresolved += 1;
            }
            Payload::ProjectRegistered(_) | Payload::ProjectAliasAdded(_) => {
                actions.push(skip_action(env.event_id, REASON_SCOPE_CONTEXT, privacy));
                totals.skipped += 1;
            }
            Payload::SessionStarted(_)
            | Payload::SessionCompleted(_)
            | Payload::SessionFailed(_)
            | Payload::UserPromptRecorded(_)
            | Payload::AssistantFinalRecorded(_) => {
                actions.push(skip_action(env.event_id, REASON_IGNORE, privacy));
                totals.skipped += 1;
            }
            // Already governed ECD kinds
            Payload::EvidenceRecorded(_)
            | Payload::EvidenceSuperseded(_)
            | Payload::ConclusionProposed(_)
            | Payload::ConclusionActivated(_)
            | Payload::ConclusionConfirmed(_)
            | Payload::ConclusionMarkedStale(_)
            | Payload::ConclusionDisputed(_)
            | Payload::ConclusionSuperseded(_)
            | Payload::ConclusionRejected(_)
            | Payload::DecisionProposed(_)
            | Payload::DecisionApproved(_)
            | Payload::DecisionSuperseded(_)
            | Payload::DecisionRevoked(_)
            | Payload::SourceRegistered(_)
            | Payload::SourceObserved(_)
            | Payload::SourceVersionRecorded(_)
            | Payload::SourceUnavailable(_)
            | Payload::ReviewItemOpened(_)
            | Payload::ReviewItemResolved(_) => {
                actions.push(skip_action(env.event_id, REASON_ALREADY_GOVERNED, privacy));
                totals.already_governed += 1;
                totals.skipped += 1;
            }
            // Out-of-matrix known kinds
            Payload::ConflictDetected(_)
            | Payload::RecipePromoted(_)
            | Payload::FeedbackMetric(_)
            | Payload::PredictionRecorded(_)
            | Payload::VerifyOutcomeRecorded(_)
            | Payload::IngestGateRejected(_)
            | Payload::SystemInitialized(_)
            | Payload::RecoveryKitCreated(_)
            | Payload::WorkspaceRegistered(_)
            | Payload::RepositoryJoinedWorkspace(_)
            | Payload::ScopeGrantIssued(_)
            | Payload::ScopeGrantRevoked(_)
            | Payload::PrincipalRegistered(_)
            | Payload::BriefingGenerated(_)
            | Payload::QueryTraceRecorded(_)
            | Payload::ContentErasureRequested(_)
            | Payload::ContentErased(_)
            | Payload::ErasureTicketAccepted(_)
            | Payload::RetentionApplied(_)
            | Payload::LegacyImportApplied(_)
            | Payload::ClaimConflictOpened(_)
            | Payload::ClaimConflictResolved(_)
            | Payload::RepositoryIdentityRegistered(_)
            | Payload::RepositoryPathAliasAdded(_)
            | Payload::PolicyDecisionRecorded(_)
            | Payload::DeviceEnrolled(_)
            | Payload::DeviceRevoked(_) => {
                actions.push(skip_action(env.event_id, REASON_OUT_OF_MATRIX, privacy));
                totals.skipped += 1;
            }
        }
    }

    // --- Pass 1b: emit Evidence for eligible pins / summaries ---
    let mut evidence_map: HashMap<MemoryId, EvidenceId> = HashMap::new();

    // Stable order: sort by event_id for deterministic plan action order
    let mut pin_list: Vec<_> = pins.into_values().collect();
    pin_list.sort_by_key(|p| p.event_id);

    for pin in pin_list {
        let final_status = status
            .get(&pin.memory_id)
            .copied()
            .unwrap_or(MemoryStatus::Active);
        if final_status == MemoryStatus::Forgotten {
            actions.push(ImportAction {
                kind: ImportActionKind::Skip,
                original_event_id: pin.event_id,
                derived_id: String::new(),
                reason_code: REASON_FORGOTTEN.into(),
                mechanism: ImportMechanism::Skip,
                source_tag: pin.source_tag.clone(),
                unsupported: None,
                content: None,
                title: None,
                statement: None,
                privacy: pin.privacy,
                scope_key: None,
                evidence_ids: Vec::new(),
                related_decision_id: None,
                original_memory_id: Some(pin.memory_id.to_string()),
                session_id: None,
            });
            totals.skipped += 1;
            continue;
        }

        if pin.content.trim().is_empty() {
            actions.push(ImportAction {
                kind: ImportActionKind::Skip,
                original_event_id: pin.event_id,
                derived_id: String::new(),
                reason_code: REASON_EMPTY_CONTENT.into(),
                mechanism: ImportMechanism::Skip,
                source_tag: pin.source_tag.clone(),
                unsupported: None,
                content: None,
                title: None,
                statement: None,
                privacy: pin.privacy,
                scope_key: None,
                evidence_ids: Vec::new(),
                related_decision_id: None,
                original_memory_id: Some(pin.memory_id.to_string()),
                session_id: None,
            });
            totals.skipped += 1;
            continue;
        }

        let scope_key = match resolve_scope(pin.project_id.as_ref(), opts) {
            Ok(k) => k,
            Err(reason) => {
                actions.push(ImportAction {
                    kind: ImportActionKind::Skip,
                    original_event_id: pin.event_id,
                    derived_id: String::new(),
                    reason_code: reason.into(),
                    mechanism: ImportMechanism::Skip,
                    source_tag: pin.source_tag.clone(),
                    unsupported: None,
                    content: None,
                    title: None,
                    statement: None,
                    privacy: pin.privacy,
                    scope_key: None,
                    evidence_ids: Vec::new(),
                    related_decision_id: None,
                    original_memory_id: Some(pin.memory_id.to_string()),
                    session_id: None,
                });
                totals.skipped += 1;
                continue;
            }
        };

        let eid = legacy_evidence_id(Some(&pin.memory_id), pin.event_id);
        evidence_map.insert(pin.memory_id, eid);

        // Always keep full content for apply (L15 / T167-R1-01). Truncation is
        // report-only via plan_report_json(include_truncated_summaries).
        actions.push(ImportAction {
            kind: ImportActionKind::Evidence,
            original_event_id: pin.event_id,
            derived_id: eid.to_string(),
            reason_code: REASON_LEGACY_PIN.into(),
            mechanism: ImportMechanism::WouldAppend,
            source_tag: pin.source_tag.clone(),
            unsupported: None,
            content: Some(pin.content.clone()),
            title: None,
            statement: None,
            privacy: pin.privacy,
            scope_key: Some(scope_key),
            evidence_ids: Vec::new(),
            related_decision_id: None,
            original_memory_id: Some(pin.memory_id.to_string()),
            session_id: None,
        });
        totals.evidence += 1;
    }

    summaries.sort_by_key(|s| s.event_id);
    for sum in summaries {
        let final_status = status
            .get(&sum.memory_id)
            .copied()
            .unwrap_or(MemoryStatus::Active);
        if final_status == MemoryStatus::Forgotten {
            actions.push(ImportAction {
                kind: ImportActionKind::Skip,
                original_event_id: sum.event_id,
                derived_id: String::new(),
                reason_code: REASON_FORGOTTEN.into(),
                mechanism: ImportMechanism::Skip,
                source_tag: None,
                unsupported: None,
                content: None,
                title: None,
                statement: None,
                privacy: sum.privacy,
                scope_key: None,
                evidence_ids: Vec::new(),
                related_decision_id: None,
                original_memory_id: Some(sum.memory_id.to_string()),
                session_id: Some(sum.session_id.clone()),
            });
            totals.skipped += 1;
            continue;
        }

        if sum.summary.trim().is_empty() {
            actions.push(ImportAction {
                kind: ImportActionKind::Skip,
                original_event_id: sum.event_id,
                derived_id: String::new(),
                reason_code: REASON_EMPTY_CONTENT.into(),
                mechanism: ImportMechanism::Skip,
                source_tag: None,
                unsupported: None,
                content: None,
                title: None,
                statement: None,
                privacy: sum.privacy,
                scope_key: None,
                evidence_ids: Vec::new(),
                related_decision_id: None,
                original_memory_id: Some(sum.memory_id.to_string()),
                session_id: Some(sum.session_id.clone()),
            });
            totals.skipped += 1;
            continue;
        }

        let scope_key = match resolve_scope(sum.project_id.as_ref(), opts) {
            Ok(k) => k,
            Err(reason) => {
                actions.push(ImportAction {
                    kind: ImportActionKind::Skip,
                    original_event_id: sum.event_id,
                    derived_id: String::new(),
                    reason_code: reason.into(),
                    mechanism: ImportMechanism::Skip,
                    source_tag: None,
                    unsupported: None,
                    content: None,
                    title: None,
                    statement: None,
                    privacy: sum.privacy,
                    scope_key: None,
                    evidence_ids: Vec::new(),
                    related_decision_id: None,
                    original_memory_id: Some(sum.memory_id.to_string()),
                    session_id: Some(sum.session_id.clone()),
                });
                totals.skipped += 1;
                continue;
            }
        };

        let eid = legacy_evidence_id(Some(&sum.memory_id), sum.event_id);
        // Collapse: one memory_id → one Evidence action (pin wins over summary).
        if evidence_map.contains_key(&sum.memory_id) {
            actions.push(ImportAction {
                kind: ImportActionKind::Skip,
                original_event_id: sum.event_id,
                derived_id: eid.to_string(),
                reason_code: REASON_ALREADY_IMPORTED.into(),
                mechanism: ImportMechanism::Skip,
                source_tag: None,
                unsupported: None,
                content: None,
                title: None,
                statement: None,
                privacy: sum.privacy,
                scope_key: Some(scope_key),
                evidence_ids: Vec::new(),
                related_decision_id: None,
                original_memory_id: Some(sum.memory_id.to_string()),
                session_id: Some(sum.session_id.clone()),
            });
            totals.skipped += 1;
            continue;
        }
        evidence_map.insert(sum.memory_id, eid);

        actions.push(ImportAction {
            kind: ImportActionKind::Evidence,
            original_event_id: sum.event_id,
            derived_id: eid.to_string(),
            reason_code: REASON_LEGACY_SUMMARY.into(),
            mechanism: ImportMechanism::WouldAppend,
            source_tag: None,
            unsupported: None,
            content: Some(sum.summary.clone()),
            title: None,
            statement: None,
            privacy: sum.privacy,
            scope_key: Some(scope_key),
            evidence_ids: Vec::new(),
            related_decision_id: None,
            original_memory_id: Some(sum.memory_id.to_string()),
            session_id: Some(sum.session_id.clone()),
        });
        totals.evidence += 1;
    }

    // --- Pass 2: conclusions + decisions using evidence map ---
    synths.sort_by_key(|s| s.event_id);
    for synth in synths {
        let final_status = status
            .get(&synth.memory_id)
            .copied()
            .unwrap_or(MemoryStatus::Active);
        if final_status == MemoryStatus::Forgotten {
            actions.push(ImportAction {
                kind: ImportActionKind::Skip,
                original_event_id: synth.event_id,
                derived_id: String::new(),
                reason_code: REASON_FORGOTTEN.into(),
                mechanism: ImportMechanism::Skip,
                source_tag: None,
                unsupported: None,
                content: None,
                title: None,
                statement: None,
                privacy: synth.privacy,
                scope_key: None,
                evidence_ids: Vec::new(),
                related_decision_id: None,
                original_memory_id: Some(synth.memory_id.to_string()),
                session_id: None,
            });
            totals.skipped += 1;
            continue;
        }

        if synth.content.trim().is_empty() {
            actions.push(ImportAction {
                kind: ImportActionKind::Skip,
                original_event_id: synth.event_id,
                derived_id: String::new(),
                reason_code: REASON_EMPTY_CONTENT.into(),
                mechanism: ImportMechanism::Skip,
                source_tag: None,
                unsupported: None,
                content: None,
                title: None,
                statement: None,
                privacy: synth.privacy,
                scope_key: None,
                evidence_ids: Vec::new(),
                related_decision_id: None,
                original_memory_id: Some(synth.memory_id.to_string()),
                session_id: None,
            });
            totals.skipped += 1;
            continue;
        }

        let scope_key = match resolve_scope(Some(&synth.project_id), opts) {
            Ok(k) => k,
            Err(reason) => {
                actions.push(ImportAction {
                    kind: ImportActionKind::Skip,
                    original_event_id: synth.event_id,
                    derived_id: String::new(),
                    reason_code: reason.into(),
                    mechanism: ImportMechanism::Skip,
                    source_tag: None,
                    unsupported: None,
                    content: None,
                    title: None,
                    statement: None,
                    privacy: synth.privacy,
                    scope_key: None,
                    evidence_ids: Vec::new(),
                    related_decision_id: None,
                    original_memory_id: Some(synth.memory_id.to_string()),
                    session_id: None,
                });
                totals.skipped += 1;
                continue;
            }
        };

        let mut evidence_ids = Vec::new();
        let mut reason_parts: Vec<&str> = Vec::new();
        let mut any_forgotten = false;
        let mut any_missing = false;

        for mid in &synth.source_memory_ids {
            let mid_status = status.get(mid).copied();
            if mid_status == Some(MemoryStatus::Forgotten) {
                any_forgotten = true;
                continue;
            }
            match evidence_map.get(mid) {
                Some(eid) => evidence_ids.push(*eid),
                None => {
                    any_missing = true;
                }
            }
        }

        let unsupported = evidence_ids.is_empty()
            || any_forgotten
            || any_missing
            || synth.source_memory_ids.is_empty();
        if any_forgotten {
            reason_parts.push(REASON_FORGOTTEN_SOURCE);
        }
        if any_missing && !any_forgotten {
            reason_parts.push(REASON_MISSING_SOURCE);
        }
        if synth.source_memory_ids.is_empty() && !any_forgotten {
            reason_parts.push(REASON_MISSING_SOURCE);
        }

        let reason_code = if reason_parts.is_empty() {
            REASON_LEGACY_SYNTH.to_string()
        } else {
            // Primary reason: forgotten_source takes precedence in cascade honesty.
            reason_parts[0].to_string()
        };

        let cid = legacy_conclusion_id(synth.event_id);
        actions.push(ImportAction {
            kind: ImportActionKind::Conclusion,
            original_event_id: synth.event_id,
            derived_id: cid.to_string(),
            reason_code,
            mechanism: ImportMechanism::WouldAppend,
            source_tag: None,
            unsupported: Some(unsupported),
            content: None,
            title: None,
            statement: Some(synth.content.clone()),
            privacy: synth.privacy,
            scope_key: Some(scope_key),
            evidence_ids,
            related_decision_id: None,
            original_memory_id: Some(synth.memory_id.to_string()),
            session_id: None,
        });
        totals.conclusion += 1;
    }

    decisions.sort_by_key(|d| d.event_id);
    for dec in decisions {
        let statement = compose_decision_statement(&dec.title, &dec.decision, &dec.context);
        if statement.trim().is_empty() && dec.title.trim().is_empty() {
            actions.push(ImportAction {
                kind: ImportActionKind::Skip,
                original_event_id: dec.event_id,
                derived_id: String::new(),
                reason_code: REASON_EMPTY_CONTENT.into(),
                mechanism: ImportMechanism::Skip,
                source_tag: None,
                unsupported: None,
                content: None,
                title: None,
                statement: None,
                privacy: dec.privacy,
                scope_key: None,
                evidence_ids: Vec::new(),
                related_decision_id: None,
                original_memory_id: Some(dec.legacy_memory_decision_id.to_string()),
                session_id: None,
            });
            totals.skipped += 1;
            continue;
        }

        let scope_key = match resolve_scope(dec.project_id.as_ref(), opts) {
            Ok(k) => k,
            Err(reason) => {
                actions.push(ImportAction {
                    kind: ImportActionKind::Skip,
                    original_event_id: dec.event_id,
                    derived_id: String::new(),
                    reason_code: reason.into(),
                    mechanism: ImportMechanism::Skip,
                    source_tag: None,
                    unsupported: None,
                    content: None,
                    title: None,
                    statement: None,
                    privacy: dec.privacy,
                    scope_key: None,
                    evidence_ids: Vec::new(),
                    related_decision_id: None,
                    original_memory_id: Some(dec.legacy_memory_decision_id.to_string()),
                    session_id: None,
                });
                totals.skipped += 1;
                continue;
            }
        };

        let did = legacy_decision_id(dec.event_id);
        let rid = legacy_review_id(dec.event_id);

        actions.push(ImportAction {
            kind: ImportActionKind::Decision,
            original_event_id: dec.event_id,
            derived_id: did.to_string(),
            reason_code: REASON_LEGACY_DECISION.into(),
            mechanism: ImportMechanism::WouldAppend,
            source_tag: None,
            unsupported: None,
            content: None,
            title: Some(dec.title.clone()),
            statement: Some(statement),
            privacy: dec.privacy,
            scope_key: Some(scope_key.clone()),
            evidence_ids: Vec::new(),
            related_decision_id: Some(did),
            original_memory_id: Some(dec.legacy_memory_decision_id.to_string()),
            session_id: None,
        });
        totals.decision += 1;

        actions.push(ImportAction {
            kind: ImportActionKind::Review,
            original_event_id: dec.event_id,
            derived_id: rid.to_string(),
            reason_code: REASON_LEGACY_REVIEW.into(),
            mechanism: ImportMechanism::WouldAppend,
            source_tag: None,
            unsupported: None,
            content: None,
            title: Some(dec.title.clone()),
            statement: None,
            privacy: dec.privacy,
            scope_key: Some(scope_key),
            evidence_ids: Vec::new(),
            related_decision_id: Some(did),
            original_memory_id: None,
            session_id: None,
        });
        totals.review += 1;
    }

    let plan_hash = compute_plan_hash(&actions)?;
    Ok(ImportPlan {
        actions,
        totals,
        plan_hash,
        principal_id: opts.principal_id,
        command_id: opts.command_id.clone(),
        dry_run: opts.dry_run,
    })
}

fn skip_action(event_id: Uuid, reason: &str, privacy: Privacy) -> ImportAction {
    ImportAction {
        kind: ImportActionKind::Skip,
        original_event_id: event_id,
        derived_id: String::new(),
        reason_code: reason.into(),
        mechanism: ImportMechanism::Skip,
        source_tag: None,
        unsupported: None,
        content: None,
        title: None,
        statement: None,
        privacy,
        scope_key: None,
        evidence_ids: Vec::new(),
        related_decision_id: None,
        original_memory_id: None,
        session_id: None,
    }
}

fn resolve_scope(
    project_id: Option<&ProjectId>,
    opts: &ImportOpts,
) -> std::result::Result<String, &'static str> {
    if let Some(pid) = project_id {
        return Ok(scope_identity_key(&ScopeRef::Repository(*pid)));
    }
    if let Some(ref scope) = opts.default_scope {
        return Ok(scope_identity_key(scope));
    }
    Err(REASON_MISSING_SCOPE)
}

fn compose_decision_statement(title: &str, decision: &str, context: &str) -> String {
    let d = decision.trim();
    if !d.is_empty() {
        return d.to_string();
    }
    let t = title.trim();
    if !t.is_empty() {
        return t.to_string();
    }
    context.trim().to_string()
}

fn truncate_body(s: &str) -> String {
    const MAX: usize = 80;
    if s.chars().count() <= MAX {
        s.to_string()
    } else {
        s.chars().take(MAX).collect::<String>() + "…"
    }
}

/// Compose durable Evidence summary including provenance sidecars (L17 / §5.1).
///
/// `source_tag` is preserved verbatim (no changeguard→ledgerful rewrite).
/// `session_id` is recorded for session-summary digests.
fn compose_evidence_summary(
    content: &str,
    source_tag: Option<&str>,
    session_id: Option<&str>,
) -> String {
    let mut meta: Vec<String> = Vec::new();
    if let Some(tag) = source_tag.filter(|t| !t.is_empty()) {
        meta.push(format!("[source_tag:{tag}]"));
    }
    if let Some(sid) = session_id.filter(|s| !s.is_empty()) {
        meta.push(format!("[session_id:{sid}]"));
    }
    if meta.is_empty() {
        content.to_string()
    } else {
        format!("{}\n{content}", meta.join("\n"))
    }
}

// ---------------------------------------------------------------------------
// Apply (raw build_event — L18)
// ---------------------------------------------------------------------------

/// Apply a previously classified plan to a destination vault via ports.
///
/// Requires [`ApplyOpts::confirm`] = true. Never opens a live vault (L2).
/// Does **not** call `observe_source` (L8/L18).
pub fn apply_legacy_import<W, Q, C>(
    writer: &W,
    query: &Q,
    clock: &C,
    plan: &ImportPlan,
    apply_opts: &ApplyOpts,
) -> Result<ImportReport>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
{
    if !apply_opts.confirm {
        return Err(ControlPlaneError::InvalidPayload(
            "apply_legacy_import requires ApplyOpts.confirm = true (L1)".into(),
        ));
    }

    // Reject nil principal on confirmed apply — default ImportOpts uses nil for
    // dry-run/classify only; governance proposer/opened-by must be real (Codex R1).
    if plan.principal_id.as_uuid().is_nil() {
        return Err(ControlPlaneError::InvalidPayload(
            "apply_legacy_import requires a non-nil plan.principal_id (nil principal is only valid for dry-run classify)".into(),
        ));
    }

    let mut report = ImportReport {
        applied: 0,
        skipped: 0,
        already_imported: 0,
        unresolved: 0,
        source_registered: 0,
        plan_hash: plan.plan_hash.clone(),
        legacy_import_applied: false,
        sample_event_ids: Vec::new(),
        error: None,
    };

    let source_id = legacy_source_id();
    let actor = Actor::System;
    let mut batch: Vec<Envelope> = Vec::new();
    let mut need_source = false;

    // Count skip/unresolved from plan for report honesty.
    for a in &plan.actions {
        match a.kind {
            ImportActionKind::Skip => report.skipped += 1,
            ImportActionKind::Unresolved => report.unresolved += 1,
            _ => {}
        }
    }

    // Detect whether any WouldAppend evidence needs the source row.
    let has_evidence_append = plan.actions.iter().any(|a| {
        a.kind == ImportActionKind::Evidence && a.mechanism == ImportMechanism::WouldAppend
    });

    if has_evidence_append {
        match query.get_source(source_id)? {
            Some(_) => {}
            None => {
                need_source = true;
            }
        }
    }

    if need_source {
        // Scope for source: first evidence action's scope, else empty.
        let scope = plan
            .actions
            .iter()
            .find(|a| a.kind == ImportActionKind::Evidence)
            .and_then(|a| a.scope_key.clone());
        let privacy = plan
            .actions
            .iter()
            .find(|a| a.kind == ImportActionKind::Evidence)
            .map(|a| a.privacy)
            .unwrap_or(Privacy::Sealed);
        batch.push(build_event(
            AggregateType::Source,
            source_id.as_uuid(),
            actor.clone(),
            privacy,
            Payload::SourceRegistered(SourceRegisteredPayload {
                source_id,
                kind: SourceKind::LegacyAiBrains,
                display_name: LEGACY_SOURCE_DISPLAY_NAME.into(),
                locator: None,
                scope,
            }),
        )?);
        report.source_registered = 1;
    }

    let now = clock.now()?;

    // In-batch de-dupe: projection probes only see pre-batch state (T167-R1-04).
    // Source is registered at most once above; no in-loop source appends.
    let mut seen_evidence: HashSet<EvidenceId> = HashSet::new();
    let mut seen_conclusion: HashSet<ConclusionId> = HashSet::new();
    let mut seen_decision: HashSet<DecisionId> = HashSet::new();
    let mut seen_review: HashSet<ReviewItemId> = HashSet::new();

    for a in &plan.actions {
        if a.mechanism != ImportMechanism::WouldAppend {
            continue;
        }

        match a.kind {
            ImportActionKind::Evidence => {
                let eid = match EvidenceId::from_str_or_err(&a.derived_id) {
                    Ok(id) => id,
                    Err(e) => {
                        return Err(ControlPlaneError::InvalidPayload(e));
                    }
                };
                if query.has_evidence(eid)? || !seen_evidence.insert(eid) {
                    report.already_imported += 1;
                    continue;
                }
                let body = a.content.as_deref().unwrap_or("");
                let summary = compose_evidence_summary(
                    body,
                    a.source_tag.as_deref(),
                    a.session_id.as_deref(),
                );
                let fingerprint = hex_sha256(summary.as_bytes());
                batch.push(build_event(
                    AggregateType::Evidence,
                    eid.as_uuid(),
                    actor.clone(),
                    a.privacy,
                    Payload::EvidenceRecorded(EvidenceRecordedPayload {
                        evidence_id: eid,
                        source_id,
                        source_version_id: None,
                        fingerprint: Some(fingerprint),
                        model_provenance: None,
                        summary,
                    }),
                )?);
                report.applied += 1;
                push_sample(
                    &mut report.sample_event_ids,
                    &a.original_event_id.to_string(),
                );
            }
            ImportActionKind::Conclusion => {
                let cid = match ConclusionId::from_str_or_err(&a.derived_id) {
                    Ok(id) => id,
                    Err(e) => {
                        return Err(ControlPlaneError::InvalidPayload(e));
                    }
                };
                if query.has_conclusion(cid)? || !seen_conclusion.insert(cid) {
                    report.already_imported += 1;
                    continue;
                }
                let statement = a.statement.clone().unwrap_or_default();
                let unsupported = a.unsupported.unwrap_or(a.evidence_ids.is_empty());
                let scope_key = a.scope_key.clone().unwrap_or_default();
                batch.push(build_event(
                    AggregateType::Conclusion,
                    cid.as_uuid(),
                    actor.clone(),
                    a.privacy,
                    Payload::ConclusionProposed(ConclusionProposedPayload {
                        conclusion_id: cid,
                        statement,
                        evidence_ids: a.evidence_ids.clone(),
                        proposer: plan.principal_id,
                        valid_from: Some(now),
                        valid_until: None,
                        scope: scope_key,
                        protected_category: None,
                        unsupported,
                        model_provenance: None,
                    }),
                )?);
                report.applied += 1;
                push_sample(
                    &mut report.sample_event_ids,
                    &a.original_event_id.to_string(),
                );
            }
            ImportActionKind::Decision => {
                let did = match DecisionId::from_str_or_err(&a.derived_id) {
                    Ok(id) => id,
                    Err(e) => {
                        return Err(ControlPlaneError::InvalidPayload(e));
                    }
                };
                if query.has_decision(did)? || !seen_decision.insert(did) {
                    report.already_imported += 1;
                    continue;
                }
                let title = a.title.clone().unwrap_or_else(|| "Legacy decision".into());
                let statement = a.statement.clone().unwrap_or_default();
                let scope_key = a.scope_key.clone().unwrap_or_default();
                batch.push(build_event(
                    AggregateType::Decision,
                    did.as_uuid(),
                    actor.clone(),
                    a.privacy,
                    Payload::DecisionProposed(DecisionProposedPayload {
                        decision_id: did,
                        title,
                        statement,
                        proposer: plan.principal_id,
                        conclusion_ids: None,
                        evidence_ids: None,
                        valid_from: Some(now),
                        valid_until: None,
                        scope: scope_key,
                    }),
                )?);
                report.applied += 1;
                push_sample(
                    &mut report.sample_event_ids,
                    &a.original_event_id.to_string(),
                );
            }
            ImportActionKind::Review => {
                let rid = match ReviewItemId::from_str_or_err(&a.derived_id) {
                    Ok(id) => id,
                    Err(e) => {
                        return Err(ControlPlaneError::InvalidPayload(e));
                    }
                };
                if query.get_review_item(rid)?.is_some() || !seen_review.insert(rid) {
                    report.already_imported += 1;
                    continue;
                }
                let did = a.related_decision_id;
                let subject = a
                    .title
                    .clone()
                    .unwrap_or_else(|| "Legacy decision review".into());
                let subject_id = did.map(|d| d.to_string()).unwrap_or_default();
                batch.push(build_event(
                    AggregateType::ReviewItem,
                    rid.as_uuid(),
                    actor.clone(),
                    a.privacy,
                    Payload::ReviewItemOpened(ReviewItemOpenedPayload {
                        review_item_id: rid,
                        subject,
                        opened_by: plan.principal_id,
                        subject_kind: ReviewSubjectKind::Decision,
                        subject_id,
                        criticality: ReviewCriticality::Medium,
                        related_conclusion_id: None,
                        related_decision_id: did,
                        related_source_id: None,
                    }),
                )?);
                report.applied += 1;
            }
            ImportActionKind::Skip | ImportActionKind::Unresolved => {}
        }
    }

    // L20: append LegacyImportApplied on successful apply (including idempotent completion).
    let batch_key = plan
        .command_id
        .clone()
        .unwrap_or_else(|| plan.plan_hash.clone());
    let agg_id = id_from_command(NS_LEGACY_IMPORT_BATCH, &batch_key);

    // Align counts with plan totals (same basis for every kind; T167-R1-07).
    let audit = Payload::LegacyImportApplied(LegacyImportAppliedPayload {
        plan_hash: plan.plan_hash.clone(),
        command_id: plan.command_id.clone(),
        evidence_count: plan.totals.evidence,
        conclusion_count: plan.totals.conclusion,
        decision_count: plan.totals.decision,
        review_count: plan.totals.review,
        source_count: report.source_registered,
        skipped_count: report.skipped,
        already_imported_count: report.already_imported,
        unresolved_count: report.unresolved,
        sample_ids: report.sample_event_ids.clone(),
    });
    batch.push(build_event(
        AggregateType::Job,
        agg_id,
        actor,
        Privacy::LocalOnly,
        audit,
    )?);

    writer.append_events(&batch)?;
    report.legacy_import_applied = true;
    Ok(report)
}

fn push_sample(samples: &mut Vec<String>, id: &str) {
    if samples.len() < 5 {
        samples.push(truncate_id(id));
    }
}

trait FromStrId: Sized {
    fn from_str_or_err(s: &str) -> std::result::Result<Self, String>;
}

impl FromStrId for EvidenceId {
    fn from_str_or_err(s: &str) -> std::result::Result<Self, String> {
        s.parse()
            .map_err(|e| format!("invalid evidence id {s}: {e}"))
    }
}
impl FromStrId for ConclusionId {
    fn from_str_or_err(s: &str) -> std::result::Result<Self, String> {
        s.parse()
            .map_err(|e| format!("invalid conclusion id {s}: {e}"))
    }
}
impl FromStrId for DecisionId {
    fn from_str_or_err(s: &str) -> std::result::Result<Self, String> {
        s.parse()
            .map_err(|e| format!("invalid decision id {s}: {e}"))
    }
}
impl FromStrId for ReviewItemId {
    fn from_str_or_err(s: &str) -> std::result::Result<Self, String> {
        s.parse().map_err(|e| format!("invalid review id {s}: {e}"))
    }
}

// ---------------------------------------------------------------------------
// Default plan serialization without bodies (L15 / report test)
// ---------------------------------------------------------------------------

/// Operator report action view (L15).
///
/// Never includes full apply bodies. Optional `truncated_summary` only when
/// `include_truncated_summaries` is requested.
#[derive(Debug, Serialize)]
struct ReportActionView<'a> {
    kind: &'a str,
    original_event_id: String,
    derived_id: &'a str,
    reason_code: &'a str,
    mechanism: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_tag: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unsupported: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    truncated_summary: Option<String>,
}

#[derive(Debug, Serialize)]
struct PlanReportView<'a> {
    totals: &'a ImportTotals,
    plan_hash: &'a str,
    actions: Vec<ReportActionView<'a>>,
    dry_run: bool,
}

/// Serialize plan for operator reports (L15).
///
/// - `include_truncated_summaries = false` (default): no body text.
/// - `include_truncated_summaries = true`: truncated snippets only — never full
///   apply bodies. Full content remains on [`ImportAction`] for apply fidelity.
pub fn plan_report_json(plan: &ImportPlan, include_truncated_summaries: bool) -> Result<String> {
    let views: Vec<ReportActionView<'_>> = plan
        .actions
        .iter()
        .map(|a| {
            let truncated_summary = if include_truncated_summaries {
                a.content
                    .as_deref()
                    .or(a.statement.as_deref())
                    .or(a.title.as_deref())
                    .map(truncate_body)
            } else {
                None
            };
            ReportActionView {
                kind: a.kind.as_str(),
                original_event_id: a.original_event_id.to_string(),
                derived_id: &a.derived_id,
                reason_code: &a.reason_code,
                mechanism: a.mechanism.as_str(),
                source_tag: &a.source_tag,
                session_id: &a.session_id,
                unsupported: a.unsupported,
                truncated_summary,
            }
        })
        .collect();
    let report = PlanReportView {
        totals: &plan.totals,
        plan_hash: &plan.plan_hash,
        actions: views,
        dry_run: plan.dry_run,
    };
    serde_json::to_string_pretty(&report)
        .map_err(|e| ControlPlaneError::InvalidPayload(e.to_string()))
}
