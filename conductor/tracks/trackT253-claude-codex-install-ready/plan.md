# T253 Plan — Claude / Codex install_ready

**Status:** 📋 **Planning** (plan-only until **go**)  
**Spec:** [spec.md](./spec.md) F0–F34 / AC1–AC20  
**Category:** FEATURE / HARNESS  
**Ledger TX (planning):** `3cadc357-5c06-46c6-b22d-6812bb9a2110` (DOCS)  
**Ledger TX (on go):** `ledgerful ledger start T253-claude-codex-install-ready --category FEATURE --message "Claude/Codex install writers + message-only UPS/Stop hooks; install_ready true; no nightly"`

---

## Preflight (plan time — 2026-08-15)

| Check | Result |
|-------|--------|
| Claude / Codex binaries | **2.1.221** / **0.145.0** on PATH |
| `~\.claude\settings.json` | **missing** (home exists) |
| `~\.codex\hooks.json` | **missing** |
| Codex `config.toml` `[features]` | present; **no** `hooks = false` (default-on) |
| `harness status` | claude/codex present, `install_ready=false`, next `T239+` |
| `preflight --summary` | same pending next |
| Official Claude hooks | Stop `last_assistant_message`; empty stdout allows stop; UPS has `prompt` |
| Official Codex hooks | default-on `features.hooks`; `/hooks` trust; Stop wants JSON; `{"continue":true}` SOOT; UPS `prompt`; SessionEnd ≤3s |
| Adapters | Claude naive NeutralEvent parse; Codex capability-only; both claim `Full` |
| Legacy scripts | `scripts/target-claude-hook.ps1` / `target-codex-hook.ps1` — SessionStart injection; **not** DoD |
| Pins | clap lock **4.6.1**, serde_json **1.0.150**, dirs **6.0.0**, is-terminal **0.4.17** — **no bumps** |
| rustc / nextest | 1.95.0 / 0.9.140 |
| Ledger | 0 pending at scan; planning TX `3cadc357` |
| Hotspots | `install.rs`/`detect.rs` not top 10; `preflight.rs` #7; `governed_common.rs` #9 — call only |
| Tree | CLEAN at plan start (`18bea6e`) |
| T252 | Completed — do not touch ingest |
| Nightly | T239 D16 — do **not** add sources |
| Identity / doctor degraded | Unrelated (`test-alias`, backup_recent, graph_density) |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Claude/Codex install_ready (T239+) | deferred.md / T239 S8 / T245 F13 | **DoD** F1–F6 / F21 |
| Placeholder F1 research 2026 surfaces | spec draft | **§3 freeze** |
| Placeholder F2 message-only install_ready | spec draft | **F1 / F10 / F11 / F25** |
| Placeholder F3 no fake ready | spec draft | **F1 / F31** |
| `pending_track` still `T239+` | T245 residual | **F2** |
| T235 F14 Claude/Codex backend_pending | T235 | **F4** real writers |
| Stale `codex_hooks` docs | May research | **F6 / F30** |
| SessionStart injection | legacy scripts / `Docs/claude-hooks.md` | **Not absorbed** F18 / F19 |
| Nightly Claude/Codex | T239 D16 | **Not absorbed** F17 / F34 |
| T254 / T255 | peers | **Not absorbed** |
| clap 5 / pin bumps | series | **Not absorbed** F29 |

---

## Phase 0 — Ledger + impact (on go)

- [ ] `ledgerful ledger status --compact`
- [ ] `ledgerful ledger start T253-claude-codex-install-ready --category FEATURE`
- [ ] `ledgerful scan --impact` + inspect `install.rs` / `detect.rs` / `harness.rs` / adapters
- [ ] Confirm no other agent editing those files
- [ ] Re-fetch official hook pages if >7 days since 2026-08-15 freeze

---

## Phase 1 — Red (TDD)

- [ ] AC1 `install_ready` / `pending_track` units (fail)
- [ ] AC6 wrapper stdout contract units (fail — helpers missing)
- [ ] AC7 message-only filter cases (Claude JSONL + Codex `response_item`)
- [ ] AC8/AC9 payload map + Grok-shaped skip
- [ ] AC2 dry-run zero-write
- [ ] AC13 doctor no-T253-pending when all five ready
- [ ] Commit red allowed

---

## Phase 2 — Green (writers + flip)

- [ ] F2 flip `install_ready` / `pending_track`
- [ ] F5 `install_claude` / uninstall (merge settings.json, wrapper, bake)
- [ ] F6 `install_codex` / uninstall (`hooks.json` only, no config.toml)
- [ ] F4 / F3 dispatch + `all-ready`
- [ ] F8 / F9 wrapper bodies
- [ ] F20 probe + `targets_for` include wrappers
- [ ] F21 doctor/preflight strings + tests
- [ ] AC3–AC5 / AC12 / AC14 / AC18 / AC19 green

---

## Phase 3 — Hook + import CLIs

- [ ] F14 `claude-hook` / `codex-hook` + schemas
- [ ] F10 / F11 live maps
- [ ] F13 `claude-import`
- [ ] F12 `codex-import` fail-open
- [ ] F15 / F16 turn ids, thinking None, binding
- [ ] F23 Grok fail-open
- [ ] F25 adapter notes (keep Full)
- [ ] clap `main.rs` + help_ia AC15
- [ ] AC7–AC11 / AC17

---

## Phase 4 — Docs

- [ ] F30 CAPABILITIES / OPERATIONS / WORKFLOWS / CHANGELOG / skill
- [ ] Research-doc stale banners (Claude + Codex)
- [ ] Grep product docs for `T239+` / `backend pending` / `codex_hooks` on these two harnesses
- [ ] Confirm **no** nightly source list growth (F17)

---

## Phase 5 — Manual (on go only)

- [ ] AC20: `harness install --harness claude --dry-run` then `--yes`
- [ ] Same for `codex` (or one `all-ready --yes`)
- [ ] `harness status` both `ok`; preflight no pending next
- [ ] Confirm `config.toml` bytes unchanged (except we never touch it)
- [ ] Confirm zero new files under `C:\dev\AI-Brains`
- [ ] Record `/hooks` trust next-action in this plan
- [ ] Optional live Stop fire (F34)

---

## Phase 6 — Gate + review

- [ ] `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace`
- [ ] `cargo deny check ; cargo audit` (if those binaries are on PATH)
- [ ] `ledgerful verify --scope full`
- [ ] Internal review.md until clean
- [ ] Cross-model `codex-review` (FEATURE)
- [ ] Conductor → Completed; deferred.md absorb line
- [ ] `ledgerful ledger commit`

---

## Out of scope reminders

- No nightly Claude/Codex.
- No SessionStart injection.
- No `config.toml` feature rewrite.
- No ingest.rs / T252 revisit.
- No T254/T255.
- Plan-only until the user says **go**.
