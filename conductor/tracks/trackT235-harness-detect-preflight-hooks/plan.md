# T235 Plan — Harness detect + preflight hook install UX

Status: **Completed** (PR #101 `b1a0ecc`). Spec: [spec.md](./spec.md) F1–F42 / AC1–AC23 / **§14**. Series: [README-T234-T239](../README-T234-T239-HARNESS-INGEST.md).

## Goal

1. Detect harnesses **installed on machine** (PATH + home roots) with honest labeling.  
2. Report wiring status per harness (`missing` / `partial` / `ok` / `backend_pending` / …).  
3. `ai-brains harness status|install|uninstall|reset-decline` with dry-run, consent prefs, user-global default.  
4. Preflight **Harnesses installed on machine** section (sibling formatter; no T214 arity growth).  
5. Real AGY install: Stop → wrapper → **mapped** `agy-hook` payload (**F34**).  
6. Others pending; doctor soft 12th check; capability honesty; hermetic PATH+home; no new crates.

## Absorbed deferred / series / live / research / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| deferred.md | Detect + preflight hook install UX | This track F1–F42 / AC1–AC23 |
| Series C6/C7 | Consent + user-global | F10–F12, F24–F25, AC12 |
| Series C1–C3 | Message-only | F27; agy-hook → T234 |
| T234 | Filter SOOT | Do not re-implement |
| T236 F8 | install agy | F14–F15 writer; T236 deep binding |
| T237/T238 | other backends | F14 pending |
| T214 | summary freeze | F8 sibling formatter **AC19** |
| T192 | doctor soft | F17; matrix **11→12 AC22** |
| T190 | reparse | F36 |
| T205 | hermetic home | F4, F32 + **PATH scrub AC17** |
| AI1 | JSON merge / PS quoting / labeling / uninstall | F15, F28, F30, F37, AC23 |
| AI2 **M1** | Stop ≠ agy-hook fields | **F34 hard + AC16** |
| AI2 M2 | PATH hermetic | F32 / AC17 |
| AI2 M3 | `--stdin` vs prompt | F24 / AC18 |
| AI2 M4 | T214 formatter arity | F8 / AC19 |
| AI2 M5 | vault-path-free | F38 / AC20 |
| AI2 M6 | atomic + reparse | F36 / AC21 |
| AI2 L1–L7 | order, Grok path, OC file, trust note, prefs, doctor matrix, pending summary | F6/F7/F12/F14/F17/F21/F28 |
| AI2 capability blind | `supports_hooks: false` lie | F19 flip antigravity capability |

**Not absorbed:** T236 history binding; T237/T238 capture; T239 nightly; T220 summary JSON; T224 display strip; project-scope install DoD; widen agy-hook schema; fullyIdle hard policy (soft F35 only).

## Live dogfood freeze (2026-08-08)

| Item | Value |
|------|-------|
| PATH | grok, agy, opencode, claude, codex **yes** |
| Hook installs | none for AI-Brains |
| AGY hooks.json | missing both locations |
| Prefer write AGY | `~/.gemini/config/hooks.json` |
| agy-hook required | `transcriptPath`, `sessionId`, `projectHash` (`additionalProperties: false`) |
| AGY Stop delivers | `conversationId`, `transcriptPath`, `workspacePaths`, `fullyIdle`, … — **no** sessionId/projectHash |

## Research freeze (2026-08-08)

| Topic | Note |
|-------|------|
| Grok hooks | `~/.grok/hooks/*.json` — managed **file** `ai-brains.json` |
| AGY hooks | named entries; Stop + fullyIdle; `~/.gemini/config/` |
| OpenCode | local plugins dir auto-load; npm list separate |
| Claude | `~/.claude/settings.json` merge |
| Deps | clap 4.5 / serde_json 1.0 / dirs 6 / is-terminal 0.4 — no bump |
| Zero new crates | PATH walk + read_line + serde_json |

## Phases

### Phase 0 — Plan freeze

- [x] Live probe PATH + homes + hooks
- [x] Code map preflight / doctor / agy-hook / is_vault_path_free / capability
- [x] Online docs + dep pins
- [x] Spec F1–F33 + AC1–AC15 (initial)
- [x] **AI fold-in** → F6/F7/F8/F11–F15/F17/F19/F21/F24/F28/F30/F32 elevate; **F34–F42**; **AC16–AC23**; **§14**
- [x] deferred + conductor Planning
- [x] pin plan-start; pin fold-in
- [ ] User **go** before code / ledger TX

### Phase 1 — Ledger + red (on go)

- [x] `ledgerful doctor` / `ledger status --compact`
- [x] `ledgerful ledger start T235-harness-detect-preflight --category FEATURE --message "harness detect + preflight UX; AGY Stop→agy-hook map; consent; vault-free status"` (TX `9f98ce73-5cc8-4ee5-a8aa-f71929d00323`)
- [x] Red: detect fixtures + **PATH scrub** (AC17)
- [x] Red: wiring path markers (Grok file, AGY key)
- [x] Red: **F34 map** Stop → payload (AC16)
- [x] Red: dry-run zero writes
- [x] Red: JSON status order + schema
- [x] Red: unknown harness exit 2

### Phase 2 — Pure detect + wiring + prefs

- [x] Fixed harness order F1/F21
- [x] Home USERPROFILE→HOME; PATH first-hit F39
- [x] Wiring probes F6/F7 (Grok file path; OpenCode file)
- [x] prefs load/save; corrupt refuse F28; decline/install/uninstall stamps F11
- [x] AC1–AC2, AC6, AC15

### Phase 3 — AGY map + writer + CLI

- [x] Pure `map_agy_stop_to_hook_payload` F34 + fullyIdle soft F35
- [x] Wrapper `agy-stop.ps1` (+ soft shell) pipes mapped JSON to `ai-brains agy-hook --payload`
- [x] Map-only hooks.json merge F37; atomic write F36; PS `-File` F15
- [x] `commands/harness.rs` status/install/uninstall/reset-decline
- [x] **`is_vault_path_free` includes Harness** F38 / AC20
- [x] Readiness gate F14; pending summary L7
- [x] Hermetic CLI tests (home + PATH TempEnv)

### Phase 4 — Preflight

- [x] `format_harness_summary_lines` sibling F8 / AC19
- [x] Header `Harnesses installed on machine:`
- [x] Never-prompt: non-TTY, `--no-hook-prompt`, **`--stdin`** F24 / AC18
- [x] TTY ask once F10; auto_install F25
- [x] No PreflightContextResponse key growth

### Phase 5 — Doctor + help + capability + docs

- [x] Doctor `harness_wiring` + update **fixed_matrix 11→12** F17 / AC22
- [x] help_ia add `harness` F18
- [x] Flip `antigravity_capability` supports_hooks / notes F19
- [x] CAPABILITIES + OPERATIONS AC11

### Phase 6 — Manual + gate

- [x] Manual: `harness status` (no vault if possible)
- [ ] Manual: `preflight --summary` Harness line (needs vault; unit-tested formatter)
- [x] Manual: `harness install --harness agy --dry-run`
- [ ] Optional real AGY install (user consent)
- [ ] Full gate + review + pins + conductor Completed

## File touch map (expected)

| Area | Paths |
|------|--------|
| CLI harness | `commands/harness.rs`, `harness/{detect,wiring,prefs,agy_map}.rs`, `mod.rs` |
| main | clap `Harness` command; **`is_vault_path_free`** (~1988) |
| Preflight | `preflight.rs` — sibling formatter only |
| Doctor | `doctor.rs` — check + **matrix tests len 12** |
| Capability | `adapters/src/antigravity.rs` capability notes / `supports_hooks` |
| help_ia | Harness inventory |
| Tests | hermetic home+PATH; map unit; merge unit; vault-free status |
| Docs | CAPABILITIES, OPERATIONS |
| **Not** | message_only rewrite; agy-hook schema widen; PreflightContextResponse growth |

## Definition of done

- [x] AC1–AC23 green (esp. **AC16 M1 map**, AC17 PATH, AC18 stdin, AC19 formatter, AC20 vault-free, AC22 doctor matrix) — unit/hermetic coverage; full workspace gate + review still for orchestrator closeout
- [x] AGY install preserves foreign hooks; atomic writes; refuse corrupt
- [x] Non-TTY / `--stdin` never prompt; decline remembered; uninstall clears installed_at
- [x] Docs + capability honesty; full gate + review (implementer done; orchestrator closeout remaining)

## Residual / handoff

| Residual | Owner |
|----------|-------|
| history.jsonl → project binding | T236 |
| fullyIdle hard policy / re-queue | T236 |
| Grok install_ready + capture + trust | T237 |
| OpenCode plugin body | T238 |
| Multi-harness nightly | T239 |
| Project-scope install | later |
| Preflight JSON harness array | T220+ |
| Widen agy-hook schema | only if F34 fails in practice |
