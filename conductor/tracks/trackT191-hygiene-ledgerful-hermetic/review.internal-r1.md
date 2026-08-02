# T191 Internal Review R1 (read-only)

- **Branch:** `agent/T191-hygiene-ledgerful-hermetic` @ `dadd75dcfe797e961fa3c125f2401b71d79a1ed6`
- **Base:** `main` @ `c24a5922f3331b1a17da06360c07d832237f267e`
- **Commit (main..HEAD):** 1 — `refactor(T191): dual-read source_tag + Ledgerful identifier renames + hermetic L13`
- **Reviewer mode:** static audit (spec F1–F24 / AC1–AC11 + plan phases). No workspace edits outside this file. Full cargo gate not re-executed in this pass (plan E4 still open).
- **Date:** 2026-08-02

---

## Findings

```
[P3] deferred.md T142 #1–2 + T186 L13 not struck (AC8)
Confidence: High
Requirement: AC8 — strike deferred T142 #1–2 + T186 L13 on ship; plan E2
Location: conductor/deferred.md:15 (promotion table still “T191 Pending”); §1–§2 open prose; §64 residual item 1 (“Long-tail 25 cargo_bin…”) still open
Problem: Implementer plan assigns strike to orchestrator closeout (E2 unchecked). Code DoD is otherwise met, but track AC8 is not satisfied on the branch as reviewed.
Evidence: deferred.md L15 still maps residual to “T191 (Pending…)”; L821 still lists L13 long-tail residual without strike.
Correction: Orchestrator (or implementer if ship includes closeout) strike T142 #1–2 + T186 L13 rows; update promotion table to Closed by T191 with merge SHA.
Deferrable: Yes — plan E2 is explicit orchestrator closeout; not a code defect.
```

```
[P3] Full gate evidence not present on branch (AC7 / plan E4)
Confidence: Medium
Requirement: AC7 / F19 — fmt, clippy -D warnings, nextest workspace, deny, audit green
Location: plan.md E4 unchecked; no gate log committed under track folder
Problem: Static review cannot certify AC7. Plan leaves E4 open; this R1 did not re-run the workspace gate.
Evidence: plan.md lines 78–80: E4/E5/E6 open; no review.md gate appendix.
Correction: Run full gate before merge; record summary in review closeout. Block ship only if gate red.
Deferrable: Yes as process residual until orchestrator/implementer gate; treat red gate as non-deferrable if found.
```

```
[P3] Residual local test identifiers still say “cg_” (not F23-forbidden)
Confidence: High
Requirement: F1 branding hygiene (soft); F23 forbidden set does not include local vars
Location: crates/ai-brains-cli/tests/cross_repo_bridge_smoke.rs — `cg_export.ndjson`, `cg_init`, `cg_scan`, `cg_export_bootstrap.ndjson`
Problem: Cosmetic leftovers from ChangeGuard-era naming in test locals/fixture filenames. Binary probe itself is ledgerful-only (F17b correct). No production identifier debt.
Evidence: grep `cg_` under cross_repo_bridge_smoke.rs; `let binary = "ledgerful"` at L176–177 with F17b comment.
Correction: Optional rename to `ledgerful_init` / `lf_export.ndjson` in a follow-up nit; not DoD-blocking.
Deferrable: Yes
```

```
[P3] Track status headers still “planning only” after implement commit
Confidence: High
Requirement: Conductor honesty / plan status alignment (not a frozen F, process)
Location: conductor/tracks/trackT191-hygiene-ledgerful-hermetic/spec.md L5; plan.md L3
Problem: Spec/plan banners still say “Pending / Expanded … planning only — not implementing” while plan phases A–D and most of E are checked and a refactor commit exists on the branch.
Evidence: plan checkboxes B/C/D complete; git log shows implement commit dadd75d.
Correction: Orchestrator updates status to Implemented / In Review at closeout (E5).
Deferrable: Yes
```

---

## Positive evidence (no finding)

### Dual-read source_tag + write flip (F2 / F12 / AC3)

| Item | Evidence |
|------|----------|
| Constants | `SOURCE_TAG_SYMBOL_LEGACY = "changeguard:symbol"`, `SOURCE_TAG_SYMBOL = "ledgerful:symbol"` in `symbol_bridge.rs:13–15` |
| Dedup dual-read | `is_symbol_source_tag` matches either tag (`symbol_bridge.rs:254–259`); used by `symbol_already_ingested` |
| New write | `source_tag: Some(SOURCE_TAG_SYMBOL.to_string())` only (`symbol_bridge.rs:86`) — no write-site bare `"changeguard:symbol"` |
| Proof tests (named as specified) | `symbol_dedup__legacy_tag_only__no_double_ingest`, `symbol_dedup__new_tag_only__no_double_ingest`, `symbol_ingest__writes_ledgerful_symbol_tag` (`symbol_bridge.rs:421–476`) |

F12’s prose “mixed dedup” is covered by dual-read of either tag via the two no_double_ingest cases + write-tag assert; no extra named mixed test required by §8 proof list.

### Identifier renames (F3 / F15 / AC1 / AC2 / AC11)

| Target | Result |
|--------|--------|
| `LedgerfulHotspot` | `safety.rs` |
| `LedgerfulVerificationBackend` | `verification_gate.rs` + `lib.rs` export; **no** `ChangeGuardVerificationBackend` alias (F15 hard rename) |
| `query_ledgerful_verification` | capture |
| `query_ledgerful_risk_alerts` | brain/intervention |
| `query_ledgerful` / `_fallback` | retrieval/preflight |
| `query_ledgerful_bridge` | retrieval/recall |
| `ingest_symbols_from_ledgerful` | symbol_bridge + nightly call |
| `refresh_ledgerful_index` | symbol_bridge |
| **`query_symbols_from_ledgerful`** (M1) | symbol_bridge def + call |
| `ingest_madr_from_ledgerful` | nightly |

Closeout greps over `crates/**/*.rs` for F23 forbidden identifiers: **0** residual matches. Allowed residuals present:

- Legacy const / T167 fixtures: `"changeguard:symbol"`
- Path discovery `.changeguard/` fallback + deprecated `find_changeguard_dir` / `extract_project_id_from_changeguard` (F6)
- T167 preserve comments

### Hermetic L13 (F7–F9 / AC5 / AC6 / AC10)

| File | `hermetic_*` sites | bare `Command::cargo_bin` |
|------|--------------------|---------------------------|
| `governed_surface.rs` | 12 | 0 |
| `cross_repo_bridge_smoke.rs` | 8 | 0 |
| `nightly_madr_ingestion.rs` | 3 | 0 |
| `dogfood_compare.rs` | 1 (factory) | 0 |
| `evaluate_governed.rs` | 1 (factory) | 0 |
| **Total** | **25** | **0** |

- Remaining `Command::cargo_bin` only in `tests/common/mod.rs` (helper def) and a **comment** in `smoke.rs` — both out of L13 set.
- `AMBIENT_DENYLIST` includes `LEDGERFUL_TX_ID` + `CHANGEGUARD_TX_ID` (`common/mod.rs:56–57`) — AC10 met.

### F17b ledgerful-only binary probe

`cross_repo_bridge_smoke.rs:173–184`: binary fixed to `"ledgerful"`; skip if missing; comment documents no changeguard fallback. Test fn renamed `test_cross_repo_e2e_integration_with_ledgerful` (F17).

### Fixtures / comments (F16 / F18 / AC9 named sites)

| Site | Status |
|------|--------|
| `cozo_proxy` `project_id: "Ledgerful"` | OK (two fixtures) |
| `bridge_record_shape` JSON **and** assert `"Ledgerful"` | OK |
| `briefings.rs:122` | “Optional Ledgerful blend…” (no ChangeGuard product blend) |
| `intervention.rs:458` | “no .ledgerful/ dir” |
| T167 “no changeguard→ledgerful rewrite” | **Kept** in `legacy_import.rs:1131` and test comment `:592` |

### T167 preserve (AC4)

- `compose_evidence_summary` still records source_tag verbatim; no rewrite.
- `classify__preserves_source_tag_metadata` still pins `changeguard:symbol` and asserts no `ledgerful:symbol` rewrite in summary.
- Spec’s alternate name `legacy_import__preserves_source_tag_unchanged` is “(existing; keep)” — existing test name retained; behavior intact.

### Other freezes

| ID | Assessment |
|----|------------|
| F1 product name | New types/fns/docs brand Ledgerful |
| F4 serde | 0 `change_guard_*` fields |
| F5 shell binary | Production/test probes use `ledgerful`; no `Command::new("changeguard")` in crates |
| F6 discovery | Deprecated wrappers kept |
| F10 behavior | Naming + tag dual-read/write + hermetic only |
| F11 archive | Not purged (correct) |
| F13 capture independence | Gate still shells ledgerful; no models/graph required for capture path |
| F14 deps | assert_cmd remains workspace `2.2`; no new prod deps observed |
| F20–F22 | REFACTOR; dual-read tests present with write using new tag |
| F24 order | Single implement commit bundles B+C+D (acceptable F3 “one coordinated commit”) |

### Production `unwrap`/`expect`

Touched production paths (`symbol_bridge`, `safety`, `verification_gate` prod surface, renames in retrieval/brain) show no new production `unwrap()`/`expect()`. Test modules retain expect/unwrap under test allows (acceptable). `symbol_already_ingested` uses `.unwrap_or(false)` (non-panicking).

### Placeholders / stubs

No `todo!` / `unimplemented!` / FIXME stubs in T191-touched production surfaces.

### CHANGELOG

Hygiene line present under CHANGELOG for T191 dual-read + renames + hermetic L13.

---

## Verdict: PASS WITH DEFERRED P3

Code/implementer scope for F1–F18, F23 and AC1–AC6, AC9–AC11 is met with high confidence. Remaining gaps are process closeout (AC7 gate evidence, AC8 deferred strike, status banners) and cosmetic `cg_*` test locals — all **P3 / deferrable** per plan E2/E4–E6 ownership.

No P0/P1/P2 code defects found. Ship blocked only if full gate fails when run.

---

## DoD matrix (one row per AC)

| AC | Criterion | Status | Notes |
|----|-----------|--------|-------|
| **AC1** | No prod `ChangeGuardHotspot` / `ChangeGuardVerificationBackend` | **PASS** | `Ledgerful*` only; no alias |
| **AC2** | No prod `query_changeguard*` / `query_symbols_from_changeguard` / `ingest_*_from_changeguard` / `refresh_changeguard_index` (F6 wrappers OK) | **PASS** | Full rename map incl. M1; F6 deprecated discovery kept |
| **AC3** | New write `ledgerful:symbol`; dedup legacy+new | **PASS** | Constants + dual-read + 3 named tests |
| **AC4** | T167 source_tag preserve tests + comment | **PASS** | Comment + classify preserve test intact |
| **AC5** | L13 five files use `common::hermetic_*` | **PASS** | 25 hermetic sites across five files |
| **AC6** | 0 bare `Command::cargo_bin` in five files | **PASS** | Grep empty for those paths |
| **AC7** | Full gate green; no product behavior beyond F10 | **UNVERIFIED** | Plan E4 open; static review only this R1 |
| **AC8** | deferred T142 #1–2 + T186 L13 struck | **NOT MET** | Plan E2 orchestrator; residual text still open |
| **AC9** | Non-archive product comments not branding ChangeGuard as current | **PASS** | Named F18 sites fixed; archive out of scope |
| **AC10** | Denylist includes both TX_ID keys if helper touched | **PASS** | Both keys present |
| **AC11** | F23 closeout grep clean | **PASS** | Forbidden identifiers 0 in crates |

---

## Completeness

| Area | Complete? |
|------|-----------|
| Phase B dual-read + write flip + named tests | Yes |
| Phase C renames (incl. M1 `query_symbols_from_ledgerful`) | Yes |
| Phase C hard rename VerificationBackend (no alias) | Yes |
| Phase C fixtures/comments (cozo, bridge_record_shape, briefings, intervention, F17 fn) | Yes |
| Phase D hermetic five files + denylist TX keys | Yes |
| Phase D F17b ledgerful-only probe | Yes |
| T167 preserve still intact | Yes |
| F23 residual greps | Clean (allowed legacy only) |
| Production unwrap/expect regressions in touch set | None found |
| Placeholders/stubs | None found |
| deferred.md strike | No (orchestrator) |
| Full gate | Not evidenced this R1 |
| Conductor ✅ / ledger commit / pin | Out of implementer commit; plan E5–E6 |

**Net:** Implementation matches T191 frozen decisions for code and hermetic work. Ready for gate run + orchestrator closeout (deferred strike, conductor, ledger). Internal R1 does not require a code rework loop unless gate fails or a later reviewer finds a miss.
