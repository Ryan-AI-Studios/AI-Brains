-- Migration 0012: Retention and Forget Support
ALTER TABLE turn_projection ADD COLUMN last_accessed_at TEXT;
UPDATE turn_projection SET last_accessed_at = occurred_at;
