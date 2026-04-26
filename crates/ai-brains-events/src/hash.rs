use crate::errors::EventError;
use crate::payload::Payload;
use sha2::{Digest, Sha256};

pub fn compute_payload_hash(payload: &Payload) -> Result<String, EventError> {
    // To ensure stable hashing, we convert to a JSON Value first.
    // serde_json::Value uses a BTreeMap for objects by default (unless preserve_order feature is enabled),
    // which ensures keys are sorted alphabetically during serialization.
    let value = serde_json::to_value(payload)?;
    let canonical_json = serde_json::to_string(&value)?;

    let mut hasher = Sha256::new();
    hasher.update(canonical_json.as_bytes());
    let result = hasher.finalize();

    Ok(hex::encode(result))
}

pub fn verify_payload_hash(payload: &Payload, expected_hash: &str) -> Result<(), EventError> {
    let actual_hash = compute_payload_hash(payload)?;
    if actual_hash == expected_hash {
        Ok(())
    } else {
        Err(EventError::HashMismatch {
            expected: expected_hash.to_string(),
            found: actual_hash,
        })
    }
}
