# T249 — Scope / daemon / doctor presentation

- **Track ID:** T249-ScopeDaemonDoctorPresentation
- **Status:** ✅ **Completed** (2026-08-14 PR #163 `5fd264a`)
- **Category:** UX / FEATURE
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P2 — `scope resolve` **Q7** always JSON; `daemon status` **Q7**; no `doctor --summary` (clap **exit 2**)
- **Depends on:** T160 `emit_scope_human` + `ScopeResolvedResponse`; T180 §5 scope JSON freeze; T192 doctor + `DoctorReport` v1; T199 vault-independent status; T201 exit matrix; T204 help IA; T241 `policy_grants`; T246/T248 `auto` TTY SOOT
- **Blocks / feeds:** Operators can read scope on a TTY, skim doctor, and see a daemon next-step. **T250** (preflight density), **T251** (device), **T255** (nightly/router doctor ports + nightly JSON) stay separate.
- **Absorbs:** deferred.md “Scope/daemon/doctor presentation”; placeholder F1–F3 / AC1–AC3; README `scope`/`daemon status`/`doctor --summary` **Q7** trio
- **Not absorbed (DoD):** T166 planner/apply; T243–T248 product rewrite; T250/T251/T255; T199 probe policies / keyless liveness; T227 `OutputFormat::parse` silent-JSON (F34); T214 is-terminal → std; clap 5; contracts DTO / `api_version` / `schema_version` bump; daemon JSON; doctor model-port checks; `sc query` service detect; live `daemon start`/`install`
- **Research date:** 2026-08-14 (live dogfood + T160/T192/T199/T248 SOOT + CLIG + crates.io pins)
- **AI fold-in:** 2026-08-14 `C:\dev\AI-review.md` **T249** AI1 + AI2. No Highs. **Agree hard:** AI2 M1 case-sensitive `--format` AC; AI2 L1 `status_next_line` helper (no live-daemon Stopped hermetic); AI2 L2 `DoctorOptions.summary` constructor ripple; AI2 L3 `--format json --summary` + `--summary --fail-on-degraded`. **Agree:** AI2 L4 drop TempEnv mandate; AI2 L5 Start-here json lock. **Decline:** AI1 remapped ACs (keep AC1–AC16). Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start T249-scope-daemon-doctor-presentation --category FEATURE`)
- **Isolation:** Do **not** rewrite `resolve_scope` / grants / soft-resolve. Do **not** change T199 Status/Safety probe constants or backend retry loop. Do **not** add doctor checks. Do **not** start/stop/install the live daemon. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Make `scope resolve` human on a TTY.** Default `auto`: existing `emit_scope_human` when stdout is a TTY; existing pretty JSON when piped or `--format json`.
2. **Keep `daemon status` honest and add a next-step.** Same `Status: Running|Stopped` / backend / vault / PID lines. When Stopped, print `next: ai-brains daemon start`. Do not invent PID, uptime, or install state.
3. **Ship a real `doctor --summary`.** Today `--summary` is unknown (exit **2**). Compact one-block of the **same** 15-check report. Default doctor stays the full human listing. `--json` stays the full `DoctorReport`.
4. **Stay capture-independent.** Presentation only. No models, no graph rebuild, no new events, no new crates, no pin bumps.

---

## 2. Live baseline (re-scan 2026-08-14)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| `scope resolve` (no flags) | Pretty JSON wall. `api_version=1`, `scope=Repository:441837f6-…`, `confidence=High`, `authoritative=true`, one `explicit_project_id` evidence. Exit **0**. |
| `scope resolve --format human` | Already good: `scope:` / `confidence: High (authoritative)` / `evidence:`. **No** `next:`. |
| `daemon status` | `Status: Stopped`. LLM `:8081` Open. Embedding `:8083` Open. **No** vault section (T128). **No** PID. **No** `next:`. Exit **0**. |
| `doctor` | `status=degraded`, `checks=15`. Warns: `backup_recent`, `recovery_kit_event`, `graph_density`. Skips: `recovery_kit_file`, `integrity`. `daemon_reachable — down` is **ok** (info-only). Exit **0**. |
| `doctor --summary` | clap `unexpected argument '--summary'` → **exit 2**. |

### 2.2 Why the audit scored Q7

| Surface | Truth |
|---------|--------|
| Scope default | clap `default_value = "json"` on `ScopeCommands::Resolve`. Human path **exists** (`emit_scope_human`) but TTY operators never see it. |
| Scope parser | `OutputFormat::parse` maps unknown/`None` → **Json** (T227 F34). Do **not** change the shared parser. |
| Daemon | Already human and keyless (T199). Q7 is scannability: Stopped with no next-step; no grouping beyond the existing lines. |
| Doctor | Default human is already the 15-check audit (T192 F10). `--summary` is **missing**, not a thin path. Agents/operators who type it get usage **2**. |

### 2.3 Code truth

| Site | Role |
|------|------|
| `commands/scope.rs` `run_resolve` | `OutputFormat::parse(options.format.as_deref())` then local/daemon |
| `governed_common::emit_scope_human` | `scope:` / `confidence:` / warnings / alternatives / evidence / non-authoritative note |
| `ScopeCommands::Resolve` | `--format` `Option<String>` default `"json"` — no `value_parser` |
| `ScopeResolvedResponse` | contracts `api_version`; keys frozen by T180 |
| `commands/daemon.rs` `run_status` | T199 early-route; Status 1×300ms; backends T85/T94; vault T128/T199; Windows PID via soft `tasklist` |
| `DaemonCommands::Status` | **No** `--format` |
| `Doctor` clap | `--format human\|json`; `--json` override; **no** `--summary` |
| `doctor::emit_report` | JSON = pretty `DoctorReport`; human = header `checks=N` + all 15 + remediations |
| `DoctorReport` | `schema_version=1`; 15-check matrix locked by unit `health_check_order_names__fixed_matrix` |
| T204 `help_ia` | Start here includes `ai-brains scope resolve --format json` |

### 2.4 Honesty (do not “fix” here)

- Non-authoritative scope is **not** a grant. Pretty must keep the T160 note. Soft-resolve elsewhere stays T203/T226.
- `daemon status` remains a **liveness report** (exit 0 Running **and** Stopped). Do not flip Stopped to exit 1.
- `daemon_reachable — down` stays **ok** on doctor (never sole Fail).
- Doctor roll-up / `--fail-on-degraded` / `--full` unchanged.
- `AI_BRAINS_VAULT_KEY` silent-zero remains honesty-only (T199 F16).

---

## 3. Research (2026-08-14)

| Topic | Finding | Use in T249 |
|-------|---------|-------------|
| **[CLIG — Output](https://clig.dev/)** | Humans first; TTY heuristic; `--json` for structure; human output may evolve; suggest next commands; say just enough | Scope default **`auto`**. Doctor `--summary` = just enough. Daemon `next:` when Stopped. |
| **CLIG — Future-proofing** | Changing human output is usually OK; scripts should pin `--json` / `--format json` | Human paths are **not** wire contracts. JSON keys frozen. |
| **PROTOCOL-COMPAT §5** | Compact↔pretty without a flag is breaking. `scope resolve --format json` is already pretty. Graph/retention document TTY/pipe when default flips | Add TTY/pipe to the scope row. JSON **keys** + pretty style frozen. |
| **T246 / T248 SOOT** | `resolve_*_format(&str, is_tty)` + clap `value_parser`; no `OutputFormat::parse`; case-sensitive tokens; `format: String` not `Option` | Copy for scope only |
| **T227 F34** | Shared `OutputFormat::parse` unknown→Json | **Do not** edit it |
| **T199 F9 / F12 / F18** | Keep `Status: Running\|Stopped`. No new status JSON. `sc query` out of DoD | Additive `next:` only. No `--format`. No service probe |
| **T192 F10** | Doctor default human; `--json` override; **no** TTY-smart doctor default | `--summary` is opt-in. Do **not** TTY-switch doctor |
| **T180** | Scope JSON keys + pretty style via production `--format json` | Hermetic AC keeps `--format json` |
| **clap** | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (2026-08-06). clap **5 not released** | `value_parser`; **no bump** |
| **serde_json** | lock **1.0.150** / crates.io **1.0.151** | **No bump**; keep `emit_json` / doctor pretty JSON |
| **chrono** | lock **0.4.44** / crates.io **0.4.45** | Unused unless we format doctor timestamps (we do not) |
| **is-terminal** | lock **0.4.17**. Crate docs prefer `std::io::IsTerminal` since 1.70 | Keep crate (T246/T248 SOOT). Migration remains **T214 F24** |
| **rustc** | toolchain **1.95.0** | Unchanged |
| **comfy-table / color / pager** | New crates / CLIG optional | **Forbidden** |
| **T255** | Doctor **model ports**; nightly `--status --format json`; embed sleep | **Not** this track |
| **systemd / sc status** | Analog is next-step, not a new probe | `next: ai-brains daemon start` — do not `sc query` |

---

## 4. Findings (DoD)

| ID | Severity | Requirement |
|----|----------|-------------|
| **F1** | Hard | `scope resolve` `--format: String` (not clap enum; not `Option`) default **`auto`**, `value_parser = ["auto","pretty","human","text","json","markdown","md"]`. Signature: `pub(crate) fn resolve_scope_format(explicit: &str, is_tty: bool) -> &'static str`. Resolve: `pretty`\|`human`\|`text`\|`markdown`\|`md` → human; `json` → existing pretty JSON; `auto` → TTY human else JSON. Probe `std::io::stdout().is_terminal()` via **`is_terminal::IsTerminal`**. Invalid token → clap usage **exit 2** (case-sensitive: `JSON`/`Pretty` fail). **No `other` passthrough**. Does **not** call `OutputFormat::parse`. Ripple: `ResolveOptions.format: String` + `main.rs` dispatch. After clap, map `"human"` → `OutputFormat::Human` and `"json"` → `OutputFormat::Json` for existing emit/fail helpers. |
| **F2** | Hard | Human path **reuses** `emit_scope_human` field order: `scope:` → `confidence: {c} (authoritative\|NOT authoritative)` → `warnings:` → `alternatives:` → `evidence:` → existing non-authoritative note. Extract `pub(crate) fn format_scope_human(resp: &ScopeResolvedResponse) -> String` (units without stdout). **Additive last line** when `!authoritative`: `next: ai-brains project whoami`. Omit `next:` when authoritative. **Never** invent a scope UUID or `policy bootstrap --scope` line. Empty `scope` still prints `scope:` (possibly blank) + the T160 note. JSON **keys frozen**: `api_version`, `scope`, `confidence`, `authoritative`, `evidence`, `warnings`, `alternatives`. `api_version` stays `"1"`. `emit_json` stays `to_string_pretty`. Human path is **not** a wire contract. |
| **F3** | Hard | Do **not** change `resolve_scope`, `map_resolved_scope`, daemon `ResolveScope` / `ScopeResolved`, soft-resolve (`resolve_scope_key_for_cli`), grants, or `#20` `authoritative: false` on Low/Ambiguous. Local vs `--daemon` / `--local` / `--require-daemon` path policy unchanged. |
| **F4** | Hard | `daemon status` keeps **exact** existing lines: `Status: Running` / `Status: Stopped`; `LLM backend …` / `Embedding backend …` (`{name} {addr} [{desc}]: {state}`); vault `Vault:` / `Vault size:` / `Memories:` (T128/T199 rules); `PID: {n}` only when tasklist yields one. **Additive:** when Stopped, last line `next: ai-brains daemon start`. When Running, **omit** `next:`. Extract `pub(crate) fn status_next_line(is_running: bool) -> Option<&'static str>` (`Some("next: ai-brains daemon start")` iff `!is_running`) — same testable-helper precedent as `format_status_vault_section` (AI2 L1). **Do not** add a hermetic that asserts `Status: Stopped` against a live daemon (T199 already skips). Unit AC7 on the helper. No `--format`. No JSON. No TTY switch. Exit **0** both states. No key required (T199 F1/F2). |
| **F5** | Hard | Daemon honesty: do **not** print `PID:` / uptime when unknown. Do **not** `sc query` / schtasks as DoD (T199 F18). Do **not** change Status/Safety probe constants, backend 5-attempt jitter, or `run_start`/`stop`/`install`/`update`. Do **not** start, stop, or install the live daemon. |
| **F6** | Hard | Real `doctor --summary` (`bool`, default false). Default human listing **unchanged** (`doctor: status=…  vault=…  checks=N` + all 15 + `         remediation:`). Summary is presentation of the **same** `DoctorReport`: one header `doctor: status={ok\|degraded\|fail}  vault={path}  ok={n} warn={n} fail={n} skip={n}` then either `No issues.` (zero warn+fail) **or** an `attention:` block listing **warn + fail only** in existing matrix order, same `[sev] name — msg` + indented `remediation:` lines. Skips are counted, not listed. **JSON wins** for both `--json` **and** `--format json`, with or without `--summary` (AI2 L3): always full `DoctorReport` (`schema_version=1`, full `checks`). No new DTO. No `schema_version` bump. Extract `pub(crate) fn format_doctor_summary(report: &DoctorReport) -> String`. Adding `DoctorOptions.summary` ripples **every** `DoctorOptions { … }` constructor (9 in `doctor.rs` tests + `main.rs` dispatch — AI2 L2; compiler catches misses). |
| **F7** | Hard | Doctor isolation: **no** new checks; **no** matrix reorder (15 names stay); **no** remediation string edits; roll-up / `exit_code_for` / `--fail-on-degraded` / `--full` / `--backup-max-age` / `--kit-path` unchanged. Do **not** add model-port checks (**T255 F1**). Do **not** add `retention_plan`. Do **not** TTY-switch doctor default (T192 F10). |
| **F8** | Hard | Zero new crates; **no version pin bumps**; no CLI reqwest; no contracts field change; capture-independent. clap 5 is **not released** (max 4.6.6). |
| **F9** | Hard docs | CAPABILITIES OutputFormat table: `scope resolve` TTY human / pipe JSON; `doctor` row notes `--summary`; daemon status notes Stopped `next:`. PROTOCOL-COMPAT §5 scope row: TTY/pipe split; JSON **keys** unchanged; JSON stays `to_string_pretty`; **human path is not a wire contract**; `--format` tokens **case-sensitive** (`JSON` / `Pretty` exit 2 — AI2 M1). OPERATIONS: TTY vs `--format json` for scope; `doctor --summary`; daemon `next:` when Stopped. Skill one-liner. Repo-root `CHANGELOG.md` T249 row only. Scope after_help: TTY example + `--format json`. Doctor after_help: add `--summary`. T204 Start here: **keep** `ai-brains scope resolve --format json`; **may add** `ai-brains scope resolve` and/or `ai-brains doctor --summary` without removing existing lines. **Lock** the json example with `ROOT_AFTER_LONG_HELP.contains("ai-brains scope resolve --format json")` in `help_ia.rs` units (AI2 L5). |
| **F10** | Hard verify | `protocol_compat_cli` `t180_c_scope_json_pretty__*` stays green. `governed_surface` `cli_scope_resolve__json__*` stays green. `doctor_cli` default listing + `--fail-on-degraded` stay green. Daemon `daemon_status_vault_independence` + smoke T85/T94/T128 stay green. T204 `cli_help_ia` still has `Start here:` **and** `scope resolve --format json` (AC lock). AC16 case-sensitive hermetic required. |
| **F11** | Hard stop-before | Do **not** run live `daemon start` / `install` / `stop`. Do **not** `graph rebuild`. Do **not** rewrite T243/T245/T246/T247/T248 product. Do **not** print or commit `AI_BRAINS_KEY`. |
| **F12** | Soft residual | Daemon uptime; `sc query` / service-registered line; daemon `--format json` (T199 F12 leftover); daemon `--quick` probes; doctor summary compact JSON DTO; T214 is-terminal → std; shared `resolve_*_format` helper; T204 Start here rewrite that **removes** the json example; color / pager / `comfy-table` |
| **F13** | Soft residual | T241 F20–F22 / L1–L2; T226 O1 shared resolve wrapper; T255 nightly/router batch; T250 preflight density |

---

## 5. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `resolve_scope_format` — `auto`+TTY → human; `auto`+pipe → json; `pretty`/`human`/`text`/`markdown`/`md` → human (TTY or not); `json` → json. |
| **AC2** | Unit: `format_scope_human` on authoritative fixture — contains `scope:` + `authoritative` + evidence; does **not** contain `next:`. Non-authoritative fixture — contains `NOT authoritative` + the T160 note + exact `next: ai-brains project whoami` as last non-empty line. |
| **AC3** | Hermetic: `scope resolve --format json --local` parses JSON with T180 keys; pretty (contains newline). |
| **AC4** | Hermetic: `scope resolve --format human --local` (or `pretty`) contains `scope:` and `confidence:`; stdout is **not** a JSON object. |
| **AC5** | Hermetic: `scope resolve --format xml` (or `yaml`) exit **2**. Zero stdout JSON. |
| **AC16** | Hermetic: `scope resolve --format JSON` **and** `--format Pretty` each exit **2** with zero stdout JSON (AI2 M1 — AC5 `xml`/`yaml` cannot catch a regression that restores `OutputFormat::parse` lowercasing). |
| **AC6** | Hermetic: `scope resolve --local` with no `--format` (piped, `is_terminal=false`) parses as JSON (auto→json). |
| **AC7** | Unit on `status_next_line(false) == Some("next: ai-brains daemon start")` and `status_next_line(true).is_none()`. Do **not** require a hermetic that forces Stopped while a live daemon may be Running. Existing `Status: Running` / `Status: Stopped` strings unchanged. |
| **AC8** | Existing daemon no-key + T85/T94/T128 suites stay green (backend line shape, vault-only-when-running, exit 0). |
| **AC9** | Unit: `format_doctor_summary` on a 15-check degraded fixture — header has `ok=`/`warn=`/`fail=`/`skip=`; `attention:` lists warn+fail names only; skips absent from attention; remediations use `         remediation:`. All-ok fixture contains `No issues.` and no `attention:`. |
| **AC10** | Hermetic: default `doctor` (no `--summary`) still prints `checks=15` (or the live matrix count) and includes `vault_exists`. |
| **AC11** | Hermetic: `doctor --summary --json` **and** `doctor --format json --summary` each parse as `DoctorReport` with `schema_version=1` and a `checks` array (full, not a compact DTO — AI2 L3). |
| **AC12** | Hermetic: `doctor --summary` is accepted (not clap exit 2). Degraded + `--summary` still exit **0**. Degraded + `--summary --fail-on-degraded` still exit **1** (AI2 L3). |
| **AC13** | Live (on go): TTY `scope resolve` is human; piped `scope resolve` is JSON. TTY `daemon status` while Stopped shows `next:`. TTY `doctor --summary` is shorter than default doctor and lists this vault’s warn names. Do **not** start the daemon. |
| **AC14** | Docs: CAPABILITIES OutputFormat + OPERATIONS + PROTOCOL-COMPAT §5 + CHANGELOG T249 |
| **AC15** | Existing `cargo nextest run -p ai-brains-cli --test protocol_compat_cli --test governed_surface --test doctor_cli --test daemon_status_vault_independence` stay green |

---

## 6. Non-goals

- Changing scope resolution, grants, or soft-resolve
- Daemon JSON / `--format` / uptime / `sc query` / probe retune
- Doctor new checks (model ports, retention, policy install-grants)
- TTY-switching default `doctor`
- Compact doctor JSON DTO
- clap 5 / serde_json / chrono / is-terminal bumps
- `comfy-table`, color, pager
- T250 / T251 / T255 / T243–T248 rewrite
- Live `daemon start` / `install` / `graph rebuild`
- Printing vault keys

---

## 7. Capture independence / contracts / exits

| Topic | Rule |
|-------|------|
| Capture | Presentation only. No new events. No models/graph |
| Contracts | **No** `ScopeResolvedResponse` / `DoctorReport` field or version change. CLI-local formatters only |
| Exits | Scope success **0**; bad `--format` **2**; daemon Running/Stopped **0**; doctor ok/degraded **0** (unless `--fail-on-degraded`); doctor fail **1** |
| Privacy | Do not print keys, recovery passphrases, or check messages that contain secrets (existing doctor rule) |

---

## 8. File touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/main.rs` | Scope default `auto` + `value_parser`; `Doctor { summary: bool }`; after_help examples |
| `crates/ai-brains-cli/src/commands/scope.rs` | `resolve_scope_format` + `format: String` |
| `crates/ai-brains-cli/src/commands/governed_common.rs` | `format_scope_human` + non-authoritative `next:` |
| `crates/ai-brains-cli/src/commands/daemon.rs` | Stopped `next:` line only |
| `crates/ai-brains-cli/src/commands/doctor.rs` | `--summary` emit path + `format_doctor_summary` |
| `crates/ai-brains-cli/src/help_ia.rs` | Additive Start here lines only (keep json example) |
| `crates/ai-brains-cli/tests/` | hermetic scope auto/human/json/reject + doctor summary + daemon next |
| `Docs/CAPABILITIES.md` | OutputFormat rows + short notes |
| `Docs/PROTOCOL-COMPAT.md` | §5 scope TTY/pipe |
| `Docs/OPERATIONS.md` | TTY vs json; `--summary`; daemon `next:` |
| `.agents/skills/ai-brains/SKILL.md` | one-liner |
| `CHANGELOG.md` | T249 row |
| `conductor/*` | status / deferred / README |

**Do not touch:** `resolve_scope` / policy evaluator; `contracts/scopes.rs` / `contracts/doctor.rs` fields; `daemon_probe.rs` constants; T243–T248 product files; nightly/router; desktop.

---

## 9. Verification plan

```powershell
# Units
cargo nextest run -p ai-brains-cli scope
cargo nextest run -p ai-brains-cli doctor
cargo nextest run -p ai-brains-cli daemon
cargo clippy -p ai-brains-cli --all-targets -- -D warnings

# Hermetic / existing
cargo nextest run -p ai-brains-cli --test protocol_compat_cli --test governed_surface --test doctor_cli --test daemon_status_vault_independence

# Live on go (do not start daemon)
ai-brains scope resolve
ai-brains scope resolve | ConvertFrom-Json | Select-Object api_version, authoritative
ai-brains scope resolve --format json
ai-brains scope resolve --format pretty
ai-brains scope resolve --format xml   # expect exit 2
ai-brains scope resolve --format JSON  # expect exit 2 (case-sensitive)
ai-brains daemon status
ai-brains doctor
ai-brains doctor --summary
ai-brains doctor --summary --json

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 10. Risk / review

- **Category:** FEATURE / UX (not SECURITY). Cross-model useful: JSON key freeze + doctor roll-up + T199 liveness non-regression.
- **Highest regression:** flipping piped `scope resolve` off JSON; changing `OutputFormat::parse`; doctor `--summary --json` emitting a new DTO; Stopped daemon exit ≠ 0; adding a 16th doctor check; TTY-switching doctor; starting the live daemon.
- **Cap deferred mediums:** ≤3; presentation softs go to F12–F13 / deferred.md.

---

## 11. Suggested implement order (locked)

1. Pure `resolve_scope_format` + `format_scope_human` (Red → Green units, no vault)
2. Wire scope clap `auto` / `value_parser` + hermetic json/human/reject
3. Daemon Stopped `next:` helper + unit; do not retune probes
4. `format_doctor_summary` units then clap `--summary` + hermetic
5. Docs

---

## 12. Placeholder disposition

| Draft | Disposition |
|-------|-------------|
| F1 Scope: pretty default TTY; keep JSON machine | **Absorbed** F1 / F2 |
| F2 Daemon: Status + backends + schedule hint one screen | **Absorbed** F4 as Stopped `next:` (not `sc query`) |
| F3 Doctor summary: real flag or documented absence | **Absorbed** F6 **real flag** (audit is exit 2) |
| AC1 Scope pretty + json | **Absorbed** AC1–AC6 |
| AC2 Daemon status scannable | **Absorbed** AC7 / AC8 |
| AC3 Doctor summary disposition | **Absorbed** AC9–AC12 |

---

## 13. Deferred fold-in

| Item | Source | Disposition |
|------|--------|-------------|
| Scope/daemon/doctor presentation | deferred.md T249 / README Q7 | **DoD** F1–F7 / AC1–AC13 |
| Placeholder F1–F3 | spec draft | All absorbed (doctor = real flag) |
| T160 human path unused as default | T160 / live | **F1** default `auto` |
| T180 scope JSON pretty + keys | T180 | **F2** / **AC3** — do not change style |
| T192 doctor human default / no TTY flip | T192 F10 | **F6/F7** summary opt-in only |
| T199 keyless status + no JSON | T199 | **F4/F5** additive `next:` only |
| T241 doctor `policy_grants` | T241 shipped | **Not absorbed** — already on matrix; F13 soft leftovers |
| T226 O1 shared resolve wrapper | T226 | **Not absorbed** F13 |
| T227 F34 OutputFormat silent-JSON | T227 | **Not absorbed** — local resolver only |
| T214 is-terminal → std | T214 F24 | **Not absorbed** F12 |
| T248/T246 format SOOT | peers | **Copy** F1; do not share helper as DoD (F12) |
| T255 doctor ports / nightly JSON / embed sleep | T255 | **Not absorbed** F7 / F13 |
| T250 preflight density | T250 | **Not absorbed** |
| Doctor `retention_plan` check | T248 F16 | **Not absorbed** F7 |
| Daemon uptime / service query | placeholder “richer” | **F12 soft** |

---

## 14. AI fold-in disposition (2026-08-14) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates planned work (resolver, formatters, `--summary`, daemon `next:`). AI2 M1 is a real test-gap: case-sensitivity is specified but AC5 `xml`/`yaml` cannot catch a lowercase restore. AI1 remapped ACs declined (T248 repeat).

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** | AI1 | **Agree** (already F1) | 2-arg `resolve_scope_format`; clap `value_parser`; **no** `other` passthrough. AI1 `main.rs:920-940` line cite is stale (Resolve is ~1006). |
| **AI1 M2** | AI1 | **Agree** (already F2/F6) | `format_scope_human` + `format_doctor_summary` |
| **AI1 M3** | AI1 | **Agree** (already F6) | Summary header/attention/`No issues.`; JSON wins |
| **AI1 M4** | AI1 | **Agree** (already F4) | Stopped `next:`; Running omit |
| **AI1 L1** | AI1 | **Agree** (already F4) | Exit **0** both states |
| **AI1 L2** | AI1 | **Agree** (already F9) | Docs; they omitted skill — keep skill |
| **AI1 O1** | AI1 | **Agree** | Already AC1/AC2/AC9; names are implementer choice |
| **AI1 remapped ACs** | AI1 | **Decline** | Keep AC1–AC16 (their AC6–AC12 collide with ours) |
| **AI2 M1** | AI2 | **Agree hard** | **AC16** `--format JSON` / `Pretty` exit 2 |
| **AI2 L1** | AI2 | **Agree hard** | F4 `status_next_line`; no live-daemon Stopped hermetic |
| **AI2 L2** | AI2 | **Agree hard** | F6 constructor ripple — **9** `doctor.rs` sites + `main.rs` (AI2 said 7; grep is 9+1) |
| **AI2 L3** | AI2 | **Agree hard** | AC11 `--format json --summary`; AC12 `--summary --fail-on-degraded` |
| **AI2 L4** | AI2 | **Agree** | Phase 2: **no** TempEnv mandate (pure + `cmd.env`) |
| **AI2 L5** | AI2 | **Agree** | F9/F10 `help_ia` contains lock for `--format json` |

### Pins locked by fold-in

1. **AC16:** `JSON` / `Pretty` clap exit **2** (case-sensitive).
2. **F4:** `status_next_line(is_running)` unit; no Stopped hermetic vs live daemon.
3. **F6/AC11/AC12:** JSON wins for `--json` **and** `--format json`; `--summary --fail-on-degraded` still 1.
4. **F6:** `DoctorOptions.summary` on every constructor.
5. **F9:** Start-here json example locked in `help_ia` units.
6. **Phase 2:** no TempEnv mandate.
