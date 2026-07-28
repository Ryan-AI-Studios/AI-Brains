//! Bounded refresh helper tests (T155 Phase D).

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_core::ids::UserId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_sources::{
    Connector, ConnectorContext, ConnectorTrustLabel, GitConnector, GitConnectorOptions,
    LedgerfulConnector, LedgerfulConnectorOptions, ListSideChannels, MockConnector, RefreshTarget,
    refresh_bounded,
};
use tempfile::tempdir;
use uuid::Uuid;

fn personal_ctx() -> ConnectorContext {
    ConnectorContext {
        principal_id: None,
        scope: ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(99))),
        privacy: Privacy::LocalOnly,
        trust: ConnectorTrustLabel::LocalOnly,
    }
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    std::env::temp_dir().join(format!("ai-brains-sources-refresh-{name}-{nanos}"))
}

fn run_git(path: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(args).current_dir(path).output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}

fn init_repo_with_commit(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = unique_temp_dir(name);
    fs::create_dir_all(&root)?;
    run_git(&root, &["init"])?;
    run_git(&root, &["config", "user.name", "AI Brains Test"])?;
    run_git(&root, &["config", "user.email", "tests@example.com"])?;
    fs::write(root.join("README.md"), "hi\n")?;
    run_git(&root, &["add", "."])?;
    run_git(&root, &["commit", "-m", "initial"])?;
    Ok(root)
}

fn sample_record() -> BridgeRecord {
    let json = r#"{
        "bridge_version":"1.0",
        "direction":"inbound",
        "timestamp":"2026-05-19T00:00:00Z",
        "parent_hash":"deadbeef",
        "project_id":"proj",
        "session_id":null,
        "tx_id":"tx-1",
        "record_kind":"prompt",
        "payload":{"text":"hello"},
        "privacy":"ProjectLocal"
    }"#;
    serde_json::from_str(json).expect("record")
}

/// Connector that sleeps during list to force deadline partial results.
struct SlowListConnector {
    inner: MockConnector,
    delay: Duration,
}

impl Connector for SlowListConnector {
    fn manifest(&self) -> &ai_brains_sources::ConnectorManifest {
        self.inner.manifest()
    }

    fn list(
        &self,
        ctx: &ConnectorContext,
    ) -> Result<Vec<ai_brains_sources::SourceHandle>, ai_brains_sources::ConnectorError> {
        std::thread::sleep(self.delay);
        self.inner.list(ctx)
    }

    fn observe(
        &self,
        ctx: &ConnectorContext,
        handle: &ai_brains_sources::SourceHandle,
    ) -> Result<ai_brains_sources::ObservePayload, ai_brains_sources::ConnectorError> {
        self.inner.observe(ctx, handle)
    }

    fn preview(
        &self,
        ctx: &ConnectorContext,
        handle: &ai_brains_sources::SourceHandle,
    ) -> Result<ai_brains_sources::Preview, ai_brains_sources::ConnectorError> {
        self.inner.preview(ctx, handle)
    }

    fn propose_write(
        &self,
        ctx: &ConnectorContext,
        proposal: &ai_brains_sources::WriteProposalInput,
    ) -> Result<ai_brains_sources::WriteProposal, ai_brains_sources::ConnectorError> {
        self.inner.propose_write(ctx, proposal)
    }
}

#[test]
fn refresh_bounded__success__fingerprints_present() -> Result<(), Box<dyn std::error::Error>> {
    let root = init_repo_with_commit("refresh-ok")?;
    let git = GitConnector::open(&root, GitConnectorOptions::default())?;
    let ledger = LedgerfulConnector::from_records(
        vec![sample_record()],
        LedgerfulConnectorOptions::default(),
    );

    let targets = [
        RefreshTarget {
            connector_id: "builtin.git",
            connector: &git,
            side_channels: Some(&git as &dyn ListSideChannels),
        },
        RefreshTarget {
            connector_id: "builtin.ledgerful",
            connector: &ledger,
            side_channels: Some(&ledger as &dyn ListSideChannels),
        },
    ];

    let report = refresh_bounded(&targets, &personal_ctx(), Duration::from_secs(30), 16);
    assert!(
        report.observed.len() >= 2,
        "expected git+ledgerful observations, got {:?}",
        report.observed
    );
    for item in &report.observed {
        assert!(!item.fingerprint.is_empty());
        assert!(!item.identity.is_empty());
    }
    // No soft-unavailable on healthy targets.
    assert!(
        report
            .failures
            .iter()
            .all(|f| f.reason != "not_a_repository" && f.reason != "missing_store"),
        "failures={:?}",
        report.failures
    );

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn refresh_bounded__connector_unavailable__reported_not_silent() {
    let unavailable = LedgerfulConnector::unavailable("missing_store");
    let targets = [RefreshTarget {
        connector_id: "builtin.ledgerful",
        connector: &unavailable,
        side_channels: Some(&unavailable as &dyn ListSideChannels),
    }];
    let report = refresh_bounded(&targets, &personal_ctx(), Duration::from_secs(5), 10);
    assert!(report.observed.is_empty());
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.connector_id == "builtin.ledgerful" && f.reason == "missing_store"),
        "failures={:?}",
        report.failures
    );
}

#[test]
fn refresh_bounded__aggregates_last_unavailable_reason() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    // Non-repo git root → soft empty + reason.
    let git = GitConnector::open(dir.path(), GitConnectorOptions::default())?;
    let unavailable = LedgerfulConnector::unavailable("missing_store");

    let targets = [
        RefreshTarget {
            connector_id: "builtin.git",
            connector: &git,
            side_channels: Some(&git as &dyn ListSideChannels),
        },
        RefreshTarget {
            connector_id: "builtin.ledgerful",
            connector: &unavailable,
            side_channels: Some(&unavailable as &dyn ListSideChannels),
        },
    ];

    let report = refresh_bounded(&targets, &personal_ctx(), Duration::from_secs(10), 10);
    assert!(report.observed.is_empty());
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.connector_id == "builtin.git" && f.reason == "not_a_repository"),
        "git side-channel missing: {:?}",
        report.failures
    );
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.connector_id == "builtin.ledgerful" && f.reason == "missing_store"),
        "ledgerful side-channel missing: {:?}",
        report.failures
    );
    Ok(())
}

#[test]
fn refresh_bounded__deadline_exceeded__partial_results_and_failures() {
    // Slow connector first so wall time is spent inside a single blocking list;
    // subsequent targets must get deadline_exceeded without starting list.
    // (Deadline is inter-operation only — see refresh_bounded rustdoc.)
    let slow = SlowListConnector {
        inner: MockConnector::new(),
        delay: Duration::from_millis(50),
    };
    let never_started = MockConnector::new();

    let targets = [
        RefreshTarget {
            connector_id: "builtin.slow",
            connector: &slow,
            side_channels: None,
        },
        RefreshTarget {
            connector_id: "builtin.never_started",
            connector: &never_started,
            side_channels: None,
        },
    ];

    // Tight deadline relative to slow list; first may overrun, second skipped.
    let report = refresh_bounded(&targets, &personal_ctx(), Duration::from_millis(10), 10);

    let deadline_ids: Vec<&str> = report
        .failures
        .iter()
        .filter(|f| f.reason == "deadline_exceeded")
        .map(|f| f.connector_id.as_str())
        .collect();

    // Subsequent target must be marked deadline_exceeded without starting.
    assert!(
        deadline_ids.contains(&"builtin.never_started"),
        "second target must be deadline_exceeded without starting; report={report:?}"
    );
    assert!(
        report.truncated,
        "deadline path must set truncated; report={report:?}"
    );
    // Honest inter-op semantics: wall elapsed may exceed deadline because the
    // first blocking list can overrun remaining budget (not asserted ≤ here).
    assert!(
        report.elapsed >= Duration::from_millis(10),
        "slow first list should consume at least the deadline window; elapsed={:?}",
        report.elapsed
    );
}
