-- Migration 0007: FTS Setup
-- FTS5 virtual table for memory content
CREATE VIRTUAL TABLE memory_fts USING fts5(
    content,
    memory_id UNINDEXED,
    content='memory_projection',
    content_rowid='rowid'
);
