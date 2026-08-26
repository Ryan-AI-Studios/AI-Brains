# T307 Plan — reqwest / tower-http unify

**Status:** **Pending**. Spec [spec.md](./spec.md). **Upstream-blocked** at plan time (reqwest 0.13.4 → tower-http `0.6.8`).
**Category:** CHORE / DEPS
**Ledger (planning):** `6e17c94a-a250-4f24-b579-3b4a66970aa6` (DOCS)
**Ledger (fold-in):** `b4094321-90a3-42c7-984b-b0ff05dd1eac` (DOCS)
**Ledger (implement):** CHORE TX on go **only if** Phase 0 F3 does not halt.

---

## Preflight (plan time — 2026-08-26)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `a084610` CLEAN; `origin/main...HEAD` **ahead 1**. Plan-write was `34379bf` / 0/0 (m1). Branch `main`. T306 `#223` on origin/main. |
| PATH `cipher_page` | **`cipher_version=4.14.0 community`** (T306 done; not this hole) |
| PATH `graph_feature` | **`available`** |
| Workspace reqwest | `"0.13"` json (`Cargo.toml:50`) |
| Lock reqwest | **0.13.4** → `tower-http 0.6.11` |
| Lock tower-http | **0.6.11** (reqwest) **and** **0.7.0** (api-server) |
| Invert 0.6.11 | reqwest → models → cli/brain/retrieval/daemon; desktop |
| Invert 0.7.0 | api-server → ai-brainsd |
| crates.io reqwest | **0.13.4** latest (`cargo info` = **version**; pin = docs.rs `Cargo.toml.orig` `0.6.8` — F22) |
| crates.io tower-http | **0.7.0** latest |
| reqwest master / 0.13.4 Cargo.toml | `tower-http = "0.6.8"` follow-redirect |
| reqwest#3062 | **open**, not merged (`2026-06-29` … last `2026-07-13`) |
| tower-http #712/#722 | merged on git; **not** on crates.io 0.7.0 |
| `routes.rs` | `:66` limit, `:68` trace; no CorsLayer |
| CORS test | `security.rs:154` |
| rustc | **1.95.0** |
| Last PR Cursor | `#223` comments/reviews/issues **empty** — N/A; no T311 |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; plan TX `6e17c94a`; fold-in TX `b4094321` |
| `ISSUES.md` | **Does not exist** |
| Planning bump | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| T304 R2 dual 0.6.11 | **DoD** F1–F3 / AC1–AC2 |
| T304 R4 csrf | **Declined** F5 |
| T308 / T309 / T310 | **Not stolen** |
| last-PR `#223` Cursor | **N/A empty** |
| clap 5 / floor retune | **Declined** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [ ] `cargo info reqwest --color never` — latest **version** only (F22; not the 0.6.8 pin)
- [ ] Read **that** version’s `tower-http` requirement from docs.rs `/crate/reqwest/<ver>/source/Cargo.toml.orig` (or crates.io download)
- [ ] `cargo info tower-http --color never` — latest 0.7.x
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] **If** reqwest still requires tower-http **0.6.x** → **F3 halt**:
  - [ ] Do **not** start a CHORE product TX
  - [ ] Conductor T307 → **Blocked** with dated evidence
  - [ ] `deferred.md` F3 row (crates.io version + `#3062` still open/merged?)
  - [ ] `ledgerful ledger commit` this docs-only halt (or note on existing TX)
  - [ ] **Stop.** Do not patch. Do not git-dep.
- [ ] **Else** (unblocked): `ledgerful ledger start T307-reqwest-tower-http-unify --category CHORE --message "Unify tower-http 0.7 via reqwest <ver>"`
- [ ] **Do not** merge Dependabot remotes

## Phase 1 — Bump (only if F3 does not halt)

- [ ] If published line is **0.14+**: workspace `Cargo.toml:50` `version = "0.14"` (or current major), features **`["json"]` only**
- [ ] If published line is **0.13.x**: **no** toml caret change (`"0.13"` already allows)
- [ ] `cargo update -p reqwest --precise <ver>` (F8). Do **not** `cargo update -p tower-http --precise 0.7.0`
- [ ] Confirm lock: **one** `tower-http` **0.7.x** (AC1). No `0.6.11` package
- [ ] F9 extras: accept resolver; abort if rusqlite / clap / thiserror / tokio leave pins
- [ ] Prefer tower-http **0.7.1+** if crates.io has it (F12); else accept 0.7.0 on json-only

## Phase 2 — Stay-green (AC3–AC6)

- [ ] `cargo clippy -p ai-brains-api-server --all-targets -- -D warnings`
- [ ] `cargo nextest run -p ai-brains-api-server` — CORS `:154` + body-limit `:184`
- [ ] `cargo nextest run -p ai-brains-models`
- [ ] Confirm `routes.rs:66/:68` unchanged (unless documented constructor break)
- [ ] Full workspace `cargo fmt --check` ; clippy `-D warnings` ; nextest ; `cargo deny check` ; `cargo audit`
- [ ] No CorsLayer / CsrfLayer added

## Phase 3 — Closeout (bump path)

- [ ] CHANGELOG Unreleased Changed (AC7)
- [ ] `git diff -- crates/` empty unless listed compile fix (AC8)
- [ ] conductor T307 **Completed**; deferred T304 R2 **Done**
- [ ] `ledgerful verify --scope fast` then full as implement-track requires
- [ ] Phase 6: `track/T307-*` → PR → watch `CI` green → squash-merge. Never `git push origin main`

## Phase 3b — Closeout (F3 halt)

- [ ] conductor **Blocked** (not Completed)
- [ ] deferred F3 evidence dated
- [ ] No `Cargo.toml` / `Cargo.lock` / CHANGELOG product diff
- [ ] T308 / T309 / T310 not stolen

## DoD

- [ ] Dual gone **or** F3 halt with crates.io evidence (AC1)
- [ ] CORS deny intact if a bump ships (AC3)
- [ ] Peer pins unchanged if a bump ships (AC6)
- [ ] Never `git push origin main`; no Dependabot remote merge
- [ ] T308 / T309 / T310 not stolen

## Evidence commands (bump path)

```
cargo info reqwest
cargo update -p reqwest --precise <ver>
Select-String -Path Cargo.lock -Pattern 'name = "tower-http"' -Context 0,1
cargo tree -i tower-http --locked
cargo clippy -p ai-brains-api-server --all-targets -- -D warnings
cargo nextest run -p ai-brains-api-server
cargo nextest run -p ai-brains-models
```
