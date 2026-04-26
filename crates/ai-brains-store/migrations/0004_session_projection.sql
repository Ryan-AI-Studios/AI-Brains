-- Migration 0004: Session Projection
CREATE TABLE session_projection (
    session_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    status TEXT NOT NULL,
    privacy TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(project_id) REFERENCES project_projection(project_id)
);

CREATE INDEX idx_session_project ON session_projection(project_id);
CREATE INDEX idx_session_status ON session_projection(status);
