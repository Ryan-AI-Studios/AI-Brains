-- Migration 0025: Briefing cache + progressive query traces (T152 Phase B)
-- Cache invalidation via source_version_vector; traces for progressive retrieval.
-- Forward-only; never edit 0001–0024.

-- Cached typed briefing packets (Project / Personal / Preflight shell).
CREATE TABLE briefing_cache_projection (
    cache_key TEXT PRIMARY KEY,
    briefing_type TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    policy_version TEXT NOT NULL DEFAULT '',
    source_version_vector TEXT NOT NULL DEFAULT '',
    budget INTEGER NOT NULL DEFAULT 0,
    packet_json TEXT NOT NULL,
    generated_at TEXT NOT NULL,
    expires TEXT
);

CREATE INDEX idx_briefing_cache_scope
    ON briefing_cache_projection (scope_key);

CREATE INDEX idx_briefing_cache_type_scope
    ON briefing_cache_projection (briefing_type, scope_key);

-- Progressive query retrieval traces (full detail by id).
CREATE TABLE query_trace_projection (
    trace_id TEXT PRIMARY KEY,
    scope TEXT NOT NULL,
    principal TEXT NOT NULL,
    query TEXT NOT NULL,
    applied_policy TEXT NOT NULL DEFAULT '',
    ranking_json TEXT NOT NULL DEFAULT '{}',
    result_handles_json TEXT NOT NULL DEFAULT '[]',
    freshness_summary TEXT,
    conflict_summary TEXT,
    recorded_at TEXT NOT NULL
);

CREATE INDEX idx_query_trace_scope
    ON query_trace_projection (scope);

CREATE INDEX idx_query_trace_principal
    ON query_trace_projection (principal);

CREATE INDEX idx_query_trace_recorded
    ON query_trace_projection (recorded_at);

-- Optional retrieval feedback stub (write path may arrive later).
CREATE TABLE retrieval_feedback_projection (
    feedback_id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    rating INTEGER,
    note TEXT,
    recorded_at TEXT NOT NULL,
    FOREIGN KEY (trace_id) REFERENCES query_trace_projection (trace_id)
);

CREATE INDEX idx_retrieval_feedback_trace
    ON retrieval_feedback_projection (trace_id);
