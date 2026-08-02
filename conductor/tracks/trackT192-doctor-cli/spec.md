# T192 — Doctor CLI (operator health surface)

- **Track ID:** T192-DoctorCli
- **Phase:** Post-P12 operator residual
- **Status:** 📋 **Pending / Expanded** (plan freeze + AI fold-in 2026-08-02; **planning only — not implementing**)
- **Depends on (hard):** T188 recovery export + restore hard-fail + `probe_restore_daemon_busy`; T187 SQLCipher page encrypt (live `cipher_version`); `VaultConnection::open_read_intent`; contracts `doctor` DTO stubs
- **Depends on (soft):** T181 RECOVERY-DRILLS; T186 hermetic CLI helpers; T189 DataKey rotation (kit re-export honesty only — no rotation work here); T190 reparse helpers for kit input
- **Blocks / feeds:** Closes **R-DOC-CLI** doctor residual + deferred **#2**; operator one-command health; honesty flip in RELEASE-CLAIMS / SECURITY-LIMITS / INSTALL
- **Category:** FEATURE / OPS
- **Deferred absorbed:** §59 / T181 **#2** doctor CLI; RELEASE-CLAIMS **R-DOC-CLI** residual (doctor half); SECURITY-LIMITS “doctor not shipped”; CAPABILITIES / RECOVERY-DRILLS / PROTOCOL-COMPAT / INSTALL absence language; `check-release-claims.ps1` “invented doctor CLI” forbid (**remove** rule #54 on ship)
- **Not absorbed:** Auto-remediation; hook doctor; ASVS/SOC2/cert language; remote monitoring SaaS; T193 path residuals; T194 kit Argon2 schema; T195 multi-user pipe/UDS; T196 systemd/launchd; inventing a well-known kit path on disk; full `PRAGMA integrity_check` as default; models/graph/embed probes (capture independence); TTY-smart format default (soft residual / opportunity only)
- **Research date:** 2026-08-02
- **AI fold-in:** AI1 §1–6 + AI2 M1–M5, L1–L4, L7. Disposition §15.
- **Ledger:** plan-only (no TX until implement)

## 1. Objective

Ship **`ai-brains doctor`** as a **real, read-only** product CLI that:

1. Reports vault / cipher / backup / recoverability / daemon health **without inventing capabilities**.
2. Never migrates, never creates a missing vault, never mutates vault state, never prints secrets.
3. Flips **R-DOC-CLI** from “export yes, doctor no” to **honest shipped language with a listed check matrix**.
4. Satisfies PRD intent (*validate recoverability*; *detect missing recovery kit*) via **explicit kit path + event audit**, not magic filesystem guesses.

## 2. Live baseline (re-scan 2026-08-02)

| Asset | Today |
|-------|--------|
| `ai-brains recovery export` | **Shipped (T188)** — kit file only; passphrase-file / rpassword; `schema_version=1` |
| `ai-brains doctor` | **Absent** (no `Commands::Doctor` in CLI) |
| `contracts::doctor::{DoctorReport, HealthCheck}` | Minimal stub: `status: String`, `checks: Vec<{name, ok, message?}>` — **DTO ≠ CLI** |
| `AppContext::from_cli` | Opens vault + **`migrate()`** — **forbidden for doctor** |
| `VaultConnection::open_read_intent` | No migrate, no create-missing, no journal_mode mutation — **doctor open path** |
| `probe_restore_daemon_busy` | T188 robust probe (3× ≥1000ms) — reuse for daemon **status** (info, not hard-fail) |
| `BackupService::list_backups` | Calls `backup_dir()` → **`create_dir_all`** when missing (brain/backup.rs) — **side effect confirmed (AI2)**; must fix via F17b |
| `backup::parse_duration` | Exists (`Nd`/`Nh`/`Nw`) for prune `--older-than` — **reuse** (no humantime) |
| `BackupInfo.timestamp` | `Option<NaiveDateTime>` — age via `and_utc()` like prune |
| `has_core_tables` | backup helper probes `events` + `memory_projection` — prefer for `schema_readable` |
| `cipher_version` pragma | Live SQLCipher proof (T187) |
| `RecoveryKitCreated` event | Audit that export happened; **not** proof offline kit still exists |
| `events.event_type` column | Stores **JSON-quoted** kind (`"RecoveryKitCreated"` with embedded quotes) — naive SQL fails (AI2 M3) |
| `EventStore` kind filter | **No** `count_events_of_kind` — raw SQL with quoting, or tiny local helper |
| `OutputFormat::parse` | Defaults **Json** when missing (governed) — doctor must **not** reuse that default |
| R-DOC-CLI | Partial non-claim (export present; doctor absent) |
| `check-release-claims.ps1` | Rule **#54** forbids affirmative ``ai-brains doctor`` — **remove on ship** (AI2 L2) |
| clap (workspace) | **4.5** pin → resolved **4.6.1** (docs.rs latest 4.6.5 as of 2026-07-31) — no bump required |
| rpassword | **7.5** / **7.5.4** workspace — reuse for optional kit unlock path |
| serde | workspace **1.0** (resolved ~1.0.228–1.0.229) — no bump |
| Exit codes (T160 governed) | 0/1/2/3–7 reserved for governed/policy — doctor must not steal 3–7 |
| humantime | **Absent** — do **not** add (F23 + M1) |

## 3. Research summary (2026-08-02)

| Source | Finding | T192 application |
|--------|---------|------------------|
| **PRD §12.3 / §25.4** | Doctor validates recoverability; detects missing recovery kit | Hard recoverability path = `--kit-path` unlock verify; soft path = `RecoveryKitCreated` event presence |
| **T188 freezes** | Export yes / doctor no; robust probe; no migrate under daemon; secrets discipline | Reuse probe + open_read_intent patterns; F13 of T188 becomes in-scope here |
| **T160 exit surface** | 0 success, 1 internal, 2 usage, 3 policy…7 hard-gate | Doctor uses **0 / 1 / 2 only** + status string; optional `--fail-on-degraded` → exit 1 |
| **clap 4.6.x** | Derive subcommands; usage → exit 2 convention | `Commands::Doctor { … }` clap 4 derive; no new dep |
| **K8s / Argo health model** | Healthy / Degraded / Missing; exit 0 only when healthy for *probes* | Doctor is **operator CLI**, not k8s probe: default exit 0 for `ok` **and** `degraded`; exit 1 for `fail`; JSON `status` is SOOT for automation |
| **Ledgerful doctor** | Multi-check report; warn vs ready; remediation lines | Human output mirrors check + remediation pattern; no gemini/GPU probes in AI-Brains doctor |
| **OWASP secrets / CLI** | No argv secrets; zeroize; no key/kit on stdout | Kit unlock uses `--passphrase-file` or TTY zero-echo; leakage tests |
| **RELEASE-CLAIMS / SECURITY-LIMITS** | Doctor listed as non-claim / absent | Flip to shipped with check matrix; residual language for optional kit path |
| **Capture independence** | Capture must work without models/graph | Doctor **must not** require models, embeddings, graph, or Ollama |
| **serde 1.0.x** | Stable contracts serialization (~1.0.229 max) | Expand DTO with skip_serializing_if; version field; no bump |
| **AI2 re-scan** | list_backups create_dir; event_type JSON quotes; parse_duration exists | F17b, F16, F17 — hard freezes |
| **T190 reparse helpers** | `is_reparse_or_symlink` / `refuse_if_reparse` | Kit input path refuse (F15b) |

## 4. Threat / honesty model

| Threat / honesty risk | Mitigation |
|----------------------|------------|
| Doctor claims “recoverable” without offline kit | `recovery_kit_file` only **ok** when `--kit-path` unlock succeeds; event-only is **warn/degraded**, not full recoverability |
| Invented kit location next to vault | **Forbidden** — no default kit path search |
| Migrate/create while daemon holds vault | **No** `AppContext::from_cli`; **only** `open_read_intent` |
| Secrets on stdout/tracing | No DataKey / kit JSON / passphrase / key hex; leakage tests |
| “Healthy” while wrong key | `vault_open` **fail** → overall `fail` |
| Auto-fix marketing | Non-goal; remediation text is **advisory only** |
| Cert / ASVS language | Forbidden (release claims gate) |
| Treating daemon-down as fail | Daemon up/down is **info** for single-owner desktop (operator may stop daemon for restore) |
| Doctor creates `backups/` on every run | **F17b** read-only backup dir — no `create_dir_all` on list |
| False “no kit event” from SQL quoting | **F16** query uses JSON-quoted `event_type` value |
| Symlinked `--kit-path` | **F15b** reparse refuse before read |

## 5. Frozen decisions (F1–F34)

| ID | Decision |
|----|----------|
| **F1 — CLI name** | Top-level `ai-brains doctor` (not under `recovery` or `vault`). |
| **F2 — Read-only** | Doctor **never** migrates, never creates vault, never appends events, never writes backups/kits, **never creates directories** (including `backups/`). Pure report. |
| **F3 — Open path** | Resolve vault path from `--vault-path` / `AI_BRAINS_VAULT_PATH` (same as other commands). Open with **`VaultConnection::open_read_intent` only** (`SQLITE_OPEN_READ_ONLY` + key pragmas + sqlite_master verify). Missing vault → check `vault_exists` **fail**, overall `fail`. |
| **F4 — Key resolution** | Same key resolution as recovery export (`AI_BRAINS_KEY` / `--key`; zero-key only with existing product escape hatch). Wrong key → `vault_open` **fail**. Never print key. |
| **F5 — No AppContext** | Do **not** call `AppContext::from_cli` (always migrates). Doctor is an **early async handler** in `main` (like `Recovery` / `Vault::RotateDatakey`) — **not** `is_vault_path_free` / `run_sync_path_free`. Needs vault path; wires **before** AppContext construction. |
| **F6 — Daemon interaction** | Report daemon reachability via **`probe_restore_daemon_busy`** (or shared rename later — freeze: reuse T188 robust probe semantics). **Daemon up is not fail.** **Daemon down is not fail.** Severity `ok` with message `up` or `down` when probe completes. Probe errors → `warn`. Probe residual honesty (detects our IPC only) documented in help/docs. |
| **F7 — Check inventory v1 (ordered)** | See §6. Hard checks contribute to `fail`; soft to `degraded`; info never fails alone. |
| **F8 — Overall status roll-up** | `fail` if any check severity=`fail`. Else `degraded` if any severity=`warn`. Else `ok`. Skipped checks do not degrade. |
| **F9 — Exit codes** | **0** = overall `ok` **or** `degraded` (default). **1** = overall `fail` or internal tool error. **2** = clap usage. **`--fail-on-degraded`**: if overall `degraded`, exit **1**. Do **not** introduce exit 3–7 for doctor (T160 collision). |
| **F10 — Output formats** | `--format human\|json` with clap **`default_value = "human"`** (operator surface — **do not** use `OutputFormat::parse`’s missing→Json default). **`--json` bool flag** forces JSON when set (overrides `--format`). JSON stdout = single `DoctorReport` object (no secrets). Human: summary line + per-check lines + remediation when present. **v1 does not** TTY-smart-default (preflight pattern is soft residual only). |
| **F11 — Contracts expand** | Expand `ai-brains-contracts::doctor` (breaking-compatible within workspace — only DTO, no prior CLI consumer): |
| | `DoctorReport { schema_version: u32 /*1*/, status: DoctorStatus, checks: Vec<HealthCheck>, vault_path: String, generated_at: String /*RFC3339*/ }` |
| | `DoctorStatus` enum: `Ok`, `Degraded`, `Fail` (serde rename_all = "snake_case" → `ok`/`degraded`/`fail`) |
| | `HealthCheck { name: String, severity: CheckSeverity, ok: bool, message: Option<String>, remediation: Option<String> }` |
| | `CheckSeverity`: `Ok`, `Warn`, `Fail`, `Skip` (snake_case). `ok` is true iff severity is `Ok` or `Skip`. |
| | Drop bare `status: String` free-form. Golden/null shape: empty checks only if tool error before inventory; message/remediation null when absent. |
| **F12 — v1 hard checks** | `vault_exists`, `vault_open`, `schema_readable`, `cipher_page` (see §6). |
| **F13 — v1 soft checks** | `backup_recent`, `recovery_kit_event`, `zero_key_escape` (warn only). |
| **F14 — v1 info checks** | `daemon_reachable` (`ok` severity always if probe completed; message `up`/`down`). Probe errors → `warn` with residual message. |
| **F15 — Kit file check (recoverability)** | Optional `--kit-path <path>` + passphrase source (`--passphrase-file` or TTY zero-echo). When provided: parse kit, unlock with passphrase (and DPAPI if present/available), compare unwrapped DataKey to vault key via `SqlCipherKey::from_data_key` / existing helpers — **ok** if unlock + key match; **fail** if path set but unlock/mismatch. When **not** provided: check `recovery_kit_file` = **Skip** with message `pass --kit-path to verify offline kit`. **Never invent kit path.** |
| **F15b — Kit input reparse refuse** | Before `fs::read` of `--kit-path`: refuse reparse/symlink/junction via `ai_brains_path::is_reparse_or_symlink` + `refuse_if_reparse` (same pattern as T188 passphrase-file / kit output). Fail check with actionable message. Passphrase-file already covered by existing helper. |
| **F16 — Kit event check** | Query event log (read-only) for ≥1 `RecoveryKitCreated`. None → **warn** + remediation `ai-brains recovery export --output <offline-path>`. Present → **ok** (does **not** prove offline copy still exists — message honesty). Event append is best-effort on export (T188) — absent event ≠ absent kit. **Query gotcha (mandatory):** `events.event_type` stores **JSON-serialized** kind with embedded quotes (e.g. column value is `"RecoveryKitCreated"` including quote chars). Naive `WHERE event_type = 'RecoveryKitCreated'` always returns 0. Use `WHERE event_type = '"RecoveryKitCreated"'` **or** a local helper that quotes via `serde_json::to_string(&EventKind::…)`. Prefer local SQL helper over expanding `EventStore` trait (no new migration). |
| **F17 — Backup age** | Threshold default **7d**. Flag `--backup-max-age` reuses existing **`backup::parse_duration`** (`Nd` / `Nh` / `Nw` — same as prune `--older-than`). Make `parse_duration` `pub(crate)` or shared if needed. **No humantime dep.** Age compare: `BackupInfo.timestamp` is `Option<NaiveDateTime>` — use **`and_utc()`** pattern like prune. No backups / dir absent / newest older than threshold / list error → **warn**. Newest within threshold → **ok**. Soft only (never hard-fail overall alone). |
| **F17b — Backup dir non-mutating (scoped refactor)** | `backup_dir()` currently `create_dir_all` for both read and write. **In scope for T192:** add **`backup_dir_read_only()`** (or equivalent) that returns `Ok(None)` / empty when dir **absent** — **never** creates. Migrate **read** callers: `list_backups`, `find_backup_files`, `preview_backup_path`. Write callers (`run_backup_from_conn`, `prune_backups`) keep creating `backup_dir()`. Doctor calls fixed `list_backups` only. Test: doctor / list on vault without backups dir leaves tree without new `backups/`. |
| **F18 — cipher_page** | After open_read_intent: `PRAGMA cipher_version` non-empty (T187). Empty/missing → **fail** (product claims page encrypt). |
| **F19 — schema_readable** | Prefer product **core-table probe** (`has_core_tables` or equivalent: `events` + `memory_projection` present) over bare `SELECT count(*) FROM sqlite_master` (empty/broken vault can still pass count). Missing core tables → **fail**. If open failed → **skip**. |
| **F20 — zero_key_escape** | If process key is all-zero **or** `AI_BRAINS_ALLOW_ZERO_KEY` is set → **warn** with honesty residual (**R-ZERO-KEY**). Normal keys → **ok** or **Skip** if not applicable. |
| **F21 — `--full`** | Optional slow path: `PRAGMA integrity_check` (or quick_check) once. Default **off**. Fail on integrity failure; skip when not requested. v1 may implement as soft if too platform-flaky — prefer hard fail when run. |
| **F22 — Secrets / leakage** | Stdout/stderr/tracing must not contain: passphrase, DataKey bytes/hex, kit JSON, wrap ciphertext. Salt/nonce in operator-facing messages **forbidden** too for doctor (unlike kit file internals). Tests: assert absence of known key material. |
| **F23 — Deps** | **Zero new production deps** (including **no humantime**). Reuse clap, serde, rpassword (kit path only), zeroize, existing crypto/store/brain backup helpers. |
| **F24 — Capture independence** | Doctor does not load models, graph feature, embeddings, or Ollama. |
| **F25 — Hermetic tests** | Use `common::hermetic_*` + tempdir vaults. No ambient project context elevation. |
| **F26 — Docs / claims flip** | On ship: INSTALL, CAPABILITIES, OPERATIONS, RECOVERY-DRILLS, SECURITY-LIMITS, SECURITY.md, RELEASE-CLAIMS (**R-DOC-CLI** → doctor shipped with listed checks), RELEASE-CHECKLIST, PROTOCOL-COMPAT, Docs/README, CHANGELOG. **`check-release-claims.ps1`: remove rule #54** (“invented doctor CLI as shipped”) entirely — doctor is product; do not fragile-retarget lookahead. |
| **F27 — Deferred strike** | Strike deferred **#2** / R-DOC-CLI doctor residual on completion. Keep export residual language only if any remain (none expected). |
| **F28 — Domain boundary** | CLI adapter only: check runners may live in `commands/doctor.rs`; pure helpers for roll-up/status in same module or thin lib if tested; **no** domain logic that belongs in store/crypto duplicated — call existing APIs. Backup read-only dir helper lives in `ai-brains-brain` (F17b). |
| **F29 — Help / after_help** | Document: read-only; does not replace RECOVERY-DRILLS; kit offline residual; daemon probe residual; `--fail-on-degraded` for CI scripts; `--backup-max-age` uses same `Nd`/`Nh`/`Nw` as prune. |
| **F30 — Determinism** | Sort checks by fixed name order (§6). `generated_at` is wall-clock (allowed volatile); tests pin status/checks, not timestamp equality (or inject clock in unit tests). |
| **F31 — Error before report** | Fatal argparse / missing vault-path config → exit 1/2 without partial JSON unless `--format json` and a minimal fail report is feasible. Prefer always emitting a `DoctorReport` when vault-path resolved. |
| **F32 — Non-claims after ship** | Doctor does **not** claim: NIST Purge, multi-user safety, third-party lock detection, perfect recoverability without offline kit, SOC2. |
| **F33 — F2 filesystem honesty** | Doctor process must leave no new files/dirs under the vault parent (assert in hermetic test when no backups dir existed). |
| **F34 — Passphrase argv** | **Forbid** `--passphrase` argv (T188 parity). |

## 6. Check matrix (v1)

| `name` | Severity class | When ok | When warn | When fail | When skip |
|--------|----------------|---------|-----------|-----------|-----------|
| `vault_exists` | hard | Path is existing regular file (reparse residual: refuse symlink vault as fail or warn — prefer **fail** if reparse to unexpected; align path helpers) | — | Missing / not a file | — |
| `vault_open` | hard | `open_read_intent` succeeds | — | Wrong key / locked / unreadable | — |
| `schema_readable` | hard | Core tables present (`has_core_tables`) | — | Missing core tables / probe fails | If open failed (skip) |
| `cipher_page` | hard | Non-empty `cipher_version` | — | Empty / error | If open failed |
| `daemon_reachable` | info | Probe completed (`up` or `down`) | Probe errored unexpectedly | — (never fail) | — |
| `backup_recent` | soft | Newest backup ≤ max-age (default 7d); dir may be absent → warn | None / dir absent / stale / list error | — | — |
| `recovery_kit_event` | soft | ≥1 row with JSON-quoted kind | None (or query error → warn) | — | — |
| `recovery_kit_file` | hard **if path set** | Unlock + key match; reparse refused | — | Path set but reparse/unlock/mismatch | Path not set |
| `zero_key_escape` | soft | Non-zero key, no escape env | Zero key or escape env set | — | — |
| `integrity` (optional) | hard if `--full` | integrity_check ok | — | Failed | Not `--full` |

## 7. CLI surface

```text
ai-brains doctor
  [--format human|json]           # default human (not governed OutputFormat::parse default)
  [--json]                        # bool force JSON
  [--fail-on-degraded]
  [--kit-path <path>] [--passphrase-file <path>]
  [--backup-max-age <Nd|Nh|Nw>]   # default 7d; same parse_duration as prune --older-than
  [--full]                        # integrity_check
  # global: --vault-path, --key / AI_BRAINS_*
```

**Passphrase argv forbidden** (F34 / T188). Kit path without passphrase source: attempt DPAPI-only unlock if kit has dpapi; else fail with actionable message. Kit path refuses reparse (F15b).

## 8. Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | `ai-brains doctor` exists in `--help` and runs hermetically in tests |
| **AC2** | Happy path: temp vault + key → overall `ok` or `degraded` (no kit path → kit_file skip; may warn on kit event/backup) with exit 0 |
| **AC3** | Wrong key → `vault_open` fail → overall `fail` → exit 1 |
| **AC4** | Missing vault → fail exit 1; no create |
| **AC5** | Daemon up and down both produce `daemon_reachable` without forcing fail |
| **AC6** | `--kit-path` good kit+pass → `recovery_kit_file` ok; bad passphrase → fail; reparse kit path refused |
| **AC7** | No secrets in stdout/stderr (key material / kit JSON / passphrase) |
| **AC8** | No migrate: spy/assert open path or unit-level no `migrate` call; daemon-up doctor still read-only |
| **AC9** | JSON report deserializes to contracts `DoctorReport` schema_version=1; default human without `--json` |
| **AC10** | `--fail-on-degraded` turns overall degraded into exit 1 |
| **AC11** | Docs + R-DOC-CLI flipped; **claims script rule #54 removed** |
| **AC12** | Zero new prod deps (no humantime); deny/audit green |
| **AC13** | Full gate + review; deferred #2 struck |
| **AC14** | Capture independence: no model/graph required |
| **AC15** | Doctor (and fixed `list_backups`) on vault **without** `backups/` does **not** create that directory |
| **AC16** | Vault with `RecoveryKitCreated` event → `recovery_kit_event` **ok** (proves JSON-quote query works; not false warn) |

## 9. Non-goals

- Auto-fix / auto `recovery export` / auto backup
- `hook doctor` / harness adapter doctor
- Remote telemetry, multi-host fleet health
- Inventing default kit paths
- Models / embeddings / graph health
- Path TOCTOU elevation (T193)
- Kit schema Argon2 fields (T194)
- Multi-user pipe hardening (T195)
- systemd/launchd (T196)
- Security certification language
- Replacing RECOVERY-DRILLS operator playbooks

## 10. Verification

```powershell
cargo nextest run -p ai-brains-cli --test doctor_cli
cargo nextest run -p ai-brains-contracts
cargo clippy -p ai-brains-cli -p ai-brains-contracts --all-targets -- -D warnings
# Full gate at closeout
.\scripts\check-release-claims.ps1
```

### Manual evidence

1. Real vault: `ai-brains doctor` human + `--json`.  
2. Daemon up/down both ok.  
3. `--kit-path` unlock success and failure.  
4. Wrong key fails closed.  
5. Confirm no secrets in output.  
6. Docs claim honesty.

## 11. Handoffs

| To | What |
|----|------|
| deferred **#2** / R-DOC-CLI | Strike / rewrite on ship |
| T188 | Reuse probe + kit unlock primitives |
| T189 | Rotation still requires kit re-export; doctor can verify new kit path |
| T193–T196 | Unrelated residuals |
| check-release-claims | Allow real doctor product language |

## 12. Definition of Done

AC1–AC16; plan checkboxes done; conductor ✅; deferred #2 struck; no open critical/high; contracts synced; claims script green (rule #54 gone).

## 13. Implementation notes (non-normative)

- Prefer extracting shared “status roll-up” pure functions for unit tests without process spawn.
- **F17b is mandatory** — do not doctor-local skip only; fix brain read paths so `backup list` is also non-mutating.
- Event query: raw SQL with **JSON-quoted** kind (F16); no new migration; no required `EventStore` trait expansion.
- Windows service message: optional mention in daemon check remediation only if down and operator wants start — do not require service.
- Desktop `probe_health` is unrelated Tauri permission — do not conflate.
- Format: clap `default_value = "human"` + `--json` bool; may still match on `OutputFormat` enum after override.

## 14. Residual after ship (honest)

| Residual | Note |
|----------|------|
| Offline kit without `--kit-path` | Still operator responsibility; event ≠ file existence |
| Daemon probe = our IPC only | Same residual as T188 F4 |
| integrity_check cost | Opt-in `--full` only |
| Hook doctor | Still absent (never in scope) |
| TTY-smart format default | Soft opportunity (preflight pattern); v1 fixed human default |

## 15. AI fold-in disposition (2026-08-02)

### 15.1 Agreed → folded

| Source | Item | Fold |
|--------|------|------|
| AI1 §1 / AI2 M2 | Non-mutating backup list / create_dir side effect | **F2**, **F17b**, **F33**, AC15; plan C4 scoped refactor |
| AI1 §2 | open_read_intent only | **F3** reaffirmed (already frozen) |
| AI1 §3 | Severity roll-up | **F7–F8** reaffirmed |
| AI1 §4 / F9 | Exit 0/1/2 + `--fail-on-degraded` | **F9** reaffirmed |
| AI1 §5 | Kit path honesty / skip | **F15** reaffirmed |
| AI1 §6 / AI2 L2 | Claims script | **F26** → **remove** rule #54 (not retarget) |
| AI2 M1 | Reuse `parse_duration`; no humantime | **F17**, **F23** |
| AI2 M3 | event_type JSON quoting | **F16**, AC16 |
| AI2 M4 | Format default human vs OutputFormat::parse | **F10** |
| AI2 M5 | Kit input reparse refuse | **F15b**, AC6 |
| AI2 L1 | NaiveDateTime `and_utc` | **F17** |
| AI2 L3 | Prefer `has_core_tables` | **F19** |
| AI2 L4 | serde patch note | Baseline cosmetic update |
| AI2 L7 | “path-free” phrasing | **F5** → early async handler |

### 15.2 Agreed as confirm-only (no freeze change)

| Source | Item | Note |
|--------|------|------|
| AI2 L6 | Event best-effort vs kit file | F16 honesty already correct |

### 15.3 Declined / deferred

| Source | Item | Why |
|--------|------|-----|
| AI2 L5 | TTY-smart format default (preflight pattern) | Soft residual only; F10 keeps fixed human default for predictable scripts without detecting TTY |
| Expanding `EventStore` trait with `count_events_of_kind` | Optional polish | Prefer local SQL + quote helper in doctor/brain; trait expansion not required |

### 15.4 AI1 reaffirmations already in freeze

AI1 summary table items 2–5 were already F3/F5, F11, F9, F26 — strengthened only where gaps existed (backup mutation, claims remove).
