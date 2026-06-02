# Track T84: Self-Healing / Auto-Restart Tooling

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P2 — usability/developer convenience.

---

## Problem Statement

When compiling or updating the global binaries (`ai-brains.exe` or `ai-brainsd.exe` in `~/.cargo/bin`), the installation step fails with a Windows permission error (`Access is denied (os error 5)`) if the background daemon process (`ai-brainsd.exe`) is actively running. The developer has to manually search for and kill the process group before they can update their local installation.

## Acceptance Criteria

**AC1:** The build/update pipeline (`scripts/Build-AIBrains.ps1`) automatically checks if the daemon is running, shuts it down gracefully (or kills it forcefully if unresponsive), replaces the binaries, and then restarts the daemon.

**AC2:** A new command `ai-brains update` (or a dedicated action in `ai-brains daemon`) is provided to execute a self-healing process: gracefully stopping the daemon, executing binary replacement/pulling, and starting the new daemon process seamlessly.

**AC3:** Detailed, actionable logging is emitted to stderr showing each phase of the shutdown, replacement, and startup processes.

## Design Notes

- The build script can use `Get-Process ai-brainsd -ErrorAction SilentlyContinue` and `Stop-Process` or taskkill.
- The CLI command `ai-brains update` should execute the cargo installation command or copy the new executable from the target directory, stopping and restarting the daemon as part of the transaction.
- Standardize process verification to ensure the new daemon starts up correctly after the replacement.

## Verification

- Run `Build-AIBrains.ps1` while the daemon is running; verify it restarts the daemon and completes without `Access is denied` errors.
- Run `ai-brains update` (or equivalent update subcommand) and verify it stops, updates, and restarts the daemon.
