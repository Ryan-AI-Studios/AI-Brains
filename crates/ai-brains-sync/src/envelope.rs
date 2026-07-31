//! Outer signed envelope encode/sign/verify + data body pack (ADR-0018 §5.2–5.3).

use crate::error::{Result, SyncError};
use crate::signed_bytes::{
    EnvelopeId, SignedBytesInput, WrapRecord, build_signed_bytes, wraps_are_sorted,
};
use ai_brains_core::ids::{ContentKeyId, DeviceId, ReplicationEventId};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/// Content type codes (R8 / §6.2).
pub const CONTENT_TYPE_DATA_EVENT: u16 = 0x0001;
pub const CONTENT_TYPE_DEVICE_ENROLLED: u16 = 0x0010;
pub const CONTENT_TYPE_DEVICE_REVOKED: u16 = 0x0011;
pub const CONTENT_TYPE_CONTENT_ERASURE_TOMBSTONE: u16 = 0x0012;
pub const CONTENT_TYPE_ERASURE_ACK: u16 = 0x0013;
pub const CONTENT_TYPE_GAP_SKIP_AUDIT: u16 = 0x0014;

/// Known content type codes; unknown → reject (fail closed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ContentTypeCode {
    DataEvent = CONTENT_TYPE_DATA_EVENT,
    DeviceEnrolled = CONTENT_TYPE_DEVICE_ENROLLED,
    DeviceRevoked = CONTENT_TYPE_DEVICE_REVOKED,
    ContentErasureTombstone = CONTENT_TYPE_CONTENT_ERASURE_TOMBSTONE,
    ErasureAck = CONTENT_TYPE_ERASURE_ACK,
    GapSkipAudit = CONTENT_TYPE_GAP_SKIP_AUDIT,
}

impl ContentTypeCode {
    pub fn from_u16(code: u16) -> Result<Self> {
        match code {
            CONTENT_TYPE_DATA_EVENT => Ok(Self::DataEvent),
            CONTENT_TYPE_DEVICE_ENROLLED => Ok(Self::DeviceEnrolled),
            CONTENT_TYPE_DEVICE_REVOKED => Ok(Self::DeviceRevoked),
            CONTENT_TYPE_CONTENT_ERASURE_TOMBSTONE => Ok(Self::ContentErasureTombstone),
            CONTENT_TYPE_ERASURE_ACK => Ok(Self::ErasureAck),
            CONTENT_TYPE_GAP_SKIP_AUDIT => Ok(Self::GapSkipAudit),
            other => Err(SyncError::UnknownContentType(other)),
        }
    }

    pub fn is_control(self) -> bool {
        !matches!(self, Self::DataEvent)
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Unsigned outer envelope fields (before signature).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OuterEnvelope {
    pub schema_version: u16,
    pub envelope_id: EnvelopeId,
    pub device_id: DeviceId,
    pub local_seq: u64,
    pub content_type_code: ContentTypeCode,
    pub event_id: ReplicationEventId,
    pub content_key_id: ContentKeyId,
    pub ciphertext: Vec<u8>,
    pub wrap_records: Vec<WrapRecord>,
}

/// Signed outer envelope with detached Ed25519 signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedEnvelope {
    pub outer: OuterEnvelope,
    pub signature: [u8; 64],
}

fn to_signed_input(outer: &OuterEnvelope) -> SignedBytesInput {
    SignedBytesInput {
        schema_version: outer.schema_version,
        envelope_id: outer.envelope_id,
        device_id: outer.device_id,
        local_seq: outer.local_seq,
        content_type_code: outer.content_type_code.as_u16(),
        event_id: outer.event_id,
        content_key_id: outer.content_key_id,
        ciphertext: outer.ciphertext.clone(),
        wrap_records: outer.wrap_records.clone(),
    }
}

/// Sign an outer envelope. Control envelopes must have `wrap_count = 0`.
pub fn sign_envelope(outer: &OuterEnvelope, signing_key: &SigningKey) -> Result<SignedEnvelope> {
    if outer.content_type_code.is_control() && !outer.wrap_records.is_empty() {
        return Err(SyncError::InvalidEncoding(
            "control envelopes must have wrap_count = 0".to_string(),
        ));
    }
    // Reject unknown is already enforced by ContentTypeCode type.
    let signed = build_signed_bytes(&to_signed_input(outer))?;
    let sig = signing_key.sign(&signed);
    Ok(SignedEnvelope {
        outer: outer.clone(),
        signature: sig.to_bytes(),
    })
}

/// Verify detached signature over canonical `signed_bytes`.
///
/// Rejects unsorted wrap lists and unknown type codes. Meta-swap fails closed.
pub fn verify_envelope(signed: &SignedEnvelope, verifying_key: &VerifyingKey) -> Result<()> {
    // Fail closed on unknown codes if reconstructed.
    let _ = ContentTypeCode::from_u16(signed.outer.content_type_code.as_u16())?;
    if signed.outer.content_type_code.is_control() && !signed.outer.wrap_records.is_empty() {
        return Err(SyncError::InvalidEncoding(
            "control envelopes must have wrap_count = 0".to_string(),
        ));
    }
    if !wraps_are_sorted(&signed.outer.wrap_records) {
        return Err(SyncError::UnsortedWrapList);
    }
    let message = build_signed_bytes(&to_signed_input(&signed.outer))?;
    let signature = Signature::from_bytes(&signed.signature);
    verifying_key
        .verify(&message, &signature)
        .map_err(|_| SyncError::SignatureInvalid)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Data body: nonce(12) ‖ ct ‖ tag(16)
// ---------------------------------------------------------------------------

pub const DATA_BODY_NONCE_LEN: usize = 12;
pub const DATA_BODY_TAG_LEN: usize = 16;
pub const DATA_BODY_MIN_LEN: usize = DATA_BODY_NONCE_LEN + DATA_BODY_TAG_LEN;

/// Pack local `(nonce, ct‖tag)` into wire `nonce ‖ ct ‖ tag`.
pub fn encode_data_body(nonce: &[u8; DATA_BODY_NONCE_LEN], ct_and_tag: &[u8]) -> Result<Vec<u8>> {
    if ct_and_tag.len() < DATA_BODY_TAG_LEN {
        return Err(SyncError::InvalidEncoding(
            "ct‖tag shorter than tag length".to_string(),
        ));
    }
    let mut out = Vec::with_capacity(DATA_BODY_NONCE_LEN + ct_and_tag.len());
    out.extend_from_slice(nonce);
    out.extend_from_slice(ct_and_tag);
    Ok(out)
}

/// Unpack wire body into `(nonce, ct‖tag)`.
pub fn decode_data_body(blob: &[u8]) -> Result<([u8; DATA_BODY_NONCE_LEN], Vec<u8>)> {
    if blob.len() < DATA_BODY_MIN_LEN {
        return Err(SyncError::InvalidEncoding(format!(
            "data body length {} < min {}",
            blob.len(),
            DATA_BODY_MIN_LEN
        )));
    }
    let mut nonce = [0u8; DATA_BODY_NONCE_LEN];
    nonce.copy_from_slice(&blob[..DATA_BODY_NONCE_LEN]);
    let ct_and_tag = blob[DATA_BODY_NONCE_LEN..].to_vec();
    Ok((nonce, ct_and_tag))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use crate::device_keys::generate_device_keys;
    use crate::enrollment::REPLICATION_SCHEMA_VERSION;
    use uuid::Uuid;

    fn uuid_n(n: u8) -> Uuid {
        let mut b = [0u8; 16];
        b[15] = n;
        Uuid::from_bytes(b)
    }

    fn sample_control_outer(device_id: DeviceId) -> OuterEnvelope {
        OuterEnvelope {
            schema_version: REPLICATION_SCHEMA_VERSION,
            envelope_id: EnvelopeId::from_uuid(uuid_n(1)),
            device_id,
            local_seq: 1,
            content_type_code: ContentTypeCode::DeviceEnrolled,
            event_id: ReplicationEventId::from_uuid(uuid_n(3)),
            content_key_id: ContentKeyId::from_uuid(Uuid::nil()),
            ciphertext: b"enroll-payload".to_vec(),
            wrap_records: vec![],
        }
    }

    #[test]
    fn verify_envelope__metadata_swap__err() {
        let keys = generate_device_keys().expect("keys");
        let device = DeviceId::from_uuid(uuid_n(2));
        let outer = sample_control_outer(device);
        let mut signed = sign_envelope(&outer, &keys.signing_key()).expect("sign");
        // Meta-swap: change local_seq under same signature.
        signed.outer.local_seq = 999;
        let err = verify_envelope(&signed, &keys.verifying_key()).expect_err("must fail");
        assert!(matches!(err, SyncError::SignatureInvalid));
    }

    #[test]
    fn control_device_enrolled__wrap_count_zero__ok() {
        let keys = generate_device_keys().expect("keys");
        let device = DeviceId::from_uuid(uuid_n(2));
        let outer = sample_control_outer(device);
        assert!(outer.wrap_records.is_empty());
        let signed = sign_envelope(&outer, &keys.signing_key()).expect("sign");
        verify_envelope(&signed, &keys.verifying_key()).expect("verify");
        assert_eq!(
            signed.outer.content_type_code,
            ContentTypeCode::DeviceEnrolled
        );
    }

    #[test]
    fn data_body_pack__nonce_ct_tag__roundtrip() {
        let nonce = [0x11u8; 12];
        let ct_tag = vec![0x22; 32]; // 16 ct + 16 tag illustrative
        let blob = encode_data_body(&nonce, &ct_tag).expect("encode");
        assert_eq!(blob.len(), 12 + 32);
        assert_eq!(&blob[..12], &nonce);
        let (n2, ct2) = decode_data_body(&blob).expect("decode");
        assert_eq!(n2, nonce);
        assert_eq!(ct2, ct_tag);
    }

    #[test]
    fn content_type__unknown__reject() {
        let err = ContentTypeCode::from_u16(0x00FF).expect_err("unknown");
        assert!(matches!(err, SyncError::UnknownContentType(0x00FF)));
    }
}
