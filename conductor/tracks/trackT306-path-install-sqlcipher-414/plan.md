# T306 Plan — PATH install SQLCipher 4.14

**Status:** **Completed** 2026-08-26. Spec [spec.md](./spec.md).
**Category:** CHORE / OPS
**Ledger (planning):** `2b0a2dec-7921-4e84-a964-b37cb703457c` (DOCS)
**Ledger (fold-in):** `b04594d2-d70a-44a1-89a1-90e408715414` (DOCS)
**Ledger (implement):** CHORE TX `927f9b00-c0a6-4fd1-833b-ddf4772baa90`

---

## Preflight (plan time — 2026-08-26)

| Check | Result |
|-------|--------|
| HEAD / tree | Review `30894bf` CLEAN; `origin/main` = `a49acbd` `#222`; ahead **2**. Plan-write was `cb5aa49` / ahead **1** (OpenCode m1). |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **25,216,512** B; mtime **2026-08-25 14:47:44**; **0.1.3** |
| PATH `cipher_page` | **`cipher_version=4.10.0 community`** (`doctor --json`) |
| PATH `graph_feature` | **`available`** |
| PATH `--summary` | degraded `ok=11 warn=2`; `cipher_page` **hidden** (ok) |
| PATH `ai-brainsd` | mtime **2026-08-22 14:48:10** — **T310**, not this DoD |
| Lock rusqlite | **0.40.2** / libsqlite3-sys **0.38.2** / hashlink **0.12.1** |
| Workspace rusqlite | exact **0.40.2** (`Cargo.toml:57`) |
| crates.io rusqlite | **0.40.2** (still latest) |
| `GRAPH_REINSTALL_SOOT` | `cargo install --path crates/ai-brains-cli --locked --features graph` |
| Perl | **v5.42.2** MSWin32-x64 |
| rustc | **1.95.0** |
| Last PR Cursor | `#222` comments/reviews/issues **empty** — N/A |
| Open PRs | **none** |
| T310 leftover | minted this pass (daemon + T84 `run_update`) |
| Ledger | 0 pending / 0 drift at scan; this TX `2b0a2dec` |
| `ISSUES.md` | **Does not exist** |
| Planning `cargo install` | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| T305 R3 PATH 4.10 | **DoD** F1–F7 / AC1–AC5 / AC8 — **Done** `4.14.0 community` |
| T305 Codex PATH older binary | **DoD** same — **Done** |
| T305 R1/R4/R5 | **Declined** §9 |
| T305 R2 `table_exists` | **T309** |
| T304 R2 tower-http dual | **T307** |
| T300 sparse remediator | **T308**; AC7 expected |
| PATH `ai-brainsd` + T84 graph-off | **T310** (minted) |
| last-PR `#222` Cursor | **N/A empty** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [x] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [x] Lock: rusqlite **0.40.2** (`Select-String` / `cargo pkgid rusqlite`)
- [x] `perl -v` succeeds (v5.42.2)
- [x] PATH `ai-brains doctor --json` `cipher_page` still **4.10** before F1
- [x] Rescan `deferred.md` open overlapping rows
- [x] `ledgerful ledger start T306-path-install-sqlcipher-414 --category CHORE --message "PATH install locked graph-on CLI (SQLCipher 4.14)"` → `927f9b00-c0a6-4fd1-833b-ddf4772baa90`
- [x] **Stop-Before** if lock is not 0.40.2, Perl missing, or cwd wrong — N/A green
- [x] **Do not** `cargo install` until this phase is checked

## Phase 1 — Install (AC1, F1)

- [x] From repo root: `cargo install --path crates/ai-brains-cli --locked --features graph`
- [x] First attempt: release built (~11m) then **Access denied** replacing PATH exe (PID 28316 hung `preflight --summary`). Halted per F13; did **not** stop daemon.
- [x] Elevated retry: replace succeeded (`Replaced package … executable ai-brains.exe`) — no `--force` needed
- [x] openssl-src / Perl Configure: ok

## Phase 2 — Prove PATH (AC2–AC5, AC7)

- [x] `where.exe ai-brains` = `C:\Users\RyanB\.cargo\bin\ai-brains.exe`; LastWriteTime **2026-08-26 6:54:32 AM** (after 2026-08-25 14:47:44) — F25 supporting
- [x] `cipher_page` message **`cipher_version=4.14.0 community`** (AC2)
- [x] `graph_feature` message **`available`**; `ai-brains graph update --format json` returned nodes/edges (AC3)
- [x] `vault_open` ok `opened read-only` (AC4)
- [x] No `AI_BRAINS_KEY` / `x'<64 hex>'` in captured `--json` (AC5)
- [x] `--summary` degraded (`graph_density` E/N=0.409, `recovery_kit_event`) — **not a fail** (AC7)

## Phase 3 — Closeout (AC6, AC8)

- [x] `git diff -- crates/ Cargo.toml Cargo.lock` empty (AC6)
- [x] Mark this plan tasks complete; `conductor.md` T306 **Completed** with PATH message evidence
- [x] `deferred.md` T305 R3 → **Done** (PATH `4.14`); T306 residuals appended
- [x] `ledgerful verify --scope fast` ; `ledgerful ledger commit`
- [x] Pin: PATH install decision with CHORE tx-id
- [x] Phase 6: `track/T306-*` → PR → watch `CI` green → squash-merge

## DoD

- [x] PATH `cipher_page` message contains **`4.14`** (AC2)
- [x] `graph_feature=available` (AC3)
- [x] No crate/lock diff (AC6); no key leak (AC5)
- [x] No live encrypt / rebuild / daemon stop (F4)
- [x] Conductor Completed (AC8)
- [x] T307 / T308 / T309 / T310 not stolen
