//! Canonical `signed_bytes` concat (ADR-0018 §5.2).
//!
//! All multi-byte integers big-endian; UUIDs as 16 raw bytes. No JSON, no LE.
//! Wrap list MUST be sorted by `recipient_device_id` ascending (unsigned byte order).

use crate::error::{Result, SyncError};
use ai_brains_core::ids::{ContentKeyId, DeviceId, ReplicationEventId};
use uuid::Uuid;

/// Outer envelope id (transport).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnvelopeId(Uuid);

impl EnvelopeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl Default for EnvelopeId {
    fn default() -> Self {
        Self::new()
    }
}

/// One per-recipient wrap record on the wire (§5.2 / §17).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapRecord {
    pub recipient_device_id: DeviceId,
    pub eph_x25519_pub: [u8; 32],
    pub wrap_nonce: [u8; 12],
    pub wrap_ct: Vec<u8>,
}

/// Fields that enter `signed_bytes` (signature itself is detached).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedBytesInput {
    pub schema_version: u16,
    pub envelope_id: EnvelopeId,
    pub device_id: DeviceId,
    pub local_seq: u64,
    pub content_type_code: u16,
    pub event_id: ReplicationEventId,
    pub content_key_id: ContentKeyId,
    /// Body field: data AEAD blob or cleartext control payload.
    pub ciphertext: Vec<u8>,
    /// Wrap records; must already be sorted by recipient ascending, or
    /// [`build_signed_bytes`] rejects with [`SyncError::UnsortedWrapList`].
    pub wrap_records: Vec<WrapRecord>,
}

/// Encode one wrap record: recipient ‖ eph_pub ‖ nonce ‖ wrap_ct_len u32 BE ‖ wrap_ct.
pub fn encode_wrap_record(rec: &WrapRecord) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + 32 + 12 + 4 + rec.wrap_ct.len());
    out.extend_from_slice(rec.recipient_device_id.as_uuid().as_bytes());
    out.extend_from_slice(&rec.eph_x25519_pub);
    out.extend_from_slice(&rec.wrap_nonce);
    let ct_len = rec.wrap_ct.len() as u32;
    out.extend_from_slice(&ct_len.to_be_bytes());
    out.extend_from_slice(&rec.wrap_ct);
    out
}

/// True iff wrap records are **strictly increasing** by recipient_device_id bytes.
///
/// Duplicate recipient_device_id values are rejected (not merely unsorted).
pub fn wraps_are_sorted(records: &[WrapRecord]) -> bool {
    records.windows(2).all(|w| {
        w[0].recipient_device_id.as_uuid().as_bytes()
            < w[1].recipient_device_id.as_uuid().as_bytes()
    })
}

/// Build canonical `signed_bytes` (ADR-0018 §5.2). Rejects unsorted wrap lists.
pub fn build_signed_bytes(input: &SignedBytesInput) -> Result<Vec<u8>> {
    if !wraps_are_sorted(&input.wrap_records) {
        return Err(SyncError::UnsortedWrapList);
    }
    let ciphertext_len = input.ciphertext.len() as u32;
    let wrap_count = input.wrap_records.len() as u32;

    let mut out = Vec::with_capacity(
        2 + 16
            + 16
            + 8
            + 2
            + 16
            + 16
            + 4
            + input.ciphertext.len()
            + 4
            + input
                .wrap_records
                .iter()
                .map(|r| 16 + 32 + 12 + 4 + r.wrap_ct.len())
                .sum::<usize>(),
    );
    out.extend_from_slice(&input.schema_version.to_be_bytes());
    out.extend_from_slice(input.envelope_id.as_bytes());
    out.extend_from_slice(input.device_id.as_uuid().as_bytes());
    out.extend_from_slice(&input.local_seq.to_be_bytes());
    out.extend_from_slice(&input.content_type_code.to_be_bytes());
    out.extend_from_slice(input.event_id.as_uuid().as_bytes());
    out.extend_from_slice(input.content_key_id.as_uuid().as_bytes());
    out.extend_from_slice(&ciphertext_len.to_be_bytes());
    out.extend_from_slice(&input.ciphertext);
    out.extend_from_slice(&wrap_count.to_be_bytes());
    for rec in &input.wrap_records {
        out.extend_from_slice(&encode_wrap_record(rec));
    }
    Ok(out)
}

/// Fixed header length before variable ciphertext: schema…content_key + ct_len.
const SIGNED_BYTES_FIXED_PREFIX: usize = 2 + 16 + 16 + 8 + 2 + 16 + 16 + 4;

/// Decode one wrap record; returns `(record, bytes_consumed)`.
pub fn parse_wrap_record(bytes: &[u8]) -> Result<(WrapRecord, usize)> {
    // recipient(16) + eph(32) + nonce(12) + ct_len(4)
    const FIXED: usize = 16 + 32 + 12 + 4;
    if bytes.len() < FIXED {
        return Err(SyncError::InvalidEncoding(format!(
            "wrap record truncated: {} < {FIXED}",
            bytes.len()
        )));
    }
    let recipient = Uuid::from_bytes(
        bytes[0..16]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("wrap recipient uuid".to_string()))?,
    );
    let mut eph_x25519_pub = [0u8; 32];
    eph_x25519_pub.copy_from_slice(&bytes[16..48]);
    let mut wrap_nonce = [0u8; 12];
    wrap_nonce.copy_from_slice(&bytes[48..60]);
    let ct_len = u32::from_be_bytes(
        bytes[60..64]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("wrap_ct_len".to_string()))?,
    ) as usize;
    let total = FIXED + ct_len;
    if bytes.len() < total {
        return Err(SyncError::InvalidEncoding(format!(
            "wrap_ct truncated: need {total}, have {}",
            bytes.len()
        )));
    }
    let wrap_ct = bytes[64..total].to_vec();
    Ok((
        WrapRecord {
            recipient_device_id: DeviceId::from_uuid(recipient),
            eph_x25519_pub,
            wrap_nonce,
            wrap_ct,
        },
        total,
    ))
}

/// Inverse of [`build_signed_bytes`]. Requires exact consumption (no leftover bytes).
pub fn parse_signed_bytes(bytes: &[u8]) -> Result<SignedBytesInput> {
    if bytes.len() < SIGNED_BYTES_FIXED_PREFIX {
        return Err(SyncError::InvalidEncoding(format!(
            "signed_bytes truncated: {} < {SIGNED_BYTES_FIXED_PREFIX}",
            bytes.len()
        )));
    }
    let mut off = 0usize;

    let schema_version = u16::from_be_bytes(
        bytes[off..off + 2]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("schema_version".to_string()))?,
    );
    off += 2;

    let envelope_id = EnvelopeId::from_uuid(Uuid::from_bytes(
        bytes[off..off + 16]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("envelope_id".to_string()))?,
    ));
    off += 16;

    let device_id = DeviceId::from_uuid(Uuid::from_bytes(
        bytes[off..off + 16]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("device_id".to_string()))?,
    ));
    off += 16;

    let local_seq = u64::from_be_bytes(
        bytes[off..off + 8]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("local_seq".to_string()))?,
    );
    off += 8;

    let content_type_code = u16::from_be_bytes(
        bytes[off..off + 2]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("content_type_code".to_string()))?,
    );
    off += 2;

    let event_id = ReplicationEventId::from_uuid(Uuid::from_bytes(
        bytes[off..off + 16]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("event_id".to_string()))?,
    ));
    off += 16;

    let content_key_id = ContentKeyId::from_uuid(Uuid::from_bytes(
        bytes[off..off + 16]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("content_key_id".to_string()))?,
    ));
    off += 16;

    let ciphertext_len = u32::from_be_bytes(
        bytes[off..off + 4]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("ciphertext_len".to_string()))?,
    ) as usize;
    off += 4;

    if bytes.len() < off + ciphertext_len + 4 {
        return Err(SyncError::InvalidEncoding(format!(
            "ciphertext/wrap_count truncated at offset {off}"
        )));
    }
    let ciphertext = bytes[off..off + ciphertext_len].to_vec();
    off += ciphertext_len;

    let wrap_count = u32::from_be_bytes(
        bytes[off..off + 4]
            .try_into()
            .map_err(|_| SyncError::InvalidEncoding("wrap_count".to_string()))?,
    ) as usize;
    off += 4;

    let mut wrap_records = Vec::with_capacity(wrap_count);
    for i in 0..wrap_count {
        let (rec, consumed) = parse_wrap_record(&bytes[off..]).map_err(|e| match e {
            SyncError::InvalidEncoding(msg) => {
                SyncError::InvalidEncoding(format!("wrap[{i}]: {msg}"))
            }
            other => other,
        })?;
        off += consumed;
        wrap_records.push(rec);
    }

    if off != bytes.len() {
        return Err(SyncError::InvalidEncoding(format!(
            "signed_bytes leftover {} bytes after parse",
            bytes.len() - off
        )));
    }

    if !wraps_are_sorted(&wrap_records) {
        return Err(SyncError::UnsortedWrapList);
    }

    Ok(SignedBytesInput {
        schema_version,
        envelope_id,
        device_id,
        local_seq,
        content_type_code,
        event_id,
        content_key_id,
        ciphertext,
        wrap_records,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use uuid::Uuid;

    fn uuid_n(n: u8) -> Uuid {
        let mut b = [0u8; 16];
        b[15] = n;
        Uuid::from_bytes(b)
    }

    /// T178-L5-sig-canonical-bytes — thin wrapper over fixture KAT (F7).
    #[test]
    fn t178_l5_sig_canonical_bytes__kat() {
        // T178-L5-sig-canonical-bytes
        signed_bytes__fixture__exact_hex();
    }

    #[test]
    fn signed_bytes__fixture__exact_hex() {
        // Fixed deterministic fixture for §5.2 KAT.
        let input = SignedBytesInput {
            schema_version: 1,
            envelope_id: EnvelopeId::from_uuid(uuid_n(1)),
            device_id: DeviceId::from_uuid(uuid_n(2)),
            local_seq: 7,
            content_type_code: 0x0010,
            event_id: ReplicationEventId::from_uuid(uuid_n(3)),
            content_key_id: ContentKeyId::from_uuid(Uuid::nil()),
            ciphertext: vec![0xAA, 0xBB, 0xCC],
            wrap_records: vec![],
        };
        let bytes = build_signed_bytes(&input).expect("build");
        // Manual expected:
        // schema 00 01
        // envelope 00..01
        // device 00..02
        // seq 00 00 00 00 00 00 00 07
        // type 00 10
        // event 00..03
        // content_key nil
        // ct_len 00 00 00 03
        // ct AA BB CC
        // wrap_count 00 00 00 00
        let expected = hex::decode(concat!(
            "0001",                             // schema
            "00000000000000000000000000000001", // envelope
            "00000000000000000000000000000002", // device
            "0000000000000007",                 // seq
            "0010",                             // type DeviceEnrolled
            "00000000000000000000000000000003", // event
            "00000000000000000000000000000000", // content_key nil
            "00000003",                         // ct len
            "aabbcc",                           // body
            "00000000",                         // wrap_count
        ))
        .expect("hex");
        assert_eq!(bytes, expected, "got {}", hex::encode(&bytes));
    }

    #[test]
    fn signed_bytes__unsorted_wraps__err() {
        let high = DeviceId::from_uuid(uuid_n(9));
        let low = DeviceId::from_uuid(uuid_n(1));
        let input = SignedBytesInput {
            schema_version: 1,
            envelope_id: EnvelopeId::from_uuid(uuid_n(1)),
            device_id: DeviceId::from_uuid(uuid_n(2)),
            local_seq: 1,
            content_type_code: 0x0001,
            event_id: ReplicationEventId::from_uuid(uuid_n(3)),
            content_key_id: ContentKeyId::from_uuid(uuid_n(4)),
            ciphertext: vec![0; 28], // min nonce+tag
            wrap_records: vec![
                WrapRecord {
                    recipient_device_id: high,
                    eph_x25519_pub: [0; 32],
                    wrap_nonce: [0; 12],
                    wrap_ct: vec![0; 48],
                },
                WrapRecord {
                    recipient_device_id: low,
                    eph_x25519_pub: [0; 32],
                    wrap_nonce: [0; 12],
                    wrap_ct: vec![0; 48],
                },
            ],
        };
        let err = build_signed_bytes(&input).expect_err("unsorted");
        assert!(matches!(err, SyncError::UnsortedWrapList));
    }

    #[test]
    fn signed_bytes__duplicate_recipient__err() {
        let same = DeviceId::from_uuid(uuid_n(5));
        let input = SignedBytesInput {
            schema_version: 1,
            envelope_id: EnvelopeId::from_uuid(uuid_n(1)),
            device_id: DeviceId::from_uuid(uuid_n(2)),
            local_seq: 1,
            content_type_code: 0x0001,
            event_id: ReplicationEventId::from_uuid(uuid_n(3)),
            content_key_id: ContentKeyId::from_uuid(uuid_n(4)),
            ciphertext: vec![0; 28],
            wrap_records: vec![
                WrapRecord {
                    recipient_device_id: same,
                    eph_x25519_pub: [1; 32],
                    wrap_nonce: [2; 12],
                    wrap_ct: vec![3; 48],
                },
                WrapRecord {
                    recipient_device_id: same,
                    eph_x25519_pub: [4; 32],
                    wrap_nonce: [5; 12],
                    wrap_ct: vec![6; 48],
                },
            ],
        };
        assert!(!wraps_are_sorted(&input.wrap_records));
        let err = build_signed_bytes(&input).expect_err("duplicate recipient");
        assert!(matches!(err, SyncError::UnsortedWrapList));
    }
}
