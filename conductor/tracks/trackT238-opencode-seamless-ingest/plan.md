# T238 Plan — OpenCode seamless ingest

**Status:** Implementing (phases 1–6 code complete; orchestrator closes after PR).  
**Category:** FEATURE  
**Depends:** T234 ✅, T235 ✅ (OpenCode marker/pending), T236/T237 lessons ✅  
**Research:** live OpenCode 1.18.15 + official plugins/CLI/SDK docs 2026-08-08; live export `{info,messages}` nested shape; no dep bumps  
**Fold-in:** deferred OpenCode plugin+export; T234 wire; live-vs-fixture gap; **AI review 2026-08-09** — M1–M2 elevated hard; M3–M9 absorbed

## Goal

Ship **real OpenCode install** (global plugin on `session.idle` with **parentID child skip** + **in-flight guard**) + **`opencode-hook`** (prefer SDK messages live / export batch → **message-only** with **synthetic drop**) + **`opencode-import`** (list + export + watermark, **never** `opencode.db`) + capability honesty.

## Deferred / AI absorption

| Source | Item | Disposition |
|--------|------|-------------|
| deferred harness | OpenCode plugin + export batch | **Absorb** |
| T234 | wire `filter_opencode_*` | **Absorb** F1–F7 / AC1–AC2 |
| T234 residual | “Live OpenCode export schema” | **Absorb** D5–D8 / F1 / AC1 |
| T235 | OpenCode `backend_pending` / `install_ready` false | **Absorb** F27–F32 / AC9–AC11 |
| T236/T237 lessons | unbound, normalize, path meta, turn-id, dry-run/`--force`, subagent/chrome | **Absorb** F10/F13–F14/F20–F22 / AC4–AC8/AC13/AC21–AC22 |
| Live probe | nested `info.role`; part type `tool`/`reasoning`; 18MB exports | **Absorb** F1/F3/F12/F18 / AC1/AC16/AC19 |
| Stale `Docs/Opencode-Hooks-Research.md` | wrong config registration | **F37** supersede banner |
| **AI2 M1** | child/subagent session.idle pollution | **F10 hard** / **AC21** |
| **AI2 M2** | synthetic/ignored/editor_context text leak | **F2/F3 hard** / **AC22** Phase 1 Red |
| **AI2 M3** | session.idle deprecated | **F34** / S12; batch backstop |
| **AI2 M4** | list cap-100 + projectId casing | **F17 hard** / **AC23** |
| **AI2 M5** | live SDK messages + in-flight | **F12 hard** (S4→DoD); **F15** |
| **AI2 M6** | full part-type union | **F3** / **AC1** |
| **AI2 M7** | compaction skip key | **S2** rewrite |
| **AI2 M8** | OPENCODE_CONFIG_DIR | **F40** / F34 |
| **AI2 M9** | prefer worktree | **F20 hard** |
| AI1 | timeout / foreign plugins / labels | affirmed |
| scheduled skip-import | SYSTEM nightly | honesty → **T239** |
| multi-harness nightly | T239 | **Not absorbed** |
| Claude/Codex install | | **Not absorbed** (S8; F32 labels → **T239+**) |
| session.created inject / compacting archive | research doc | **Not absorbed** (non-goals / S10) |
| npm plugin publish | | Soft **S5** |

## Phase checklist

### Phase 0 — Preflight (go day)

- [x] `ledgerful doctor` + `ledgerful ledger status --compact`
- [x] `ledgerful ledger start T238-opencode-seamless-ingest --category FEATURE --message "OpenCode plugin + export import"` (TX 091b1edc — do not re-start)
- [x] `ledgerful scan --impact` on harness/adapters/cli commands
- [x] No intentional dep bumps
- [x] Re-export one small live session if shape drift suspected (redact into fixture); sample synthetic parts if available

### Phase 1 — Synthetic chrome + nested export filter (blocking first)

- [x] **Red AC22 (AI2 M2):** synthetic Read-tool / executed-by-user / ignored / editor_context / compaction_continue → **zero** user turns; bare real prompt kept
- [x] **Red AC1:** nested fixture with full part-type union (tool, reasoning, step-*, snapshot, patch, file, subtask, agent, retry, compaction) → only non-synthetic text
- [x] **Red AC19:** part type `tool` dropped even with stray text fields
- [x] **Red AC2:** flat `opencode_messages.jsonl` still green
- [x] **Green F1–F7:** normalize + `filter_opencode_export`; structured synthetic/ignored drop; extended denylist
- [x] source_ts from `info.time.created` when present (F5)

### Phase 2 — `opencode-hook` + turn ids + child skip

- [x] **Red AC3/AC4:** hermetic hook with fixture path; msg-id turn keys; thinking None; live==batch ids
- [x] **Red AC21:** parentID / child session → zero ingest + `skipped_child_session`
- [x] **Green F10–F15:** hook payload schema; parent re-check; project resolve worktree→directory; unbound anti-hijack (AC5/AC6/AC13)
- [x] Live path accepts SDK-shaped messages file (F12); export fallback timeout 120s (batch + live fallback)

### Phase 3 — Batch `opencode-import` + watermark

- [x] **Red AC7/AC8/AC12/AC14/AC16/AC20/AC23:** watermark, `--force`, dry-run, missing binary, no db, timeout fail-open, hermetic inject, list cap warn
- [x] **Green F16–F26:** list `-n` sizing, tolerant `projectId`, cursor atomic, stats (`list_capped`, `skipped_child_session`), source_meta, worktree bind
- [x] CLI + help_ia; unsummarized path sanity (F25)

### Phase 4 — Install + plugin (idle, parentID, in-flight, SDK messages)

- [x] **Red AC9–AC11, AC18:** install/uninstall/dry-run/foreign preserve/idempotent managed marker
- [x] **Green F8–F10, F12, F15, F27–F33, F40:** `install_opencode`; `install_ready` true; plugin: idle → parentID skip → in-flight → `client.session.messages` → opencode-hook; fail-open; OPENCODE_CONFIG_DIR when set
- [x] Update detect tests: Agy+Grok+**Opencode** ready; Claude/Codex **T239+**

### Phase 5 — Docs + capability honesty

- [x] CAPABILITIES (F34/F39: synthetic drop, child skip, idle deprecation, SDK live, list cap, config dir)
- [x] OPERATIONS (F35) + series README (CHANGELOG file absent — ops/capabilities carry ship notes)
- [x] F37 banner on `Docs/Opencode-Hooks-Research.md`
- [x] F32 Claude/Codex pending labels

### Phase 6 — Verify / close

- [x] Targeted nextest adapters + cli; clippy -D warnings
- [x] Manual smoke: hermetic fixture suites + install dry-run paths (live OpenCode turn optional on operator machine)
- [x] Full local gate: fmt, clippy workspace, nextest 2373, deny, audit (warnings allowed)
- [x] Internal + codex-review → final **PASS** (`review.codex.final.md`)
- [x] Pins: nested export SOOT; synthetic drop; child skip; live SDK messages; no SQLite; msg-id turns
- [x] Feature PR #106 + CI Win/Linux/macOS green + squash-merge `3378a02`
- [x] `ledgerful ledger commit`; conductor Completed (closeout)

## Implement order (risk)

1. **Synthetic chrome + nested filter** (blocks memory pollution — M2)  
2. opencode-hook + **child skip** (M1) + msg-id turns  
3. opencode-import + watermark + list cap + missing binary  
4. install plugin (SDK messages + in-flight + parentID) + install_ready  
5. Docs  

## Files likely touched

| Area | Path |
|------|------|
| Filter | `crates/ai-brains-adapters/src/message_only.rs` + fixtures (`opencode_export_live.json`, synthetic matrix, flat jsonl) |
| OpenCode lib | `crates/ai-brains-adapters/src/opencode.rs` (capability + helpers if split) and/or cli-side parse module |
| Hook / import | `commands/opencode_hook.rs`, `opencode_import.rs`, `main.rs`, `help_ia.rs` |
| Install | `harness/install.rs` (`install_opencode`, plugin body with parentID + messages), `detect.rs`, `wiring.rs` |
| Schema / docs | `Docs/schemas/opencode-hook-payload.json`, CAPABILITIES, OPERATIONS, CHANGELOG, Opencode-Hooks-Research banner, series README |

## Out of scope

T239 body; Claude/Codex install_ready; raw `opencode.db`; session.created inject; npm publish; project-local plugins default; default child ingest; dep bumps; MSI; rewriting user `opencode.json` plugin arrays.

## Manual test script (go day)

```powershell
ai-brains harness install --harness opencode --dry-run
ai-brains harness install --harness opencode
ai-brains harness status
# Restart OpenCode; one short user turn; wait until idle
ai-brains opencode-import --days 1
ai-brains recall "<phrase from that turn>" --limit 5
# Confirm no tool/reasoning/synthetic chrome memories
# (rename PATH briefly) ai-brains opencode-import --days 1  → soft skip
```

## Stop-before

- Force-push / push main  
- Project-local `.opencode/plugins` without opt-in  
- Opening `opencode.db` as content SOOT  
- Claiming Claude/Codex install shipped  
- Destructive rewrite of user `opencode.jsonc`  
- Default child/subagent ingest without S11 opt-in  
- Code / ledger TX until user says **go**
