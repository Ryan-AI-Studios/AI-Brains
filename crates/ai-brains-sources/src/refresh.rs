//! Bounded multi-connector refresh helper (T155 / P6.3).
//!
//! Aggregates `list`/`observe` results under a wall-clock deadline and max
//! observe budget. Always harvests connector side-channels
//! ([`ListSideChannels`]) into [`RefreshReport::failures`] when provided so
//! soft-empty unavailability is not silent (anti-#22).
//!
//! # Placement
//!
//! Lives in `ai-brains-sources` — **no** control-plane dependency. Briefing
//! callers can map [`RefreshFailure`] reason strings into T152 freshness /
//! `LedgerfulSectionDto` degraded paths later.
//!
//! # Direct callers
//!
//! `refresh_bounded` is the easy correct path. Callers that invoke
//! `Connector::list` directly must still check connector-local
//! `last_unavailable_reason` / `last_list_truncated` getters (contract-tested
//! on Git and Ledgerful connectors).

use std::time::{Duration, Instant};

use ai_brains_core::source::SourceKind;

use crate::connector::{Connector, ConnectorContext, ConnectorError, ObservePayload};
use crate::fingerprint_bytes;
use crate::fingerprint_ledgerful;

/// Default wall deadline for multi-connector refresh (10_000 ms).
pub const DEFAULT_REFRESH_DEADLINE_MS: u64 = 10_000;

/// Default wall deadline as a [`Duration`].
pub const DEFAULT_REFRESH_DEADLINE: Duration = Duration::from_millis(DEFAULT_REFRESH_DEADLINE_MS);

/// Side-channel getters for list truncation / unavailability (port frozen).
///
/// Implemented by connectors that expose `last_list_truncated` /
/// `last_unavailable_reason` (Git, Ledgerful; Obsidian exposes truncation).
pub trait ListSideChannels: Send + Sync {
    fn last_list_truncated(&self) -> bool;
    fn last_unavailable_reason(&self) -> Option<String>;
}

/// One connector target for [`refresh_bounded`].
pub struct RefreshTarget<'a> {
    pub connector_id: &'a str,
    pub connector: &'a dyn Connector,
    /// Optional side-channel harvest after `list` (and after observe failures).
    pub side_channels: Option<&'a dyn ListSideChannels>,
}

/// Successfully observed item (lightweight; no claim bodies).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedItem {
    pub connector_id: String,
    pub identity: String,
    pub fingerprint: String,
}

/// Per-connector failure or soft-unavailability signal (no claim bodies).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshFailure {
    pub connector_id: String,
    /// Reason code / string (e.g. `not_a_repository`, `timeout:…`, `list_truncated`).
    pub reason: String,
}

/// Aggregate result of a bounded refresh pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshReport {
    pub observed: Vec<ObservedItem>,
    pub failures: Vec<RefreshFailure>,
    /// True if max_observes budget or per-connector list truncation applied.
    pub truncated: bool,
    pub elapsed: Duration,
}

/// List + observe connectors under a wall deadline and observe budget.
///
/// - Partial results are returned when the deadline expires mid-pass.
/// - Side-channels (`last_unavailable_reason`, `last_list_truncated`) are always
///   folded into `failures` when `side_channels` is provided.
/// - Observe fingerprints use kind-aware algorithms (git content hash /
///   ledgerful authoritative hash / generic content hash).
pub fn refresh_bounded(
    targets: &[RefreshTarget<'_>],
    ctx: &ConnectorContext,
    deadline: Duration,
    max_observes: usize,
) -> RefreshReport {
    let start = Instant::now();
    let mut observed = Vec::new();
    let mut failures = Vec::new();
    let mut truncated = false;
    let mut observes_left = max_observes;

    for target in targets {
        if start.elapsed() >= deadline {
            truncated = true;
            failures.push(RefreshFailure {
                connector_id: target.connector_id.to_string(),
                reason: "deadline_exceeded".into(),
            });
            // Remaining targets also missed — report each as deadline skipped.
            // We already recorded this target; continue marking the rest.
            continue_marking_deadline(targets, target.connector_id, &mut failures, &mut truncated);
            break;
        }

        if observes_left == 0 {
            truncated = true;
            failures.push(RefreshFailure {
                connector_id: target.connector_id.to_string(),
                reason: "max_observes_exhausted".into(),
            });
            continue;
        }

        let handles = match target.connector.list(ctx) {
            Ok(h) => h,
            Err(e) => {
                failures.push(RefreshFailure {
                    connector_id: target.connector_id.to_string(),
                    reason: format_connector_error(&e),
                });
                harvest_side_channels(target, &mut failures, &mut truncated);
                continue;
            }
        };

        // Harvest soft-empty / truncation after every list.
        harvest_side_channels(target, &mut failures, &mut truncated);

        for handle in handles {
            if start.elapsed() >= deadline {
                truncated = true;
                failures.push(RefreshFailure {
                    connector_id: target.connector_id.to_string(),
                    reason: "deadline_exceeded".into(),
                });
                break;
            }
            if observes_left == 0 {
                truncated = true;
                failures.push(RefreshFailure {
                    connector_id: target.connector_id.to_string(),
                    reason: "max_observes_exhausted".into(),
                });
                break;
            }

            match target.connector.observe(ctx, &handle) {
                Ok(payload) => {
                    let fingerprint = fingerprint_observe(&payload);
                    observed.push(ObservedItem {
                        connector_id: target.connector_id.to_string(),
                        identity: payload.identity,
                        fingerprint,
                    });
                    observes_left = observes_left.saturating_sub(1);
                }
                Err(e) => {
                    failures.push(RefreshFailure {
                        connector_id: target.connector_id.to_string(),
                        reason: format_connector_error(&e),
                    });
                    harvest_side_channels(target, &mut failures, &mut truncated);
                }
            }
        }
    }

    RefreshReport {
        observed,
        failures,
        truncated,
        elapsed: start.elapsed(),
    }
}

fn continue_marking_deadline(
    targets: &[RefreshTarget<'_>],
    first_id: &str,
    failures: &mut Vec<RefreshFailure>,
    truncated: &mut bool,
) {
    *truncated = true;
    let mut past = false;
    for t in targets {
        if t.connector_id == first_id {
            past = true;
            continue;
        }
        if past {
            failures.push(RefreshFailure {
                connector_id: t.connector_id.to_string(),
                reason: "deadline_exceeded".into(),
            });
        }
    }
}

fn harvest_side_channels(
    target: &RefreshTarget<'_>,
    failures: &mut Vec<RefreshFailure>,
    truncated: &mut bool,
) {
    let Some(sc) = target.side_channels else {
        return;
    };
    if sc.last_list_truncated() {
        *truncated = true;
        push_unique_failure(failures, target.connector_id, "list_truncated");
    }
    if let Some(reason) = sc.last_unavailable_reason()
        && !reason.is_empty()
    {
        push_unique_failure(failures, target.connector_id, &reason);
    }
}

fn push_unique_failure(failures: &mut Vec<RefreshFailure>, connector_id: &str, reason: &str) {
    let exists = failures
        .iter()
        .any(|f| f.connector_id == connector_id && f.reason == reason);
    if !exists {
        failures.push(RefreshFailure {
            connector_id: connector_id.to_string(),
            reason: reason.to_string(),
        });
    }
}

fn format_connector_error(err: &ConnectorError) -> String {
    match err {
        ConnectorError::OperationNotSupported { operation } => {
            format!("operation_not_supported:{operation}")
        }
        ConnectorError::HandleNotFound { locator } => format!("handle_not_found:{locator}"),
        ConnectorError::UndeclaredSourceKind { kind } => {
            format!("undeclared_source_kind:{kind}")
        }
        ConnectorError::Internal { detail } => detail.clone(),
    }
}

fn fingerprint_observe(payload: &ObservePayload) -> String {
    match payload.handle.kind {
        SourceKind::GitRepository => fingerprint_bytes(&payload.content),
        SourceKind::Ledgerful => fingerprint_ledgerful(&payload.identity, &payload.content)
            .unwrap_or_else(|_| fingerprint_bytes(&payload.content)),
        _ => fingerprint_bytes(&payload.content),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod unit_tests {
    use super::*;
    use crate::manifest::ConnectorTrustLabel;
    use crate::mock::MockConnector;
    use ai_brains_core::ids::UserId;
    use ai_brains_core::privacy::Privacy;
    use ai_brains_core::scope::ScopeRef;
    use uuid::Uuid;

    fn personal_ctx() -> ConnectorContext {
        ConnectorContext {
            principal_id: None,
            scope: ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(1))),
            privacy: Privacy::LocalOnly,
            trust: ConnectorTrustLabel::LocalOnly,
        }
    }

    #[test]
    fn refresh_bounded__success__fingerprints_present() {
        let mock = MockConnector::new();
        let targets = [RefreshTarget {
            connector_id: "builtin.mock",
            connector: &mock,
            side_channels: None,
        }];
        let report = refresh_bounded(&targets, &personal_ctx(), Duration::from_secs(5), 10);
        assert!(
            !report.observed.is_empty(),
            "expected at least one observation"
        );
        for item in &report.observed {
            assert!(!item.fingerprint.is_empty());
            assert!(!item.identity.is_empty());
            assert_eq!(item.connector_id, "builtin.mock");
        }
        assert!(!report.truncated);
    }
}
