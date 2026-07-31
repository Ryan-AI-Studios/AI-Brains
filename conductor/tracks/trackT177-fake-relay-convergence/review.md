# T177 Review Log — Fake Relay + Multi-Client Convergence (P11.2)

- **Track status:** Completed (Codex R5 **PASS**, zero findings)
- **Spec / plan:** `conductor/tracks/trackT177-fake-relay-convergence/{spec,plan}.md`
- **Cross-model artifacts:** `review.codex.md` (R1), `review.codex.r2.md` (R2), `review.codex.r3.md` (R3), `review.codex.r4.md` (R4), `review.codex.r5.md` (R5 final)

---

## Review rounds

| Round | Source | Verdict | Notes |
|-------|--------|---------|-------|
| Internal R1 | subagent | NEEDS_FIX | H1 C10 vacuous ACK; M1 C6; M2 outbox; M3 destroy verify; L1–L3 |
| Internal R2 | subagent | CLEAN | Prior findings verified_fixed |
| Codex R1 | gpt-5.4 | FAIL | P1 domain append; P2 GapSkipAudit; P2 JSON; P2 gov |
| Codex R2 | gpt-5.4 | FAIL | P1 smuggle membership; P2 atomicity; P3 JSON field |
| Codex R3 | gpt-5.4 | FAIL | P2 pending ACK leak on TX rollback |
| Codex R4 | gpt-5.4 | FAIL | P1 ErasureAck peer_device_id unbound to signer |
| Codex R5 | gpt-5.4 | **PASS** | No findings; F1–F22 met; no deferred P3 |

---

## Findings (all closed)

### Internal R1

| ID | Sev | Status | Fix |
|----|-----|--------|-----|
| H1 C10 vacuous ACK | high | verified_fixed | Hard assert A records B ErasureAck as `acked` |
| M1 C6 pull_limit | medium | verified_fixed | `with_pull_limit` + multi-round test |
| M2 durable outbox | medium | verified_fixed | Migration **0028** + restart push test |
| M3 destroy verify | medium | verified_fixed | Re-read status after destroy when row existed |
| L1–L3 | low | verified_fixed | Docs, CLI marker, behind-seq reject |

### Codex R1–R4

| ID | Sev | Status | Fix |
|----|-----|--------|-----|
| R1-P1 domain append | high | verified_fixed | `append_event_in_tx` after open; C1 domain event assert |
| R1-P2 GapSkipAudit | medium | verified_fixed | `apply_gap_skip_on` advances + drain |
| R1-P2 JSON CLI | medium | verified_fixed | `--format json` on push/pull |
| R1-P2 governance | low | verified_fixed | This log; Complete after R5 |
| R2-P1 smuggle | high | verified_fixed | Reject DeviceEnrolled/Revoked in DataEvent path |
| R2-P2 atomic apply | medium | verified_fixed | Single TX project+index+cursor; queue helpers TX |
| R2-P3 applied field | medium | verified_fixed | Pull JSON `applied`; CLI JSON tests |
| R3-P2 pending leak | medium | verified_fixed | `queue_control_on` outbox-only; pending after commit |
| R4-P1 ErasureAck bind | high | verified_fixed | `peer_device_id == outer.device_id`; mismatch test |

---

## Gates (orchestrator-observed)

| Gate | Result |
|------|--------|
| `cargo nextest run -p ai-brains-store --test replication_converge` | **22/22** |
| `cargo nextest run -p ai-brains-cli --test device_replicate_cli` | **14/14** (prior) |
| `cargo nextest run -p ai-brains-sync -p ai-brains-store` | **145+** (pre-R4 growth) |
| `cargo clippy -p ai-brains-sync -p ai-brains-store -p ai-brains-cli --all-targets -- -D warnings` | clean |
| `cargo deny check` / `cargo audit` | ok (after store→sync edge) |
| Codex R5 | **PASS** zero findings |

---

## Manual evidence

```powershell
# Two-vault file fake (CLI)
$relay = Join-Path $env:TEMP "aib-t177-relay"
$va = Join-Path $env:TEMP "aib-t177-a.db"
$vb = Join-Path $env:TEMP "aib-t177-b.db"
# init + device bootstrap each vault; enroll OOB packages mutually;
# ai-brains --vault-path $va replicate push --fake-relay $relay
# ai-brains --vault-path $vb replicate pull --fake-relay $relay
# status shows relay: file:... and cursors
```

Primary proof is TwinVaults integration matrix (C1–C13/C15) in `replication_converge.rs`.

---

## Deferred

None from this track for ISSUES (Codex R5: no deferred P3). Residual product items remain in `conductor/deferred.md`:

- **#34.2** DataKey rotation (out of scope)
- **T178** full threat-model §7 / WRAP KAT / adversarial crypto suite
- C14 optional file twin smoke (file unit tests present)
- Bootstrap CLI does not yet enqueue DeviceEnrolled to outbox (engine/OOB enroll path used for convergence; CLI push may report 0 until seal APIs wire)

---

## Completion decision

Engineering DoD met. Internal clean. **Codex R5 PASS** fresh final gate. Ready for PR + merge.
