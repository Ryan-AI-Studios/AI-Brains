//! Propose-only pin graduation (T336).
//!
//! Scans in-scope pinned memories, classifies with
//! [`ai_brains_retrieval::classify_pin_kind`], and appends
//! `DecisionProposed` / `ConclusionProposed` plus `ReviewItemOpened`.
//! Never emits `DecisionApproved`, `ConclusionActivated`, or `ReviewItemResolved`.
//! Does not call `propose_decision` (policy-gated). Does not flip
//! `AI_BRAINS_GOVERNED_SYNTHESIS`.

use crate::memory_synthesis::system_synthesis_principal;
use ai_brains_core::ids::{ConclusionId, DecisionId, MemoryId, ProjectId, ReviewItemId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::review::{ReviewCriticality, ReviewSubjectKind};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    ConclusionProposedPayload, DecisionProposedPayload, ReviewItemOpenedPayload,
};
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_retrieval::{PinKind, classify_pin_kind, first_contentful_line};
use ai_brains_store::{
    EventStore, MemoryListFilter, MemoryListStatus, QueryStore, errors::StoreError,
};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

/// Env override for the per-run graduation cap (`usize`). Invalid or unset → 10.
pub const GRADUATION_CAP_ENV: &str = "AI_BRAINS_GRADUATION_CAP";

/// Default pin-graduation cap (F7).
pub const DEFAULT_GRADUATION_CAP: usize = 10;

/// Named UUID v5 namespace for deterministic proposal / review ids (F11).
///
/// Not [`Uuid::nil`]. Stable across runs so a second nightly is idempotent.
pub const PIN_GRADUATION_NAMESPACE: Uuid =
    Uuid::from_u128(0xA1B2_A1B2_A1B2_A1B2_A1B2_A1B2_A1B2_0336);

/// How nightly should treat pin graduation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraduationMode {
    /// Scan and append proposals + review items.
    Run,
    /// Do not scan (SYSTEM wrapper default).
    Skip,
    /// Scan and report counts; do not append.
    DryRun,
}

/// Counts from one graduation pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GraduationReport {
    /// Events actually appended this pass (0 on dry-run).
    pub proposed: usize,
    /// Eligible after cap that would append if not dry-run (excludes existing).
    pub would_propose: usize,
    pub skipped_existing: usize,
    pub skipped_privacy: usize,
    pub skipped_kind: usize,
    pub skipped_empty: usize,
    /// Decision/Constraint pins that passed privacy + non-empty, before cap.
    pub eligible_before_cap: usize,
    pub cap: usize,
}

/// Pin-graduation failures (fail-open at the nightly hook).
#[derive(Debug, Error)]
pub enum PinGraduationError {
    #[error("pin graduation store: {0}")]
    Store(#[from] StoreError),
    #[error("pin graduation event: {0}")]
    Event(#[from] ai_brains_events::EventError),
}

/// Parse `AI_BRAINS_GRADUATION_CAP`. Invalid / unset → [`DEFAULT_GRADUATION_CAP`].
pub fn graduation_cap_from_env() -> usize {
    match std::env::var(GRADUATION_CAP_ENV) {
        Ok(raw) => raw
            .trim()
            .parse::<usize>()
            .unwrap_or(DEFAULT_GRADUATION_CAP),
        Err(_) => DEFAULT_GRADUATION_CAP,
    }
}

fn kind_tag(kind: PinKind) -> Option<&'static str> {
    match kind {
        PinKind::Decision => Some("decision"),
        PinKind::Constraint => Some("conclusion"),
        PinKind::Hotspot | PinKind::Other => None,
    }
}

fn proposal_uuid(project_id: ProjectId, memory_id: MemoryId, kind_tag: &str) -> Uuid {
    let name = format!("{project_id}:{memory_id}:{kind_tag}");
    Uuid::new_v5(&PIN_GRADUATION_NAMESPACE, name.as_bytes())
}

fn review_uuid(project_id: ProjectId, memory_id: MemoryId, kind_tag: &str) -> Uuid {
    let name = format!("{project_id}:{memory_id}:review:{kind_tag}");
    Uuid::new_v5(&PIN_GRADUATION_NAMESPACE, name.as_bytes())
}

struct EligiblePin {
    memory_id: MemoryId,
    content: String,
    created_at: String,
    kind: PinKind,
    privacy: Privacy,
}

/// Scan project pins and optionally append propose + review events.
///
/// Cap applies to the sorted eligible list **before** idempotency skips
/// (do not refill from pins beyond the cap). Sort key is
/// `(memory_projection.created_at, memory_id)` (F7).
pub fn graduate_pins(
    query_store: &dyn QueryStore,
    event_store: &dyn EventStore,
    project_id: ProjectId,
    dry_run: bool,
) -> Result<GraduationReport, PinGraduationError> {
    let cap = graduation_cap_from_env();
    let mut report = GraduationReport {
        cap,
        ..GraduationReport::default()
    };

    let count_filter = MemoryListFilter {
        status: MemoryListStatus::Pinned,
        project_id: Some(project_id),
        tag: None,
        limit: 1,
    };
    let total = query_store.count_memories(&count_filter)?;
    if total == 0 {
        return Ok(report);
    }
    let list_limit = usize::try_from(total).unwrap_or(usize::MAX);
    let list_filter = MemoryListFilter {
        status: MemoryListStatus::Pinned,
        project_id: Some(project_id),
        tag: None,
        limit: list_limit.max(1),
    };
    let rows = query_store.list_memories(&list_filter)?;

    let mut eligible: Vec<EligiblePin> = Vec::new();
    for row in rows {
        let memory_id = match MemoryId::from_str(&row.memory_id) {
            Ok(id) => id,
            Err(e) => {
                tracing::warn!(
                    memory_id = %row.memory_id,
                    error = %e,
                    "pin graduation skipped unparseable memory_id"
                );
                continue;
            }
        };
        let kind = classify_pin_kind(&row.content);
        if kind_tag(kind).is_none() {
            report.skipped_kind = report.skipped_kind.saturating_add(1);
            continue;
        }
        let privacy = match query_store.get_memory_privacy(&memory_id)? {
            Some(p) => p,
            None => Privacy::LocalOnly,
        };
        if matches!(privacy, Privacy::Sealed | Privacy::NeverInject) {
            report.skipped_privacy = report.skipped_privacy.saturating_add(1);
            continue;
        }
        let title = first_contentful_line(&row.content).trim();
        let statement = row.content.trim();
        if title.is_empty() || statement.is_empty() {
            report.skipped_empty = report.skipped_empty.saturating_add(1);
            continue;
        }
        eligible.push(EligiblePin {
            memory_id,
            content: row.content,
            created_at: row.created_at,
            kind,
            privacy,
        });
    }

    eligible.sort_by(|a, b| {
        a.created_at
            .cmp(&b.created_at)
            .then_with(|| a.memory_id.as_uuid().cmp(&b.memory_id.as_uuid()))
    });
    report.eligible_before_cap = eligible.len();
    eligible.truncate(cap);

    let proposer = system_synthesis_principal();
    let scope = format!("Repository:{project_id}");

    for pin in eligible {
        let Some(tag) = kind_tag(pin.kind) else {
            continue;
        };
        let aggregate_uuid = proposal_uuid(project_id, pin.memory_id, tag);
        let existing = event_store.read_events(aggregate_uuid)?;
        if !existing.is_empty() {
            report.skipped_existing = report.skipped_existing.saturating_add(1);
            continue;
        }
        report.would_propose = report.would_propose.saturating_add(1);
        if dry_run {
            continue;
        }

        let title = first_contentful_line(&pin.content).trim().to_string();
        let statement = pin.content.clone();
        let review_id = ReviewItemId::from_uuid(review_uuid(project_id, pin.memory_id, tag));

        let envelopes = match pin.kind {
            PinKind::Decision => {
                let decision_id = DecisionId::from_uuid(aggregate_uuid);
                let proposed = EventBuilder::new(
                    AggregateType::Decision,
                    decision_id.as_uuid(),
                    Actor::System,
                    pin.privacy,
                )
                .build(Payload::DecisionProposed(DecisionProposedPayload {
                    decision_id,
                    title: title.clone(),
                    statement,
                    proposer,
                    conclusion_ids: None,
                    evidence_ids: None,
                    valid_from: None,
                    valid_until: None,
                    scope: scope.clone(),
                }))?;
                let opened = EventBuilder::new(
                    AggregateType::ReviewItem,
                    review_id.as_uuid(),
                    Actor::System,
                    pin.privacy,
                )
                .build(Payload::ReviewItemOpened(ReviewItemOpenedPayload {
                    review_item_id: review_id,
                    subject: title,
                    opened_by: proposer,
                    subject_kind: ReviewSubjectKind::Decision,
                    subject_id: decision_id.to_string(),
                    criticality: ReviewCriticality::Medium,
                    related_conclusion_id: None,
                    related_decision_id: Some(decision_id),
                    related_source_id: None,
                }))?;
                vec![proposed, opened]
            }
            PinKind::Constraint => {
                let conclusion_id = ConclusionId::from_uuid(aggregate_uuid);
                let proposed = EventBuilder::new(
                    AggregateType::Conclusion,
                    conclusion_id.as_uuid(),
                    Actor::System,
                    pin.privacy,
                )
                .build(Payload::ConclusionProposed(ConclusionProposedPayload {
                    conclusion_id,
                    statement,
                    evidence_ids: vec![],
                    proposer,
                    valid_from: None,
                    valid_until: None,
                    scope: scope.clone(),
                    protected_category: None,
                    // Payload flag is false (user pin, not LLM synthesis). Store
                    // `conclusion_projection.unsupported` still becomes 1 when
                    // `evidence_ids` is empty (T167 coerce). Do not invent evidence.
                    unsupported: false,
                    model_provenance: None,
                }))?;
                let opened = EventBuilder::new(
                    AggregateType::ReviewItem,
                    review_id.as_uuid(),
                    Actor::System,
                    pin.privacy,
                )
                .build(Payload::ReviewItemOpened(ReviewItemOpenedPayload {
                    review_item_id: review_id,
                    subject: title,
                    opened_by: proposer,
                    subject_kind: ReviewSubjectKind::Conclusion,
                    subject_id: conclusion_id.to_string(),
                    criticality: ReviewCriticality::Medium,
                    related_conclusion_id: Some(conclusion_id),
                    related_decision_id: None,
                    related_source_id: None,
                }))?;
                vec![proposed, opened]
            }
            PinKind::Hotspot | PinKind::Other => continue,
        };

        event_store.append_events(&envelopes)?;
        report.proposed = report.proposed.saturating_add(1);
    }

    Ok(report)
}
