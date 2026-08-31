-- Migration 0029: Allow PINNED_IN_PROJECT on graph_edge (T335).
-- SQLite cannot ALTER CHECK constraints; rebuild graph_edge with the 0022
-- 12-label list plus PINNED_IN_PROJECT. Never edits 0013 or 0022. Forward-only.
-- Node kind allow-list unchanged (project already present).

PRAGMA foreign_keys = OFF;

CREATE TABLE graph_edge_new (
    src_id       INTEGER NOT NULL REFERENCES graph_node(node_id) ON DELETE CASCADE,
    label        TEXT NOT NULL CHECK (
        label IN (
            'IN_PROJECT',
            'IN_SESSION',
            'RECALLS',
            'SOURCE_FOR',
            'SYNTHESIZED_FROM',
            'CONFLICTS_WITH',
            'PART_OF_RECIPE',
            'OBSERVED_FROM',
            'DERIVED_FROM',
            'SUPPORTED_BY',
            'SUPERSEDES',
            'CONTAINS',
            'PINNED_IN_PROJECT'
        )
    ),
    dst_id       INTEGER NOT NULL REFERENCES graph_node(node_id) ON DELETE CASCADE,
    weight       REAL NOT NULL DEFAULT 1.0,
    created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (src_id, label, dst_id)
) WITHOUT ROWID;

INSERT INTO graph_edge_new (src_id, label, dst_id, weight, created_at)
SELECT src_id, label, dst_id, weight, created_at FROM graph_edge;

DROP TABLE graph_edge;
ALTER TABLE graph_edge_new RENAME TO graph_edge;

CREATE INDEX IF NOT EXISTS graph_edge_by_dst
    ON graph_edge(dst_id, label, src_id);

PRAGMA foreign_keys = ON;
