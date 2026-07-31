-- Migration 0027: Multi-device replication side stores + operational projections (T176 / P11.1)
-- Side stores (device_identity, device_id_tombstone, device_private_key_store,
-- peer_content_key_wrap, encrypted_envelope_index) hold durable crypto/replication
-- material and are NOT truncated on rebuild_projections.
-- Operational tables (replication_cursor, replication_gap_buffer,
-- erasure_ack_projection, replication_gap_skip_audit) are retained by default (v1).
-- Forward-only; never edit 0001–0026. No plaintext event bodies. No content_hash column.
-- Normative: ADR-0018, trackT176-sync-crate-schema/spec.md §5.

-- Local + peer public identity (public keys only for peers).
CREATE TABLE device_identity (
    device_id            TEXT PRIMARY KEY,
    schema_version       INTEGER NOT NULL DEFAULT 1,
    ed25519_public       BLOB NOT NULL,   -- 32 bytes
    x25519_public        BLOB NOT NULL,   -- 32 bytes
    display_name         TEXT,
    status               TEXT NOT NULL,   -- 'active' | 'revoked' | 'local'
    enrolled_at          TEXT NOT NULL,
    revoked_at           TEXT,
    enrolled_by_device_id TEXT NOT NULL,  -- signer of DeviceEnrolled; self for first local
    fingerprint_sha256   BLOB NOT NULL,   -- 32 bytes of enrollment_package hash
    CHECK (status IN ('active', 'revoked', 'local')),
    CHECK (length(ed25519_public) = 32),
    CHECK (length(x25519_public) = 32),
    CHECK (length(fingerprint_sha256) = 32)
);

-- Permanently retired DeviceIds (L3/L4). Insert on revoke; never delete.
CREATE TABLE device_id_tombstone (
    device_id     TEXT PRIMARY KEY,
    revoked_at    TEXT NOT NULL,
    reason_code   TEXT NOT NULL DEFAULT ''
);

-- Local device private key material (this vault's device only).
-- Inner AES-GCM under DataKey (§5.1.1); Windows: outer DPAPI on stored blob (R6).
CREATE TABLE device_private_key_store (
    device_id            TEXT PRIMARY KEY,
    wrap_schema_version  INTEGER NOT NULL DEFAULT 1,
    algorithm            TEXT NOT NULL DEFAULT 'AES-256-GCM',
    protection           TEXT NOT NULL DEFAULT 'datakey',
    -- 'datakey' | 'datakey_dpapi' (Windows dual-layer)
    wrap_nonce           BLOB NOT NULL,  -- 12-byte GCM nonce for DataKey layer
    wrap_ciphertext      BLOB NOT NULL,  -- ct‖tag of inner seeds; or DPAPI(ct‖tag) when protection=datakey_dpapi
    created_at           TEXT NOT NULL,
    CHECK (protection IN ('datakey', 'datakey_dpapi')),
    FOREIGN KEY (device_id) REFERENCES device_identity(device_id)
);

-- Per-recipient multi-device content DEK wraps.
-- PK upsert: latest verified wrap wins (R5); sender_device_id is audit only.
CREATE TABLE peer_content_key_wrap (
    content_key_id       TEXT NOT NULL,
    recipient_device_id  TEXT NOT NULL,
    sender_device_id     TEXT NOT NULL,
    schema_version       INTEGER NOT NULL DEFAULT 1,
    eph_x25519_public    BLOB NOT NULL,  -- 32 bytes
    wrap_nonce           BLOB NOT NULL,  -- 12 bytes
    wrap_ciphertext      BLOB NOT NULL,  -- ct‖tag of content DEK
    created_at           TEXT NOT NULL,
    PRIMARY KEY (content_key_id, recipient_device_id),
    CHECK (length(eph_x25519_public) = 32),
    CHECK (length(wrap_nonce) = 12)
);

-- Opaque envelope index for replication (no plaintext bodies).
-- Integrity of body: outer Ed25519 over signed_bytes only (R29 — no content_hash column).
CREATE TABLE encrypted_envelope_index (
    envelope_id          TEXT PRIMARY KEY,
    event_id             TEXT NOT NULL UNIQUE,
    sender_device_id     TEXT NOT NULL,
    local_seq            INTEGER NOT NULL,
    content_type_code    INTEGER NOT NULL,
    content_key_id       TEXT,            -- zero UUID for most control
    body_len             INTEGER NOT NULL,
    padding_bucket       INTEGER,         -- 256 | 4096 | 65536 when applied
    applied_at           TEXT,
    UNIQUE (sender_device_id, local_seq)
);

CREATE INDEX idx_envelope_sender_seq
    ON encrypted_envelope_index (sender_device_id, local_seq);

-- Per peer stream cursor + gap state (L13).
CREATE TABLE replication_cursor (
    peer_device_id       TEXT PRIMARY KEY,
    high_water_seq       INTEGER NOT NULL DEFAULT 0,
    expected_local_seq   INTEGER NOT NULL DEFAULT 1,
    state                TEXT NOT NULL DEFAULT 'in_sync',
    -- 'in_sync' | 'sync_gap' | 'blocked'
    updated_at           TEXT NOT NULL,
    CHECK (state IN ('in_sync', 'sync_gap', 'blocked'))
);

-- Gap buffer: seq/envelope_id metadata only (bodies NOT stored here).
CREATE TABLE replication_gap_buffer (
    peer_device_id       TEXT NOT NULL,
    local_seq            INTEGER NOT NULL,
    envelope_id          TEXT NOT NULL,
    buffered_at          TEXT NOT NULL,
    PRIMARY KEY (peer_device_id, local_seq)
);

-- Erasure ACK projection (L7 / deferred #34.1 implement).
CREATE TABLE erasure_ack_projection (
    erasure_id           TEXT NOT NULL,
    peer_device_id        TEXT NOT NULL,
    content_key_id       TEXT NOT NULL,
    status               TEXT NOT NULL,
    -- 'pending' | 'acked' | 'failed' | 'unreachable'
    sync_cycles_waiting  INTEGER NOT NULL DEFAULT 0,
    updated_at           TEXT NOT NULL,
    PRIMARY KEY (erasure_id, peer_device_id),
    CHECK (status IN ('pending', 'acked', 'failed', 'unreachable'))
);

CREATE INDEX idx_erasure_ack_status
    ON erasure_ack_projection (status);

-- Operator gap-skip audit index (R13): authoritative signed envelope lives in the event log.
CREATE TABLE replication_gap_skip_audit (
    audit_id             TEXT PRIMARY KEY,
    peer_device_id        TEXT NOT NULL,
    skipped_seq          INTEGER NOT NULL,
    signed_event_id      TEXT NOT NULL,
    created_at           TEXT NOT NULL
);
