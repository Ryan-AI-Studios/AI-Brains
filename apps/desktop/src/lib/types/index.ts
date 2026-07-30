/**
 * Hand-synced wire DTOs for T161 / contracts (E1: arrays default to []).
 * No domain authority — presentation shapes only.
 */

export interface EvidenceHandle {
  evidence_id: string;
  cite_label?: string | null;
}

export interface BudgetReport {
  max_words: number;
  used_words: number;
  truncated_sections: string[];
  more_available: boolean;
}

export interface BriefingScope {
  scope_key: string;
  confidence: string;
  warnings: string[];
  alternatives: string[];
  authoritative: boolean;
}

export interface BriefingClaim {
  id: string;
  kind: string;
  statement: string;
  state: string;
  evidence_handles: EvidenceHandle[];
  title?: string | null;
}

export interface BriefingConstraint {
  id: string;
  statement: string;
  evidence_handles: EvidenceHandle[];
}

export interface BriefingWarning {
  kind: string;
  message: string;
  subject_id?: string | null;
  subject_kind?: string | null;
}

export interface FreshnessSummary {
  total_sources: number;
  fresh_count: number;
  stale_count: number;
  unavailable_count: number;
  worst_state: string;
}

export interface ProjectBriefingPacket {
  api_version: string;
  briefing_id: string;
  kind: string;
  scope: BriefingScope;
  handoff?: { summary: string; session_ids: string[] } | null;
  decisions: BriefingClaim[];
  conclusions: BriefingClaim[];
  constraints: BriefingConstraint[];
  warnings: BriefingWarning[];
  freshness: FreshnessSummary;
  ledgerful?: {
    hotspots: string[];
    impact_notes: string[];
    degraded: boolean;
  } | null;
  evidence_handles: EvidenceHandle[];
  budget: BudgetReport;
  generated_at?: string | null;
  denied: boolean;
  denial_reason?: string | null;
}

export interface ProjectBriefingResponse {
  api_version: string;
  packet: ProjectBriefingPacket;
}

export interface PersonalPreference {
  id: string;
  statement: string;
  evidence_handles: EvidenceHandle[];
}

export interface PersonalReviewItem {
  id: string;
  subject: string;
  criticality: string;
  status: string;
}

export interface AppliedGrant {
  grant_id: string;
  scope_key: string;
  capability: string;
  privacy: string;
}

export interface PersonalBriefingPacket {
  api_version: string;
  briefing_id: string;
  kind: string;
  scope_key: string;
  preferences: PersonalPreference[];
  continuity: { summary: string; thread_handles: string[] };
  open_review_items: PersonalReviewItem[];
  grants_applied: AppliedGrant[];
  warnings: BriefingWarning[];
  budget: BudgetReport;
  generated_at?: string | null;
  denied: boolean;
  denial_reason?: string | null;
}

export interface PersonalBriefingResponse {
  api_version: string;
  packet: PersonalBriefingPacket;
}

export interface RankingComponents {
  authority: number;
  valid_time: number;
  relevance?: number | null;
}

export interface ProgressiveQueryHit {
  id: string;
  kind: string;
  statement: string;
  state: string;
  evidence_handles: EvidenceHandle[];
  source_versions: string[];
  freshness: string;
  conflict_status?: string | null;
  ranking: RankingComponents;
}

export interface ProgressiveQueryResponse {
  api_version: string;
  results: ProgressiveQueryHit[];
  applied_scope: string;
  applied_policy: string;
  query_trace_id: string;
  more_available: boolean;
  freshness_summary?: FreshnessSummary | null;
  conflict_summary?: string | null;
  denied: boolean;
  denial_reason?: string | null;
}

export interface HandlePreview {
  api_version: string;
  handle_id: string;
  kind: string;
  preview: string;
  truncated: boolean;
  source_version_id?: string | null;
}

export interface SourceDto {
  id: string;
  kind: string;
  display_name: string;
  locator?: string | null;
  last_observed_at?: string | null;
}

export interface ReviewItem {
  id: string;
  subject: string;
  status: string;
  opened_at?: string | null;
}

export interface ReviewQueueResponse {
  api_version: string;
  items: ReviewItem[];
}

export interface ReviewResolvedResponse {
  api_version: string;
  id: string;
  status: string;
  warnings: string[];
}

export interface ScopeEvidence {
  signal: string;
  detail: string;
}

export interface ScopeResolvedResponse {
  api_version: string;
  scope: string;
  confidence: string;
  authoritative: boolean;
  evidence: ScopeEvidence[];
  warnings: string[];
  alternatives: string[];
}

export interface ErasureAcceptedResponse {
  api_version: string;
  request_id: string;
  status: string;
  warnings: string[];
}

export interface WipePurgedCounts {
  fts_rows: number;
  embeddings: number;
  projection_rows: number;
}

export interface ContentEnvelopeWipedResponse {
  api_version: string;
  status: string;
  content_key_id: string;
  tombstone_id?: string | null;
  wrap_destroyed: boolean;
  blobs_considered: number;
  purged: WipePurgedCounts;
  dependents_marked: number;
  warnings: string[];
  verify: { wrap_absent: boolean };
  validation: {
    fts_clear: boolean;
    store_open_refused: boolean;
    wal_checkpoint: string;
  };
}

export interface PingResponse {
  ok: boolean;
  service: string;
  version: string;
}

export interface DaemonConnectionInfo {
  loopback_base_url: string | null;
  token_file_present: boolean;
}

/** Normalize possibly-missing arrays from wire JSON (E1). */
export function asStringArray(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((x): x is string => typeof x === "string")
    : [];
}

export function asArray<T>(value: unknown): T[] {
  return Array.isArray(value) ? (value as T[]) : [];
}
