# T297 Plan — daemon status Stopped vs backend TCP Open

**Status:** **Pending** (Planned; not Placeholder). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F34 / AC1–AC14
**Category:** UX / HONESTY
**Ledger TX (planning):** `3f147d91-b4f9-42b2-a8c3-8ea01dd1292d` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `338d0a5b-9dba-446a-8e07-89e0af81f610` (DOCS)
**Ledger TX (implement):** BUGFIX TX on **go**

---

## AI fold-in (2026-08-24) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 1**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F28/AC8:** keep-bound `TcpListener` hermetic is required `run_status` wiring proof (OpenCode M1).
2. **F35/AC1:** rstest 8 triples; include `(false, true, true)`.
3. **F30/AC6:** both-Open tail is exactly `[const, next:]`.
4. **F36:** capture by backend `name`, not both-bools-from-one-state.
5. **F20/AC7:** exact after_help uses `TCP connect`; in-process `try_parse_from`.
6. **F19:** docs distinguish T281 HTTP contrast vs T297 IPC vs TCP.

---

## Preflight (plan time — 2026-08-24)

| Check | Result |
|-------|--------|
| HEAD / tree | `0132707` T296 `#212` on `main`. CLEAN. `origin/main` = HEAD (`0 0`). |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T249 `next:` + T85 TCP.** No T297 contrast. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **4123**. In-context **0/0/0**. Word **393**. Scope `3581317d`. |
| PATH `daemon status` | **Running** + Vault 145.5 MB + Memories **48639** + LLM **8081 Open** + Embedding **8083 Open** + `PID: 4536`. No `next:`. No contrast. Exit **0**. |
| PATH `--no-project-context` | Running + LLM **11434 Open** (Ollama default unset) + Embedding **8080 Closed**. Const must **not** say llama.cpp. |
| Last PR comments | #212 T296 — Cursor/Bugbot/reviews/issue **empty**. **N/A. No T301.** |
| Open PR on HEAD | none (Dependabot remotes: rusqlite `#61`, chrono `#62`, tokio `#59`, thiserror `#60`, tower-http `#58`, actions `#68–#72`) |
| Pins | clap lock **4.6.1** (crates.io **4.6.6**; **no clap 5**); rusqlite **0.39.0** (0.40.2); serde_json **1.0.150**; thiserror **2.0.18**; tokio **1.52.3**; reqwest **0.13.4** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** (**3.897**) — do not touch. `daemon.rs` **1188** — not top-10. |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX) |
| `ISSUES.md` | **Does not exist** (F23) |
| ledgerful search | `status_next_line` `daemon.rs:697` / `run_status` `:706` / early-route `main.rs:4138` |
| Online | clig.dev just-enough + next; Kubernetes tcpSocket ≠ httpGet; llama.cpp #20684 / closed #20799; clap 4.6.6 after_help |
| Skill | Did **not** `daemon start`/`stop`. Did **not** mutate schtasks. |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before BUGFIX TX)
- [ ] Re-read `run_status` `daemon.rs` **`:706–805`** — TCP loop + `status_next_line` print; confirm no contrast yet; names `"LLM backend"` / `"Embedding backend"` for F36
- [ ] Re-read T94 `smoke.rs` **`:3103–3146`** — **drop-then-delay**; AC8 is **keep-bound** (do not copy the drop)
- [ ] Re-read T85 `smoke.rs` **`:2901–2916`** — `contains("Closed")` only; do **not** add `!contains` const
- [ ] Re-read `status_next_line` **`:696–703`** — **do not change strings** (F32)
- [ ] Re-read TCP 5×100 ms **`:747–770`** — **do not retune** (F7)
- [ ] Re-read early-route `main.rs` **`:4133–4142`** — **do not move**
- [ ] Re-read `DaemonCommands::Status` **`:3109`** — additive after_help only
- [ ] Re-read T199 hermetic `tests/daemon_status_vault_independence.rs` + smoke T85/T94
- [ ] Re-read T281 `HTTP_VS_TCP_CONTRAST` — **do not import**
- [ ] Rescan `conductor/deferred.md` — T297 absorbed; T298–T300 / T240 F2 / 750 ms not stolen
- [ ] Confirm `#212` still empty Cursor; no mint; Dependabot `#61` still not this track
- [ ] Re-dogfood `daemon status` **read-only**. Record Status + Open/Closed. **Did not** `daemon stop`/`start`
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0** — **no bump**
- [ ] BUGFIX TX (new)
- [ ] Did **not** `cargo install`; did **not** grow `project.rs` / `doctor.rs` / `nightly.rs`

---

## Absorbed deferred

- [ ] Audit Stopped vs LLM Open → F1–F6 / AC1–AC6 / AC10
- [ ] Placeholder Manual `daemon status` / do not start daemon → AC10 / F11
- [ ] T281 F27 Stopped + port Open → this track
- [ ] T296 F11 / OpenCode m3 → this track
- [ ] last-PR #212 Cursor N/A → F14 no T301
- [x] Fold-in OpenCode M1 keep-bound AC8; F35 rstest; F36 name-match; F30 both-Open

---

## Declined (written)

| Item | Why |
|------|-----|
| JSON / `--format` | F8 |
| Unify HTTP `/health` | F9 |
| Raise 750 | F10 |
| Live `daemon stop`/`start` | F11 |
| Doctor Safety vs Status IPC | F27 |
| T298–T300 / leftover `--write` | F17 |
| clap 5 / rusqlite 0.40 | F12 |
| T240 F2 | F17 |

---

## Phase 1 — TDD red

- [ ] `status_backend_contrast_line__matrix` rstest 8 triples fails (includes `(false, true, true)`)
- [ ] `backend_open_ne_daemon__uses_u2260_not_ascii` fails
- [ ] `status_report_tail__stopped_open_pair__single_contrast_then_next` rstest (F30) fails
- [ ] `status_report_tail__stopped_closed__next_only` fails
- [ ] `status_report_tail__running__empty` fails
- [ ] `daemon__help__status_names_backend_tcp` (`try_parse_from`) fails
- [ ] `daemon_status__keep_bound_listener__contrast_when_stopped` AC8 fails
- [ ] Red commit allowed

---

## Phase 2 — green

- [ ] Const `BACKEND_OPEN_NE_DAEMON` + helpers (F1–F5)
- [ ] `run_status` **F36 name-match** Open bools; prints `status_report_tail` after PID
- [ ] `status_next_line` units stay green
- [ ] Status exact F20 `after_help` (`TCP connect`)
- [ ] AC8 keep-bound listener hermetic (hold `TcpListener`; last line `next:` when Stopped)
- [ ] Green commit allowed

---

## Phase 3 — docs

- [ ] `Docs/CAPABILITIES.md` `:110` last-line `next:` kept; additive contrast; T281 vs T297 distinction (F19)
- [ ] `Docs/OPERATIONS.md` `:558` additive; T281 vs T297 distinction
- [ ] Root `CHANGELOG.md` T297 Unreleased
- [ ] PROTOCOL-COMPAT untouched

---

## Phase 4 — verify

- [ ] Targeted: `cargo nextest run -p ai-brains-cli -- status_backend_contrast status_report_tail status_next_line daemon_status daemon__help keep_bound`
- [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [ ] Manual AC10 (`cargo run`, read-only `daemon status`). Record transcript in `review.md`. **No** daemon stop/start
- [ ] `scripts/dev-check.ps1`
- [ ] Phase-1 review → `review.md`
- [ ] `codex-review` (F22) → `review.codex.md`

---

## Phase 5 — closeout

- [ ] conductor T297 **Completed**
- [ ] deferred.md T297 closeout table
- [ ] README-T285-T300 T297 Completed
- [ ] `ai-brains pin` DECISION (Stopped+Open prints `backend TCP Open ≠ daemon`; next: still last; do not start daemon)
- [ ] BUGFIX TX commit
- [ ] 0 pending / 0 drift

---

## Phase 6 — publish (standing)

- [ ] Push `track/T297-*` (never `git push origin main`)
- [ ] Open PR to `main` if none
- [ ] `gh run watch --exit-status` until GHA `CI` **every job green**
- [ ] `gh pr merge --squash --delete-branch`
- [ ] `git fetch --all --prune`; point local `main` at `origin/main`; delete merged local `track/T297-*` only

---

## DoD

- [ ] Stopped+Open prints frozen contrast; `next:` still last; Running+Open omits contrast
- [ ] Units AC1–AC6 red-then-green; U+2260 locked
- [ ] Manual AC10 recorded; **did not** start/stop daemon
- [ ] Full gate green; contracts/pins unchanged
- [ ] Published (Phase 6)
