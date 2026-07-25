-- Migration 0022: Expand relational graph CHECKs for governed provenance (T149 Phase G).
-- SQLite cannot ALTER CHECK constraints; rebuild tables with expanded allow-lists.
-- Never edits 0001–0019. Forward-only.

PRAGMA foreign_keys = OFF;

CREATE TABLE graph_node_new (
    node_id      INTEGER PRIMARY KEY,
    kind         TEXT NOT NULL CHECK (
        kind IN (
            'project',
            'session',
            'turn',
            'memory',
            'conflict',
            'recipe',
            'source',
            'source_version',
            'evidence',
            'conclusion',
            'decision',
            'workspace'
        )
    ),
    external_id  TEXT NOT NULL UNIQUE,
    created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO graph_node_new (node_id, kind, external_id, created_at)
SELECT node_id, kind, external_id, created_at FROM graph_node;

DROP TABLE graph_node;
ALTER TABLE graph_node_new RENAME TO graph_node;

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
            'CONTAINS'
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
