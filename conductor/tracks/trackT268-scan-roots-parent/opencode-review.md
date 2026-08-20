# opencode-review — T268 scan-roots parent (`--root`)

**Reviewer**: opencode (plan review — no implementation, no folding)
**Date**: 2026-08-19
**Scope**: `spec.md` (F0–F27 / AC1–AC15) + `plan.md` only. Harness file only; `review.md`, `spec.md`, `plan.md`, `conductor.md`, `deferred.md`, and `src/` are untouched.
**Live baseline**: HEAD `a4ac170` (merged PR #183, T271). Tree clean. Ledger 0 pending / 0 drift.

## Summary

T268 adds `project scan-roots --root` (explicit parent hint override) plus a parent-path hint on implicit-cwd scans, and folds in deferred items 4/5 (`scan-roots cwd-only`) and 4/5 (cwd-only scan + re-register suggestion). The plan is high-quality and unusually well-attested: every feature maps to a numbered AC, every AC maps to existing test helpers, and the absorbed-deferred table is explicit. No **B** or **M** findings. Four **m** items (all non-blocking, all one-line fixes or test additions) and one **O**. Verdict: **Planned**.

## Findings

| # | Sev | File / Loc | Finding |
|---|-----|-----------|---------|
| 1 | **m** | `plan.md` F2 (implicit-cwd hint) | Error handling for the new `git rev-parse --show-toplevel` spawn is unspecified. If the git binary is unavailable (`Err`), the whole implicit-cwd human scan would fail. Precedent: `crates/ai-brains-cli/src/commands/identity_warn.rs:100` and `doctor.rs:742` use `.unwrap_or_default()` for best-effort git. Real risk is low (git is on CI PATH; a non-repo dir returns status 1 → `Ok(default)`), but the plan should pin best-effort fallback to keep capture-independent paths resilient. |
| 2 | **m** | `plan.md` F21/F12 + `AC12` | The volume-root guard only names `C:\` / `/`. It does not address (a) drive-letter case-insensitivity on Windows (`c:\` vs `C:\`) and (b) UNC share roots (`\\server\share` — the `.parent()` of a UNC share root is itself, and `parent().parent()` walks into the server node). Recommend a `path_utils` normalization guard covering both, plus a test. |
| 3 | **m** | `plan.md` hint path form | `git rev-parse --show-toplevel` on Windows emits forward slashes (`C:/dev/AI-Brains`), so `toplevel.parent()` hint text may print `C:/dev` while the docs example shows `C:\dev`. Consider display normalization (e.g., reuse `ai_brains_path::normalize_for_location_compare`, which `scan_rows_for_hits` already uses at `project_paths.rs:268`) so hint output matches Windows-native separators. Cosmetic but user-visible. |
| 4 | **m** | `plan.md` F3 / `AC6` | The empty-`suggested` JSON shape needs an explicit note: when a root is registered (`suggested: ""`), the existing JSON test at `tests/project_path_aliases.rs:550` does `as_str().unwrap_or("")` then asserts `contains("register-path")` only in the unregistered case — so F3's `null` vs `""` choice must be pinned in the plan (recommend `""` to match `ScanRootRow`'s frozen string field and the `unwrap_or("")`-tolerant test). Also state that `scan_roots` dispatch keeps `root.or(path).as_deref()` so implicit-cwd detection via `path.is_none()` is unaffected. |
| 5 | **O** | `spec.md` F2 condition (b) | "zero unregistered hits" is vacuously true on zero total hits. A plain git repo with no `.ledgerful` anywhere therefore prints the parent hint — arguably a desirable discoverability side effect, but the spec should state it explicitly so the behavior is intentional, not emergent. |
| 6 | **note** | `plan.md` F10/F12 | `--root` is only meaningful for implicit-cwd scans (explicit `PATH` ignores it). The plan should confirm this is documented in the `--help` text (`ScanRoots` clap at `main.rs:2527`), otherwise `scan-roots --root X PATH` silently ignores the flag — a CLI footgun the XOR-peer pattern at `main.rs:2428–2434` (Resolve) already avoids. |

No **B** / **M** findings. Four **m** (non-blocking) + one **O** + one **note**.

## What looks solid

- **AC → helper mapping is concrete and verifiable.** Every AC cites the existing fixture to reuse. E.g.:
  - `AC6` (hermetic temp git repo + implicit cwd) has precedent at `tests/project_identity_convergence.rs:92` and `tests/project_detect_honesty.rs:84`.
  - `AC5` JSON keys match the frozen `ScanRootsJson` (`api_version, scan_root, truncated, roots`) at `project_paths.rs:182–188`; the live JSON test `scan_roots__format_json__api_version_1` (`tests/project_path_aliases.rs:468`) only asserts top-level keys, so adding `parent_hint` is non-breaking.
  - `AC10–AC12` dry-run tests exist at `tests/project_path_aliases.rs:507,563,602`.
- **F3 does not break the existing already-registered test.** `scan_roots__already_registered__shows_project_id` (`tests/project_path_aliases.rs:638`) asserts only `registered_project_id`, never `suggested` — so empty-suggested in the registered case is safe without a fixture change.
- **Capture independence preserved.** The `--root` hint path is purely a human/JSON display string computed from the cwd parent; it touches no model, embedding, or graph path. F0 (scan-roots CLI + event log) stays dependency-free.
- **Deferred absorption is explicit and correct** (see table below): deferred.md:233 `scan-roots cwd-only (4/5)` → T268; deferred.md:494 cwd-only scan + re-register suggestion → **F1–F3 / AC1–AC7**.
- **Clap field addition is pattern-safe.** `ScanRoots { path: Option<String>, format }` at `main.rs:2527–2533`; adding `root: Option<String>` keeps the existing unit test `scan_roots__format_pretty__parses` (`main.rs:446` uses `format, ..`) green.

## Deferred fold-in table

| Row | Deferred.md entry | Action | Notes |
|-----|-------------------|--------|-------|
| :233 | `scan-roots cwd-only (4/5)` | **T268 Planned** | Confirmed current in conductor.md:215. |
| :494 | "Audit cwd-only scan (4/5) + re-register suggestion" | **Absorb F1–F3 / AC1–AC7** | Matches plan's absorbed-deferred table. |
| :495–503 | Affirm T254 positional / F21 cwd default / F5 F20–F23 | Affirm | Keep deferred; consistent with plan Preflight table. |
| :503 | F12 closeout | Decline | Not in T268 scope; plan defers F12's volume-root guard (see finding #2). |
| :503 | Leftover `7d97a456` (T259 rebind-path) | Leave deferred | Out of scope; plan does not claim it. |
| :503 | T266 / T269 / T270 / T272 | Leave pending | Pending track queue unchanged. |
| :503 | T240 F2 / T255 | Leave deferred | Not claimed. |
| :503 | clap 5 / pins | Leave deferred | Pins verified: `Cargo.lock` clap 4.6.1, crates.io max 4.6.6; no clap 5. Not load-bearing for T268. |
| **new** | Dash-query ledger parse bug (PR #183) | **Mint T273** | See Last-PR Cursor comments below. Still true at `sync_query_ledger.rs:157`. |

## Last-PR Cursor comments

- PR **#183** (merged 2026-08-19, T271) Bugbot review flagged **1 potential issue** on commit `6aba57e44efd4450ad6d69783ad6e32803b35aff`: a **Medium** finding at `crates/ai-brains-cli/src/commands/sync_query_ledger.rs:160` — dash-prefixed queries (e.g. `sync-query-ledger search "--json"`) are parsed as ledgerful flags instead of the raw query. Live code confirms: `sync_query_ledger.rs:157` runs `cmd.args(["ledger", "search", "--json", query])` with the raw query after `--json`.
- **Resolution for fold-in**: this is NOT T268-scope. It is a genuine Medium (no current track claims it). Action: **mint placeholder `conductor/conductor.md:220` T273** (dash-query / `sync-query-ledger` raw-query escaping) during fold-in, and add a row to deferred.md. This matches the standing fold-in rule "mint a placeholder if a leftover fits no current track."

## Research / tools notes

- **Pins** (verified against `Cargo.lock` + crates.io): `clap` = **4.6.1** (workspace pin `4.5` in `Cargo.toml`), crates.io newest = **4.6.6**, **no clap 5** exists. `serde_json` = **1.0.150**. `rust-toolchain.toml` channel = **1.95.0**; workspace `version = "0.1.1"`, edition **2024**. `nextest 0.9.140` is a cargo subcommand, not in the lock, not load-bearing.
- **Live source** opened (not assumed): `project_paths.rs` (ScanRootsJson 182–188, ScanRootRow 190–196, `scan_roots` 199, `discover_scan_hits` 32–64, `scan_rows_for_hits` 264–283, `emit_scan_json` 285, `emit_scan_human` 299–316, unit-test mod 442); `project.rs` (`collect_git_identity` 165–208, `git_command` 1019–1023); `main.rs` (`ScanRoots` clap 2527–2533, dispatch 4595–4597, Resolve XOR peer 2428–2434, unit test 446); `tests/project_path_aliases.rs` (hermetic helpers 15–56, scan-roots tests 430/468/507/563/602/638/841/876); `tests/common/mod.rs` (`hermetic_bin`, `isolate_empty_home`, `AMBIENT_DENYLIST`).
- **Tools**: `ai-brains preflight --summary` OK (project `C:\dev\ai-brains`, 3 active sessions, 0 hotspots, 5 harnesses ok, discovery grants empty 0 of 3). `ledgerful doctor` ready (git found, gemini NOT FOUND — noted). `ledgerful ledger status --compact` → 0 pending, 0 unaudited drift. `rg` NOT available on this machine — greps done via `Select-String`. `ai-brains recall` (semantic) returned the T266 session context only; no conflicting decision on scan-roots found.
- **Contracts check**: `ai-brains-contracts/src/bridge.rs` has no scan-roots DTO (`suggested` only appears as `suggested_remediation` on a coupling finding struct, line 58). The CLI `ScanRootRow` type is local. Contract-surface update per AGENTS.md is **not** triggered by T268.

## Verdict

**Planned** — proceed to implementation on `/go`. The four **m** findings are one-line plan clarifications (best-effort git error handling, volume-root guard normalization incl. UNC, hint path display normalization, JSON `""` vs `null` pin + `--root`-ignored-by-explicit-PATH help text). No **B**/**M**; no re-plan required. Fold-in should (a) apply agreed findings to `plan.md`, (b) mint **T273** for the PR #183 dash-query Medium.
