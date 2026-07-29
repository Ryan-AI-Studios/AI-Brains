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

/// Maximum accepted horizon days from env (~100 years). Larger values fall back
/// to the class default so `chrono::Duration` construction cannot overflow.
pub const MAX_RETENTION_HORIZON_DAYS: i64 = 36_500;

impl RetentionConfig {
    /// Load from `AI_BRAINS_RETENTION_*` env vars; missing keys keep defaults.
    ///
    /// Invalid overrides (non-integer, ≤0, or > [`MAX_RETENTION_HORIZON_DAYS`])
    /// fall back to the class default. Negative values are never applied (no
    /// future cutoffs / no chrono panic).
    pub fn from_env() -> Self {
        let d = Self::default();
        Self {
            raw_turn_days: env_horizon_days("AI_BRAINS_RETENTION_RAW_TURN_DAYS", d.raw_turn_days),
            evidence_days: env_horizon_days("AI_BRAINS_RETENTION_EVIDENCE_DAYS", d.evidence_days),
            secret_days: env_horizon_days("AI_BRAINS_RETENTION_SECRET_DAYS", d.secret_days),
            query_trace_days: env_horizon_days(
                "AI_BRAINS_RETENTION_QUERY_TRACE_DAYS",
                d.query_trace_days,
            ),
            review_trace_days: env_horizon_days(
                "AI_BRAINS_RETENTION_REVIEW_TRACE_DAYS",
                d.review_trace_days,
            ),
            decision_revoked_cooldown_days: env_horizon_days(
                "AI_BRAINS_RETENTION_DECISION_REVOKED_COOLDOWN_DAYS",
                d.decision_revoked_cooldown_days,
            ),
            orphan_envelope_days: env_horizon_days(
                "AI_BRAINS_RETENTION_ORPHAN_ENVELOPE_DAYS",
                d.orphan_envelope_days,
            ),
            apply_ce_on_nightly: parse_env_bool("AI_BRAINS_RETENTION_APPLY_CE")
                || parse_env_bool("AI_BRAINS_RETENTION_APPLY_CE_ON_NIGHTLY"),
        }
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

/// Parse a retention horizon day count: must be in `1..=MAX_RETENTION_HORIZON_DAYS`
/// and constructible as a chrono duration.
pub fn parse_positive_horizon_days(raw: &str) -> std::result::Result<i64, &'static str> {
    let v: i64 = raw
        .trim()
        .parse()
        .map_err(|_| "horizon days must be an integer")?;
    if v <= 0 {
        return Err("horizon days must be > 0");
    }
    if v > MAX_RETENTION_HORIZON_DAYS {
        return Err("horizon days exceed maximum (36500)");
    }
    if Duration::try_days(v).is_none() {
        return Err("horizon days overflow Duration");
    }
    Ok(v)
}

fn env_horizon_days(key: &str, default: i64) -> i64 {
    match std::env::var(key) {
        Err(_) => default,
        Ok(s) => match parse_positive_horizon_days(&s) {
            Ok(v) => v,
            Err(_) => default,
        },
    }
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

/// RFC3339 cutoff for `now - days` using checked chrono arithmetic (no panic).
///
/// On impossible durations (should not occur after config sanitization), falls
/// back to the Unix epoch so the scan selects almost nothing rather than everything.
fn cutoff_days_before(now: chrono::DateTime<Utc>, days: i64) -> String {
    match Duration::try_days(days).and_then(|d| now.checked_sub_signed(d)) {
        Some(t) => t.to_rfc3339(),
        None => "1970-01-01T00:00:00+00:00".to_string(),
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
    let turn_cutoff = cutoff_days_before(now, config.raw_turn_days);
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
    let qt_cutoff = cutoff_days_before(now, config.query_trace_days);
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
    let rt_cutoff = cutoff_days_before(now, config.review_trace_days);
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
    let dec_cutoff = cutoff_days_before(now, config.decision_revoked_cooldown_days);
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
        let orphan_cutoff = cutoff_days_before(now, config.orphan_envelope_days);
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

    let cutoff = cutoff_days_before(now, horizon_days);
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

/// Outcome of retention prepare (production CLI path).
///
/// Holds the pre-mutation audit report plus deferred CE keys and projection
/// delete actions. **No** projection deletes run in [`prepare_retention_apply`].
///
/// Production CE-first order when `pending_ce_keys` is non-empty:
/// 1. prepare (audit only)
/// 2. daemon CE wipe batch (track successful keys)
/// 3. [`execute_retention_projection_deletes`]
/// 4. [`finalize_retention_apply`] for successful CE only
///
/// Projection-only (`pending_ce_keys` empty): prepare then execute deletes.
#[derive(Debug, Clone)]
pub struct RetentionProjectionApplyOutcome {
    pub report: RetentionPlanReport,
    /// Sorted unique content_key_ids needing CE (daemon path).
    pub pending_ce_keys: Vec<String>,
    /// content_key_id → memory subject ids for R15 cascade after **successful** CE.
    pub pending_cascade_by_key: BTreeMap<String, Vec<String>>,
    /// Deferred stream-A turn projection deletes (not applied until execute).
    pub turns_to_delete: Vec<TurnKey>,
    /// Deferred query_trace projection deletes.
    pub query_traces_to_delete: Vec<String>,
    /// Deferred review_item projection deletes.
    pub reviews_to_delete: Vec<String>,
    /// Deferred decision projection deletes.
    pub decisions_to_delete: Vec<String>,
}

/// Audit only + collect CE / projection actions; **no** deletes or CE wipe.
///
/// R12 durability: builds the apply report from planned candidates and appends
/// `RetentionApplied` **before** any mutation. If the audit append fails, the
/// caller must not wipe or delete. When CE keys remain pending for the daemon,
/// a warning is recorded; finalize after wipe appends a second audit.
pub fn prepare_retention_apply<W: EventWriter>(
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

    let mut turns_to_delete: Vec<TurnKey> = Vec::new();
    let mut query_traces: Vec<String> = Vec::new();
    let mut reviews: Vec<String> = Vec::new();
    let mut decisions: Vec<String> = Vec::new();
    let mut ce_keys: Vec<String> = Vec::new();
    let mut cascade_by_key: BTreeMap<String, Vec<String>> = BTreeMap::new();

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
                    if !c.memory_ids.is_empty() {
                        cascade_by_key
                            .entry(key.clone())
                            .or_default()
                            .extend(c.memory_ids.iter().cloned());
                    }
                }
            }
            _ => {}
        }
    }

    ce_keys.sort();
    ce_keys.dedup();
    for ids in cascade_by_key.values_mut() {
        ids.sort();
        ids.dedup();
    }

    // Planned report first (counts from candidates; no mutation yet).
    let deferred_ce = !ce_keys.is_empty();
    let mut report = build_report(
        RetentionReportMode::Apply,
        &generated_at,
        config,
        &candidates,
        RetentionCascade {
            parents_marked_for_resynthesis: 0,
        },
        0,
        Vec::new(),
    );

    if deferred_ce {
        report.warnings.push(format!(
            "ce_pending={} (RetentionApplied pre-mutation; CE-first then finalize after wipe)",
            ce_keys.len()
        ));
    }

    // R12: audit BEFORE any delete/wipe. If append fails, do not mutate.
    append_retention_applied(writer, command_id, &report)?;

    Ok(RetentionProjectionApplyOutcome {
        report,
        pending_ce_keys: ce_keys,
        pending_cascade_by_key: cascade_by_key,
        turns_to_delete,
        query_traces_to_delete: query_traces,
        reviews_to_delete: reviews,
        decisions_to_delete: decisions,
    })
}

/// Run deferred projection deletes recorded in `outcome`; merge errors into
/// `outcome.report`. Safe to call after CE wipe (CE-first production order).
pub fn execute_retention_projection_deletes(
    store: &SqliteEventStore,
    outcome: &mut RetentionProjectionApplyOutcome,
) -> Result<()> {
    let mut errors: Vec<String> = Vec::new();
    {
        let conn = store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        if let Err(e) = ret_scan::delete_turns(&conn, &outcome.turns_to_delete) {
            errors.push(format!("delete_turns: {e}"));
        }
        if let Err(e) = ret_scan::delete_query_traces(&conn, &outcome.query_traces_to_delete) {
            errors.push(format!("delete_query_traces: {e}"));
        }
        if let Err(e) = ret_scan::delete_review_items(&conn, &outcome.reviews_to_delete) {
            errors.push(format!("delete_review_items: {e}"));
        }
        if let Err(e) = ret_scan::delete_decisions(&conn, &outcome.decisions_to_delete) {
            errors.push(format!("delete_decisions: {e}"));
        }
    }
    if !errors.is_empty() {
        outcome.report.errors.extend(errors);
        outcome.report.errors_count = outcome.report.errors.len() as u64;
    }
    Ok(())
}

/// Convenience: prepare (audit) then execute projection deletes.
///
/// Does **not** CE-wipe. Prefer the split path in production CLI so CE can run
/// **before** projection deletes (CE-first). Fixture/tests may use this for
/// projection-only apply.
///
/// Production CLI uses prepare → daemon CE → execute → finalize so CE can go
/// through the daemon (T165 parity) without monomorphizing
/// `wipe_content_envelope` into the CLI binary.
pub fn apply_retention_projections<W: EventWriter>(
    store: &SqliteEventStore,
    writer: &W,
    config: &RetentionConfig,
    command_id: &str,
    confirm: bool,
    dry_run: bool,
) -> Result<RetentionProjectionApplyOutcome> {
    let mut outcome = prepare_retention_apply(store, writer, config, command_id, confirm, dry_run)?;
    execute_retention_projection_deletes(store, &mut outcome)?;
    Ok(outcome)
}

/// Complete deferred-CE apply: R15 cascade for **successful** CE subjects only,
/// then append a final R12 `RetentionApplied` (second event when pre-CE audit
/// already exists — durable final tallies / errors).
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

/// Collect memory subject ids for R15 cascade from keys that wiped successfully.
pub fn cascade_memory_ids_for_keys(
    cascade_by_key: &BTreeMap<String, Vec<String>>,
    successful_keys: impl IntoIterator<Item = impl AsRef<str>>,
) -> Vec<String> {
    let mut out = Vec::new();
    for key in successful_keys {
        if let Some(ids) = cascade_by_key.get(key.as_ref()) {
            out.extend(ids.iter().cloned());
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Apply a retention plan in-process (fixture / test path). Refuses without
/// `confirm && !dry_run` (R1).
///
/// CE candidates call [`wipe_content_envelope`] only (R2). Appends planned
/// [`Payload::RetentionApplied`] **before** any wipe/delete (R12). Order is
/// CE-first (wipe, then projection deletes, then R15 cascade) so policy-denied
/// CE cannot leave projection deletes already applied — same as production.
///
/// **Production CLI must not use this for CE** — use
/// [`prepare_retention_apply`] + daemon wipe +
/// [`execute_retention_projection_deletes`] + [`finalize_retention_apply`]
/// (T165 E8 parity).
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

    let mut turns_to_delete: Vec<TurnKey> = Vec::new();
    let mut query_traces: Vec<String> = Vec::new();
    let mut reviews: Vec<String> = Vec::new();
    let mut decisions: Vec<String> = Vec::new();
    let mut ce_keys: Vec<String> = Vec::new();
    let mut cascade_by_key: BTreeMap<String, Vec<String>> = BTreeMap::new();

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
                    if !c.memory_ids.is_empty() {
                        cascade_by_key
                            .entry(key.clone())
                            .or_default()
                            .extend(c.memory_ids.iter().cloned());
                    }
                }
            }
            _ => {}
        }
    }

    ce_keys.sort();
    ce_keys.dedup();
    for ids in cascade_by_key.values_mut() {
        ids.sort();
        ids.dedup();
    }

    // 1–2) Planned report + R12 audit BEFORE any wipe/delete (CE-first order).
    let mut report = build_report(
        RetentionReportMode::Apply,
        &generated_at,
        config,
        &candidates,
        RetentionCascade {
            parents_marked_for_resynthesis: 0,
        },
        0,
        Vec::new(),
    );
    if !ce_keys.is_empty() {
        report.warnings.push(format!(
            "ce_pending={} (RetentionApplied pre-mutation; CE-first wipe then projections)",
            ce_keys.len()
        ));
    }
    append_retention_applied(writer, &cmd.command_id, &report)?;

    let mut errors: Vec<String> = Vec::new();

    // 3) CE via T165 only (R2) — fixture / in-process path, **before** projections
    //    so policy-denied CE cannot leave projection deletes already applied.
    let mut successful_ce_keys: Vec<String> = Vec::new();
    for key_str in &ce_keys {
        let key_disp = truncate_id(key_str);
        let content_key_id = match ContentKeyId::from_str(key_str) {
            Ok(k) => k,
            Err(e) => {
                errors.push(format!("invalid content_key_id {key_disp}: {e}"));
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
                if resp.status == "wiped" || resp.status == "already_erased" {
                    successful_ce_keys.push(key_str.clone());
                } else {
                    errors.push(format!(
                        "ce_wipe {key_disp}: unexpected status {}",
                        resp.status
                    ));
                }
            }
            Err(e) => {
                // Codex R2 P3: do not echo raw error Display (may embed full key).
                errors.push(format!("ce_wipe {key_disp}: {}", ce_wipe_error_code(&e)));
            }
        }
    }

    // 4) Projection deletes after CE batch (CE-first consistency with production).
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

    // 5) R15 cascade — only subjects of successful CE keys
    let disposed_memory_ids = cascade_memory_ids_for_keys(&cascade_by_key, &successful_ce_keys);
    let parents_marked = {
        let conn = store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        ret_scan::mark_parents_for_resynthesis(&conn, &disposed_memory_ids, &generated_at)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?
    };

    report.cascade.parents_marked_for_resynthesis = parents_marked;
    report.errors = errors.clone();
    report.errors_count = report.errors.len() as u64;

    // Non-zero CE failures → error after pre-mutation audit (spec: non-zero if any CE fail)
    let ce_failed = errors.iter().any(|e| e.starts_with("ce_wipe "));
    if ce_failed {
        return Err(ControlPlaneError::Query(format!(
            "retention apply completed with {} error(s); CE failure(s) present",
            report.errors_count
        )));
    }

    Ok(report)
}

/// Stable short code for CE wipe failures (avoids full content_key_id in reports).
fn ce_wipe_error_code(e: &ControlPlaneError) -> &'static str {
    match e {
        ControlPlaneError::NotEnvelopeBacked(_) => "not_envelope_backed",
        ControlPlaneError::PolicyDenied(_) => "policy_denied",
        ControlPlaneError::InvalidPayload(_) => "invalid_payload",
        ControlPlaneError::Query(_) => "query_failed",
        ControlPlaneError::EventAppend(_) => "event_append_failed",
        ControlPlaneError::NotFound(_) => "not_found",
        ControlPlaneError::ApprovalRequired(_) => "approval_required",
        _ => "wipe_failed",
    }
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
