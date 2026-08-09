# T221 Plan — Governed first-run + deny exit honesty

**Status:** 🛠️ **Implementing** (TX eef6a954; AI fold-in 2026-08-09)  
**Category:** UX / CONTRACT / BUGFIX  
**Depends:** T210 ✅, T203 ✅, T202 ✅, T201 ✅, T152 progressive packet ✅  
**Spec:** [spec.md](./spec.md) — includes AI fold-in **§14**

## Goal

1. Progressive policy deny → process exit **3** (not 0), keep `ProgressiveQueryResponse` on stdout.  
2. **In-band** bootstrap via hard **`denial_hint`** (F17) for stdout-only agents + stderr **CODE then hint** (F4).  
3. Expand `kind: "Denied"` → exit **3**; Unknown stays 0.  
4. Human `emit_error` prints `details.hint`; CP errors via **fail_cp** (exit 3 not 1).  
5. AC3 hermetic bootstraps **System** principal (no Human `bbbb` trap).  
6. Soft doctor `policy_grants` only with matrix/cwd honesty (M4); docs; hermetic ACs.  
7. Zero new crates; no briefing exit flip; no auto-grant.

## Absorbed deferred / audit / research / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| deferred.md series | progressive deny exit 0 | Hard F1–F6, F17, AC1–AC8 |
| Series README T221 | governed dead-end honesty | This track |
| Placeholder draft | exit 3 / doctor soft / human bootstrap / no interactive | Expanded F1–F36 |
| T210 F28 | briefing soft exit 0 | F7 / AC10 |
| clig.dev | non-zero on failure; next command | F1, F4, F5, F17 |
| clap 4.6.6 | workspace 4.5 / resolved 4.6.1 | **no bump** F16 |
| Dogfood 2026-08-09 | progressive exit 0 + denied true | AC1 freeze |
| **AI1 M1** | AC3 principal trap | **F31 hard** + AC3 |
| **AI1 M2** | `--principal-id` progressive | Soft F32 |
| **AI1 M3** | stdout-only need remediation | **F17 hard** + AC2/AC11; F28 override |
| **AI1 M4** | doctor matrix + cwd | F12 constraints if ships |
| **AI1 M5** | CP `?` → exit 1 | **F33 hard** + AC12 soft |
| **AI1 M6** | dry-run deny | **F34** + **AC1b** |
| **AI1 M7** | CODE then hint | **F4** |
| **AI1 M8** | Denied semantics | **F35** docs |
| **AI1 M9** | trace applied_policy | Soft F36 |
| **AI1 M10** | F18 daemon 200 | Affirm residual |
| **AI2** | architecture / dual-stream / AC map | Affirmed |

**Not absorbed as DoD:** briefing hard deny; auto-init; interactive; T226; daemon 200→403; clap 5; MSI; full admin; force F12 if matrix slips.

## Live dogfood freeze (2026-08-09)

| Command | Observed |
|---------|----------|
| `query progressive "why graph"` | `denied: true`, empty results, **exit 0** |
| `source list --format json` | `POLICY_DENIED` + bootstrap hint, exit **3** |
| `briefing project --format json` | soft `denied: true`, exit **0** |
| `query expand` unknown | `kind: Unknown`, exit **0** |
| `policy bootstrap --help` | T210 discovery path present |

## Research freeze (2026-08-09)

| Topic | Note |
|-------|------|
| Root cause | CLI always `Ok` after progressive packet; ignores `denied` |
| clig.dev | zero only success; map failure modes; suggest next command |
| Exit SOOT | lists/check already 3; progressive outlier |
| clap | pin 4.5; latest 4.6.6 — no bump |
| Packet vs fail_api | keep packet (F2); exit 3 after emit (F3) |
| stdout-only agents | F17 `denial_hint` hard (AI1 M3) |
| Principal SOOT | progressive = `cli_principal()`; AC3 must match (AI1 M1) |

## Implementation sketch (on go)

```rust
// contracts ProgressiveQueryResponse — additive:
// denial_hint: Option<String>  // serde default + skip_serializing_if

// CP deny construction: set denial_hint to bootstrap wording
// OR CLI after Ok: if resp.denied && resp.denial_hint.is_none() { fill from POLICY_DENIED_HINT }

// governed_query.rs run_progressive
let resp = match progressive_query(...) {
    Ok(r) => r,
    Err(e) => return fail_cp(OutputFormat::Json, e), // F33
};
emit_json(&resp)?;
if resp.denied {
    eprintln!("POLICY_DENIED: progressive query denied");
    eprintln!("{POLICY_DENIED_HINT}");
    return Err(Box::new(GovernedCliError::emitted(
        EXIT_POLICY_DENIED,
        "POLICY_DENIED: progressive query denied",
    )));
}
Ok(())
```

```rust
// emit_error Human: CODE: message then details.hint if string
// run_expand: fail_cp on Err; if kind == "Denied" → F4 + exit 3
// Hermetic AC3: policy bootstrap WITHOUT --principal-id (System)
```

## Phases

### Phase 0 — Plan freeze

- [x] Preflight / doctor / ledger status  
- [x] Live dogfood progressive / list / briefing / expand / bootstrap help  
- [x] Code map + online research  
- [x] Spec F1–F30 + AC1–AC10 (initial)  
- [x] deferred.md + conductor → **Planning**  
- [x] series README status note  
- [x] `ai-brains pin` plan-start + freeze  
- [x] AI fold-in §14 → F1–F36 + AC1b/AC11/AC12  

### Phase 1 — Red (TDD)

- [x] Hermetic AC1 progressive deny exit 3 + denied true  
- [x] Hermetic AC1b `--dry-run` deny exit 3  
- [x] Hermetic AC2/AC11 stderr CODE+bootstrap + stdout denial_hint  
- [x] Hermetic AC3 bootstrap **System** then progressive exit 0  
- [x] Hermetic/unit AC4 human emit_error hint  
- [x] Hermetic/unit AC5 expand Denied/Unknown (Unknown hermetic; Denied via CLI kind=="Denied" path — seed Denied residual if needed)  
- [x] AC6 missing project still 2  
- [x] AC10 briefing deny still 0  
- [x] Soft AC12 fail_cp / PolicyDenied exit 3 unit  

### Phase 2 — Green

- [x] F17 contracts `denial_hint` + CP/CLI fill + golden  
- [x] `run_progressive` F1–F4 + F33 + F34  
- [x] `run_expand` F6/F30/F33  
- [x] `emit_error` F5  
- [ ] Soft F12 doctor + matrix if free — **skipped** (matrix/cwd risk; residual)  
- [x] Docs F20/F35  

### Phase 3 — Verify

- [x] Targeted nextest + clippy package (`contracts`/`control-plane`/`cli` clippy; nextest hermetic suite green)  
- [ ] Manual live vault dogfood (principal-correct bootstrap)  
- [ ] Full gate  
- [ ] Review + closeout  

## Manual test script (implement)

```powershell
# No grants (or temp vault):
ai-brains query progressive "test" ; echo "exit=$LASTEXITCODE"   # expect 3
# stdout should include denial_hint / bootstrap; stderr POLICY_DENIED + hint
ai-brains query progressive "test" --dry-run ; echo "exit=$LASTEXITCODE"  # expect 3
ai-brains source list --format human ; echo "exit=$LASTEXITCODE" # expect 3 + bootstrap
ai-brains policy bootstrap   # System principal — omit --principal-id when matching progressive
ai-brains query progressive "test" ; echo "exit=$LASTEXITCODE"   # expect 0
ai-brains briefing project --format json ; echo "exit=$LASTEXITCODE"  # expect 0 soft deny or data
```

## Stop-before

- Changing briefing deny exit without product re-decision  
- Auto-grant on init  
- Replacing progressive stdout with ApiError without explicit go on BREAKING  
- Full grant admin surface  
- T226 scope work in this TX  
- Daemon/HTTP progressive 200→403 without separate decision  
