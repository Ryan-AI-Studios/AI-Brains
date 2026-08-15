# T251 completeness check (product + docs)

**Track:** T251-DeviceDiscoverability  
**Authority:** `spec.md` F1–F16 / AC1–AC16 + `plan.md` Phase 1–3  
**Workspace:** `C:\dev\AI-Brains`  
**Date:** 2026-08-14  
**Verdict:** **COMPLETE**

No missing product, docs, or hermetic items versus F1–F16 / AC1–AC16 / Phase 1–3. Internal r1 / r1b CLEAN stands.

Out of this check (orchestrator owns; **not** product defects): full workspace CI gate, live vault dogfood, `conductor.md` Completed, `deferred.md` strike, coordinated update, PR, ledger commit, canonical `review.md`.

---

## Verdict

Phase 1–3 product+docs are fully implemented. First-class `device status` prints the shared enrolled roster and **always** appends `next: ai-brains replicate status`. List / fingerprint / replicate / T198 plural SOOT / singular error copies / contracts / clap pins stay frozen. Named hermetics cover AC1–AC8 / AC10–AC11 / AC12-by-design. F7/AC9 docs set is complete (CHANGELOG is repo-root `CHANGELOG.md`, not `Docs/CHANGELOG.md`).

AC16 is **N/A** after go. AC13–AC15 remain operator/gate work, not missing implementation.

---

## F / AC / Phase matrix

| ID | Status | Evidence |
|----|--------|----------|
| **F1** First-class `Status` after `List`, not `visible_alias` | **Met** | `crates/ai-brains-cli/src/main.rs` `DeviceCommands`: `List` then unit `Status` (no flags). Dispatch `DeviceCommands::Status => run_status`. Unit `device_status__parses`. `visible_alias` remains T243 `search` / unrelated args only. |
| **F2** Roster + always `next:`; no inline replicate; revoked-only ≡ empty | **Met** | `run_status` = `emit_device_roster` then `println!("{DEVICE_STATUS_NEXT}")` with `next: ai-brains replicate status`. Uses `list_enrolled_devices` (`status IN ('active','local')`). No `--format` / `--quiet` / `--fake-relay`. No revoke hermetic. |
| **F3** No multi-device fill / no top-level status / no default `device` / no 16th doctor | **Met** | Bootstrap/enroll/revoke/package role unchanged. No `Commands::Status`. No default subcommand. Doctor still `assert_eq!(report.checks.len(), 15)`. |
| **F4** Shared emitter; one plural const; fingerprint const-only; singular errors stay | **Met** | `EMPTY_ENROLL_HINT` is the only production plural T198 sentence. Fingerprint prints the const, not the emitter. Singular `No enrolled device on this vault…` remains at `device.rs` L139 and `replicate.rs` L206. |
| **F5** List / fingerprint frozen (no `next:`) | **Met** | `run_list` is emitter only. Hermetics empty+enrolled list + fingerprint companion reject `next:`. Existing `device_fingerprint__no_enroll__bootstrap_message_exit_0` still asserts the full T198 line. |
| **F6 / AC11** Human-only; no DTO; no Status flags | **Met** | Unit variant. No `DeviceStatus*` in `crates/ai-brains-contracts`. AC8 hermetic: `--format json` → clap **2**. |
| **F7 / AC9** Docs | **Met** | CAPABILITIES OutputFormat rows + operator note; PROTOCOL-COMPAT §5 additive human-only rows (explicitly not compact↔pretty); OPERATIONS one-liner; INSTALL §7 tip; root CHANGELOG Unreleased **always** appends `next:` (empty and enrolled); CLI-EXIT-CODES table row + Device status footnote (0 empty/enrolled; extra args clap 2). Device `after_help` includes `ai-brains device status`. |
| **F8** Capture-independent; zero new crates | **Met** | String emit + existing `list_enrolled_devices`. `ai-brains-cli` Cargo.toml: no new deps. |
| **F9** Exit 0 empty+enrolled; extra args clap 2 | **Met** | AC1/AC3 hermetics exit 0; AC8 exit 2. No VAULT_KEY rewrite. |
| **F10** No pin bumps | **Met** | Workspace `clap = { version = "4.5", ... }`. |
| **F11** Isolation | **Met** (code) | No T243–T250 product rewrite. `replicate.rs` not given T251 logic. No `OutputFormat::parse` change. No live bootstrap/daemon/`AI_BRAINS_KEY` print in this change set. |
| **F12** Soft residuals stay deferred | **Met** | No list JSON, no doctor enrollment check, no `stat` alias, no default `device`→status, no singular-copy unify. |
| **F13** T198 exact sentence | **Met** | Const + hermetics use `No enrolled devices. Run \`ai-brains device bootstrap\` first.` |
| **F14** Honesty owner stays after_help / replicate | **Met** | Status emits roster + `next:` only. Device after_help honesty line retained. |
| **F15** High-finding classes absent | **Met** | Not a List alias; does not inline `replicate::run_status`; no DTO; T198 unchanged; no 16th doctor check; no `sync`/`safety sync` extension; no clap 5. |
| **F16 / AC16** Plan-only until go | **N/A** | Implementation exists after go. |
| **AC1** Recognized; empty exit 0 | **Met** | `device_status__empty_vault__outputs_hint_and_next_replicate_status`. |
| **AC2** Empty = exact T198 + last data line `next:` | **Met** | Same test: `contains(T198_EMPTY)` + `last_nonempty_line == DEVICE_STATUS_NEXT`. |
| **AC3** Enrolled table + `next:`; exit 0 | **Met** | `device_status__enrolled_vault__outputs_table_and_next_replicate_status`. |
| **AC4** List no `next:` (empty + enrolled) | **Met** | Split named tests (stronger than plan’s single `device_list__regression__does_not_contain_next`; no `for` in one `#[test]`). |
| **AC5** Fingerprint T198, no `next:` | **Met** | Existing empty-states test + companion `device_fingerprint__empty_vault__does_not_contain_next`. |
| **AC6** Help lists `status` + example | **Met** | `device_status__help__lists_status`; parent Device `after_help` includes `ai-brains device status`. |
| **AC7** `replicate status` empty unchanged | **Met** | `replicate_status__empty_vault__still_prints_enrolled_count_honesty_hint`. |
| **AC8** `--format json` clap 2 | **Met** | `device_status__with_format_json_flag__fails_exit_2`. |
| **AC10** Shared const + grep plural; singular remain | **Met** | See F4. |
| **AC12** `cli_help_ia` still valid | **Met** (design) | Suite does not snapshot Device examples. Device `after_help` is additive only. Not re-executed here. |
| **AC13** Full CI gate | **Orchestrator** | Code is capture-independent / no new crates. Workspace gate not this check. |
| **AC14** Live empty vault dogfood | **Orchestrator** | Spec: do not bootstrap live vault. Hermetic empty path matches specified live outcome. |
| **AC15** Live list / fingerprint / replicate unchanged | **Orchestrator** | Isolation + AC4/AC5/AC7 hermetics lock frozen surfaces. |

---

## Plan Phase 1–3

| Phase | Status | Notes |
|-------|--------|-------|
| **Phase 1** Shared roster + Status + named hermetics | **Done** | `EMPTY_ENROLL_HINT` + `emit_device_roster` + first-class `Status`. Planned names present (list regression split empty/enrolled). No revoke hermetic. |
| **Phase 2** Help | **Done** | `after_help` + AC6 hermetic. `cli_help_ia` untouched. |
| **Phase 3** Docs | **Done** | All six F7 surfaces. CHANGELOG Unreleased uses **always** appends. |

Phase 4–5 (targeted/manual/full gate/closeout) are orchestrator-owned.

---

## Missing DoD (product / docs / tests)

None.

README is **not** in F7 / AC9 / Phase 3. Spec §13 “README P3” maps to shipping the missing command (F1–F2 / AC1–AC3), which is done.
