-- Migration 0002: Identity Projection
-- Tracks users and devices if necessary, but for now we'll keep it simple
-- based on what we might need.

CREATE TABLE identity_projection (
    user_id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
