# T177 — Fake Relay + Multi-Client Convergence (P11.2)

- **Track ID:** T177-FakeRelayConvergence
- **Phase:** P11 Task 11.2
- **Status:** 📋 **Proposed / Expanded** — ready to implement after human go-ahead
- **Depends on:** T176 **Completed** (`ai-brains-sync`, mig `0027`, CLI `device`/`replicate` stubs); ADR-0018 **Accepted** (L6/L7/L8/L9/L13); T175 threat-model §7 (subset exercised here; full suite T178)
- **Blocks:** T178 (security suite needs harness + adversarial relay knobs)
- **Category:** FEATURE / SECURITY
- **Normative design:** [ADR-0018](../../../Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md) + [T176 R1–R30](../trackT176-sync-crate-schema/spec.md) + [threat-model §7](../trackT175-sync-threat-model-adr/threat-model.md)
- **Deferred absorbed:** #50 freezes (engine path); #34.1 multi-device CE / ACK **over relay**; #51 signer-must-be-enrolled on relay apply; #51 ID-13 package `schema_version` allowlist on enroll apply. **Not** #34.2 DataKey rotation.

## 1. Objective

Prove **two personal devices converge** on encrypted event envelopes through an **untrusted fake relay** — **before** any production network relay — under ADR-0018:

1. **`RelayPort` trait** + **in-memory** and optional **file-backed** fake relays (test/dev only).
2. **`ReplicateEngine`** (client sync session): push/pull sealed envelopes, enrolled-set gate, gap buffer, idempotent apply, ACK emit/tick.
3. **Convergence scenario matrix** — offline diverge, duplicate, reorder, retry/cursor resume, sequence gap, explicit conflict (no LWW), multi-device erasure tombstone + ACK round-trip.
4. Wire CLI **`ai-brains replicate push|pull`** to a **explicitly configured** fake relay only (never silent default network).

After T177:

| Capability | Present |
|------------|---------|
| Two temp vaults converge via `MemoryFakeRelay` | Yes |
| Duplicate / reorder / gap / retry scenarios | Yes |
| Erasure control + signed `ErasureAck` over relay (attestation) | Yes |
| Adversarial drop/reorder knobs for T178 | Yes (test API) |
| Production TCP/HTTP/libp2p relay | **No** |
| Full threat-model §7 claim matrix | **No** — T178 |

**Does not** ship public internet relay, NAT traversal, sealed-sender/metadata hiding, multi-user vaults, MLS, HPKE, DataKey rotation, or Ledgerful `sync` changes.

## 2. Live baseline (re-scan 2026-07-31)

| Area | Live state |
|------|------------|
| T175 / ADR-0018 | **Accepted** — fake-relay-first (§20); L6 order/no LWW; L7 ACK; L8 enrolled-set; L9 store/drop/reorder/duplicate; L13 gap fail-closed |
| T176 | **Completed** — `crates/ai-brains-sync` (sign/wrap/control/apply_order); mig **`0027_replication_state`**; store `projections/replication.rs`; CLI `device` + `replicate status/cursors`; **push/pull → `RelayNotConfigured`** |
| Store → sync | **dev-dependency only** today — T177 production apply path must add a careful **store → sync** edge (capture still forbidden) |
| Capture | No `ai-brains-sync` dep — must stay that way |
| Migrations | Through **0027**; T177 prefers **no new migration** unless a proven hole forces 0028 |
| Workspace | tempfile, serde_json, rusqlite already deny-green; no network crate required for fake path |

## 3. Research summary (online + standards, 2026-07-31)

### 3.1 Dependency posture

| Item | Pin / status | License | T177 action |
|------|--------------|---------|-------------|
| **No new crypto crates** | dalek/hkdf already from T176 | BSD-3 / MIT-Apache | **Reuse** |
| **tempfile** | workspace 3.x (crates.io ~3.27) | MIT/Apache | Integration tests |
| **serde_json** | workspace 1.x | MIT/Apache | Optional file-relay framing only |
| **proptest** | crates.io ~1.11 | MIT/Apache | **Optional** property tests for reorder/duplicate; not required if scenario table is exhaustive |
| **libp2p / cloud SDKs / Matrix AGPL** | — | — | **Forbidden** |
| **axum loopback HTTP** | already in workspace (P7) | MIT | **Out of scope for v1 fake** — memory + file only; public bind forbidden |

**Prefer zero new production dependencies.** Fake relay = `std` + existing serde/tempfile.

### 3.2 Architecture practice — ports & adapters

Industry pattern (hexagonal / Cockburn ports-and-adapters, 2025 refresh):

- Application core talks to a **secondary port** (`RelayPort`).
- **Test adapter first** (`MemoryFakeRelay`); production network adapter later (post-T177).
- Tests drive the hexagon with the real apply pipeline; only the I/O border is faked.

AI-Brains mapping:

```
Vault A / Vault B  (SQLCipher + projections)
        │
        ▼
  ReplicateEngine  (enroll gate → verify → unwrap → append/project → cursor)
        │
        ▼
   RelayPort trait
        │
   ┌────┴────┐
MemoryFake   FileFake   (future: HttpRelay — NOT T177)
```

### 3.3 Delivery semantics (industry consensus)

Sources: common distributed-systems practice (at-least-once + idempotent consumers; Kafka-style “exactly-once” is effectively that pair):

| Claim | T177 freeze |
|-------|-------------|
| Network/relay delivery | **At-least-once** (duplicate allowed — L9) |
| Local apply | **Exactly-once effect** via idempotent `event_id` / `envelope_id` (L5) |
| Order on wire | Untrusted (reorder/drop OK) |
| Apply order | Per-stream `local_seq` continuity + pre-decrypt tie-break `(device_id, local_seq, event_id)` (T176 R30); domain topo **after** DEK open (existing projectors) |
| Never | Last-write-wins merge of epistemic conclusions (L6 / ADR-0014) |

### 3.4 Multi-device E2EE product lessons

| Lesson | Source class | T177 use |
|--------|--------------|----------|
| Linked devices need explicit enrollment ceremony | Signal linked-device model | Use T176 package-export → enroll; then relay carries history |
| Relay is untrusted blob store | ADR-0018 L9 | Fake relay stores **opaque** signed envelopes only |
| Future exclusion on revoke | Signal/Matrix lessons | Stop wraps; reject post-revoke streams after apply |
| ACK ≠ wipe proof | ADR L7 residual | Convergence tests assert **attestation state**, not remote media erase |
| Metadata residual | Threat model | Do not claim privacy from fake-relay logs |

### 3.5 Test hygiene (project + industry)

| Rule | T177 |
|------|------|
| No real network in unit/integration | Memory/file only; no `TcpListener` unless explicitly out-of-scope rejected |
| `tempfile::tempdir()` per test | Two vaults under one temp root |
| No sleep-for-async | Synchronous engine API; poll helpers only if needed with `wait_for_condition` |
| Deterministic fixtures | Fixed seeds for golden membership packages where needed; OS CSPRNG for real keys OK if assertions are structural |
| No plaintext in assertions of relay contents | Assert ciphertext non-empty / lengths / ids — never log DEKs or plaintext bodies |
| Naming | `feature__condition__expected` |

## 4. Design locks (normative for implement)

| ID | Lock |
|----|------|
| **F1** | **Fake-relay-first:** no production network adapter in T177. Memory required; file optional. |
| **F2** | **`RelayPort`** is the only engine↔relay surface. Engine MUST NOT call sockets. All trait methods take **`&self`** (interior mutability in fakes) so TwinVaults can share `Arc<dyn RelayPort>` / `Arc<MemoryFakeRelay>` without exclusive `&mut` contention. |
| **F3** | Relay stores **opaque** envelope blobs + routing metadata the protocol already treats as public (`device_id`, `local_seq`, `envelope_id`, sizes). **No** private keys, DataKey, or plaintext content. Soft body size cap **16 MiB** → structured error (hardening). |
| **F4** | **Convergence oracle (primary):** same set of applied **event_ids** (and control membership effects) on both vaults after N push/pull rounds — not SQL file equality. Harness helper `assert_converged(a, b)`. |
| **F5** | **Secondary oracle:** enrolled device sets match (status active/local); revoked/tombstoned ids match; for CE scenarios, `content_key_store` destroyed status + ACK projection states match per policy. |
| **F6** | **Conflict oracle:** concurrent contradictory conclusions produce **explicit conflict records** on both sides (or same open conflict set) — **never** silent LWW winner. Fallback: both event_ids present at log level if projection wiring is costly (C9). |
| **F7** | **Idempotent apply:** same `event_id` / `envelope_id` re-push → no double projection. **Different** `event_id` for same `(sender_device_id, local_seq)` → **reject** + cursor `blocked` (protocol violation; SQLite UNIQUE must not crash). |
| **F8** | **Gap (L13):** out-of-order / multi-gap → buffer by `(peer_device_id, local_seq)`; **never** apply past `expected_local_seq`. After each successful `put`/`pull`/`apply`, run **sequential drain loop** (F19). Bodies re-fetched via `pull_range`. Permanent loss → fail-closed until `GapSkipAudit` (not min-bar auto-skip). |
| **F9** | **L8 pre-verify (strict order):** (1) parse header `sender_device_id`/`local_seq`; (2) lookup `device_identity`; (3) if missing **or** status ≠ `active`\|`local` (includes **`revoked`**) → reject with structured error **before** Ed25519 verify / DEK unwrap. Signer of enroll/revoke must be enrolled. Engine enforces **pre-append**; projector remains post-append trust of log. |
| **F10** | **Package schema allowlist (#51 ID-13):** reject when `schema_version != REPLICATION_SCHEMA_VERSION` (fail closed). Check lives in **engine apply / enroll path**, not only CLI. |
| **F11** | **ACK cycles:** `sync_round` → store **`tick_ack_cycle`** after each round; after `ACK_TIMEOUT_SYNC_CYCLES` (3) without ack → `unreachable`. |
| **F12** | **CLI:** `replicate push|pull` only when fake relay **explicitly configured** (`--fake-relay <path>` / `AI_BRAINS_SYNC_FAKE_RELAY_PATH`). Unset → T176-class error. **Never** default-on. **No** `replicate sync` alias (L10 spirit). Support `--format json` and `--quiet` where project CLI standards apply. |
| **F13** | **Naming:** engine ops are **`replicate`** only — do not touch Ledgerful `sync` / `safety sync`. |
| **F14** | Capture independence preserved; no capture→sync edge. |
| **F15** | Prefer **no new migration**; use 0027 tables. New migration only if a proven schema hole blocks scenarios (then **0028+**). |
| **F16** | File fake: `open_or_create` may `create_dir_all`; write **`.aibrains_fake_relay_marker`** sentinel; refuse paths that look like system roots or non-relay trees without marker. Residual: local disk ≠ secure relay. |
| **F17** | Adversarial knobs as composable **`AdversarialRelay<R: RelayPort>`** decorator (drop/reorder/duplicate/delay) — test/T178 handoff; not production defaults. |
| **F18** | Zero `unwrap`/`expect`/panic in production engine paths. |
| **F19** | **Gap drain:** after any successful receive/apply, repeatedly apply buffered blob for `expected_local_seq` until no sequential match; support **discontiguous** multi-gaps (e.g. missing 4,7,9). `replicate status` surfaces unresolved gap / `sync_gap` / blocked. |
| **F20** | **Wire codec (blocker):** canonical binary `encode_signed_envelope` / `decode_signed_envelope` in `ai-brains-sync` (**Phase A0**). Deterministic BE / length-prefix; **no** serde_json / bincode/postcard required (hand-rolled; zero new deps). Version prefix byte. KAT test. |
| **F21** | **CE tombstone apply (C10):** on `ContentErasureTombstone` control: (1) `destroy_content_key_wrap` (local CE; also clears peer wraps for key via store hook); (2) ensure destroyed status; (3) upsert local erasure_ack pending for peers; (4) queue signed `ErasureAck` for push. ACK = attestation only. |
| **F22** | **Peer discovery:** `pull_all_peers` peer list from **`list_enrolled_devices`** (local vault), **not** from the relay. Do not add `list_devices` to `RelayPort`. |

## 5. API sketch

### 5.0 Wire codec (Phase A0 prerequisite — F20)

`SignedEnvelope` has **no** encode/decode today (`envelope.rs` holds fields only). **Blocker** before relay put/pull.

Add `ai-brains-sync` module (`wire.rs` or extend `envelope.rs`):

```text
encode_signed_envelope(env: &SignedEnvelope) -> Result<Vec<u8>>
decode_signed_envelope(bytes: &[u8]) -> Result<SignedEnvelope>
```

**Canonical binary framing (normative — hand-rolled; zero new crates):**

```text
wire_v1 =
    magic[4]           = b"AIBR"          # AI-Brains Relay
  ‖ version[1]         = 0x01
  ‖ signed_bytes       = build_signed_bytes(...)   # ADR-0018 §5.2 (already canonical)
  ‖ signature[64]      = Ed25519 detached
```

| Rule | Value |
|------|--------|
| Integers | Big-endian (match T176) |
| Domain events | **Not** embedded as JSON on wire; body is already AEAD or cleartext control bytes inside `signed_bytes` |
| Forbidden | `serde_json` for interop wire; ad-hoc field reordering; bincode/postcard **not required** (avoid dep unless review forces) |
| KAT | Fixed fixture keys/ids → exact hex of `wire_v1` |
| Size | Reject encode/decode if total > **16 MiB** (F3) |

`RelayBlob.body` = full `wire_v1` bytes. Routing fields on `RelayBlob` are denormalized for pull indexes and **must match** decoded outer fields (mismatch → reject).

### 5.1 `RelayPort` (`ai-brains-sync::relay`)

```rust
/// Opaque transport unit. `body` = wire_v1 of SignedEnvelope (F20).
pub struct RelayBlob {
    pub envelope_id: Uuid,
    pub sender_device_id: DeviceId,
    pub local_seq: u64,
    pub content_type_code: u16,
    pub body: Vec<u8>,
}

/// Interior-mutability friendly: all methods &self (F2).
pub trait RelayPort: Send + Sync {
    fn put(&self, blob: &RelayBlob) -> Result<()>;
    /// Blobs with local_seq > after_seq, ascending, up to limit.
    fn pull(
        &self,
        sender_device_id: &DeviceId,
        after_seq: u64,
        limit: usize,
    ) -> Result<Vec<RelayBlob>>;
    /// Inclusive range [from_seq, to_seq] for gap fill.
    fn pull_range(
        &self,
        sender_device_id: &DeviceId,
        from_seq: u64,
        to_seq: u64,
    ) -> Result<Vec<RelayBlob>>;
}
```

| Put semantics | Prefer **idempotent by `envelope_id`** (same id re-put = Ok no-op). Same `(device, seq)` different envelope_id → reject or last-write policy frozen as **reject** for protocol honesty. |
| Soft cap | Body > 16 MiB → error before store |

### 5.2 Fake implementations

| Impl | Location | Role |
|------|----------|------|
| `MemoryFakeRelay` | `ai-brains-sync` | `Mutex<HashMap<…>>` interior mutability; share via `Arc` |
| `FileFakeRelay` | `ai-brains-sync` | `relay_root/<device_id>/<seq>.blob` + index; `open_or_create` + **`.aibrains_fake_relay_marker`** (F16). C14 does **not** require crash-safe fsync unless marked `__slow`. |
| `AdversarialRelay<R: RelayPort>` | `ai-brains-sync` (test-visible) | Decorator: delay/drop/reorder/duplicate; T178 reuses |

**Adversary `drop_seq` default for C5:** **delay** (pull_range returns empty until restored), not permanent delete. Restore = **sender re-push** of missing envelope (not silent relay resurrection). Permanent loss → gap stays `sync_gap` until operator `GapSkipAudit`.

### 5.3 `ReplicateEngine` (store + sync)

| Preferred placement | Rationale |
|---------------------|-----------|
| `ai-brains-store` prod dep on `ai-brains-sync` + `replication_engine` module | SQL + crypto types; **Phase B1** triggers `cargo deny` + `cargo audit` |
| Integration tests | `ai-brains-store/tests` and/or `ai-brains-sync/tests` |

Engine operations:

| Method | Behavior |
|--------|----------|
| `push_pending(&mut self)` | Encode pending local envelopes → wire_v1 → `relay.put`; advance send cursor |
| `pull_all_peers(&mut self)` | Peers from **`list_enrolled_devices`** (F22); per peer `pull`; apply; **drain gaps** (F19) |
| `sync_round(&mut self)` | `push_pending` then `pull_all_peers` then store **`tick_ack_cycle`** (F11) |
| `apply_blob(&mut self, blob)` | Single-envelope path |

**Apply pipeline (normative order):**

```text
1. Decode wire_v1 → SignedEnvelope; check RelayBlob routing fields match outer
2. L8 PRE-VERIFY (F9):
     parse sender_device_id
     lookup device_identity
     if unknown OR status ∉ {active, local}  →  DeviceNotEnrolled / DeviceRevoked
     (includes revoked — no Ed25519, no X25519)
3. Lookup enrolled Ed25519 pub; verify signature over signed_bytes
4. Seq / gap gate:
     if local_seq > expected → buffer metadata only; set sync_gap; return (no project)
     if local_seq < expected → if same event_id idempotent Ok; else reject
     if local_seq == expected → continue apply
5. Control vs data:
   5a. Control:
       - DeviceEnrolled / DeviceRevoked: F9 signer enrolled; F10 schema_version; project
       - ContentErasureTombstone (F21):
           destroy_content_key_wrap(content_key_id)   // CE + peer wrap cleanup hook
           assert/is destroyed
           upsert erasure_ack_projection pending for relevant peers
           queue ErasureAck for push_pending
       - ErasureAck: upsert peer row → acked (attestation only)
       - GapSkipAudit: apply skip only if valid signed audit policy path
   5b. Data: peer wrap for self → unwrap DEK → open body → append domain event / project
6. insert_envelope_index (idempotent by event_id); on UNIQUE(sender,seq) conflict with different event_id → reject + cursor blocked (F7)
7. expected_local_seq += 1; high_water update
8. DRAIN LOOP (F19): while gap_buffer has row for (peer, expected_local_seq):
     re-fetch body via pull_range if needed; apply; advance; repeat
9. If no further sequential buffer and no missing: state = in_sync else sync_gap
```

**Engine vs projector:** L8/schema/signer checks are **pre-append** in the engine. `ReplicationProjection` continues to project from trusted local event log after append.

### 5.4 TwinVaults harness

```text
// Preferred: TwinVaults::new_enrolled_pair() in store or sync tests
struct TwinVaults {
  a: TestVault,
  b: TestVault,
  relay: Arc<MemoryFakeRelay>,  // shared; &self put/pull
}
impl TwinVaults {
  fn new_enrolled_pair() -> Self;       // dual bootstrap + mutual package OOB + enroll
  fn sync_round_both(&mut self);        // A then B sync_round (or interleaved variants)
  fn assert_converged(&self);           // F4 event_id sets equal
}
```

Ceremony: symmetric package exchange (both devices' packages OOB-enrolled on the other vault), then membership control envelopes may still ride the relay for projection parity. Aim: **&lt;10 lines** per scenario test body.

## 6. Scenario matrix (normative tests)

| ID | Scenario | Expected |
|----|----------|----------|
| **C1** | Happy path: A seals data + wrap for B; push/pull | B applies; `assert_converged`; B opens DEK |
| **C2** | Offline diverge; mutual sync | Event_id **union** both sides |
| **C3** | Duplicate put (same envelope_id) | Single apply |
| **C4** | Reorder (seq 3 before 2); multi-gap drain | Buffer; drain applies 2 then 3 in order |
| **C5** | Adversary **delays** middle seq (empty pull_range); sender **re-pushes**; fill | `sync_gap` while missing; then `in_sync`; no past-gap project |
| **C6** | Cursor resume limit=1 | Full converge after rounds |
| **C7** | Unknown `device_id` inject | Reject pre-verify |
| **C8** | Self-enroll unknown signer | Reject (F9) |
| **C9** | Concurrent contradictory domain events | Prefer conflict projection both sides; **fallback:** both event_ids present, neither dropped; ISSUES if projection deferred |
| **C10** | Erasure tombstone A→B; CE destroy; ErasureAck B→A | F21; ACK `acked`; not wipe claim |
| **C11** | No ack; `tick_ack_cycle` ×3 | `unreachable` |
| **C12** | Revoke B; further B-signed envelopes | A rejects **revoked** pre-verify (F9); R23 wrap cleanup; no future wraps to B |
| **C13** | schema_version ≠ current | Fail closed |
| **C14** | FileFakeRelay smoke | C1 over `.blob` path optional / `__slow` |
| **C15** | Same `(sender, local_seq)`, **different** `event_id` | Reject + cursor `blocked` (F7) |

Minimum bar Complete: **C1–C8, C10** green. Strongly recommended: C9 (with fallback), C11–C13, C15. C14 optional.

## 7. CLI / config

| Surface | Behavior |
|---------|----------|
| `ai-brains replicate status` | Enrolled, cursors, gap/blocked state (F19), **relay: file\|not configured** |
| `ai-brains replicate cursors` | Dump cursors |
| `ai-brains replicate push` | Engine push if configured; else error |
| `ai-brains replicate pull` | Engine pull if configured |
| **No** `replicate sync` | Document `push` then `pull` only (L10) |
| Flags | `--fake-relay <dir>`; env `AI_BRAINS_SYNC_FAKE_RELAY_PATH`; **`--format json`**, **`--quiet`** where CLI standards apply |
| File open | `create_dir_all` + marker file (F16) |

Never bind `0.0.0.0`. Never auto-create relay without explicit path.

## 8. Placement & dependency edges

```
ai-brains-sync
  + wire codec encode/decode SignedEnvelope (F20)
  + relay::{RelayPort, MemoryFakeRelay, FileFakeRelay, AdversarialRelay, RelayBlob}
  + (existing) envelope/wrap/control/apply_order

ai-brains-store
  + PROD depends on ai-brains-sync (Phase B1: deny+audit after edge)
  + replication_engine.rs (apply pipeline F9–F21)
  + projections/replication + content_envelope CE destroy

ai-brains-cli
  + replicate push/pull → engine + FileFakeRelay when configured
  + --format json / --quiet

ai-brains-capture
  + still NO sync dependency
```

## 9. Deferred.md absorption

| Item | Disposition in T177 |
|------|---------------------|
| **#50** ADR freezes | Engine implements L6/L7/L8/L9/L13 under fake relay |
| **#34.1** multi-device CE / ACK over relay | **C10–C11** — tombstone propagate + ACK projection + timeout |
| **#51** signer-must-be-enrolled on remote apply | **F9 / C8** |
| **#51 ID-13** schema_version allowlist | **F10 / C13** |
| **#34.2** DataKey rotation | **Out of scope** — remains open |
| T176 residual “full WRAP KAT matrix” | Still **T178** |
| Adversarial crypto (meta-swap, forged ACK) | Prefer **T178**; T177 may add thin smoke if free |

## 10. Non-goals

- Production HTTP/WebSocket/libp2p relay
- Public bind / discovery / NAT
- Sealed sender / metadata-private relay
- Multi-user / MLS
- SQLCipher file sync
- Changing capture pipeline
- DataKey rotation
- Claiming multi-device CE = NIST Purge / remote wipe
- Repurposing CLI `sync` / `safety sync`

## 11. Testing strategy

### 11.1 Unit (`ai-brains-sync`)

| Test | Intent |
|------|--------|
| `wire_signed_envelope__fixture__exact_hex` | F20 KAT |
| `wire_signed_envelope__roundtrip__eq` | Encode/decode |
| `memory_relay__put_pull__roundtrip` | Blob fidelity |
| `memory_relay__duplicate_envelope_id__idempotent` | F7 put semantics |
| `memory_relay__arc_shared_put__ok` | F2 interior mutability |
| `file_relay__put_pull__roundtrip` | Optional |
| `adversary__delay_seq__empty_until_repush` | C5 semantics |

### 11.2 Integration (two vaults)

Named tests matching §6 matrix IDs (`converge__offline_diverge__event_id_union`, etc.).

### 11.3 CLI smoke

- Configure file fake relay; bootstrap+enroll ceremony; push/pull; status shows configured  
- Unset relay → structured error  

### 11.4 Gates

```powershell
cargo nextest run -p ai-brains-sync -p ai-brains-store
cargo clippy -p ai-brains-sync -p ai-brains-store -p ai-brains-cli --all-targets -- -D warnings
cargo deny check
cargo audit
```

## 12. Risks & honesty

| Residual | Handling |
|----------|----------|
| Fake relay ≠ production security | Docs: adversarial model still L9; file path is local trust |
| ACK attestation ≠ wipe | C10 asserts projection only |
| Offline peer lag | Document; C10 partial until B syncs |
| Conflict fixtures complexity | C9 fallback: event-level both present; ISSUES if projection deferred |
| store→sync edge | deny+audit after B1; capture graph clean |
| Multi-gap stall | F19 drain + status surfaces sync_gap |

## 13. Definition of Done

- [ ] **A0:** wire codec + KAT (F20)
- [ ] `RelayPort` (`&self`) + `MemoryFakeRelay` + optional `FileFakeRelay` + `AdversarialRelay`
- [ ] `ReplicateEngine` with F9 L8 pre-verify, F19 drain, F21 CE tombstone, F11 `tick_ack_cycle`
- [ ] Scenario minimum **C1–C8, C10** green; C15 recommended
- [ ] CLI push/pull via explicit fake-relay only; no `replicate sync`; status shows gaps
- [ ] #51 signer + schema_version on engine path; #34.1 ACK over relay
- [ ] No production network; capture independence held
- [ ] deny/audit green after store→sync prod edge; manual evidence
- [ ] T178 handoff: `AdversarialRelay` export + harness notes

## 14. Review fold-in (2026-07-31 AI1–AI2)

| Source | Disposition |
|--------|-------------|
| AI1 BS1 `&self` + interior mutability | **Agree** → F2 (prefer over AI2-only `&mut` for TwinVaults Arc share) |
| AI1 BS2 / AI2 #1 wire codec | **Agree** → F20 / §5.0 hand-rolled `AIBR` framing (not bincode required) |
| AI1 BS3 multi-gap drain | **Agree** → F19 |
| AI1 BS4 CE tombstone → store CE | **Agree** → F21 (`destroy_content_key_wrap`) |
| AI1 BS5 revoked pre-verify | **Agree** → F9 reinforced |
| AI1 TwinVaults helper | **Agree** → §5.4 |
| AI1 file marker + create_dir_all | **Agree** → F16 |
| AI1 CLI json/quiet | **Agree** → F12 |
| AI2 peer list local not relay | **Agree** → F22 |
| AI2 C5 delay not delete | **Agree** → §5.2 / C5 |
| AI2 seq collision different event_id | **Agree** → F7 / C15 |
| AI2 C9 fallback | **Agree** → F6 / C9 |
| AI2 drop `replicate sync` | **Agree** → F12 |
| AI2 AdversarialRelay decorator | **Agree** → F17 |
| AI2 assert_converged | **Agree** → F4 / §5.4 |
| AI2 tick_ack_cycle explicit | **Agree** → F11 |
| AI2 deny after store→sync | **Agree** → plan B1 |
| AI2 16 MiB soft cap | **Agree** → F3 |
| AI2 `.blob` extension | **Agree** → §5.2 |
| AI2 EphemeralSecret wrap migrate | **Defer** — T176 hygiene, not T177 scope |
| AI2 proptest optional | **Agree** — remains optional dev-only |

## 15. References

- ADR-0018 §§5–9, 13, 20; threat-model §7
- T176 spec R1–R30; deferred #50, #34.1, #51
- Hexagonal ports/adapters (test adapter first)
- At-least-once + idempotent apply industry practice
- Live: `crates/ai-brains-sync`, `projections/replication.rs`, `content_envelope::destroy_content_key_wrap`, `commands/replicate.rs`
