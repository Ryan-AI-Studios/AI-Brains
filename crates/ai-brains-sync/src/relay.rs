//! Fake relay ports for multi-device sync (T177 F1–F3, F16–F17).
//!
//! **No production network.** In-memory and file-backed fakes only.
//! [`AdversarialRelay`] decorates any [`RelayPort`] for T178 security tests.

use crate::error::{Result, SyncError};
use crate::wire::WIRE_MAX_SIZE;
use ai_brains_core::ids::DeviceId;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

/// Marker file written by [`FileFakeRelay::open_or_create`].
pub const FAKE_RELAY_MARKER: &str = ".aibrains_fake_relay_marker";

/// Opaque transport unit. `body` = full `wire_v1` of a [`crate::envelope::SignedEnvelope`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayBlob {
    pub envelope_id: Uuid,
    pub sender_device_id: DeviceId,
    pub local_seq: u64,
    pub content_type_code: u16,
    pub body: Vec<u8>,
}

/// Interior-mutability friendly relay port (all methods take `&self`).
pub trait RelayPort: Send + Sync {
    fn put(&self, blob: &RelayBlob) -> Result<()>;

    /// Blobs with `local_seq > after_seq`, ascending, up to `limit`.
    fn pull(
        &self,
        sender_device_id: &DeviceId,
        after_seq: u64,
        limit: usize,
    ) -> Result<Vec<RelayBlob>>;

    /// Inclusive range `[from_seq, to_seq]` for gap fill.
    fn pull_range(
        &self,
        sender_device_id: &DeviceId,
        from_seq: u64,
        to_seq: u64,
    ) -> Result<Vec<RelayBlob>>;
}

// ---------------------------------------------------------------------------
// MemoryFakeRelay
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct MemoryInner {
    /// `(device_uuid, local_seq) → blob`
    by_seq: BTreeMap<(Uuid, u64), RelayBlob>,
    /// `envelope_id → (device_uuid, local_seq)` for idempotent put
    by_envelope: HashMap<Uuid, (Uuid, u64)>,
}

/// In-memory fake relay with `Mutex` interior mutability (share via `Arc`).
#[derive(Debug, Default)]
pub struct MemoryFakeRelay {
    inner: Mutex<MemoryInner>,
}

impl MemoryFakeRelay {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MemoryInner::default()),
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, MemoryInner>> {
        self.inner
            .lock()
            .map_err(|_| SyncError::RelayConflict("memory relay lock poisoned".to_string()))
    }
}

impl RelayPort for MemoryFakeRelay {
    fn put(&self, blob: &RelayBlob) -> Result<()> {
        if blob.body.len() > WIRE_MAX_SIZE {
            return Err(SyncError::RelayBlobTooLarge);
        }
        let mut guard = self.lock()?;
        let device_uuid = blob.sender_device_id.as_uuid();

        if let Some(&(prev_dev, prev_seq)) = guard.by_envelope.get(&blob.envelope_id) {
            // Idempotent re-put of the same envelope_id.
            if prev_dev == device_uuid && prev_seq == blob.local_seq {
                return Ok(());
            }
            return Err(SyncError::RelayConflict(format!(
                "envelope_id {} already stored at different (device, seq)",
                blob.envelope_id
            )));
        }

        let key = (device_uuid, blob.local_seq);
        if let Some(existing) = guard.by_seq.get(&key)
            && existing.envelope_id != blob.envelope_id
        {
            return Err(SyncError::SeqCollision);
        }

        guard.by_envelope.insert(blob.envelope_id, key);
        guard.by_seq.insert(key, blob.clone());
        Ok(())
    }

    fn pull(
        &self,
        sender_device_id: &DeviceId,
        after_seq: u64,
        limit: usize,
    ) -> Result<Vec<RelayBlob>> {
        let guard = self.lock()?;
        let device_uuid = sender_device_id.as_uuid();
        let mut out = Vec::new();
        for ((dev, seq), blob) in guard.by_seq.iter() {
            if *dev != device_uuid {
                continue;
            }
            if *seq <= after_seq {
                continue;
            }
            out.push(blob.clone());
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }

    fn pull_range(
        &self,
        sender_device_id: &DeviceId,
        from_seq: u64,
        to_seq: u64,
    ) -> Result<Vec<RelayBlob>> {
        let guard = self.lock()?;
        let device_uuid = sender_device_id.as_uuid();
        let mut out = Vec::new();
        for ((dev, seq), blob) in guard.by_seq.iter() {
            if *dev != device_uuid {
                continue;
            }
            if *seq < from_seq || *seq > to_seq {
                continue;
            }
            out.push(blob.clone());
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// FileFakeRelay
// ---------------------------------------------------------------------------

/// File-backed fake relay: `relay_root/<device_id>/<seq>.blob` + `.meta`.
///
/// Meta format (no extra deps): `{envelope_id_uuid}\n{content_type_code}\n`
#[derive(Debug)]
pub struct FileFakeRelay {
    root: PathBuf,
    /// Serialize all file ops (tests may share via Arc).
    lock: Mutex<()>,
}

impl FileFakeRelay {
    /// Open an existing fake-relay tree or create a new one with marker.
    ///
    /// Refuses filesystem roots and existing non-relay directories without
    /// [`.aibrains_fake_relay_marker`](FAKE_RELAY_MARKER).
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        if is_filesystem_root(&root) {
            return Err(SyncError::RelayPathRefused(
                "refusing filesystem root as fake relay path".to_string(),
            ));
        }

        let marker = root.join(FAKE_RELAY_MARKER);
        if root.exists() {
            if !marker.exists() {
                // Empty dir is claimable; non-empty without marker is refused.
                let is_empty = fs::read_dir(&root)
                    .map_err(|e| SyncError::RelayIo(e.to_string()))?
                    .next()
                    .is_none();
                if !is_empty {
                    return Err(SyncError::RelayPathRefused(
                        "path exists without .aibrains_fake_relay_marker".to_string(),
                    ));
                }
            }
        } else {
            fs::create_dir_all(&root).map_err(|e| SyncError::RelayIo(e.to_string()))?;
        }

        if !marker.exists() {
            fs::write(&marker, b"ai-brains fake relay v1\n")
                .map_err(|e| SyncError::RelayIo(e.to_string()))?;
        }

        Ok(Self {
            root,
            lock: Mutex::new(()),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ()>> {
        self.lock
            .lock()
            .map_err(|_| SyncError::RelayConflict("file relay lock poisoned".to_string()))
    }

    fn device_dir(&self, device_id: &DeviceId) -> PathBuf {
        self.root.join(device_id.as_uuid().to_string())
    }

    fn blob_paths(&self, device_id: &DeviceId, seq: u64) -> (PathBuf, PathBuf) {
        let dir = self.device_dir(device_id);
        (
            dir.join(format!("{seq}.blob")),
            dir.join(format!("{seq}.meta")),
        )
    }

    fn encode_meta(envelope_id: Uuid, content_type_code: u16) -> String {
        format!("{envelope_id}\n{content_type_code}\n")
    }

    fn decode_meta(raw: &str) -> Result<(Uuid, u16)> {
        let mut lines = raw.lines();
        let id_line = lines
            .next()
            .ok_or_else(|| SyncError::RelayIo("meta missing envelope_id line".to_string()))?;
        let code_line = lines
            .next()
            .ok_or_else(|| SyncError::RelayIo("meta missing content_type_code line".to_string()))?;
        let envelope_id = Uuid::parse_str(id_line.trim())
            .map_err(|e| SyncError::RelayIo(format!("meta envelope_id: {e}")))?;
        let content_type_code: u16 = code_line
            .trim()
            .parse()
            .map_err(|e| SyncError::RelayIo(format!("meta content_type_code: {e}")))?;
        Ok((envelope_id, content_type_code))
    }

    fn read_blob(&self, device_id: &DeviceId, seq: u64) -> Result<Option<RelayBlob>> {
        let (blob_path, meta_path) = self.blob_paths(device_id, seq);
        if !blob_path.exists() {
            return Ok(None);
        }
        let body = fs::read(&blob_path).map_err(|e| SyncError::RelayIo(e.to_string()))?;
        let meta_raw =
            fs::read_to_string(&meta_path).map_err(|e| SyncError::RelayIo(e.to_string()))?;
        let (envelope_id, content_type_code) = Self::decode_meta(&meta_raw)?;
        Ok(Some(RelayBlob {
            envelope_id,
            sender_device_id: *device_id,
            local_seq: seq,
            content_type_code,
            body,
        }))
    }

    fn list_seqs(&self, device_id: &DeviceId) -> Result<Vec<u64>> {
        let dir = self.device_dir(device_id);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut seqs = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| SyncError::RelayIo(e.to_string()))? {
            let entry = entry.map_err(|e| SyncError::RelayIo(e.to_string()))?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(stem) = name.strip_suffix(".blob")
                && let Ok(seq) = stem.parse::<u64>()
            {
                seqs.push(seq);
            }
        }
        seqs.sort_unstable();
        Ok(seqs)
    }

    /// Scan all devices for envelope_id (idempotency check).
    fn find_envelope(&self, envelope_id: Uuid) -> Result<Option<(DeviceId, u64)>> {
        if !self.root.exists() {
            return Ok(None);
        }
        for entry in fs::read_dir(&self.root).map_err(|e| SyncError::RelayIo(e.to_string()))? {
            let entry = entry.map_err(|e| SyncError::RelayIo(e.to_string()))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let Ok(uuid) = Uuid::parse_str(&name) else {
                continue;
            };
            let device = DeviceId::from_uuid(uuid);
            for seq in self.list_seqs(&device)? {
                if let Some(blob) = self.read_blob(&device, seq)?
                    && blob.envelope_id == envelope_id
                {
                    return Ok(Some((device, seq)));
                }
            }
        }
        Ok(None)
    }
}

/// True if `path` is a filesystem root (`/`, `C:\`, …) with no normal component.
fn is_filesystem_root(path: &Path) -> bool {
    !path.components().any(|c| matches!(c, Component::Normal(_)))
}

impl RelayPort for FileFakeRelay {
    fn put(&self, blob: &RelayBlob) -> Result<()> {
        if blob.body.len() > WIRE_MAX_SIZE {
            return Err(SyncError::RelayBlobTooLarge);
        }
        let _guard = self.lock()?;

        if let Some((prev_dev, prev_seq)) = self.find_envelope(blob.envelope_id)? {
            if prev_dev.as_uuid() == blob.sender_device_id.as_uuid() && prev_seq == blob.local_seq {
                return Ok(());
            }
            return Err(SyncError::RelayConflict(format!(
                "envelope_id {} already stored at different (device, seq)",
                blob.envelope_id
            )));
        }

        if let Some(existing) = self.read_blob(&blob.sender_device_id, blob.local_seq)?
            && existing.envelope_id != blob.envelope_id
        {
            return Err(SyncError::SeqCollision);
        }

        let dir = self.device_dir(&blob.sender_device_id);
        fs::create_dir_all(&dir).map_err(|e| SyncError::RelayIo(e.to_string()))?;
        let (blob_path, meta_path) = self.blob_paths(&blob.sender_device_id, blob.local_seq);
        let meta = Self::encode_meta(blob.envelope_id, blob.content_type_code);
        fs::write(&blob_path, &blob.body).map_err(|e| SyncError::RelayIo(e.to_string()))?;
        fs::write(&meta_path, meta).map_err(|e| SyncError::RelayIo(e.to_string()))?;
        Ok(())
    }

    fn pull(
        &self,
        sender_device_id: &DeviceId,
        after_seq: u64,
        limit: usize,
    ) -> Result<Vec<RelayBlob>> {
        let _guard = self.lock()?;
        let mut out = Vec::new();
        for seq in self.list_seqs(sender_device_id)? {
            if seq <= after_seq {
                continue;
            }
            if let Some(blob) = self.read_blob(sender_device_id, seq)? {
                out.push(blob);
                if out.len() >= limit {
                    break;
                }
            }
        }
        Ok(out)
    }

    fn pull_range(
        &self,
        sender_device_id: &DeviceId,
        from_seq: u64,
        to_seq: u64,
    ) -> Result<Vec<RelayBlob>> {
        let _guard = self.lock()?;
        let mut out = Vec::new();
        for seq in self.list_seqs(sender_device_id)? {
            if seq < from_seq || seq > to_seq {
                continue;
            }
            if let Some(blob) = self.read_blob(sender_device_id, seq)? {
                out.push(blob);
            }
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// AdversarialRelay
// ---------------------------------------------------------------------------

/// Decorator over any [`RelayPort`] with adversarial pull knobs (T178).
///
/// **C5 semantics:** delayed sequences are **hidden** (not deleted). Restore via
/// [`Self::clear_delay`] or by the sender re-pushing (put remains available on
/// the inner relay).
#[derive(Debug)]
pub struct AdversarialRelay<R: RelayPort> {
    inner: R,
    /// `(device_uuid, seq)` pairs hidden from pull / pull_range.
    delayed: Mutex<HashSet<(Uuid, u64)>>,
    /// Permanently filtered from pull views (still present on inner).
    dropped: Mutex<HashSet<(Uuid, u64)>>,
    reorder: Mutex<bool>,
    duplicate: Mutex<bool>,
}

impl<R: RelayPort> AdversarialRelay<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            delayed: Mutex::new(HashSet::new()),
            dropped: Mutex::new(HashSet::new()),
            reorder: Mutex::new(false),
            duplicate: Mutex::new(false),
        }
    }

    pub fn inner(&self) -> &R {
        &self.inner
    }

    /// Hide `local_seq` for `sender` from pulls until [`Self::clear_delay`].
    pub fn delay_seq(&self, sender: &DeviceId, seq: u64) -> Result<()> {
        self.delayed
            .lock()
            .map_err(|_| SyncError::RelayConflict("adversary lock poisoned".to_string()))?
            .insert((sender.as_uuid(), seq));
        Ok(())
    }

    pub fn clear_delay(&self, sender: &DeviceId, seq: u64) -> Result<()> {
        self.delayed
            .lock()
            .map_err(|_| SyncError::RelayConflict("adversary lock poisoned".to_string()))?
            .remove(&(sender.as_uuid(), seq));
        Ok(())
    }

    pub fn clear_all_delays(&self) -> Result<()> {
        self.delayed
            .lock()
            .map_err(|_| SyncError::RelayConflict("adversary lock poisoned".to_string()))?
            .clear();
        Ok(())
    }

    /// Hide seq permanently from pull views (inner store unchanged).
    pub fn drop_seq(&self, sender: &DeviceId, seq: u64) -> Result<()> {
        self.dropped
            .lock()
            .map_err(|_| SyncError::RelayConflict("adversary lock poisoned".to_string()))?
            .insert((sender.as_uuid(), seq));
        Ok(())
    }

    pub fn set_reorder(&self, enabled: bool) -> Result<()> {
        *self
            .reorder
            .lock()
            .map_err(|_| SyncError::RelayConflict("adversary lock poisoned".to_string()))? =
            enabled;
        Ok(())
    }

    pub fn set_duplicate(&self, enabled: bool) -> Result<()> {
        *self
            .duplicate
            .lock()
            .map_err(|_| SyncError::RelayConflict("adversary lock poisoned".to_string()))? =
            enabled;
        Ok(())
    }

    fn apply_adversary(&self, mut blobs: Vec<RelayBlob>) -> Result<Vec<RelayBlob>> {
        let delayed = self
            .delayed
            .lock()
            .map_err(|_| SyncError::RelayConflict("adversary lock poisoned".to_string()))?;
        let dropped = self
            .dropped
            .lock()
            .map_err(|_| SyncError::RelayConflict("adversary lock poisoned".to_string()))?;
        blobs.retain(|b| {
            let key = (b.sender_device_id.as_uuid(), b.local_seq);
            !delayed.contains(&key) && !dropped.contains(&key)
        });

        let reorder = *self
            .reorder
            .lock()
            .map_err(|_| SyncError::RelayConflict("adversary lock poisoned".to_string()))?;
        if reorder {
            blobs.reverse();
        }

        let duplicate = *self
            .duplicate
            .lock()
            .map_err(|_| SyncError::RelayConflict("adversary lock poisoned".to_string()))?;
        if duplicate {
            let mut out = Vec::with_capacity(blobs.len() * 2);
            for b in blobs {
                out.push(b.clone());
                out.push(b);
            }
            return Ok(out);
        }
        Ok(blobs)
    }
}

impl<R: RelayPort> RelayPort for AdversarialRelay<R> {
    fn put(&self, blob: &RelayBlob) -> Result<()> {
        // Puts always reach the inner relay (C5: delay is pull-side only).
        self.inner.put(blob)
    }

    fn pull(
        &self,
        sender_device_id: &DeviceId,
        after_seq: u64,
        limit: usize,
    ) -> Result<Vec<RelayBlob>> {
        // Over-fetch then filter so limit applies to visible blobs.
        let raw = self.inner.pull(sender_device_id, after_seq, usize::MAX)?;
        let mut filtered = self.apply_adversary(raw)?;
        if filtered.len() > limit {
            filtered.truncate(limit);
        }
        Ok(filtered)
    }

    fn pull_range(
        &self,
        sender_device_id: &DeviceId,
        from_seq: u64,
        to_seq: u64,
    ) -> Result<Vec<RelayBlob>> {
        let raw = self.inner.pull_range(sender_device_id, from_seq, to_seq)?;
        self.apply_adversary(raw)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use crate::wire::WIRE_MAX_SIZE;
    use std::sync::Arc;
    use uuid::Uuid;

    fn uuid_n(n: u8) -> Uuid {
        let mut b = [0u8; 16];
        b[15] = n;
        Uuid::from_bytes(b)
    }

    fn device(n: u8) -> DeviceId {
        DeviceId::from_uuid(uuid_n(n))
    }

    fn blob(device_n: u8, seq: u64, env_n: u8) -> RelayBlob {
        RelayBlob {
            envelope_id: uuid_n(env_n),
            sender_device_id: device(device_n),
            local_seq: seq,
            content_type_code: 0x0010,
            body: vec![env_n, seq as u8],
        }
    }

    #[test]
    fn memory_relay__put_pull__roundtrip() {
        let relay = MemoryFakeRelay::new();
        let d = device(1);
        relay.put(&blob(1, 1, 10)).expect("put 1");
        relay.put(&blob(1, 2, 11)).expect("put 2");
        relay.put(&blob(1, 3, 12)).expect("put 3");

        let all = relay.pull(&d, 0, 10).expect("pull");
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].local_seq, 1);
        assert_eq!(all[1].local_seq, 2);
        assert_eq!(all[2].local_seq, 3);

        let after = relay.pull(&d, 1, 10).expect("pull after 1");
        assert_eq!(after.len(), 2);
        assert_eq!(after[0].local_seq, 2);

        let limited = relay.pull(&d, 0, 1).expect("limit");
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].local_seq, 1);

        let range = relay.pull_range(&d, 2, 3).expect("range");
        assert_eq!(range.len(), 2);
        assert_eq!(range[0].local_seq, 2);
        assert_eq!(range[1].local_seq, 3);
    }

    #[test]
    fn memory_relay__arc_shared_put__ok() {
        let relay = Arc::new(MemoryFakeRelay::new());
        let a = Arc::clone(&relay);
        let b = Arc::clone(&relay);
        a.put(&blob(2, 1, 20)).expect("a put");
        b.put(&blob(2, 2, 21)).expect("b put");
        let pulled = relay.pull(&device(2), 0, 10).expect("pull");
        assert_eq!(pulled.len(), 2);
        assert_eq!(pulled[0].envelope_id, uuid_n(20));
        assert_eq!(pulled[1].envelope_id, uuid_n(21));
    }

    #[test]
    fn memory_relay__duplicate_envelope_id__idempotent() {
        let relay = MemoryFakeRelay::new();
        let b1 = blob(3, 1, 30);
        relay.put(&b1).expect("first");
        // Same envelope_id, same (device, seq) → Ok no-op.
        relay.put(&b1).expect("idempotent");
        let pulled = relay.pull(&device(3), 0, 10).expect("pull");
        assert_eq!(pulled.len(), 1);
    }

    #[test]
    fn memory_relay__seq_collision_diff_id__reject() {
        let relay = MemoryFakeRelay::new();
        relay.put(&blob(4, 5, 40)).expect("first");
        // Same (device, seq), different envelope_id.
        let err = relay.put(&blob(4, 5, 41)).expect_err("collision");
        assert!(matches!(err, SyncError::SeqCollision), "got: {err:?}");
    }

    #[test]
    fn memory_relay__body_too_large__err() {
        let relay = MemoryFakeRelay::new();
        let mut b = blob(5, 1, 50);
        b.body = vec![0u8; WIRE_MAX_SIZE + 1];
        let err = relay.put(&b).expect_err("too large");
        assert!(matches!(err, SyncError::RelayBlobTooLarge), "got: {err:?}");
    }

    #[test]
    fn adversary__delay_seq__empty_until_clear() {
        let inner = MemoryFakeRelay::new();
        let adv = AdversarialRelay::new(inner);
        let d = device(6);
        adv.put(&blob(6, 1, 60)).expect("put 1");
        adv.put(&blob(6, 2, 61)).expect("put 2");
        adv.put(&blob(6, 3, 62)).expect("put 3");

        adv.delay_seq(&d, 2).expect("delay");
        let pulled = adv.pull(&d, 0, 10).expect("pull");
        assert_eq!(pulled.len(), 2);
        assert_eq!(pulled[0].local_seq, 1);
        assert_eq!(pulled[1].local_seq, 3);

        let range = adv.pull_range(&d, 2, 2).expect("range delayed");
        assert!(range.is_empty(), "delayed seq must be hidden");

        // Inner still holds the blob (delay ≠ delete).
        let inner_range = adv.inner().pull_range(&d, 2, 2).expect("inner");
        assert_eq!(inner_range.len(), 1);

        adv.clear_delay(&d, 2).expect("clear");
        let after = adv.pull_range(&d, 2, 2).expect("after clear");
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].local_seq, 2);
    }

    #[test]
    fn adversary__reorder_and_duplicate__ok() {
        let adv = AdversarialRelay::new(MemoryFakeRelay::new());
        let d = device(7);
        adv.put(&blob(7, 1, 70)).expect("1");
        adv.put(&blob(7, 2, 71)).expect("2");
        adv.set_reorder(true).expect("reorder");
        let pulled = adv.pull(&d, 0, 10).expect("pull");
        assert_eq!(pulled[0].local_seq, 2);
        assert_eq!(pulled[1].local_seq, 1);

        adv.set_reorder(false).expect("off");
        adv.set_duplicate(true).expect("dup");
        let duped = adv.pull(&d, 0, 10).expect("dup pull");
        assert_eq!(duped.len(), 4);
    }

    #[test]
    fn file_relay__put_pull__roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("relay");
        let relay = FileFakeRelay::open_or_create(&path).expect("open");
        assert!(path.join(FAKE_RELAY_MARKER).exists());

        let d = device(8);
        relay.put(&blob(8, 1, 80)).expect("put 1");
        relay.put(&blob(8, 2, 81)).expect("put 2");

        let pulled = relay.pull(&d, 0, 10).expect("pull");
        assert_eq!(pulled.len(), 2);
        assert_eq!(pulled[0].body, vec![80, 1]);
        assert_eq!(pulled[1].envelope_id, uuid_n(81));

        // Re-open same tree.
        let relay2 = FileFakeRelay::open_or_create(&path).expect("reopen");
        let again = relay2.pull_range(&d, 1, 2).expect("range");
        assert_eq!(again.len(), 2);

        // Idempotent put.
        relay2.put(&blob(8, 1, 80)).expect("idempotent");
    }

    #[test]
    fn file_relay__refuse_root_without_marker() {
        #[cfg(windows)]
        let root = PathBuf::from(r"C:\");
        #[cfg(not(windows))]
        let root = PathBuf::from("/");
        let err = FileFakeRelay::open_or_create(&root).expect_err("root");
        assert!(
            matches!(err, SyncError::RelayPathRefused(_)),
            "got: {err:?}"
        );
    }
}
