-- Migration 0006: Memory Projection
CREATE TABLE memory_projection (
    memory_id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    privacy TEXT NOT NULL,
    status TEXT NOT NULL, -- 'pinned' or 'forgotten'
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
