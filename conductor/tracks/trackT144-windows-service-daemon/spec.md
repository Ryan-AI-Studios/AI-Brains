# T144 — Windows Service Daemon + Cross-Session Pipe Security

## Problem

The AI-Brains daemon (`ai-brainsd`) runs via a Windows Scheduled Task (`AI-Brains-Daemon`)
with `LogonType: Interactive` / `RunLevel: Limited`. On some systems, Task Scheduler
launches it in **Session 0** (the services session) despite the `Interactive` logon type.
Session 0 created the named pipe `\\.\pipe\aibrains-sync` with a default security
descriptor that does NOT grant Session 1 (the user's interactive console) access.

Observed symptoms:
1. CLI clients in Session 1 cannot connect — `Access to the path is denied`.
2. A second `ai-brainsd` instance (started manually in Session 1) spins forever in a
   retry loop printing `Failed to create named pipe instance: Access is denied (os error 5)`
   because it can't create a subsequent pipe instance.
3. The Session 0 daemon cannot be cleanly shutdown from Session 1 (IPC is blocked too).

## Root Causes (in code)

- `crates/ai-brainsd/src/main.rs:64-103` — no single-instance guard; infinite retry loop
  with undifferentiated error messages (access-denied = fatal vs pipe-busy = transient).
- `crates/ai-brainsd/src/main.rs:66-68` — `ServerOptions` created with default security
  descriptor; no `SECURITY_ATTRIBUTES` granting the target user cross-session access.
- `crates/ai-brains-cli/src/daemon_client.rs:105` — `ClientOptions::new().open()` does
  not request any access scope; cross-session connect fails silently.
- `crates/ai-brains-cli/src/commands/daemon.rs` — `schedule`/`unschedule` use `schtasks`
  ONLOGON trigger; no Windows service option.

## Definition of Done

### Fix 2 — Single-instance guard
- [ ] Before entering the pipe-create loop, `ai-brainsd` probes `\\.\pipe\aibrains-sync`
      as a client with a 2 s timeout. If a `Ping` → `Pong` succeeds, it logs
      "Daemon already running (PID discoverable via `ai-brains daemon status`), exiting"
      and exits with code 0 (not an error).
- [ ] Guard is testable via a pure helper that classifies probe result → `enum
      InstanceDecision { Proceed, AlreadyRunning, ProbeFailed }`.

### Fix 3 — Pipe security descriptor
- [ ] On Windows, the pipe server creates the pipe with a security descriptor that
      grants the current user (resolved from the process token) `GENERIC_READ |
      GENERIC_WRITE | FILE_CREATE_PIPE_INSTANCE` so that:
      a) CLI clients in a different session of the same user can connect.
      b) A legitimate second instance (same user, different session) can create a
         subsequent pipe instance if multi-instance is ever desired.
- [ ] The SD is built via `windows` crate (`InitializeSecurityDescriptor`,
      `SetSecurityDescriptorDacl`, `BuildSecurityDescriptor` or a self-relative SD
      constructed manually) — no `unsafe` beyond the minimum required by the Win32 API
      wrappers.
- [ ] The SD builder is a pure, testable function `build_pipe_security_attributes() ->
      Result<SecurityAttributes, io::Error>` that returns an owned, self-relative SD.

### Fix 4 — Error classification
- [ ] `classify_pipe_error(&io::Error) -> PipeErrorKind` where `PipeErrorKind` is
      `AccessDenied` (fatal), `PipeBusy` (retry), `Other` (fatal).
- [ ] AccessDenied prints a single clear message: "Access denied creating pipe
      `<name>` — another instance owns it or the security descriptor denies access.
      Exiting." and exits with code 1.
- [ ] PipeBusy retries with the existing 1 s backoff.
- [ ] `Other` prints the error and exits with code 1.

### Fix 5 — Windows service
- [ ] `ai-brainsd` gains a `--service` flag that, when present, runs the daemon as a
      Windows Service (via the `windows-service` crate) instead of a console process.
      The existing console path (no `--service` flag) remains the default for
      `ai-brains daemon start` and development.
- [ ] Service name: `AI-Brains-Daemon`. Display name: `AI-Brains Daemon`. Description:
      "Local-first AI coding memory vault — captures conversation history without tool
      logs or hidden thinking." Startup type: Automatic (delayed start).
- [ ] Service runs as `LocalSystem` (Session 0). The pipe SD from Fix 3 grants the
      interactive user cross-session access.
- [ ] `ai-brains daemon install` — registers the Windows service (elevated). Writes
      env vars (vault path, model URLs) to
      `HKLM\SYSTEM\CurrentControlSet\Services\AI-Brains-Daemon\Environment` (or a
      sidecar `.env` in `%ProgramData%\AI-Brains\daemon.env`). Requires elevation.
- [ ] `ai-brains daemon uninstall` — removes the service (elevated).
- [ ] `ai-brains daemon start` — if the service is installed, starts it via SCM;
      otherwise falls back to the existing detached-process spawn.
- [ ] `ai-brains daemon stop` — if the service is installed, stops it via SCM;
      otherwise sends IPC shutdown, then force-kill.
- [ ] `ai-brains daemon schedule` / `unschedule` — **deprecated** (still work but print
      a deprecation hint pointing to `install`/`uninstall`). Kept for backward
      compatibility; no new schtasks path.

### Fix 6 — Cross-session client connect
- [ ] `DaemonClient::probe` and `DaemonClient::shutdown` use
      `ClientOptions::new().access(...)` (or the raw `CreateFile` with
      `GENERIC_READ | GENERIC_WRITE`) so cross-session connect succeeds when the pipe
      SD grants the user access.
- [ ] No behavior change on non-Windows.

## Testing Strategy (TDD)

### Unit tests (pure, no IO)
- `classify_pipe_error__access_denied__returns_access_denied`
- `classify_pipe_error__pipe_busy__returns_pipe_busy`
- `classify_pipe_error__other__returns_other`
- `instance_decision__probe_pong__returns_already_running`
- `instance_decision__probe_failed__returns_probe_failed`
- `instance_decision__no_response__returns_proceed`
- `build_pipe_security_attributes__returns_valid_self_relative_sd` (assert non-null
  SD length > 0, DACL present)
- `render_service_install_command__includes_sc_create`
- `render_service_install_command__includes_env_sidecar_path`
- `render_service_uninstall_command__includes_sc_delete`

### Integration tests (Windows-only, gated on `cfg(windows)`)
- `single_instance_guard__existing_daemon__exits_cleanly` — start a daemon, start a
  second in-process, assert the second exits 0 without creating a pipe instance.
  (Use unique pipe name per test to avoid cross-test interference.)

### Manual tests (recorded in plan.md)
- `ai-brains daemon install` from elevated PowerShell → service appears in
  `services.msc`, starts, pipe is connectable from Session 1.
- `ai-brains daemon status` from Session 1 → "Status: Running", PID reported.
- `ai-brains recall "test" --limit 1` from Session 1 → connects to Session 0 daemon.
- `ai-brains daemon uninstall` from elevated PowerShell → service removed.

## Non-Goals

- Linux/macOS service management (systemd/launchd) — out of scope.
- Multi-user pipe access (granting `Everyone` or `Authenticated Users`) — security
  risk; only the current user is granted.
- Auto-elevation prompt in `daemon install` — user must run from an elevated shell
  (matches existing `schedule --run-as-system` behavior).
- Migrating the nightly task to a service — only the daemon.

## Affected Crates

- `ai-brainsd` — main.rs (pipe loop, single-instance guard, error classification,
  `--service` flag, service entry point), lib.rs (no change to DaemonWriter), new
  module `windows_service.rs` + `pipe_security.rs`.
- `ai-brains-cli` — `daemon_client.rs` (cross-session connect), `commands/daemon.rs`
  (`install`/`uninstall` subcommands, deprecate `schedule`).
- `ai-brains-scheduler` — new `ServiceScheduler` with `render_install_command` /
  `render_uninstall_command` (pure rendering, testable).
- `Cargo.toml` (workspace) — add `windows-service` crate, extend `windows` crate
  features (`Win32_Security_Authorization`, `Win32_System_Pipes`,
  `Win32_Storage_FileSystem`, `Win32_System_Threading`).

## Contracts

- New CLI subcommands: `daemon install`, `daemon uninstall`. Both require elevation;
  print a clear error if run non-elevated.
- `daemon schedule` / `unschedule` print a one-line deprecation hint but still work.
- No `ai-brainsd` API payload changes — IPC protocol unchanged.
- `daemon status` output unchanged (still "Status: Running/Stopped" + vault info + PID).