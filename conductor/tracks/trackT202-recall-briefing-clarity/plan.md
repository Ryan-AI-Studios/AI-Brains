# T202 Plan — Recall + Briefing Clarity

Status: **Proposed / Expanded + AI fold-in** (plan-only). Spec: [spec.md](./spec.md).

## Preconditions

- [x] Live re-scan (2026-08-04): T101 pretty shipped; semantic soft-fail; model hardcode; progressive terse error; briefing default json  
- [x] Online research: clig.dev human-first/TTY; embed health via product path not Ollama-only  
- [x] Expand F1–F28 + AC1–AC14  
- [x] **AI fold-in** M1–M7 + L1/L3–L6/L8 + L2/L5→T204 — §14; F29–F36; AC2b/AC3b/AC15  
- [x] Deferred roll-in  
- [ ] `ledgerful ledger start T202-RecallBriefingClarity --category FEATURE` *(on go only)*  

## Deferred rolled in

| Item | Disposition |
|------|-------------|
| Recall/briefing/query progressive clarity (deferred.md series row) | **Absorb** |
| Semantic silent fail (unwrap_or_else + eprintln only) | **Absorb** F2–F4, F6, F27 |
| `AI_BRAINS_EMBEDDING_MODEL` ignored in `semantic.rs` | **Absorb** F5 |
| TTY recall pretty | **Closed T101** — F1 verify only |
| Briefing deny without scannable why / missing warnings kind | **Absorb** F7–F9 |
| Progressive project ceremony | **Absorb** F10–F11 |
| Soft scope resolve / discovery lists | **T203** (F12) |
| Ranking / new models | **Decline** F13 |
| Help IA + broader TTY format (L2/L5) | **T204** (F32) |

## Research pins (post fold-in)

| Fact | Pin |
|------|-----|
| Recall TTY pretty | **Done T101** — residual only |
| Semantic SOOT | `embedding.status` closed set on `--semantic` |
| Soft-fail | Never abort whole recall on embed fail; exit 0 |
| Status map (M6) | refused/timeout → `unreachable`; HTTP/parse/panic → `error` |
| no_stored_embeddings (M7) | embed ok + empty fetch **or** all rows undecodable |
| Model env | `AI_BRAINS_EMBEDDING_MODEL` default nomic; URL `:8083` = nightly |
| Briefing deny (M2) | Seed in `empty_denied`; dedup pushes; Personal-scope refuse covered |
| Briefing format (M1) | **TTY → markdown** (not BREAKING; dogfood uses `--format json`) |
| Progressive (M3) | `fail_usage` → EXIT_USAGE **2** + example message |
| Daemon (M4) | `lib.rs:273` semantic:false — no parity |
| Pretty/hint (M5) | Status line once; hint next-action only |
| Deps | clap 4.5 / is-terminal 0.4 — no bump |
| Capture independence | FTS without models |

## Phases

### Phase A — Freeze + inventory

- [x] **A0** Expand complete  
- [x] **A1** F9 dogfood safety — **confirmed** (`dogfood-shadow.ps1` `--format json`); CHANGELOG minor not BREAKING  
- [x] **A2** F11 mechanism — **`fail_usage` → GovernedCliError EXIT_USAGE** (handle_cli_result already maps)  
- [x] **A3** Grep `RecallResponse` consumers at implement — only CLI + contracts  
- [x] **A4** AI fold-in M1–M7  
- [x] **A5** Ledger start (on go) — TX `82e6e899-2a97-4d4d-828b-daaede7e7c5b`  

### Phase B — Semantic honesty (core)

- [x] **B1** Contracts: `EmbeddingStatusDto` + `RecallResponse.embedding`  
- [x] **B2** `semantic.rs`: read `AI_BRAINS_EMBEDDING_MODEL` (F5)  
- [x] **B3** Return status from semantic path; F4 map (M6); no_stored_embeddings (M7/L6)  
- [x] **B4** Unit F35: error samples → unreachable vs error  
- [x] **B5** CLI: map status into response; pretty one-line when `status != ok` (F6)  
- [x] **B6** Hint next-action only when status already explains cause (AC15)  
- [x] **B7** Unit/hermetic AC1–AC5, AC3b  

### Phase C — Briefing clarity

- [x] **C1** `empty_denied` (Project + Personal) seeds `kind=denied` (F7)  
- [x] **C2** Dedup manual denied pushes at grant-deny sites (no double warning)  
- [x] **C3** Confirm Personal-scope refuse ~181–188 inherits seed (AC6)  
- [x] **C4** Renderer Denied one-liner (F8, AC7) — left as-is  
- [x] **C5** F9 format resolve TTY markdown  
- [x] **C6** CP/CLI tests AC6–AC8  

### Phase D — Progressive ceremony

- [x] **D1** `governed_common::fail_usage` (F11/M3)  
- [x] **D2** Progressive + expand call `fail_usage` with F30 template  
- [x] **D3** Hermetic AC9–AC10 (exit **2** asserted)  
- [x] **D4** Confirm trace excluded (F31)  

### Phase E — Docs + hermetic suite + closeout

- [x] **E1** F18 suite (≥6 locks; lock2 = unreachable specifically)  
- [x] **E2** CAPABILITIES / OPERATIONS (AC14)  
- [x] **E3** CHANGELOG minor F9 + additive embedding + progressive exit 2 (AC13)  
- [ ] **E4** Full gate  
- [ ] **E5** Review (F22) + soft cross-model  
- [ ] **E6** deferred.md strike; conductor Completed; optional ISSUES L2  
- [ ] **E7** Pin decisions if non-obvious  

## Verification matrix

| AC | Proof |
|----|-------|
| AC1 | unit `resolve_format` |
| AC2, AC2b, AC3, AC3b, AC4–AC5 | unit/hermetic retrieval+CLI |
| AC6–AC7 | CP tests (Personal-scope refuse + grant-deny) |
| AC8 | unit/hermetic briefing format |
| AC9–AC10 | hermetic progressive/expand exit 2 |
| AC11 | review diff |
| AC12 | full gate |
| AC13–AC14 | docs |
| AC15 | unit/snapshot pretty+hint |

## Out of scope

- [ ] T203 soft-default / list discovery  
- [ ] Ranking changes  
- [ ] New embedding providers  
- [ ] Progressive human pretty format  
- [ ] Daemon semantic parity (F25)  
- [ ] Full governed TTY format matrix (T204)  
- [ ] clap / is-terminal bumps  

## Implement notes

1. **Order:** contracts → retrieval status+model+map → CLI wire/hint → briefing deny/format → `fail_usage` progressive → hermetic → docs → gate.  
2. **High findings:** silent semantic fail; model hardcode; bare empty_denied; progressive exit 1; double status+hint; vague unreachable/error.  
3. **Stop-before:** hard-fail recall on embed down; T203 scope work; ranking; daemon semantic.  
4. **After ship:** T203; T204 L2/L5.  
5. **Highest complexity:** F11 `fail_usage` + F18 hermetic; status map unit (F35).  
6. **F9:** proceed preferred TTY markdown; CHANGELOG minor only.  

## empty_denied call-site inventory (M2)

| Site | File | Disposition |
|------|------|-------------|
| Helper Project | `contracts/.../briefings.rs` `empty_denied` | **Seed** `kind=denied` |
| Helper Personal | same | **Seed** |
| Personal-scope refuse | `control-plane/.../project.rs` ~181–188 | **Fixed by helper** (was missing push) |
| Full grant deny | `project.rs` ~205–216 | **Dedup** remove manual push after seed |
| Partial decision/conclusion deny | `project.rs` ~284, ~364 | **Keep** if not using empty_denied (section-level) **or** ensure single denied entry |
| Personal grant deny | `personal.rs` ~114–125 | **Dedup** if helper seeds |
| Tests / evaluation empty | evaluation runner/metrics, daemon-api wire | Still valid; may gain warnings (assert kind if they check empty warnings) |

## Manual checklist (on implement)

- [ ] TTY: `recall "test"` pretty without `--format`  
- [ ] TTY: `recall "test" --semantic` shows status when embed down; hint does not restate same cause  
- [ ] Non-TTY: JSON includes `embedding` only when semantic  
- [ ] `briefing personal` denied: markdown one-line why + warnings kind  
- [ ] TTY: `briefing project` without `--format` → markdown  
- [ ] `query progressive "x"` no project: example + exit 2  
- [ ] `query expand` same; `query trace` unchanged  

## Plan notes (filled at fold-in)

| Decision | Choice | Evidence |
|----------|--------|----------|
| F9 TTY markdown | **Freeze preferred** | dogfood-shadow.ps1:539,629 `--format json` |
| F11 exit 2 | **`fail_usage` → GovernedCliError EXIT_USAGE** | handle_cli_result downcast; no clap-required project |
| Daemon recall parity | **Soft skip** | `ai-brainsd` lib.rs:273 `semantic: false` |
| F7 empty_denied | **Seed helper + dedup** | M2; covers ~181–188 |
| F4 unreachable vs error | **Closed map** | M6 |
| no_stored_embeddings | empty fetch **or** all undecodable | M7 + L6 |
)
