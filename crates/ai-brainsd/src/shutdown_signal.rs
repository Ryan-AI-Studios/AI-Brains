//! Process-wide graceful shutdown signals for the interactive daemon (T196 F36).
//!
//! - Always waits for Ctrl-C (**SIGINT**).
//! - On Unix, also waits for **SIGTERM** (systemd / launchd default stop).
//! - Does **not** daemonize, double-fork, or call `setsid`.
//! - Windows SCM service path uses its own control handler — not this helper.

/// Wait until a graceful shutdown signal is received.
///
/// Errors from signal registration are logged and ignored where a fallback path
/// remains (Unix falls back to Ctrl-C only if SIGTERM cannot be installed).
pub async fn wait_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        match signal(SignalKind::terminate()) {
            Ok(mut term) => {
                tokio::select! {
                    result = tokio::signal::ctrl_c() => {
                        if let Err(e) = result {
                            eprintln!("warning: Ctrl-C handler error: {e}");
                        }
                    }
                    _ = term.recv() => {}
                }
            }
            Err(e) => {
                eprintln!(
                    "warning: could not install SIGTERM handler ({e}); waiting for Ctrl-C only"
                );
                if let Err(e) = tokio::signal::ctrl_c().await {
                    eprintln!("warning: Ctrl-C handler error: {e}");
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        if let Err(e) = tokio::signal::ctrl_c().await {
            eprintln!("warning: Ctrl-C handler error: {e}");
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)] // tokio::test expands expect; AGENTS test names
mod tests {
    use super::wait_shutdown_signal;

    /// Ensures the helper future is constructible and abortable without panic
    /// (signal registration path compiles on the host OS).
    #[tokio::test]
    async fn wait_shutdown_signal__spawn_and_abort__no_panic() {
        let handle = tokio::spawn(wait_shutdown_signal());
        handle.abort();
        match handle.await {
            Err(_join_err) => {}
            Ok(()) => {
                // Aborted tasks must not resolve as Ok; treat as test failure without expect().
                panic!("expected JoinError after abort, got Ok(())");
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn wait_shutdown_signal__unix_cfg__module_links() {
        // Compile-time presence of the unix branch (link smoke).
        let _f = wait_shutdown_signal;
    }
}
