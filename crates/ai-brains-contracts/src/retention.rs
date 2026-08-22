//! Class-based retention plan/apply wire surface (T166 / P8.4).
//!
//! Dry-run first (R1). Envelope disposal reuses T165 CE wipe only (R2).
//! Legacy projection delete is **never** labeled cryptographic erasure (R3).
//! Reports carry counts / truncated ids only — no plaintext bodies (R4).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const API_VERSION: &str = "1";

// ---------------------------------------------------------------------------
// Canonical class ids (R5)
// ---------------------------------------------------------------------------

pub const CLASS_RAW_TURN: &str = "raw_turn";
pub const CLASS_EVIDENCE: &str = "evidence";
pub const CLASS_DECISION_APPROVED: &str = "decision_approved";
pub const CLASS_SECRET: &str = "secret";
pub const CLASS_REVIEW_TRACE: &str = "review_trace";
pub const CLASS_QUERY_TRACE: &str = "query_trace";
pub const CLASS_MEMORY_LEGACY: &str = "memory_legacy";
pub const CLASS_ORPHANED_ENVELOPE: &str = "orphaned_envelope";
pub const CLASS_UNCLASSIFIED: &str = "unclassified";

/// Frozen v1 class set (R5).
pub const CANONICAL_CLASSES: &[&str] = &[
    CLASS_RAW_TURN,
    CLASS_EVIDENCE,
    CLASS_DECISION_APPROVED,
    CLASS_SECRET,
    CLASS_REVIEW_TRACE,
    CLASS_QUERY_TRACE,
    CLASS_MEMORY_LEGACY,
    CLASS_ORPHANED_ENVELOPE,
    CLASS_UNCLASSIFIED,
];

/// True when `class` is a known retention class id.
pub fn is_canonical_class(class: &str) -> bool {
    CANONICAL_CLASSES.contains(&class)
}

// ---------------------------------------------------------------------------
// Mechanisms
// ---------------------------------------------------------------------------

pub const MECHANISM_PROJECTION_DELETE: &str = "projection_delete";
pub const MECHANISM_CE_WIPE: &str = "ce_wipe";
pub const MECHANISM_SOFT_FORGET: &str = "soft_forget";
pub const MECHANISM_SKIP: &str = "skip";
pub const MECHANISM_HELD: &str = "held";

// ---------------------------------------------------------------------------
// Honesty / residual warnings (R3, R4 family + stream independence)
// ---------------------------------------------------------------------------

/// Projection delete is not CE (R3).
pub const RETENTION_HONESTY_LEGACY_NOT_CE: &str =
    "legacy projection delete is not cryptographic erasure";

/// Pre-erase backups residual when CE candidates present (T165 family).
pub const RETENTION_HONESTY_PRE_ERASE_BACKUP: &str =
    "pre-erase backups, exports, and offline copies remain decryptable if restored";

/// Not NIST media Purge.
pub const RETENTION_HONESTY_NOT_NIST_PURGE: &str =
    "not NIST Purge/Destroy; not physical media sanitization (TRUNCATE is not Purge)";

/// Stream A turns and stream B keys are independent until subject join exists (R13).
pub const RETENTION_HONESTY_STREAM_INDEPENDENCE: &str =
    "stream_a_and_stream_b_independent_until_subject_join";

/// Soft forget / ticket are not CE.
pub const RETENTION_HONESTY_TICKET_NOT_CE: &str =
    "erasure ticket and soft forget are not cryptographic erasure";

/// Live `memory_projection` inventory overlay (T270). Pins are held, not auto-forgotten.
pub const RETENTION_HONESTY_MEMORY_LEGACY_INVENTORY: &str =
    "memory_legacy inventory is none_auto; pins held; apply does not auto-forget";

/// Future turn↔envelope join convention (document only until capture seals turns).
pub const TURN_ENVELOPE_SUBJECT_KIND: &str = "turn";

/// Build `subject_id` for a sealed turn: `{session_id}:{turn_index}`.
pub fn turn_envelope_subject_id(session_id: &str, turn_index: i64) -> String {
    format!("{session_id}:{turn_index}")
}

fn default_api_version() -> String {
    API_VERSION.to_string()
}

// ---------------------------------------------------------------------------
// Report DTOs
// ---------------------------------------------------------------------------

/// Plan or apply mode for [`RetentionPlanReport`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionReportMode {
    DryRun,
    Apply,
}

impl RetentionReportMode {
    pub fn as_str(self) -> &'static str {
        match self {
            RetentionReportMode::DryRun => "dry_run",
            RetentionReportMode::Apply => "apply",
        }
    }
}

/// Per-class horizon labels for the report (`days` or policy text).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetentionHorizons {
    /// Class id → horizon label (e.g. `"90"`, `"none_auto"`, `"revoked+30d"`).
    #[serde(default, flatten)]
    pub by_class: BTreeMap<String, String>,
}

/// One class bucket in the plan/apply report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetentionClassBucket {
    pub class: String,
    pub candidate_count: u64,
    /// `projection_delete` | `ce_wipe` | `soft_forget` | `skip` | `held`
    pub mechanism: String,
    /// Truncated UUIDs / composite keys only — max 5 (R4).
    #[serde(default)]
    pub sample_ids: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    /// Per-class CE candidates (T284). Omitted when 0 so inventory JSON stays five keys.
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub would_ce_wipe: u64,
    /// Per-class projection-delete candidates (T284). Omitted when 0.
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub would_projection_delete: u64,
    /// Dispose identities (CE first, then projection). Cap 5. Omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dispose_sample_ids: Vec<String>,
}

fn is_zero_u64(n: &u64) -> bool {
    *n == 0
}

/// Sum of class-level dispose counters (CE + projection). Not the dominant `mechanism`.
///
/// `pub` so control-plane `audit_sample_ids` can share the sum (F27's `pub(crate)`
/// would be crate-local to contracts and unused in the lib).
pub fn class_dispose_count(bucket: &RetentionClassBucket) -> u64 {
    bucket
        .would_ce_wipe
        .saturating_add(bucket.would_projection_delete)
}

/// Aggregate totals across classes (no double-count of same identity — R13).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetentionTotals {
    pub candidates: u64,
    pub would_ce_wipe: u64,
    pub would_projection_delete: u64,
    pub would_skip: u64,
    pub would_held: u64,
}

/// Cascade estimate / result for hierarchy parents (R15).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetentionCascade {
    pub parents_marked_for_resynthesis: u64,
}

/// Class-based retention plan or apply report (T166).
///
/// **E1:** empty vault → zero counts, not error.
/// **Forbidden:** embedding full memory/turn content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetentionPlanReport {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    /// RFC3339 timestamp.
    pub generated_at: String,
    /// `dry_run` | `apply`
    pub mode: String,
    pub horizons: BTreeMap<String, String>,
    #[serde(default)]
    pub classes: Vec<RetentionClassBucket>,
    pub totals: RetentionTotals,
    #[serde(default)]
    pub cascade: RetentionCascade,
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Apply path: count of per-candidate errors (CE failures, etc.).
    #[serde(default)]
    pub errors_count: u64,
    /// Apply path: non-secret error messages (no bodies).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

impl RetentionPlanReport {
    /// Empty-vault / zero-work report (E1).
    pub fn empty(mode: RetentionReportMode, generated_at: impl Into<String>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            generated_at: generated_at.into(),
            mode: mode.as_str().to_string(),
            horizons: default_horizon_labels(),
            classes: Vec::new(),
            totals: RetentionTotals::default(),
            cascade: RetentionCascade::default(),
            warnings: base_honesty_warnings(false),
            errors_count: 0,
            errors: Vec::new(),
        }
    }

    /// Base honesty warnings always appropriate for plan/apply reports.
    pub fn honesty_warnings(has_ce_candidates: bool) -> Vec<String> {
        base_honesty_warnings(has_ce_candidates)
    }
}

/// Default horizon labels matching v1 class matrix.
pub fn default_horizon_labels() -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert(CLASS_RAW_TURN.to_string(), "90".into());
    m.insert(CLASS_EVIDENCE.to_string(), "365".into());
    m.insert(CLASS_SECRET.to_string(), "7".into());
    m.insert(CLASS_QUERY_TRACE.to_string(), "30".into());
    m.insert(CLASS_REVIEW_TRACE.to_string(), "90".into());
    m.insert(
        CLASS_DECISION_APPROVED.to_string(),
        "revoked_superseded+30d_cooldown".into(),
    );
    m.insert(CLASS_MEMORY_LEGACY.to_string(), "none_auto".into());
    m.insert(CLASS_ORPHANED_ENVELOPE.to_string(), "7".into());
    m.insert(CLASS_UNCLASSIFIED.to_string(), "skip_apply".into());
    m
}

fn base_honesty_warnings(has_ce_candidates: bool) -> Vec<String> {
    let mut w = vec![
        RETENTION_HONESTY_LEGACY_NOT_CE.to_string(),
        RETENTION_HONESTY_NOT_NIST_PURGE.to_string(),
        RETENTION_HONESTY_STREAM_INDEPENDENCE.to_string(),
        RETENTION_HONESTY_TICKET_NOT_CE.to_string(),
    ];
    if has_ce_candidates {
        w.insert(1, RETENTION_HONESTY_PRE_ERASE_BACKUP.to_string());
    }
    w
}

/// Cap sample ids at 5 and truncate each id for reports (R4).
pub fn truncate_sample_ids(ids: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
    ids.into_iter()
        .take(5)
        .map(|s| truncate_id(s.as_ref()))
        .collect()
}

/// Truncate a single identity for report samples (keep prefix, no bodies).
pub fn truncate_id(id: &str) -> String {
    const MAX: usize = 36;
    if id.chars().count() <= MAX {
        id.to_string()
    } else {
        let prefix: String = id.chars().take(MAX).collect();
        format!("{prefix}…")
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn retention_plan_report__empty__zero_counts() {
        let r = RetentionPlanReport::empty(RetentionReportMode::DryRun, "2026-07-29T00:00:00Z");
        assert_eq!(r.api_version, API_VERSION);
        assert_eq!(r.mode, "dry_run");
        assert_eq!(r.totals.candidates, 0);
        assert_eq!(r.totals.would_ce_wipe, 0);
        assert!(r.classes.is_empty());
        let json = serde_json::to_string(&r).expect("ser");
        assert!(!json.contains("content"));
        assert!(!json.contains("body"));
    }

    #[test]
    fn retention_plan_report__roundtrip() {
        let mut horizons = default_horizon_labels();
        horizons.insert(CLASS_RAW_TURN.to_string(), "90".into());
        let r = RetentionPlanReport {
            api_version: API_VERSION.to_string(),
            generated_at: "2026-07-29T12:00:00Z".into(),
            mode: "dry_run".into(),
            horizons,
            classes: vec![RetentionClassBucket {
                class: CLASS_RAW_TURN.into(),
                candidate_count: 2,
                mechanism: MECHANISM_PROJECTION_DELETE.into(),
                sample_ids: vec!["sess:0".into()],
                notes: vec!["event log retained".into()],
                would_ce_wipe: 0,
                would_projection_delete: 2,
                dispose_sample_ids: vec!["sess:0".into()],
            }],
            totals: RetentionTotals {
                candidates: 2,
                would_ce_wipe: 0,
                would_projection_delete: 2,
                would_skip: 0,
                would_held: 0,
            },
            cascade: RetentionCascade::default(),
            warnings: RetentionPlanReport::honesty_warnings(false),
            errors_count: 0,
            errors: Vec::new(),
        };
        let json = serde_json::to_string(&r).expect("ser");
        let decoded: RetentionPlanReport = serde_json::from_str(&json).expect("de");
        assert_eq!(decoded, r);
    }

    #[test]
    fn retention_class_bucket__zero_dispose__json_keys_exactly_five() {
        let b = RetentionClassBucket {
            class: CLASS_MEMORY_LEGACY.into(),
            candidate_count: 1,
            mechanism: MECHANISM_HELD.into(),
            sample_ids: vec!["aaaaaaaa-aaaa-aaaa-aaaa-000000000001".into()],
            notes: vec!["inventory".into()],
            would_ce_wipe: 0,
            would_projection_delete: 0,
            dispose_sample_ids: Vec::new(),
        };
        let value = serde_json::to_value(&b).expect("ser");
        let obj = value.as_object().expect("object");
        let mut keys: Vec<&String> = obj.keys().collect();
        keys.sort();
        assert_eq!(
            keys,
            [
                "candidate_count",
                "class",
                "mechanism",
                "notes",
                "sample_ids"
            ]
        );
        assert!(!obj.contains_key("would_ce_wipe"));
        assert!(!obj.contains_key("would_projection_delete"));
        assert!(!obj.contains_key("dispose_sample_ids"));
        assert_eq!(API_VERSION, "1");
        assert_eq!(class_dispose_count(&b), 0);
    }

    #[test]
    fn retention_report__contains_honesty_warnings() {
        let w = RetentionPlanReport::honesty_warnings(true);
        let joined = w.join(" ");
        assert!(joined.contains("not cryptographic erasure") || joined.contains("legacy"));
        assert!(joined.to_ascii_lowercase().contains("purge"));
        assert!(joined.to_ascii_lowercase().contains("backup"));
        assert!(joined.contains("stream_a_and_stream_b"));
    }

    #[test]
    fn is_canonical_class__known_and_unknown() {
        assert!(is_canonical_class(CLASS_SECRET));
        assert!(!is_canonical_class("mystery_blob"));
    }

    #[test]
    fn turn_envelope_subject_id__join_convention() {
        assert_eq!(
            turn_envelope_subject_id("00000000-0000-0000-0000-000000000001", 3),
            "00000000-0000-0000-0000-000000000001:3"
        );
    }
}
