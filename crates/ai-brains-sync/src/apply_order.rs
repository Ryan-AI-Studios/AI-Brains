//! Pre-decryption envelope stream order (R30).
//!
//! Sort key: `(device_id, local_seq, event_id)` only.
//! Domain topology (parent / correlation) is store/projector **after** DEK open
//! — not in `ai-brains-sync`.

use ai_brains_core::ids::{DeviceId, ReplicationEventId};

/// Minimal apply-order key for a replication envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplyOrderKey {
    pub device_id: DeviceId,
    pub local_seq: u64,
    pub event_id: ReplicationEventId,
}

impl PartialOrd for ApplyOrderKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ApplyOrderKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.device_id
            .as_uuid()
            .as_bytes()
            .cmp(other.device_id.as_uuid().as_bytes())
            .then_with(|| self.local_seq.cmp(&other.local_seq))
            .then_with(|| {
                self.event_id
                    .as_uuid()
                    .as_bytes()
                    .cmp(other.event_id.as_uuid().as_bytes())
            })
    }
}

/// Sort a slice of items by apply-order key (stable for equal keys via sort_by).
pub fn sort_by_apply_order<T, F>(items: &mut [T], key_fn: F)
where
    F: Fn(&T) -> ApplyOrderKey,
{
    items.sort_by_key(key_fn);
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

    #[test]
    fn apply_order__tie_break__device_seq_event() {
        let d1 = DeviceId::from_uuid(uuid_n(1));
        let d2 = DeviceId::from_uuid(uuid_n(2));
        let mut items = vec![
            ApplyOrderKey {
                device_id: d2,
                local_seq: 1,
                event_id: ReplicationEventId::from_uuid(uuid_n(9)),
            },
            ApplyOrderKey {
                device_id: d1,
                local_seq: 2,
                event_id: ReplicationEventId::from_uuid(uuid_n(1)),
            },
            ApplyOrderKey {
                device_id: d1,
                local_seq: 1,
                event_id: ReplicationEventId::from_uuid(uuid_n(5)),
            },
            ApplyOrderKey {
                device_id: d1,
                local_seq: 1,
                event_id: ReplicationEventId::from_uuid(uuid_n(3)),
            },
        ];
        sort_by_apply_order(&mut items, |k| *k);
        // device1 seq1 event3, device1 seq1 event5, device1 seq2, device2 seq1
        assert_eq!(items[0].device_id, d1);
        assert_eq!(items[0].local_seq, 1);
        assert_eq!(items[0].event_id.as_uuid(), uuid_n(3));
        assert_eq!(items[1].event_id.as_uuid(), uuid_n(5));
        assert_eq!(items[2].local_seq, 2);
        assert_eq!(items[3].device_id, d2);
    }
}
