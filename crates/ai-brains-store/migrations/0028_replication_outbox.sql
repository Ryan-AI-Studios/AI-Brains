-- Migration 0028: Durable replication outbox for pending push (T177 M2).
-- Survives process restarts so CLI `replicate push` can put previously sealed
-- envelopes. Side store — not truncated on rebuild_projections.
-- Forward-only; never edit 0001–0027.

CREATE TABLE IF NOT EXISTS replication_outbox (
    envelope_id          TEXT PRIMARY KEY,
    event_id             TEXT NOT NULL UNIQUE,
    sender_device_id     TEXT NOT NULL,
    local_seq            INTEGER NOT NULL,
    content_type_code    INTEGER NOT NULL,
    wire_body            BLOB NOT NULL,
    created_at           TEXT NOT NULL,
    pushed_at            TEXT
);

CREATE INDEX IF NOT EXISTS idx_replication_outbox_sender_seq
    ON replication_outbox (sender_device_id, local_seq);

CREATE INDEX IF NOT EXISTS idx_replication_outbox_unpushed
    ON replication_outbox (sender_device_id, local_seq)
    WHERE pushed_at IS NULL;
