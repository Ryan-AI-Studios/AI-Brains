//! Class-based retention plan + apply (T166 / P8.4).
//!
//! - **R1** Dry-run default; apply requires `confirm=true`.
//! - **R2** Envelope CE only via [`crate::wipe_content_envelope`] — no parallel destroy.
//! - **R3** Projection delete never labeled CE.
//! - **R4** Reports: counts + truncated ids only.
//! - **R5** Canonical classes; unknown → unclassified skip on apply.
//! - **R6** Active approved decisions not age-wiped.
//! - **R7** Nightly CE is opt-in elsewhere (`apply_ce_on_nightly` default false).
//! - **R11** Pinned memories held from age-based apply.
//! - **R12** Apply appends `RetentionApplied` (not dry-run).
//! - **R13** Stream A/B de-dupe; unique identities; CE wins when turn subject join known.
//! - **R14** Cooldown clocks use terminal `updated_at`.
//! - **R15** Hierarchy parents marked stale for resynthesis when child disposed.
//! - **R16** Orphan envelopes: active wrap, 0 blobs, age ≥ 7d.

use ai_brains_contracts::retention::{
    self, CLASS_DECISION_APPROVED, CLASS_EVIDENCE, CLASS_MEMORY_LEGACY, CLASS_ORPHANED_ENVELOPE,
    CLASS_QUERY_TRACE, CLASS_RAW_TURN, CLASS_REVIEW_TRACE, CLASS_SECRET, CLASS_UNCLASSIFIED,
    MECHANISM_CE_WIPE, MECHANISM_HELD, MECHANISM_PROJECTION_DELETE, MECHANISM_SKIP,
    RETENTION_HONESTY_LEGACY_NOT_CE, RETENTION_HONESTY_NOT_NIST_PURGE,
    RETENTION_HONESTY_PRE_ERASE_BACKUP, RETENTION_HONESTY_STREAM_INDEPENDENCE,
    RETENTION_HONESTY_TICKET_NOT_CE, RetentionCascade, RetentionClassBucket, RetentionPlanReport,
    RetentionReportMode, RetentionTotals, default_horizon_labels, is_canonical_class, truncate_id,
    truncate_sample_ids,
};
use ai_brains_core::ids::ContentKeyId;
use ai_brains_core::principal::Principal;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_events::payload::{RetentionAppliedPayload, RetentionClassCount};
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::projections::retention::{self as ret_scan, EnvelopeKeyScan, TurnKey};
use chrono::{Duration, Utc};
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use crate::command_id::id_from_command;
use crate::cryptographic_erasure::{
    ContentEnvelopeWipeStore, WipeContentEnvelopeCommand, wipe_content_envelope,
};
use crate::errors::{ControlPlaneError, Result};
use crate::ports::{Clock, EventWriter, GovernedQueryStore, PolicyEvaluator};
use crate::sources::build_event;

/// Namespace for retention apply command_id → audit aggregate id.
pub const NS_RETENTION_APPLY: &str = "ai-brains.command.retention_apply";

// ---------------------------------------------------------------------------
// Config (env-driven; no migration)
// ---------------------------------------------------------------------------

/// Class horizons and nightly CE opt-in (R7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionConfig {
    pub raw_turn_days: i64,
    pub evidence_days: i64,
    pub secret_days: i64,
    pub query_trace_days: i64,
    pub review_trace_days: i64,
    pub decision_revoked_cooldown_days: i64,
    pub orphan_envelope_days: i64,
    /// Default **false** — never auto-CE on nightly without opt-in (R7).
    pub apply_ce_on_nightly: bool,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            raw_turn_days: 90,
            evidence_days: 365,
            secret_days: 7,
            query_trace_days: 30,
            review_trace_days: 90,
            decision_revoked_cooldown_days: 30,
            orphan_envelope_days: 7,
            apply_ce_on_nightly: false,
        }
    }
}

impl RetentionConfig {
    /// Load from `AI_BRAINS_RETENTION_*` env vars; missing keys keep defaults.
    pub fn from_env() -> Self {
        let mut c = Self::default();
        if let Some(v) = parse_env_i64("AI_BRAINS_RETENTION_RAW_TURN_DAYS") {
            c.raw_turn_days = v;
        }
        if let Some(v) = parse_env_i64("AI_BRAINS_RETENTION_EVIDENCE_DAYS") {
            c.evidence_days = v;
        }
        if let Some(v) = parse_env_i64("AI_BRAINS_RETENTION_SECRET_DAYS") {
            c.secret_days = v;
        }
        if let Some(v) = parse_env_i64("AI_BRAINS_RETENTION_QUERY_TRACE_DAYS") {
            c.query_trace_days = v;
        }
        if let Some(v) = parse_env_i64("AI_BRAINS_RETENTION_REVIEW_TRACE_DAYS") {
            c.review_trace_days = v;
        }
        if let Some(v) = parse_env_i64("AI_BRAINS_RETENTION_DECISION_REVOKED_COOLDOWN_DAYS") {
            c.decision_revoked_cooldown_days = v;
        }
        if let Some(v) = parse_env_i64("AI_BRAINS_RETENTION_ORPHAN_ENVELOPE_DAYS") {
            c.orphan_envelope_days = v;
        }
        c.apply_ce_on_nightly = parse_env_bool("AI_BRAINS_RETENTION_APPLY_CE")
            || parse_env_bool("AI_BRAINS_RETENTION_APPLY_CE_ON_NIGHTLY");
        c
    }

    pub fn horizon_labels(&self) -> BTreeMap<String, String> {
        let mut m = default_horizon_labels();
        m.insert(CLASS_RAW_TURN.to_string(), self.raw_turn_days.to_string());
        m.insert(CLASS_EVIDENCE.to_string(), self.evidence_days.to_string());
        m.insert(CLASS_SECRET.to_string(), self.secret_days.to_string());
        m.insert(
            CLASS_QUERY_TRACE.to_string(),
            self.query_trace_days.to_string(),
        );
        m.insert(
            CLASS_REVIEW_TRACE.to_string(),
            self.review_trace_days.to_string(),
        );
        m.insert(
            CLASS_DECISION_APPROVED.to_string(),
            format!(
                "revoked_superseded+{}d_cooldown",
                self.decision_revoked_cooldown_days
            ),
        );
        m.insert(
            CLASS_ORPHANED_ENVELOPE.to_string(),
            self.orphan_envelope_days.to_string(),
        );
        m
    }
}

fn parse_env_i64(key: &str) -> Option<i64> {
    std::env::var(key).ok().and_then(|s| s.parse().ok())
}

fn parse_env_bool(key: &str) -> bool {
    match std::env::var(key) {
        Ok(s) => {
            let t = s.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

/// Whether nightly may run CE bulk (R7). Default false.
pub fn nightly_ce_enabled(config: &RetentionConfig) -> bool {
    config.apply_ce_on_nightly
}

// ---------------------------------------------------------------------------
// Internal candidates
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Stream {
    A,
    B,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Candidate {
    stream: Stream,
    class: String,
    /// Unique identity key (R13).
    id: String,
    mechanism: String,
    notes: Vec<String>,
    content_key_id: Option<String>,
    turn: Option<TurnKey>,
    query_trace_id: Option<String>,
    review_item_id: Option<String>,
    decision_id: Option<String>,
    memory_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// Plan
// ---------------------------------------------------------------------------

/// Build a dry-run retention plan (read-only).
pub fn plan_retention(
    store: &SqliteEventStore,
    config: &RetentionConfig,
) -> Result<RetentionPlanReport> {
    let now = Utc::now();
    let generated_at = now.to_rfc3339();
    let candidates = collect_candidates(store, config, now)?;
    Ok(build_report(
        RetentionReportMode::DryRun,
        &generated_at,
        config,
        &candidates,
        RetentionCascade {
            parents_marked_for_resynthesis: estimate_cascade(store, &candidates)?,
        },
        0,
        Vec::new(),
    ))
}

fn estimate_cascade(store: &SqliteEventStore, candidates: &[Candidate]) -> Result<u64> {
    let conn = store
        .connection()
        .lock()
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
    let mut children = Vec::new();
    for c in candidates {
        if c.mechanism == MECHANISM_CE_WIPE || c.mechanism == MECHANISM_PROJECTION_DELETE {
            children.extend(c.memory_ids.iter().cloned());
        }
    }
    ret_scan::count_parents_for_resynthesis(&conn, &children)
        .map_err(|e| ControlPlaneError::Query(e.to_string()))
}

fn collect_candidates(
    store: &SqliteEventStore,
    config: &RetentionConfig,
    now: chrono::DateTime<Utc>,
) -> Result<Vec<Candidate>> {
    let conn = store
        .connection()
        .lock()
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;

    let pinned = ret_scan::list_pinned_memory_ids(&conn)
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;

    // Stream B first so turn subject joins can suppress stream A (R13 precedence).
    let envelopes =
        ret_scan::list_envelope_keys(&conn).map_err(|e| ControlPlaneError::Query(e.to_string()))?;

    let mut candidates: Vec<Candidate> = Vec::new();
    let mut seen_ids: BTreeSet<String> = BTreeSet::new();
    // R13: any known turn↔envelope join suppresses stream-A projection_delete
    // for that turn, regardless of stream-B mechanism (ce_wipe / held / skip).
    let mut turn_ids_covered_by_envelope: BTreeSet<String> = BTreeSet::new();
    let mut seen_content_keys: BTreeSet<String> = BTreeSet::new();

    for env in &envelopes {
        if !seen_content_keys.insert(env.content_key_id.clone()) {
            continue; // R13: never double-count same content_key
        }
        let c = classify_envelope(env, config, now, &pinned)?;
        for t in &env.turn_subject_ids {
            turn_ids_covered_by_envelope.insert(t.clone());
        }
        if seen_ids.insert(c.id.clone()) {
            candidates.push(c);
        }
    }

    // Stream A — raw turns
    let turn_cutoff = (now - Duration::days(config.raw_turn_days)).to_rfc3339();
    let turns = ret_scan::list_old_turns(&conn, &turn_cutoff)
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
    for t in turns {
        let id = t.identity();
        // R13: when turn↔envelope join known, stream B wins — skip projection_delete.
        if turn_ids_covered_by_envelope.contains(&id) {
            continue;
        }
        if !seen_ids.insert(format!("turn:{id}")) {
            continue;
        }
        candidates.push(Candidate {
            stream: Stream::A,
            class: CLASS_RAW_TURN.to_string(),
            id: format!("turn:{id}"),
            mechanism: MECHANISM_PROJECTION_DELETE.to_string(),
            notes: vec!["event log retained".into()],
            content_key_id: None,
            turn: Some(t),
            query_trace_id: None,
            review_item_id: None,
            decision_id: None,
            memory_ids: Vec::new(),
        });
    }

    // Query traces
    let qt_cutoff = (now - Duration::days(config.query_trace_days)).to_rfc3339();
    let qts = ret_scan::list_old_query_traces(&conn, &qt_cutoff)
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
    for q in qts {
        let id = format!("query_trace:{}", q.trace_id);
        if !seen_ids.insert(id.clone()) {
            continue;
        }
        candidates.push(Candidate {
            stream: Stream::A,
            class: CLASS_QUERY_TRACE.to_string(),
            id,
            mechanism: MECHANISM_PROJECTION_DELETE.to_string(),
            notes: vec!["event log retained".into()],
            content_key_id: None,
            turn: None,
            query_trace_id: Some(q.trace_id),
            review_item_id: None,
            decision_id: None,
            memory_ids: Vec::new(),
        });
    }

    // Review traces (closed + aged)
    let rt_cutoff = (now - Duration::days(config.review_trace_days)).to_rfc3339();
    let rts = ret_scan::list_old_closed_reviews(&conn, &rt_cutoff)
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
    for r in rts {
        let id = format!("review:{}", r.review_item_id);
        if !seen_ids.insert(id.clone()) {
            continue;
        }
        candidates.push(Candidate {
            stream: Stream::A,
            class: CLASS_REVIEW_TRACE.to_string(),
            id,
            mechanism: MECHANISM_PROJECTION_DELETE.to_string(),
            notes: vec![
                "event log retained".into(),
                "cooldown uses terminal updated_at (R14)".into(),
            ],
            content_key_id: None,
            turn: None,
            query_trace_id: None,
            review_item_id: Some(r.review_item_id),
            decision_id: None,
            memory_ids: Vec::new(),
        });
    }

    // Decisions: only revoked/superseded + cooldown (R6/R14)
    let dec_cutoff = (now - Duration::days(config.decision_revoked_cooldown_days)).to_rfc3339();
    let decs = ret_scan::list_disposable_decisions(&conn, &dec_cutoff)
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
    for d in decs {
        let id = format!("decision:{}", d.decision_id);
        if !seen_ids.insert(id.clone()) {
            continue;
        }
        candidates.push(Candidate {
            stream: Stream::A,
            class: CLASS_DECISION_APPROVED.to_string(),
            id,
            mechanism: MECHANISM_PROJECTION_DELETE.to_string(),
            notes: vec![
                format!("terminal state {}", d.state),
                "cooldown uses terminal updated_at (R14)".into(),
            ],
            content_key_id: None,
            turn: None,
            query_trace_id: None,
            review_item_id: None,
            decision_id: Some(d.decision_id),
            memory_ids: Vec::new(),
        });
    }

    // Drop pure "within horizon" noise; always keep unclassified dry-run debt (R5)
    // and actionable / held candidates.
    candidates.retain(|c| c.mechanism != MECHANISM_SKIP || c.class == CLASS_UNCLASSIFIED);

    // Stable sort (stream, class, id) — R13 determinism
    candidates.sort_by(|a, b| {
        (a.stream, a.class.as_str(), a.id.as_str()).cmp(&(
            b.stream,
            b.class.as_str(),
            b.id.as_str(),
        ))
    });
    Ok(candidates)
}

fn classify_envelope(
    env: &EnvelopeKeyScan,
    config: &RetentionConfig,
    now: chrono::DateTime<Utc>,
    pinned: &BTreeSet<String>,
) -> Result<Candidate> {
    let id = format!("content_key:{}", env.content_key_id);

    // R16 orphans
    if env.blob_count == 0 {
        let orphan_cutoff = (now - Duration::days(config.orphan_envelope_days)).to_rfc3339();
        if env.created_at.as_str() < orphan_cutoff.as_str() {
            return Ok(Candidate {
                stream: Stream::B,
                class: CLASS_ORPHANED_ENVELOPE.to_string(),
                id,
                mechanism: MECHANISM_CE_WIPE.to_string(),
                notes: vec!["active wrap, zero blobs, age ≥ orphan horizon".into()],
                content_key_id: Some(env.content_key_id.clone()),
                turn: None,
                query_trace_id: None,
                review_item_id: None,
                decision_id: None,
                memory_ids: env.memory_subject_ids.clone(),
            });
        }
        // Young orphan — skip (in-flight seal window)
        return Ok(Candidate {
            stream: Stream::B,
            class: CLASS_ORPHANED_ENVELOPE.to_string(),
            id,
            mechanism: MECHANISM_SKIP.to_string(),
            notes: vec!["orphan younger than horizon; skip to avoid seal race".into()],
            content_key_id: Some(env.content_key_id.clone()),
            turn: None,
            query_trace_id: None,
            review_item_id: None,
            decision_id: None,
            memory_ids: env.memory_subject_ids.clone(),
        });
    }

    // Mixed / unknown class (R5)
    let class = resolve_blob_class(&env.content_classes);
    if class == CLASS_UNCLASSIFIED {
        return Ok(Candidate {
            stream: Stream::B,
            class: CLASS_UNCLASSIFIED.to_string(),
            id,
            mechanism: MECHANISM_SKIP.to_string(),
            notes: vec![if env.content_classes.len() > 1 {
                "mixed content_class under one key".into()
            } else {
                "unknown or missing content_class".into()
            }],
            content_key_id: Some(env.content_key_id.clone()),
            turn: None,
            query_trace_id: None,
            review_item_id: None,
            decision_id: None,
            memory_ids: env.memory_subject_ids.clone(),
        });
    }

    // R11 pin hold: if **any** linked memory subject is pinned, hold the whole key
    // (do not age-based CE-wipe; protects pinned subjects sharing an envelope).
    if env.memory_subject_ids.iter().any(|m| pinned.contains(m)) {
        return Ok(Candidate {
            stream: Stream::B,
            class: class.to_string(),
            id,
            mechanism: MECHANISM_HELD.to_string(),
            notes: vec!["held:pinned".into()],
            content_key_id: Some(env.content_key_id.clone()),
            turn: None,
            query_trace_id: None,
            review_item_id: None,
            decision_id: None,
            memory_ids: env.memory_subject_ids.clone(),
        });
    }

    let horizon_days = match class {
        CLASS_SECRET => config.secret_days,
        CLASS_EVIDENCE => config.evidence_days,
        // Blob content_class=raw_turn is stream B only; v1 default skip (spec §5)
        CLASS_RAW_TURN => {
            return Ok(Candidate {
                stream: Stream::B,
                class: CLASS_RAW_TURN.to_string(),
                id,
                mechanism: MECHANISM_SKIP.to_string(),
                notes: vec![
                    "blob content_class=raw_turn does not drive stream A turn delete".into(),
                    "v1: skip CE age for stream B raw_turn label unless horizon configured".into(),
                ],
                content_key_id: Some(env.content_key_id.clone()),
                turn: None,
                query_trace_id: None,
                review_item_id: None,
                decision_id: None,
                memory_ids: env.memory_subject_ids.clone(),
            });
        }
        _ => {
            return Ok(Candidate {
                stream: Stream::B,
                class: CLASS_UNCLASSIFIED.to_string(),
                id,
                mechanism: MECHANISM_SKIP.to_string(),
                notes: vec!["no auto age policy for class".into()],
                content_key_id: Some(env.content_key_id.clone()),
                turn: None,
                query_trace_id: None,
                review_item_id: None,
                decision_id: None,
                memory_ids: env.memory_subject_ids.clone(),
            });
        }
    };

    let cutoff = (now - Duration::days(horizon_days)).to_rfc3339();
    if env.age_anchor.as_str() < cutoff.as_str() {
        Ok(Candidate {
            stream: Stream::B,
            class: class.to_string(),
            id,
            mechanism: MECHANISM_CE_WIPE.to_string(),
            notes: vec!["CE via T165 wipe_content_envelope only".into()],
            content_key_id: Some(env.content_key_id.clone()),
            turn: None,
            query_trace_id: None,
            review_item_id: None,
            decision_id: None,
            memory_ids: env.memory_subject_ids.clone(),
        })
    } else {
        Ok(Candidate {
            stream: Stream::B,
            class: class.to_string(),
            id,
            mechanism: MECHANISM_SKIP.to_string(),
            notes: vec!["within horizon".into()],
            content_key_id: Some(env.content_key_id.clone()),
            turn: None,
            query_trace_id: None,
            review_item_id: None,
            decision_id: None,
            memory_ids: env.memory_subject_ids.clone(),
        })
    }
}

fn resolve_blob_class(classes: &[String]) -> &'static str {
    if classes.is_empty() {
        return CLASS_UNCLASSIFIED;
    }
    if classes.len() > 1 {
        // Mixed → unclassified
        return CLASS_UNCLASSIFIED;
    }
    let c = classes[0].as_str();
    if is_canonical_class(c) {
        // Map to static for match ergonomics
        match c {
            CLASS_SECRET => CLASS_SECRET,
            CLASS_EVIDENCE => CLASS_EVIDENCE,
            CLASS_RAW_TURN => CLASS_RAW_TURN,
            CLASS_ORPHANED_ENVELOPE => CLASS_ORPHANED_ENVELOPE,
            CLASS_MEMORY_LEGACY => CLASS_MEMORY_LEGACY,
            _ => CLASS_UNCLASSIFIED,
        }
    } else {
        CLASS_UNCLASSIFIED
    }
}

fn build_report(
    mode: RetentionReportMode,
    generated_at: &str,
    config: &RetentionConfig,
    candidates: &[Candidate],
    cascade: RetentionCascade,
    errors_count: u64,
    errors: Vec<String>,
) -> RetentionPlanReport {
    let mut by_class: BTreeMap<String, Vec<&Candidate>> = BTreeMap::new();
    for c in candidates {
        by_class.entry(c.class.clone()).or_default().push(c);
    }

    let mut classes = Vec::new();
    let mut totals = RetentionTotals::default();
    let mut has_ce = false;

    for (class, items) in by_class {
        // Dominant mechanism: first non-skip if mixed, else majority
        let mechanism = dominant_mechanism(&items);
        if mechanism == MECHANISM_CE_WIPE {
            has_ce = true;
        }
        let sample_ids = truncate_sample_ids(items.iter().map(|c| c.id.as_str()));
        let mut notes: BTreeSet<String> = BTreeSet::new();
        for i in &items {
            notes.extend(i.notes.iter().cloned());
        }
        // Per-mechanism sub-counts into totals (by candidate, not class)
        for i in &items {
            totals.candidates = totals.candidates.saturating_add(1);
            match i.mechanism.as_str() {
                MECHANISM_CE_WIPE => {
                    totals.would_ce_wipe = totals.would_ce_wipe.saturating_add(1);
                    has_ce = true;
                }
                MECHANISM_PROJECTION_DELETE => {
                    totals.would_projection_delete =
                        totals.would_projection_delete.saturating_add(1);
                }
                MECHANISM_HELD => {
                    totals.would_held = totals.would_held.saturating_add(1);
                }
                _ => {
                    totals.would_skip = totals.would_skip.saturating_add(1);
                }
            }
        }
        classes.push(RetentionClassBucket {
            class,
            candidate_count: items.len() as u64,
            mechanism,
            sample_ids,
            notes: notes.into_iter().collect(),
        });
    }

    // Suppress pure within-horizon / young-orphan noise: drop classes where every
    // candidate is skip and class is not unclassified (optional cleanliness).
    // Keep all for transparency (spec lists unclassified skip in dry-run).

    RetentionPlanReport {
        api_version: retention::API_VERSION.to_string(),
        generated_at: generated_at.to_string(),
        mode: mode.as_str().to_string(),
        horizons: config.horizon_labels(),
        classes,
        totals,
        cascade,
        warnings: honesty_warnings(has_ce),
        errors_count,
        errors,
    }
}

fn dominant_mechanism(items: &[&Candidate]) -> String {
    let mut counts: BTreeMap<&str, u64> = BTreeMap::new();
    for i in items {
        *counts.entry(i.mechanism.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(m, _)| m.to_string())
        .unwrap_or_else(|| MECHANISM_SKIP.to_string())
}

fn honesty_warnings(has_ce: bool) -> Vec<String> {
    let mut w = vec![
        RETENTION_HONESTY_LEGACY_NOT_CE.to_string(),
        RETENTION_HONESTY_NOT_NIST_PURGE.to_string(),
        RETENTION_HONESTY_STREAM_INDEPENDENCE.to_string(),
        RETENTION_HONESTY_TICKET_NOT_CE.to_string(),
    ];
    if has_ce {
        w.insert(1, RETENTION_HONESTY_PRE_ERASE_BACKUP.to_string());
    }
    w
}

// ---------------------------------------------------------------------------
// Apply
// ---------------------------------------------------------------------------

/// Inputs for retention apply.
#[derive(Debug, Clone)]
pub struct RetentionApplyCommand {
    pub principal: Principal,
    pub scope: ScopeRef,
    pub command_id: String,
    pub confirm: bool,
    /// When true, plan only — refuse destructive work.
    pub dry_run: bool,
}

/// Outcome of projection-only apply (production CLI path).
///
/// CE keys are listed for daemon wipe; cascade + audit are completed via
/// [`finalize_retention_apply`] after CE.
#[derive(Debug, Clone)]
pub struct RetentionProjectionApplyOutcome {
    pub report: RetentionPlanReport,
    /// Sorted unique content_key_ids needing CE (daemon path).
    pub pending_ce_keys: Vec<String>,
    /// Memory subject ids for R15 cascade after successful CE.
    pub pending_cascade_memory_ids: Vec<String>,
}

/// Apply **projection** candidates only (local). Does not CE-wipe and does not
/// append RetentionApplied when CE is still pending.
///
/// Production CLI uses this so CE can go through the daemon (T165 parity) without
/// monomorphizing `wipe_content_envelope` into the CLI binary.
pub fn apply_retention_projections<W: EventWriter>(
    store: &SqliteEventStore,
    writer: &W,
    config: &RetentionConfig,
    command_id: &str,
    confirm: bool,
    dry_run: bool,
) -> Result<RetentionProjectionApplyOutcome> {
    if dry_run || !confirm {
        return Err(ControlPlaneError::InvalidPayload(
            "retention apply requires confirm=true and dry_run=false (R1)".into(),
        ));
    }

    let now = Utc::now();
    let generated_at = now.to_rfc3339();
    let candidates = collect_candidates(store, config, now)?;

    let mut errors: Vec<String> = Vec::new();
    let mut turns_to_delete: Vec<TurnKey> = Vec::new();
    let mut query_traces: Vec<String> = Vec::new();
    let mut reviews: Vec<String> = Vec::new();
    let mut decisions: Vec<String> = Vec::new();
    let mut ce_keys: Vec<String> = Vec::new();
    let mut ce_memory_ids: Vec<String> = Vec::new();

    for c in &candidates {
        match c.mechanism.as_str() {
            MECHANISM_HELD | MECHANISM_SKIP => {}
            MECHANISM_PROJECTION_DELETE => {
                if let Some(ref t) = c.turn {
                    turns_to_delete.push(t.clone());
                }
                if let Some(ref id) = c.query_trace_id {
                    query_traces.push(id.clone());
                }
                if let Some(ref id) = c.review_item_id {
                    reviews.push(id.clone());
                }
                if let Some(ref id) = c.decision_id {
                    decisions.push(id.clone());
                }
            }
            MECHANISM_CE_WIPE => {
                if let Some(ref key) = c.content_key_id {
                    ce_keys.push(key.clone());
                    ce_memory_ids.extend(c.memory_ids.iter().cloned());
                }
            }
            _ => {}
        }
    }

    ce_keys.sort();
    ce_keys.dedup();
    ce_memory_ids.sort();
    ce_memory_ids.dedup();

    {
        let conn = store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        if let Err(e) = ret_scan::delete_turns(&conn, &turns_to_delete) {
            errors.push(format!("delete_turns: {e}"));
        }
        if let Err(e) = ret_scan::delete_query_traces(&conn, &query_traces) {
            errors.push(format!("delete_query_traces: {e}"));
        }
        if let Err(e) = ret_scan::delete_review_items(&conn, &reviews) {
            errors.push(format!("delete_review_items: {e}"));
        }
        if let Err(e) = ret_scan::delete_decisions(&conn, &decisions) {
            errors.push(format!("delete_decisions: {e}"));
        }
    }

    let deferred_ce = !ce_keys.is_empty();
    let errors_count = errors.len() as u64;
    let report = build_report(
        RetentionReportMode::Apply,
        &generated_at,
        config,
        &candidates,
        RetentionCascade {
            parents_marked_for_resynthesis: 0,
        },
        errors_count,
        errors,
    );

    if !deferred_ce {
        // No CE pending: cascade N/A for CE, append audit now.
        append_retention_applied(writer, command_id, &report)?;
    }

    Ok(RetentionProjectionApplyOutcome {
        report,
        pending_ce_keys: ce_keys,
        pending_cascade_memory_ids: ce_memory_ids,
    })
}

/// Complete deferred-CE apply: R15 cascade + R12 RetentionApplied.
pub fn finalize_retention_apply<W: EventWriter>(
    store: &SqliteEventStore,
    writer: &W,
    command_id: &str,
    cascade_memory_ids: &[String],
    report: &mut RetentionPlanReport,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let parents_marked = {
        let conn = store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        ret_scan::mark_parents_for_resynthesis(&conn, cascade_memory_ids, &now)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?
    };
    report.cascade.parents_marked_for_resynthesis = report
        .cascade
        .parents_marked_for_resynthesis
        .saturating_add(parents_marked);
    report.errors_count = report.errors.len() as u64;
    append_retention_applied(writer, command_id, report)?;
    Ok(())
}

/// Apply a retention plan in-process (fixture / test path). Refuses without
/// `confirm && !dry_run` (R1).
///
/// CE candidates call [`wipe_content_envelope`] only (R2). Appends
/// [`Payload::RetentionApplied`] on successful apply path (R12).
///
/// **Production CLI must not use this for CE** — use
/// [`apply_retention_projections`] + daemon wipe instead (T165 E8 parity).
#[allow(clippy::too_many_arguments)]
pub fn apply_retention<W, Q, C, P, S>(
    store: &SqliteEventStore,
    writer: &W,
    query: &Q,
    clock: &C,
    policy: &P,
    wipe_side: &S,
    config: &RetentionConfig,
    cmd: RetentionApplyCommand,
) -> Result<RetentionPlanReport>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
    S: ContentEnvelopeWipeStore,
{
    if cmd.dry_run || !cmd.confirm {
        return Err(ControlPlaneError::InvalidPayload(
            "retention apply requires confirm=true and dry_run=false (R1)".into(),
        ));
    }

    let now = Utc::now();
    let generated_at = now.to_rfc3339();
    let candidates = collect_candidates(store, config, now)?;

    let mut errors: Vec<String> = Vec::new();
    let mut disposed_memory_ids: Vec<String> = Vec::new();
    let mut turns_to_delete: Vec<TurnKey> = Vec::new();
    let mut query_traces: Vec<String> = Vec::new();
    let mut reviews: Vec<String> = Vec::new();
    let mut decisions: Vec<String> = Vec::new();
    let mut ce_keys: Vec<String> = Vec::new();

    for c in &candidates {
        match c.mechanism.as_str() {
            MECHANISM_HELD | MECHANISM_SKIP => {}
            MECHANISM_PROJECTION_DELETE => {
                if let Some(ref t) = c.turn {
                    turns_to_delete.push(t.clone());
                }
                if let Some(ref id) = c.query_trace_id {
                    query_traces.push(id.clone());
                }
                if let Some(ref id) = c.review_item_id {
                    reviews.push(id.clone());
                }
                if let Some(ref id) = c.decision_id {
                    decisions.push(id.clone());
                }
            }
            MECHANISM_CE_WIPE => {
                if let Some(ref key) = c.content_key_id {
                    ce_keys.push(key.clone());
                    disposed_memory_ids.extend(c.memory_ids.iter().cloned());
                }
            }
            _ => {}
        }
    }

    ce_keys.sort();
    ce_keys.dedup();

    // Projection deletes (batch, local)
    {
        let conn = store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        if let Err(e) = ret_scan::delete_turns(&conn, &turns_to_delete) {
            errors.push(format!("delete_turns: {e}"));
        }
        if let Err(e) = ret_scan::delete_query_traces(&conn, &query_traces) {
            errors.push(format!("delete_query_traces: {e}"));
        }
        if let Err(e) = ret_scan::delete_review_items(&conn, &reviews) {
            errors.push(format!("delete_review_items: {e}"));
        }
        if let Err(e) = ret_scan::delete_decisions(&conn, &decisions) {
            errors.push(format!("delete_decisions: {e}"));
        }
    }

    // CE via T165 only (R2) — fixture / in-process path
    for key_str in &ce_keys {
        let content_key_id = match ContentKeyId::from_str(key_str) {
            Ok(k) => k,
            Err(e) => {
                errors.push(format!("invalid content_key_id {key_str}: {e}"));
                continue;
            }
        };
        let wipe_cmd = WipeContentEnvelopeCommand {
            principal: cmd.principal.clone(),
            content_key_id,
            scope: cmd.scope.clone(),
            reason: Some(format!("retention_apply:{}", cmd.command_id)),
            tombstone_id: None,
            dry_run: false,
            confirm: true,
        };
        match wipe_content_envelope(writer, query, clock, policy, wipe_side, wipe_cmd) {
            Ok(resp) => {
                if resp.status != "wiped" && resp.status != "already_erased" {
                    errors.push(format!(
                        "ce_wipe {key_str}: unexpected status {}",
                        resp.status
                    ));
                }
            }
            Err(e) => {
                errors.push(format!("ce_wipe {key_str}: {e}"));
            }
        }
    }

    // R15 cascade
    let parents_marked = {
        let conn = store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        ret_scan::mark_parents_for_resynthesis(&conn, &disposed_memory_ids, &generated_at)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?
    };

    let errors_count = errors.len() as u64;
    let report = build_report(
        RetentionReportMode::Apply,
        &generated_at,
        config,
        &candidates,
        RetentionCascade {
            parents_marked_for_resynthesis: parents_marked,
        },
        errors_count,
        errors.clone(),
    );

    // R12 RetentionApplied audit
    append_retention_applied(writer, &cmd.command_id, &report)?;

    // Non-zero CE failures → error after audit (spec: aggregate errors; non-zero if any CE fail)
    let ce_failed = errors.iter().any(|e| e.starts_with("ce_wipe "));
    if ce_failed {
        return Err(ControlPlaneError::Query(format!(
            "retention apply completed with {errors_count} error(s); CE failure(s) present"
        )));
    }

    Ok(report)
}

fn append_retention_applied<W: EventWriter>(
    writer: &W,
    command_id: &str,
    report: &RetentionPlanReport,
) -> Result<()> {
    let agg_id = id_from_command(NS_RETENTION_APPLY, command_id);
    let class_counts: Vec<RetentionClassCount> = report
        .classes
        .iter()
        .map(|c| RetentionClassCount {
            class: c.class.clone(),
            count: c.candidate_count,
            mechanism: c.mechanism.clone(),
        })
        .collect();
    let mut sample_ids = Vec::new();
    for c in &report.classes {
        for s in &c.sample_ids {
            if sample_ids.len() >= 5 {
                break;
            }
            sample_ids.push(truncate_id(s));
        }
    }
    let payload = Payload::RetentionApplied(RetentionAppliedPayload {
        command_id: command_id.to_string(),
        mode: "apply".into(),
        class_counts,
        would_ce_wipe: report.totals.would_ce_wipe,
        would_projection_delete: report.totals.would_projection_delete,
        would_skip: report.totals.would_skip,
        would_held: report.totals.would_held,
        errors_count: report.errors_count,
        sample_ids,
    });
    let env = build_event(
        AggregateType::Job,
        agg_id,
        Actor::System,
        Privacy::LocalOnly,
        payload,
    )?;
    writer.append_events(&[env])?;
    Ok(())
}
