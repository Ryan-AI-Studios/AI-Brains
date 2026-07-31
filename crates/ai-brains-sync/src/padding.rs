//! Size-bucket padding helpers (L14 / R19).
//!
//! Buckets: 256, 4096, 65536. Best-effort only — **not** a metadata-privacy claim.

/// Canonical padding bucket sizes (ascending).
pub const PAD_BUCKETS: [usize; 3] = [256, 4096, 65536];

/// Map plaintext length to the smallest bucket that can hold it.
///
/// Lengths above the largest bucket return the largest bucket (caller may
/// leave unpadded or refuse — product policy). Zero length maps to 256.
pub fn pad_to_bucket(len: usize) -> usize {
    for &bucket in &PAD_BUCKETS {
        if len <= bucket {
            return bucket;
        }
    }
    PAD_BUCKETS[PAD_BUCKETS.len() - 1]
}

/// Pad `plaintext` with trailing zeros to the selected bucket (if larger).
/// Does not shrink; if already above largest bucket, returns clone as-is.
pub fn pad_plaintext(plaintext: &[u8]) -> Vec<u8> {
    let bucket = pad_to_bucket(plaintext.len());
    if plaintext.len() >= bucket {
        return plaintext.to_vec();
    }
    let mut out = plaintext.to_vec();
    out.resize(bucket, 0);
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn padding__len_to_bucket__256_4k_64k() {
        assert_eq!(pad_to_bucket(0), 256);
        assert_eq!(pad_to_bucket(1), 256);
        assert_eq!(pad_to_bucket(256), 256);
        assert_eq!(pad_to_bucket(257), 4096);
        assert_eq!(pad_to_bucket(4096), 4096);
        assert_eq!(pad_to_bucket(4097), 65536);
        assert_eq!(pad_to_bucket(65536), 65536);
        assert_eq!(pad_to_bucket(65537), 65536);
        let padded = pad_plaintext(&[1u8; 10]);
        assert_eq!(padded.len(), 256);
        assert_eq!(&padded[..10], &[1u8; 10]);
    }

    /// T178-L14-pad-buckets — PAD_BUCKETS membership.
    #[test]
    fn t178_l14_pad_buckets__membership() {
        // T178-L14-pad-buckets
        assert_eq!(PAD_BUCKETS, [256, 4096, 65536]);
        for &b in &PAD_BUCKETS {
            assert_eq!(pad_to_bucket(b), b);
        }
        assert!(PAD_BUCKETS.contains(&pad_to_bucket(0)));
        assert!(PAD_BUCKETS.contains(&pad_to_bucket(100)));
        assert!(PAD_BUCKETS.contains(&pad_to_bucket(1000)));
    }
}
