-- Migration 0011: Memory Hierarchy
-- Add level to memory_projection
ALTER TABLE memory_projection ADD COLUMN level INTEGER DEFAULT 0;

-- Table for tracking memory hierarchy (RAPTOR)
CREATE TABLE memory_hierarchy (
    parent_memory_id TEXT NOT NULL,
    child_memory_id TEXT NOT NULL,
    PRIMARY KEY (parent_memory_id, child_memory_id),
    FOREIGN KEY (parent_memory_id) REFERENCES memory_projection(memory_id),
    FOREIGN KEY (child_memory_id) REFERENCES memory_projection(memory_id)
);
