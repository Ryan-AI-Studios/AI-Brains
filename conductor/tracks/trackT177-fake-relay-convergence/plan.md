# T177 Plan — Fake Relay + Multi-Client Convergence (P11.2)

Status: **Completed** (Codex R5 PASS, 2026-07-31).
Normative: ADR-0018 + T176 Complete + `spec.md` locks **F1–F22**.

## Handoff freezes (do not re-litigate)

- Fake-relay-first; no production network
- **A0 wire codec** before relay work (F20)
- `RelayPort` all **`&self`** + interior mutability (F2)
- Gap **drain loop** multi-gap (F19); L8 revoked pre-verify (F9)
- CE tombstone → `destroy_content_key_wrap` + ErasureAck queue (F21)
- Convergence = event_id sets; never LWW; no `replicate sync` alias
- Absorb #51 signer + schema_version; #34.1 ACK over relay
- Prefer no new migration (0027)

## Preflight (before first edit)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [x] `ledgerful scan --impact` (sync, store replication, cli replicate)
- [x] Confirm T176 green; push/pull currently `RelayNotConfigured`
- [x] Confirm capture has no sync dependency
- [x] Note: Phase B1 prod store→sync requires **deny + audit** immediately after

## Phase A0 — Wire codec (blocker)

- [x] **A0.1** RED: KAT hex for fixed SignedEnvelope fixture
- [x] **A0.2** GREEN: `encode_signed_envelope` / `decode_signed_envelope` (`AIBR` ‖ v1 ‖ signed_bytes ‖ sig64)
- [x] **A0.3** Roundtrip + size-cap (>16 MiB err) tests
- [x] **A0.4** Export from `ai-brains-sync` lib

## Phase A — Relay port + fakes

- [x] **A1** RED: memory put/pull; Arc shared put from two handles
- [x] **A2** GREEN: `RelayPort` (`&self`), `MemoryFakeRelay` (`Mutex` map), `RelayBlob`
- [x] **A3** Put idempotent by `envelope_id`
- [x] **A4** Optional `FileFakeRelay` (`.blob`, marker, `open_or_create`)
- [x] **A5** `AdversarialRelay<R>` decorator (delay/drop/reorder/duplicate) — export for T178

## Phase B — Apply pipeline + engine

- [x] **B1** Prod `ai-brains-store` → `ai-brains-sync`; **`cargo deny check` + `cargo audit`**
- [x] **B2** RED: L8 unknown / revoked reject pre-verify
- [x] **B3** GREEN: decode wire → F9 → verify → apply control enroll
- [x] **B4** F10 schema_version reject on engine path
- [x] **B5** Data path: unwrap peer DEK + open + append/index
- [x] **B6** Gap buffer + **drain loop** (discontiguous gaps); multi-gap test
- [x] **B7** `ReplicateEngine::{push_pending, pull_all_peers, sync_round}`; after round call store **`tick_ack_cycle`**
- [x] **B8** F21 tombstone → `destroy_content_key_wrap` + queue ErasureAck
- [x] **B9** F7 seq collision different event_id → blocked
- [x] **B10** `TwinVaults::new_enrolled_pair` + `assert_converged`

## Phase C — Scenario matrix

- [x] **C1–C8, C10** (minimum Complete)
- [x] **C5** delay-not-delete + sender re-push restore
- [x] **C9** with fallback documented
- [x] **C11–C13, C15** strongly recommended
- [x] **C14** optional file smoke

## Phase D — CLI

- [x] **D1** `--fake-relay` / env; marker + create_dir_all
- [x] **D2** push/pull → engine; else structured err
- [x] **D3** status: relay config + gap/blocked (F19)
- [x] **D4** `--format json` / `--quiet` where standards apply
- [x] **D5** **No** `replicate sync` subcommand
- [x] **D6** Honesty strings; leave Ledgerful `sync` alone

## Phase E — Deferred hygiene

- [x] **E1** #34.1 ACK over relay when C10/C11 land
- [x] **E2** #51 signer + ID-13 absorbed when F9/F10 ship
- [x] **E3** #34.2 remains open
- [x] **E4** T178 handoff: AdversarialRelay + harness

## Phase F — Verification & close

- [x] Targeted nextest + clippy (sync/store/cli)
- [x] Full gate including deny/audit
- [x] Manual two-vault file fake evidence
- [x] ledgerful verify + review.md + cross-model SECURITY
- [x] conductor Complete only after review convergence

## RED tests named (minimum)

```
wire_signed_envelope__fixture__exact_hex
wire_signed_envelope__roundtrip__eq
memory_relay__put_pull__roundtrip
memory_relay__arc_shared_put__ok
memory_relay__duplicate_envelope_id__idempotent
engine_apply__unknown_device__reject_preverify
engine_apply__revoked_device__reject_preverify
engine_apply__enroll_bad_signer__reject
engine_apply__schema_version_unknown__reject
engine_apply__seq_collision_diff_event__blocked
gap_drain__discontiguous_missing__ordered_apply
converge__happy_path_wrap__event_id_match
converge__offline_diverge__event_id_union
converge__duplicate_push__single_apply
converge__reorder_pull__gap_then_fill
converge__delay_seq__repush_restore
converge__erasure_tombstone__ack_acked
ack_tick__three_cycles_no_ack__unreachable
cli_replicate_push__no_config__err
cli_replicate_push__fake_relay__ok
```

## Out of scope (reject if PR sneaks them in)

- [x] Public TCP/HTTP/libp2p relay
- [x] `replicate sync` alias
- [x] bincode/postcard as required prod deps (hand-roll wire)
- [x] EphemeralSecret wrap migration (T176 hygiene)
- [x] DataKey rotation
- [x] Capture → sync dependency
- [x] `unwrap`/`expect`/panic in engine

## Implementation order

```
A0 wire codec → relay &self fakes → store→sync + engine apply → gap drain → twin harness → matrix → CLI → gate
```

No implement until user go-ahead.
