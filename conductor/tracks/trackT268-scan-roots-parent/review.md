# T268 review log — scan-roots parent / `--root`

**Track:** `conductor/tracks/trackT268-scan-roots-parent`
**Category:** UX (FEATURE TX)
**FEATURE TX:** `b1f31b8e-43e2-4e2e-a4cf-9825b45b859a`
**Date:** 2026-08-19

## Scope

`project scan-roots --root DIR` is a named XOR of the existing positional path.
Default stays **cwd**. Already-registered hits keep the owner and set JSON
`suggested` to `""` (human `—`). Implicit-cwd human with zero unregistered hits
may print `next: ai-brains project scan-roots --root <git-toplevel-parent>`.
Dry-run freeze. No events, `.env`, auto-register, leftover rebind, default
flip, JSON `next_step`, `project.rs` growth, clap 5, pin bumps, or T273 steal.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC17 / F0–F30 / §13 | PASS |
| R1b | Independent explore (read-only) | **PASS** (no P0–P3 product findings) |
| CX1 | Codex FEATURE `gpt-5.6-luna` high | **FAIL** — P1 Windows-only units (fixed); P1/P2 process (gate/bookkeeping) |
| CX2 | Codex FEATURE `gpt-5.6-luna` high | **PASS** (no P0–P3 product findings) |

## Findings

### R1 / R1b

No product findings. `unwrap_or_default()` on git spawn is **F22**, not a
Rust-safety violation.

Noted out of DoD (not findings): `Docs/WORKFLOWS.md` still says “copy suggested
`register-path`” without `--root`. F11 required CAPABILITIES / OPERATIONS /
CHANGELOG only.

### CX1 (Codex)

| ID | Sev | Disposition |
|----|-----|-------------|
| P1 Windows-only `Path` units fail on Linux CI | P1 | **fixed** — `cfg(windows)` / `cfg(not(windows))` native paths for AC12/AC17/F29 display |
| P1 full `dev-check` / `verify --scope full` pending | P1 | **process** — Phase 5 gate; not a product gap |
| P2 uncommitted / checkboxes / deferred row | P2 | **process** — Phase 5–6 finalize |

### CX2 (Codex)

**PASS.** Prior P1 Windows-path units verified fixed (`cfg` split). No new product findings.
Process items (full gate, bookkeeping) remain Phase 5–6.

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 clap `--root` XOR PATH ArgumentConflict exit 2 | met | `scan_roots__root_and_path__clap_argument_conflict` + reversed; live EXIT=2 |
| AC2 `--root` JSON roots == positional | met | `scan_roots__root_flag_matches_positional_json` |
| AC3 implicit cwd not parent | met | `scan_roots__implicit_cwd__scans_current_dir_not_parent` |
| AC4 registered `suggested == ""` / human `—` | met | unit + hermetic JSON/human |
| AC5 unregistered still `register-path` | met | existing `scan_roots__ledgerful_child_hits_plain_misses` + unit |
| AC6 implicit-cwd git registered → human `next:` / JSON no `next_step` | met | `scan_roots__implicit_cwd_registered_git__human_parent_hint` |
| AC7 explicit `--root` / positional → no `next:` | met | `scan_roots__explicit_root_on_git_repo__no_parent_hint` |
| AC8 JSON keys frozen | met | `scan_roots__json_envelope_keys_frozen` |
| AC9 never writes events | met | existing `scan_roots__never_writes_events` |
| AC10 T254/T266 hermetics + clap format units | met | changeguard / grandchild / marked root / pretty/JSON/Pretty/xml |
| AC11 empty `--root` / positional exit 2 same copy | met | `scan_roots__empty_root_flag__exit_2` + empty positional |
| AC12 `parent_scan_hint` volume/share matrix | met | units `/`, `C:\`, `c:\`, `C:`, UNC, `C:\dev\AI-Brains` → `C:\dev` |
| AC13 after_help + CAPABILITIES/OPERATIONS/CHANGELOG; no `project.rs` scan growth | met | `scan_roots__help__names_root_and_positional`; `project.rs` untouched |
| AC14 no production unwrap/expect/panic; no pin bumps; no DTO | met | clippy `-D warnings`; lock clap 4.6.1 |
| AC15 manual source bin | met | see Manual evidence |
| AC16 git fail-open `None` → no hint | met | `parent_scan_hint__toplevel_none__none` |
| AC17 vacuous zero hits still hints | met | `parent_scan_hint__zero_hits_vacuous__returns_parent` |
| F1/F20/F30 clap XOR + `root.or(path)` | met | clap units + dispatch |
| F2/F21/F22/F28/F29 hint helper | met | pure helper + human-only spawn + `\` display |
| F3 empty suggested | met | AC4 |
| F4–F10 / F12–F19 / F23–F27 / F9 pins | met | no events, bounds freeze, format freeze, no DTO, T273 stays Pending |
| §13 fold-in pins | met | F21/F22/F28/F29/F2-empty/F3 `""`/F10 no JSON hint |

## Targeted gates (observed)

- `cargo fmt --check` exit 0
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` exit 0
- `cargo nextest run -p ai-brains-cli --bins -E "test(project_paths) + test(scan_roots)"` **23 passed**
- `cargo nextest run -p ai-brains-cli -E "test(scan_roots)"` **25 passed**
- `ledgerful verify --scope fast` nextest step **timed out** (`timeout_secs` default vs 3193 workspace tests; not a product failure). fmt/clippy/deny/audit ok in that run.

## Manual evidence (AC15)

```text
# implicit cwd (C:\dev\AI-Brains), source bin, --format human
cargo run -q -p ai-brains-cli -- project scan-roots --format human
  path C:\dev\AI-Brains  registered_to 3581317d-…  disk ok  suggested —
  next: ai-brains project scan-roots --root C:\dev
  exit 0

# --root C:\dev --format human (siblings; do not register/rebind)
cargo run -q -p ai-brains-cli -- project scan-roots --root C:\dev --format human
  C:\dev unregistered suggested register-path
  C:\dev\AI-Brains 3581317d-… suggested —
  other registered siblings suggested —
  no next: line
  exit 0

# XOR
cargo run -q -p ai-brains-cli -- project scan-roots --root C:\dev C:\dev
  clap: the argument '--root <DIR>' cannot be used with '[PATH]'
  EXIT=2

# JSON implicit cwd
cargo run -q -p ai-brains-cli -- project scan-roots --format json
  keys: api_version, scan_root, truncated, roots
  suggested: ""
  no next_step
```

No live mutate (no register-path / rebind-path / `.env` write / `cargo install`).

## Full gate (observed)

- `ai-brains daemon stop` first (restore hermetics; unrelated `backup_restore__daemon_down_force` needs daemon down).
- `.\scripts\dev-check.ps1` **[SUCCESS]** nextest **3193** passed (1 skipped)
- `ledgerful verify --scope full` **passed** (fmt 2.5s / clippy 6.1s / nextest 115.2s / deny 4.1s / audit 2.7s)

## Residual / decline

- PATH `cargo install` — F16 operator
- T259 leftover `7d97a456` sibling owners — operator `rebind-path`
- JSON `next_step` — declined F10
- Default=parent — declined F15
- T269 / T270 / T272 / T273 — peers
- WORKFLOWS.md `--root` mention — F11 out of required docs set
