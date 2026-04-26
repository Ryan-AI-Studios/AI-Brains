-- Migration 0003: Project Projection
CREATE TABLE project_projection (
    project_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE project_alias_projection (
    alias TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    FOREIGN KEY(project_id) REFERENCES project_projection(project_id)
);
