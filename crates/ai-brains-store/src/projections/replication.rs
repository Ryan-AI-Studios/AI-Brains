//! Multi-device replication side stores + operational projections (T176 / ADR-0018).
//!
//! Side stores: device_identity, device_id_tombstone, device_private_key_store,
//! peer_content_key_wrap, encrypted_envelope_index, signed_replication_control —
//! retained on rebuild.
//! Operational: replication_cursor, gap buffer, erasure_ack, gap_skip_audit — v1 retain.
//!
//! No plaintext event bodies. No crypto seal/open here (opaque wrap bytes).

use crate::errors::{Result, StoreError};
use rusqlite::{Connection, OptionalExtension, params};
use std::fmt;

/// Default ACK timeout in sync cycles (R14). Unit-testable; production T176 does not tick.
pub const ACK_TIMEOUT_SYNC_CYCLES: u32 = 3;

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceIdentityRow {
    pub device_id: String,
    pub schema_version: i64,
    pub ed25519_public: Vec<u8>,
    pub x25519_public: Vec<u8>,
    pub display_name: Option<String>,
    pub status: String,
    pub enrolled_at: String,
    pub revoked_at: Option<String>,
    pub enrolled_by_device_id: String,
    pub fingerprint_sha256: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DevicePrivateKeyRow {
    pub device_id: String,
    pub wrap_schema_version: i64,
    pub algorithm: String,
    pub protection: String,
    pub wrap_nonce: Vec<u8>,
    pub wrap_ciphertext: Vec<u8>,
    pub created_at: String,
}

impl fmt::Debug for DevicePrivateKeyRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DevicePrivateKeyRow")
            .field("device_id", &self.device_id)
            .field("wrap_schema_version", &self.wrap_schema_version)
            .field("algorithm", &self.algorithm)
            .field("protection", &self.protection)
            .field(
                "wrap_nonce",
                &format!("<redacted len={}>", self.wrap_nonce.len()),
            )
            .field(
                "wrap_ciphertext",
                &format!("<redacted len={}>", self.wrap_ciphertext.len()),
            )
            .field("created_at", &self.created_at)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct PeerContentKeyWrapRow {
    pub content_key_id: String,
    pub recipient_device_id: String,
    pub sender_device_id: String,
    pub schema_version: i64,
    pub eph_x25519_public: Vec<u8>,
    pub wrap_nonce: Vec<u8>,
    pub wrap_ciphertext: Vec<u8>,
    pub created_at: String,
}

impl fmt::Debug for PeerContentKeyWrapRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeerContentKeyWrapRow")
            .field("content_key_id", &self.content_key_id)
            .field("recipient_device_id", &self.recipient_device_id)
            .field("sender_device_id", &self.sender_device_id)
            .field("schema_version", &self.schema_version)
            .field(
                "eph_x25519_public",
                &format!("<redacted len={}>", self.eph_x25519_public.len()),
            )
            .field(
                "wrap_nonce",
                &format!("<redacted len={}>", self.wrap_nonce.len()),
            )
            .field(
                "wrap_ciphertext",
                &format!("<redacted len={}>", self.wrap_ciphertext.len()),
            )
            .field("created_at", &self.created_at)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeIndexRow {
    pub envelope_id: String,
    pub event_id: String,
    pub sender_device_id: String,
    pub local_seq: i64,
    pub content_type_code: i64,
    pub content_key_id: Option<String>,
    pub body_len: i64,
    pub padding_bucket: Option<i64>,
    pub applied_at: Option<String>,
}

/// Local signed control envelope (cleartext body + Ed25519 signature).
#[derive(Clone, PartialEq, Eq)]
pub struct SignedControlRow {
    pub event_id: String,
    pub envelope_id: String,
    pub sender_device_id: String,
    pub content_type_code: i64,
    pub body: Vec<u8>,
    pub signature: Vec<u8>,
    pub schema_version: i64,
    pub local_seq: i64,
    pub created_at: String,
}

impl fmt::Debug for SignedControlRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignedControlRow")
            .field("event_id", &self.event_id)
            .field("envelope_id", &self.envelope_id)
            .field("sender_device_id", &self.sender_device_id)
            .field("content_type_code", &self.content_type_code)
            .field("body_len", &self.body.len())
            .field("signature_len", &self.signature.len())
            .field("schema_version", &self.schema_version)
            .field("local_seq", &self.local_seq)
            .field("created_at", &self.created_at)
            .finish()
    }
}

/// Inputs for atomic first-device bootstrap (identity + private key + signed control).
#[derive(Debug, Clone)]
pub struct BootstrapLocalDeviceInput {
    pub identity: DeviceIdentityRow,
    pub private_key: DevicePrivateKeyRow,
    pub signed_control: SignedControlRow,
    pub envelope_index: EnvelopeIndexRow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplicationCursorRow {
    pub peer_device_id: String,
    pub high_water_seq: i64,
    pub expected_local_seq: i64,
    pub state: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasureAckRow {
    pub erasure_id: String,
    pub peer_device_id: String,
    pub content_key_id: String,
    pub status: String,
    pub sync_cycles_waiting: i64,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Device identity
// ---------------------------------------------------------------------------

/// Insert a device identity. Rejects if `device_id` is tombstoned (R16).
pub fn insert_device_identity(conn: &Connection, row: &DeviceIdentityRow) -> Result<()> {
    if row.ed25519_public.len() != 32 || row.x25519_public.len() != 32 {
        return Err(StoreError::ConfigError(
            "ed25519_public and x25519_public must be 32 bytes".to_string(),
        ));
    }
    if row.fingerprint_sha256.len() != 32 {
        return Err(StoreError::ConfigError(
            "fingerprint_sha256 must be 32 bytes".to_string(),
        ));
    }
    if row.enrolled_by_device_id.is_empty() {
        return Err(StoreError::ConfigError(
            "enrolled_by_device_id must not be empty".to_string(),
        ));
    }
    let tombstoned: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM device_id_tombstone WHERE device_id = ?)",
        params![row.device_id],
        |r| r.get(0),
    )?;
    if tombstoned {
        return Err(StoreError::ConfigError(format!(
            "device_id is permanently tombstoned: {}",
            row.device_id
        )));
    }
    conn.execute(
        "INSERT INTO device_identity (
            device_id, schema_version, ed25519_public, x25519_public,
            display_name, status, enrolled_at, revoked_at,
            enrolled_by_device_id, fingerprint_sha256
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            row.device_id,
            row.schema_version,
            row.ed25519_public,
            row.x25519_public,
            row.display_name,
            row.status,
            row.enrolled_at,
            row.revoked_at,
            row.enrolled_by_device_id,
            row.fingerprint_sha256,
        ],
    )?;
    Ok(())
}

/// List devices with status `active` or `local` (enrolled-set for L8).
pub fn list_enrolled_devices(conn: &Connection) -> Result<Vec<DeviceIdentityRow>> {
    let mut stmt = conn.prepare(
        "SELECT device_id, schema_version, ed25519_public, x25519_public,
                display_name, status, enrolled_at, revoked_at,
                enrolled_by_device_id, fingerprint_sha256
         FROM device_identity
         WHERE status IN ('active', 'local')
         ORDER BY enrolled_at ASC, device_id ASC",
    )?;
    let rows = stmt.query_map([], map_device_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// True if any device with status active or local exists (R27 bootstrap guard).
pub fn has_active_or_local_device(conn: &Connection) -> Result<bool> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM device_identity WHERE status IN ('active', 'local')
         )",
        [],
        |r| r.get(0),
    )?;
    Ok(exists)
}

pub fn get_device(conn: &Connection, device_id: &str) -> Result<Option<DeviceIdentityRow>> {
    let mut stmt = conn.prepare(
        "SELECT device_id, schema_version, ed25519_public, x25519_public,
                display_name, status, enrolled_at, revoked_at,
                enrolled_by_device_id, fingerprint_sha256
         FROM device_identity WHERE device_id = ?",
    )?;
    let row = stmt
        .query_row(params![device_id], map_device_row)
        .optional()?;
    Ok(row)
}

fn map_device_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeviceIdentityRow> {
    Ok(DeviceIdentityRow {
        device_id: row.get(0)?,
        schema_version: row.get(1)?,
        ed25519_public: row.get(2)?,
        x25519_public: row.get(3)?,
        display_name: row.get(4)?,
        status: row.get(5)?,
        enrolled_at: row.get(6)?,
        revoked_at: row.get(7)?,
        enrolled_by_device_id: row.get(8)?,
        fingerprint_sha256: row.get(9)?,
    })
}

/// Mark device revoked, insert tombstone, delete peer wraps for recipient (R23).
pub fn tombstone_device(
    conn: &Connection,
    device_id: &str,
    revoked_at: &str,
    reason_code: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE device_identity
         SET status = 'revoked', revoked_at = ?
         WHERE device_id = ?",
        params![revoked_at, device_id],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO device_id_tombstone (device_id, revoked_at, reason_code)
         VALUES (?, ?, ?)",
        params![device_id, revoked_at, reason_code],
    )?;
    delete_peer_wraps_for_recipient(conn, device_id)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Device private key store
// ---------------------------------------------------------------------------

pub fn put_device_private_key_wrap(conn: &Connection, row: &DevicePrivateKeyRow) -> Result<()> {
    if row.wrap_nonce.is_empty() || row.wrap_ciphertext.is_empty() {
        return Err(StoreError::ConfigError(
            "wrap_nonce and wrap_ciphertext must be non-empty".to_string(),
        ));
    }
    conn.execute(
        "INSERT OR REPLACE INTO device_private_key_store (
            device_id, wrap_schema_version, algorithm, protection,
            wrap_nonce, wrap_ciphertext, created_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![
            row.device_id,
            row.wrap_schema_version,
            row.algorithm,
            row.protection,
            row.wrap_nonce,
            row.wrap_ciphertext,
            row.created_at,
        ],
    )?;
    Ok(())
}

pub fn get_device_private_key_wrap(
    conn: &Connection,
    device_id: &str,
) -> Result<Option<DevicePrivateKeyRow>> {
    let mut stmt = conn.prepare(
        "SELECT device_id, wrap_schema_version, algorithm, protection,
                wrap_nonce, wrap_ciphertext, created_at
         FROM device_private_key_store WHERE device_id = ?",
    )?;
    let row = stmt
        .query_row(params![device_id], |row| {
            Ok(DevicePrivateKeyRow {
                device_id: row.get(0)?,
                wrap_schema_version: row.get(1)?,
                algorithm: row.get(2)?,
                protection: row.get(3)?,
                wrap_nonce: row.get(4)?,
                wrap_ciphertext: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .optional()?;
    Ok(row)
}

// ---------------------------------------------------------------------------
// Peer content key wraps (R5 / R23)
// ---------------------------------------------------------------------------

/// Upsert peer wrap; same (content_key_id, recipient) replaces prior row.
pub fn upsert_peer_content_key_wrap(conn: &Connection, row: &PeerContentKeyWrapRow) -> Result<()> {
    if row.eph_x25519_public.len() != 32 {
        return Err(StoreError::ConfigError(
            "eph_x25519_public must be 32 bytes".to_string(),
        ));
    }
    if row.wrap_nonce.len() != 12 {
        return Err(StoreError::ConfigError(
            "wrap_nonce must be 12 bytes".to_string(),
        ));
    }
    conn.execute(
        "INSERT INTO peer_content_key_wrap (
            content_key_id, recipient_device_id, sender_device_id, schema_version,
            eph_x25519_public, wrap_nonce, wrap_ciphertext, created_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(content_key_id, recipient_device_id) DO UPDATE SET
            sender_device_id = excluded.sender_device_id,
            schema_version = excluded.schema_version,
            eph_x25519_public = excluded.eph_x25519_public,
            wrap_nonce = excluded.wrap_nonce,
            wrap_ciphertext = excluded.wrap_ciphertext,
            created_at = excluded.created_at",
        params![
            row.content_key_id,
            row.recipient_device_id,
            row.sender_device_id,
            row.schema_version,
            row.eph_x25519_public,
            row.wrap_nonce,
            row.wrap_ciphertext,
            row.created_at,
        ],
    )?;
    Ok(())
}

pub fn get_peer_wrap(
    conn: &Connection,
    content_key_id: &str,
    recipient_device_id: &str,
) -> Result<Option<PeerContentKeyWrapRow>> {
    let mut stmt = conn.prepare(
        "SELECT content_key_id, recipient_device_id, sender_device_id, schema_version,
                eph_x25519_public, wrap_nonce, wrap_ciphertext, created_at
         FROM peer_content_key_wrap
         WHERE content_key_id = ? AND recipient_device_id = ?",
    )?;
    let row = stmt
        .query_row(params![content_key_id, recipient_device_id], |row| {
            Ok(PeerContentKeyWrapRow {
                content_key_id: row.get(0)?,
                recipient_device_id: row.get(1)?,
                sender_device_id: row.get(2)?,
                schema_version: row.get(3)?,
                eph_x25519_public: row.get(4)?,
                wrap_nonce: row.get(5)?,
                wrap_ciphertext: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .optional()?;
    Ok(row)
}

/// Delete all peer wraps for a content key (CE destroy hook).
pub fn delete_peer_wraps_for_key(conn: &Connection, content_key_id: &str) -> Result<usize> {
    let n = conn.execute(
        "DELETE FROM peer_content_key_wrap WHERE content_key_id = ?",
        params![content_key_id],
    )?;
    Ok(n)
}

/// Delete all peer wraps for a recipient device (R23 revoke cleanup).
pub fn delete_peer_wraps_for_recipient(
    conn: &Connection,
    recipient_device_id: &str,
) -> Result<usize> {
    let n = conn.execute(
        "DELETE FROM peer_content_key_wrap WHERE recipient_device_id = ?",
        params![recipient_device_id],
    )?;
    Ok(n)
}

// ---------------------------------------------------------------------------
// Envelope index (idempotent on event_id)
// ---------------------------------------------------------------------------

/// Insert envelope index row. Same `event_id` is idempotent (no-op success).
pub fn insert_envelope_index(conn: &Connection, row: &EnvelopeIndexRow) -> Result<()> {
    if envelope_exists(conn, &row.event_id)? {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO encrypted_envelope_index (
            envelope_id, event_id, sender_device_id, local_seq,
            content_type_code, content_key_id, body_len, padding_bucket, applied_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            row.envelope_id,
            row.event_id,
            row.sender_device_id,
            row.local_seq,
            row.content_type_code,
            row.content_key_id,
            row.body_len,
            row.padding_bucket,
            row.applied_at,
        ],
    )?;
    Ok(())
}

pub fn envelope_exists(conn: &Connection, event_id: &str) -> Result<bool> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM encrypted_envelope_index WHERE event_id = ?)",
        params![event_id],
        |r| r.get(0),
    )?;
    Ok(exists)
}

/// Next `local_seq` for a sender (max existing + 1, or 1 if none).
pub fn next_local_seq(conn: &Connection, sender_device_id: &str) -> Result<i64> {
    let max: Option<i64> = conn.query_row(
        "SELECT MAX(local_seq) FROM encrypted_envelope_index WHERE sender_device_id = ?",
        params![sender_device_id],
        |r| r.get(0),
    )?;
    Ok(max.unwrap_or(0) + 1)
}

// ---------------------------------------------------------------------------
// Signed control side store
// ---------------------------------------------------------------------------

/// Persist a signed control envelope (cleartext body + signature).
pub fn insert_signed_control(conn: &Connection, row: &SignedControlRow) -> Result<()> {
    if row.signature.len() != 64 {
        return Err(StoreError::ConfigError(
            "signed control signature must be 64 bytes".to_string(),
        ));
    }
    if row.body.is_empty() {
        return Err(StoreError::ConfigError(
            "signed control body must be non-empty".to_string(),
        ));
    }
    conn.execute(
        "INSERT INTO signed_replication_control (
            event_id, envelope_id, sender_device_id, content_type_code,
            body, signature, schema_version, local_seq, created_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            row.event_id,
            row.envelope_id,
            row.sender_device_id,
            row.content_type_code,
            row.body,
            row.signature,
            row.schema_version,
            row.local_seq,
            row.created_at,
        ],
    )?;
    Ok(())
}

/// Load a signed control by event_id.
pub fn get_signed_control(conn: &Connection, event_id: &str) -> Result<Option<SignedControlRow>> {
    let mut stmt = conn.prepare(
        "SELECT event_id, envelope_id, sender_device_id, content_type_code,
                body, signature, schema_version, local_seq, created_at
         FROM signed_replication_control WHERE event_id = ?",
    )?;
    let row = stmt
        .query_row(params![event_id], |row| {
            Ok(SignedControlRow {
                event_id: row.get(0)?,
                envelope_id: row.get(1)?,
                sender_device_id: row.get(2)?,
                content_type_code: row.get(3)?,
                body: row.get(4)?,
                signature: row.get(5)?,
                schema_version: row.get(6)?,
                local_seq: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .optional()?;
    Ok(row)
}

/// List signed controls for a sender ordered by local_seq.
pub fn list_signed_controls_for_sender(
    conn: &Connection,
    sender_device_id: &str,
) -> Result<Vec<SignedControlRow>> {
    let mut stmt = conn.prepare(
        "SELECT event_id, envelope_id, sender_device_id, content_type_code,
                body, signature, schema_version, local_seq, created_at
         FROM signed_replication_control
         WHERE sender_device_id = ?
         ORDER BY local_seq ASC, event_id ASC",
    )?;
    let rows = stmt.query_map(params![sender_device_id], |row| {
        Ok(SignedControlRow {
            event_id: row.get(0)?,
            envelope_id: row.get(1)?,
            sender_device_id: row.get(2)?,
            content_type_code: row.get(3)?,
            body: row.get(4)?,
            signature: row.get(5)?,
            schema_version: row.get(6)?,
            local_seq: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Atomic bootstrap (identity + private key + signed DeviceEnrolled control)
// ---------------------------------------------------------------------------

/// First-device bootstrap in a single SQLite transaction (R27 / ID-3).
///
/// Fails with a structured `ConfigError` containing `BootstrapAlreadyEnrolled`
/// if any active or local device already exists.
pub fn bootstrap_local_device(
    conn: &mut Connection,
    input: &BootstrapLocalDeviceInput,
) -> Result<()> {
    if has_active_or_local_device(conn)? {
        return Err(StoreError::ConfigError(
            "BootstrapAlreadyEnrolled: an active or local device already exists".to_string(),
        ));
    }
    if input.identity.status != "local" {
        return Err(StoreError::ConfigError(
            "bootstrap_local_device requires identity.status = 'local'".to_string(),
        ));
    }
    if input.identity.device_id != input.identity.enrolled_by_device_id {
        return Err(StoreError::ConfigError(
            "bootstrap self-enroll requires enrolled_by_device_id == device_id".to_string(),
        ));
    }
    let tx = conn.transaction()?;
    insert_device_identity(&tx, &input.identity)?;
    put_device_private_key_wrap(&tx, &input.private_key)?;
    insert_signed_control(&tx, &input.signed_control)?;
    insert_envelope_index(&tx, &input.envelope_index)?;
    tx.commit()?;
    Ok(())
}

/// Enroll a peer: identity + signed DeviceEnrolled control + envelope index (one TX).
pub fn enroll_peer_device(
    conn: &mut Connection,
    identity: &DeviceIdentityRow,
    signed_control: &SignedControlRow,
    envelope_index: &EnvelopeIndexRow,
) -> Result<()> {
    if !has_active_or_local_device(conn)? {
        return Err(StoreError::ConfigError(
            "No enrolled device on this vault; run bootstrap first".to_string(),
        ));
    }
    if identity.status != "active" {
        return Err(StoreError::ConfigError(
            "enroll_peer_device requires identity.status = 'active'".to_string(),
        ));
    }
    let tx = conn.transaction()?;
    insert_device_identity(&tx, identity)?;
    insert_signed_control(&tx, signed_control)?;
    insert_envelope_index(&tx, envelope_index)?;
    tx.commit()?;
    Ok(())
}

/// Revoke: signed DeviceRevoked control + envelope index + tombstone/R23 (one TX).
pub fn revoke_device_with_control(
    conn: &mut Connection,
    device_id: &str,
    revoked_at: &str,
    reason_code: &str,
    signed_control: &SignedControlRow,
    envelope_index: &EnvelopeIndexRow,
) -> Result<()> {
    let existing = get_device(conn, device_id)?;
    let Some(row) = existing else {
        return Err(StoreError::ConfigError(format!(
            "Device not found: {device_id}"
        )));
    };
    if row.status == "revoked" {
        return Err(StoreError::ConfigError(format!(
            "Device already revoked: {device_id}"
        )));
    }
    let tx = conn.transaction()?;
    insert_signed_control(&tx, signed_control)?;
    insert_envelope_index(&tx, envelope_index)?;
    // Inline tombstone + R23 so we stay in the same transaction.
    tx.execute(
        "UPDATE device_identity
         SET status = 'revoked', revoked_at = ?
         WHERE device_id = ?",
        params![revoked_at, device_id],
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO device_id_tombstone (device_id, revoked_at, reason_code)
         VALUES (?, ?, ?)",
        params![device_id, revoked_at, reason_code],
    )?;
    tx.execute(
        "DELETE FROM peer_content_key_wrap WHERE recipient_device_id = ?",
        params![device_id],
    )?;
    tx.commit()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Cursors + gap
// ---------------------------------------------------------------------------

pub fn get_cursor(conn: &Connection, peer_device_id: &str) -> Result<Option<ReplicationCursorRow>> {
    let mut stmt = conn.prepare(
        "SELECT peer_device_id, high_water_seq, expected_local_seq, state, updated_at
         FROM replication_cursor WHERE peer_device_id = ?",
    )?;
    let row = stmt
        .query_row(params![peer_device_id], |row| {
            Ok(ReplicationCursorRow {
                peer_device_id: row.get(0)?,
                high_water_seq: row.get(1)?,
                expected_local_seq: row.get(2)?,
                state: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .optional()?;
    Ok(row)
}

pub fn set_cursor(conn: &Connection, row: &ReplicationCursorRow) -> Result<()> {
    conn.execute(
        "INSERT INTO replication_cursor (
            peer_device_id, high_water_seq, expected_local_seq, state, updated_at
         ) VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(peer_device_id) DO UPDATE SET
            high_water_seq = excluded.high_water_seq,
            expected_local_seq = excluded.expected_local_seq,
            state = excluded.state,
            updated_at = excluded.updated_at",
        params![
            row.peer_device_id,
            row.high_water_seq,
            row.expected_local_seq,
            row.state,
            row.updated_at,
        ],
    )?;
    Ok(())
}

pub fn set_gap_state(
    conn: &Connection,
    peer_device_id: &str,
    state: &str,
    updated_at: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE replication_cursor SET state = ?, updated_at = ? WHERE peer_device_id = ?",
        params![state, updated_at, peer_device_id],
    )?;
    Ok(())
}

pub fn buffer_gap_seq(
    conn: &Connection,
    peer_device_id: &str,
    local_seq: i64,
    envelope_id: &str,
    buffered_at: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO replication_gap_buffer (
            peer_device_id, local_seq, envelope_id, buffered_at
         ) VALUES (?, ?, ?, ?)",
        params![peer_device_id, local_seq, envelope_id, buffered_at],
    )?;
    Ok(())
}

pub fn list_cursors(conn: &Connection) -> Result<Vec<ReplicationCursorRow>> {
    let mut stmt = conn.prepare(
        "SELECT peer_device_id, high_water_seq, expected_local_seq, state, updated_at
         FROM replication_cursor
         ORDER BY peer_device_id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ReplicationCursorRow {
            peer_device_id: row.get(0)?,
            high_water_seq: row.get(1)?,
            expected_local_seq: row.get(2)?,
            state: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Erasure ACK projection
// ---------------------------------------------------------------------------

pub fn upsert_erasure_ack(conn: &Connection, row: &ErasureAckRow) -> Result<()> {
    conn.execute(
        "INSERT INTO erasure_ack_projection (
            erasure_id, peer_device_id, content_key_id, status,
            sync_cycles_waiting, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(erasure_id, peer_device_id) DO UPDATE SET
            content_key_id = excluded.content_key_id,
            status = excluded.status,
            sync_cycles_waiting = excluded.sync_cycles_waiting,
            updated_at = excluded.updated_at",
        params![
            row.erasure_id,
            row.peer_device_id,
            row.content_key_id,
            row.status,
            row.sync_cycles_waiting,
            row.updated_at,
        ],
    )?;
    Ok(())
}

pub fn list_pending_acks(conn: &Connection) -> Result<Vec<ErasureAckRow>> {
    let mut stmt = conn.prepare(
        "SELECT erasure_id, peer_device_id, content_key_id, status,
                sync_cycles_waiting, updated_at
         FROM erasure_ack_projection
         WHERE status = 'pending'
         ORDER BY erasure_id ASC, peer_device_id ASC",
    )?;
    let rows = stmt.query_map([], map_ack_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn map_ack_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ErasureAckRow> {
    Ok(ErasureAckRow {
        erasure_id: row.get(0)?,
        peer_device_id: row.get(1)?,
        content_key_id: row.get(2)?,
        status: row.get(3)?,
        sync_cycles_waiting: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

/// Increment `sync_cycles_waiting` for pending acks; mark `unreachable` at N.
///
/// Production T176 does not invoke this (no relay cycles). Unit-testable; T177 wires cycles.
pub fn tick_ack_cycle(conn: &Connection, updated_at: &str) -> Result<usize> {
    // Increment pending.
    conn.execute(
        "UPDATE erasure_ack_projection
         SET sync_cycles_waiting = sync_cycles_waiting + 1,
             updated_at = ?
         WHERE status = 'pending'",
        params![updated_at],
    )?;
    // Promote to unreachable at threshold.
    let n = conn.execute(
        "UPDATE erasure_ack_projection
         SET status = 'unreachable', updated_at = ?
         WHERE status = 'pending' AND sync_cycles_waiting >= ?",
        params![updated_at, ACK_TIMEOUT_SYNC_CYCLES as i64],
    )?;
    Ok(n)
}

// ---------------------------------------------------------------------------
// Gap skip audit index
// ---------------------------------------------------------------------------

pub fn insert_gap_skip_audit(
    conn: &Connection,
    audit_id: &str,
    peer_device_id: &str,
    skipped_seq: i64,
    signed_event_id: &str,
    created_at: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO replication_gap_skip_audit (
            audit_id, peer_device_id, skipped_seq, signed_event_id, created_at
         ) VALUES (?, ?, ?, ?, ?)",
        params![
            audit_id,
            peer_device_id,
            skipped_seq,
            signed_event_id,
            created_at
        ],
    )?;
    Ok(())
}
