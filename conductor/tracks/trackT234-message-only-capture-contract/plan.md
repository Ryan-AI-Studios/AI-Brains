# T234 Plan — Message-only capture contract

Status: **Completed** (2026-08-08; PR #100 `2ea8897`). Spec: [spec.md](./spec.md). Series: [README-T234-T239](../README-T234-T239-HARNESS-INGEST.md).

## Goal

1. Ship pure **`ai_brains_adapters::message_only`** SOOT: user + final assistant only.  
2. Drop tools / thinking / reasoning / system chrome with harness-aware normalizers.  
3. Migrate **antigravity `extract_turns`** and **agy** simple path onto SOOT.  
4. Fixture matrices AGY / Grok / OpenCode-like; docs; unblock T236–T239.  
5. **UTF-8-safe** strip (AI1 §4 / F43); AGY type-strict drops (AI1 §2); Grok multipart extract (AI1 §1).

## Absorbed deferred / series / live / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| README C1–C3 | Capture Privacy / no tools / no thinking | F3–F6 hard |
| antigravity extract_turns | Partial SOOT, AGY-only | F7/F12 migrate to shared |
| agy-hook role-only filter | Weak | F13/AC14 |
| Live AGY thinking+tool_calls | Pollute risk | F6/F7/AC2 |
| Live Grok type mix | reasoning/tool_result/… | F8/AC3 |
| Series T236–T239 blocks | Shared filter first | F20 |
| Online CoT/tool blocks | Keep text parts only | F11/F27 |
| AI1 §1 | Grok multipart + user_query | F10/F37/AC6 |
| AI1 §2 | VIEW_FILE/RUN_COMMAND content leak | F7/AC16 regardless of content |
| AI1 §3 | Interleaved text+tool parts | F11/AC7 |
| AI1 §4 | UTF-8 slice panic | **F43/AC15** hard |
| AI1 §5 | Keep thinking DTO field | F17/F46 |
| AI2 | Stale T216 report | **F45 out of scope** |

**Not absorbed:** T235 install; T236 binding; T237 hooks; T238 plugin; T239 nightly; T224 display strip; contracts remove `thinking` field.

## Live dogfood freeze (2026-08-08)

| Metric | Value |
|--------|-------|
| AGY transcript | PLANNER with thinking+tool_calls empty content; VIEW_FILE with content |
| Grok large history | tool_result/reasoning/assistant/user/backend_tool_call/system |
| Grok user content | often **Object[]** (parts); assistant **string** |
| extract_turns | Exists; not shared |
| clap/serde_json | 4.5 / 1.0 — no bump |

## Research freeze (2026-08-08)

| Topic | Note |
|-------|------|
| Anthropic thinking/tool_use blocks | drop thinking & tool_use; keep text |
| OpenAI interleaved text+tools | keep text; drop tool items |
| AGENTS Capture Privacy | hard product law |
| Zero new crates | pure serde_json |

## Phases

### Phase 0 — Plan freeze

- [x] Live dogfood AGY + Grok shapes
- [x] Code map (extract_turns, agy, ingest thinking)
- [x] Online / dep research
- [x] Spec F1–F42 + AC1–AC14
- [x] **AI fold-in** → F7/F10/F11/F17 elevate; F43–F47; AC15–AC16; **§14**
- [x] deferred + conductor → Planning
- [x] pin plan-start + fold-in
- [x] User **go** before code / ledger TX

### Phase 1 — Ledger + red

- [x] `ledgerful doctor` / `ledger status --compact`
- [x] `ledgerful ledger start T234-message-only-capture --category ARCHITECTURE --message "shared message-only capture SOOT: user+assistant; drop tools/thinking; UTF-8-safe strip"`
- [x] Red pure: filter_turn; AGY/Grok/OpenCode fixtures fail until green
- [x] Red: UTF-8 emoji USER_REQUEST strip (AC15)
- [x] Red: extract_turns still passes existing tests (baseline)

### Phase 2 — Pure SOOT module (F1–F6, F10–F11, F37, F43)

- [x] `message_only.rs`: IngestRole, IngestableTurn, filter_turn
- [x] extract_user_text (XML + user_query) **char-boundary safe**
- [x] extract_text_from_json_content (string | array; type==text only)
- [x] DropReason optional for tests
- [x] Unit matrix AC5–AC8, AC15

### Phase 3 — Harness filters + fixtures (F7–F9, F21, AC16)

- [x] filter_antigravity_steps: **strict (source, type)**; drop VIEW_FILE/RUN_COMMAND with content
- [x] migrate extract_turns (F12)
- [x] filter_grok_history (F8/F37)
- [x] filter_opencode_messages (F9 fixture)
- [x] Synthetic fixtures under tests/fixtures/message_only/
- [x] AC2–AC4, AC16

### Phase 4 — Wire agy path (F13/AC14/F46)

- [x] parse_agy_transcript output through message_only
- [x] agy-hook uses filtered turns; `thinking: None` SOOT
- [x] Existing antigravity import path green (AC9)

### Phase 5 — Docs + gate

- [x] CAPABILITIES Capture Privacy; OPERATIONS; CHANGELOG; thinking DTO honesty (F17); series README
- [x] Full gate: fmt; clippy -D warnings; nextest; deny; audit
- [x] `ledgerful verify` + ledger commit
- [x] conductor **Completed**; deferred strike; pin closeout
- [x] PR

## Stop-before

- Destructive git / force-push / push main without approval  
- Removing `IngestRequest.thinking` from contracts without separate decision  
- Implementing T235–T239 scope  
- Live import of production secrets into fixtures  

## Manual checklist (on go)

```powershell
cargo nextest run -p ai-brains-adapters message_only
cargo nextest run -p ai-brains-adapters extract_turns
# optional: no live vault mutation required for T234 DoD
```

## Notes

- Plan-only until **go**.  
- Library-first; no new CLI surface required.  
- Stricter drops OK (F32); silent tool keep forbidden.  
- AI fold-in disposition: spec **§14**.
