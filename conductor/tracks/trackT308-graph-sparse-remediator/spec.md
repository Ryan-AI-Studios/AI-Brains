# T308 — Sparse graph remediator after T300 rebuild

- **Track ID:** T308-GraphSparseRemediator
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / CLI
- **Owner:** Grok
- **Source:** T300 live rebuild still `sparse`; PATH doctor 2026-08-26: nodes=62965 edges=25726 **E/N=0.409** pinned=49286 memory_nodes=39326; remediator still **`ai-brains graph rebuild`**.
- **Depends on:** T213 floors; T232 remediator exact `ai-brains graph rebuild`; T300 rebuild UX. **Floors frozen.**
- **F0:** Plan-only until go. Do **not** retune `MIN_EDGE_NODE_RATIO`. Do **not** live rebuild as planning.

## 1. Objective

Stop looping **rebuild** as the doctor SOOT when verdict is **Sparse** after a successful projection (E/N improved, still below 0.50). Typed-lineage sparse is **honest**. Keep rebuild SOOT for empty_lag / orphan / projection_lag.

## 2. Live baseline (2026-08-26)

| Signal | Observation |
|--------|-------------|
| PATH doctor | `graph_density` **warn** sparse E/N **0.409**; `remediation: ai-brains graph rebuild` |
| Assessor | `graph_density.rs:214–226` Sparse: note uses `sparse_nuance` (“rebuild if projection lag suspected”) but **`remediation` is still `REMEDIATION_REBUILD`** (`:139–148`) |
| Floors | `MIN_EDGE_NODE_RATIO = 0.50` (`:14`). Env override exists; **do not change default**. |

**Research:** N/A product copy. T213/T232/T300: honest sparse; no auto-rebuild; remediator exact for lag/empty — not for “already rebuilt, still sparse.”

last-PR `#222` Cursor **empty**. **T310** minted from T306 plan (daemon + T84), not Cursor.

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Default floors **frozen** (0.50 / MIN_PINNED 100 / MIN_NODES 50 / coverage 0.10). |
| **F2** | Sparse: doctor `remediation` **must not** be `ai-brains graph rebuild`. Note may keep lag nuance. |
| **F3** | empty_lag / orphan / projection_lag keep rebuild SOOT (graph-on). |
| **F4** | No projector rewrite. No auto-rebuild. No live rebuild as DoD. |
| **F5** | JSON doctor keys frozen except `remediation` **value** for sparse. |
| **F6** | Never `git push origin main`. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Sparse fixture (e.g. nodes=100 edges=40): `remediation` ≠ `ai-brains graph rebuild`. |
| **AC2** | Orphan / empty_lag / projection_lag still remediate rebuild when graph-on. |
| **AC3** | `MIN_EDGE_NODE_RATIO` still **0.50** in src. |
| **AC4** | Live doctor (optional): sparse vault does not SOOT rebuild; or hermetic-only if owner skips live. |
| **AC5** | clippy + nextest `-p ai-brains-cli` graph_density / doctor. |

## 5–12

**Non-goals:** T278 floor retune; Cozo; forcing `live`; T306 install.

**§9:** Absorb T300 leftover “still sparse after rebuild.” Decline floor retune. last-PR `#222` N/A. PATH install → **T306**.

**Touch:** `graph_density.rs` + tests; maybe doctor pretty; CHANGELOG.

**Isolation:** No live `graph rebuild` unless owner later asks (not this DoD).
