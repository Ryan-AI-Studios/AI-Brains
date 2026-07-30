# T170 Plan — Shadow Dogfood Gate + Live Enablement Stop (P9.4)

Status: **Completed** (2026-07-30). Codex R2 PASS WITH DEFERRED P3 (Stage C/D operator/user deferred).

Authority: `spec.md` locks **D1–D26**. Process/docs-first. **No live enablement without explicit user approval.**

## Phase 0 — Preconditions

- [x] T168/T169 Complete on main.
- [x] Confirm `briefing project --format json` emits typed packet; `preflight --format json` is `{text,word_count}` only.
- [x] Confirm `resolve_live_vault_path` uses `AI_BRAINS_VAULT_PATH` — dogfood **must** use `--vault-path` for shadow compare (**D26**).
- [x] Confirm `BriefingWarningDto` has no `id` — use `warning_refs` (**D7**).
- [x] `scripts/shadow-vault.ps1` style reference for PS conventions.
- [x] evidence/.gitignore for `*.db`.
- [x] #40 / new crates out of scope.

## Phase A — Runbook + templates

- [x] `Docs/EVALUATION/SHADOW-DOGFOOD-GATE.md` — Stages A–D, D24–D26, §8 verification (**not** `--summary` for governed), Stage B `init`+ingest vault, Stage C re-evaluate, test-vault preferred.
- [x] Checklist template — claim id/kind; warning **kind+subject_id**; live sha256 pre/post.
- [x] OPERATIONS.md — dogfood link; flag rollback; **never** set `AI_BRAINS_VAULT_PATH` to shadow; emergency User-env clear snippet.
- [x] Cross-link GOVERNED-MEMORY-MVP.md → SHADOW-DOGFOOD-GATE.md.

## Phase B — Stage A linkage

- [x] Document evaluate exit 0/1/7 branching.
- [x] Capture baseline `report_hash` in evidence (synthetic).

## Phase C — Orchestrator script (recommended)

- [x] `scripts/dogfood-shadow.ps1` (`#Requires -Version 5.1`, `$ErrorActionPreference='Stop'`, `[CmdletBinding()]`).
- [x] Stage A evaluate; abort non-0; store report_hash.
- [x] **D24** live vault file SHA-256 pre/post (skip+note if no live vault).
- [x] Shadow/migrate only under WorkDir; **never** set process `AI_BRAINS_VAULT_PATH` to shadow.
- [x] Governed capture: `ai-brains briefing project --vault-path … --format json`.
- [x] Legacy capture: `ai-brains preflight --vault-path … --format json` flag off.
- [x] Emit/fill compare packet fields (or write partial JSON for human).
- [x] Refuse Stage D; print approval required.
- [x] Quality: `Invoke-ScriptAnalyzer` if available; else manual review vs shadow-vault.ps1.

## Phase D — Optional compare CLI

- [x] RED: compare_hash excludes created_at; warning_refs sort stable
- [x] GREEN: thin dogfood compare; zero new deps
- [x] Inputs: governed packet JSON + legacy preflight JSON

## Phase E — Stage B synthetic rehearsal (required)

- [x] Fixture vault: `init --vault-path` + pin events; persist `fixture-project-id.txt`; fail on pin non-zero (R1-01).
- [x] Stage B briefing passes `--project-id` from fixture file (R1-01); runbook §2 updated.
- [x] D24 pre/post hash (script); locked live vault → N/A warn, not abort; mismatch still fails when both readable.
- [x] Shadow (+ optional migrate) under WorkDir (script); `--migrate-report` wired into compare (R1-07).
- [x] Rollback drill **executed** on work `migrated.db`: flag 0 → no `(governed)`; flag 1 → `(governed)` in preflight JSON; `briefing project --project-id` for authority — **not** `--summary` (D21). See evidence/stage-b-notes.md.
- [x] D23 document User-env circuit breaker (manual only; script never sets User scope).
- [x] D20 re-run idempotency: evaluate report remove + `--allow-report-overwrite`; regenerate compare partials (R1-02).
- [x] BOM-less UTF-8 for governed/legacy JSON captures (R1-05).
- [x] Evidence: stage-b-notes.md + stage-b-compare-summary.json (hashes/counts only; no claim bodies).
- [ ] **Partial honesty:** fixture pin path yields `briefing denied=true` (no ReadDecisions grant / pin≠governed Decision). Pipeline + project_id wiring proven; richer authority sample deferred to Stage C / improved seed if needed.

## Phase F — Stage C (operator-dependent)

- [ ] Prefer **operator-provided test vault**; active user vault only with explicit note.
- [ ] Step 0: re-evaluate; report_hash match or document drift.
- [ ] Redacted shadow; D24; compare via §9 procedure; D15 stratified sample; human sign-off.
- [x] **Defer Stage C** with owner + reason (operator test vault not available in CI).

## Phase G — Stage D / closeout

- [x] Stage D: **deferred** (no live enablement approval).
- [ ] If enabled: **D25** observation (≥1 session or ≥3 governed invocations); record. — N/A while deferred
- [x] Rust gate if code added (`cargo clippy -p ai-brains-cli --all-targets -- -D warnings`; nextest dogfood/compare).
- [ ] ledgerful verify if code touched (orchestrator/finalize).
- [ ] Conductor → Completed; residuals → deferred (leave In Progress until R1 findings `verified_fixed`).
- [x] Pin candidates documented: D26 vault-path; D21 no-summary; D24 checksum; stop-before live.

## Out of scope

- [x] Auto live enablement / User env from scripts (refused)
- [x] Config-file governed flag
- [x] Making preflight --summary governed-aware (optional later polish)
- [x] #40, soft-canonicalize, desktop, sync

## Suggested commits

1. Docs runbook + checklist + OPERATIONS  
2. dogfood-shadow.ps1  
3. dogfood compare CLI  
4. Stage B evidence (sanitized)  

## Definition of Done

Mirror spec §14: runbook+checklist; A/B+rollback+D24; C done or deferred; D approval or deferred; live never mutated; D26/D21 locked in docs.
