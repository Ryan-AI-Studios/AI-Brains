-- Migration 0026: Content envelope side stores + erasure/tombstone projections (T163 / P8.1)
-- Side stores (content_key_store, encrypted_content_blob) hold wrapped DEKs + ciphertext
-- and are NOT truncated on rebuild_projections. Event projections (erasure_request_projection,
-- tombstone_projection) ARE truncated and re-applied from the append-only event log.
-- Forward-only; never edit 0001–0025. No plaintext content or DEK columns.
-- Normative: ADR-0016, trackT163-content-envelope-schema/spec.md §5.

-- Durable DEK wrap store (CE destroy target). Not event-sourced.
CREATE TABLE content_key_store (
    content_key_id TEXT PRIMARY KEY,
    wrap_schema_version INTEGER NOT NULL DEFAULT 1,
    algorithm TEXT NOT NULL DEFAULT 'AES-256-GCM',
    wrap_nonce BLOB,
    wrap_ciphertext BLOB,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL,
    destroyed_at TEXT,
    CHECK (
        status IN ('active', 'destroyed')
        AND (
            (status = 'active'
                AND wrap_nonce IS NOT NULL
                AND wrap_ciphertext IS NOT NULL)
            OR
            (status = 'destroyed'
                AND wrap_nonce IS NULL
                AND wrap_ciphertext IS NULL)
        )
    )
);

CREATE INDEX idx_content_key_store_status
    ON content_key_store (status);

-- Durable ciphertext blobs (opaque). Not event-sourced.
CREATE TABLE encrypted_content_blob (
    blob_id TEXT PRIMARY KEY,
    content_key_id TEXT NOT NULL,
    envelope_schema_version INTEGER NOT NULL DEFAULT 1,
    algorithm TEXT NOT NULL DEFAULT 'AES-256-GCM',
    nonce BLOB NOT NULL,
    ciphertext BLOB NOT NULL,
    content_class TEXT,
    subject_kind TEXT,
    subject_id TEXT,
    size_bytes INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_encrypted_content_blob_content_key
    ON encrypted_content_blob (content_key_id);

CREATE INDEX idx_encrypted_content_blob_subject
    ON encrypted_content_blob (subject_kind, subject_id);

-- Event projection: ContentErasureRequested / ContentErased request lifecycle.
CREATE TABLE erasure_request_projection (
    content_key_id TEXT PRIMARY KEY,
    requester TEXT NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL,
    requested_at TEXT NOT NULL,
    completed_at TEXT,
    tombstone_id TEXT
);

CREATE INDEX idx_erasure_request_status
    ON erasure_request_projection (status);

-- Event projection: ContentErased tombstones (minimal safe fields only).
CREATE TABLE tombstone_projection (
    tombstone_id TEXT PRIMARY KEY,
    content_key_id TEXT NOT NULL UNIQUE,
    erased_at TEXT NOT NULL,
    reason_code TEXT NOT NULL DEFAULT ''
);
