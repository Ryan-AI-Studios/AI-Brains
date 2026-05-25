# Plan: Track T52 - Nightly Resilience & Async Alignment

- [x] **Phase 1: Async Refactoring**
    - [x] Change `commands::nightly::run` signature to `pub async fn run`.
    - [x] Update `crates/ai-brains-cli/src/main.rs` to `.await` the nightly run.
    - [x] Replace `tokio::runtime::Runtime::new()` with direct calls to async functions.

- [x] **Phase 2: Lifecycle & Auto-Start**
    - [x] Call `DaemonClient::ensure_running` at the start of `nightly::run`.
    - [x] Verify that summarization captures are correctly spooled if the daemon was just started.

- [x] **Phase 3: Verification**
    - [x] Run `ai-brains nightly` manually and verify no panic occurs.
    - [x] Check `recall` to ensure new summaries are present.
