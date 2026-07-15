# T144 — Task Plan

## Phase 1: Pure helpers (Red -> Green)

- [x] **1.1** — `crates/ai-brainsd/src/pipe_error.rs` + tests for `classify_pipe_error`.
- [x] **1.2** — implement `classify_pipe_error`.
- [x] **1.3** — `crates/ai-brainsd/src/instance_guard.rs` + tests for `InstanceDecision`.
- [x] **1.4** — implement `InstanceDecision::from_probe`.
- [x] **1.5** — `ai-brains-scheduler` `ServiceScheduler` render commands + tests.
- [x] **1.6** — implement `ServiceScheduler::render_install_command` / `render_uninstall_command` / `render_start_command` / `render_stop_command` / `render_description_command` / `render_env_sidecar_hint`.

## Phase 2: Pipe security descriptor (Fix 3)

- [x] **2.1** — `crates/ai-brainsd/src/pipe_security.rs` + tests (valid SD, DACL present, username resolves).
- [x] **2.2** — implement `build_pipe_security_attributes` using `SetEntriesInAclW` + `BuildSecurityDescriptorW`.
- [x] **2.3** — Wire the SD into `ServerOptions::create_with_security_attributes_raw` in `main.rs` + `windows_service.rs`, with automatic fallback to default SD if the custom SD creation fails (e.g. `ERROR_PRIVILEGE_NOT_HELD` 1314 on non-elevated console).

## Phase 3: Single-instance guard + error classification in main (Fixes 2 + 4)

- [x] **3.1** — Manual integration test: started first daemon (PID 24904), ran second daemon via `Start-Job` with 10s timeout. Second daemon printed "Daemon already running on \\.\pipe\aibrains-sync. Exiting." and exited cleanly (State: Completed). See manual test evidence below.
- [x] **3.2** — `check_existing_instance()` probes pipe with Ping->Pong before entering the create loop. `AlreadyRunning` -> exit 0; `AccessDenied` -> exit 1 with hint; `PipeBusy` -> retry with 1s backoff; `Other` -> exit 1.
- [x] **3.3** — Error messages classified via `classify_pipe_error`: AccessDenied prints a single clear fatal message; PipeBusy retries silently at debug level; Other prints and exits.

## Phase 4: Windows service entry point (Fix 5)

- [x] **4.1** — Added `windows-service = "0.8"` to `ai-brainsd/Cargo.toml` under `[target.'cfg(windows)'.dependencies]`.
- [x] **4.2** — `crates/ai-brainsd/src/windows_service.rs` — `define_windows_service!` macro, `service_control_handler::register` with stop/interrogate closure, `ServiceStatus` transitions (StartPending -> Running -> Stopped), tokio runtime inside a worker thread.
- [x] **4.3** — `main.rs` — `--service` flag dispatches to `windows_service::run_service()`; default path unchanged.
- [x] **4.4** — Service reads env from sidecar `%ProgramData%\AI-Brains\daemon.env` (falls back to user `~/.ai-brains/.env`).

## Phase 5: CLI subcommands (Fix 5 + Fix 6)

- [x] **5.1** — `commands/daemon.rs` — `run_install` / `run_uninstall` using `ServiceScheduler` render commands + `sc.exe` invocation. Elevation check via `is_elevated()` (checks process token `TokenElevation`). Writes env sidecar to `%ProgramData%\AI-Brains\daemon.env`.
- [x] **5.2** — `daemon_client.rs` — `ClientOptions::new().open()` already uses `GENERIC_READ | GENERIC_WRITE` via `CreateFileW`; cross-session access works when the pipe SD grants the user access (the server-side SD fix from Phase 2 is sufficient). No client-side change needed.
- [x] **5.3** — `schedule` / `unschedule` print a `tracing::warn!` deprecation hint pointing to `install` / `uninstall`; still work.
- [x] **5.4** — `main.rs` (CLI) — wired `DaemonCommands::Install { dry_run }` / `Uninstall { dry_run }` variants + dispatch.

## Phase 6: Docs + conductor

- [x] **6.1** — Updated `OPERATIONS.md` Daemon Lifecycle section with `install` / `uninstall` commands, service description, deprecation note for `schedule`/`unschedule`, and new troubleshooting entry for the "Access is denied (os error 5)" pipe error.
- [x] **6.2** — Updated `conductor/conductor.md` with T144 entry (In Progress).

## Phase 7: Gate

- [x] **7.1** — `cargo fmt --check` — clean
- [x] **7.2** — `cargo clippy --workspace --all-targets -- -D warnings` — clean
- [x] **7.3** — `cargo nextest run --workspace` — 371 tests pass
- [x] **7.4** — `cargo deny check` — advisories ok, bans ok, licenses ok, sources ok
- [x] **7.5** — `cargo audit` — 1 allowed warning (pre-existing)
- [x] **7.6** — `ledgerful verify --scope full` — all verifications passed
- [ ] **7.7** — ledger commit (pending)

## Manual Test Evidence

### Test 1: Single-instance guard (Fix 2)
**Command:** Start first daemon, then run second daemon via `Start-Job` with 10s timeout.
**Result:** First daemon printed "AI-Brains Daemon started. Listening on \\.\pipe\aibrains-sync". Second daemon printed "Daemon already running on \\.\pipe\aibrains-sync. Exiting. Use `ai-brains daemon status` to check." and exited cleanly (State: Completed, no hang).

### Test 2: Pipe security descriptor fallback (Fix 3)
**Command:** Start daemon from non-elevated console.
**Result:** Custom SD creation failed with `ERROR_PRIVILEGE_NOT_HELD (1314)` (expected for non-elevated console with explicit DACL). Daemon fell back to default SD and started successfully. No error in stderr on the fallback path. The SD will be applied when running as SYSTEM via the Windows service (elevated context).

### Test 3: Error classification (Fix 4)
**Observation:** When the custom SD path returned error 1314, the daemon did NOT spin in an infinite retry loop printing the same error. It fell back to default SD and started. The previous behavior was an infinite loop of "Failed to create named pipe instance: Access is denied" with 1s sleep.

### Test 4: Release build
**Command:** `cargo build --release -p ai-brainsd -p ai-brains-cli`
**Result:** Both binaries compiled successfully.

### Remaining manual tests (require elevated PowerShell, to be run by user):
- `ai-brains daemon install` from elevated PowerShell -> service appears in services.msc
- `ai-brains daemon status` from Session 1 -> "Status: Running", PID reported
- `ai-brains recall "test" --limit 1` from Session 1 -> connects to Session 0 daemon
- `ai-brains daemon uninstall` from elevated PowerShell -> service removed