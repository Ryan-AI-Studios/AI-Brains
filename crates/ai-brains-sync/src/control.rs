//! Cleartext control payload encode/decode (ADR-0018 §5.1.1).
//!
//! Control envelopes always have `wrap_count = 0`. Bodies are membership /
//! integrity signaling under outer Ed25519 — not content secrecy.

use crate::enrollment::{ENROLLMENT_PACKAGE_LEN, enrollment_package};
use crate::envelope::ContentTypeCode;
use crate::error::{Result, SyncError};
use ai_brains_core::ids::{ContentKeyId, DeviceId, ReplicationEventId};

/// DeviceEnrolled control payload = enrollment package bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceEnrolledPayload {
    pub schema_version: u16,
    pub device_id: DeviceId,
    pub ed25519_pub: [u8; 32],
    pub x25519_pub: [u8; 32],
}

/// DeviceRevoked control payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRevokedPayload {
    pub device_id: DeviceId,
    pub reason_code: String,
}

/// ContentErasureTombstone (sole erasure code 0x0012).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentErasureTombstonePayload {
    pub content_key_id: ContentKeyId,
    pub reason_code: String,
}

/// ErasureAck — peer attestation (not wipe proof).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasureAckPayload {
    pub erasure_id: ReplicationEventId,
    pub content_key_id: ContentKeyId,
    pub peer_device_id: DeviceId,
    /// `"acked"` | `"failed"` status bytes (UTF-8 short token).
    pub status: String,
}

/// GapSkipAudit — operator skip of a missing seq (R13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapSkipAuditPayload {
    pub peer_device_id: DeviceId,
    pub skipped_seq: u64,
    pub reason: String,
}

/// Encode a control payload as cleartext body bytes.
pub fn encode_control_payload(kind: ContentTypeCode, payload: &ControlPayload) -> Result<Vec<u8>> {
    match (kind, payload) {
        (ContentTypeCode::DeviceEnrolled, ControlPayload::DeviceEnrolled(p)) => Ok(
            enrollment_package(&p.device_id, &p.ed25519_pub, &p.x25519_pub),
        ),
        (ContentTypeCode::DeviceRevoked, ControlPayload::DeviceRevoked(p)) => {
            encode_device_revoked(p)
        }
        (ContentTypeCode::ContentErasureTombstone, ControlPayload::ContentErasureTombstone(p)) => {
            encode_erasure_tombstone(p)
        }
        (ContentTypeCode::ErasureAck, ControlPayload::ErasureAck(p)) => encode_erasure_ack(p),
        (ContentTypeCode::GapSkipAudit, ControlPayload::GapSkipAudit(p)) => {
            encode_gap_skip_audit(p)
        }
        _ => Err(SyncError::InvalidEncoding(
            "control kind/payload mismatch".to_string(),
        )),
    }
}

/// Tagged control payload union.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlPayload {
    DeviceEnrolled(DeviceEnrolledPayload),
    DeviceRevoked(DeviceRevokedPayload),
    ContentErasureTombstone(ContentErasureTombstonePayload),
    ErasureAck(ErasureAckPayload),
    GapSkipAudit(GapSkipAuditPayload),
}

/// Decode cleartext control body by type code.
pub fn decode_control_payload(kind: ContentTypeCode, body: &[u8]) -> Result<ControlPayload> {
    match kind {
        ContentTypeCode::DeviceEnrolled => {
            let parsed = crate::enrollment::parse_enrollment_package(body)?;
            Ok(ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
                schema_version: parsed.schema_version,
                device_id: parsed.device_id,
                ed25519_pub: parsed.ed25519_pub,
                x25519_pub: parsed.x25519_pub,
            }))
        }
        ContentTypeCode::DeviceRevoked => {
            Ok(ControlPayload::DeviceRevoked(decode_device_revoked(body)?))
        }
        ContentTypeCode::ContentErasureTombstone => Ok(ControlPayload::ContentErasureTombstone(
            decode_erasure_tombstone(body)?,
        )),
        ContentTypeCode::ErasureAck => Ok(ControlPayload::ErasureAck(decode_erasure_ack(body)?)),
        ContentTypeCode::GapSkipAudit => {
            Ok(ControlPayload::GapSkipAudit(decode_gap_skip_audit(body)?))
        }
        ContentTypeCode::DataEvent => Err(SyncError::InvalidEncoding(
            "DataEvent is not a control payload".to_string(),
        )),
    }
}

fn encode_device_revoked(p: &DeviceRevokedPayload) -> Result<Vec<u8>> {
    let reason = p.reason_code.as_bytes();
    if reason.len() > u16::MAX as usize {
        return Err(SyncError::InvalidEncoding("reason too long".to_string()));
    }
    let mut out = Vec::with_capacity(16 + 2 + reason.len());
    out.extend_from_slice(p.device_id.as_uuid().as_bytes());
    out.extend_from_slice(&(reason.len() as u16).to_be_bytes());
    out.extend_from_slice(reason);
    Ok(out)
}

fn decode_device_revoked(body: &[u8]) -> Result<DeviceRevokedPayload> {
    if body.len() < 18 {
        return Err(SyncError::InvalidEncoding(
            "DeviceRevoked body too short".to_string(),
        ));
    }
    let uuid = uuid::Uuid::from_slice(&body[0..16])
        .map_err(|e| SyncError::InvalidEncoding(e.to_string()))?;
    let rlen = u16::from_be_bytes([body[16], body[17]]) as usize;
    if body.len() != 18 + rlen {
        return Err(SyncError::InvalidEncoding(
            "DeviceRevoked length mismatch".to_string(),
        ));
    }
    let reason_code = std::str::from_utf8(&body[18..])
        .map_err(|e| SyncError::InvalidEncoding(e.to_string()))?
        .to_string();
    Ok(DeviceRevokedPayload {
        device_id: DeviceId::from_uuid(uuid),
        reason_code,
    })
}

fn encode_erasure_tombstone(p: &ContentErasureTombstonePayload) -> Result<Vec<u8>> {
    let reason = p.reason_code.as_bytes();
    if reason.len() > u16::MAX as usize {
        return Err(SyncError::InvalidEncoding("reason too long".to_string()));
    }
    let mut out = Vec::with_capacity(16 + 2 + reason.len());
    out.extend_from_slice(p.content_key_id.as_uuid().as_bytes());
    out.extend_from_slice(&(reason.len() as u16).to_be_bytes());
    out.extend_from_slice(reason);
    Ok(out)
}

fn decode_erasure_tombstone(body: &[u8]) -> Result<ContentErasureTombstonePayload> {
    if body.len() < 18 {
        return Err(SyncError::InvalidEncoding(
            "ContentErasureTombstone body too short".to_string(),
        ));
    }
    let uuid = uuid::Uuid::from_slice(&body[0..16])
        .map_err(|e| SyncError::InvalidEncoding(e.to_string()))?;
    let rlen = u16::from_be_bytes([body[16], body[17]]) as usize;
    if body.len() != 18 + rlen {
        return Err(SyncError::InvalidEncoding(
            "ContentErasureTombstone length mismatch".to_string(),
        ));
    }
    let reason_code = std::str::from_utf8(&body[18..])
        .map_err(|e| SyncError::InvalidEncoding(e.to_string()))?
        .to_string();
    Ok(ContentErasureTombstonePayload {
        content_key_id: ContentKeyId::from_uuid(uuid),
        reason_code,
    })
}

fn encode_erasure_ack(p: &ErasureAckPayload) -> Result<Vec<u8>> {
    let status = p.status.as_bytes();
    if status.len() > u16::MAX as usize {
        return Err(SyncError::InvalidEncoding("status too long".to_string()));
    }
    let mut out = Vec::with_capacity(16 + 16 + 16 + 2 + status.len());
    out.extend_from_slice(p.erasure_id.as_uuid().as_bytes());
    out.extend_from_slice(p.content_key_id.as_uuid().as_bytes());
    out.extend_from_slice(p.peer_device_id.as_uuid().as_bytes());
    out.extend_from_slice(&(status.len() as u16).to_be_bytes());
    out.extend_from_slice(status);
    Ok(out)
}

fn decode_erasure_ack(body: &[u8]) -> Result<ErasureAckPayload> {
    if body.len() < 50 {
        return Err(SyncError::InvalidEncoding(
            "ErasureAck body too short".to_string(),
        ));
    }
    let erasure = uuid::Uuid::from_slice(&body[0..16])
        .map_err(|e| SyncError::InvalidEncoding(e.to_string()))?;
    let content = uuid::Uuid::from_slice(&body[16..32])
        .map_err(|e| SyncError::InvalidEncoding(e.to_string()))?;
    let peer = uuid::Uuid::from_slice(&body[32..48])
        .map_err(|e| SyncError::InvalidEncoding(e.to_string()))?;
    let slen = u16::from_be_bytes([body[48], body[49]]) as usize;
    if body.len() != 50 + slen {
        return Err(SyncError::InvalidEncoding(
            "ErasureAck length mismatch".to_string(),
        ));
    }
    let status = std::str::from_utf8(&body[50..])
        .map_err(|e| SyncError::InvalidEncoding(e.to_string()))?
        .to_string();
    Ok(ErasureAckPayload {
        erasure_id: ReplicationEventId::from_uuid(erasure),
        content_key_id: ContentKeyId::from_uuid(content),
        peer_device_id: DeviceId::from_uuid(peer),
        status,
    })
}

fn encode_gap_skip_audit(p: &GapSkipAuditPayload) -> Result<Vec<u8>> {
    let reason = p.reason.as_bytes();
    if reason.len() > u16::MAX as usize {
        return Err(SyncError::InvalidEncoding("reason too long".to_string()));
    }
    let mut out = Vec::with_capacity(16 + 8 + 2 + reason.len());
    out.extend_from_slice(p.peer_device_id.as_uuid().as_bytes());
    out.extend_from_slice(&p.skipped_seq.to_be_bytes());
    out.extend_from_slice(&(reason.len() as u16).to_be_bytes());
    out.extend_from_slice(reason);
    Ok(out)
}

fn decode_gap_skip_audit(body: &[u8]) -> Result<GapSkipAuditPayload> {
    if body.len() < 26 {
        return Err(SyncError::InvalidEncoding(
            "GapSkipAudit body too short".to_string(),
        ));
    }
    let peer = uuid::Uuid::from_slice(&body[0..16])
        .map_err(|e| SyncError::InvalidEncoding(e.to_string()))?;
    let mut seq_bytes = [0u8; 8];
    seq_bytes.copy_from_slice(&body[16..24]);
    let skipped_seq = u64::from_be_bytes(seq_bytes);
    let rlen = u16::from_be_bytes([body[24], body[25]]) as usize;
    if body.len() != 26 + rlen {
        return Err(SyncError::InvalidEncoding(
            "GapSkipAudit length mismatch".to_string(),
        ));
    }
    let reason = std::str::from_utf8(&body[26..])
        .map_err(|e| SyncError::InvalidEncoding(e.to_string()))?
        .to_string();
    Ok(GapSkipAuditPayload {
        peer_device_id: DeviceId::from_uuid(peer),
        skipped_seq,
        reason,
    })
}

/// Document fixed enrollment package size for control DeviceEnrolled.
pub const DEVICE_ENROLLED_BODY_LEN: usize = ENROLLMENT_PACKAGE_LEN;

/// Built and signed control envelope ready for local persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedControlEnvelope {
    pub signed: crate::envelope::SignedEnvelope,
    /// Cleartext control body (also present as `signed.outer.ciphertext`).
    pub body: Vec<u8>,
}

/// Build cleartext control payload, wrap as outer envelope (wrap_count=0,
/// content_key_id = nil UUID except caller may override), and Ed25519-sign.
///
/// Used by bootstrap / enroll / revoke local control persistence (T176).
pub fn build_and_sign_control(
    kind: ContentTypeCode,
    payload: &ControlPayload,
    sender_device_id: DeviceId,
    local_seq: u64,
    signing_key: &ed25519_dalek::SigningKey,
) -> Result<SignedControlEnvelope> {
    if !kind.is_control() {
        return Err(SyncError::InvalidEncoding(
            "build_and_sign_control requires a control content type".to_string(),
        ));
    }
    let body = encode_control_payload(kind, payload)?;
    let outer = crate::envelope::OuterEnvelope {
        schema_version: crate::enrollment::REPLICATION_SCHEMA_VERSION,
        envelope_id: crate::signed_bytes::EnvelopeId::new(),
        device_id: sender_device_id,
        local_seq,
        content_type_code: kind,
        event_id: ReplicationEventId::new(),
        content_key_id: ContentKeyId::from_uuid(uuid::Uuid::nil()),
        ciphertext: body.clone(),
        wrap_records: vec![],
    };
    let signed = crate::envelope::sign_envelope(&outer, signing_key)?;
    Ok(SignedControlEnvelope { signed, body })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use crate::device_keys::generate_device_keys;
    use crate::enrollment::REPLICATION_SCHEMA_VERSION;
    use crate::envelope::{OuterEnvelope, sign_envelope, verify_envelope};
    use crate::signed_bytes::EnvelopeId;
    use uuid::Uuid;

    #[test]
    fn control_device_enrolled__wrap_count_zero__ok() {
        let keys = generate_device_keys().expect("keys");
        let device = DeviceId::from_uuid(Uuid::new_v4());
        let payload = DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: device,
            ed25519_pub: keys.verifying_key().to_bytes(),
            x25519_pub: keys.x25519_public().to_bytes(),
        };
        let body = encode_control_payload(
            ContentTypeCode::DeviceEnrolled,
            &ControlPayload::DeviceEnrolled(payload.clone()),
        )
        .expect("encode");
        assert_eq!(body.len(), DEVICE_ENROLLED_BODY_LEN);
        let decoded = decode_control_payload(ContentTypeCode::DeviceEnrolled, &body).expect("dec");
        match decoded {
            ControlPayload::DeviceEnrolled(p) => {
                assert_eq!(p.device_id, device);
                assert_eq!(p.ed25519_pub, payload.ed25519_pub);
            }
            _ => panic!("wrong variant"),
        }
        let outer = OuterEnvelope {
            schema_version: REPLICATION_SCHEMA_VERSION,
            envelope_id: EnvelopeId::new(),
            device_id: device,
            local_seq: 1,
            content_type_code: ContentTypeCode::DeviceEnrolled,
            event_id: ReplicationEventId::from_uuid(Uuid::new_v4()),
            content_key_id: ContentKeyId::from_uuid(Uuid::nil()),
            ciphertext: body,
            wrap_records: vec![],
        };
        let signed = sign_envelope(&outer, &keys.signing_key()).expect("sign");
        assert!(signed.outer.wrap_records.is_empty());
        verify_envelope(&signed, &keys.verifying_key()).expect("verify");
    }

    #[test]
    fn build_and_sign_control__device_enrolled__verifiable() {
        let keys = generate_device_keys().expect("keys");
        let device = DeviceId::from_uuid(Uuid::new_v4());
        let payload = DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: device,
            ed25519_pub: keys.verifying_key().to_bytes(),
            x25519_pub: keys.x25519_public().to_bytes(),
        };
        let built = build_and_sign_control(
            ContentTypeCode::DeviceEnrolled,
            &ControlPayload::DeviceEnrolled(payload),
            device,
            1,
            &keys.signing_key(),
        )
        .expect("sign control");
        assert!(built.signed.outer.wrap_records.is_empty());
        assert_eq!(built.signed.outer.content_key_id.as_uuid(), Uuid::nil());
        verify_envelope(&built.signed, &keys.verifying_key()).expect("verify");
        // Meta-swap must fail.
        let mut swapped = built.signed.clone();
        swapped.outer.local_seq = 999;
        let err = verify_envelope(&swapped, &keys.verifying_key()).expect_err("meta-swap");
        assert!(matches!(err, SyncError::SignatureInvalid));
    }
}
