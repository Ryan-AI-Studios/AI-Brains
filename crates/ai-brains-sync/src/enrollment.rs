//! Enrollment package + dual-key fingerprint (ADR-0018 L3 / R7).
//!
//! ```text
//! enrollment_package =
//!     schema_version u16 BE
//!   ‖ DeviceId 16
//!   ‖ Ed25519_pub 32
//!   ‖ X25519_pub 32
//! fingerprint = SHA-256(package)
//! ```

use crate::error::{Result, SyncError};
use ai_brains_core::ids::DeviceId;
use sha2::{Digest, Sha256};

/// Wire / enrollment schema version (R / §6.1).
pub const REPLICATION_SCHEMA_VERSION: u16 = 1;

/// Fixed enrollment package length: 2 + 16 + 32 + 32.
pub const ENROLLMENT_PACKAGE_LEN: usize = 2 + 16 + 32 + 32;

/// Build the canonical enrollment package bytes.
pub fn enrollment_package(
    device_id: &DeviceId,
    ed25519_pub: &[u8; 32],
    x25519_pub: &[u8; 32],
) -> Vec<u8> {
    enrollment_package_with_version(
        REPLICATION_SCHEMA_VERSION,
        device_id,
        ed25519_pub,
        x25519_pub,
    )
}

/// Build enrollment package with an explicit schema version (tests / future).
pub fn enrollment_package_with_version(
    schema_version: u16,
    device_id: &DeviceId,
    ed25519_pub: &[u8; 32],
    x25519_pub: &[u8; 32],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(ENROLLMENT_PACKAGE_LEN);
    out.extend_from_slice(&schema_version.to_be_bytes());
    out.extend_from_slice(device_id.as_uuid().as_bytes());
    out.extend_from_slice(ed25519_pub);
    out.extend_from_slice(x25519_pub);
    out
}

/// SHA-256 of the full dual-key enrollment package (not Ed25519 alone).
pub fn fingerprint_sha256(package: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(package);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

/// Parsed enrollment package fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEnrollmentPackage {
    pub schema_version: u16,
    pub device_id: DeviceId,
    pub ed25519_pub: [u8; 32],
    pub x25519_pub: [u8; 32],
}

/// Parse a fixed-length enrollment package. Wrong length → error.
pub fn parse_enrollment_package(bytes: &[u8]) -> Result<ParsedEnrollmentPackage> {
    if bytes.len() != ENROLLMENT_PACKAGE_LEN {
        return Err(SyncError::InvalidEncoding(format!(
            "enrollment package length {} != {}",
            bytes.len(),
            ENROLLMENT_PACKAGE_LEN
        )));
    }
    let schema_version = u16::from_be_bytes([bytes[0], bytes[1]]);
    let uuid = uuid::Uuid::from_slice(&bytes[2..18])
        .map_err(|e| SyncError::InvalidEncoding(format!("device_id: {e}")))?;
    let mut ed25519_pub = [0u8; 32];
    ed25519_pub.copy_from_slice(&bytes[18..50]);
    let mut x25519_pub = [0u8; 32];
    x25519_pub.copy_from_slice(&bytes[50..82]);
    Ok(ParsedEnrollmentPackage {
        schema_version,
        device_id: DeviceId::from_uuid(uuid),
        ed25519_pub,
        x25519_pub,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use crate::device_keys::generate_device_keys;
    use uuid::Uuid;

    fn fixed_device() -> DeviceId {
        DeviceId::from_uuid(Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("uuid"))
    }

    #[test]
    fn enrollment_package__dual_keys__fingerprint_stable() {
        let keys = generate_device_keys().expect("keys");
        let ed_pub = keys.verifying_key().to_bytes();
        let x_pub = keys.x25519_public().to_bytes();
        let device = fixed_device();
        let pkg = enrollment_package(&device, &ed_pub, &x_pub);
        assert_eq!(pkg.len(), ENROLLMENT_PACKAGE_LEN);
        assert_eq!(&pkg[0..2], &REPLICATION_SCHEMA_VERSION.to_be_bytes());
        let fp1 = fingerprint_sha256(&pkg);
        let fp2 = fingerprint_sha256(&pkg);
        assert_eq!(fp1, fp2);
        // Changing X25519 alone changes fingerprint (dual-key bind).
        let mut x_flip = x_pub;
        x_flip[0] ^= 0x01;
        let pkg2 = enrollment_package(&device, &ed_pub, &x_flip);
        assert_ne!(fingerprint_sha256(&pkg2), fp1);
        let parsed = parse_enrollment_package(&pkg).expect("parse");
        assert_eq!(parsed.device_id, device);
        assert_eq!(parsed.ed25519_pub, ed_pub);
        assert_eq!(parsed.x25519_pub, x_pub);
    }
}
