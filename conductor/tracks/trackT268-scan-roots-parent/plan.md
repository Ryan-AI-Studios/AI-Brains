# T268 Plan — scan-roots parent / `--root`

**Status:** ✅ **Completed** (2026-08-19)
**Spec:** [spec.md](./spec.md) F0–F30 / AC1–AC17 + §13 AI fold-in
**Category:** UX
**Ledger TX (planning):** `7cccacdb-e7fb-41e4-b073-ea4cfb3b3e1a` (DOCS)
**Ledger TX (fold-in):** `52dc7831-9393-4b30-ac82-099bfbf2d435` (DOCS)
**Ledger TX (implement):** `b1f31b8e-43e2-4e2e-a4cf-9825b45b859a` (FEATURE)

---

## AI fold-in (2026-08-19) — `agy-review.md` + `opencode-review.md`

No Highs / Blockers / Majors. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F22:** git spawn `unwrap_or_default()` — scan does not fail if git is missing.
2. **F21:** `/` + case-insensitive `X:\` + UNC `\\server\share`.
3. **F28:** pure `parent_scan_hint(implicit_cwd, unregistered_count, git_toplevel)`.
4. **F29:** hint print `\` on Windows; not `normalize_for_location_compare`.
5. **F2 (b) / AC17:** empty scan still hints.
6. **F3:** JSON `suggested` is `""`, never `null`.
7. **F10:** no JSON `parent_hint` / `next_step`.

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

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact`
- [x] Re-read `project_paths.rs` `scan_roots` / `scan_rows_for_hits` / `emit_scan_human` and `ScanRoots` clap
- [x] Rescan `deferred.md` for new open rows that overlap
- [x] Confirm T273 still the right home for #183 (do not absorb)
- [x] FEATURE TX `b1f31b8e-43e2-4e2e-a4cf-9825b45b859a`

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

- [x] F0 go + FEATURE TX `b1f31b8e-43e2-4e2e-a4cf-9825b45b859a`
- [x] Red: `scan_roots__already_registered__suggested_empty` (AC4)
- [x] Green: `scan_rows_for_hits` empty suggested when owner set
- [x] Red: clap `--root` + positional conflict (AC1)
- [x] Green: `ScanRoots.root` `conflicts_with = "path"` + dispatch `root.or(path)`
- [x] Red: implicit-cwd git + zero unregistered → human `next: … --root` (AC6)
- [x] Green: F28 `parent_scan_hint` + F22 fail-open + F29 `\` display + human-only emit
- [x] AC2 / AC3 / AC7 / AC8 / AC9 / AC10 / AC12 / AC16 / AC17 stay or land
- [x] after_help + CAPABILITIES / OPERATIONS / CHANGELOG (AC13)
- [x] Manual AC15 (no register / rebind / `.env`)
- [x] Phase-1 review → `codex-review` (F25) CX2 PASS → full gate
- [x] conductor **Completed** after local DoD + gate; Phase 6 publish follows

---

## Definition of Done

- [x] F0–F30 + AC1–AC17
- [x] §13 fold-in pins honored
- [x] Medium+ review findings not silently dropped (CX1 P1 tests fixed)
- [x] T273 remains a separate Pending placeholder
- [x] No product commits under this DOCS TX
- [x] Implemented after go; FEATURE TX

**Evidence:** `dev-check.ps1` **[SUCCESS]** nextest **3193** passed (1 skipped). `ledgerful verify --scope full` passed (fmt 2.5s / clippy 6.1s / nextest 115.2s / deny 4.1s / audit 2.7s). Manual AC15: implicit cwd `next: … --root C:\dev` + suggested `—`; `--root C:\dev` siblings; XOR EXIT=2.

---

**Planning 2026-08-19.** Plan-only until **go**.
