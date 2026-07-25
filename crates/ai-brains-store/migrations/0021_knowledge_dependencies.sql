-- Migration 0021: Knowledge dependency edges and invalidation queue (T149)
-- Forward-only; never edit 0001–0019.

CREATE TABLE knowledge_dependency_projection (
    dependency_id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_type TEXT NOT NULL,
    parent_id TEXT NOT NULL,
    evidence_id TEXT,
    source_version_id TEXT,
    recorded_at TEXT NOT NULL,
    CHECK (evidence_id IS NOT NULL OR source_version_id IS NOT NULL)
);

CREATE INDEX idx_kd_parent ON knowledge_dependency_projection (parent_type, parent_id);
CREATE INDEX idx_kd_evidence ON knowledge_dependency_projection (evidence_id);
CREATE INDEX idx_kd_source_version ON knowledge_dependency_projection (source_version_id);

CREATE TABLE invalidation_queue_projection (
    queue_id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_type TEXT NOT NULL,
    parent_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    source_version_id TEXT,
    status TEXT NOT NULL DEFAULT 'Pending',
    enqueued_at TEXT NOT NULL
);

CREATE INDEX idx_iq_status ON invalidation_queue_projection (status);
CREATE INDEX idx_iq_parent ON invalidation_queue_projection (parent_type, parent_id);
CREATE INDEX idx_iq_source_version ON invalidation_queue_projection (source_version_id);
CREATE INDEX idx_iq_enqueued_at ON invalidation_queue_projection (enqueued_at);
