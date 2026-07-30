# T177 — Fake Relay + Multi-Client Convergence (P11.2)

- **Track ID:** T177-FakeRelayConvergence
- **Phase:** P11 Task 11.2
- **Status:** 📋 **Proposed / Unblocked (design)** — T175 Complete / ADR-0018 **Accepted**; still depends on T176 types (not implementing yet)
- **Depends on:** T176 types/schema; T175 ordering/conflict rules (ADR-0018 **Accepted** ✅)
- **Category:** FEATURE / SECURITY

## Handoff freezes from T175 / ADR-0018 (Accepted — design unblocked; needs T176)

| Freeze | Value |
|--------|--------|
| **Fake-relay-first** | In-memory/file relay **before** any real network (ADR-0018 §20) |
| **Ordering** | Topological apply; tie-break `(device_id, local_seq, event_id)`; **never LWW** (L6) |
| **Gap** | Buffer / request missing seq; fail-closed skip default (L13) |
| **Idempotency** | At-least-once delivery → exactly-once apply by event/envelope id (L5) |
| **CLI engine ops** | Exercised via **`replicate`** semantics (not Ledgerful `sync`) |
| **Design refs** | ADR-0018 L6/L13; threat-model peer-via-relay STRIDE |

## Placeholder objective

Implement an **in-memory and/or file-backed fake relay** used only in tests (and optional local dev). Prove **two clients converge** after offline divergence, duplicates, reordering, and retry. **Only then** design a real network relay.

## Master-plan proof points

| Scenario | Expected |
|----------|----------|
| Offline divergence | Both clients push/pull; converge to same event set (modulo explicit conflicts) |
| Duplicates | Idempotent; no double-apply |
| Reordering | Apply order safe; projectors deterministic |
| Retry | Partial upload/download resumes via cursors |
| Sequence gap | Gap buffer / fill; no corrupt project-past-gap |
| Explicit conflicts | Surface conflict records — **not** silent LWW |

## Expected deliverables (sketch)

| Item | Notes |
|------|--------|
| `FakeRelay` trait + memory/file impls | No TCP required |
| Client sync engine (test harness) | Push/pull encrypted envelopes |
| Convergence tests | Two temp vaults; assert projection digests / event id sets |
| Dev flag | Optional `AI_BRAINS_SYNC_FAKE_RELAY_PATH` — never default-on |
| Docs | How to run convergence tests; non-claims about network security |

## Best practices applied

- **Test double first** — network is an adapter later (hexagonal).  
- **Deterministic fixtures** — fixed keys in tests; zeroize still used.  
- **No real network in unit/integration** — project test rule.  
- **Cursor-based resume** — standard sync hygiene.  
- **Conflict visibility** — matches epistemic model (ADR-0011/0014).  

## License / commercial constraints

- Prefer **zero new network crates** in this track.  
- File relay: `std::fs` + existing serde_json.  
- Do not embed cloud SDKs (AWS/GCP sync) for the fake path.  
- If a tiny HTTP loopback appears for multi-process tests, reuse axum/hyper **only if already approved in P7** and deny-green — still bind `127.0.0.1`.

## Non-goals

- Public internet relay deployment  
- NAT traversal / libp2p  
- Mobile clients  
- Replacing Ledgerful `sync query`  

## Expand before implement

- [x] ADR-0018 Accepted (T175 Complete 2026-07-30)
- [ ] T176 types available
- [ ] FakeRelay API from ADR  
- [ ] Convergence oracle (what equality means: event ids vs full projection hash)  
- [ ] Conflict fixture design  
- [ ] RED multi-client tests first  

## Definition of Done (when fleshed out)

Two-client matrix green on fake relay; duplicates/reorder/retry/gap covered; no production network; local-only default unchanged.
