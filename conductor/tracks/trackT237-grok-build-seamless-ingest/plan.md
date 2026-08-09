# T237 Plan — Grok Build seamless ingest

**Status:** Implementation complete — pending CI/merge closeout.  
**Category:** FEATURE  
**Depends:** T234 ✅, T235 ✅ (Grok marker/pending), T236 lessons ✅  
**Research:** live Grok 1.0.0 + user-guide hooks/sessions 2026-08-08; no dep bumps  
**Fold-in:** AI review 2026-08-08 (§14 of spec) — M1–M6 elevated; M7–M9 / AI1 absorbed  

## Goal

Ship **real Grok install** (hooks + empty-stdout wrapper) + **`grok-hook`** live ingest from `chat_history.jsonl` + **`grok-import`** batch with summary binding, **user_query-only** user keep, subagent skip, percent-encode resolve, unified turn ids, and capability honesty.

## Deferred / AI absorption

| Source | Item | Disposition |
|--------|------|-------------|
| deferred harness | Grok hooks + chat_history batch | **Absorb** |
| T234 | wire `filter_grok_history_*` | **Absorb** F4/F11 / AC1–AC2 |
| T235 | Grok backend_pending / install_ready false | **Absorb** F21–F25 / AC9–AC11 |
| T236 lessons | unbound, normalize, path meta, turn-id | **Absorb** F8/F14/F16 / AC4/AC6/AC13 |
| **AI2 M1** | Stop empty stdout ≠ AGY allow JSON | **F6 hard** / **AC12** |
| **AI2 M2** | chrome w/o synthetic_reason; user_query keep | **F11 hard** / **AC2** |
| **AI2 M3** | subagent walk pollution | **F12 hard** / **AC18** |
| **AI2 M4** | percent encode/decode in grok.rs | **F7 hard** / **AC19** |
| **AI2 M5** | turn-id + source_ts honesty | **F35** / CAPABILITIES; **S8** soft |
| **AI2 M6** | no `$` in hook command | **F34** / **AC19** |
| **AI2 M7** | timeout budget | **F23** timeout **120** |
| **AI2 M8** | Claude/Cursor vendor merge | **F27** caveat |
| **AI2 M9** | Phase 1 Red = live chrome | Phase 1 reorder |
| AI1 path/locks/multipart | F7/F15/F4 | affirmed |
| scheduled skip-import | SYSTEM nightly | honesty → T239 |
| OpenCode / multi nightly | T238 / T239 | **Not absorbed** |
| Claude/Codex install | | **Not absorbed** (S6; F33 labels) |
| UserPromptSubmit | | Soft **S1** |
| Subagent **include** | | Soft **S2** (default skip hard) |

## Phase checklist

### Phase 0 — Preflight (go day)

- [ ] `ledgerful doctor` + `ledgerful ledger status --compact`
- [ ] `ledgerful ledger start T237-grok-build-seamless-ingest --category FEATURE --message "Grok hooks + chat_history import"`
- [ ] `ledgerful scan --impact` on harness/adapters
- [ ] No intentional dep bumps
- [ ] Re-probe live chrome shapes if needed

### Phase 1 — F11 user keep + chrome matrix (blocking first)

- [ ] **Red AC2 (live chrome):** user_info/git_status (no synthetic_reason); synthetic_reason set; compaction prose without user_query → zero user turns
- [ ] **Red AC1:** tool/reasoning/system still dropped; real user_query kept
- [ ] **Green F11:** Grok user keep only via `<user_query>` / `<USER_REQUEST>`
- [ ] Multipart assistant text-parts only (F4 affirm)

### Phase 2 — Percent codec + path resolve + grok-hook

- [ ] **Red AC19:** encode `C:\dev\AI-Brains` → `C%3A%5Cdev%5CAI-Brains`
- [ ] **Red AC20:** `.cwd` / summary `info.id` fallback
- [ ] **Red AC3/AC4:** grok-hook ingest + turn-id parity
- [ ] **Green:** `grok.rs` helpers + `grok-hook --payload` + schema + `--schema`
- [ ] Project resolve + unbound anti-hijack (AC6/AC13)

### Phase 3 — Install + **empty** Stop wrapper (M1)

- [ ] **Red AC12:** wrapper body — **empty stdout**, exit 0, no decision/continue keys; **must not** reuse AGY allow JSON
- [ ] **Red AC9–AC11, AC16, AC19 ($):** install/uninstall/dry-run/corrupt/no-dollar command
- [ ] **Green F1/F6/F21–F25/F34:** `install_grok`; `install_ready` true; create hooks dir
- [ ] Update detect tests (agy + grok ready)
- [ ] timeout 120 on Stop/SessionEnd

### Phase 4 — Batch `grok-import`

- [ ] **Red AC5–AC8, AC14, AC18:** bind, unbound, delta, quiescence/`--force`, no updates.jsonl, **subagent skip**
- [ ] **Green F12–F18:** discover, stats (`skipped_subagent`), source_meta path key
- [ ] CLI + help_ia; unsummarized path sanity (F19)

### Phase 5 — Docs + capability honesty

- [ ] CAPABILITIES (F27/F35: user_query keep, source_ts none, empty Stop, vendor-compat, filter-version)
- [ ] OPERATIONS / CHANGELOG / series README
- [ ] F33 Claude/Codex pending labels

### Phase 6 — Verify / close

- [ ] Targeted nextest adapters + cli; clippy -D warnings
- [ ] Manual: install → one Stop (confirm Grok not blocked) → import → scoped recall
- [ ] Full gate + `ledgerful verify --scope full`
- [ ] Internal + codex-review
- [ ] Pins: empty Stop stdout, user_query keep, subagent skip, percent encode
- [ ] `ledgerful ledger commit`; conductor Completed

## Implement order (risk)

1. **F11 chrome keep rule** (blocks memory pollution)  
2. Percent encode + path resolve  
3. grok-hook  
4. **Empty-stdout install wrapper** (M1)  
5. grok-import + subagent skip  
6. Docs  

## Files likely touched

| Area | Path |
|------|------|
| Filter | `message_only.rs` + fixtures |
| Grok lib | `adapters/src/grok.rs` (encode/decode, resolve, import, subagent skip) |
| Hook / import | `commands/grok_hook.rs`, `grok_import.rs`, `main.rs`, `help_ia.rs` |
| Install | `harness/install.rs` (Grok wrapper ≠ AGY), `detect.rs`, `wiring.rs` |
| Schema / docs | `Docs/schemas/grok-hook-payload.json`, CAPABILITIES, OPERATIONS, CHANGELOG |

## Out of scope

T238/T239 bodies; Claude/Codex install_ready; UserPromptSubmit DoD; default subagent ingest; fingerprint turn-ids (S8); dep bumps; MSI; AGY allow-JSON on Grok Stop.

## Manual test script (go day)

```powershell
ai-brains harness install --harness grok --dry-run
ai-brains harness install --harness grok
ai-brains harness status
# Reload hooks in Grok; one short turn — agent must stop normally (empty Stop stdout)
ai-brains grok-import --days 1
ai-brains recall "<phrase from that turn>" --limit 5
# Confirm no user_info chrome memories; no subagent worktree junk
```

## Stop-before

- Force-push / push main  
- Project-local `.grok/hooks` without opt-in  
- Blocking Grok Stop or emitting AGY allow JSON on Grok Stop  
- Reading updates.jsonl as content  
- Claiming Claude/Codex install shipped  
- Code / ledger TX until user says **go**
