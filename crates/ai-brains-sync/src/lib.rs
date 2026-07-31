//! Multi-device encrypted event envelope replication (ADR-0018 / T176–T177).
//!
//! This crate provides **wire format, fake relays, and crypto primitives**:
//! device identity, enrollment package, outer sign/verify, per-recipient DEK
//! wrap, control payloads, padding, apply-order tie-break, `wire_v1` framing,
//! and fake relays (memory / file / adversarial).
//!
//! **`ReplicateEngine`** (push/pull apply pipeline, durable outbox, cursors)
//! lives in **`ai-brains-store`**, not here.
//!
//! **No production network.** Fake relays are test/dev only (T177 Phase A).
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
pub mod relay;
pub mod signed_bytes;
pub mod wire;
pub mod wrap;

pub use apply_order::{ApplyOrderKey, sort_by_apply_order};
pub use control::{
    ContentErasureTombstonePayload, ControlPayload, DeviceEnrolledPayload, DeviceRevokedPayload,
    ErasureAckPayload, GapSkipAuditPayload, SignedControlEnvelope, build_and_sign_control,
    decode_control_payload, encode_control_payload, nil_content_key_id,
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
pub use relay::{
    AdversarialRelay, FAKE_RELAY_MARKER, FileFakeRelay, MemoryFakeRelay, RelayBlob, RelayPort,
};
pub use signed_bytes::{
    EnvelopeId, WrapRecord, build_signed_bytes, parse_signed_bytes, wraps_are_sorted,
};
pub use wire::{
    WIRE_MAGIC, WIRE_MAX_SIZE, WIRE_SIGNATURE_LEN, WIRE_VERSION_V1, decode_signed_envelope,
    encode_signed_envelope,
};
pub use wrap::{
    LABEL_AIB_SYNC_DEK_WRAP, PeerDekWrap, build_wrap_aad, build_wrap_info, unwrap_content_dek,
    wrap_content_dek_for_recipient,
};

/// Default ACK timeout in sync cycles (R14 / L7).
pub const ACK_TIMEOUT_SYNC_CYCLES: u32 = 3;
