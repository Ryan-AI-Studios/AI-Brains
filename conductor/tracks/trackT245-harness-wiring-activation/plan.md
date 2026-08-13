# T245 Plan — Harness wiring activation

**Status:** ✅ **Completed** (PR #155 `f05e2f6`)  
**Spec:** [spec.md](./spec.md) F0–F30 / AC1–AC21 + §14  
**Category:** OPS / FEATURE / UX  
**Ledger:** plan-only until go

---

## AI fold-in (2026-08-12) — `C:\dev\AI-review.md` AI1 + AI2

AI2 **H1** agreed and **changes F7**. Official AGY CLI hooks load from `~\.gemini\antigravity-cli\plugins\<name>\{plugin.json,hooks.json}`, not top-level `antigravity-cli\hooks.json`. Old dual-write would recreate false `wiring=ok`.

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI2 H1** | AI2 | **Agree hard** | Retract top-level CLI hooks.json. **F7** = IDE merge + CLI **plugin bundle** |
| **AI1 M2** | AI1 | **Agree guard / disagree target** | Keep “never create antigravity-cli”; write bundle not top-level hooks |
| **AI1 M1** | AI1 | **Agree hard** | F8 PS single-quote + JS stringify |
| **AI1 M3 / AI2 M1 M2** | both | **Agree** | F9 exact `"idle"`; idle **not** deprecated |
| **AI1 M4 / AI2 L3** | both | **Agree** | F6 filters + pinned message |
| **AI2 M3 L4 L9 L12** | AI2 | **Agree hard** | `is_ai_brains_exe`; `String` bodies; bake ai-brains spawn only |
| **AI2 L2** | AI2 | **Agree hard** | `all-ready` before `parse_harness_id` |
| **AI2 L1 L5–L8 L10 L11** | AI2 | **Agree** | T243 shipped; JS status.type; uninstall dir; marker T238; after_help; F25 |
| **AI1 L1 L2 O1** | AI1 | **Agree** | AC6 / F14 / Phase 1 names |
| **AI2 O12** | AI2 | **Decline** | Live AGY turn not hard DoD; file-proof is |

### Pins locked

1. F7/F7b plugin bundle + probe honesty  
2. F8 bake/escape/signature  
3. F9 idle exact  
4. F5 list token  
5. F6 doctor message

---

## Preflight (plan time — 2026-08-12)

| Check | Result |
|-------|--------|
| `harness status` | grok/agy/opencode **missing + ready**; claude/codex **missing + pending** |
| Dry-run ×3 | Plans match T235/T237/T238; **zero writes** |
| Live hook dirs | grok hooks, gemini/config, opencode/plugins, ai-brains/hooks **all missing** |
| AGY CLI home | `~\.gemini\antigravity-cli` **exists**; official `~\.gemini\config` **missing** |
| Doctor `harness_wiring` | `5 present, 5 missing (…claude, codex); next: harness status` |
| `resolve_harness_list` | `all` yes; **`all-ready` no** |
| Wrappers / plugin | `Get-Command ai-brains` / `spawn("ai-brains")` — PATH-fragile |
| OpenCode plugin | `session.idle` only (S12 still soft until this track) |
| Pins | clap lock 4.6.1 (crates.io 4.6.6) — **no bump**; serde_json 1.0.150; dirs 6.0.0; is-terminal 0.4.17 |
| Host versions | grok **1.0.3**; agy **1.1.12**; opencode **1.18.16** |
| Ledger | 0 pending, 0 unaudited drift |
| T243 | Completed PR #153 — no rewrite |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| wiring=missing despite install_ready | deferred.md / audit P1 | **DoD** F3/F10/AC13 |
| Recommended order grok→agy→opencode | placeholder F1 | **F1** / `HARNESS_ORDER` filter |
| `--yes` after dry-run; preflight opt-in | placeholder F2 | **F2 / F15** |
| status ok + doctor improves | placeholder F3 | **F3 / F6** |
| No repo pollution | placeholder F4 | **F4 / AC11 / AC13** |
| batch `all-ready` | placeholder F5 (was soft) | **Elevate F5 hard** |
| OpenCode `session.status` idle | T238 S12 | **Elevate F9 hard** |
| Doctor next = status | live | **F6** next `all-ready --dry-run` |
| PATH-loss silent skip | live + wrappers | **F8 bake** |
| AGY write vs CLI home | live + AI2 H1 official plugins doc | **F7 plugin bundle** (not top-level hooks.json) |
| Claude/Codex install_ready | T239+ / S8 | **Not absorbed → T253** |
| UserPromptSubmit / project hooks / npm plugin | T237/T238 soft | **F17 soft** |
| AI-review T245 H1/M1–M4 | `C:\dev\AI-review.md` | **Folded §14** |

---

## Phases

### Phase 0 — Plan freeze

- [x] Full spec + plan
- [x] Live dogfood + official docs + dep pins (2026-08-12)
- [x] Roll T235–T238 residuals + placeholder F1–F5
- [x] T245 AI1+AI2 fold-in — **H1 retracts old F7**; F5/F6/F8/F9 pins
- [x] T243 shipped — no rewrite
- [x] User **go** before production code **or** live `--yes`

### Phase 1 — Red (TDD, on go)

- [x] Unit: `resolve_harness_list__all_ready__returns_grok_agy_opencode`
- [x] Unit: `all` still five ids; `parse_harness_id("all-ready")` still Err; unknown → usage
- [x] Unit: `agy_cli_plugin_dir__when_cli_dir_exists__some_bundle`
- [x] Unit: `agy_cli_plugin_dir__when_cli_dir_absent__none` (does not invent `antigravity-cli`)
- [x] Unit: `doctor_harness_wiring_message__separates_ready_from_pending` + next SOOT
- [x] Unit: `opencode_is_idle_event` idle / status-idle / status-retry / status-busy / `done` (AC20)
- [x] Unit: `is_ai_brains_exe` matrix (AC21) — **not** real `current_exe()`
- [x] Hermetic: `all-ready --dry-run` zero writes (AC1)
- [x] Keep T235–T238 refuse / empty-stdout / allow-stop tests compiling

### Phase 2 — Green (list + AGY documented paths + doctor)

- [x] `resolve_harness_list` `all-ready` **before** parse arm
- [x] after_help `--harness all-ready --dry-run` (no help_ia Daily)
- [x] `install_agy`: always IDE merge; iff CLI home → write bundle `plugin.json` + `hooks.json`
- [x] **Never** write top-level `antigravity-cli/hooks.json`
- [x] `uninstall_agy`: IDE key + `remove_dir_all` bundle only
- [x] `probe_agy` / `targets_for`: documented paths only (F7b)
- [x] Dry-run lists IDE + bundle when CLI home exists
- [x] `check_harness_wiring` uses F6 helper

### Phase 3 — Green (bake + OpenCode status)

- [x] Body builders `Option<&Path> -> String`; update 6+ call sites + tests
- [x] PS `$aiExe = '…'` single quotes; JS `JSON.stringify`
- [x] OpenCode: bake **ai-brains** spawn only — not `opencode` export spawn
- [x] Plugin: `session.idle` **or** `properties.status.type === "idle"`; shared in-flight
- [x] Marker stays `(T238)`
- [x] Grok command line still no `$`
- [x] Foreign OpenCode refuse still

### Phase 4 — Docs

- [x] CAPABILITIES harness table
- [x] OPERATIONS activation recipe + AGY IDE + CLI plugin bundle + reinstall-after-upgrade
- [x] WORKFLOWS **Activate harness capture**
- [x] CHANGELOG T245
- [x] Skill one-liner (project + claude skill if they already mention harness)

### Phase 5 — Live dogfood (go only)

- [x] `ai-brains harness install --harness all-ready --dry-run`
- [x] `ai-brains harness install --harness all-ready --yes`
- [x] `ai-brains harness status` — three **ok**
- [x] `ai-brains doctor` — ready-missing empty; pending T253 honest
- [x] `ai-brains preflight --summary` — no grok/agy/opencode missing next
- [x] Confirm no new files under `C:\dev\AI-Brains` (C7)
- [x] F24 file proof: `~\.gemini\antigravity-cli\plugins\ai-brains-capture\{plugin.json,hooks.json}` exist
- [x] Confirm **no** `~\.gemini\antigravity-cli\hooks.json` created
- [ ] Optional: one short harness turn (not DoD) — skipped (F24)
- [x] Record outputs below

### Phase 6 — Review + gate (go)

- [x] `ledgerful ledger start T245-harness-wiring-activation --category FEATURE`
- [x] Primary review vs spec
- [x] Cross-model **hard** (F25)
- [x] Full gate: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit`
- [x] `ledgerful verify --scope full`
- [x] Pin decisions; conductor Completed; deferred strike T245
- [x] T243 shipped — no rewrite

---

## Live dogfood log (fill on go)

```
# dry-run (exit 0)
ai-brains harness install --harness all-ready --dry-run
# printed grok + agy (IDE + plugin bundle because antigravity-cli exists) + opencode; zero writes

# install --yes (exit 0) via target\debug\ai-brains.exe
# Installed grok + agy + opencode

# status — three ready ok
#   grok present=yes wiring=ok install_ready=true
#   agy present=yes wiring=ok install_ready=true
#   opencode present=yes wiring=ok install_ready=true
#   claude/codex present=yes wiring=missing install_ready=false (T253)

# doctor harness_wiring
#   severity=ok message="3/3 ready wired (2 pending backend support: claude, codex)"

# preflight --summary harness block
#   grok/agy/opencode wiring=ok (ready) — no ready-missing next
#   claude/codex missing pending T239+

# F24 file proof
#   ~\.gemini\antigravity-cli\plugins\ai-brains-capture\{plugin.json,hooks.json} exist
#   ~\.gemini\antigravity-cli\hooks.json NOT created
#   ~\.grok\hooks\ai-brains.json + wrappers + OpenCode plugin exist

# C7: no new C:\dev\AI-Brains\.grok\hooks / .opencode / .gemini
# Bake: $aiExe = 'C:\dev\AI-Brains\target\debug\ai-brains.exe'; JS bakedCli JSON.stringify; spawn("opencode") not baked
# Re-run harness install after cargo install so bake points at the installed exe.
```

---

## Verification hints

Prefer:

- `cargo nextest run -p ai-brains-cli -E "test(harness) | test(doctor)"`
- Existing hermetic binaries: harness install suites, `doctor_cli`, preflight harness section

Hotspots: `install.rs` (large), `doctor.rs` matrix tests (message only), OpenCode plugin string (fragile raw string).

---

## Stop-before

Halt if:

- User has not said **go**
- Live install requested while another agent is mid-edit on `harness/install.rs` (check git status)
- AGY official CLI plugin schema drops `hooks.json` in bundle (re-research)
- Tempted to write undocumented top-level `antigravity-cli/hooks.json` (H1 — do not)
- Required secret / model for optional live fire (skip F24 turn; file-proof still required)
