# T193 Plan — Path Residual Elevation

Status: **In Progress** (implement 2026-08-02). Spec: [spec.md](./spec.md).

## Preconditions

- [x] Inventory residual sites (`cap_open`, `artifact_security`, `token.rs`, `recovery::write_kit_file`, migrate/shadow/dogfood)
- [x] Research cap-std / cap-fs-ext **4.0.2** (still current; hold pin); FollowSymlinks::No required on write
- [x] Expand freezes F1–F30 + AC1–AC12 + priority matrix P0/P1/P2/R
- [x] Roll deferred T190 write/token/ambient → this track
- [x] Pin plan decision (`ai-brains pin` T193 planning)
- [x] **AI fold-in** (AI1 affirm + handle hardlink; AI2 M1–M5, L1–L6, O1 preferred) — disposition spec §15
- [x] `ledgerful ledger start T193-PathResidualElevation --category SECURITY` *(TX a52b3a65-fe17-4553-a919-2494a1c56426)*

## Deferred rolled in

| Item | Disposition |
|------|-------------|
| T190 ambient CLI / write / token residuals | **Absorb** — core |
| T188 artifact/kit write pre/post reparse | **Absorb P0** |
| R-12 residual rewrite | **Absorb** on ship |
| Migrate/shadow/evaluate report paths | **P1** elevate if free |
| Backup pure-ambient create | **P1** evaluate (AI2 L3) |
| Soft-canon as TOCTOU close | **Not absorbed** |
| Soft-skip symlink CI proof | Keep verification residual |
| T195 multi-user residuals | **Not absorbed** |
| T196 units / CONTRIBUTING | **Not absorbed** |
| WASI / plugin host | **Not absorbed** |

## Research pins (2026-08-02)

| Dep / fact | Pin |
|------------|-----|
| `cap-std` | workspace `"4.0"` → lock **4.0.2** |
| `cap-fs-ext` | `"4.0"` + feature `cap-std` → **4.0.2** |
| Write OpenOptions | `write`/`create_new` + **`follow(FollowSymlinks::No)`** |
| Unix | `O_NOFOLLOW` (+ `O_DIRECTORY` dirs) |
| Windows | `FILE_FLAG_OPEN_REPARSE_POINT` (+ `FILE_FLAG_BACKUP_SEMANTICS` dirs) |
| **Windows ban** | **No** `TRUNCATE_EXISTING` / `CREATE_ALWAYS` with OPEN_REPARSE_POINT (M1) |
| Replace strategy | **Preferred:** temp `create_new` + atomic rename (**O1**); alt: delete-regular-then-create_new |
| New crates | **Zero** preferred |

## Phases

### Phase A — Design freeze (plan-only ✅)

- [x] **A1** Site list with risk rank (spec §2.2) + concrete P2 paths (L2)
- [x] **A2** Write SOOT: ambient parent Dir once + leaf nofollow (F4–F8); `&Dir` API (M3)
- [x] **A3** Token **in scope** (override T190 F6) — F15
- [x] **A4** Soft-canon document-only residual — F16
- [x] **A5** P0 hard / P1 preferred / P2 inventory — F12–F14
- [x] **A6** ADR-0021 short amend preferred over ADR-0022 — F22
- [x] **A7** R-12 residual rewrite shape — F23 + AC8
- [x] **A8** Parent `create_dir_all` residual honesty (F26) — not P0 blocker
- [x] **A9** F9 rewrite: ban Windows truncate+reparse; CreateNew \| Replace only (M1/M2/L5)
- [x] **A10** Handle-based nlink (AI1); post-open is_file (M4); no maybe_dir (L4)
- [x] **A11** AC13/AC14 + F31–F35

### Phase B — Shared write SOOT (TDD)

- [x] **B0** Ledger start SECURITY
- [x] **B1 Red:** `cap_open` write tests — create_new leaf under ambient parent; refuse symlink leaf; open-fail has no ambient success path
- [x] **B1b Red (AC13/F35 mandatory):** replace/force with symlink leaf → refuse **and** symlink target bytes unchanged
- [x] **B1c Red (AC14):** hardlink leaf (nlink>1) refuse via handle metadata
- [x] **B2 Green:** `nofollow_write_options_create_new()` + `create_file_component_nofollow` / `write_file_nofollow_leaf(parent: &Dir, …)` + CreateMode::{CreateNew, Replace}
- [x] **B2b Green:** Replace = temp-rename (**preferred O1**) or delete-regular-then-create_new; **never** truncate-open on Windows with OPEN_REPARSE
- [x] **B3** Platform flags parity with read; post-open is_file; no maybe_dir
- [x] **B4** Optional soft: `open_ambient_dir` alias (F34); optional components write (F33) — alias done; components write not required
- [x] **B5** Targeted: `cargo nextest run -p ai-brains-path` + clippy package

### Phase C — P0 call sites (TDD)

- [x] **C1 Red→Green:** `write_protected_artifact` → SOOT write; keep hardlink refuse + ACL order; **caller** Windows-only (F27); shared helper **not** cfg-gated
- [x] **C2 Red→Green:** `token.rs` write + load nofollow handle read/write; ACL apply/verify unchanged
- [x] **C3 Red→Green:** `recovery::write_kit_file` SOOT; force → Replace mode (no `path.exists()`+truncate); Unix mode 0o600 preserved
- [x] **C4** Retarget existing reparse/hardlink tests; prove AC13 on kit/artifact if creatable
- [x] **C5** Targeted: `-p ai-brains-cli` / `-p ai-brains-api-server` nextest + clippy

### Phase D — P1 opportunistic + residual register

- [x] **D1** Evaluate migrate report/manifest, shadow dest, dogfood/evaluate report — elevate if free via helper
- [x] **D1b** Backup: note pure-ambient baseline; add parent reparse refuse if free (L3)
- [x] **D2** Fill residual register (spec §13) for anything not elevated — include concrete P2 file:line
- [x] **D3** P2 long-tail: inventory only (no mass rewrite)

### Phase E — Claims + closeout

- [x] **E1** Docs (spec §14): RELEASE-CLAIMS R-12; SECURITY-LIMITS §5; ADR-0021 residual table; *(deferred.md / conductor Completed = orchestrator)*
- [ ] **E2** `conductor.md` status → Completed on ship *(orchestrator)*
- [ ] **E3** Full gate: fmt / clippy -D / nextest workspace / deny / audit / ledgerful verify *(orchestrator final gate)*
- [ ] **E4** SECURITY review: internal → fix → cross-model until clean (or deferred P3 only)
- [ ] **E5** Manual smoke: protected artifact refuse; recovery export kit write; token ensure path
- [ ] **E6** Ledger commit; pin DECISION; no vault-root read regression note

## Verification matrix

| AC | Proof |
|----|-------|
| AC1 write SOOT | Unit tests in `cap_open` |
| AC2 artifact | Code + reparse refuse tests |
| AC3 token | Code + security tests |
| AC4 kit | recovery export refuse tests |
| AC5 symlink refuse create | Soft-skip OK for create privilege |
| **AC13 replace ≠ truncate reparse** | **Mandatory** force/replace + symlink leaf; target bytes intact |
| **AC14 handle nlink** | Unit/integration hardlink refuse |
| AC6 no ambient fallback | Code review + open-fail test |
| AC7 T190 regression | vault_fs / walk / hermes-honcho green |
| AC8 claims | Doc diff |
| AC9 soft-canon | Doc non-claim retained |
| AC10 deps | deny/audit; hold 4.0.2 |
| AC11 residual register | Spec §13 filled + P2 paths |
| AC12 gate + review | Process |

## Out of scope checklist

- [ ] Soft-canon as TOCTOU close
- [ ] WASI plugin host
- [ ] All ambient CLI adapter rewrites
- [ ] T195 multi-user pipe/UDS
- [ ] T196 systemd/launchd
- [ ] cap-std major bump
- [ ] Claiming product-wide path TOCTOU closed
- [ ] Mass rename `open_ambient_vault_dir` call sites (alias only)

## Implement notes (for go-ahead)

1. **TDD order:** path helpers (incl. AC13/AC14) → artifact → token → kit → P1 → docs.
2. **High findings if:** silent ambient fallback; missing `FollowSymlinks::No`; Windows TRUNCATE+OPEN_REPARSE replace; P0 docs-only; vault read regression; shared helpers Windows-gated.
3. **Stop-before:** destructive git; broad unrelated failures; inventing second permanent path stack without migration.
4. **Suggested residual order after T193 ship:** T195 → T196.
