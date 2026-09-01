//! Wall-clock budget for nightly summarize + embed catch-up (T338).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Default `AI_BRAINS_NIGHTLY_DEADLINE_MINUTES` when missing or unparseable.
pub const DEFAULT_DEADLINE_MINUTES: u64 = 150;
/// Env key for the nightly wall-clock budget (minutes). `0` is an immediate deadline.
pub const DEADLINE_ENV: &str = "AI_BRAINS_NIGHTLY_DEADLINE_MINUTES";
/// Env key for embed catch-up page size. Invalid / empty / `0` → [`DEFAULT_EMBED_CHUNK`].
pub const EMBED_CHUNK_ENV: &str = "AI_BRAINS_EMBED_CHUNK";
/// Default embed keyset page size when env is unset or invalid.
pub const DEFAULT_EMBED_CHUNK: usize = 200;

enum DeadlineKind {
    AlreadyExpired,
    Until(Instant),
    /// Test clock: first `n` [`NightlyDeadline::expired`] calls return false, then true.
    ExpireAfter {
        remaining: AtomicUsize,
    },
}

/// Nightly loop clock. Production uses [`Self::from_minutes`]; tests use
/// [`Self::already_expired`] / [`Self::expire_after_checks`] (no sleep-for-async).
pub struct NightlyDeadline {
    kind: DeadlineKind,
}

impl NightlyDeadline {
    /// Production clock: `Instant::now() + minutes`. `0` is already expired.
    pub fn from_minutes(minutes: u64) -> Self {
        if minutes == 0 {
            return Self::already_expired();
        }
        let secs = minutes.saturating_mul(60);
        Self {
            kind: DeadlineKind::Until(Instant::now() + Duration::from_secs(secs)),
        }
    }

    /// Immediate deadline (AC1 / env `0`).
    pub fn already_expired() -> Self {
        Self {
            kind: DeadlineKind::AlreadyExpired,
        }
    }

    /// Test helper: first `n` [`Self::expired`] checks return false, then true.
    pub fn expire_after_checks(n: usize) -> Self {
        Self {
            kind: DeadlineKind::ExpireAfter {
                remaining: AtomicUsize::new(n),
            },
        }
    }

    /// Effective minutes from env (missing/empty/garbage/`-1` → 150; `0` stays 0).
    pub fn from_env() -> Self {
        Self::from_minutes(parse_deadline_minutes_from_env())
    }

    /// True when the summarize/embed loop must stop starting new work.
    pub fn expired(&self) -> bool {
        match &self.kind {
            DeadlineKind::AlreadyExpired => true,
            DeadlineKind::Until(deadline) => Instant::now() >= *deadline,
            DeadlineKind::ExpireAfter { remaining } => {
                let cur = remaining.load(Ordering::SeqCst);
                if cur == 0 {
                    true
                } else {
                    remaining.fetch_sub(1, Ordering::SeqCst);
                    false
                }
            }
        }
    }

    /// Seconds remaining for chunk-size / stale-refresh caps.
    ///
    /// `ExpireAfter` is a test clock: treat remaining wall time as plenty so tests
    /// control page size via `AI_BRAINS_EMBED_CHUNK`, not the 60s floor.
    pub fn remaining_secs(&self) -> u64 {
        match &self.kind {
            DeadlineKind::AlreadyExpired => 0,
            DeadlineKind::Until(deadline) => deadline
                .checked_duration_since(Instant::now())
                .map(|d| d.as_secs())
                .unwrap_or(0),
            DeadlineKind::ExpireAfter { remaining: _ } => 3600,
        }
    }
}

/// Parse `AI_BRAINS_NIGHTLY_DEADLINE_MINUTES`: `0` valid; garbage → 150.
pub fn parse_deadline_minutes(raw: Option<&str>) -> u64 {
    match raw {
        None => DEFAULT_DEADLINE_MINUTES,
        Some(s) => {
            let t = s.trim();
            if t.is_empty() {
                DEFAULT_DEADLINE_MINUTES
            } else {
                t.parse::<u64>().unwrap_or(DEFAULT_DEADLINE_MINUTES)
            }
        }
    }
}

/// Read [`DEADLINE_ENV`] from the process environment.
pub fn parse_deadline_minutes_from_env() -> u64 {
    parse_deadline_minutes(std::env::var(DEADLINE_ENV).ok().as_deref())
}

/// Parse `AI_BRAINS_EMBED_CHUNK`: invalid / empty / `0` → 200.
pub fn parse_embed_chunk(raw: Option<&str>) -> usize {
    match raw {
        None => DEFAULT_EMBED_CHUNK,
        Some(s) => {
            let t = s.trim();
            if t.is_empty() {
                DEFAULT_EMBED_CHUNK
            } else {
                match t.parse::<usize>() {
                    Ok(0) | Err(_) => DEFAULT_EMBED_CHUNK,
                    Ok(n) => n,
                }
            }
        }
    }
}

/// Read [`EMBED_CHUNK_ENV`] from the process environment.
pub fn parse_embed_chunk_from_env() -> usize {
    parse_embed_chunk(std::env::var(EMBED_CHUNK_ENV).ok().as_deref())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn nightly__deadline_unparseable__defaults_150() {
        assert_eq!(parse_deadline_minutes(None), 150);
        assert_eq!(parse_deadline_minutes(Some("")), 150);
        assert_eq!(parse_deadline_minutes(Some("abc")), 150);
        assert_eq!(parse_deadline_minutes(Some("-1")), 150);
        assert_eq!(parse_deadline_minutes(Some("  ")), 150);
        assert_eq!(parse_deadline_minutes(Some("0")), 0);
        assert_eq!(parse_deadline_minutes(Some("150")), 150);
        assert_eq!(parse_deadline_minutes(Some("30")), 30);
    }

    #[test]
    fn parse_embed_chunk__invalid_or_zero__defaults_200() {
        assert_eq!(parse_embed_chunk(None), 200);
        assert_eq!(parse_embed_chunk(Some("")), 200);
        assert_eq!(parse_embed_chunk(Some("0")), 200);
        assert_eq!(parse_embed_chunk(Some("abc")), 200);
        assert_eq!(parse_embed_chunk(Some("50")), 50);
        assert_eq!(parse_embed_chunk(Some("200")), 200);
    }

    #[test]
    fn expire_after_checks__first_n_false_then_true() {
        let d = NightlyDeadline::expire_after_checks(1);
        assert!(!d.expired());
        assert!(d.expired());
        assert!(d.expired());
    }

    #[test]
    fn already_expired__always_true() {
        let d = NightlyDeadline::already_expired();
        assert!(d.expired());
        assert_eq!(d.remaining_secs(), 0);
    }

    #[test]
    fn from_minutes_zero__already_expired() {
        let d = NightlyDeadline::from_minutes(0);
        assert!(d.expired());
    }
}
