# T249 Plan — Scope / daemon / doctor presentation

**Status:** ✅ **Completed** (2026-08-14 PR #163 `5fd264a`)  
**Spec:** [spec.md](./spec.md) F1–F13 / AC1–AC16 + §14 AI fold-in  
**Category:** UX / FEATURE  
**Ledger TX (on go):** `ledgerful ledger start T249-scope-daemon-doctor-presentation --category FEATURE --message "TTY human for scope resolve (auto); daemon Stopped next:; doctor --summary; JSON keys frozen"`

---

## AI fold-in (2026-08-14) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates planned work. AI2 M1 is a must-pin test lock (case-sensitive `--format`). AI1 remapped ACs declined.

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1–M4 / L1–L2 / O1** | AI1 | **Agree** | Already F1/F2/F4/F6/F9 / AC1–AC9 |
| **AI1 remapped ACs** | AI1 | **Decline** | Keep AC1–AC16 |
| **AI2 M1** | AI2 | **Agree hard** | AC16 `JSON`/`Pretty` exit 2 |
| **AI2 L1** | AI2 | **Agree hard** | Phase 3 `status_next_line`; no live-daemon Stopped hermetic |
| **AI2 L2** | AI2 | **Agree hard** | Phase 4 all `DoctorOptions` constructors (9+1) |
| **AI2 L3** | AI2 | **Agree hard** | AC11 `--format json --summary`; AC12 `--summary --fail-on-degraded` |
| **AI2 L4** | AI2 | **Agree** | Phase 2: no TempEnv mandate |
| **AI2 L5** | AI2 | **Agree** | Phase 5 `help_ia` contains lock |

### Pins locked by fold-in

1. **AC16:** case-sensitive `--format` (`JSON` / `Pretty` exit 2).
2. **F4:** `status_next_line` unit only.
3. **AC11/AC12:** JSON wins for `--format json`; summary+fail-on-degraded still 1.
4. **F6:** `summary` field on every `DoctorOptions` constructor.
5. **F9:** Start-here `--format json` locked.
6. **No TempEnv mandate** on scope hermetics.

---

## Preflight (plan time — 2026-08-14)

| Check | Result |
|-------|--------|
| `scope resolve` | Pretty JSON; `authoritative=true`; `Repository:441837f6-…`; exit 0 |
| `scope resolve --format human` | `scope:` / `confidence: High (authoritative)` / evidence; **no** `next:` |
| clap Scope | `default_value = "json"`; `Option<String>`; no `value_parser`; unknown → `OutputFormat::parse` → Json |
| `emit_scope_human` | Exists in `governed_common.rs`; unused as default |
| `daemon status` | `Status: Stopped`; backends Open; no vault; no PID; no `next:`; exit 0 |
| Daemon clap | `Status` has **no** `--format` |
| `doctor` | `degraded`; 15 checks; 3 warn / 2 skip; exit 0 |
| `doctor --summary` | clap unexpected argument; **exit 2** |
| Doctor clap | `--format human\|json`; `--json`; **no** `--summary` |
| T180 / governed_surface | Already pass `--format json --local` |
| clap / serde_json / is-terminal / chrono | lock 4.6.1 / 1.0.150 / 0.4.17 / 0.4.44 — **no bumps** (crates.io clap 4.6.6, serde_json 1.0.151, chrono 0.4.45) |
| rustc | 1.95.0 |
| Ledger | 0 pending, 0 unaudited drift |
| T243 / T245 / T246 / T247 / T248 | Completed — no rewrite |
| Live daemon start/install | **Not** run (F11) |
| Preflight | Scope `test-alias`; doctor degraded (backup_recent / recovery_kit / graph sparse) — unrelated |
| Recall | T160 #20 + T192 doctor + T199 keyless status; no prior T249 pin |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Scope/daemon/doctor presentation | deferred.md / audit Q7 | **DoD** F1–F7 |
| Placeholder F1 TTY scope pretty | spec draft | **F1/F2** |
| Placeholder F2 daemon one screen | spec draft | **F4** Stopped `next:` (not sc query) |
| Placeholder F3 doctor flag or absence | spec draft | **F6 real `--summary`** (exit 2 today) |
| T160 human unused as default | live | **F1** |
| T180 JSON pretty + keys | T180 | **AC3** — freeze |
| T192 no TTY-smart doctor | T192 F10 | **F7** summary opt-in |
| T199 no JSON / keep Status strings | T199 | **F4/F5** |
| T255 / T250 / T241 leftovers / T226 O1 | peers | **Not absorbed** |

---

## Phase 0 — Ledger + impact (on go)

- [x] `ledgerful ledger status --compact` — expect 0 pending, 0 unaudited drift
- [x] `ledgerful ledger start T249-scope-daemon-doctor-presentation --category FEATURE`
- [x] `ledgerful scan --impact`
- [x] Confirm no other agent is editing `scope.rs` / `doctor.rs` / `daemon.rs` / `governed_common.rs` / `contracts/scopes.rs` / `contracts/doctor.rs`

---

## Phase 1 — Red → Green: scope formatter (F1 / F2 / AC1–AC2)

- [x] `resolve_scope_format(&str, is_tty)` — clap rejects unknown; `_` fail-closed json after parser; **no** `other` passthrough
- [x] `format_scope_human` — existing field order + `next: ai-brains project whoami` last iff `!authoritative`
- [x] Authoritative fixture has **no** `next:`
- [x] Units AC1–AC2 (constructed `ScopeResolvedResponse`; no vault)

---

## Phase 2 — Wire scope clap (F1 / F3 / F9 / AC3–AC6)

- [x] Resolve `--format: String` default `auto` + `value_parser` (not `Option<String>`)
- [x] `ResolveOptions.format: String` + `main.rs` dispatch
- [x] `run_resolve` uses local resolver (not `OutputFormat::parse`)
- [x] JSON path still `emit_json` (`to_string_pretty`); keys frozen
- [x] after_help: TTY example + `--format json`
- [x] Hermetic AC3–AC6 + **AC16** (`--format JSON` / `Pretty` exit 2). **No TempEnv mandate** (AI2 L4 — units are pure; hermetics use `cmd.env`)

---

## Phase 3 — Daemon Stopped next-step (F4 / F5 / AC7–AC8)

- [x] Extract `status_next_line(is_running) -> Option<&'static str>` (AI2 L1)
- [x] When `!is_running`, print exact `next: ai-brains daemon start` after existing lines
- [x] Running path omits `next:`
- [x] Do **not** change probe policy, backend retries, vault section, or tasklist
- [x] Unit AC7 on the helper; **do not** add a hermetic that asserts Stopped against a live daemon
- [x] Existing T85/T94/T128/T199 suites AC8
- [x] Do **not** start/install the live daemon

---

## Phase 4 — Doctor `--summary` (F6 / F7 / AC9–AC12)

- [x] clap `--summary` bool on `Doctor` + `DoctorOptions.summary` on **all** constructors (9 in `doctor.rs` tests + `main.rs` — AI2 L2)
- [x] `format_doctor_summary` — header counts + `attention:` warn/fail only or `No issues.`
- [x] Default emit path unchanged (no `--summary`)
- [x] JSON wins for `--json` **and** `--format json` (AC11)
- [x] Units AC9; hermetic AC10–AC12 including `--summary --fail-on-degraded` exit 1 (AI2 L3)
- [x] No new checks; matrix 15 unchanged

---

## Phase 5 — Docs (F9 / AC14)

- [x] `Docs/CAPABILITIES.md` OutputFormat `scope resolve` row + doctor `--summary` + daemon `next:`
- [x] `Docs/PROTOCOL-COMPAT.md` §5 scope TTY/pipe + keys unchanged + case-sensitive tokens
- [x] `Docs/OPERATIONS.md` TTY vs json / `--summary` / Stopped next-step
- [x] Skill one-liner (`.agents/skills/ai-brains/SKILL.md`)
- [x] `CHANGELOG.md` T249 row only
- [x] T204 Start here: additive lines only (keep `scope resolve --format json`)
- [x] `help_ia` unit: `ROOT_AFTER_LONG_HELP.contains("ai-brains scope resolve --format json")` (AI2 L5)

---

## Phase 6 — Live dogfood + gate (on go; **no daemon start**)

- [x] TTY `scope resolve` is human
- [x] Piped `scope resolve` parses as JSON (`api_version`, `authoritative`)
- [x] `--format xml` **and** `--format JSON` exit 2 (AC16)
- [x] TTY `daemon status` while Stopped shows `next:`
- [x] TTY `doctor --summary` lists this vault’s warn names and is shorter than default
- [x] Targeted nextest + clippy (full gate after review)
- [x] Review log + conductor Completed only after go+ship

---

## Isolation checklist

- [x] No live `daemon start` / `install` / `stop`
- [x] No `resolve_scope` / grants / soft-resolve rewrite
- [x] No `contracts/scopes.rs` / `contracts/doctor.rs` field change
- [x] No `daemon_probe.rs` constant change
- [x] No doctor new checks / T255 model ports / retention_plan
- [x] No `OutputFormat::parse` change (T227 F34)
- [x] No new crates / lock bumps
- [x] No T243 / T245 / T246 / T247 / T248 rewrite
- [x] No `AI_BRAINS_KEY` print/commit
