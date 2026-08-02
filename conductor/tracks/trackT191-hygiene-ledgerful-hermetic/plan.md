# T191 Plan — Hygiene Batch (Ledgerful rename + hermetic long-tail)

Status: **Implemented on branch** (2026-08-02). Awaiting final Codex + PR.  
Spec: [spec.md](./spec.md) (F1–F24, AC1–AC11). Disposition: spec §15.

## Preconditions

- [x] Product SOOT: **Ledgerful** (ChangeGuard product rename already shipped)  
- [x] Live inventory + AI fold-in (incl. **M1** `query_symbols_from_changeguard`)  
- [x] Expand freezes F1–F24  
- [x] A0: hard rename `LedgerfulVerificationBackend` (no deprecated alias — workspace-only)  
- [x] `ledgerful doctor` + `scan --impact` at implement start  
- [x] `ledgerful ledger start T191-HygieneLedgerfulHermetic --category REFACTOR` (TX `c3db2dce-8a6b-4bfe-8a97-af7805c5491e`)

## License / deps

- [x] Zero new production deps  
- [x] assert_cmd remains workspace 2.2.x (2.2.2 OK via caret)  

---

## Phase A — Inventory + denylist

- [x] **A1** Re-count L13 `cargo_bin` (expect 25 / 5 files)  
- [x] **A2** Confirm dual-read constant home (`symbol_bridge` or shared)  
- [x] **A3** When helper touched: add `LEDGERFUL_TX_ID` + `CHANGEGUARD_TX_ID` to `AMBIENT_DENYLIST` (F9 / AC10)  

---

## Phase B — source_tag (TDD **before** write flip / renames)

- [x] **B1** Dual-read in `symbol_already_ingested` (legacy **or** new)  
- [x] **B2** Tests: legacy-only, new-only, no double-ingest  
- [x] **B3** Flip **write** to `ledgerful:symbol`  
- [x] **B4** Test new writes use new tag  
- [x] **B5** T167 preserve tests green; **keep** “no changeguard→ledgerful rewrite” comment  

---

## Phase C — Identifier rename batch

- [x] **C1** `LedgerfulHotspot` + safety.rs  
- [x] **C2** `LedgerfulVerificationBackend` + capture exports (**hard rename**, no alias)  
- [x] **C3** Rename map (complete):  
  - `query_ledgerful_verification`  
  - `query_ledgerful_risk_alerts`  
  - `query_ledgerful` / `_fallback`  
  - `query_ledgerful_bridge`  
  - `ingest_symbols_from_ledgerful`  
  - `refresh_ledgerful_index`  
  - **`query_symbols_from_ledgerful`** ← was missing (AI1 M1)  
  - `ingest_madr_from_ledgerful`  
- [x] **C4** nightly + retrieval + brain call sites  
- [x] **C5** Fixtures/comments named:  
  - `cozo_proxy` project_id `"Ledgerful"`  
  - `bridge_record_shape` JSON **+** assertion pair  
  - `intervention.rs:458` comment → `.ledgerful/`  
  - test fn `…_with_ledgerful`  
- [x] **C6** F23 closeout grep clean (allowed residuals only)  

---

## Phase D — Hermetic L13

- [x] **D1** `governed_surface.rs` (12)  
- [x] **D2** `cross_repo_bridge_smoke.rs` (8): hermetic_*; **F17b** drop `changeguard` binary fallback (or exception note)  
- [x] **D3** `nightly_madr_ingestion.rs` (3)  
- [x] **D4** `dogfood_compare.rs` / `evaluate_governed.rs` factories  
- [x] **D5** AC6: zero bare `Command::cargo_bin` in those five files  

---

## Phase E — Docs + deferred + gate

- [x] **E1** `briefings.rs:122` blend comment → Ledgerful-only; other non-archive nits  
- [x] **E2** Strike T142 #1–2 + T186 L13 in `deferred.md`  
- [x] **E3** CHANGELOG hygiene line  
- [x] **E4** Full gate (local: fmt+clippy green; nextest with ALLOW_ZERO_KEY=1; deny+audit)  
- [ ] **E5** Conductor ✅ (orchestrator after PR merge)  
- [ ] **E6** ledger commit + pin (orchestrator)  

---

## Out of scope checklist (remain unchecked as work)

- [ ] Archive purge  
- [ ] Remove `.changeguard/` discovery fallback  
- [ ] Force vault-wide tag backfill as DoD  
- [ ] doctor CLI  
- [ ] Deprecated type alias for VerificationBackend  
- [ ] Product feature work  

---

## Implement notes

1. **Ledgerful is the product** — renames are residual cleanup.  
2. **M1:** `query_symbols_from_changeguard` is easy to miss — include in C3 + F23 greps.  
3. Tags: dual-read before write flip (B before C).  
4. Hard rename capture backend; no alias.  
5. Hermetic denylist: add both TX_ID env vars when editing helper.  
