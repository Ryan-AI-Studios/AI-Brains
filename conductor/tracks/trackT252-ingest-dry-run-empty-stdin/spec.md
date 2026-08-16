# T252 — Ingest dry-run empty stdin honesty

- **Track ID:** T252-IngestDryRunEmptyStdin
- **Status:** ✅ **Completed** (2026-08-15)
- **Category:** UX / BUGFIX
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P3 — `ingest --dry-run` empty stdin **5/7** (E5/Q7: opaque EOF JSON error)
- **Depends on:** T86 stdin TTY guard pattern; T107 `--dry-run` no-write; T114 `DryRunIngestRequest` placeholder UUIDs; T180 dual-path deny_unknown_fields vs prod open; T198/T201/T202 `fail_usage` → exit **2**; T234 thinking field honesty
- **Blocks / feeds:** Operators who forget to pipe JSON get a copy-paste example instead of `COMMAND_FAILED` EOF. **T253** (Claude/Codex), **T254** (multi-root soft), **T255** (nightly/router) stay separate.
- **Absorbs:** deferred.md “ingest dry-run empty stdin”; placeholder F1–F3 / AC1–AC2; README `ingest --dry-run` empty stdin **5/7**
- **Not absorbed (DoD):** vault-free `ingest --dry-run` (still opens `AppContext`); `ingest --schema`; clap 5 / pin bumps; `std::io::IsTerminal` migrate (T249 F12); rewriting `parse_ingest_request` / `IngestRequest`; T180 deny_unknown_fields flip; T114 placeholder UUID rewrite; T107 write-isolation rewrite; T234 thinking populate; T86 `read_json_from_stdin` swallow; shared stdin helper; `events[0]` panic; T253–T255
- **Research date:** 2026-08-15 (live dogfood + ingest.rs / T86 / T114 / T180 / CLI-EXIT-CODES + crates.io pins)
- **AI fold-in:** 2026-08-15 `C:\dev\AI-review.md` **T252** AI1 + AI2. No Highs. **Agree hard:** AI2 quoting (single-quoted JSON, not unquoted ScriptBlock); AI2 after_help wrap (multiline + AC8 key presence); AI1 AC14 TTY dogfood; AI1 BREAKING grep of `COMMAND_FAILED` / hook ingest callers. **Agree:** AI1 T86 `read_json_from_stdin` swallow + shared stdin helper + `events[0]` panic = F12; AI2 vault-key gate already F9; AI2 empty-field vs empty-stdin already F13; AI2 fail_usage reuse + zero-UUID payload already F1/F5. **Decline:** AI1 flatten F6 (`is_tty` stays — unit-testable without mocking stdin); AI1 move `INGEST_EMPTY_STDIN_USAGE` to `governed_common.rs` (T202 consts live in the command module, `governed_query.rs`; do not grow hotspot); AI2 “single quotes work in cmd.exe” (cmd.exe echoes the quotes). Disposition **§12**.
- **Ledger:** plan TX `6cc460e2-6f28-40bb-bb38-0ca787ec4985` (DOCS). Fold-in TX `33e14b3a-8ec3-402d-bfdb-8c40b2460873` (DOCS). Implement go starts a new FEATURE/UX TX.
- **Isolation:** Do **not** change `IngestRequest` / `IngestResponse`. Do **not** change `parse_ingest_request` validation. Do **not** change T180 dual-path. Do **not** route ingest through `run_sync_path_free`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Empty or missing stdin is usage-class.** Piped empty / whitespace-only / interactive TTY must exit **2** with a copy-paste JSON example, not `Invalid JSON: EOF while parsing a value…` wrapped as `COMMAND_FAILED`.
2. **Valid dry-run still previews without write.** T107/T114 stay: placeholder UUIDs, stdout `[dry-run] Would ingest…`, exit 0, no event append.
3. **Mid-payload parse stays a payload error.** Non-empty garbage (`{`, `{not json`) keeps today’s machine `ApiResult` JSON on stderr and exit **1**.
4. **Stay capture-independent.** CLI usage only. No models, no graph, no new events, no new crates, no pin bumps, no DTO change.

---

## 2. Live baseline (re-scan 2026-08-15)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| `echo '' \| ai-brains ingest --dry-run` | `{"ok":false,"status":"error","message":"Invalid JSON: EOF while parsing a value at line 2 column 0","error":{"code":"COMMAND_FAILED","message":"Invalid JSON: EOF while parsing a value at line 2 column 0"}}` → **exit 1**. (`echo ''` is a newline; serde sees EOF at line 2.) |
| `ai-brains ingest --help` | About: “reads JSON from stdin”. Flag: `--dry-run`. **No** after_help example payload. Exit **0**. |
| `ai-brains ingest --dry-run` on TTY | Blocks on `read_to_string` until EOF (Ctrl+Z / Ctrl+D). Same EOF JSON after empty close. |
| Project identity | `.env` pins `test-alias` (`441837f6-…`). Unrelated to ingest stdin. |
| Doctor | degraded (backup_recent / recovery_kit / graph_density). Unrelated. |
| Daemon | Stopped. Ingest is local CLI; daemon not required. |

### 2.2 Why the audit scored 5/7

| Surface | Truth |
|---------|--------|
| Empty pipe | serde EOF is a parser accident, not operator guidance. |
| Interactive | Hang, then the same EOF error. Spec draft said “or interactive”. |
| Mid-payload | A real parse failure *should* stay a JSON error. Do not swallow `{` into usage. |
| `--help` | Names stdin, does not show a payload. |

### 2.3 Code truth

| Site | Role |
|------|------|
| `commands/ingest.rs` `run()` | `io::stdin().read_to_string` then, if `dry_run`, `serde_json::from_str::<DryRunIngestRequest>` mapped to `Invalid JSON: {e}`. No empty/TTY gate. `ctx` unused on dry-run. |
| `DryRunIngestRequest` | `deny_unknown_fields`; UUID fields are `String` (T114). Empty `content`/`role` → string errors. |
| `parse_ingest_request` (`ai-brains-capture` `malformed.rs`) | Live path. Required fields + role enum + UUID types. Empty string → `CaptureError`. |
| `main.rs` `Commands::Ingest` | `--dry-run` only. No after_help. Dispatch **after** `AppContext::from_cli` (vault+key required even for dry-run). |
| `handle_cli_result` | Generic `Box<dyn Error>` → `COMMAND_FAILED` `ApiResult` JSON on **stderr**, **exit 1**. `GovernedCliError` / `fail_usage` → printed message, exit **2**. |
| T86 `read_query_from_stdin` | TTY → error before read; empty query → error. Pattern to copy, not the function (query vs JSON). |
| Tests | `ingest_reads_json_stdin.rs` (happy / T114 placeholders / empty content / live UUID reject). `protocol_compat_cli.rs` T180 unknown-field dual-path. **No** empty-stdin case. |
| `cli_help_ia.rs` | Group order only (`ingest` is Harness). Does **not** snapshot ingest after_help. |

### 2.4 Honesty (do not “fix” here)

- Dry-run still requires a vault/key because dispatch is after `AppContext`. Empty-stdin usage after a successful vault open is the audit finding. Vault-free dry-run is **F12**.
- `echo ''` is whitespace, not a zero-length file. Treat `trim()` empty as empty.
- Empty **field** `content: ""` is a valid JSON object and stays today’s field error (exit 1), not usage 2.
- `AI_BRAINS_KEY` never printed.

---

## 3. Research (2026-08-15)

| Topic | Finding | Use in T252 |
|-------|---------|-------------|
| **[CLIG — Help your users succeed](https://clig.dev/#help)** | Errors should say what to do next; show examples | `fail_usage` with copy-paste JSON + `ingest --dry-run` |
| **CLIG — Human-readable / future-proof** | Human usage may evolve; scripts pin JSON | Empty/TTY → human stderr (exit 2). Mid-payload → frozen `ApiResult` JSON (exit 1) |
| **CLIG — Suggest next commands** | Name the command to run | Example line includes `ai-brains ingest --dry-run` |
| **T86** | TTY guard before `read_to_string`; empty after trim | Same gate for ingest; do **not** reuse query helpers |
| **T202 `fail_usage`** | `PROGRESSIVE_PROJECT_USAGE` style: problem + `Example:` + command | `INGEST_EMPTY_STDIN_USAGE` same shape |
| **T107 / T114** | Dry-run preview, no write; placeholder UUIDs; `DryRunIngestRequest` | Leave structurally; only gate *before* `from_str` |
| **T180 F26** | Dry-run `deny_unknown_fields`; prod open | Regression ACs only |
| **CLI-EXIT-CODES** | `fail_usage` → **2**; generic parse → **1** `COMMAND_FAILED` | Empty/TTY = new fail_usage site; mid-payload unchanged |
| **clap** | Workspace `4.5` / lock **4.6.1** / crates.io **4.6.6** (2026-08-06). clap **5 not released** | Additive `after_help` only. **No bump** |
| **serde_json** | lock **1.0.150** / crates.io **1.0.151** | **No bump**; no DTO |
| **is-terminal** | lock **0.4.17** (T86 already uses it) | Use the same crate. `std::io::IsTerminal` is T249 F12 residual — **not** DoD |
| **assert_cmd** | lock **2.2.2** | Existing hermetic helper |
| **rustc** | **1.95.0** | Edition 2024 unchanged |
| **CI tools** | nextest **0.9.140** / deny **0.20.2** / audit **0.22.2** | No tool bump |
| **Capture independence** | `ai-brains-capture` tests prove ingest does not need models/graph | CLI gate only; do not touch capture crate |

---

## 4. Frozen decisions (F1–F16)

| ID | Decision |
|----|----------|
| **F1 — Empty/whitespace stdin is usage (hard)** | After a successful read, if `input.trim().is_empty()`, call `fail_usage(INGEST_EMPTY_STDIN_USAGE)`. **Stderr** human, **zero stdout**, exit **2**. Applies to **`ingest` and `ingest --dry-run`**. Do **not** emit `COMMAND_FAILED` / `Invalid JSON: EOF`. |
| **F2 — Valid dry-run preview frozen (hard)** | Existing T107/T114 path unchanged after the gate: `DryRunIngestRequest`, placeholder UUIDs, empty `content`/`role` field errors, stdout `[dry-run] Would ingest…`, exit 0, no `append_event`. |
| **F3 — Mid-payload parse keeps envelope (hard)** | Non-empty input that fails JSON (`{`, `{not json`, truncated object) stays `Invalid JSON: …` → `handle_cli_result` `ApiResult` JSON on **stderr**, exit **1**. Do **not** map “any serde error” to usage 2. |
| **F4 — TTY refuses before read (hard)** | If `stdin().is_terminal()`, **do not** `read_to_string`. Same `INGEST_EMPTY_STDIN_USAGE` (one SOOT for empty + interactive). Use `is_terminal::IsTerminal` like T86. No hang. |
| **F5 — Example payload SOOT (hard)** | One `const INGEST_EMPTY_STDIN_USAGE` in **`ingest.rs`** (command-local, same as T202 `PROGRESSIVE_PROJECT_USAGE` in `governed_query.rs` — **not** `governed_common.rs`). Same JSON in `ingest --help` `after_help`, formatted as **multiline indented** object (AI2 wrap — do not rely on one 200-char line). Payload uses hermetic zero UUIDs so copy-paste works for **live** ingest *and* `--dry-run`. Required keys: `session_id`, `project_id`, `harness_id`, `turn_id`, `role`, `content`, `privacy`. Example command line names `ai-brains ingest --dry-run`. **Quoting (AI2 L1):** wrap the JSON in **single quotes** (`echo '{…}' \| …`). Unquoted `{…}` is a PowerShell ScriptBlock. Do **not** claim cmd.exe compatibility (cmd.exe would echo the quotes). |
| **F6 — Pure gate helper (hard)** | Keep `pub(crate) fn ingest_stdin_needs_usage(is_tty: bool, raw: Option<&str>) -> bool` — `true` if `is_tty` **or** `raw` is `None`/trim-empty. Units cover TTY / empty / whitespace / `{`. `run()` calls it; do not inline trim in two places. **Do not** flatten to “check `stdin().is_terminal()` inside” (AI1 suggestion declined — that makes TTY untestable without a real TTY). |
| **F7 — Docs (hard)** | CLI-EXIT-CODES `fail_usage` footnote: empty/TTY ingest → **2**. CAPABILITIES §4 ingest bullet. OPERATIONS one-liner after the existing `echo $json \| … ingest` sample. CHANGELOG Unreleased: **BREAKING** empty-stdin exit **1** JSON → exit **2** human (0.x allowed). **Before** the BREAKING line: grep hooks / `ingest.ps1` / nightly / `scripts/*` for empty-stdin or `COMMAND_FAILED` consumers (AI1). Record the grep in `plan.md` Phase 3. |
| **F8 — No contract / daemon / DTO change (hard)** | `IngestRequest` / `IngestResponse` frozen. No `ai-brains-contracts` growth. No daemon ingest path. No `--schema`. |
| **F9 — Vault still required (hard)** | Keep ingest behind `AppContext`. Do **not** add ingest to `is_vault_path_free` / `run_sync_path_free`. Missing key still `VAULT_KEY_MISSING` exit **1** (before stdin). |
| **F10 — Pins (hard)** | No workspace/lock bumps. clap 4.5 / lock 4.6.1; serde_json 1.0.150; is-terminal 0.4.17. |
| **F11 — Isolation (hard)** | Do not rewrite `parse_ingest_request`, T180 tests, T114 struct, T234 thinking, capture crate, graph hook, nightly/symbol ingest. Do not change `cli_help_ia` group order. |
| **F12 — Soft residuals** | Vault-free `--dry-run`; `ingest --schema`; `std::io::IsTerminal`; clap 4.6 workspace pin; T86 `read_json_from_stdin` swallows parse errors (`unwrap_or(Object)` — preflight `--stdin`, not ingest); shared `read_stdin_trimmed` SOOT across T86+ingest (isolation — not DoD); `outcome.events[0]` panic if `primary_event()` is `None` and `events` is empty (`ingest.rs` response build — pre-existing, not empty-stdin); T253–T255 |
| **F13 — Empty field ≠ empty stdin (hard)** | JSON `{"…","content":"","…"}` stays `content field is empty` (or live validation), exit **1**. Existing `ingest__dry_run__errors_on_empty_content` stays green and must **not** contain `INGEST_EMPTY_STDIN_USAGE`. |
| **F14 — High findings** | Mapping all serde errors to usage 2; hanging on TTY; unquoted `{…}` usage example (PowerShell ScriptBlock); changing T180 deny_unknown_fields; skipping vault open for live ingest; printing the product key; adding crates; swallowing mid-payload as “empty”. |
| **F15 — Capture independence** | String gate + existing parse. No models, embeddings, graph rebuild, or new events on the usage path. |
| **F16 — Plan-only until go** | No production file change until the user says **go**. |

### `INGEST_EMPTY_STDIN_USAGE` (normative text)

```
stdin is empty or not piped. Pipe a JSON turn. Example:
  echo '{"session_id":"00000000-0000-0000-0000-000000000001","project_id":"00000000-0000-0000-0000-000000000000","harness_id":"00000000-0000-0000-0000-000000000002","turn_id":"00000000-0000-0000-0000-000000000003","role":"user","content":"hello","privacy":"CloudOk"}' | ai-brains ingest --dry-run
```

`after_help` prints the **same JSON multiline** (pretty-indented object, not the `echo` line). Tests assert the seven keys + `ingest --dry-run` + single-quoted `'{` in the **usage const only** — not exact `after_help` whitespace.

---

## 5. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Hermetic: `ingest --dry-run` + `write_stdin("")` → exit **2**. Stderr contains `stdin is empty or not piped` **and** `ingest --dry-run` **and** `session_id`. Stderr does **not** contain `COMMAND_FAILED` or `EOF while parsing`. Stdout empty. |
| **AC2** | Hermetic: `ingest --dry-run` + `write_stdin("\n  \n")` → same as AC1 (trim-empty). |
| **AC3** | Hermetic: live `ingest` (no `--dry-run`) + empty stdin → same usage class as AC1 (exit 2, example, no EOF JSON). |
| **AC4** | Existing `ingest__dry_run__accepts_placeholder_uuids` stays green (preview + exit 0). |
| **AC5** | Hermetic: `ingest --dry-run` + `write_stdin("{")` → exit **1**. Combined streams contain `COMMAND_FAILED` (or `Invalid JSON`) and do **not** contain `stdin is empty or not piped`. |
| **AC6** | Unit: `ingest_stdin_needs_usage(true, None)` / `(false, Some(""))` / `(false, Some(" \n"))` are `true`; `(false, Some("{"))` / `(false, Some("{…valid…}"))` are `false`. |
| **AC7** | Unit: `INGEST_EMPTY_STDIN_USAGE` contains the seven required keys + `ai-brains ingest --dry-run` + a single-quoted payload (`'{` … `}'`). |
| **AC8** | Hermetic: `ingest --help` contains the seven JSON keys (after_help). Assert **key presence**, not exact whitespace (clap may wrap). Re-read `cli_help_ia.rs` on go — today it only checks Harness group order (`ingest` position), not ingest body. Group-order tests still pass. |
| **AC9** | Existing T180 dry-run unknown-field reject + prod unknown-field accept stay green. |
| **AC10** | Existing `ingest__dry_run__errors_on_empty_content` stays green and is **not** exit 2 usage (still field error). |
| **AC11** | Existing `ingest__non_dry_run__still_validates_uuids` stays green. |
| **AC12** | Docs: CLI-EXIT-CODES fail_usage footnote + CAPABILITIES §4 + OPERATIONS one-liner + CHANGELOG BREAKING empty-stdin. |
| **AC13** | No new `ai-brains-contracts` type. No `IngestRequest` field add/remove. Zero new crates. |
| **AC14** | Manual dogfood (record exact commands): (1) `echo '' \| ai-brains ingest --dry-run` → exit 2 + example (not EOF JSON). (2) `echo '{' \| ai-brains ingest --dry-run` → exit 1 JSON envelope. (3) **TTY (AI1):** run `ai-brains ingest --dry-run` in an interactive console — must print usage **immediately** (no hang). Do not wait for Ctrl+C to “prove” it. |
| **AC15** | Targeted nextest: `ingest_reads_json_stdin` + `protocol_compat_cli` T180 ingest + `cli_help_ia` + ingest unit tests. Clippy `-p ai-brains-cli --all-targets -- -D warnings`. |
| **AC16** | No production file change until **go**. (Plan-time lock; flips to N/A after implement.) |

---

## 6. Non-goals

- Vault-independent `ingest --dry-run` / moving ingest to `run_sync_path_free`
- `ingest --schema` JSON Schema printer
- Rewriting `parse_ingest_request` or relaxing live UUID validation
- Changing T180 deny_unknown_fields asymmetry
- Populating `thinking` (T234)
- Harness hook / nightly / symbol ingest
- clap 5 / lockfile pin bumps / `std::io::IsTerminal` migrate
- Daemon ingest protocol
- T253–T255

---

## 7. Verification plan

| Phase | Proof |
|-------|-------|
| Red | AC1–AC3 fail today (exit 1 `COMMAND_FAILED` EOF). AC6 helper missing. AC8 no after_help example. |
| Green F1/F4/F6 | Gate + `fail_usage` + TTY-before-read |
| Green F2/F3/F13 | AC4 / AC5 / AC9–AC11 stay green |
| Docs | AC12 |
| Manual | AC14 twice on same repo state (human text stable), including TTY no-hang |
| Targeted | AC15 |
| Full gate | On finalize only (`dev-check.ps1` / workspace nextest + deny + audit + `ledgerful verify --scope full`) |

---

## 8. Files (expected)

| File | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/ingest.rs` | Gate helper + `fail_usage` + TTY; units |
| `crates/ai-brains-cli/src/main.rs` | `Ingest` `after_help` only |
| `crates/ai-brains-cli/tests/ingest_reads_json_stdin.rs` | AC1–AC5 / AC10 keep |
| `Docs/CLI-EXIT-CODES.md` | fail_usage footnote |
| `Docs/CAPABILITIES.md` | §4 ingest empty-stdin |
| `Docs/OPERATIONS.md` | one-liner after ingest sample |
| `CHANGELOG.md` | Unreleased BREAKING |
| `conductor/conductor.md` | Planning → Completed on close |
| `conductor/deferred.md` | T252 planned / closed |
| `conductor/tracks/README-T240-T255-CLI-EFFECTIVENESS.md` | status |

Do **not** edit `.ledgerful/` state by hand. Do **not** edit `ai-brains-contracts` / `ai-brains-capture` unless a compile forces a comment-only citation (it should not).

---

## 9. Contracts

- **DTO:** none. `IngestRequest` / `IngestResponse` frozen (T180).
- **CLI exit:** empty/TTY ingest is `fail_usage` **2**. Mid-payload stays **1** `COMMAND_FAILED`.
- **E1 empty-state:** empty stdin is **usage**, not an `IngestResponse { processed: false }` and not `null`.
- **Daemon / HTTP:** unchanged.

---

## 10. Risks

| Risk | Mitigation |
|------|------------|
| Scripts parse empty-stdin `COMMAND_FAILED` JSON | CHANGELOG BREAKING; only empty/TTY change; mid-payload JSON stays |
| TTY hermetic hang | Never read on TTY; prove TTY via unit `is_tty=true`, not a live TTY child |
| Mapping all serde errors to usage | F3 + AC5 `{` fixture |
| Help-ia snapshot surprise | after_help additive; group tests do not snapshot ingest body |
| Vault-missing masks usage | F9 honesty; hermetics keep `--vault-path` + zero key |

---

## 11. Success

Track is **Completed** when AC1–AC15 are green (AC16 N/A after go), review log has no open critical/high, mediums fixed or justified (cap ≤3), full gate green, manual AC14 recorded, contracts/docs synced, conductor + deferred updated, ledger FEATURE/UX TX committed.

---

## 12. AI fold-in (2026-08-15) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates planned work and verifies pins. AI2 quoting + after_help wrap are must-pins. AI1 remapped F6 flatten and `governed_common` const move declined.

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 §1–3 verify** | AI1 | **Agree** | Code / pins / docs claims already matched |
| **AI1 BS1** `read_json_from_stdin` swallow | AI1 | **Agree soft** | **F12** — T86 preflight `--stdin`, not ingest |
| **AI1 BS2** `events[0]` panic | AI1 | **Agree soft** | **F12** — pre-existing live-path only; not empty-stdin |
| **AI1 BS3** TTY unit-only | AI1 | **Agree** | Keep unit AC6; **add** live TTY to **AC14** |
| **AI1 BS4** `cli_help_ia` snapshot | AI1 | **Agree** | **AC8** — re-read on go; assert keys not body snapshot |
| **AI1 BS5** POSIX `echo` vs pwsh | AI1 | **Agree hard** | Absorbed by **F5** quoting (was F12) |
| **AI1 O1** flatten F6 | AI1 | **Decline** | Keep `is_tty` param — TTY unit without mocking stdin |
| **AI1 O2** shared stdin helper | AI1 | **Agree soft** | **F12** — isolation |
| **AI1 O3** const in `governed_common` | AI1 | **Decline** | T202 consts live in the **command** module; do not grow rank-9 hotspot |
| **AI1 O4** TTY dogfood | AI1 | **Agree hard** | **AC14(3)** + plan Phase 5 |
| **AI1 O5** BREAKING grep | AI1 | **Agree hard** | **F7** + plan Phase 3 |
| **AI2 BS1** unquoted `{…}` ScriptBlock | AI2 | **Agree hard** | **F5** single-quoted JSON |
| **AI2 BS1** “works in cmd.exe” | AI2 | **Decline** | cmd.exe would echo the quotes; Windows-first = PowerShell |
| **AI2 BS2** vault-key pre-check | AI2 | **Agree** | Already **F9**; hermetics keep `--vault-path` |
| **AI2 BS3** empty field vs empty stdin | AI2 | **Agree** | Already **F13** / **AC10** |
| **AI2 BS4** clap `after_help` wrap | AI2 | **Agree hard** | Multiline after_help; **AC8** key presence |
| **AI2 O1** reuse `fail_usage` | AI2 | **Agree** | Already **F1** |
| **AI2 O2** gate helper | AI2 | **Agree** | Already **F6** (keep `is_tty`) |
| **AI2 O3** zero-UUID payload | AI2 | **Agree** | Already **F5** |

### Pins locked by fold-in

1. Usage example JSON is **single-quoted** (`echo '{…}'`).
2. `after_help` is **multiline** JSON; tests assert keys, not wrapping.
3. AC14 includes a real-TTY no-hang check.
4. Phase 3 greps hook/`ingest.ps1`/`COMMAND_FAILED` consumers before CHANGELOG BREAKING.
5. F6 keeps `is_tty: bool`.
6. Const stays in `ingest.rs`.
