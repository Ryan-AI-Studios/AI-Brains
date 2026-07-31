//! Structured errors for multi-device replication (T176–T177 / ADR-0018).
//!
//! Auth-failure paths use generic messages (no plaintext / key material leak).

use thiserror::Error;

/// Errors from envelope codec, device keys, wrap, wire framing, and relay helpers.
#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Entropy source failed: {0}")]
    EntropyFailed(String),

    #[error("Unknown content_type_code: 0x{0:04x}")]
    UnknownContentType(u16),

    #[error("Signature invalid")]
    SignatureInvalid,

    #[error("Unsorted wrap list (must be recipient_device_id ascending)")]
    UnsortedWrapList,

    #[error("Wrap open failed")]
    WrapOpenFailed,

    #[error("Invalid wrap or envelope encoding: {0}")]
    InvalidEncoding(String),

    #[error("Invalid wire framing: {0}")]
    InvalidWire(String),

    #[error("Wire envelope exceeds 16 MiB size cap")]
    WireTooLarge,

    #[error("Relay blob body exceeds 16 MiB size cap")]
    RelayBlobTooLarge,

    #[error("Relay sequence collision: same (device, seq) with different envelope_id")]
    SeqCollision,

    #[error("Relay conflict: {0}")]
    RelayConflict(String),

    #[error("Relay path refused: {0}")]
    RelayPathRefused(String),

    #[error("Relay I/O error: {0}")]
    RelayIo(String),

    #[error("Schema version mismatch: got {got}, expected {expected}")]
    SchemaVersionMismatch { got: u16, expected: u16 },

    #[error("Device id is tombstoned and permanently retired")]
    TombstonedDeviceId,

    #[error("Device not enrolled or not in enrolled-set")]
    NotEnrolled,

    #[error("Device revoked")]
    DeviceRevoked,

    #[error("Bootstrap already enrolled (active/local device exists)")]
    BootstrapAlreadyEnrolled,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Invalid key length")]
    InvalidKeyLength,

    #[error("Invalid nonce length")]
    InvalidNonceLength,

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Relay not configured / deferred to T177")]
    RelayNotConfigured,
}

pub type Result<T> = std::result::Result<T, SyncError>;
