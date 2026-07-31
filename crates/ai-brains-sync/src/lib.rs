//! Multi-device encrypted event envelope replication (ADR-0018 / T176).
//!
//! Library only: device identity, enrollment package, outer sign/verify,
//! per-recipient DEK wrap, control payloads, padding, apply-order tie-break.
//! **No sockets / no relay client** (T177).
//!
//! # Honesty
//!
//! - Optional multi-device; local-only default.
//! - **Not** post-quantum.
//! - **Not** remote wipe / NIST Purge.
//! - **Not** metadata-private (padding is best-effort only).

#![forbid(unsafe_code)]

pub mod apply_order;
pub mod control;
pub mod device_keys;
pub mod enrollment;
pub mod envelope;
pub mod error;
pub mod fingerprint_fmt;
pub mod padding;
pub mod private_blob;
pub mod signed_bytes;
pub mod wrap;

pub use apply_order::{ApplyOrderKey, sort_by_apply_order};
pub use control::{
    ContentErasureTombstonePayload, DeviceEnrolledPayload, DeviceRevokedPayload, ErasureAckPayload,
    GapSkipAuditPayload, decode_control_payload, encode_control_payload,
};
pub use device_keys::{DeviceKeyPair, generate_device_keys};
pub use enrollment::{
    ENROLLMENT_PACKAGE_LEN, REPLICATION_SCHEMA_VERSION, enrollment_package, fingerprint_sha256,
    parse_enrollment_package,
};
pub use envelope::{
    CONTENT_TYPE_CONTENT_ERASURE_TOMBSTONE, CONTENT_TYPE_DATA_EVENT, CONTENT_TYPE_DEVICE_ENROLLED,
    CONTENT_TYPE_DEVICE_REVOKED, CONTENT_TYPE_ERASURE_ACK, CONTENT_TYPE_GAP_SKIP_AUDIT,
    ContentTypeCode, OuterEnvelope, SignedEnvelope, decode_data_body, encode_data_body,
    sign_envelope, verify_envelope,
};
pub use error::{Result, SyncError};
pub use fingerprint_fmt::format_fingerprint_hyphen;
pub use padding::{PAD_BUCKETS, pad_to_bucket};
pub use private_blob::{
    AAD_KIND_DEVICE_PRIVATE_KEY, DEVICE_PRIVATE_PLAINTEXT_LEN, DevicePrivateSeeds,
    SealedDevicePrivate, open_device_private_blob, seal_device_private_blob,
};
pub use signed_bytes::{WrapRecord, build_signed_bytes};
pub use wrap::{
    LABEL_AIB_SYNC_DEK_WRAP, PeerDekWrap, build_wrap_aad, build_wrap_info, unwrap_content_dek,
    wrap_content_dek_for_recipient,
};

/// Default ACK timeout in sync cycles (R14 / L7).
pub const ACK_TIMEOUT_SYNC_CYCLES: u32 = 3;
