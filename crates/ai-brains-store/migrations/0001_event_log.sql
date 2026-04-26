-- Initial migration for the event log
CREATE TABLE events (
    event_id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    actor_json TEXT NOT NULL,
    causation_id TEXT,
    correlation_id TEXT,
    privacy TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_events_aggregate
ON events(aggregate_type, aggregate_id, occurred_at);

CREATE INDEX idx_events_type_time
ON events(event_type, occurred_at);

CREATE INDEX idx_events_correlation
ON events(correlation_id);

CREATE INDEX idx_events_privacy
ON events(privacy);

-- Strict append-only constraint
CREATE TRIGGER prevent_event_update
BEFORE UPDATE ON events
BEGIN
    SELECT RAISE(ABORT, 'events are immutable');
END;

-- Prevent deletion
CREATE TRIGGER prevent_event_delete
BEFORE DELETE ON events
BEGIN
    SELECT RAISE(ABORT, 'events are immutable');
END;
