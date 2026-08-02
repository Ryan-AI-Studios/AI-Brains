# T192 Plan — Doctor CLI

Status: **✅ Implemented** (2026-08-02). Spec: [spec.md](./spec.md) (F1–F34, AC1–AC16).

**Implement complete — awaiting PR CI / squash-merge closeout.**

## Preconditions

- [x] Expand freezes (checks matrix, exit codes, contracts, daemon rules)
- [x] Research live DTO stubs + SECURITY-LIMITS / R-DOC-CLI wording
- [x] Research clap 4.6.x, T160 exit surface, T188 probe/open_read_intent, BackupService list
- [x] Online: health-check CLI / degraded status conventions (operator CLI ≠ k8s probe)
- [x] AI fold-in (AI1 §1–6 + AI2 M1–M5, L1–L4, L7) → spec §15
- [x] `ai-brains preflight --summary` + recall + pin decision
- [x] `ledgerful doctor` + `scan --impact` (plan-only; dirty conductor docs expected)
- [x] `ledgerful ledger start T192-DoctorCli --category FEATURE` (**at implement**, not plan)

## Deferred roll-up (absorb on implement closeout)

| Item | Action |
|------|--------|
| deferred **#2** doctor CLI / R-DOC-CLI | **Absorb** → strike when AC11–AC13 green |
| R-DOC-CLI in RELEASE-CLAIMS | Rewrite: doctor shipped with §6 matrix; residual = kit offline without `--kit-path` |
| SECURITY-LIMITS §6 / forbidden doctor language | Flip shipped |
| CAPABILITIES / INSTALL / OPERATIONS / RECOVERY-DRILLS / PROTOCOL-COMPAT / Docs/README | Flip “absent” → shipped + flags |
| `check-release-claims.ps1` invented-doctor forbid | **Remove rule #54** entirely (F26 / AI2 L2) |
| T188 residual “doctor remains” | Close via this track |
| T181 residual #2 | Strike |
| Pre-existing `list_backups` create_dir side effect | **Absorb** via F17b (read-path fix in brain) |

**Not absorbed:** T193 path, T194 Argon2 kit schema, T195 multi-user, T196 units, hook doctor, auto-fix, TTY-smart format (soft residual).

---

## Phase A — Design freeze (this plan)

- [x] **A1** Check inventory + severity + roll-up (F7–F8, §6)
- [x] **A2** Exit-code policy (F9) — 0 ok|degraded, 1 fail, 2 usage; `--fail-on-degraded`
- [x] **A3** Daemon = info only (F6)
- [x] **A4** Kit recoverability = optional `--kit-path` + event soft check (F15–F16) + reparse (F15b)
- [x] **A5** Contracts expansion shape (F11)
- [x] **A6** Read-only open_read_intent / no AppContext (F2–F5)
- [x] **A7** Zero new deps; no humantime (F23 / M1)
- [x] **A8** F17b backup_dir_read_only scope; F16 JSON-quote; F10 human default; F19 core tables

---

## Phase B — Contracts (TDD)

- [x] **B1** Expand `crates/ai-brains-contracts/src/doctor.rs`:
  - `DoctorStatus`, `CheckSeverity`, `DoctorReport` schema_version=1
  - serde snake_case; skip empty optionals
- [x] **B2** Unit/serde roundtrip tests in contracts (or cli golden)
- [x] **B3** Document empty/null: missing message/remediation omitted; checks ordered

**Tests:**  
`doctor_report__serde__roundtrip_schema_v1`  
`doctor_status__roll_up__fail_beats_degraded` (if pure fn in contracts or cli)

---

## Phase C0 — Brain backup non-mutating read paths (F17b, before doctor soft check)

- [x] **C0.1** Add `backup_dir_read_only()` (returns absent without `create_dir_all`)
- [x] **C0.2** Migrate **read** callers: `list_backups`, `find_backup_files`, `preview_backup_path`
- [x] **C0.3** Write callers (`run_backup_from_conn`, `prune_backups`) keep `backup_dir()` create
- [x] **C0.4** Make `parse_duration` reachable for doctor (`pub(crate)` or re-export) — `Nd`/`Nh`/`Nw` only
- [x] **C0.5** Tests: list/preview with missing backups dir → no dir created; prune/create still create

**Tests:**  
`list_backups__missing_dir__no_create`  
`preview_backup_path__missing_dir__no_create` (or equivalent)  
`run_backup__creates_dir` regression

---

## Phase C — Core doctor runner (TDD)

- [x] **C1** `commands/doctor.rs` module + pure roll-up helpers
- [x] **C2** Checks: `vault_exists`, `vault_open` (open_read_intent), `schema_readable` (**has_core_tables**), `cipher_page`
- [x] **C3** Checks: `daemon_reachable` via `probe_restore_daemon_busy`
- [x] **C4** Checks: `backup_recent` — default 7d via **`parse_duration`**; age via `NaiveDateTime::and_utc()`; uses F17b list
- [x] **C5** Checks: `recovery_kit_event` — SQL with **JSON-quoted** `event_type` (`'"RecoveryKitCreated"'` or serde_json quote helper)
- [x] **C6** Checks: `recovery_kit_file` optional path + **reparse refuse** + passphrase-file / TTY / DPAPI-only
- [x] **C7** Checks: `zero_key_escape`
- [x] **C8** Optional `--full` integrity
- [x] **C9** Emitters: clap `--format` **default human**; `--json` bool override; exit map + `--fail-on-degraded` (do **not** use `OutputFormat::parse` missing→Json default)
- [x] **C10** Wire `Commands::Doctor` as **early async** in `main.rs` **before** AppContext (like Recovery — not path-free)

**Tests (hermetic):**  
`doctor__happy_temp_vault__ok_or_degraded_exit_0`  
`doctor__wrong_key__fail_exit_1`  
`doctor__missing_vault__fail_no_create`  
`doctor__daemon_up_or_down__not_fail` (inject or process)  
`doctor__kit_path_good__recovery_kit_file_ok`  
`doctor__kit_path_bad_pass__fail`  
`doctor__kit_path_reparse__refused`  
`doctor__no_kit_path__recovery_kit_file_skip`  
`doctor__json__schema_v1_deserializes`  
`doctor__default_format_human__without_json_flag`  
`doctor__fail_on_degraded__exit_1`  
`doctor__stdout__no_secrets`  
`doctor__no_migrate_while_daemon_up` (unit injectable)  
`doctor__no_backups_dir__does_not_create` (AC15)  
`doctor__recovery_kit_created_event__ok_not_false_warn` (AC16)

---

## Phase D — Docs / claims

- [x] **D1** CAPABILITIES: doctor shipped + flags + check list
- [x] **D2** INSTALL / OPERATIONS: command table
- [x] **D3** RECOVERY-DRILLS §4: doctor present; still offline kit residual
- [x] **D4** SECURITY-LIMITS §6 + forbidden marketing line
- [x] **D5** RELEASE-CLAIMS R-DOC-CLI rewrite
- [x] **D6** PROTOCOL-COMPAT / Docs/README / SECURITY.md
- [x] **D7** `scripts/check-release-claims.ps1` — **remove rule #54** (invented doctor)
- [x] **D8** CHANGELOG
- [x] **D9** RELEASE-CHECKLIST if it cites doctor absence

---

## Phase E — Closeout

- [x] **E1** Full gate: fmt / clippy / nextest / deny / audit
- [x] **E2** `ledgerful verify --scope full` (or fast + documented)
- [x] **E3** Review log `review.md` + cross-model if FEATURE high-risk (ops CLI — use judgment; at least primary review)
- [x] **E4** Manual evidence recorded
- [x] **E5** Strike deferred #2 in `deferred.md`; update `conductor.md` → Completed
- [x] **E6** Pin ship decision; ledger commit

---

## License / deps gate

- [x] Zero new prod deps (**no humantime**)
- [x] rpassword only for optional kit unlock (already present)
- [x] deny + audit green

---

## Suggested implement order

1. Contracts (B)  
2. **Brain F17b + parse_duration visibility (C0)**  
3. Pure roll-up + hard checks (C1–C2)  
4. Soft/info checks (C3–C7) with F16 quote + F15b reparse  
5. CLI wire + hermetic tests (incl. AC15/AC16)  
6. Docs/claims + **remove rule #54** (D)  
7. Gate + strike deferred (E)

---

## Out of scope checklist

- [ ] Auto-remediation  
- [ ] Hook doctor  
- [ ] Default kit path search  
- [ ] Models/graph/embed health  
- [ ] T193–T196 product work  
- [ ] Security certification language  
- [ ] TTY-smart format default (soft residual)  
- [ ] EventStore trait expansion (local SQL quote helper preferred)  

