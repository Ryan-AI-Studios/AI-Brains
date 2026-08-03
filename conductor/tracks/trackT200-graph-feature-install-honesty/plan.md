# T200 Plan — Graph Feature Install Honesty

Status: **In Progress / Implementing** (A2=no docs-only A). Spec: [spec.md](./spec.md).

## Preconditions

- [x] Live re-scan (AI2 11/11)  
- [x] Expand F1–F26  
- [x] **AI fold-in** (AI1; AI2 M1–M7; prefer docs-only A) — §14; F27–F35; AC12–AC13  
- [x] Deferred roll-in  
- [x] A2 decision: **docs-only A** (A2=no) — no Cargo `default = ["graph"]`  
- [x] A1 size measure *(skipped — A2=no; not required)*  
- [ ] `ledgerful ledger start T200-GraphFeatureInstallHonesty --category DOCS` *(or FEATURE if A2=yes)*  

## Deferred rolled in

| Item | Disposition |
|------|-------------|
| Graph install honesty | **Absorb** |
| Feature-off exit 0 | **T198** regression only |
| CAPABILITIES all graph refs | **Absorb** F7 |
| Graph-on missing CI | **Absorb** F14 hard if A2=no |
| Release vs source divergence | **Absorb** F5 honesty (no release flip DoD) |
| JSON envelope | **T201** |
| Help grouping | **T204** soft |
| binstall / dual artifact | Out |

## Research pins (post fold-in)

| Fact | Pin |
|------|-----|
| Preferred A2 | **docs-only A** (`default = []`) |
| INSTALL primary SOOT | `cargo install --path crates/ai-brains-cli --locked --features graph` |
| Release binary | **graph-off** (document) |
| Exit off | **2** + FEATURE_UNAVAILABLE |
| F2 if flip | absolute Δ ≤ **8 MB** only |
| CI | **Both** on and off covered (F13∨F14 hard) |
| F9 | Grep guard; stubs already match |
| New crates | Zero |

## Phases

### Phase A — Decision freeze

- [ ] **A0** Ledger start  
- [x] **A-fold** AI fold-in M1–M7  
- [x] **A2** **docs-only A** (A2=no) — no Cargo default flip  
- [x] **A1** Size measure *(skipped — A2=no)*  

### Phase B — Docs (always; branch slim on A2)

- [x] **B1a** INSTALL primary = F27 SOOT  
- [x] **B1b** INSTALL slim: A2=no → bare locked  
- [x] **B1c** INSTALL **GitHub Release graph-off** honesty (AC12)  
- [x] **B2** CAPABILITIES: §9 + command table + needs (F7)  
- [x] **B3** CONTRIBUTING matrix + smoke run command  
- [x] **B4** Docs/README one-liner  
- [x] **B5** F9 grep regression on both stubs (no edit if already match)  

### Phase C — Cargo default (only if A2=yes)

- [x] **C*** skipped (A2=no)  

### Phase D — CI graph-on (if A2=no) + closeout

- [x] **D0** **Required** CI: `--features graph` nextest for health smoke (F14 hard) — Windows + Linux  
- [x] **D1** CHANGELOG  
- [ ] **D2** Full gate  
- [ ] **D3** Review AC1–AC13  
- [ ] **D4** deferred strike; conductor Completed  
- [ ] **D5** Pin policy + ledger commit  

## Verification matrix

| AC | Proof |
|----|-------|
| AC1–2, AC5, AC12 | docs |
| AC3–4, AC13 | smoke + CI |
| AC6 | grep test |
| AC7 | tree |
| AC8–9 | A2 branch |
| AC10–11 | gate + claims |

## Out of scope

- [ ] release.yml graph-on flip  
- [ ] dual release artifact  
- [ ] binstall  
- [ ] T201 JSON  
- [ ] T204 help group  
- [ ] stub dedupe refactor  

## Implement notes

1. **Default path:** docs-only A → B* → D0 CI graph-on → CHANGELOG → gate.  
2. **High findings:** missing release honesty; missing F14/F13 CI; CAPABILITIES partial; exit 2 regress.  
3. **Stop-before:** Cargo flip without F13; release flip without go.  
4. **After ship:** T201.  
5. **Category:** `DOCS` if docs-only; `FEATURE` if Cargo default flips.  

## Manual checklist

```powershell
cargo run -p ai-brains-cli -- graph update
# exit 2 + FEATURE_UNAVAILABLE

cargo run -p ai-brains-cli --features graph -- graph update
# after init path: health / live
```
)
