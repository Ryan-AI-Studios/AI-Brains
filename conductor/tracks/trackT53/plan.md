# Plan: Track T53 - Daemon Lifecycle & Global Install UX

- [x] **Phase 1: Daemon API & Shutdown Handler**
    - [x] Add `Shutdown` variant to `DaemonRequest` in `ai-brains-daemon-api`.
    - [x] Implement shutdown logic in `ai-brainsd/src/main.rs` to break the loop and exit.

- [x] **Phase 2: CLI Command Implementation**
    - [x] Add `daemon` subcommand to CLI with `stop` nested command.
    - [x] Implement IPC call to send the `Shutdown` request.
    - [x] Add `--force` flag logic (using `taskkill` as fallback if signal fails).

- [x] **Phase 3: Verification**
    - [x] Verify `ai-brains daemon stop` correctly terminates the process.
    - [x] Verify `cargo install` works immediately after stop.
