//! Briefing DTOs and typed Project / Personal packets (T152).
//!
//! **Hard rules**
//! - Personal sections are **never** nested inside [`ProjectBriefingPacket`].
//! - Stale / Disputed / Rejected conclusions appear only under warnings, not current lists.
//! - Every authoritative decision/conclusion entry carries ≥1 evidence handle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::knowledge::EvidenceHandle;

pub const API_VERSION: &str = "1";

// ---------------------------------------------------------------------------
// Shell (backward compatible)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingDto {
    pub id: String,
    pub kind: String,
    /// Evidence handles (ids + optional labels), not prose-only body.
    #[serde(default)]
    pub evidence_handles: Vec<EvidenceHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingResponse {
    pub api_version: String,
    pub briefing: BriefingDto,
}

impl BriefingResponse {
    pub fn new(briefing: BriefingDto) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            briefing,
        }
    }
}

// ---------------------------------------------------------------------------
// Shared packet pieces
// ---------------------------------------------------------------------------

/// Budget metering for a generated briefing packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BudgetReportDto {
    pub max_words: usize,
    pub used_words: usize,
    /// Section names dropped or truncated (snake_case).
    #[serde(default)]
    pub truncated_sections: Vec<String>,
    pub more_available: bool,
}

/// Resolved scope summary embedded in a Project briefing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BriefingScopeDto {
    /// Scope identity key, e.g. `Repository:{uuid}`.
    pub scope_key: String,
    /// High | Medium | Low | Ambiguous
    pub confidence: String,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub alternatives: Vec<String>,
    /// True when callers may inject high-authority claims for this scope.
    pub authoritative: bool,
}

/// Compact claim entry (decision or conclusion) with required evidence handles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BriefingClaimDto {
    pub id: String,
    /// Decision | Conclusion
    pub kind: String,
    pub statement: String,
    /// Approved | Active | Confirmed (current authority only).
    pub state: String,
    #[serde(default)]
    pub evidence_handles: Vec<EvidenceHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Constraint / invariant entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BriefingConstraintDto {
    pub id: String,
    pub statement: String,
    #[serde(default)]
    pub evidence_handles: Vec<EvidenceHandle>,
}

/// Non-current or risk signal (stale, disputed, open conflict, unavailable, denied).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BriefingWarningDto {
    /// stale | disputed | open_conflict | unavailable | denied | low_confidence | other
    pub kind: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_kind: Option<String>,
}

/// Aggregate freshness summary for sources feeding the packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FreshnessSummaryDto {
    pub total_sources: u32,
    pub fresh_count: u32,
    pub stale_count: u32,
    pub unavailable_count: u32,
    /// Best-effort worst state label (Fresh | Stale | Unavailable | Unknown).
    pub worst_state: String,
}

/// Optional Ledgerful blend (degrades to null/empty on failure).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerfulSectionDto {
    #[serde(default)]
    pub hotspots: Vec<String>,
    #[serde(default)]
    pub impact_notes: Vec<String>,
    /// True when the bridge failed and the section is empty/degraded.
    #[serde(default)]
    pub degraded: bool,
}

/// Optional active handoff / session summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandoffSectionDto {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub session_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// Project briefing packet
// ---------------------------------------------------------------------------

/// Cold-start Project Briefing packet.
///
/// **Does not** embed Personal continuity sections. Personal is a separate packet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBriefingPacket {
    pub api_version: String,
    pub briefing_id: String,
    /// Always `"Project"` for this packet type.
    pub kind: String,
    pub scope: BriefingScopeDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff: Option<HandoffSectionDto>,
    #[serde(default)]
    pub decisions: Vec<BriefingClaimDto>,
    #[serde(default)]
    pub conclusions: Vec<BriefingClaimDto>,
    #[serde(default)]
    pub constraints: Vec<BriefingConstraintDto>,
    #[serde(default)]
    pub warnings: Vec<BriefingWarningDto>,
    pub freshness: FreshnessSummaryDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledgerful: Option<LedgerfulSectionDto>,
    /// Flattened evidence handles cited by authoritative claims (and extras).
    #[serde(default)]
    pub evidence_handles: Vec<EvidenceHandle>,
    pub budget: BudgetReportDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<DateTime<Utc>>,
    /// When policy denied ReadDecisions/ReadConclusions for the principal.
    #[serde(default)]
    pub denied: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denial_reason: Option<String>,
}

impl ProjectBriefingPacket {
    pub fn empty_denied(
        briefing_id: String,
        scope: BriefingScopeDto,
        reason: impl Into<String>,
    ) -> Self {
        // F7/M2: seed kind=denied so every denied path has ≥1 structured warning
        // (covers Personal-scope refuse and bare helper returns).
        let reason = reason.into();
        Self {
            api_version: API_VERSION.to_string(),
            briefing_id,
            kind: "Project".to_string(),
            scope,
            handoff: None,
            decisions: Vec::new(),
            conclusions: Vec::new(),
            constraints: Vec::new(),
            warnings: vec![BriefingWarningDto {
                kind: "denied".into(),
                message: reason.clone(),
                subject_id: None,
                subject_kind: None,
            }],
            freshness: FreshnessSummaryDto {
                total_sources: 0,
                fresh_count: 0,
                stale_count: 0,
                unavailable_count: 0,
                worst_state: "Unknown".to_string(),
            },
            ledgerful: None,
            evidence_handles: Vec::new(),
            budget: BudgetReportDto {
                max_words: 0,
                used_words: 0,
                truncated_sections: Vec::new(),
                more_available: false,
            },
            generated_at: None,
            denied: true,
            denial_reason: Some(reason),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBriefingResponse {
    pub api_version: String,
    pub packet: ProjectBriefingPacket,
}

impl ProjectBriefingResponse {
    pub fn new(packet: ProjectBriefingPacket) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            packet,
        }
    }
}

// ---------------------------------------------------------------------------
// Personal continuity briefing packet
// ---------------------------------------------------------------------------

/// Preference / personal constraint (Personal scope only).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonalPreferenceDto {
    pub id: String,
    pub statement: String,
    #[serde(default)]
    pub evidence_handles: Vec<EvidenceHandle>,
}

/// Compact continuity thread summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContinuitySummaryDto {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub thread_handles: Vec<String>,
}

/// Open review item in personal continuity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonalReviewItemDto {
    pub id: String,
    pub subject: String,
    pub criticality: String,
    pub status: String,
}

/// Grant that authorized this personal packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppliedGrantDto {
    pub grant_id: String,
    pub scope_key: String,
    pub capability: String,
    pub privacy: String,
}

/// Personal Continuity Briefing — never nested inside a Project packet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalContinuityBriefingPacket {
    pub api_version: String,
    pub briefing_id: String,
    /// Always `"Personal"` for this packet type.
    pub kind: String,
    /// Personal scope key `Personal:{user_id}`.
    pub scope_key: String,
    #[serde(default)]
    pub preferences: Vec<PersonalPreferenceDto>,
    #[serde(default)]
    pub continuity: ContinuitySummaryDto,
    #[serde(default)]
    pub open_review_items: Vec<PersonalReviewItemDto>,
    #[serde(default)]
    pub grants_applied: Vec<AppliedGrantDto>,
    #[serde(default)]
    pub warnings: Vec<BriefingWarningDto>,
    pub budget: BudgetReportDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub denied: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denial_reason: Option<String>,
}

impl PersonalContinuityBriefingPacket {
    pub fn empty_denied(
        briefing_id: String,
        scope_key: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        // F7/M2: seed kind=denied so every denied path has ≥1 structured warning.
        let reason = reason.into();
        Self {
            api_version: API_VERSION.to_string(),
            briefing_id,
            kind: "Personal".to_string(),
            scope_key: scope_key.into(),
            preferences: Vec::new(),
            continuity: ContinuitySummaryDto {
                summary: String::new(),
                thread_handles: Vec::new(),
            },
            open_review_items: Vec::new(),
            grants_applied: Vec::new(),
            warnings: vec![BriefingWarningDto {
                kind: "denied".into(),
                message: reason.clone(),
                subject_id: None,
                subject_kind: None,
            }],
            budget: BudgetReportDto {
                max_words: 0,
                used_words: 0,
                truncated_sections: Vec::new(),
                more_available: false,
            },
            generated_at: None,
            denied: true,
            denial_reason: Some(reason),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalBriefingResponse {
    pub api_version: String,
    pub packet: PersonalContinuityBriefingPacket,
}

impl PersonalBriefingResponse {
    pub fn new(packet: PersonalContinuityBriefingPacket) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            packet,
        }
    }
}

// ---------------------------------------------------------------------------
// Progressive query DTOs (T152 Phase E surface)
// ---------------------------------------------------------------------------

/// Ranking component breakdown for a progressive query hit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankingComponentsDto {
    /// Authority ordinal (higher = more authoritative).
    pub authority: i32,
    /// Valid-time preference score (higher = better fit for query time).
    pub valid_time: i32,
    /// Optional secondary relevance score (vector/FTS); never sole authority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevance: Option<f64>,
}

/// Compact progressive query result row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressiveQueryHitDto {
    pub id: String,
    /// Decision | Conclusion | Evidence | Memory
    pub kind: String,
    pub statement: String,
    pub state: String,
    #[serde(default)]
    pub evidence_handles: Vec<EvidenceHandle>,
    #[serde(default)]
    pub source_versions: Vec<String>,
    pub freshness: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_status: Option<String>,
    pub ranking: RankingComponentsDto,
}

/// Governed progressive query response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressiveQueryResponse {
    pub api_version: String,
    #[serde(default)]
    pub results: Vec<ProgressiveQueryHitDto>,
    pub applied_scope: String,
    pub applied_policy: String,
    pub query_trace_id: String,
    pub more_available: bool,
    #[serde(default)]
    pub freshness_summary: Option<FreshnessSummaryDto>,
    #[serde(default)]
    pub conflict_summary: Option<String>,
    #[serde(default)]
    pub denied: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denial_reason: Option<String>,
    /// Bootstrap remediation when `denied` (T221 F17). Omitted when not denied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denial_hint: Option<String>,
}

impl ProgressiveQueryResponse {
    pub fn new(
        results: Vec<ProgressiveQueryHitDto>,
        applied_scope: impl Into<String>,
        applied_policy: impl Into<String>,
        query_trace_id: impl Into<String>,
        more_available: bool,
    ) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            results,
            applied_scope: applied_scope.into(),
            applied_policy: applied_policy.into(),
            query_trace_id: query_trace_id.into(),
            more_available,
            freshness_summary: None,
            conflict_summary: None,
            denied: false,
            denial_reason: None,
            denial_hint: None,
        }
    }
}

/// Full retrieval trace by id (not dumped in the default query response body).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTraceDto {
    pub api_version: String,
    pub query_trace_id: String,
    pub scope: String,
    pub principal: String,
    pub query: String,
    pub applied_policy: String,
    #[serde(default)]
    pub ranking_json: serde_json::Value,
    #[serde(default)]
    pub result_handles: Vec<EvidenceHandle>,
    #[serde(default)]
    pub freshness_summary: Option<String>,
    #[serde(default)]
    pub conflict_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<DateTime<Utc>>,
}

/// Bounded preview when expanding an evidence/handle (no full raw dump by default).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlePreviewDto {
    pub api_version: String,
    pub handle_id: String,
    pub kind: String,
    /// Bounded text preview (truncated).
    pub preview: String,
    pub truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_version_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Governed briefing / query request DTOs (daemon protocol — T158)
// ---------------------------------------------------------------------------

fn default_api_version() -> String {
    API_VERSION.to_string()
}

/// Options for a project briefing over the daemon protocol.
///
/// **E1:** optional fields absent or null → handler defaults; never secrets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectBriefingRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// Scope identity key (e.g. `Repository:{uuid}`). Optional when cwd is provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_words: Option<usize>,
    /// Optional request-level override for governed briefing path (T152-R1-07).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governed_briefing: Option<bool>,
}

impl Default for ProjectBriefingRequest {
    fn default() -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            principal_id: None,
            scope: None,
            cwd: None,
            max_words: None,
            governed_briefing: None,
        }
    }
}

/// Options for a personal continuity briefing over the daemon protocol.
///
/// **E1:** empty personal packet uses empty arrays / empty continuity summary, never null lists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonalBriefingRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// Personal scope key `Personal:{user_id}` or bare user id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_words: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governed_briefing: Option<bool>,
}

impl Default for PersonalBriefingRequest {
    fn default() -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            principal_id: None,
            scope: None,
            max_words: None,
            governed_briefing: None,
        }
    }
}

/// Progressive knowledge query over the daemon protocol.
///
/// **E1 response shape** ([`ProgressiveQueryResponse`]): `results: []`, `more_available: false`;
/// policy deny → `denied: true` + optional `denial_hint` (bootstrap) or outer `Error(POLICY_DENIED)`,
/// never silent ok with empty.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueryKnowledgeRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Expand an evidence handle to a bounded preview (no full raw dump).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InspectEvidenceRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    /// Evidence / handle id.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// Max characters in preview body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<usize>,
}

// ---------------------------------------------------------------------------
// Evidence discovery list (T203)
// ---------------------------------------------------------------------------

/// Default max summary characters on evidence list items (F31).
pub const EVIDENCE_LIST_SUMMARY_MAX_CHARS: usize = 160;

/// One evidence row on a discovery list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceListItemDto {
    pub id: String,
    pub summary: String,
    pub status: String,
    pub source_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<DateTime<Utc>>,
}

/// Evidence discovery list response.
///
/// **E1:** `items: []` not null when empty.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceListResponse {
    pub api_version: String,
    #[serde(default)]
    pub items: Vec<EvidenceListItemDto>,
    #[serde(default)]
    pub more_available: bool,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl EvidenceListResponse {
    pub fn new(items: Vec<EvidenceListItemDto>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            items,
            more_available: false,
            warnings: Vec::new(),
        }
    }

    pub fn with_more(mut self, more_available: bool) -> Self {
        self.more_available = more_available;
        self
    }
}

/// List (and optional FTS search) evidence for a scope (daemon protocol — T203).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListEvidenceRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional FTS query over evidence summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Truncate evidence list summary to [`EVIDENCE_LIST_SUMMARY_MAX_CHARS`].
pub fn truncate_evidence_list_summary(summary: &str) -> String {
    let max = EVIDENCE_LIST_SUMMARY_MAX_CHARS;
    let count = summary.chars().count();
    if count <= max {
        return summary.to_string();
    }
    summary.chars().take(max).collect()
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn project_briefing_request__optional_fields_default() {
        let decoded: ProjectBriefingRequest =
            serde_json::from_str(r#"{"api_version":"1"}"#).expect("deserialize");
        assert!(decoded.principal_id.is_none());
        assert!(decoded.scope.is_none());
        assert!(decoded.max_words.is_none());
        assert!(decoded.governed_briefing.is_none());
    }

    #[test]
    fn evidence_list_response__empty_e1() {
        let resp = EvidenceListResponse::new(vec![]);
        let json = serde_json::to_string(&resp).expect("serialize");
        let v: serde_json::Value = serde_json::from_str(&json).expect("value");
        assert!(v["items"].as_array().expect("items").is_empty());
        assert_eq!(v["more_available"], false);
    }

    #[test]
    fn list_evidence_request__roundtrip() {
        let req = ListEvidenceRequest {
            api_version: API_VERSION.to_string(),
            principal_id: None,
            scope: Some("Repository:00000000-0000-0000-0000-0000000000a1".into()),
            query: Some("fts".into()),
            limit: Some(25),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: ListEvidenceRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, req);
    }

    #[test]
    fn truncate_evidence_list_summary__over_160__clips() {
        let long: String = "a".repeat(200);
        let out = truncate_evidence_list_summary(&long);
        assert_eq!(out.chars().count(), EVIDENCE_LIST_SUMMARY_MAX_CHARS);
    }

    #[test]
    fn query_knowledge_request__roundtrip() {
        let req = QueryKnowledgeRequest {
            api_version: API_VERSION.to_string(),
            query: "briefing budget".into(),
            scope: Some("Repository:00000000-0000-0000-0000-0000000000a1".into()),
            principal_id: Some("00000000-0000-0000-0000-0000000000p1".into()),
            limit: Some(10),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: QueryKnowledgeRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, req);
    }

    #[test]
    fn project_empty_denied__seeds_kind_denied_warning() {
        let packet = ProjectBriefingPacket::empty_denied(
            "b1".into(),
            BriefingScopeDto {
                scope_key: "Repository:x".into(),
                confidence: "High".into(),
                warnings: Vec::new(),
                alternatives: Vec::new(),
                authoritative: true,
            },
            "no grant",
        );
        assert!(packet.denied);
        assert_eq!(packet.denial_reason.as_deref(), Some("no grant"));
        assert_eq!(packet.warnings.len(), 1);
        assert_eq!(packet.warnings[0].kind, "denied");
        assert_eq!(packet.warnings[0].message, "no grant");
    }

    #[test]
    fn personal_empty_denied__seeds_kind_denied_warning() {
        let packet =
            PersonalContinuityBriefingPacket::empty_denied("b1".into(), "Personal:u", "denied");
        assert!(packet.denied);
        assert_eq!(packet.warnings.len(), 1);
        assert_eq!(packet.warnings[0].kind, "denied");
        assert_eq!(packet.warnings[0].message, "denied");
    }
}
