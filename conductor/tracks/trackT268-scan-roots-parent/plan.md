# T268 Plan — scan-roots parent / `--root`

**Status:** **Pending** (Planned; not Placeholder; not In Progress)
**Spec:** [spec.md](./spec.md) F0–F27 / AC1–AC15
**Category:** UX
**Ledger TX (planning):** `7cccacdb-e7fb-41e4-b073-ea4cfb3b3e1a` (DOCS)

---

## Preflight (plan time — 2026-08-19)

| Check | Result |
|-------|--------|
| HEAD / tree | `a4ac170` CLEAN; `main` == `origin/main` |
| T254 / T266 | ✅ scan-roots + `--format` in source |
| PATH `ai-brains` | `0.1.1`; cwd scan suggests re-register of owned `C:\dev\AI-Brains`; `--root` missing |
| `scan-roots C:\dev` | 17 `.ledgerful` hits; leftover `7d97a456` still owns several siblings |
| Last PR comments | #183 Cursor Bugbot Medium dash-query — **minted T273** |
| Open PR on HEAD | none |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** — do not grow |
| Ledger | 0 pending at scan; planning TX `7cccacdb` |
| `ISSUES.md` | **Does not exist** |
| ledgerful ask | Semantic index dim mismatch (384≠768) — search still hit `scan_roots` |

---

## Phase 0 — on go (re-verify)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact`
- [ ] Re-read `project_paths.rs` `scan_roots` / `scan_rows_for_hits` / `emit_scan_human` and `ScanRoots` clap
- [ ] Rescan `deferred.md` for new open rows that overlap
- [ ] Confirm T273 still the right home for #183 (do not absorb)
- [ ] FEATURE TX (not this DOCS TX)

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| cwd-only default (audit 4/5) | **DoD** F1–F2 / AC1–AC3 / AC6–AC7 |
| already-registered `suggested` | **DoD** F3 / AC4 |
| T254 positional path | **Keep** — `--root` is XOR alias, not a replacement |
| T254 F21 cwd default | **Affirm** F15 |
| last-PR #183 dash-query | **Mint T273** — not this DoD |

## Declined (written)

| Item | Why |
|------|-----|
| Default scan = parent / `C:\dev` | T254 F21 — machine habit |
| Auto-register / `--apply` | T254 F23 |
| Leftover rebind | T259 |
| T266 format retune | Closed |
| T269 / T270 / T272 | Peers |
| #183 dash-query | T273 |
| T254 F12 TTY-auto hermetic | Soft, not parent/`suggested` |
| clap 5 / pin bumps / camino / DTO | F9 / F10 |
| T240 F2 / T255 bag | Standing |

---

## Tasks (on go)

- [ ] F0 go + FEATURE TX
- [ ] Red: `scan_roots__already_registered__suggested_empty` (AC4)
- [ ] Green: `scan_rows_for_hits` empty suggested when owner set
- [ ] Red: clap `--root` + positional conflict (AC1)
- [ ] Green: `ScanRoots.root` `conflicts_with = "path"` + dispatch `root.or(path)`
- [ ] Red: implicit-cwd git + zero unregistered → human `next: … --root` (AC6)
- [ ] Green: `parent_scan_hint` + human-only emit (no JSON key)
- [ ] AC2 / AC3 / AC7 / AC8 / AC9 / AC10 / AC12 stay or land
- [ ] after_help + CAPABILITIES / OPERATIONS / CHANGELOG (AC13)
- [ ] Manual AC15 (no register / rebind / `.env`)
- [ ] Phase-1 review → `codex-review` (F25) → full gate
- [ ] conductor **Completed** only after merge hygiene (implement-track Phase 6)

---

## Definition of Done

- [ ] F0–F27 + AC1–AC15
- [ ] Medium+ review findings not silently dropped
- [ ] T273 remains a separate Pending placeholder
- [ ] No product commits under this DOCS TX
- [ ] Status stays **Pending** until implement-track go

---

**Planning 2026-08-19.** Plan-only until **go**.
