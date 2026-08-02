# T191 — Hygiene Batch (Ledgerful rename + hermetic long-tail)

- **Track ID:** T191-HygieneLedgerfulHermetic
- **Phase:** Post-T142 / post-T186 cleanup
- **Status:** 🔄 **In Progress / Implemented on branch** (2026-08-02; awaiting review + PR)
- **Depends on:** T142 product rename (**.ledgerful**, binary `ledgerful`, `LEDGERFUL_TX_ID`); T186 hermetic helper + inventory
- **Blocks / feeds:** Consistent **Ledgerful** naming in code; hermetic long-tail CLI tests; strike T142 #1–2 + T186 L13
- **Category:** REFACTOR / TESTING
- **Deferred absorbed:** T142 **#1** type/fn renames; T142 **#2** `source_tag` strategy; T186 **L13** long-tail `cargo_bin` (**25** sites / **5** files — AI1 re-count exact)
- **Not absorbed:** `conductor/archive/**` full purge (T142 #4); historical ledger rewrite; doctor CLI (#2); T189/T190 product crypto/path work; bridge wire protocol rename
- **Research date:** 2026-08-02
- **AI fold-in:** AI1 M1–M4, L1–L5 + AI2 §1–5. Disposition §15.
- **Ledger:** plan-only (no TX until implement)

## 0. Naming SOOT (user / product)

| Concept | Canonical name | Residual (this track) |
|---------|----------------|------------------------|
| Product / binary / state dir | **Ledgerful** / `ledgerful` / `.ledgerful/` | None for product surface (T97/T142 done) |
| Legacy state dir fallback | `.changeguard/` read-only discovery fallback | **Keep** for migration honesty (T142) — not a product rename target |
| Code identifiers still saying ChangeGuard | Debt | **Rename in T191** |
| Durable `source_tag` value `"changeguard:symbol"` | Historical identity string in vault events | **Migrate carefully** (F2) — not a free string replace |

**Invariant:** Do not reintroduce “ChangeGuard” as the product name in new docs or new write paths. Comments may say “legacy ChangeGuard alias” only where describing compatibility.

## 1. Objective

1. Rename remaining **public/internal Rust identifiers** still saying ChangeGuard → **Ledgerful** in one coordinated PR.  
2. Migrate `source_tag` write/dedup for symbol memories **without** breaking idempotency against already-ingested rows.  
3. Finish T186 **L13** hermetic migration for inventoried long-tail CLI tests (no new test framework).

## 2. Live baseline (re-scan 2026-08-02)

### 2.1 Product rename (already done — do not re-litigate)

| Asset | Status |
|-------|--------|
| CLI binary / shell-outs | `ledgerful` (T97) |
| State dir discovery | `.ledgerful/` first; `.changeguard/` fallback (T142) |
| Env | `LEDGERFUL_TX_ID` preferred; `CHANGEGUARD_TX_ID` deprecated alias (T142) |
| Deprecated path APIs | `find_changeguard_dir` / `extract_project_id_from_changeguard` → wrappers (keep until callers gone or keep deprecated forever) |

### 2.2 Identifier debt (T142 #1 — in scope)

| Identifier | Location (approx) | Target |
|------------|-------------------|--------|
| `ChangeGuardHotspot` | `cli/.../safety.rs` (internal only) | `LedgerfulHotspot` |
| `ChangeGuardVerificationBackend` | `capture/verification_gate.rs` + `lib.rs` export | `LedgerfulVerificationBackend` (**hard rename** — workspace-only consumers; F15) |
| `query_changeguard_verification` | `verification_gate.rs` | `query_ledgerful_verification` |
| `query_changeguard_risk_alerts` | `brain/intervention.rs` | `query_ledgerful_risk_alerts` |
| `query_changeguard` / `_fallback` | `retrieval/preflight.rs` | `query_ledgerful` / `_fallback` |
| `query_changeguard_bridge` | `retrieval/recall.rs` | `query_ledgerful_bridge` |
| `ingest_symbols_from_changeguard` | `cli/symbol_bridge.rs` | `ingest_symbols_from_ledgerful` |
| `refresh_changeguard_index` | `symbol_bridge.rs` | `refresh_ledgerful_index` |
| **`query_symbols_from_changeguard`** | `symbol_bridge.rs` (:33 call, :142 def) | **`query_symbols_from_ledgerful`** (AI1 **M1** — was missing from seed map) |
| `ingest_madr_from_changeguard` | `cli/nightly.rs` | `ingest_madr_from_ledgerful` |
| Deprecated discovery re-exports | `path/lib.rs` | Keep deprecated wrappers **or** remove if zero external callers after rename batch |

### 2.3 `source_tag` debt (T142 #2 — in scope)

| Site | Behavior |
|------|----------|
| `symbol_bridge.rs` write | `source_tag: Some("changeguard:symbol")` on new `MemoryPinned` |
| `symbol_already_ingested` | Equality check `== Some("changeguard:symbol")` only |
| T167 legacy import | **Preserves** source_tag; tests pin `changeguard:symbol` durability (no silent rewrite) |
| Risk | Flip write tag alone → re-ingest duplicates; flip check alone → never dedup old rows |

### 2.4 Hermetic L13 (T186 — in scope)

| File | ~`cargo_bin` sites (T186 inventory; re-count at implement) |
|------|-----------------------------------------------------------|
| `governed_surface.rs` | 12 |
| `cross_repo_bridge_smoke.rs` | 8 |
| `nightly_madr_ingestion.rs` | 3 |
| `dogfood_compare.rs` | 1 |
| `evaluate_governed.rs` | 1 |
| **Total** | **25** exact (AI1 re-scan); `smoke.rs` comment-only; `common/mod.rs` is helper def — excluded |

Helper SOOT: `crates/ai-brains-cli/tests/common/mod.rs` — `hermetic_bin` / `hermetic_vault` / `hermetic_cmd` / `hermetic_cmd_with_ids` + `AMBIENT_DENYLIST` (**11** keys today; expand F9). **No new framework.**

### 2.5 Fixture / comment sweep (in-scope, named)

| Item | Location | Disposition |
|------|----------|-------------|
| `LEDGERFUL_TX_ID` / `CHANGEGUARD_TX_ID` denylist | `tests/common/mod.rs` | **F9 — do when touching helper** (AI1 L3) |
| Fixture `project_id: "ChangeGuard"` | `graph/cozo_proxy.rs` tests; `contracts/tests/bridge_record_shape.rs` (JSON + assert pair) | **C5** rename to `"Ledgerful"` in lockstep (AI1 M4) |
| Doc “Ledgerful / ChangeGuard blend” | `contracts/src/briefings.rs:122` | **E1** → Ledgerful-only wording (AI1 M3) |
| Test comment “no .changeguard/ dir” | `brain/intervention.rs:458` | **C5/E1** → `.ledgerful/` (AI1 M2) |
| Intentional “no changeguard→ledgerful rewrite” | `control-plane/legacy_import.rs` (T167) | **Keep** — compatibility doc (AI1 L1) |
| Binary fallback `changeguard` | `cross_repo_bridge_smoke.rs` body | **F17b** drop fallback or one-line exception (AI1 L2) |

## 3. Research summary (2026-08-02)

| Source | Finding | T191 application |
|--------|---------|------------------|
| T142 / product SOOT | ChangeGuard → **Ledgerful** product rename already shipped | Rename **identifiers** only; keep legacy **paths/env/tags** compatibility where durable |
| T167 import | `source_tag` preserved; no changeguard→ledgerful rewrite on import | Dual-read layer must not break import tests |
| T186 hermetic | Shared helper + denylist; nextest process isolation | L13 = migrate long-tail to helper; still strip ambient |
| assert_cmd **2.2** (workspace) | crates.io max stable **2.2.2** (caret resolves) | Keep `2.2`; no forced bump (AI1 L4) |
| cargo-nextest | **0.9.140** min / current; process isolation | Hermetic still required for child env |
| Dual-read migration pattern | Accept old+new identity keys until backfill | **F2** preferred over big-bang rewrite |
| `change_guard_*` serde fields | **0** matches (AI1 L5) | F4 non-issue; hard type rename OK |
| Capture consumers of `ChangeGuardVerificationBackend` | Workspace-only; tests use trait `VerificationBackend` | **Hard rename; no deprecated type alias required** (AI1 L5; reject AI2 mandatory alias) |

## 4. Frozen decisions (F1–F24)

| ID | Decision |
|----|----------|
| **F1 — Product name** | Canonical product is **Ledgerful**. New user-facing strings and new Rust type/fn names use Ledgerful/ledgerful. No new “ChangeGuard” product branding. |
| **F2 — source_tag strategy (normative)** | **Dual-read + new-write flip:** (1) define constants `SOURCE_TAG_SYMBOL_LEGACY = "changeguard:symbol"` and `SOURCE_TAG_SYMBOL = "ledgerful:symbol"`; (2) **dedup** treats either tag as “already ingested”; (3) **new writes** use `ledgerful:symbol` only; (4) optional offline/one-shot backfill of old tags is **soft** (not required for AC if dual-read proven). **Do not** change T167 to rewrite tags on import. |
| **F3 — Single rename PR batch** | Type/fn renames land in **one** coordinated commit/PR (or stacked PRs that land together) — no multi-week half-renamed tree. |
| **F4 — Serde/JSON field names** | Live: **no** `change_guard_*` serde fields (AI1 L5). Internal Rust types free to rename. Hotspot CLI text already says “Ledgerful” — keep. |
| **F5 — Shell binary** | Production call sites use `ledgerful`; do not reintroduce `changeguard` binary spawns. |
| **F6 — Deprecated discovery APIs** | Keep `find_changeguard_dir` / `extract_project_id_from_changeguard` as `#[deprecated]` thin wrappers **unless** zero callers — then remove if safe. `.changeguard/` **directory** fallback stays. |
| **F7 — Hermetic helper only** | Long-tail uses `common::hermetic_*`; no new test harness; no full `env_clear`. |
| **F8 — L13 file set** | Migrate the **five** T186 files (**25** sites). Justified exceptions: one-line + owner. |
| **F9 — Denylist expand (DoD when helper touched)** | When touching `common/mod.rs` for L13, **add** `LEDGERFUL_TX_ID` + `CHANGEGUARD_TX_ID` to `AMBIENT_DENYLIST` (promoted from soft prefer — AI1 L3). |
| **F10 — No product behavior change** | Beyond naming + tag dual-read/write + hermetic env pinning — no feature work. |
| **F11 — Archive out of scope** | `conductor/archive/**` keep historical “ChangeGuard” spelling. |
| **F12 — Tests for tag** | RED/GREEN: legacy-only / new-only / mixed dedup; new write uses `ledgerful:symbol`. |
| **F13 — Capture independence** | No models/graph required for gate. |
| **F14 — Deps** | **Zero new production deps.** assert_cmd workspace **2.2.x** (2.2.2 max stable OK). |
| **F15 — Type rename (hard)** | **Hard rename** `ChangeGuardVerificationBackend` → `LedgerfulVerificationBackend` inside workspace. **No** deprecated type alias required (workspace-only; capture tests use trait). Optional alias rejected as noise unless external consumer appears before implement. |
| **F16 — Fixture project_id strings** | Rename pure fixtures: `cozo_proxy` tests + `bridge_record_shape` JSON **and** assertion in same commit (not free partial swap). |
| **F17 — Cross-repo test** | Rename test fn to `…_with_ledgerful`. |
| **F17b — Binary probe body** | Prefer **drop** `changeguard` binary fallback in `cross_repo_bridge_smoke` (ledgerful-only + skip if missing). If kept as intentional alias, one-line exception + owner in plan (F8). |
| **F18 — Docs / comments** | Non-archive: `briefings.rs` blend comment; `intervention.rs` test comment → Ledgerful/`.ledgerful`. **Keep** T167 “no changeguard→ledgerful rewrite” comment (intentional). |
| **F19 — Full gate** | fmt, clippy -D warnings, nextest workspace, deny, audit. |
| **F20 — Category** | REFACTOR/TESTING — cross-model optional (no SECURITY behavior change expected). |
| **F21 — Doctor residual** | **Not** absorbed. |
| **F22 — Design-before-write for tags** | Dual-read tests **before** write-side tag flip. |
| **F23 — Closeout grep set** | Forbidden residual patterns (prod, excl. allowed legacy): `ChangeGuardHotspot`, `ChangeGuardVerificationBackend`, `query_changeguard`, `query_symbols_from_changeguard`, `ingest_.*_from_changeguard`, `refresh_changeguard_index`, write-site only `"changeguard:symbol"` (legacy const + dual-read OK). |
| **F24 — Phase order** | Phase B (tags TDD) **before** Phase C (cosmetic renames). Hermetic D independent. |

## 5. Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | No production `ChangeGuardHotspot` / `ChangeGuardVerificationBackend` types |
| **AC2** | No production `query_changeguard*`, `query_symbols_from_changeguard`, `ingest_*_from_changeguard`, `refresh_changeguard_index` (except F6 discovery wrappers) |
| **AC3** | New symbol ingest writes `ledgerful:symbol`; dedup accepts legacy + new (F2/F12) |
| **AC4** | T167 source_tag preserve tests still green (no import rewrite); preserve-comment stays |
| **AC5** | L13 five files use `common::hermetic_*` or documented exceptions |
| **AC6** | Re-grep bare `Command::cargo_bin("ai-brains")` in the five long-tail files → **0** |
| **AC7** | Full gate green; no intentional product behavior change beyond F10 |
| **AC8** | deferred T142 #1–2 + T186 L13 struck |
| **AC9** | Non-archive product comments/docs not branding ChangeGuard as current (F18) |
| **AC10** | Denylist includes `LEDGERFUL_TX_ID` + `CHANGEGUARD_TX_ID` if helper touched (F9) |
| **AC11** | Closeout grep F23 clean |

## 6. Non-goals

| Out of scope | Why |
|--------------|-----|
| Purge `changeguard` from archive tracks | Historical record (T142 #4) |
| Remove `.changeguard/` discovery fallback | Breaks legacy machines |
| Force-migrate all vault DBs offline | Dual-read sufficient for v1 |
| Doctor CLI | Separate residual |
| assert_cmd major bump | Unrelated |
| Rewriting ledgerful ledger history | Immutable |

## 7. Affected crates / surfaces

| Area | Change |
|------|--------|
| `ai-brains-cli` | safety, symbol_bridge, nightly; tests long-tail |
| `ai-brains-capture` | verification_gate + exports |
| `ai-brains-retrieval` | preflight, recall |
| `ai-brains-brain` | intervention |
| `ai-brains-path` | deprecated wrappers only if touched |
| `ai-brains-control-plane` tests | source_tag fixtures dual-tag aware |
| `tests/common/mod.rs` | optional denylist keys |
| `deferred.md` | strike rows on ship |

## 8. Verification

```powershell
# Inventory greps (implement start + closeout)
rg "ChangeGuardHotspot|ChangeGuardVerificationBackend|query_changeguard|ingest_.*changeguard|changeguard:symbol" crates
rg "Command::cargo_bin" crates/ai-brains-cli/tests

cargo nextest run -p ai-brains-cli -p ai-brains-capture -p ai-brains-retrieval -p ai-brains-brain
cargo nextest run -p ai-brains-control-plane --test legacy_import
# Full gate at closeout
```

### Proof tests (names)

- `symbol_dedup__legacy_tag_only__no_double_ingest`
- `symbol_dedup__new_tag_only__no_double_ingest`
- `symbol_ingest__writes_ledgerful_symbol_tag`
- `legacy_import__preserves_source_tag_unchanged` (existing; keep)
- Hermetic: long-tail suites pass under polluted ambient env (T186 pattern)

### Manual evidence

1. Grep clean for rename targets (or alias list).  
2. Nightly/symbol ingest dry path if available.  
3. Long-tail nextest files green.

## 9. Handoffs

| To | What |
|----|------|
| deferred T142 #1–2 | Strike |
| deferred T186 L13 | Strike |
| doctor #2 | Remain |
| T142 #4 archive purge | Remain optional |

## 10. Definition of Done

AC1–AC11; plan checked; conductor ✅; deferred updated.

## 11. Risks

| Risk | Mitigation |
|------|------------|
| Tag flip doubles symbols | Dual-read first (F22); tests F12 |
| Missed rename (e.g. `query_symbols_from_changeguard`) | **F23** closeout grep set |
| Hermetic breaks intentional ambient tests | Classify per-file; exceptions documented |
| Public type break | F15 hard rename — workspace-only verified |

## 12. Verification greps (normative closeout)

```powershell
# Renames (expect only allowed: legacy const, dual-read, .changeguard path, deprecated discovery, T167 comment, archive)
rg "ChangeGuardHotspot|ChangeGuardVerificationBackend|query_changeguard|query_symbols_from_changeguard|ingest_.*_from_changeguard|refresh_changeguard_index" crates

# Long-tail bare cargo_bin (expect 0 in the five files)
rg "Command::cargo_bin" crates/ai-brains-cli/tests/governed_surface.rs crates/ai-brains-cli/tests/cross_repo_bridge_smoke.rs crates/ai-brains-cli/tests/nightly_madr_ingestion.rs crates/ai-brains-cli/tests/dogfood_compare.rs crates/ai-brains-cli/tests/evaluate_governed.rs
```

## 15. AI fold-in disposition (2026-08-02)

### 15.1 Agreed → folded

| Source | Item | Fold |
|--------|------|------|
| **AI1 M1** | `query_symbols_from_changeguard` missing | §2.2 table, F23, AC2, plan C3, §12 greps |
| **AI1 M2** | intervention test comment | §2.5 + F18 / C5 |
| **AI1 M3** | briefings blend doc | §2.5 + F18 / E1 |
| **AI1 M4** | cozo_proxy + bridge_record_shape fixtures | F16 + C5 named |
| **AI1 L1** | Keep T167 preserve comment | F18 keep + AC4 |
| **AI1 L2** | changeguard binary fallback body | **F17b** |
| **AI1 L3** | TX_ID denylist | **F9** promoted when helper touched; AC10 |
| **AI1 L4** | assert_cmd 2.2.2 | §3 research only |
| **AI1 L5** | No serde `change_guard_*`; hard rename safe | F4, F15 |
| **AI2 §2** | Dual-read constants pattern | Already F2 — reaffirmed |
| **AI2 §3** | Hermetic 5 files + denylist | F8, F9 |
| **AI2 §4** | `.changeguard/` fallback + wrappers | F6 — keep |

### 15.2 Softened / partial

| Source | Item | Disposition |
|--------|------|-------------|
| AI2 §1 mandatory deprecated type alias | Prefer hard rename | **F15** no alias unless external consumer found at implement |

### 15.3 Rejected

| Source | Item | Why |
|--------|------|-----|
| Mandatory `pub type ChangeGuardVerificationBackend = …` | AI2 | Workspace-only; alias adds dual public names without consumers |

### 15.4 Net freezes

**F1–F24**, **AC1–AC11**. Only medium that could miss a call site at closeout was **M1** — now in inventory + greps.
