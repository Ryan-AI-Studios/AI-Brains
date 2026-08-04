# T204 Plan — CLI Help Information Architecture

Status: **Completed** 2026-08-04 (PR #87 `c3a7d66`). Spec: [spec.md](./spec.md).

## Absorbed from deferred / prior tracks

| Residual | Disposition |
|----------|-------------|
| CLI help grouping IA | **This track** F1–F7 |
| T202 F32 progressive after_help `--project-id` | **Required** F9 / AC6 |
| T202 F32 OutputFormat TTY matrix | **Soft F8** — consolidated CAPABILITIES table; no default flip |
| T203 F29 / list after_help | **Verify only** (F10/F28) |
| clap#1553 multi-heading | **Out** — clap 5 residual |
| man / help topics | Soft F12 |

## AI fold-in (2026-08-04)

| ID | Action in plan |
|----|----------------|
| **M1** | §5 Daily includes `stop-session`; AC11; inventory 37 |
| **M2** | B4: F31 bands Setup 0–9 … Harness 50–59; Graph both cfg same order |
| **M3** | B3: implement §12.1 draft text; AC1 exact labels |
| **M4** | C3: **one** consolidated OutputFormat defaults table in CAPABILITIES |
| **M5** | B3: `after_help` = **one-line** tip only; groups in `after_long_help` |
| **L7/L8** | B5: markers on subcommand enums (wipe/apply/encrypt/rotate/install/…) |
| **L2** | Soft B7: `next_help_heading` Global options |
| **L1** | Do **not** use flatten_help for grouping |
| F28 | B6 verify-only |

## Research applied

| Source | Use |
|--------|-----|
| clig.dev | F5–F7 start-here |
| clap 4.6: after_long_help, display_order, after_help dual surface | F5/M5 |
| clap#1553 open | F2 |
| AI2 live inventory | M1 StopSession; F9 gap real |

## Phases

### A0 — Inventory & freezes

- [x] Online research + expand F1–F30  
- [x] **AI fold-in** → F1–F36 + AC11–AC12 + §12 drafts  
- [x] On go: `ledgerful doctor`; `ledgerful ledger start T204-CliHelpIA --category DOCS --message "CLI help groups + F32 residuals"`  
- [x] On go: `ledgerful scan --impact` after first edit set  

### A1 — Help IA core (Red → Green)

- [x] **B1** Red: long help must contain exact labels `Daily`, `Operator`, `Governed`, `Dangerous`, `Harness` (AC1/AC4); soft `Setup`  
- [x] **B2** Red: dangerous marker for erase/rotate/apply class (AC2)  
- [x] **B3** Green: `ROOT_AFTER_LONG_HELP` from **spec §12.1**; short **one-line** `after_help` tip (**§12.2** / F5/M5)  
- [x] **B4** Green: `display_order` per **F31** (§12.3); within-group §5 order; **both Graph cfg** same (M2)  
- [x] **B5** Green **subcommand markers (F33/L7/L8):**  
  - top-level: `forget`, `erasure`  
  - `ErasureCommands::Wipe`  
  - `RetentionCommands::Apply`  
  - `VaultCommands::Encrypt`, `RotateDatakey`  
  - `MigrateCommands::Governed` (confirm-path honesty / about)  
  - `DaemonCommands::Install`, `Uninstall`  
  - Appendix Dangerous lines cover dual-ops  
- [x] **B6** F28/F10: **verify** evidence/source/review list after_help — edit only if drift  
- [x] **B7** Soft: root `next_help_heading = "Global options"` (L2)  

### B — F32 residuals

- [x] **C1** Parent `query` + `progressive` + `expand` after_help: `--project-id` and/or `AI_BRAINS_PROJECT_ID` (F9/AC6)  
- [x] **C2** Soft: briefing project-id line if missing  
- [x] **C3** CAPABILITIES: short help-IA note + **consolidated table** (M4/AC8), e.g. rows:  
  | Surface | Default TTY | Default non-TTY | Notes |  
  | recall | pretty | json | T101 |  
  | preflight | pretty/human | json | |  
  | briefing | markdown | json | T202 F9 |  
  | progressive/expand/trace | json | json | no TTY flip |  
  | source/list/show (governed) | json (Human render if `--format human`) | json | `OutputFormat::parse` |  
  | doctor | human | human | `--json` override |  
- [x] **C4** Soft: `parse_with_tty` skipped (F8)  

### C — Docs + closeout

- [x] **D1** CONTRIBUTING: help groups pointer (CLI-EXIT-CODES already linked) — AC3  
- [x] **D2** CAPABILITIES (+ OPERATIONS soft); CHANGELOG minor — AC8/AC9  
- [x] **D3** deferred help-IA strike complete; conductor Completed; README series complete  
- [x] **D4** Primary CLEAN; Codex R1 FAIL→fix; Claude final **PASS**  
- [x] **D5** Full gate 2020; PR #87; ledger TX; CI green; squash-merge `c3a7d66`  

## Test plan (minimum)

| Lock | Assert |
|------|--------|
| AC1/AC4 | long help labels Daily/Operator/Governed/Dangerous/Harness |
| AC2 | dangerous marker class |
| AC5 | recall/doctor/erasure still parse |
| AC6 | query progressive/expand help has project-id ceremony |
| AC7 | Daily before evaluate/dogfood |
| AC11 | stop-session in Daily inventory text |
| Soft AC10/AC12 | manual -h compact; wipe/apply markers |

Prefer: `Cli::command().render_long_help()` unit + optional hermetic bin.

## Manual checklist

- [x] `ai-brains --help` — groups + Start here  
- [x] `ai-brains -h` — one-line tip only (no full wall)  
- [x] `ai-brains query --help` / `query progressive --help` — project-id  
- [x] `ai-brains erasure wipe --help` — dangerous  
- [x] `ai-brains retention apply --help` — dangerous  
- [x] `ai-brains vault rotate-datakey --help` — dangerous  
- [x] `ai-brains source --help` — list examples still present  

## Order / parallel

Single-agent (`main.rs` + nested enums). Optional extract `help_ia.rs` for const only.

## Stop-before

- Command rename → halt (F1)  
- clap 5 multi-heading as DoD → residual only  
- OutputFormat default flip without audit → halt  

## Done when

AC1–AC9 + AC11 green; AC10/AC12 soft; series closed; gate green; review clear (or deferred lows documented).
