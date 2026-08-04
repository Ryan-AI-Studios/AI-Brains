# T201 Plan — CLI Error Envelope + Exit Code Contract

Status: **In Progress** (implement 2026-08-03). Spec: [spec.md](./spec.md).

## Preconditions

- [x] Live re-scan (AI2 confirmed)  
- [x] Expand F1–F28  
- [x] **AI fold-in** M1–M7 + L2/L4/L9 + O3 — §14; F29–F36; AC3b/AC6a/AC12–13  
- [x] Deferred roll-in  
- [x] F36 scope Option inventory on go  
- [x] `ledgerful ledger start T201-CliErrorExitContract --category FEATURE` *(TX `1997ee74-e467-48b2-9bc0-e55a9e3023a8`)*  

## Deferred rolled in

| Item | Disposition |
|------|-------------|
| Exit + envelope residual | **Absorb** |
| clap 2 vs app 6 missing-scope | **Absorb** F3/F4 (+ erasure request) |
| POLICY_DENIED no hint | **Absorb** F6 structured |
| stderr docs bug | **Absorb** F8 |
| Graph exit 0 | **T198** + F18 4a lock |
| T203 soft-default | Boundary F27 — do not implement in T201 |
| --exit-codes flag | Soft deferred |

## Research pins (post fold-in)

| Fact | Pin |
|------|-----|
| Missing required | Exit **2** clap-required preferred |
| F4 sites | policy show, review list, **erasure request** + grep |
| Daemon None | **Keep** defensive |
| Deny hint | **`details.hint`** structured |
| F4 flip | **BREAKING** CHANGELOG |
| Hermetic | **≥6** locks incl graph 2 + vault 1 |
| Stream docs | CAPABILITIES + OPERATIONS required fix |
| Cross-model | Required after F4 |

## Phases

### Phase A — Inventory + docs draft

- [x] **A0** Ledger start  
- [x] **A1** F36: grep `scope: Option` → disposition table in plan notes  
- [x] **A2** Confirm F4 clap-required (preferred) vs USAGE alt — **clap-required preferred**  
- [x] **A3** Draft CLI-EXIT-CODES.md (full table + footnotes)  
- [x] **A4** AI fold-in M1–M7  

### Phase B — Code align

- [x] **B1** clap-required: policy show, review list, erasure request  
- [x] **B2** Remove CLI None → INVALID_PAYLOAD branches only  
- [x] **B3** Confirm daemon arms retained (M1) — `services.rs` list_review_items / request_erasure None arms **untouched**  
- [x] **B4** POLICY_DENIED `.with_details(hint)` (F6) — policy check + soft review list local  
- [x] **B5** Soft: INVALID_TRANSITION map arm → EXIT_INTERNAL=1  
- [x] **B6** If USAGE alt only: map arm + unit (skip if clap path) — **skipped (clap path)**  

### Phase C — Docs + tests

- [x] **C1** Ship CLI-EXIT-CODES.md  
- [x] **C2** AC6a: CAPABILITIES + OPERATIONS stream + exit list  
- [x] **C3** AC6b: CONTRIBUTING link  
- [x] **C4** `tests/exit_contract.rs` F18 suite  
- [x] **C5** CHANGELOG **BREAKING** (AC7)  

### Phase D — Gate + closeout

- [x] **D1** Full gate — `cargo fmt --check` OK; `clippy --workspace --all-targets -D warnings` OK; `nextest --workspace` **1931 passed** (1 skipped); `cargo deny check` OK; `cargo audit` OK (warnings only, exit 0)  
- [x] **D2** Cross-model review (F22/AC13) — Codex R1 product **PASS** (P1 process-only: gates/closeout); disposition in review.md; final Codex after PR green  
- [x] **D3** Review AC1–AC13 — internal CLEAN; Codex verified product AC; process residual closes with CI + closeout  
- [ ] **D4** deferred strike; conductor Completed *(after squash-merge)*  
- [ ] **D5** Pin + ledger commit *(after squash-merge)*  

## Verification matrix

| AC | Proof |
|----|-------|
| AC1, AC6a/b, AC10 | docs |
| AC2, AC3, AC3b, AC4, AC5, AC8, AC11 | hermetic |
| AC7 | CHANGELOG |
| AC9 | gate |
| AC12–13 | review process |

## Out of scope

- [ ] T203 soft-default  
- [ ] Remove daemon None arms  
- [ ] Force single envelope  
- [ ] --exit-codes flag  
- [ ] clap 4.6  

## Implement notes

1. **Order:** inventory → clap flips → deny hint → docs honesty → exit_contract suite → BREAKING changelog → cross-model → gate.  
2. **High findings:** missing-scope still 6; no details.hint; stderr docs wrong; silent break without CHANGELOG; graph exit 2 not in suite.  
3. **Stop-before:** T203 conflict reopening Option→6; daemon arm removal.  
4. **After ship:** T202 / T203 (F27).  
5. **Breaking:** document like T198 graph 0→2.  

## F36 inventory notes (implement 2026-08-03)

CLI `scope: Option` sites confirmed via grep (`main.rs` + command modules):

| Site | File | Disposition |
|------|------|-------------|
| `policy show` | `main.rs` PolicyCommands::Show; `policy_cmd::ShowOptions` | **F4 clap-required** → `String` |
| `review list` | `main.rs` ReviewCommands::List; `review::ListOptions` | **F4 clap-required** → `String` |
| `erasure request` | `main.rs` ErasureCommands::Request; `erasure::RequestOptions` | **F4 clap-required** → `String` (wire `RequestErasureRequest.scope` remains `Option`; CLI wraps `Some(...)`) |
| `evidence show` | `main.rs` / `evidence::ShowOptions` | **leave** — still has runtime INVALID_PAYLOAD check; not F4 flip target |
| `source show` | `main.rs` / `source::ShowOptions` | **leave** — same class residual |
| `retention apply` | `main.rs` / `retention` | **leave** — CE candidates only; optional when projection-only |
| `migrate --default-scope` | `main.rs` / `migrate` | **leave** — optional default fill |
| Other Options (principal_id, format, command_id, …) | main.rs | **leave** — not missing-required scope class |

Daemon: `services.rs` `list_review_items` / `request_erasure` None → INVALID_PAYLOAD **retained** (M1 / F35).

Wire DTOs: `ListReviewItemsRequest.scope` and `RequestErasureRequest.scope` remain `Option` for HTTP/IPC.

## Manual checklist

```powershell
ai-brains policy show          # exit 2 clap
ai-brains review list          # exit 2 clap
ai-brains erasure request ...  # missing scope → exit 2
ai-brains policy check --capability ProposeConclusion --scope '...'  # deny → 3 + details.hint if json
```
