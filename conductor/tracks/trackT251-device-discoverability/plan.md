# T251 Plan — Device discoverability

**Status:** ✅ **Completed** (2026-08-14 PR #167 `038098e`)  
**Spec:** [spec.md](./spec.md) F1–F16 / AC1–AC16 + §14 AI fold-in  
**Category:** UX / FEATURE  
**Ledger TX (on go):** `ledgerful ledger start T251-device-discoverability --category UX --message "device status = enrolled roster + next: replicate status; list/fingerprint frozen"`

---

## AI fold-in (2026-08-14) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates planned work. AI2 L1–L3 are must-pins. AI1 remapped ACs declined (keep AC1–AC16).

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1–M4 / L1 / O1** | AI1 | **Agree** | Already F1/F2/F4–F7 / AC1–AC6 / AC8; named hermetics Phase 1 |
| **AI1 remapped AC7/AC8** | AI1 | **Decline** | Keep AC1–AC16 |
| **AI2 L1** | AI2 | **Agree hard** | Phase 3 CLI-EXIT-CODES.md footnote |
| **AI2 L2** | AI2 | **Agree hard** | Phase 1 extract **plural** T198 only; leave singular error copies |
| **AI2 L3** | AI2 | **Agree hard** | CHANGELOG “always appends `next:`” |
| **AI2 L4** | AI2 | **Agree** | AC1 empty = `list_enrolled_devices` empty; no revoke hermetic |

### Pins locked by fold-in

1. First-class `Status`, not `visible_alias`.
2. `next:` always; CHANGELOG says so.
3. Plural T198 const only; grep that exact sentence.
4. Singular `No enrolled device on this vault…` at `device.rs:139` / `replicate.rs:206` untouched.
5. CLI-EXIT-CODES.md footnote.
6. Keep AC1–AC16.

---

## Preflight (plan time — 2026-08-14)

| Check | Result |
|-------|--------|
| `device status` | clap `unrecognized subcommand 'status'` → **exit 2** |
| `device list` | T198 empty line; exit 0; live vault **0** enrolled |
| `device fingerprint` | Same T198 line; exit 0 |
| `device --help` | No `status` subcommand; after_help omits status |
| `replicate status` | `relay: not configured`; `enrolled_count: 0`; honesty + bootstrap hint; exit 0 |
| `DeviceCommands` | Bootstrap / Fingerprint / List / PackageExport / Enroll / Revoke — **no Status** |
| `run_list` | Table-only; no `--format`; no `next:` |
| T198 F7 | Fingerprint empty aligned with list — keep |
| clap / serde_json / is-terminal / chrono | lock 4.6.1 / 1.0.150 / 0.4.17 / 0.4.44 — **no bumps** (crates.io clap **4.6.6**, serde_json **1.0.151**, chrono **0.4.45**) |
| rustc | 1.95.0 |
| Ledger | 0 pending, 0 unaudited drift |
| T243–T250 | Completed — no rewrite |
| Live daemon start / live `device bootstrap` | **Not** run (F11) |
| `preflight --summary` | Scope `test-alias`; harnesses grok/agy/opencode `wiring=ok`; doctor degraded (backup_recent / recovery_kit / graph_density) — unrelated |
| Recall | ADR-0018 `device`/`replicate` split; T243 visible_alias is a different pattern; no prior T251 pin |
| Embedding / completion :8083/:8081 | Completion unreachable — capture-independent; not blocking |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| device status missing | deferred.md / audit P3 | **DoD** F1–F2 |
| Placeholder F1 alias or combined view | spec draft | **F1/F2** first-class + pointer; combined **declined** |
| Placeholder F2 after_help | spec draft | **F7 / AC6** |
| Placeholder F3 no product fill | spec draft | **F3** |
| T176 list JSON | T176 residual | **Not absorbed** F12 |
| T177 bootstrap→outbox | deferred.md §53 | **Not absorbed** |
| T198 empty copy | T198 F7 | **Keep** F13 |
| Singular error copy (device.rs:139 / replicate.rs:206) | AI2 L2 | **Not absorbed** F12 — leave untouched |
| T252–T255 / T255 doctor ports | peers | **Not absorbed** |

---

## Phase 0 — Ledger + impact (on go)

- [x] `ledgerful ledger status --compact` — expect 0 pending, 0 unaudited drift
- [x] `ledgerful ledger start T251-device-discoverability --category FEATURE` (TX `627392d8-3bc7-4943-97f0-831b455497e9`)
- [x] `ledgerful scan --impact` — LOW (planning dirty only at start)
- [x] Confirm no other agent is editing `device.rs` / `main.rs` DeviceCommands / `device_replicate_cli.rs` / `empty_states_exit_hygiene.rs`

---

## Phase 1 — Red → Green: shared roster + Status (F1 / F2 / F4 / AC1–AC5 / AC8 / AC10)

- [x] Extract **plural** `EMPTY_ENROLL_HINT` + `emit_device_roster`; `run_list` / `run_fingerprint` use the const
- [x] Do **not** edit singular `No enrolled device on this vault…` in `load_local_signing_key` / `load_local_device` (AI2 L2)
- [x] Red: `device status` hermetic still exit 2 unrecognized
- [x] Add `DeviceCommands::Status` after `List`; dispatch `run_status`
- [x] `run_status` = roster + **always** `next: ai-brains replicate status`
- [x] Green AC1–AC5 / AC8 / AC10
- [x] Named hermetics (AI1 O1): `device_status__empty_vault__outputs_hint_and_next_replicate_status`; `device_status__enrolled_vault__outputs_table_and_next_replicate_status`; `device_status__with_format_json_flag__fails_exit_2`; `device_list__*__does_not_contain_next` (split empty/enrolled)
- [x] Do **not** add `--format` / flags on Status
- [x] Do **not** add a revoke-ceremony hermetic (AI2 L4 / F3)

---

## Phase 2 — Help (F7 / AC6 / AC12)

- [x] Device `after_help` adds `ai-brains device status`
- [x] Hermetic `device --help` lists `status`
- [x] `cli_help_ia` still green

---

## Phase 3 — Docs (F7 / AC9)

- [x] CAPABILITIES OutputFormat: `device list` / `device status` / `replicate status` JSON note
- [x] PROTOCOL-COMPAT §5 additive human-only rows (not a compact↔pretty flip)
- [x] OPERATIONS multi-device residuals one-liner
- [x] INSTALL §7 optional tip
- [x] CHANGELOG Unreleased: **always** appends `next:` (empty and enrolled)
- [x] CLI-EXIT-CODES.md one-line footnote: `device status` exit **0** empty/enrolled; extra args clap **2** (AI2 L1)

---

## Phase 4 — Targeted + manual (AC7 / AC14 / AC15)

- [x] `cargo nextest run -p ai-brains-cli` device_status_discoverability + empty_states + device_replicate_cli + cli_help_ia — 37/37
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` (and workspace clippy)
- [x] Manual live vault (**do not bootstrap**): `device status` / `list` / `fingerprint` / `replicate status` / `device status --format json`

---

## Phase 5 — Gate + closeout (AC13)

- [x] `cargo fmt --check` + workspace clippy + workspace nextest PASS. Local deny/audit binaries missing; CI jobs run both
- [x] `ledgerful verify --scope fast` ran fmt/clippy/nextest (deny/audit skipped locally)
- [x] `review.md`; UX/FEATURE; Codex CX2 **PASS**
- [x] conductor → Completed; deferred.md strike; README T251 Completed
- [x] `ai-brains pin` T251 go decision (`dbd728a3-d6de-4227-ae16-84a253400420`)
- [ ] `ledgerful ledger commit` (after closeout squash)

---

## Isolation (do not touch)

- T243–T250 product files
- `replicate.rs` / T176–T178 crypto / schema / enroll / revoke / bootstrap ceremony
- T198 empty sentence (except extract the **plural** success const)
- Singular error copies in `load_local_signing_key` / `load_local_device` (and `replicate.rs`)
- `OutputFormat::parse`
- Doctor 15-check matrix
- Live vault bootstrap / daemon start / install
- `AI_BRAINS_KEY` print / commit
- clap 5 / lockfile pin bumps

---

## Stop-before

- Live `device bootstrap` / enroll / revoke
- Live `daemon start` / `install` / service control
- Ambiguous request to “just enroll this machine”
- Combined-view rewrite of `replicate status`

Do not implement until **go**.
