-- Migration 0010: Conflict and Recipe Projections
CREATE TABLE conflict_projection (
    conflict_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    explanation TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(session_id) REFERENCES session_projection(session_id)
);

CREATE TABLE recipe_projection (
    recipe_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    steps_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_conflict_session ON conflict_projection(session_id);
