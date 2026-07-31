**Verdict**

FAIL

**Finding**
1. `P1/security` - `ErasureAck` apply trusts the payload’s `peer_device_id` instead of binding the ACK to the authenticated sender. The payload is defined as peer attestation in [control.rs](/C:/dev/AI-Brains/crates/ai-brains-sync/src/control.rs:34), but the receiver path in [replication_engine.rs](/C:/dev/AI-Brains/crates/ai-brains-store/src/replication_engine.rs:809) upserts the ACK row under `p.peer_device_id` without checking that it matches `signed.outer.device_id`. That lets any enrolled device sign an `ErasureAck` naming some other peer and cause the receiver to record that other peer as `acked`. The happy-path test in [replication_converge.rs](/C:/dev/AI-Brains/crates/ai-brains-store/tests/replication_converge.rs:989) does not cover the sender/payload mismatch case. Required fix: reject `ErasureAck` unless `p.peer_device_id == signed.outer.device_id`, and add a regression test for forged/mismatched ACK identity.

**Assumption**
- I treated erasure ACKs as integrity-relevant per-peer attestations, not cosmetic metadata, because the contract calls them “peer attestation” and the projection key is `(erasure_id, peer_device_id)`.

**R3 closure**
- The specific R3 pending-leak fix itself looks closed. [replication_engine.rs](/C:/dev/AI-Brains/crates/ai-brains-store/src/replication_engine.rs:240) and [replication_engine.rs](/C:/dev/AI-Brains/crates/ai-brains-store/src/replication_engine.rs:252) keep `queue_control_on` durable-only and only mirror to in-memory `pending` after success. [replication_engine.rs](/C:/dev/AI-Brains/crates/ai-brains-store/src/replication_engine.rs:893) queues inbound `ErasureAck` via outbox-only inside the apply transaction. [replication_engine.rs](/C:/dev/AI-Brains/crates/ai-brains-store/src/replication_engine.rs:282) drains from durable outbox, and the local revoke/tombstone helpers only push to `pending` after commit at [replication_engine.rs](/C:/dev/AI-Brains/crates/ai-brains-store/src/replication_engine.rs:1165) and [replication_engine.rs](/C:/dev/AI-Brains/crates/ai-brains-store/src/replication_engine.rs:1237).

I could not rerun `ledgerful` or cargo/nextest in this session because the sandbox was read-only and those commands attempted DB/build writes.