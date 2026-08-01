# T179 Plan — Multi-Platform Compatibility Matrix (P12.1)

Status: **In Progress** (Phase A/C/D + partial B)  
Spec: [spec.md](./spec.md) (F1–F32, AC1–AC13).

## Preconditions

- [x] Read `Docs/PRD.md` primary/secondary platform language  
- [x] Read `Docs/Deviations.md` §1 (SQLCipher → bundled)  
- [x] Read T174 residual multi-OS note  
- [x] Read live `daemon_client.rs` (pipe vs UDS) + `private_blob.rs` (DPAPI vs DataKey)  
- [ ] Prefer docs-first before CI; `ledgerful doctor` + `ledgerful scan --impact` before code edits  

## License gate

- [x] No AGPL CI tooling  
- [x] Document any container base images if introduced (none for T1)  
- [x] GHA: `actions/checkout` + `dtolnay/rust-toolchain` (or setup-rust-toolchain); **SHA-pin on release** (F26) — PR uses `@v1` / `@v4` floating majors  
- [x] Zero new production Cargo dependencies (F9)  
- [x] Do **not** use archived `actions-rs/*`  

---

## Phase A — Inventory + matrix doc (docs-first)

- [x] **A1** **Grep-complete** inventory → `evidence/CFG-INVENTORY.md`  
  - Workspace-wide `cfg(windows)` / `cfg(not(windows))` / `target.'cfg(windows)'`  
  - §2.1 is **illustrative only** (F28)  
  - Include: nightly/schtasks, device DPAPI, git askpass, vault_fs, private_blob, desktop webview2, path reparse  
  - Subsection: **6 `windows` crate consumers** (api-server, path, crypto, ai-brainsd, cli, desktop)  
- [x] **A2** Draft `Docs/COMPATIBILITY.md` from spec §5  
  - F8 **exact** vault encryption wording  
  - F23 transport matrix (pipe / UDS / HTTP)  
  - F29 device seed DPAPI non-portability  
  - F32 `/bin/true` askpass note  
  - Desktop engines: WebView2 vs WKWebView vs WebKitGTK; Isolation Windows-only  
  - WSL column = Linux binary + `/mnt/c` (F4)  
- [x] **A3** Cross-link from `Docs/OPERATIONS.md` and/or `Docs/ci-tooling.md`  
- [x] **A4** Align PRD one-liner if needed (no silent “fully supported” without evidence)  
- [x] **A5** T174 multi-OS / WDIO residual disposition (F21) in desktop section  

## Phase B — Unix compile hygiene + honesty proofs

- [ ] **B0** First Linux dry-run **before** committing CI workflow if possible (WSL or cross `cargo check --target x86_64-unknown-linux-gnu`); else first GHA Linux job is the dry-run — record in `evidence/UNIX-BUILD.md` (AI2 F)  
- [ ] **B1** `cargo check --workspace` on Linux; log first errors  
- [ ] **B2** Minimal cfg/stub fixes — fail-closed, **no** fake success for DPAPI/service/pipe  
- [ ] **B3** Expand `rust-toolchain.toml` `targets` only when multi-OS CI needs them (F10)  
- [x] **B4** Verify `private_blob` `#[cfg(not(windows))]` compiles; opening `PROTECTION_DATAKEY_DPAPI` on Unix returns clear error (F29) — unit `device_private_blob__open_dpapi_junk__fails_with_dpapi_message` (works on Win too)  
- [x] **B5** Verify Unix `DaemonClient` UDS path (`/tmp/ledgerful-bridge.sock`); test path construction; document HTTP as portable smoke (F23) — unit `daemon_client__new__uses_os_native_transport_path` + `transport_path()` / `DEFAULT_DAEMON_TRANSPORT_PATH`  
- [x] **B6** Capture independence still holds (no new capture→sync/models edges) — matrix invariant documented; no new edges introduced  
- [x] **B7** Optional: unit asserting Unix transport is UDS (not Windows pipe APIs) — covered by B5 cfg test  

## Phase C — CI workflows

- [x] **C1** Create `.github/workflows/ci.yml` with **pinned** matrix (F24):  
  - required: `windows-2025`  
  - required: `ubuntu-24.04` — steps: **cargo check --workspace** → clippy → nextest → deny → audit  
  - soft: `macos-15` **or** `macos-26` (document choice; never claim 15 if running latest=26) (F25) — **chose macos-15**  
  - **no macos-14**  
- [x] **C2** Install toolchain **1.95.0** + rustfmt + clippy; release jobs **SHA-pin** action (F26) — documented; PR uses floating major  
- [x] **C3** Install nextest/deny/audit at `Docs/ci-tooling.md` mins; **audit: exit code only** (F27); optional `--json`  
- [x] **C4** Shell: Windows PowerShell (`;` not `&&`); Linux/macOS `set -euo pipefail`  
- [x] **C5** Optional `Swatinem/rust-cache` (SHA-pin if release) — soft continue-on-error  
- [ ] **C6** Optional `workflow_dispatch` WSL smoke (F5) — residual  
- [x] **C7** Document GHA vs local `dev-check.ps1` in `Docs/ci-tooling.md`  
- [x] **C8** Add **`scripts/dev-check.sh`** POSIX mirror of gate (F30 / AC13)  
- [x] **C9** Soft optional: `ubuntu-24.04-arm` single `cargo check` **or** leave arm as T3 in COMPATIBILITY (F14) — **arm remains T3**  

## Phase D — Smoke evidence + handoffs

- [x] **D1** Smoke §6.3 Windows → `evidence/SMOKE-windows.md` (label `windows-2025`)  
- [x] **D2** Smoke Ubuntu → `evidence/SMOKE-linux.md` (label `ubuntu-24.04`)  
- [x] **D3** macOS: smoke with **matching** label **or** explicit T2 residual  
- [x] **D4** Optional WSL2: Linux binary + `/mnt/c` vault → `evidence/SMOKE-wsl.md`  
- [x] **D5** Handoff **T183**: install order Windows-first; UDS/HTTP; DPAPI seed; askpass `/bin/true`; Isolation engines  
- [x] **D6** Handoff **T185**: platform smoke checkbox; runner label must match COMPATIBILITY tier; F8 SQLCipher honesty  
- [x] **D7** Desktop: Windows T1; Linux/macOS T2; no WDIO release plugins  

## Phase E — Verification + closeout

- [ ] **E1** Full gate Windows:  
  `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit`  
- [ ] **E2** Linux CI green (or recorded equivalent)  
- [ ] **E3** `ledgerful verify` as applicable  
- [ ] **E4** Review log `review.md`  
- [ ] **E5** Conductor → **Completed**; deferred.md update  
- [ ] **E6** Pin: `ai-brains pin "DECISION: T179 platform tiers — Win T1; Ubuntu 24.04 T1 after CI; macOS soft pin 15/26; WSL path interop; UDS Unix live / HTTP portable; DPAPI seed non-portable; SQLCipher honesty F8"`  

---

## Test / evidence naming

| Artifact | Purpose |
|----------|---------|
| `evidence/CFG-INVENTORY.md` | Grep-complete Windows/Unix surfaces |
| `evidence/SMOKE-<os>.md` | Manual/CI smoke + runner label |
| `evidence/UNIX-BUILD.md` | First Linux breakages + fixes |
| `Docs/COMPATIBILITY.md` | Normative user-facing matrix |
| `scripts/dev-check.sh` | POSIX local gate |

Any Rust tests: `feature__condition__expected_result`; hermetic; no real network; no bare unwrap in production.

## Out of scope checklist (do not implement)

- [ ] SQLCipher feature flip as DoD  
- [ ] systemd/launchd production units as DoD  
- [ ] Desktop multi-OS Playwright/WDIO hard gate  
- [ ] arm64 T1  
- [ ] Force Unix CLI→HTTP-only migration as DoD  
- [ ] #34.2 DataKey rotation  
- [ ] T180 protocol goldens  
- [ ] Electron / prod CSP weaken  

## Phase F — GHA PR #51 red → green (expand T179; **not** a new track)

**Disposition:** Failures are **T179’s own CI gate** (first multi-OS workflow). Fix **on branch** `track/T179-compatibility-matrix` / PR #51. Do **not** open T186 or defer to T180/T185.

### Root causes (gh run 30681897520)

| Gate | Step | Cause |
|------|------|--------|
| `gate-windows` | `cargo fmt --check` | `smoke.rs` pin `.env(...)` single-line; rustfmt wants multi-arg form |
| `gate-linux` / `gate-macos` | `nextest` | `test_backup_restore_dry_run` sets `AI_BRAINS_PROJECT_ID` via `.env()` but **omits** `--no-project-context`. Without repo-root `.env`, CLI **removes** those vars (T80 stale-context clear) → pin fails |

**Incomplete prior fix (4496f59):** added `.env()` only — insufficient on clean GHA.

### Checklist

- [x] **F1** Document dual failure (fmt + hermetic pin) from `gh pr checks 51` / failed job logs  
- [x] **F2** Fix pin invocation: `.env(PROJECT)` + `.env(SESSION)` + **`--no-project-context`** (match smoke hermetic pattern)  
- [x] **F3** `cargo fmt` so Windows `fmt --check` green  
- [x] **F4** Targeted local: `cargo nextest run -p ai-brains-cli --test smoke test_backup_restore_dry_run` → **PASS** (2026-08-01)  
- [ ] **F5** Push to PR #51; confirm all three gates green via `gh pr checks 51`  
- [ ] **F6** Record GHA green in `evidence/SMOKE-*.md` / residual table  
- [ ] **F7** Codex re-review only if F5 green and track closeout  

### Policy freezes (from failure)

| ID | Rule |
|----|------|
| **F39** | Hermetic CLI tests that supply `AI_BRAINS_PROJECT_ID` / `SESSION_ID` **must** pass `--no-project-context` (or create a project `.env`); otherwise main clears them when no `.env` exists. |
| **F40** | Multi-OS CI must run `cargo fmt --check` on the same rustfmt version as local; multiline `env(` is required for long string literals under max_width 100. |

### Advice if T179 were abandoned instead

Would **not** recommend: failures block the matrix claim itself. A follow-on track would only re-do T179 closeout under a new number. Keep PR #51, fix here, then close T179.

---

## Residual log (fill during implement)

| Item | Severity | Owner |
|------|----------|-------|
| First GHA green on PR #51 (fmt + nextest all OS) | **High** | Phase F — in progress |
| First Linux `cargo check` dry-run / UNIX-BUILD fill | Medium | WSL evidence partial; GHA is source of truth |
| Expand `rust-toolchain.toml` targets for multi-OS | Low | When Linux CI needs host targets |
| Optional WSL workflow_dispatch smoke (C6) | Low | T183/T185 residual |
| macOS remains T2 until soft job green | Info | After F5 |
| arm64 T3 honesty | Info | Future soft job |
| F26 release SHA-pin | Low | T185 |
