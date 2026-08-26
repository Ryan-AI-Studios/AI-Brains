# T306 Plan — PATH install SQLCipher 4.14

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Category:** CHORE / OPS
**Ledger (planning):** `2b0a2dec-7921-4e84-a964-b37cb703457c` (DOCS)
**Ledger (implement):** CHORE TX on **go**. **Do not install until go.**

---

## Preflight (plan time — 2026-08-26)

| Check | Result |
|-------|--------|
| HEAD / tree | `cb5aa49` CLEAN; `origin/main` = `a49acbd` `#222`; ahead **1** |
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
| T305 R3 PATH 4.10 | **DoD** F1–F7 / AC1–AC5 / AC8 |
| T305 Codex PATH older binary | **DoD** same |
| T305 R1/R4/R5 | **Declined** §9 |
| T305 R2 `table_exists` | **T309** |
| T304 R2 tower-http dual | **T307** |
| T300 sparse remediator | **T308**; AC7 expected |
| PATH `ai-brainsd` + T84 graph-off | **T310** (minted) |
| last-PR `#222` Cursor | **N/A empty** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [ ] Lock: rusqlite **0.40.2** (`Select-String` / `cargo pkgid rusqlite`)
- [ ] `perl -v` succeeds
- [ ] PATH `ai-brains doctor --json` `cipher_page` still **4.10** (or already 4.14 → skip F1, still record AC2)
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] `ledgerful ledger start T306-path-install-sqlcipher-414 --category CHORE --message "PATH install locked graph-on CLI (SQLCipher 4.14)"`
- [ ] **Stop-Before** if lock is not 0.40.2, Perl missing, or cwd wrong
- [ ] **Do not** `cargo install` until this phase is checked

## Phase 1 — Install (AC1, F1)

- [ ] From repo root: `cargo install --path crates/ai-brains-cli --locked --features graph`
- [ ] If cargo refuses same-version: retry with `--force` (F1 optional). Do **not** drop `--features graph` or `--locked`
- [ ] If sharing violation on `ai-brains.exe`: **halt** (F13). Do **not** `daemon stop`
- [ ] If openssl-src / Perl Configure fails: **halt** (F12)

## Phase 2 — Prove PATH (AC2–AC5, AC7)

- [ ] `where.exe ai-brains` still cargo bin; LastWriteTime newer than **2026-08-25 14:47:44** (AC1 / F25)
- [ ] `ai-brains doctor --json` filtered: `cipher_page` message contains **`4.14`** (AC2)
- [ ] `graph_feature` message **`available`** (AC3)
- [ ] `vault_open` ok (AC4)
- [ ] No `AI_BRAINS_KEY` / `x'<64 hex>'` in captured output (AC5)
- [ ] `--summary` may stay degraded (`graph_density` / `recovery_kit_event`) — **not a fail** (AC7)
- [ ] If `cipher_page` still `4.10`: **halt** (F26)

## Phase 3 — Closeout (AC6, AC8)

- [ ] `git diff -- crates/ Cargo.toml Cargo.lock` empty (AC6)
- [ ] Mark this plan tasks complete; `conductor.md` T306 **Completed** with PATH message evidence
- [ ] `deferred.md` T305 R3 → **Done** (PATH `4.14`)
- [ ] `ledgerful verify --scope fast` ; `ledgerful ledger commit`
- [ ] Pin: `ai-brains pin "DECISION: PATH ai-brains installed --locked --features graph; doctor cipher_page 4.14.x (T306). Daemon/update leftover is T310." --tx-id <chore-tx>`
- [ ] Phase 6: `track/T306-*` if a closeout commit exists → PR → watch `CI` green → `gh pr merge --squash --delete-branch`. Never `git push origin main`

## DoD

- [ ] PATH `cipher_page` message contains **`4.14`** (AC2)
- [ ] `graph_feature=available` (AC3)
- [ ] No crate/lock diff (AC6); no key leak (AC5)
- [ ] No live encrypt / rebuild / daemon stop (F4)
- [ ] Conductor Completed (AC8)
- [ ] T307 / T308 / T309 / T310 not stolen
