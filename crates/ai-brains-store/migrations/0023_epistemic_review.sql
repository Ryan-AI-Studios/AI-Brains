-- Migration 0023: Epistemic lifecycle projections (T150)
-- Conclusions, decisions, review items, claim conflicts + valid time columns.
-- Forward-only; never edit 0001–0022.

CREATE TABLE conclusion_projection (
    conclusion_id TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    statement TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT '',
    privacy TEXT NOT NULL DEFAULT '',
    proposer TEXT NOT NULL,
    valid_from TEXT NOT NULL,
    valid_until TEXT,
    recorded_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    supersedes TEXT,
    superseded_by TEXT,
    protected_category TEXT,
    unsupported INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_conclusion_state ON conclusion_projection (state);
CREATE INDEX idx_conclusion_scope ON conclusion_projection (scope);
CREATE INDEX idx_conclusion_valid_from ON conclusion_projection (valid_from);

CREATE TABLE conclusion_evidence_projection (
    conclusion_id TEXT NOT NULL,
    evidence_id TEXT NOT NULL,
    PRIMARY KEY (conclusion_id, evidence_id),
    FOREIGN KEY (conclusion_id) REFERENCES conclusion_projection (conclusion_id)
);

CREATE INDEX idx_conclusion_evidence_evidence
    ON conclusion_evidence_projection (evidence_id);

CREATE TABLE decision_projection (
    decision_id TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    title TEXT NOT NULL,
    statement TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT '',
    proposer TEXT NOT NULL,
    approver TEXT,
    proposal_event_id TEXT,
    valid_from TEXT,
    valid_until TEXT,
    recorded_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    superseded_by TEXT
);

CREATE INDEX idx_decision_state ON decision_projection (state);
CREATE INDEX idx_decision_scope ON decision_projection (scope);

CREATE TABLE decision_support_projection (
    decision_id TEXT NOT NULL,
    -- Empty string when not applicable (SQLite composite PK cannot use NULL).
    conclusion_id TEXT NOT NULL DEFAULT '',
    evidence_id TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (decision_id, conclusion_id, evidence_id),
    FOREIGN KEY (decision_id) REFERENCES decision_projection (decision_id)
);

CREATE INDEX idx_decision_support_conclusion
    ON decision_support_projection (conclusion_id);

CREATE TABLE review_item_projection (
    review_item_id TEXT PRIMARY KEY,
    subject_kind TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    criticality TEXT NOT NULL,
    status TEXT NOT NULL,
    opened_by TEXT NOT NULL,
    subject TEXT NOT NULL DEFAULT '',
    resolution TEXT,
    resolved_by TEXT,
    related_conclusion_id TEXT,
    related_decision_id TEXT,
    related_source_id TEXT,
    recorded_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_review_item_status ON review_item_projection (status);
CREATE INDEX idx_review_item_subject ON review_item_projection (subject_kind, subject_id);

CREATE TABLE claim_conflict_projection (
    conflict_id TEXT PRIMARY KEY,
    claim_a_kind TEXT NOT NULL,
    claim_a_id TEXT NOT NULL,
    claim_b_kind TEXT NOT NULL,
    claim_b_id TEXT NOT NULL,
    status TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT '',
    valid_from TEXT,
    valid_until TEXT,
    explanation TEXT NOT NULL,
    resolution TEXT,
    recorded_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_claim_conflict_status ON claim_conflict_projection (status);
CREATE INDEX idx_claim_conflict_scope ON claim_conflict_projection (scope);
