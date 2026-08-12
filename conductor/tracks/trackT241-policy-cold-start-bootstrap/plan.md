# T241 Plan — Policy cold-start bootstrap

**Status:** ✅ **Completed** (2026-08-12) PR #151 squash `930d0ed`  
**Spec:** [spec.md](./spec.md) F0–F32 / AC1–AC14 + §12 AI fold-in  
**Category:** FEATURE / UX / GOVERNED  
**Ledger:** FEATURE T241-policy-cold-start committed; closeout DOCS

---

## AI fold-in (2026-08-12) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. Spec design affirmed. Three AI1 mediums are **must-fold** before go; AI2 mediums restate/affirm design with concrete shapes.

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** F6 fail_usage vs clap | AI1 | **Agree hard** | F6/F30 — catalog message; no `required arguments were not provided` |
| **AI1 M2** StorePorts in doctor | AI1 | **Agree hard** | F1b construction path |
| **AI1 M3** denial_hint sites | AI1 | **Agree hard** | F7 + site list below; empty_denied → None |
| **AI1 L1** shared DISCOVERY_CAPS | AI1 | **Agree** | F6b → `governed_common` |
| **AI1 L2** preflight arity | AI1 | **Agree hard** | F3 post-hoc append |
| **AI1 L3** warn → Degraded | AI1 | **Agree** | F1 |
| **AI1 L4** insert point | AI1 | **Agree** | F2 |
| **AI1 L5** short SOOT | AI1 | **Agree** | F14 short/long |
| **AI1 L6** CLI-set next_step | AI1 | **Agree** | F5 |
| **AI1 L7** CAPABILITY_CATALOG | AI1 | **Agree** | F6b |
| **AI1 L8–L12** | AI1 | **Agree** | F15–F17, F27, F6 |
| **AI1 O12** hard cross-model | AI1 | **Agree hard** | F24 |
| **AI1 O11** live sequence | AI1 | **Agree** | F25 |
| **AI2 M1** authoritative skip | AI2 | **Agree** | F16 |
| **AI2 M2** `<3` warn | AI2 | **Agree hard** | F1/F31 (fixed from “zero only”) |
| **AI2 M3** contracts fields | AI2 | **Agree** | F5/F7/F19 |
| **AI2 M4** check catalog | AI2 | **Agree** | F6 |

### Pins locked by fold-in

1. **F1/F31:** warn when discovery `active_count < 3` (partial = incomplete).  
2. **F1b:** `StorePorts::from_store(SqliteEventStore::new(vault_conn.clone()))` — no AppContext.  
3. **F3:** post-hoc grants line; keep 9-arg formatters.  
4. **F6/F30:** `fail_usage` catalog; hermetic forbids clap required-arg text.  
5. **F7:** `empty_denied` → `denial_hint: None`; CP callers set `Some(...)`.  
6. **F14:** short SOOT default; long only on doctor rem.  
7. **F24:** hard cross-model.

---

## Preflight (plan time — 2026-08-12)

| Check | Result |
|-------|--------|
| `ai-brains preflight --summary` | Scope test-alias; **no grants line** |
| `policy show` | `grants: []` exit 0; human `(none)` no next |
| `policy check` (no cap) | clap required-arg exit 2 |
| `policy bootstrap --dry-run` | `would_register` + 3× `would_issue` — **path works** |
| `briefing project` human | Denied + markdown next bootstrap |
| `briefing project` json | `denied: true`, **no `denial_hint`** |
| `query progressive "test"` | exit 3 + denial_hint bootstrap |
| `evidence list` | exit 3 + hint bootstrap |
| `doctor` | 14 checks; **no `policy_grants`** |
| Doctor StorePorts | None today — F1b must add (AI1 M2) |
| Briefing literals needing `denial_hint` | **11+ sites** (AI1 M3 list) |
| Preflight arity AC19 | 9 args — must not break (AI1 L2) |
| clap | Workspace **4.5**; lock ~4.6.1; crates.io 4.6.6 — no bump |
| Capture | Ungoverned preflight/recall independent of grants |
| Live mutate | **Not run** in plan-only |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Policy grants empty → governed dead-end | deferred.md | **DoD** F1–F7, F17 |
| T221 F12 doctor `policy_grants` | T221 soft / deferred | **DoD** F1–F2 |
| Guided bootstrap from doctor/preflight | placeholder F1 | **DoD** F1, F3 |
| `policy show` empty guidance | placeholder F3 | **DoD** F5 |
| `policy check` discoverability | placeholder F4 | **DoD** F6 |
| Briefing denied next-step residual | T227 markdown-only | **DoD** F7 JSON |
| AI1 M1–M3 implementation pins | AI fold-in | **DoD** F1b, F6/F30, F7 sites |
| T210 skill one-liner | T210 residual | **Soft F21** |
| Bootstrap success soft-resolve hermetic | T210/T226 residual | **Soft F22** |
| `preflight --install-grants` | placeholder F2 | **Soft F20** (not DoD) |
| Full admin / auto-init / interactive | T210 decline | **Not absorbed** |
| Progressive ranking | T243 | **Not absorbed** |

---

## Phases

### Phase 0 — Plan freeze

- [x] Live dogfood empty-grant vault (2026-08-12)
- [x] Full spec F0–F32 + AC1–AC14
- [x] Roll T221 F12 + deferred empty-grant row
- [x] Research clig.dev + clap pin
- [x] AI fold-in AI1+AI2 → F1/F1b/F3/F5–F7/F14–F17/F24–F25/F29–F32, §12
- [x] Conductor status → Planning
- [x] User **go** before production code **or** live `policy bootstrap` (non-dry-run)

### Phase 1 — Red (TDD)

- [x] Unit: doctor matrix 15 — `[12]=project_identity`, `[13]=policy_grants`, `[14]=integrity` (AC3 / AI2 O1)
- [x] Unit/hermetic: active_count 0 or 1–2 → `policy_grants` **warn** + rem contains `policy bootstrap` (AC1/F31)
- [x] Unit/hermetic: non-authoritative / no scope → **skip** (AC2)
- [x] Hermetic: `policy show` empty human + JSON `next_step` (AC4–AC5)
- [x] Hermetic: `policy check` missing capability → exit 2 + discovery names + **no** `required arguments were not provided` (AC6/F30)
- [x] Hermetic/unit: briefing denied JSON `denial_hint` (AC7)
- [x] Unit: preflight post-hoc grants line; 9-arg formatters still compile (AC9)
- [x] Regression: progressive deny exit 3 still (AC8) — existing hermetics

### Phase 2 — Green (doctor + policy show/check)

- [x] **F6b:** move `DISCOVERY_CAP_LABELS` + `CAPABILITY_CATALOG` to `governed_common`; short SOOT constant
- [x] **F1b:** `check_policy_grants` StorePorts path
- [x] Wire push between project_identity and integrity; capacity 15; all len sites (F29)
- [x] `run_show`: human short SOOT; `resp.next_step = Some(short)` when empty (F5)
- [x] `run_check`: `Option` capability; `if None { fail_usage(catalog) }` (F6)
- [x] Clap `capability: Option<String>` + after_help from CAPABILITY_CATALOG

### Phase 3 — Green (briefing + preflight)

- [x] Contracts: `next_step` on ScopeGrantsResponse; `denial_hint` on both packets
- [x] **F7 site list** — denial_hint None on non-denied; CP denied paths Some(bootstrap)
- [x] Bootstrap hint constant `BRIEFING_DENIED_DENIAL_HINT` in control-plane
- [x] Preflight: post-hoc append grants line + optional JSON field (F3) — **no** arity change

### Phase 4 — Docs + contracts honesty

- [x] CAPABILITIES doctor table: `policy_grants` after `project_identity`, warn, scope-coupled (F18)
- [x] OPERATIONS/INSTALL: open vault → bootstrap dry-run → bootstrap → list/briefing
- [x] CHANGELOG minor
- [x] Contracts E1: `next_step` / `denial_hint` omit rules

### Phase 5 — Manual + gate (on go)

- [x] Live sequence F25 (record each exit code) — debug binary 2026-08-12:
  1. `policy bootstrap --dry-run` → exit **0** (would_register + 3× would_issue)
  2. `policy bootstrap` → exit **0** (registered + 3× issued)
  3. `policy show` → exit **0**, 3 grants, no `next_step`
  4. `briefing project --format json` → exit **0**, `denied: false` (empty_authority)
  5. `evidence list` → exit **0**
  - Also: doctor `policy_grants` empty→warn then after→ok (3 of 3); show empty next_step; check catalog exit 2
- [x] Soft F22: soft-resolve bootstrap omit --scope worked (authoritative PROJECT_ID)
- [ ] `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo nextest run --workspace ; cargo deny check ; cargo audit`
- [ ] `ledgerful verify --scope full` (or fast then full)
- [ ] Review log; **hard** cross-model (F24)
- [ ] Conductor Completed; deferred.md close; pin decisions

---

## Implementation notes

### F6 fail_usage catalog shape (AI1 O1)

```text
--capability is required. Valid capabilities:
  ReadEvidence (discovery)
  ReadConclusions (discovery)
  ReadDecisions (discovery)
  ApproveConclusion
  ApproveDecision
  Erase
  Export
  ProposeConclusion
  ProposeDecision
```

(Order: discovery first, then remaining catalog order per F6b/F26. Exact remaining sort free as long as stable and hermetic asserts discovery names present.)

### Doctor skip vs warn

| Condition | Severity |
|-----------|----------|
| Vault open-failed / no conn | skip |
| Scope resolve fail / non-authoritative | skip |
| Authoritative + active_count 0–2 | **warn** → Degraded |
| Authoritative + active_count 3 | ok |

### Contracts additive shapes

```json
// policy show empty
{ "api_version": "1", "grants": [], "next_step": "next: run `ai-brains policy bootstrap --dry-run` then `ai-brains policy bootstrap`" }

// policy show non-empty — no next_step key
{ "api_version": "1", "grants": [ … ] }

// briefing denied
{ "denied": true, "denial_reason": "…", "denial_hint": "…policy bootstrap…", … }
```

### Stop-before

- Live mutating bootstrap only after **go**
- No auto-init grants
- No force doctor **Fail** solely from empty/incomplete grants
- No production `unwrap` on capability Option

---

## Soft residuals after ship (if not free)

| ID | Item |
|----|------|
| F20 | `preflight --install-grants` opt-in |
| F21 | Skill one-liner |
| F22 | Bootstrap success soft-resolve hermetic |
| AC14 | Explicit partial-grant hermetic if not covered by AC1 |
| — | T243 progressive ranking |
| — | T249/T250 doctor/preflight presentation density |

---

## Evidence log (fill on go)

| Step | Command | Result |
|------|---------|--------|
| Plan dogfood | see Preflight table | 2026-08-12 |
| AI fold-in | AI-review.md AI1+AI2 | 2026-08-12 |
| Red | unit/hermetic AC1–AC9 written with green | 2026-08-12 |
| Green | doctor/policy/show/check/briefing/preflight/docs | 2026-08-12 |
| Live F25 | dry-run 0 → bootstrap 0 → show 3 grants → briefing denied:false → evidence list 0 | 2026-08-12 |
| Cross-model | Internal R2 CLEAN; Codex CX1/CX2 FAIL→fix→**CX3 PASS** | 2026-08-12 |
| Gate | nextest 2692; fmt/clippy/deny/audit; CI Win/Linux/macOS green PR #151 | 2026-08-12 |
