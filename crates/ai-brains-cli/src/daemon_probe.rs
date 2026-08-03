//! Shared daemon IPC reachability probe (T199).
//!
//! Single source of truth for interactive **Status** (liveness) and **Safety**
//! (doctor / restore / recovery / vault rotate gates). Policies differ only in
//! attempt count and per-attempt timeout — Ping→Pong truth is identical.

use crate::daemon_client::DaemonClient;
use std::time::Duration;

/// Probe attempt/timeout policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonProbePolicy {
    /// Interactive `daemon status`: single-shot, low latency.
    Status,
    /// Destructive / health-audit gates: multi-attempt robust (T188).
    Safety,
}

impl DaemonProbePolicy {
    /// Number of Ping attempts.
    pub const fn attempts(self) -> u32 {
        match self {
            Self::Status => 1,
            Self::Safety => 3,
        }
    }

    /// Per-attempt timeout for [`DaemonClient::probe`].
    pub const fn per_attempt(self) -> Duration {
        match self {
            Self::Status => Duration::from_millis(300),
            Self::Safety => Duration::from_millis(1000),
        }
    }
}

const SAFETY_BACKOFF: Duration = Duration::from_millis(50);

/// Probe whether the daemon answers Ping→Pong under the given policy.
///
/// Safety inserts a short backoff between failed attempts. Status is a
/// single shot (no backoff path).
pub async fn probe_daemon_reachable(client: &DaemonClient, policy: DaemonProbePolicy) -> bool {
    let attempts = policy.attempts();
    let per_attempt = policy.per_attempt();
    for attempt in 0..attempts {
        if client.probe(per_attempt).await {
            return true;
        }
        if attempt + 1 < attempts {
            tokio::time::sleep(SAFETY_BACKOFF).await;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    /// AC5: Status == 1 × 300ms (pub const).
    #[test]
    fn daemon_probe_policy__status__single_shot_300ms() {
        assert_eq!(DaemonProbePolicy::Status.attempts(), 1);
        assert_eq!(
            DaemonProbePolicy::Status.per_attempt(),
            Duration::from_millis(300)
        );
    }

    /// AC5: Safety ≥ 3 × ≥1000ms (pub const).
    #[test]
    fn daemon_probe_policy__safety__at_least_3x1000ms() {
        assert!(DaemonProbePolicy::Safety.attempts() >= 3);
        assert!(DaemonProbePolicy::Safety.per_attempt() >= Duration::from_millis(1000));
    }
}
