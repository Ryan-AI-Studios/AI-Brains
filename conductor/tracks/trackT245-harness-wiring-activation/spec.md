# T245 — Harness wiring activation

- **Track ID:** T245-HarnessWiringActivation
- **Status:** ✅ **Completed** (PR #155 `f05e2f6`)
- **Category:** OPS / FEATURE / UX
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P1 — harness status **9/9** but **wiring=missing** for every `install_ready` harness (grok / agy / opencode)
- **Depends on:** T235 detect/install UX; T236 AGY Stop writer; T237 Grok install_ready; T238 OpenCode install_ready
- **Blocks / feeds:** Live message-only capture on this machine; doctor `harness_wiring` honesty; preflight next-action credibility
- **Absorbs:** deferred.md “Harness wiring=missing”; placeholder F1–F5; T238 **S12** (`session.status` idle dual-subscribe) as activation reliability; T235 doctor next-action weakness; PATH-loss silent skip (wrappers + OpenCode `spawn("ai-brains")`)
- **Not absorbed (DoD):** Claude/Codex `install_ready` (**T253**); project-scope install / Grok `/hooks-trust`; UserPromptSubmit (T237 S1); npm `@ai-brains/opencode-plugin` (T238 S5); child-session opt-in (S11); clap 5; auto-install without `--yes` / TTY / `auto_install`
- **Research date:** 2026-08-12 (live dogfood + official hook docs + crate pins + T235–T239 residuals)
- **AI fold-in:** 2026-08-12 `C:\dev\AI-review.md` **T245** AI1 + AI2. **H1 agreed:** retract undocumented top-level `antigravity-cli/hooks.json` dual-write; CLI path is **plugin bundle**. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start T245-harness-wiring-activation --category FEATURE`)
- **Isolation:** T243 **Completed** (PR #153 `7a19d40`). No T243 rewrite. Coordinate only if residuals (F23 footer / F24 daemon `next_step`) overlap.

---

## 1. Objective

1. **Activate** ready harnesses (grok, agy, opencode) from **install_ready + wiring=missing** → **wiring=ok** on this machine via dry-run → `--yes`, user-global only (C7).
2. **Make activation honest:** doctor / preflight next-actions copy-paste a command that actually wires ready backends; do not count Claude/Codex pending as the same “missing wiring” bucket.
3. **Make activation durable:** hooks must still fire when interactive PATH is broken (recent live incident). Bake the installing `ai-brains` absolute path into wrappers / plugin spawn, with PATH fallback.
4. **Make AGY actually load on documented paths:** always write official IDE `~/.gemini/config/hooks.json`. When `~/.gemini/antigravity-cli` already exists, also stage the **CLI plugin bundle** `plugins/ai-brains-capture/{plugin.json,hooks.json}` ([CLI plugins doc](https://antigravity.google/docs/cli/plugins)). **Never** write undocumented top-level `antigravity-cli/hooks.json` (AI2 **H1** — that path would recreate false `wiring=ok`).
5. **Keep capture independence, consent, and message-only.** Zero new crates. No live hook writes until **go**.

---

## 2. Live baseline (re-scan 2026-08-12)

### 2.1 Dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| `harness status` | grok / agy / opencode **present=yes wiring=missing install_ready=true**; claude / codex present + **install_ready=false** (T239+ / T253) |
| `preflight --summary` | Harness block lists all five; next-action is per-harness `--dry-run` (correct opt-in) |
| Dry-run grok | `~\.grok\hooks\ai-brains.json` + `~\.ai-brains\hooks\grok-capture.ps1`; empty-stdout Stop contract |
| Dry-run agy | `~\.gemini\config\hooks.json` + `~\.ai-brains\hooks\agy-stop.ps1`; F34 map + `{"decision":"allow"}` |
| Dry-run opencode | `~\.config\opencode\plugins\ai-brains-capture.js`; `session.idle` only; **no** `opencode.json` rewrite |
| Live hook dirs | **`~\.grok\hooks` missing**; **`~\.gemini\config` missing**; **`~\.config\opencode\plugins` missing**; **`~\.ai-brains\hooks` missing**; **no** `harness_hooks.json` |
| AGY home that exists | `~\.gemini\antigravity-cli` (bin/brain/log/history.jsonl). **No** `plugins/`, **no** `settings.json`, **no** top-level `hooks.json`, **no** `~\.gemini\config` |
| Doctor `harness_wiring` | Soft ok: `5 present, 5 missing AI-Brains wiring (grok, agy, opencode, claude, codex); next: ai-brains harness status` — pending backends lumped with ready-missing; next-action is status, not install |
| Binaries | `grok 1.0.3`; `agy 1.1.12`; `opencode 1.18.16` |
| `--harness all` | Already implemented: ready writers run; pending print T239+ honesty (F14). No `all-ready` token |

### 2.2 Why “wiring=missing” is both product + ops

| Layer | Truth |
|-------|--------|
| Writers | **Shipped.** T235 AGY, T237 Grok, T238 OpenCode. Dry-run plans match. |
| Ops | Nobody ran `--yes` on this machine (non-TTY agents never prompt; F9/F24). |
| AGY load path | Official **IDE** hooks = `~/.gemini/config/hooks.json`. Official **CLI** hooks = `~/.gemini/antigravity-cli/plugins/<name>/{plugin.json,hooks.json}`. Current writer = IDE only. Current probe also treats undocumented top-level `antigravity-cli/hooks.json` as ok (**H1** false-positive if we wrote that file). |
| PATH | Wrappers `Get-Command ai-brains`; OpenCode `spawn("ai-brains", …)`. PATH loss → silent skip (stderr only). Just demonstrated this session. |
| Doctor | 5/5 “missing” mixes ready + T253 pending; next is `harness status` not a write command. |

T235–T238 closed **writers**. T245 closes **activation + load-path honesty + PATH durability**.

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Readiness | `harness/detect.rs` `install_ready` | `Agy \| Grok \| Opencode` |
| List resolve | `commands/harness.rs` `resolve_harness_list` | `all` → `HARNESS_ORDER`; no `all-ready` |
| AGY write | `install.rs` `agy_hooks_soot_path` | **Only** `.gemini/config/hooks.json` |
| AGY probe | `wiring.rs` `probe_agy` | config **or** `antigravity-cli/hooks.json` |
| Grok write | `grok_hooks_marker_path` + wrapper body | PATH `Get-Command`; empty stdout |
| OpenCode plugin | `opencode_plugin_js_body` | `session.idle` only; `spawn("ai-brains", …)` |
| Doctor | `doctor.rs` `check_harness_wiring` | Missing = Missing\|Partial\|Unknown **including pending**; next `harness status` |
| Preflight | `format_harness_summary_lines` | next-action from `next_action_for` (dry-run for ready-missing) |
| Consent | `run_install` | Non-TTY requires `--yes`; TTY ask once |
| C7 | install targets | User-global homes only |

### 2.4 Dependency / standards research (2026-08-12)

| Pin | Workspace / lock | Ecosystem | Action |
|-----|------------------|-----------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** | **No bump** (series non-goal; clap 5 forbidden) |
| `serde_json` | **1.0** / lock **1.0.150** | 1.0.151 train | **No bump** |
| `dirs` | **6.0** / lock **6.0.0** | 6.0.0 | **No bump** |
| `is-terminal` | **0.4** / lock **0.4.17** | 0.4.17 | **No bump** |

**Zero new crates.** PATH resolve + `current_exe` + existing `atomic_write_str`.

### 2.5 Official hook freeze (2026-08-12)

| Harness | Source | Freeze |
|---------|--------|--------|
| **Grok Build 1.0.3** | [docs.x.ai/build/features/hooks](https://docs.x.ai/build/features/hooks); changelog v1.0.3 same day | Personal JSON still `~/.grok/hooks/*.json`. Events include **Stop** + **SessionEnd**. stdin JSON + `GROK_*` env. **Passive events: stdout ignored; exit 0.** Project hooks need `/hooks-trust` — **stay user-global**. Changelog also allows `config.toml` hooks — **not** T245 SOOT (JSON file remains). Do **not** emit `decision` / `continue` / `hookSpecificOutput` (empty stdout stay). |
| **AGY 1.1.12** | [hooks](https://antigravity.google/docs/hooks) + [CLI plugins](https://antigravity.google/docs/cli/plugins) (re-fetched 2026-08-12 fold-in) | **IDE global:** `~/.gemini/config/hooks.json` (named Stop handlers). **CLI:** plugin bundle `~/.gemini/antigravity-cli/plugins/<name>/` with required `plugin.json` (`name` `^[a-zA-Z0-9-_]+$`) + optional `hooks.json`. CLI plugins page: hooks “inside a plugin’s `hooks.json` or … primary `settings.json`”. **No** official top-level `antigravity-cli/hooks.json`. stdin/fullyIdle/allow-stop unchanged. |
| **OpenCode 1.18.16** | [opencode.ai/docs/plugins](https://opencode.ai/docs/plugins/); AI2 SDK types / PR #40984 | Global plugins auto-load; **no** `opencode.json` rewrite. Official example still **`session.idle`**. **`session.idle` is not deprecated** (SDK `EventSessionIdle` + `EventSessionStatus` both live; PR #40984 emits **both** on idle). F9 = resilience + in-flight dedup, **not** deprecation response. `SessionStatus.type` is literal `"idle" \| "retry" \| "busy"` — **no** aliases. |

### 2.6 Prior residuals rolled in

| Source | Item | T245 disposition |
|--------|------|------------------|
| deferred.md / audit | wiring=missing despite install_ready | **DoD** live + product |
| Placeholder F1 | order grok → agy → opencode | **F1** |
| Placeholder F2 | `--yes` after dry-run; preflight opt-in | **F2** |
| Placeholder F3 | status ok + doctor improves | **F3 / F6** |
| Placeholder F4 | no repo pollution | **F4** |
| Placeholder F5 | `all-ready` batch | **Elevate F5 hard** (`--harness all-ready`) |
| T238 S12 | `session.status` idle | **Elevate F9 hard** |
| T235 F17 | doctor never fail | Keep severity; **change message + next** (F6) |
| T235 F12 | project trust note | Docs only; **not** project install |
| PATH incident | wrappers skip if `ai-brains` missing | **F8 hard** |
| T237 S1 UserPromptSubmit | — | Soft **F17** |
| Claude/Codex | T239+ / S8 | **T253** |
| AI-review.md T245 AI1+AI2 | H1 AGY path; M1–M4 / F8–F9 pins | **§14** — H1 retracts old F7 |

---

## 3. Problem analysis

1. **Writers exist; hooks were never written** on this machine (ops).
2. **AGY CLI documented load path is a plugin bundle**, not top-level `antigravity-cli/hooks.json`. Writing that undocumented file + probing it would recreate false `wiring=ok` (AI2 **H1**).
3. **PATH-relative spawn** makes live capture fail-open-silent after PATH breakage.
4. **Doctor lumps T253 pending with ready-missing** and points at `harness status` instead of install.
5. **OpenCode emits `session.idle` and `session.status` together** (PR #40984). Dual-subscribe is resilience + in-flight dedup — idle is **not** deprecated today.
6. Track is **activation UX + three small product honesty/durability fixes + live `--yes` on go** — not new capture parsers.

---

## 4. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Plan-only** | No production code, **no live `harness install` without `--dry-run`**, no writes under `~\.grok\hooks`, `~\.gemini\**\hooks.json`, `~\.config\opencode\plugins`, or `~\.ai-brains\hooks` until **go**. Dry-run research OK (already done). |
| **F1 — Order** | Recommended operator order **grok → agy → opencode**. `all-ready` iterates `HARNESS_ORDER` filtered by `install_ready()` (same order). |
| **F2 — Consent** | Real writes require `--yes` **or** TTY confirm **or** `auto_install: true`. Preflight TTY offer remains opt-in (unchanged F10/F24/F25). Agents/non-TTY must pass `--yes`. Decline does **not** block explicit `--yes`. |
| **F3 — Post-install probe** | After successful install of a ready harness, `probe_wiring` → **`ok`**. Prefs `installed_at` stamped (existing). Re-run idempotent (update wrapper/plugin bodies). |
| **F4 — C7** | User-global only. Never write repo `.grok/hooks`, `.agents/hooks.json`, or `.opencode/plugins`. Assert hermetic targets under temp home. |
| **F5 — `all-ready` (hard / AI2 L2)** | `resolve_harness_list`: match **`Some("all-ready")` before** `parse_harness_id_or_usage`. Token is **not** a `HarnessId`. `parse_harness_id("all-ready")` stays **Err** (exit 2 if used on reset-decline etc.). `all` unchanged. Dry-run prints three plans, zero writes. Order = `HARNESS_ORDER` filter `install_ready()` — **no extra sort** (AI2 O11). |
| **F6 — Doctor honesty (hard / AI1 M4 + AI2 L3)** | Soft **ok** severity unchanged. Filters on `HarnessStatus`: `ready_missing` = `present && install_ready && wiring != Ok`; `pending_present` = `present && !install_ready`; `ok_ready` = `present && install_ready && wiring == Ok`. **Pinned message:** if `ready_missing` nonempty → `{ok_ready}/{ready_present} ready wired, {n} ready missing ({names}); next: ai-brains harness install --harness all-ready --dry-run` then, if pending, ` {p} backend pending (T253): {pending_names}`. If ready all ok and pending exist → `{ok_ready}/{ready_present} ready wired ({p} pending backend support: {names})`. **Never** “5 missing wiring” treating Claude as installable. Matrix 15 unchanged. |
| **F7 — AGY documented paths (hard / AI2 H1)** | **Retract** top-level `~\.gemini\antigravity-cli\hooks.json` write/merge. **Always** merge official IDE `~\.gemini\config\hooks.json` (create `config/` only). **Iff** `~\.gemini\antigravity-cli` **already exists**: stage CLI **plugin bundle** `plugins/ai-brains-capture/plugin.json` + `hooks.json` (Stop handler = same command as IDE). **Never** create `antigravity-cli` just to host plugins. **Never** write top-level CLI `hooks.json`. Do **not** shell out to `agy plugin install` (write staged layout; capture independence). `plugin.json`: `{ "$schema": "https://antigravity.google/schemas/v1/plugin.json", "name": "ai-brains-capture", "description": "AI-Brains message-only capture (Stop hook)" }` — `name` matches `^[a-zA-Z0-9-_]+$`. Bundle `hooks.json` is **owned file** (not merge-into-foreign). `merge_agy_hooks_map` applies **only** to IDE config. Dry-run lists IDE path + bundle paths when CLI home exists. |
| **F7b — AGY probe honesty (hard / H1)** | `wiring=ok` if IDE config has managed key **or** plugin bundle exists (`plugin.json` name + `hooks.json`). **Do not** treat undocumented top-level `antigravity-cli/hooks.json` alone as ok (legacy file may exist; ignore for ok). `targets_for(Agy)` lists IDE path + (when CLI home exists) bundle `plugin.json`/`hooks.json` — not top-level CLI hooks. |
| **F8 — Bake CLI path (hard / AI1 M1 + AI2 M3/L4/L9/L12)** | Extract `is_ai_brains_exe(filename: &str) -> bool` (`ai-brains` \| `ai-brains.exe`). `resolve_cli_exe_for_wrapper()`: `current_exe()` **Err → None** (no unwrap); if `is_ai_brains_exe(file_name)` use it; else PATH resolve. Body builders accept `Option<&Path>` and return **`String`** (or keep no-arg as `None` wrapper). **6+ call sites + tests update.** Tests inject `Some(Path::new(r"C:\fake\ai-brains.exe"))` — **never** assert real `current_exe()`. **PS:** assign `$aiExe = '<path>'` (single quotes). **JS:** `JSON.stringify(baked)` / escaped backslashes. Bake **only** the `ai-brains` spawn (`install.rs` ~769). **Do not** bake `spawn("opencode", …)` export fallback (~669). Grok **hooks.json command** stays `powershell.exe -File "<wrapper>"` — **no `$`**. Fallback `Get-Command` / `spawn("ai-brains")` when bake is `None`. |
| **F9 — OpenCode `session.status` (hard / S12 / AI2 M1 M2 L5)** | Dual-subscribe: `session.idle` **or** (`session.status` **and** `status.type == "idle"`). **Justification:** resilience + both events fire together (PR #40984); **not** because idle is deprecated. `opencode_is_idle_event(event_type, status_type)` true iff `event_type` equals `session.idle` (case-insensitive) **or** (`session.status` and `status_type == Some("idle")` case-insensitive). **No** aliases (`done`/`finished`/`complete`). **`retry` and `busy` are not idle.** Plugin JS: `const statusType = event?.properties?.status?.type` (the **string**, not the object). Shared `inFlight` by sessionID. Fail-open. No `opencode.json` rewrite. |
| **F10 — Live dogfood (hard on go)** | Mutating, only with go. Order: dry-run `all-ready` → `install --harness all-ready --yes` → `harness status` all three **ok** → `doctor` ready-missing empty → `preflight --summary` no ready-missing next. Confirm **zero** new files under `C:\dev\AI-Brains` (C7). Record exact commands/outputs in plan. |
| **F11 — Capture independence** | Detect / install / doctor text must not open models, embeddings, or graph. Vault-path-free `harness *` unchanged. |
| **F12 — Zero new crates / no clap 5** | No pin bumps. |
| **F13 — T253 fence** | Claude/Codex stay `install_ready=false`. `all-ready` skips them. Do not implement their writers. |
| **F14 — Docs (hard)** | CAPABILITIES: IDE config + CLI **plugin bundle** (not top-level CLI hooks.json); bake; idle+status resilience; `all-ready`. OPERATIONS activation recipe. WORKFLOWS **Activate harness capture**. CHANGELOG T245. Skill one-liner. |
| **F15 — Preflight next-action** | Ready-missing stays **`--dry-run` first** (clig.dev). Do **not** jump to `--yes` from preflight. After ok, next = `harness status`. |
| **F16 — Idempotent bodies (AI2 L10)** | Reinstall overwrites managed wrappers + IDE merge + **owned** CLI bundle files. OpenCode marker stays **`// AI-Brains managed (T238)`** — stability contract, **do not** bump to T245. Foreign OpenCode same-name without header still refuse. |
| **F17 — Soft residuals** | UserPromptSubmit; project-scope + `/hooks-trust`; npm plugin; child opt-in; Grok `config.toml` hooks; live Stop/idle fire e2e (optional smoke F24); wrapper Unix `.sh` counterparts if missing (Windows is DoD). |
| **F18 — Uninstall (H1 / AI2 L7)** | IDE: remove managed key only; leave `{}` if empty; preserve foreign keys. CLI: `remove_dir_all` **only** `plugins/ai-brains-capture/` (our bundle). Do **not** delete `antigravity-cli` or sibling plugins. Wrapper deleted. Prefs `mark_uninstalled`. |
| **F19 — Hermetic** | Temp `USERPROFILE`+`HOME` + PATH scrub (T235 F32). Never touch real user harness homes in tests. |
| **F20 — High findings if…** | Live go without three `wiring=ok`; write undocumented top-level `antigravity-cli/hooks.json`; probe ok from that file alone; bake omitted; doctor “5 missing” including pending as installable; `all-ready` writes Claude; repo-local hooks; clap 5 / new crates; capture/graph dep; Grok stdout JSON; OpenCode `opencode.json` rewrite; treat `retry` as idle. |
| **F21 — Parallel** | Touches `install.rs`, `wiring.rs` probe/targets, `commands/harness.rs`, `doctor.rs` message only, docs, hermetic harness tests. T243 **shipped** — no rewrite. T249: no `doctor --summary`. |
| **F22 — Grok contract frozen** | Empty stdout + exit 0; Stop + SessionEnd; dedicated `ai-brains.json`. No `config.toml` writer. |
| **F23 — AGY payload frozen** | F34 map + allow-stop JSON + fullyIdle skip unchanged. Write **shape** is IDE merge + optional plugin bundle (not payload). |
| **F24 — Optional live Stop fire** | One real agy/grok/opencode turn after install is **not** DoD (batch import remains backstop; needs model/network). **Hard file proof on go:** if CLI home exists, staged `plugins/ai-brains-capture/{plugin.json,hooks.json}` exist. Soft: `agy plugin list` if the command works. Decline AI2 O12 elevating a live agent turn to hard. |
| **F25 — Ledger / review** | On go: ledger start **FEATURE**. Primary review FEATURE/OPS. Cross-model **hard** (writes user-global hook surfaces + PATH bake). |
| **F26 — Exit codes frozen** | status 0; install success 0; unknown harness **2**; refuse rewrite **1** + path; all-pending (not `all-ready`) 0 + summary. |
| **F27 — Determinism** | `HARNESS_ORDER`; sorted doc tables; stable next-action strings. |
| **F28 — Secrets** | Never write keys/tokens into hook/plugin/wrapper files. Baked path is a local exe path only. |
| **F29 — help (AI2 L8)** | `harness` after_help (`main.rs`) adds `--harness all-ready --dry-run`. **No** `help_ia.rs` Daily change (harness not in that string). |
| **F30 — Plan-only / go** | No production code until **go**. |

---

## 5. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | `install --harness all-ready --dry-run` in temp home: prints grok+agy+opencode plans; **zero** files created | Hermetic |
| **AC2** | `install --harness all-ready --yes` in temp home **with** `antigravity-cli` dir: grok marker + IDE `config/hooks.json` managed key + **plugin bundle** `plugins/ai-brains-capture/{plugin.json,hooks.json}` + OpenCode header; **no** top-level `antigravity-cli/hooks.json`; wiring **ok** for three; claude/codex untouched | Hermetic |
| **AC3** | Temp home **without** `antigravity-cli`: AGY writes **only** `config/hooks.json`; does **not** create `antigravity-cli` or `plugins/` | Hermetic |
| **AC4** | Uninstall agy: removes IDE managed key (foreign keys stay); deletes **only** `plugins/ai-brains-capture/`; sibling plugins remain | Hermetic |
| **AC5** | Wrapper/plugin bodies contain baked absolute path when helper returns `Some`; fallback string present when `None` | Unit on body builders |
| **AC6** | Grok command line still has **no `$`**; Grok wrapper stdout contract still empty | Unit + existing AC12 |
| **AC7** | OpenCode plugin listens for `session.idle` **and** idle `session.status`; in-flight shared | Install-body assert + unit if extracted |
| **AC8** | Doctor: 3 ready-missing + 2 pending → message lists ready-missing separately; next contains `all-ready --dry-run`; severity ok | Hermetic / unit |
| **AC9** | Doctor: 3 ready ok + 2 pending → not “missing wiring” for grok/agy/opencode; pending mentioned as T253 | Hermetic |
| **AC10** | Unknown `--harness foo` / `allready` → exit **2**; `all` still accepted | Existing + new case |
| **AC11** | Install never writes under repo tree (targets under home) | Hermetic C7 |
| **AC12** | Docs: CAPABILITIES + OPERATIONS + WORKFLOWS recipe + CHANGELOG | Review |
| **AC13** | Live on go: three `wiring=ok`; doctor ready-missing empty; no repo pollution | Manual plan evidence |
| **AC14** | Capture independence; vault-path-free `harness status` | Existing + grep |
| **AC15** | Full gate: fmt, clippy `-D warnings`, nextest workspace, deny, audit | CI / local |
| **AC16** | No `unwrap`/`expect` on touched production paths | clippy + review |
| **AC17** | OpenCode still refuses foreign same-name file without managed header | Existing refuse test green |
| **AC18** | Install never writes top-level `antigravity-cli/hooks.json`; probe does not mark ok from that file alone | Unit + hermetic |
| **AC19** | CLI `plugin.json` `name` is `ai-brains-capture` and matches `^[a-zA-Z0-9-_]+$` | Unit |
| **AC20** | `opencode_is_idle_event`: idle true; status+idle true; status+retry/busy/`done` false | Unit `#[case]` |
| **AC21** | `is_ai_brains_exe`: `ai-brains` / `ai-brains.exe` true; `ai_brains_cli-hash.exe` / `rustc.exe` false | Unit |

---

## 6. Pure helpers (preferred extraction)

TDD Red first:

```text
fn ready_harness_ids() -> Vec<HarnessId>
// HARNESS_ORDER.filter(|id| id.install_ready())

fn resolve_harness_list("all-ready") -> ready_harness_ids()
// match before parse_harness_id_or_usage

fn is_ai_brains_exe(filename: &str) -> bool
fn resolve_cli_exe_for_wrapper() -> Option<PathBuf>
// current_exe Err → None; filename gate; else PATH

fn agy_ide_hooks_path(home: &Path) -> PathBuf
// always .gemini/config/hooks.json

fn agy_cli_plugin_dir(home: &Path) -> Option<PathBuf>
// Some(.gemini/antigravity-cli/plugins/ai-brains-capture) iff antigravity-cli dir exists

fn doctor_harness_wiring_message(statuses: &[HarnessStatus]) -> String
// F6 filters + pinned next-action

fn opencode_is_idle_event(event_type: &str, status_type: Option<&str>) -> bool
// session.idle OR (session.status && status_type == idle); no aliases
```

---

## 7. Non-goals

- Claude/Codex writers or `install_ready` flip (**T253**)
- Project-local hooks; Grok `/hooks-trust` automation
- New capture parsers / F11 keep-rule changes
- Growing preflight JSON DTO with `harnesses[]` (T220 soft)
- `doctor --summary` (**T249**)
- Nightly Last Result / latency (**T247**)
- MSI / PATH registration / clap 5
- Rewriting T243 residuals
- Auto-install on every preflight without consent
- Creating `~\.gemini\antigravity-cli` solely to host plugins
- Writing undocumented top-level `antigravity-cli/hooks.json`
- Shelling out to `agy plugin install` as required DoD
- Live AGY agent turn as hard DoD (F24 optional)

---

## 8. Risks

| Risk | Mitigation |
|------|------------|
| AGY CLI ignores IDE `config/hooks.json` | **F7** plugin bundle on documented CLI path |
| Undocumented top-level CLI hooks.json false-ok | **F7/F7b** never write; never probe-ok from that file alone |
| Plugin bundle not registered until `agy plugin install` | Write staged layout (official install destination); F24 soft `agy plugin list`; docs next-action if list empty |
| `current_exe()` is test/rustc binary | `is_ai_brains_exe` + inject `Some` in tests (**F8**) |
| Baked path goes stale after `cargo install` | Reinstall updates bodies; docs say re-run `harness install` after CLI upgrade |
| `session.status` payload drift | Exact `"idle"` only; keep `session.idle`; fail-open (**F9**) |
| Double ingest idle+status | Shared in-flight + existing hook idempotency |
| Live `--yes` without go | **F0** / Phase 0 gate |
| T249 concurrent doctor edits | **F21** message-only; no new doctor flags |
| Grok Stop feedback JSON | **F22** empty stdout frozen |
| PATH bake contains `$` in Grok **command** | Command stays `-File wrapper`; bake lives **inside** `.ps1` |

---

## 9. Implement order (when go)

1. Pure helpers + Red (list token, AGY IDE vs plugin-dir, doctor message, idle event, `is_ai_brains_exe`).
2. Green: `all-ready`; IDE merge + CLI bundle + uninstall; probe honesty; doctor message.
3. Green: bake path (`String` bodies); OpenCode idle+status-idle; PS/JS escape.
4. Docs + after_help.
5. Live dogfood **on go** (F10 + F24 file proof).
6. Review + full gate. T243 shipped — no rewrite.

---

## 10. Contracts

No `ai-brains-contracts` DTO change. `harness status --format json` schema_version **1** unchanged. Doctor check id `harness_wiring` unchanged. AGY `targets` may add plugin-bundle paths when CLI home exists (same array field).

---

## 11. Tests

- Unit: helpers in §6 (`function__condition__expected_result`).
- Hermetic CLI: temp home + PATH scrub (`harness` / existing `preflight` harness fixtures).
- Keep T235–T238 install refuse / empty-stdout / allow-stop / foreign-plugin tests green.
- No real network; no writes outside tempdir.

---

## 12. Docs

| Doc | Change |
|-----|--------|
| `Docs/CAPABILITIES.md` | Activation; IDE + CLI **plugin bundle**; bake; idle+status resilience; `all-ready` |
| `Docs/OPERATIONS.md` | Activation recipe; AGY documented paths; reinstall after CLI upgrade |
| `Docs/WORKFLOWS.md` | **Activate harness capture** |
| `CHANGELOG.md` | T245 |
| `.claude/skills` / project skill | One-liner `all-ready --dry-run` |

---

## 13. Parallel / isolation

- T243 **Completed** PR #153 — no T243 rewrite.
- F29 is `harness` after_help only — no `help_ia.rs` Daily edit.
- T249 owns `doctor --summary`; T245 only `harness_wiring` message.

---

## 14. AI-review disposition (2026-08-12)

Source: `C:\dev\AI-review.md` **T245** AI1 + AI2 (not the earlier Ledgerful 0184 leftover). Official CLI plugins + hooks docs re-fetched this fold-in.

### AI1

| ID | Verdict | Action |
|----|---------|--------|
| Exec / baseline | **Agree** | Affirms F5/F6/F8/F9; F7 **superseded by AI2 H1** |
| M1 PS/JS escape + `resolve_cli_exe` | **Agree hard** | **F8** single-quote PS; `JSON.stringify` JS; `is_ai_brains_exe` |
| M2 `agy_hooks_write_paths` dual-write top-level CLI hooks.json | **Disagree target** | Guard “never create `antigravity-cli`” kept; **write target** is plugin bundle (**F7**), not top-level `hooks.json` |
| M3 OpenCode idle+status JS | **Agree** + AI2 pin | **F9** exact `"idle"`; no aliases |
| M4 doctor split | **Agree** | **F6** filters; do **not** use `present.len()` as ready denominator |
| L1 Grok no `$` | **Agree** | Already **AC6 / F8 / F22** |
| L2 docs | **Agree** | **F14** |
| O1 unit names | **Agree** | Plan Phase 1 names |

### AI2

| ID | Verdict | Action |
|----|---------|--------|
| **H1** undocumented `antigravity-cli/hooks.json` | **Agree hard** | Retract old F7. Official CLI path = **plugin bundle**. Verified [plugins doc](https://antigravity.google/docs/cli/plugins) 2026-08-12. |
| H1 option (a) plugin bundle | **Agree — absorb as F7** | Bounded: write staged `plugins/ai-brains-capture/` + `plugin.json`. No `agy plugin install` shell-out DoD. |
| H1 option (b) live-test config-only | **Decline as blocker** | Keep writing IDE config anyway; CLI still needs documented bundle if CLI home exists. Optional live fire stays F24. |
| H1 option (c) drop CLI write | **Decline** | Would leave this machine’s AGY CLI unwired (CLI home exists; no `config/` today). |
| M1 idle not deprecated | **Agree hard** | Correct §2.5 / F9 justification |
| M2 no status aliases | **Agree hard** | **F9 / AC20** |
| M3 body signature + `is_ai_brains_exe` | **Agree hard** | **F8 / AC21** |
| L1 T243 isolation moot | **Agree** | §13 updated |
| L2 `all-ready` list token | **Agree hard** | **F5** |
| L3 doctor helper filters | **Agree hard** | **F6** |
| L4 `current_exe` Err | **Agree hard** | **F8** |
| L5 JS `status.type` string | **Agree hard** | **F9** |
| L6 merge only IDE | **Agree** | **F7** |
| L7 uninstall plugin dir | **Agree** | **F18** |
| L8 after_help only | **Agree** | **F29** |
| L9 do not bake `opencode` spawn | **Agree hard** | **F8** |
| L10 marker stays T238 | **Agree** | **F16** |
| L11 cross-model hard | **Agree** | **F25** |
| L12 `&'static str` → `String` | **Agree** | **F8** |
| O12 elevate live AGY turn to hard | **Decline** | File-proof hard (F24); agent turn optional (model/network / Stop-Before) |

### Pins locked by fold-in

1. **F7/F7b:** IDE merge + CLI plugin bundle; never top-level CLI `hooks.json`; probe-ok only on documented paths.
2. **F8:** `is_ai_brains_exe`; body builders `Option<&Path> -> String`; bake `ai-brains` spawn only; PS single-quote / JS stringify.
3. **F9:** idle **or** status+`"idle"` only; not deprecation.
4. **F5:** `all-ready` matched before `parse_harness_id`.
5. **F6:** ready vs pending filters; pinned next `all-ready --dry-run`.
6. **F16:** OpenCode marker remains `(T238)`.

