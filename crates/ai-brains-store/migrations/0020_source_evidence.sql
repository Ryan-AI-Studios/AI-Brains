-- Migration 0020: Source registry, versions, and evidence projections (T149)
-- Forward-only; never edit 0001–0019.

CREATE TABLE source_projection (
    source_id TEXT PRIMARY KEY,
    scope TEXT NOT NULL DEFAULT '',
    kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    locator TEXT,
    status TEXT NOT NULL DEFAULT 'Active',
    last_observed_at TEXT,
    recorded_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Stable identity within a scope: kind + locator (or display_name when locator absent).
CREATE UNIQUE INDEX idx_source_stable_identity
    ON source_projection (scope, kind, COALESCE(locator, display_name));

CREATE INDEX idx_source_scope ON source_projection (scope);
CREATE INDEX idx_source_status ON source_projection (status);
CREATE INDEX idx_source_recorded_at ON source_projection (recorded_at);

CREATE TABLE source_alias_projection (
    alias TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    FOREIGN KEY (source_id) REFERENCES source_projection (source_id)
);

CREATE TABLE source_version_projection (
    version_id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    normalizer_version INTEGER NOT NULL DEFAULT 1,
    recorded_at TEXT NOT NULL,
    FOREIGN KEY (source_id) REFERENCES source_projection (source_id),
    UNIQUE (source_id, fingerprint)
);

CREATE INDEX idx_source_version_source_id ON source_version_projection (source_id);
CREATE INDEX idx_source_version_fingerprint ON source_version_projection (fingerprint);
CREATE INDEX idx_source_version_recorded_at ON source_version_projection (recorded_at);

CREATE TABLE evidence_projection (
    evidence_id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    source_version_id TEXT,
    status TEXT NOT NULL DEFAULT 'Active',
    summary TEXT NOT NULL,
    privacy TEXT NOT NULL,
    model_provenance_json TEXT,
    fingerprint TEXT,
    recorded_at TEXT NOT NULL,
    FOREIGN KEY (source_id) REFERENCES source_projection (source_id),
    FOREIGN KEY (source_version_id) REFERENCES source_version_projection (version_id)
);

CREATE INDEX idx_evidence_source_id ON evidence_projection (source_id);
CREATE INDEX idx_evidence_source_version_id ON evidence_projection (source_version_id);
CREATE INDEX idx_evidence_status ON evidence_projection (status);
CREATE INDEX idx_evidence_recorded_at ON evidence_projection (recorded_at);

-- FTS5 over evidence summary (mirrors memory_fts / 0007–0008 pattern).
CREATE VIRTUAL TABLE evidence_fts USING fts5(
    summary,
    evidence_id UNINDEXED,
    content = 'evidence_projection',
    content_rowid = 'rowid'
);

CREATE TRIGGER evidence_fts_ai AFTER INSERT ON evidence_projection BEGIN
    INSERT INTO evidence_fts (rowid, summary, evidence_id)
    VALUES (new.rowid, new.summary, new.evidence_id);
END;

CREATE TRIGGER evidence_fts_ad AFTER DELETE ON evidence_projection BEGIN
    INSERT INTO evidence_fts (evidence_fts, rowid, summary, evidence_id)
    VALUES ('delete', old.rowid, old.summary, old.evidence_id);
END;

CREATE TRIGGER evidence_fts_au AFTER UPDATE ON evidence_projection BEGIN
    INSERT INTO evidence_fts (evidence_fts, rowid, summary, evidence_id)
    VALUES ('delete', old.rowid, old.summary, old.evidence_id);
    INSERT INTO evidence_fts (rowid, summary, evidence_id)
    VALUES (new.rowid, new.summary, new.evidence_id);
END;
