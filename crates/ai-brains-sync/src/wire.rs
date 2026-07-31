//! Wire framing for `SignedEnvelope` (T177 F20 / ADR-0018).
//!
//! ```text
//! wire_v1 = magic[4]=b"AIBR" ‖ version[1]=0x01 ‖ signed_bytes ‖ signature[64]
//! ```
//!
//! Integers inside `signed_bytes` are big-endian (see [`crate::signed_bytes`]).
//! Total framed size is soft-capped at 16 MiB on encode and decode.

use crate::envelope::{ContentTypeCode, OuterEnvelope, SignedEnvelope};
use crate::error::{Result, SyncError};
use crate::signed_bytes::{build_signed_bytes, parse_signed_bytes};

/// Magic bytes for AI-Brains Relay wire_v1.
pub const WIRE_MAGIC: &[u8; 4] = b"AIBR";

/// Wire format version byte.
pub const WIRE_VERSION_V1: u8 = 0x01;

/// Ed25519 detached signature length.
pub const WIRE_SIGNATURE_LEN: usize = 64;

/// Soft cap on total `wire_v1` size (F3).
pub const WIRE_MAX_SIZE: usize = 16 * 1024 * 1024;

/// Minimum framed length: magic + version + empty signed_bytes + signature.
const WIRE_MIN_FRAMING: usize = 4 + 1 + WIRE_SIGNATURE_LEN;

/// Encode a signed envelope as `wire_v1` bytes.
pub fn encode_signed_envelope(env: &SignedEnvelope) -> Result<Vec<u8>> {
    let signed = build_signed_bytes(&crate::signed_bytes::SignedBytesInput {
        schema_version: env.outer.schema_version,
        envelope_id: env.outer.envelope_id,
        device_id: env.outer.device_id,
        local_seq: env.outer.local_seq,
        content_type_code: env.outer.content_type_code.as_u16(),
        event_id: env.outer.event_id,
        content_key_id: env.outer.content_key_id,
        ciphertext: env.outer.ciphertext.clone(),
        wrap_records: env.outer.wrap_records.clone(),
    })?;

    let total = 4 + 1 + signed.len() + WIRE_SIGNATURE_LEN;
    if total > WIRE_MAX_SIZE {
        return Err(SyncError::WireTooLarge);
    }

    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(WIRE_MAGIC);
    out.push(WIRE_VERSION_V1);
    out.extend_from_slice(&signed);
    out.extend_from_slice(&env.signature);
    Ok(out)
}

/// Decode `wire_v1` bytes into a [`SignedEnvelope`].
///
/// Rejects bad magic, bad version, truncated frames, leftover interior bytes,
/// unknown content types, and frames larger than [`WIRE_MAX_SIZE`].
pub fn decode_signed_envelope(bytes: &[u8]) -> Result<SignedEnvelope> {
    if bytes.len() > WIRE_MAX_SIZE {
        return Err(SyncError::WireTooLarge);
    }
    if bytes.len() < WIRE_MIN_FRAMING {
        return Err(SyncError::InvalidWire(format!(
            "truncated: {} < min {WIRE_MIN_FRAMING}",
            bytes.len()
        )));
    }
    if &bytes[0..4] != WIRE_MAGIC.as_slice() {
        return Err(SyncError::InvalidWire(format!(
            "bad magic: got {:02x?}",
            &bytes[0..4]
        )));
    }
    let version = bytes[4];
    if version != WIRE_VERSION_V1 {
        return Err(SyncError::InvalidWire(format!(
            "unsupported wire version: 0x{version:02x}"
        )));
    }

    let sig_start = bytes.len() - WIRE_SIGNATURE_LEN;
    let signed_bytes = &bytes[5..sig_start];
    let mut signature = [0u8; WIRE_SIGNATURE_LEN];
    signature.copy_from_slice(&bytes[sig_start..]);

    let input = parse_signed_bytes(signed_bytes)?;
    let content_type_code = ContentTypeCode::from_u16(input.content_type_code)?;

    Ok(SignedEnvelope {
        outer: OuterEnvelope {
            schema_version: input.schema_version,
            envelope_id: input.envelope_id,
            device_id: input.device_id,
            local_seq: input.local_seq,
            content_type_code,
            event_id: input.event_id,
            content_key_id: input.content_key_id,
            ciphertext: input.ciphertext,
            wrap_records: input.wrap_records,
        },
        signature,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use crate::envelope::ContentTypeCode;
    use crate::error::SyncError;
    use crate::signed_bytes::{EnvelopeId, WrapRecord};
    use ai_brains_core::ids::{ContentKeyId, DeviceId, ReplicationEventId};
    use uuid::Uuid;

    fn uuid_n(n: u8) -> Uuid {
        let mut b = [0u8; 16];
        b[15] = n;
        Uuid::from_bytes(b)
    }

    /// Fixed control envelope used for wire KAT (empty wraps).
    fn fixture_signed() -> SignedEnvelope {
        SignedEnvelope {
            outer: OuterEnvelope {
                schema_version: 1,
                envelope_id: EnvelopeId::from_uuid(uuid_n(1)),
                device_id: DeviceId::from_uuid(uuid_n(2)),
                local_seq: 7,
                content_type_code: ContentTypeCode::DeviceEnrolled,
                event_id: ReplicationEventId::from_uuid(uuid_n(3)),
                content_key_id: ContentKeyId::from_uuid(Uuid::nil()),
                ciphertext: vec![0xAA, 0xBB, 0xCC],
                wrap_records: vec![],
            },
            signature: [0xAB; 64],
        }
    }

    #[test]
    fn wire_signed_envelope__fixture__exact_hex() {
        let env = fixture_signed();
        let bytes = encode_signed_envelope(&env).expect("encode");
        // magic AIBR + version 01 + signed_bytes (same as signed_bytes KAT) + sig AB*64
        let expected = hex::decode(concat!(
            "41494252",                         // AIBR
            "01",                               // version
            "0001",                             // schema
            "00000000000000000000000000000001", // envelope
            "00000000000000000000000000000002", // device
            "0000000000000007",                 // seq
            "0010",                             // DeviceEnrolled
            "00000000000000000000000000000003", // event
            "00000000000000000000000000000000", // content_key nil
            "00000003",                         // ct len
            "aabbcc",                           // body
            "00000000",                         // wrap_count
            // 64 × 0xAB
            "abababababababababababababababab",
            "abababababababababababababababab",
            "abababababababababababababababab",
            "abababababababababababababababab",
        ))
        .expect("hex");
        assert_eq!(bytes, expected, "got {}", hex::encode(&bytes));
    }

    #[test]
    fn wire_signed_envelope__roundtrip__eq() {
        let env = fixture_signed();
        let bytes = encode_signed_envelope(&env).expect("encode");
        let decoded = decode_signed_envelope(&bytes).expect("decode");
        assert_eq!(decoded, env);

        // With a wrap record as well.
        let mut with_wrap = env;
        with_wrap.outer.content_type_code = ContentTypeCode::DataEvent;
        with_wrap.outer.ciphertext = vec![0u8; 28];
        with_wrap.outer.wrap_records = vec![WrapRecord {
            recipient_device_id: DeviceId::from_uuid(uuid_n(9)),
            eph_x25519_pub: [0x11; 32],
            wrap_nonce: [0x22; 12],
            wrap_ct: vec![0x33; 48],
        }];
        with_wrap.signature = [0xCD; 64];
        let bytes = encode_signed_envelope(&with_wrap).expect("encode wrap");
        let decoded = decode_signed_envelope(&bytes).expect("decode wrap");
        assert_eq!(decoded, with_wrap);
    }

    #[test]
    fn wire_signed_envelope__size_cap__err() {
        // Header overhead: magic(4)+ver(1)+signed fixed prefix(2+16+16+8+2+16+16+4)
        // + wrap_count(4) + sig(64) = 5 + 80 + 4 + 64 = 153 without ciphertext.
        // Choose ciphertext so total exceeds 16 MiB.
        let overhead = 4 + 1 + (2 + 16 + 16 + 8 + 2 + 16 + 16 + 4) + 4 + WIRE_SIGNATURE_LEN;
        let ct_len = WIRE_MAX_SIZE - overhead + 1;
        let env = SignedEnvelope {
            outer: OuterEnvelope {
                schema_version: 1,
                envelope_id: EnvelopeId::from_uuid(uuid_n(1)),
                device_id: DeviceId::from_uuid(uuid_n(2)),
                local_seq: 1,
                content_type_code: ContentTypeCode::DeviceEnrolled,
                event_id: ReplicationEventId::from_uuid(uuid_n(3)),
                content_key_id: ContentKeyId::from_uuid(Uuid::nil()),
                ciphertext: vec![0u8; ct_len],
                wrap_records: vec![],
            },
            signature: [0xAB; 64],
        };
        let err = encode_signed_envelope(&env).expect_err("must exceed cap");
        assert!(matches!(err, SyncError::WireTooLarge), "got: {err:?}");

        // Decode path: oversized buffer.
        let huge = vec![0u8; WIRE_MAX_SIZE + 1];
        let err = decode_signed_envelope(&huge).expect_err("decode cap");
        assert!(matches!(err, SyncError::WireTooLarge), "got: {err:?}");
    }

    #[test]
    fn wire_signed_envelope__bad_magic__err() {
        let mut bytes = encode_signed_envelope(&fixture_signed()).expect("encode");
        bytes[0] = b'X';
        let err = decode_signed_envelope(&bytes).expect_err("magic");
        assert!(
            matches!(err, SyncError::InvalidWire(ref m) if m.contains("magic")),
            "got: {err:?}"
        );
    }

    #[test]
    fn wire_signed_envelope__bad_version__err() {
        let mut bytes = encode_signed_envelope(&fixture_signed()).expect("encode");
        bytes[4] = 0x99;
        let err = decode_signed_envelope(&bytes).expect_err("version");
        assert!(
            matches!(err, SyncError::InvalidWire(ref m) if m.contains("version")),
            "got: {err:?}"
        );
    }

    #[test]
    fn wire_signed_envelope__truncated__err() {
        let err = decode_signed_envelope(&[0x41, 0x49, 0x42, 0x52, 0x01]).expect_err("short");
        assert!(matches!(err, SyncError::InvalidWire(_)), "got: {err:?}");
    }
}
