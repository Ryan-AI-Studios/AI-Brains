-- Migration 0005: Turn Projection
CREATE TABLE turn_projection (
    session_id TEXT NOT NULL,
    turn_index INTEGER NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    PRIMARY KEY(session_id, turn_index),
    FOREIGN KEY(session_id) REFERENCES session_projection(session_id)
);

CREATE INDEX idx_turn_session ON turn_projection(session_id);
