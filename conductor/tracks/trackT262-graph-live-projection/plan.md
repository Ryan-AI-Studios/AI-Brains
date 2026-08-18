# T262 Plan — Graph live projection

**Status:** **Completed** (implement-track)
**Spec:** [spec.md](./spec.md) F0–F35 / AC1–AC19 + §13 fold-in
**Category:** FEATURE / BUGFIX
**Ledger TX (planning):** `41977238-d85e-4e8a-bc80-3baba4937c90` (DOCS)
**Ledger TX (fold-in):** `4ea8db83-618e-4b28-af8b-4bc5e7b886d7` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## AI fold-in (2026-08-17) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. OpenCode **O2** folded as AC2 `node_kind(event_id) == None`. OpenCode **O1** / Agy **O1** folded as **AC19** (`vault_memory_present` Err → F1b). Disposition in spec **§13**.

### Pins locked by fold-in

1. **AC2 / F9:** `Some(turn_id)` path asserts no turn node at `envelope.event_id`.
2. **F18 / AC19:** never `?` `memory_exists` on graph reads.
3. **§2.1:** `da785c1` product vs `f58b4a9` plan.
4. **AC18:** payload literals `turn_id: None`.

---

## Preflight (plan time — 2026-08-17)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `da785c1`. Plan commit `f58b4a9`. Fold-in docs on that product src. |
| T262 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (2026-08-17 18:20). Pre-T260 (`--symbols` unknown). Graph-on T246 pretty. **Do not `cargo install`.** |
| Source debug | `target\debug\ai-brains.exe` (2026-08-17 21:32) — newer (T261). Projector/pin/hook same hole. |
| Live hole | Hook **works** for recall `MemoryPinned` (`7c3634fe` → `in RECALLS`). Pin path emits capture events **without** `turn_id`; `TurnProjection` uses `MemoryId::new()`; projector `DefaultHasher` → `kind=turn`. Neighbors/hierarchy missing-node next is still `graph update`. `graph update` human: 21477 / 1363 / E/N **0.063** / remediation **rebuild**. |
| SoT | `projector.rs` capture arms; `turn.rs:61`; `pin.rs` print + `StoreSink`; `PRETTY_NEXT` in `graph.rs:12`. |
| clap / rusqlite / serde_json | lock clap **4.6.1** / crates.io **4.6.6**; rusqlite **0.39.0**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute. |
| Last PR Cursor | #176 comments/reviews/inline **0**. Dependabot only. **N/A.** |
| `deferred.md` | Full scan. Overlap: audit T262 **absorb**; T213 projector half **absorb** / auto-rebuild **decline**; T246 F18 **absorb** / F3 supersede; T147 #10 **partial**; T267+ **decline**. |
| ai-brains | `preflight --summary` ok (3581317d / 2902). Recall PATH still stubs. No prior pin-id pin. |
| ledgerful | doctor ready (hygiene warns). 0 pending at start. Hotspot **#1** `project.rs` — do not touch. Index incremental completed. |
| Research | Rust `DefaultHasher` not durable; Azure event sourcing: replayable ids from events; T213 typed-sparse; CLIG next-action honesty. |
| `ISSUES.md` | **Does not exist** |
| Live rebuild / `.env` / nightly | **Not run** / **not written** / **not scheduled** this pass. |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Sparse graph / 4h pin no node / neighbors 4/5 / hierarchy 3/4 | audit T262 | **DoD** F1–F12 / AC1–AC19 |
| T246 F18 projector completeness | T246 soft | **DoD** F6–F11 / AC2 / AC6–AC7 |
| T213 projector more edges | T213 “not T213” | **Partial** F9 (typed RECALLS from capture). No invented edges. |
| T246 F3 `next: graph update` | T246 AC3 | **Supersede** F1 / F31 / AC8–AC9 |
| T88 print turn_id | T88 | **Absorb** F12 |
| T147 #10 random turn memory_id | T147 | **Partial** F8/F27 (`None` stays random) |
| T213 auto rebuild / default-on / WCC | T213 | **Decline** F4/F16 |
| T213 F31 freshness | T213/T246 F19 | **Decline** (soft) |
| T267 harness/whoami/list | audit | **Decline** F30 |
| T263–T271 / T240 F2 / T255 | deferred / standing | **Decline** |
| last-PR Cursor | #176 | **N/A** — no leftover to mint |
| `DefaultHasher` as node id | live `projector.rs` | **Absorb** F10 / AC3 |
| AC2 no turn node | OpenCode O2 | **Absorb** AC2 `node_kind(event_id) == None` |
| `memory_exists` Err unit | Agy O1 / OpenCode O1 | **Absorb** F18 helper + AC19 |

---

## Phase 0 — on go (re-verify)

- [x] Re-read `projector.rs` capture + `MemoryPinned` arms. Confirm hasher still live; no `turn_id` match.
- [x] Re-read `turn.rs` `MemoryId::new()` and `pin.rs` print + `StoreSink` hook apply.
- [x] Re-read `graph.rs` `PRETTY_NEXT` and T246 empty-pretty unit.
- [x] Classify-only: `graph update --format human` (21498/1386 E/N 0.064, remediation rebuild); neighbors of `7c3634fe-…` still `in RECALLS` (hook). Did **not** rebuild. Did **not** pin to the live vault. Debug bin is graph-off; used PATH graph-on.
- [x] Re-check lock clap **4.6.1** / crates.io **4.6.6**. rustc **1.95.0**. No clap 5.
- [x] Rescan **entire** `conductor/deferred.md` — no new open graph-projection rows beyond the planned T262 absorb.
- [x] Last merged PR #176 comments/reviews/inline **0**. Dependabot only. **N/A.**
- [x] `ledgerful ledger start T262-graph-live-projection --category FEATURE` → `f71cb7ac-8710-4120-8828-1817da6ee5fc`

---

## Phase 1 — Red (failing tests first)

- [x] Events AC1 serde units (`turn_id` missing / present) — names in spec §7.
- [x] Store AC4/AC5 turn projection units.
- [x] Graph AC2/AC3 projector units (AC2 includes `node_kind(event_id) == None`).
- [x] CLI AC8/AC9/AC10 pretty + JSON units (replace T246 `…graph_update` asserts).
- [x] CLI AC19 `vault_memory_present(Err(_)) == false` + unknown copy.
- [x] Hermetic graph-on AC6/AC7 pin → neighbors (tempdir; `--features graph`).
- [x] Confirm red: hermetic AC6/AC7 on pre-green bin: printed id `neighbors:[]` + pretty `next: graph update`.

---

## Phase 2 — Green

- [x] F6 payload field + F32 serde defaults.
- [x] F7 capture `build_*` set `Some(request.turn_id)`.
- [x] F8 `TurnProjection` branch.
- [x] F9/F10 projector (memory+RECALLS vs event_id turn node).
- [x] F1/F18/F29 pretty helpers + `vault_memory_present(ctx.conn.memory_exists(id))` on miss only (never `?`).
- [x] F12 pin comment.
- [x] Compile-fix ~15 payload literals with `turn_id: None` (AC18).
- [x] No `unwrap`/`expect`/`panic` in production. Graph apply stays non-fatal.

---

## Phase 3 — Docs + keep-green

- [x] F25 CAPABILITIES / OPERATIONS / PROTOCOL-COMPAT / CHANGELOG.
- [x] Skill one-liner only if the graph section lacks pin-id honesty.
- [x] Keep green: T74 update JSON; T198/T222 feature-off; T213/T232 density; T246 format/sort/limit (except superseded AC3); capture independence; live_graph MemoryPinned unit; AC14.

---

## Phase 4 — Verify

- [x] `cargo fmt --check`
- [x] `cargo clippy -p ai-brains-events -p ai-brains-capture -p ai-brains-store -p ai-brains-graph -p ai-brains-cli --all-targets --features graph -- -D warnings` (adjust feature matrix if a crate has no `graph` feature — events/capture/store clippy without it).
- [x] Targeted nextest: events + store turn + graph projector + cli graph + hermetic pin neighbors.
- [x] Full workspace gate only at finalize (implement-track), not this plan pass.
- [x] `ledgerful verify --scope fast` after FEATURE edits (implement). Fast nextest timed out (600s lock contention); `dev-check.ps1` + `ledgerful verify --scope full` both passed.
- [x] AC15 classify-only live neighbors on a known recall id. **No** live rebuild (F23). PATH graph-on: `7c3634fe` still `in RECALLS`.

---

## Phase 5 — Close (implement-track, not this skill)

- [x] FEATURE TX commit + `review.md` + `codex-review` (F20).
- [ ] conductor Completed only after implement + merge.
- [x] Append soft residuals to `deferred.md`.
- [ ] Pin: `DECISION: pin turn_id is the graph memory id; graph update is not a remediator.`
- [ ] Publish: branch → PR → GHA green → squash-merge → prune. Never `git push origin main`.

---

## Definition of done

- [ ] AC1–AC19 green or explicitly Phase-0 retargeted in spec (not silently dropped).
- [ ] F0–F35 honored (declines stay declined).
- [ ] No live `graph rebuild`. No historical memory_id remint.
- [ ] T213 floors unchanged. T246 JSON keys unchanged.
- [ ] Medium+ review findings not silently dropped.
- [ ] Registry stays **Pending** until implement-track marks Completed.

---

## Stop-before

- Live `graph rebuild` / nightly schedule mutate
- `cargo install` / silent `.env` / leftover rebind
- Scope exceeds this track (T263–T271, T240 F2, T255, prefix match, default-on)
- Ambiguous conflict with a new fold-in finding
