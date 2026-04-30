-- Migration 0014: Memory Session Context
-- Add session_id to memory_projection to enable thread reconstruction and session-aware recall.
-- This column is nullable to maintain compatibility with RAPTOR synthesized memories that may span multiple sessions.

ALTER TABLE memory_projection ADD COLUMN session_id TEXT;

-- Create an index for performance on session-filtered recall queries
CREATE INDEX idx_memory_session ON memory_projection(session_id);
