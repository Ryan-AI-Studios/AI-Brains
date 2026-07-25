-- Migration 0024: Scopes, principals, grants, policy decision log (T151 Phase B)
-- Workspace/repository identity, principal bindings, active grant uniqueness.
-- Forward-only; never edit 0001–0023.

-- Workspace registry (privacy default CloudOk / "ok" for multi-repo shared boundary).
CREATE TABLE workspace_projection (
    workspace_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    privacy TEXT NOT NULL DEFAULT 'CloudOk',
    recorded_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_workspace_name ON workspace_projection (name);

-- Workspace ↔ repository membership.
CREATE TABLE workspace_repository_projection (
    workspace_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    PRIMARY KEY (workspace_id, project_id),
    FOREIGN KEY (workspace_id) REFERENCES workspace_projection (workspace_id)
);

CREATE INDEX idx_workspace_repository_project
    ON workspace_repository_projection (project_id);

-- Repository identity: durable key is project_id; remote is normalized URL hash only.
CREATE TABLE repository_identity_projection (
    project_id TEXT PRIMARY KEY,
    remote_url_hash TEXT,
    ledgerful_project_id TEXT,
    last_verified_at TEXT
);

-- Unique when a normalized remote hash is present (prevent silent dual identity).
CREATE UNIQUE INDEX idx_repository_identity_remote_hash
    ON repository_identity_projection (remote_url_hash)
    WHERE remote_url_hash IS NOT NULL AND remote_url_hash != '';

CREATE INDEX idx_repository_identity_ledgerful
    ON repository_identity_projection (ledgerful_project_id);

-- Path aliases (Windows + WSL normalized forms) — signal only, never sole identity.
CREATE TABLE repository_path_alias_projection (
    normalized_path TEXT PRIMARY KEY,
    project_id TEXT NOT NULL
);

CREATE INDEX idx_repository_path_alias_project
    ON repository_path_alias_projection (project_id);

-- Principal registry with typed kind string + binding JSON.
CREATE TABLE principal_projection (
    principal_id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    bound_source_kinds TEXT NOT NULL DEFAULT '[]',
    bound_capabilities TEXT NOT NULL DEFAULT '[]',
    recorded_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_principal_kind ON principal_projection (kind);

-- Scope grants: revoked_at NULL means active.
CREATE TABLE scope_grant_projection (
    grant_id TEXT PRIMARY KEY,
    principal_id TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    capability TEXT NOT NULL,
    -- Default LocalOnly when payload has no privacy (ScopeGrantIssued has none).
    privacy TEXT NOT NULL DEFAULT 'LocalOnly',
    issued_at TEXT NOT NULL,
    revoked_at TEXT
);

CREATE INDEX idx_scope_grant_principal ON scope_grant_projection (principal_id);
CREATE INDEX idx_scope_grant_scope ON scope_grant_projection (scope_key);
CREATE INDEX idx_scope_grant_principal_scope
    ON scope_grant_projection (principal_id, scope_key);

-- At most one active grant per (principal, scope, capability).
-- Partial unique index: SQLite supports WHERE on unique indexes.
CREATE UNIQUE INDEX idx_scope_grant_active_unique
    ON scope_grant_projection (principal_id, scope_key, capability)
    WHERE revoked_at IS NULL;

-- Policy audit log (projected from PolicyDecisionRecorded) — no content bodies / claim text.
CREATE TABLE policy_decision_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    principal_id TEXT NOT NULL,
    capability TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    allowed INTEGER NOT NULL,
    reason_code TEXT NOT NULL,
    privacy TEXT,
    recorded_at TEXT NOT NULL
);

CREATE INDEX idx_policy_decision_principal ON policy_decision_log (principal_id);
CREATE INDEX idx_policy_decision_recorded_at ON policy_decision_log (recorded_at);
