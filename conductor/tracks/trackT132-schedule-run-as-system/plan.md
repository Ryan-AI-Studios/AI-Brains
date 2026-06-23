# Track T132 Plan: `--run-as-system` Flag for Schedule Commands

## Tasks

- [x] Add `#[arg(long)] run_as_system: bool` to `Commands::Nightly` in `main.rs`
- [x] Add `#[arg(long)] run_as_system: bool` to `DaemonCommands::Schedule` in `main.rs`
- [x] Wire `run_as_system` through `run()` dispatch to `commands::nightly::run` and `commands::daemon::run_schedule`
- [x] Update `nightly.rs` schedule path to append `/ru SYSTEM` before `/f` when `run_as_system` is true
- [x] Add `build_schtasks_args` helper in `nightly.rs` for testable arg construction
- [x] Return clear elevation error when `run_as_system` and schtasks reports "Access is denied"
- [x] Update `daemon.rs` `schedule_inner` and `run_schedule` to accept and apply `run_as_system`
- [x] Add `render_daemon_schedule_command` helper in `daemon.rs` to append `/ru SYSTEM` for dry-run output
- [x] Add unit tests in `nightly.rs`:
  - [x] `nightly_schedule__run_as_system__adds_ru_system`
  - [x] `nightly_schedule__no_run_as_system__omits_ru_system`
  - [x] `nightly_schedule__run_as_system_not_elevated__clear_error`
- [x] Add unit tests in `daemon.rs`:
  - [x] `schedule_inner__run_as_system__adds_ru_system`
  - [x] `schedule_inner__no_run_as_system__omits_ru_system`
- [x] Run `cargo fmt`
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [x] Run `cargo nextest run -p ai-brains-cli`
- [x] Run `cargo nextest run --workspace`
- [x] Update `conductor/conductor.md` status to Complete
- [x] Create this `plan.md`
- [x] Close ledger transaction

## Notes

- Kept nightly dry-run behavior unchanged; the `Nightly` command does not expose a `--dry-run` flag (AC4 is conditional on it existing).
- Added `#[allow(clippy::too_many_arguments)]` to `nightly::run` because adding `run_as_system` pushed it over the default 7-argument limit; the existing CLI surface already carries many bool/options and a refactor into a struct is out of scope for this track.
- No `.changeguard/` files edited directly.
- Pre-existing `cargo audit` finding (`quinn-proto` RUSTSEC-2026-0185) is unrelated to this track; no dependencies were added or changed.
