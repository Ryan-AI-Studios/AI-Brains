# T281 Plan — Nightly HTTP `/health` vs daemon TCP contrast line

**Status:** **Completed** 2026-08-22
**Spec:** [spec.md](./spec.md) F0–F32 / AC1–AC14 + §13 AI fold-in
**Category:** OPS / UX / HONESTY
**Ledger TX (planning):** `b9b8c77d-3a92-476d-9887-1b7dfeed7fe2` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `40fde806-a4f2-49da-9ff7-d917fa3605cd` (DOCS)
**Ledger TX (implement):** BUGFIX `435f6228-5052-406c-baf1-5bd2234cafaf`

---

## AI fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 1**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F32 / AC2:** `"timeout (750ms)"` → None; call site raw `completion_label`.
2. **AC1 / F25:** U+2260 byte-exact; `assert_ne!` ASCII `!=`.
3. **F1:** 31 chars `HTTP /health 750ms ≠ daemon TCP` — **not** OpenCode invented dotted/TCP-budget string; **not** strip T269 suffix.
4. **F19:** no skill `--status` section → skip skill edit.
5. **Phase 4:** `scripts/dev-check.ps1`.
6. **AC7:** extend live test `:77`; keep T255/T269 comment numbers.
7. **Affirm:** #196 N/A; no T285.

---

## Preflight (plan time — 2026-08-22)

| Check | Result |
|-------|--------|
| HEAD / tree | **Plan dogfood:** `d89f5e6` T280 `#196`. CLEAN. `origin/main` = HEAD (`0 0`). Nightly product last T269 `9008074` `#186` |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-21 05:55. **T270** on PATH (T269 chrome present). Contrast line **absent**. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **3581** (volatile); in-context 0/0/0; grants **0 of 3**; Scope `3581317d` |
| PATH `nightly --status` | Heading + Last Result **0** + Completion **`probe=ok`** + Embedding `ok` + Router **267009**. Timeout **not** reproduced this session |
| PATH `nightly --status --format json --quick` | `schema_version` 1; probes `"skipped"`; frozen keys present |
| `daemon status` | Stopped + LLM/Embedding **Open** (TCP). `next: ai-brains daemon start` |
| `nightly --help` after_help | Already TCP + `/health` + 750 (T269 AC6) |
| Last PR comments | #196 T280 — **empty** (N/A). #188 closed by T284. No T285 |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2); uuid lock **1.23.1** (1.25.0); tokio **1.52.3** (1.53.1) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** — do not grow. `sync.rs` #2 / `forget.rs` #3 / `governed_common.rs` #4 / `context.rs` #5. `nightly.rs` **2133** / `nightly_status.rs` **638** / `daemon.rs` **1188** |
| Ledger | 0 pending / 0 drift at scan; planning TX `b9b8c77d` |
| `ISSUES.md` | **Does not exist** (F23) |
| ledgerful search | `format_probe_label_human` `nightly_status.rs:11`; `NIGHTLY_STATUS_PROBE_TIMEOUT` `nightly.rs:13`; daemon TCP `daemon.rs:749` |
| Online | clig.dev human-first + just-enough; k8s tcpSocket ≠ httpGet; llama.cpp #20684 `/health` queued; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [x] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`) (OpenCode O-5)
- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before BUGFIX TX)
- [x] Re-read `NIGHTLY_STATUS_PROBE_TIMEOUT` `nightly.rs` `:13` — **do not edit** (F2)
- [x] Re-read `--quick` skip `:52–66` and Completion print `:195–212`
- [x] Re-read `format_probe_label_human` `nightly_status.rs` `:11` — **do not restyle** (F5)
- [x] Re-read `FROZEN_KEYS` `:260` — **do not add** (F3)
- [x] Re-read `daemon.rs` `:749` TCP — **do not import `probe_health`** (F10)
- [x] Re-read after_help `main.rs` `:1137` + AC6 `:658` — **freeze** (F7)
- [x] Confirm T269 hermetic `--quick` `tests/nightly_status.rs` `:77`
- [x] Rescan `conductor/deferred.md` — T281 rows absorbed; no new overlapping open rows
- [x] Confirm #196 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [x] Re-dogfood `nightly --status` + `daemon status` **read-only**. **Did not** mutate schtasks. **Did not** force llama load
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [x] BUGFIX TX
- [x] Did **not** `cargo install`; did **not** grow `project.rs` / `sync.rs` / `forget.rs` / `daemon.rs` / `doctor.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit nightly Completion timeout vs daemon Open | **DoD** F1–F5 / AC1–AC2 / AC7 / AC10 |
| T269 closeout two-truths on status block | **Lift** F1 — after_help already shipped |

## Declined (written)

| Item | Why |
|------|-----|
| Raise 750 ms | F2 |
| Unify daemon HTTP | F10 |
| JSON budget / contrast field | F3 |
| TCP-probe from nightly | F1 / F27 |
| Doctor 16th | F11 |
| T282 / T283 / leftover rebind / T240 F2 / clap 5 / rusqlite 0.40 | F12/F17 |
| last-PR #196 Cursor | N/A empty |
| Dependabot rusqlite `#61` | F12 — no T285 |
| Embedding-only timeout line | F26 |
| OpenCode invented F1 (`  · … timeout (100ms×5)`) | §13 — freeze is 31-char F1 |
| OpenCode “nightly blocks on daemon TCP” | §13 — live is `probe_health` HTTP |
| Strip T269 `timeout (750ms)` suffix | F5 |

---

## Phase 1 — Red (TDD)

- [x] `http_vs_tcp_contrast__equals_frozen_line` — AC1 (`assert_eq!` F1; `assert_ne!` ASCII `!=`)
- [x] `completion_timeout_contrast_line__timeout__some_frozen` — AC2 Some
- [x] `completion_timeout_contrast_line__passthrough_labels__none` rstest — AC2 None including `#[case("timeout (750ms)")]` (F32 / OpenCode M-1)
- [x] Commit red allowed

---

## Phase 2 — Green

- [x] F1 const `HTTP_VS_TCP_CONTRAST` + `completion_timeout_contrast_line` in `nightly_status.rs`
- [x] `nightly.rs`: after Completion `println!`, `if let Some(line) = completion_timeout_contrast_line(completion_label) { println!("{line}"); }` — **raw** `completion_label`, not `completion_human` (F32)
- [x] AC7 hermetic `--quick` does not contain `HTTP /health` or `daemon TCP` (additive comment on `:77`; do not renumber T255 AC10 / T269 AC8)
- [x] AC3–AC6 / AC8–AC9 / AC13–AC14 stay green
- [x] Commit green

---

## Phase 3 — Docs

- [x] CAPABILITIES T269 bullet: additive timeout next-line
- [x] OPERATIONS Completion/Embedding: same sentence
- [x] PROTOCOL-COMPAT: no new required keys
- [x] CHANGELOG T281
- [x] Skill one-liner: **skip** — no `nightly --status` subsection (F19 / OpenCode O-3)
- [x] conductor Completed on implement closeout (not this planning pass)

---

## Phase 4 — Verify

- [x] Targeted nextest: `-p ai-brains-cli` F1/F2 units; `--test nightly_status`; `nightly__help__names_nightly_heading_and_probe_budget`; T247/T255/T269 stay-green
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [x] `cargo fmt --check`
- [x] Primary review → `review.md`; mediums not silently dropped
- [x] Cross-model `codex-review` (F22)
- [x] Full workspace gate (`scripts/dev-check.ps1` / `ledgerful verify --scope full`)
- [x] Classify-only live `cargo run -p ai-brains-cli -- nightly --status` (AC10). **No** schtasks mutate. **No** forced llama load

---

## DoD (checkable)

- [x] Unit: F1 const exact 31-char line (AC1)
- [x] Unit: helper Some iff `== "timeout"`; None for `"timeout (750ms)"` (AC2 / F32)
- [x] T269 suffix / heading / after_help stay green (AC3/AC6)
- [x] JSON timeout still raw `"timeout"` (AC4)
- [x] Router 267009 frozen (AC5)
- [x] Hermetic `--quick` no F1 / no `(750ms)` (AC7)
- [x] Schedule block not injected (AC8)
- [x] Live classify-only AC10 (`cargo run`, not PATH); pass-with-observed-data
- [x] 750 ms not raised; `daemon.rs` / `llama_cpp.rs` untouched (AC12)
- [x] No `cargo install`
- [x] Diff omits `project.rs` / `sync.rs` / `forget.rs` / `daemon.rs` / `doctor.rs`
- [x] implement-track Phase 6: push `track/T281-*` → PR → watch GHA `CI` green → squash-merge → prune (never `git push origin main`)

---

## Stop-before

- Live schtasks mutate / write `.cmd` / Router registration / `.env` rewrite / `cargo install` / leftover rebind / grant bootstrap
- Raise 750 ms / unify daemon HTTP / doctor 16th / JSON schema bump
- Force llama.cpp generation to reproduce timeout
- Scope exceeds T281 (do not steal T282–T283, T269 suffix, T255 JSON keys, T199 TCP)
- Ambiguous spec vs src after Phase 0 — halt and ask
