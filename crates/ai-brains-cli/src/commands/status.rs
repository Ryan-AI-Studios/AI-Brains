//! `ai-brains status` — unified vault glance (T320).
//!
//! In-process compose of daemon IPC Running/Stopped, doctor summary, graph
//! density one-liner, and nightly last-run + schedule. Fail-open per section.
//! Exit 0 always for a successful emit. No HTTP probes; no daemon TCP; no
//! AppContext / migrate. CLI-local JSON envelope (`schema_version: 1`).

use crate::commands::daemon::status_next_line;
use crate::commands::doctor::{DoctorOptions, build_report, format_doctor_summary};
use crate::commands::format_resolve::resolve_human_json_format;
use crate::daemon_client::DaemonClient;
use crate::daemon_probe::{DaemonProbePolicy, probe_daemon_reachable};
use crate::graph_density::{
    Assessment, GatherResult, GraphDensitySnapshot, PINNED_COUNT_FAILED_MSG, assess_graph_density,
    format_ratio, gather_density_snapshot,
};
use crate::key_resolve::resolve_operator_sqlcipher_key;
use ai_brains_contracts::doctor::{CheckSeverity, DoctorReport, DoctorStatus};
use ai_brains_store::connection::VaultConnection;
use serde::Serialize;
use std::io::IsTerminal;
use std::path::PathBuf;

/// Prefix-less JSON `next_step` when daemon Stopped (F27). Human uses
/// [`status_next_line`] verbatim (`next: ai-brains daemon start`).
pub(crate) const NEXT_STEP_DAEMON_START: &str = "ai-brains daemon start";

const NIGHTLY_TASK: &str = "AI-Brains-Nightly";

/// CLI options for top-level `status`.
pub struct StatusOptions {
    pub vault_path: PathBuf,
    pub key: Option<String>,
    pub format: String,
}

/// Production entry: Status-policy IPC probe + compose + emit. Always exit 0
/// on successful emit (do not call `doctor::exit_code_for`).
pub async fn run(opts: StatusOptions) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();
    let daemon_up = probe_daemon_reachable(&client, DaemonProbePolicy::Status).await;
    run_with_daemon_state(opts, daemon_up)
}

/// Injectable daemon-up for unit / hermetic paths.
pub fn run_with_daemon_state(
    opts: StatusOptions,
    daemon_up: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let envelope = build_status_envelope(&opts, daemon_up);
    let is_tty = std::io::stdout().is_terminal();
    let resolved = resolve_human_json_format(&opts.format, is_tty);
    if resolved == "json" {
        let pretty = serde_json::to_string_pretty(&envelope)
            .map_err(|e| format!("status JSON serialize failed: {e}"))?;
        println!("{pretty}");
    } else {
        print!("{}", format_status_human(&envelope));
    }
    Ok(())
}

/// Frozen JSON envelope (`schema_version: 1`).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct StatusEnvelope {
    pub schema_version: u32,
    pub daemon: DaemonSection,
    pub doctor: DoctorSection,
    pub graph: GraphSection,
    pub nightly: NightlySection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step: Option<String>,
    /// Human-only: verbatim `format_doctor_summary` when doctor built (F38 / AC8).
    #[serde(skip)]
    pub doctor_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct DaemonSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct DoctorSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warn: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attention: Option<Vec<AttentionItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AttentionItem {
    pub name: String,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct GraphSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_node_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edges: Option<i64>,
    /// Human-only pinned count (not in §5.1 JSON keys).
    #[serde(skip)]
    pub pinned: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Nightly glance section. Ok path always emits `last_run` / `scheduled` /
/// `last_task_result` (JSON null when absent). Err path emits only `error`.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub(crate) enum NightlySection {
    Ok {
        last_run: Option<String>,
        scheduled: Option<bool>,
        last_task_result: Option<String>,
    },
    Err {
        error: String,
    },
}

impl NightlySection {
    fn ok(
        last_run: Option<String>,
        scheduled: Option<bool>,
        last_task_result: Option<String>,
    ) -> Self {
        Self::Ok {
            last_run,
            scheduled,
            last_task_result,
        }
    }

    fn err(msg: impl Into<String>) -> Self {
        Self::Err { error: msg.into() }
    }
}

/// Build the four-section envelope (fail-open per section).
pub(crate) fn build_status_envelope(opts: &StatusOptions, daemon_up: bool) -> StatusEnvelope {
    let daemon = DaemonSection {
        state: Some(if daemon_up {
            "Running".into()
        } else {
            "Stopped".into()
        }),
        error: None,
    };

    let (doctor, doctor_summary) = match build_doctor_section(opts, daemon_up) {
        Ok((d, summary)) => (d, Some(summary)),
        Err(e) => (
            DoctorSection {
                status: None,
                vault_path: None,
                ok: None,
                warn: None,
                fail: None,
                skip: None,
                attention: None,
                error: Some(e),
            },
            None,
        ),
    };

    let (graph, nightly) = match open_glance_conn(opts) {
        Ok(conn) => {
            let graph = match build_graph_section(&conn) {
                Ok(g) => g,
                Err(e) => GraphSection {
                    status: None,
                    density: None,
                    edge_node_ratio: None,
                    nodes: None,
                    edges: None,
                    pinned: None,
                    error: Some(e),
                },
            };
            let nightly = match build_nightly_section(&conn) {
                Ok(n) => n,
                Err(e) => NightlySection::err(e),
            };
            (graph, nightly)
        }
        Err(e) => (
            GraphSection {
                status: None,
                density: None,
                edge_node_ratio: None,
                nodes: None,
                edges: None,
                pinned: None,
                error: Some(e.clone()),
            },
            NightlySection::err(e),
        ),
    };

    let next_step = if daemon_up {
        None
    } else {
        Some(NEXT_STEP_DAEMON_START.to_string())
    };

    StatusEnvelope {
        schema_version: 1,
        daemon,
        doctor,
        graph,
        nightly,
        next_step,
        doctor_summary,
    }
}

fn build_doctor_section(
    opts: &StatusOptions,
    daemon_up: bool,
) -> Result<(DoctorSection, String), String> {
    let doctor_opts = DoctorOptions {
        vault_path: opts.vault_path.clone(),
        key: opts.key.clone(),
        format: "human".into(),
        json: false,
        fail_on_degraded: false,
        kit_path: None,
        passphrase_file: None,
        backup_max_age: "7d".into(),
        full: false,
        // build_report ignores summary; glance calls format_doctor_summary itself.
        summary: false,
    };
    let report = build_report(&doctor_opts, daemon_up).map_err(|e| e.to_string())?;
    let summary = format_doctor_summary(&report);
    Ok((slim_doctor_section(&report), summary))
}

pub(crate) fn slim_doctor_section(report: &DoctorReport) -> DoctorSection {
    let mut ok = 0usize;
    let mut warn = 0usize;
    let mut fail = 0usize;
    let mut skip = 0usize;
    let mut attention = Vec::new();
    for c in &report.checks {
        match c.severity {
            CheckSeverity::Ok => ok += 1,
            CheckSeverity::Warn => warn += 1,
            CheckSeverity::Fail => fail += 1,
            CheckSeverity::Skip => skip += 1,
        }
        if matches!(c.severity, CheckSeverity::Warn | CheckSeverity::Fail) {
            attention.push(AttentionItem {
                name: c.name.clone(),
                severity: severity_label(c.severity).to_string(),
                message: c.message.clone().unwrap_or_default(),
                remediation: c.remediation.clone(),
            });
        }
    }
    DoctorSection {
        status: Some(doctor_status_label(report.status).to_string()),
        vault_path: Some(report.vault_path.clone()),
        ok: Some(ok),
        warn: Some(warn),
        fail: Some(fail),
        skip: Some(skip),
        attention: Some(attention),
        error: None,
    }
}

fn doctor_status_label(s: DoctorStatus) -> &'static str {
    match s {
        DoctorStatus::Ok => "ok",
        DoctorStatus::Degraded => "degraded",
        DoctorStatus::Fail => "fail",
    }
}

fn severity_label(s: CheckSeverity) -> &'static str {
    match s {
        CheckSeverity::Ok => "ok",
        CheckSeverity::Warn => "warn",
        CheckSeverity::Fail => "fail",
        CheckSeverity::Skip => "skip",
    }
}

fn open_glance_conn(opts: &StatusOptions) -> Result<VaultConnection, String> {
    let key = resolve_operator_sqlcipher_key(opts.key.clone()).map_err(|e| e.to_string())?;
    VaultConnection::open_read_intent(&opts.vault_path, &key).map_err(|e| e.to_string())
}

fn build_graph_section(conn: &VaultConnection) -> Result<GraphSection, String> {
    let guard = conn.lock().map_err(|e| e.to_string())?;
    let gather = gather_density_snapshot(&guard)?;
    graph_section_from_gather(gather)
}

fn graph_section_from_gather(gather: GatherResult) -> Result<GraphSection, String> {
    match gather {
        GatherResult::TablesMissing => Err("graph tables missing".into()),
        GatherResult::PinnedCountFailed { .. } => Err(PINNED_COUNT_FAILED_MSG.into()),
        GatherResult::Ok(snap) => {
            let assessment = assess_graph_density(&snap);
            Ok(graph_from_assessment(&snap, &assessment))
        }
    }
}

fn graph_from_assessment(snap: &GraphDensitySnapshot, a: &Assessment) -> GraphSection {
    GraphSection {
        status: Some(a.status.to_string()),
        density: Some(a.density.to_string()),
        edge_node_ratio: Some(a.edge_node_ratio),
        nodes: Some(snap.nodes),
        edges: Some(snap.edges),
        pinned: Some(snap.pinned_memories),
        error: None,
    }
}

fn build_nightly_section(conn: &VaultConnection) -> Result<NightlySection, String> {
    let last_run = read_last_nightly_run(conn)?;
    #[cfg(windows)]
    let (scheduled, last_task_result) = {
        let snap = crate::commands::nightly::fetch_schedule_snapshot(NIGHTLY_TASK);
        // Mapper identity with nightly --status JSON (T255 / F12): next_run.is_some().
        let scheduled = Some(snap.snap.next_run.is_some());
        let last_task_result = snap.snap.last_result.clone();
        (scheduled, last_task_result)
    };
    #[cfg(not(windows))]
    let (scheduled, last_task_result): (Option<bool>, Option<String>) = (None, None);
    let _ = NIGHTLY_TASK; // used on Windows; silence unused on other targets
    Ok(NightlySection::ok(last_run, scheduled, last_task_result))
}

fn read_last_nightly_run(conn: &VaultConnection) -> Result<Option<String>, String> {
    let guard = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = guard
        .prepare("SELECT value FROM sync_state WHERE key = ?1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query(["last_nightly_run"])
        .map_err(|e| e.to_string())?;
    match rows.next().map_err(|e| e.to_string())? {
        Some(row) => {
            let v: String = row.get(0).map_err(|e| e.to_string())?;
            if v.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(v))
            }
        }
        None => Ok(None),
    }
}

/// Human layout (F3 / §5.2). Doctor block is `format_doctor_summary` verbatim.
pub(crate) fn format_status_human(envelope: &StatusEnvelope) -> String {
    let mut out = String::new();

    // daemon
    if let Some(err) = &envelope.daemon.error {
        out.push_str(&format!("daemon: error={err}\n"));
    } else {
        let state = envelope.daemon.state.as_deref().unwrap_or("unknown");
        out.push_str(&format!("daemon: {state}\n"));
    }

    // doctor — verbatim format_doctor_summary when available (F38 / AC8)
    if let Some(err) = &envelope.doctor.error {
        out.push_str(&format!("doctor: error={err}\n"));
    } else if let Some(summary) = &envelope.doctor_summary {
        out.push_str(summary);
        if !out.ends_with('\n') {
            out.push('\n');
        }
    } else {
        out.push_str(&format_doctor_block(&envelope.doctor));
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }

    // graph
    out.push_str(&format_status_graph_line(&envelope.graph));
    if !out.ends_with('\n') {
        out.push('\n');
    }

    // nightly
    out.push_str(&format_status_nightly_line(&envelope.nightly));
    if !out.ends_with('\n') {
        out.push('\n');
    }

    // next: only when Stopped — reuse daemon helper (F27 / AC7)
    let is_running = envelope
        .daemon
        .state
        .as_deref()
        .is_some_and(|s| s == "Running");
    if let Some(line) = status_next_line(is_running) {
        out.push_str(line);
        out.push('\n');
    }

    out
}

fn format_doctor_block(doctor: &DoctorSection) -> String {
    // Mirror format_doctor_summary layout from slim fields (same text contract).
    let status = doctor.status.as_deref().unwrap_or("unknown");
    let vault = doctor.vault_path.as_deref().unwrap_or("");
    let ok = doctor.ok.unwrap_or(0);
    let warn = doctor.warn.unwrap_or(0);
    let fail = doctor.fail.unwrap_or(0);
    let skip = doctor.skip.unwrap_or(0);
    let mut lines = Vec::new();
    lines.push(format!(
        "doctor: status={status}  vault={vault}  ok={ok} warn={warn} fail={fail} skip={skip}"
    ));
    let attention = doctor.attention.as_deref().unwrap_or(&[]);
    if attention.is_empty() {
        lines.push("No issues.".into());
    } else {
        lines.push("attention:".into());
        for a in attention {
            lines.push(format!("  [{}] {} — {}", a.severity, a.name, a.message));
            if let Some(rem) = &a.remediation {
                lines.push(format!("         remediation: {rem}"));
            }
        }
    }
    lines.join("\n") + "\n"
}

/// Human graph one-liner (AC16): `E/N=` three decimals.
pub(crate) fn format_status_graph_line(graph: &GraphSection) -> String {
    if let Some(err) = &graph.error {
        return format!("graph: error={err}");
    }
    let status = graph.status.as_deref().unwrap_or("unknown");
    let nodes = graph.nodes.unwrap_or(0);
    let edges = graph.edges.unwrap_or(0);
    let ratio = graph.edge_node_ratio.unwrap_or(0.0);
    let pinned = graph.pinned.unwrap_or(0);
    format!(
        "graph: {status}  nodes={nodes} edges={edges} E/N={} pinned={pinned}",
        format_ratio(ratio)
    )
}

pub(crate) fn format_status_nightly_line(nightly: &NightlySection) -> String {
    match nightly {
        NightlySection::Err { error } => format!("nightly: error={error}"),
        NightlySection::Ok {
            last_run,
            scheduled,
            last_task_result,
        } => {
            let last = last_run.as_deref().unwrap_or("never");
            let scheduled = match scheduled {
                Some(true) => "Yes",
                Some(false) => "No",
                None => "unknown",
            };
            let last_result = last_task_result.as_deref().unwrap_or("n/a");
            format!("nightly: last={last}  scheduled={scheduled}  last_result={last_result}")
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::commands::daemon::status_next_line;
    use ai_brains_contracts::doctor::{DoctorReport, HealthCheck};

    fn envelope_from_parts(
        daemon_running: bool,
        doctor: DoctorSection,
        graph: GraphSection,
        nightly: NightlySection,
    ) -> StatusEnvelope {
        let doctor_summary = if doctor.error.is_none() {
            Some(format_doctor_block(&doctor))
        } else {
            None
        };
        StatusEnvelope {
            schema_version: 1,
            daemon: DaemonSection {
                state: Some(if daemon_running {
                    "Running".into()
                } else {
                    "Stopped".into()
                }),
                error: None,
            },
            doctor,
            graph,
            nightly,
            next_step: if daemon_running {
                None
            } else {
                Some(NEXT_STEP_DAEMON_START.to_string())
            },
            doctor_summary,
        }
    }

    fn apply_doctor_error(mut envelope: StatusEnvelope, err: impl Into<String>) -> StatusEnvelope {
        envelope.doctor = DoctorSection {
            status: None,
            vault_path: None,
            ok: None,
            warn: None,
            fail: None,
            skip: None,
            attention: None,
            error: Some(err.into()),
        };
        envelope.doctor_summary = None;
        envelope
    }

    fn apply_graph_error(mut envelope: StatusEnvelope, err: impl Into<String>) -> StatusEnvelope {
        envelope.graph = GraphSection {
            status: None,
            density: None,
            edge_node_ratio: None,
            nodes: None,
            edges: None,
            pinned: None,
            error: Some(err.into()),
        };
        envelope
    }

    fn fixture_doctor_ok() -> DoctorSection {
        DoctorSection {
            status: Some("ok".into()),
            vault_path: Some("C:\\tmp\\vault.db".into()),
            ok: Some(15),
            warn: Some(0),
            fail: Some(0),
            skip: Some(0),
            attention: Some(vec![]),
            error: None,
        }
    }

    fn fixture_graph_sparse() -> GraphSection {
        GraphSection {
            status: Some("sparse".into()),
            density: Some("warn".into()),
            edge_node_ratio: Some(0.41550127416247123),
            nodes: Some(64356),
            edges: Some(26740),
            pinned: Some(51267),
            error: None,
        }
    }

    fn fixture_nightly_never_unscheduled() -> NightlySection {
        NightlySection::ok(None, Some(false), None)
    }

    #[test]
    fn status_envelope__fixture__frozen_keys_schema_1() {
        let env = envelope_from_parts(
            true,
            fixture_doctor_ok(),
            fixture_graph_sparse(),
            fixture_nightly_never_unscheduled(),
        );
        let v = serde_json::to_value(&env).expect("serialize");
        assert_eq!(v["schema_version"], 1);
        assert!(v.get("daemon").is_some());
        assert!(v.get("doctor").is_some());
        assert!(v.get("graph").is_some());
        assert!(v.get("nightly").is_some());
        assert!(v.get("next_step").is_none(), "Running omits next_step");
        assert_eq!(v["daemon"]["state"], "Running");
    }

    #[test]
    fn status_envelope__doctor_err__error_keeps_daemon() {
        let env = apply_doctor_error(
            envelope_from_parts(
                true,
                fixture_doctor_ok(),
                fixture_graph_sparse(),
                fixture_nightly_never_unscheduled(),
            ),
            "vault open failed",
        );
        let v = serde_json::to_value(&env).expect("serialize");
        assert!(
            v["doctor"]["error"].as_str().is_some_and(|s| !s.is_empty()),
            "doctor.error nonempty"
        );
        assert_eq!(v["daemon"]["state"], "Running");
        assert!(v["doctor"].get("status").is_none());
    }

    #[test]
    fn cargo_pkg_version__workspace__is_0_1_5() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.5");
    }

    #[test]
    fn graph_section_from_gather__pinned_count_failed__error_not_fake_zero() {
        use crate::graph_density::PINNED_COUNT_FAILED_MSG;

        let empty = graph_section_from_gather(GatherResult::PinnedCountFailed {
            nodes: 0,
            edges: 0,
            memory_nodes: Some(0),
        });
        match empty {
            Err(e) => assert_eq!(e, PINNED_COUNT_FAILED_MSG),
            Ok(g) => panic!(
                "empty-graph COUNT fail must be Err, not Ok pinned={:?} status={:?}",
                g.pinned, g.status
            ),
        }

        let would_be_sparse = graph_section_from_gather(GatherResult::PinnedCountFailed {
            nodes: 100,
            edges: 10,
            memory_nodes: Some(0),
        });
        match would_be_sparse {
            Err(e) => assert_eq!(e, PINNED_COUNT_FAILED_MSG),
            Ok(g) => panic!(
                "COUNT fail must be Err, not Ok sparse pinned={:?} status={:?}",
                g.pinned, g.status
            ),
        }

        let env = apply_graph_error(
            envelope_from_parts(
                true,
                fixture_doctor_ok(),
                fixture_graph_sparse(),
                fixture_nightly_never_unscheduled(),
            ),
            PINNED_COUNT_FAILED_MSG,
        );
        let v = serde_json::to_value(&env).expect("serialize");
        assert!(
            v["graph"]["error"].as_str().is_some_and(|s| !s.is_empty()),
            "graph.error nonempty"
        );
        assert!(v["graph"].get("status").is_none());
        assert!(v["graph"].get("nodes").is_none());
        assert!(v["graph"].get("edges").is_none());
        assert!(v.get("daemon").is_some());
        assert_eq!(v["schema_version"], 1);

        let err_section = GraphSection {
            status: None,
            density: None,
            edge_node_ratio: None,
            nodes: None,
            edges: None,
            pinned: None,
            error: Some(PINNED_COUNT_FAILED_MSG.into()),
        };
        let line = format_status_graph_line(&err_section);
        assert!(
            line.contains("graph: error="),
            "human error arm; got {line}"
        );
        assert!(
            !line.contains("pinned=0"),
            "must not invent pinned=0; got {line}"
        );
    }

    #[test]
    fn status_envelope__graph_err__error_keeps_others() {
        let env = apply_graph_error(
            envelope_from_parts(
                false,
                fixture_doctor_ok(),
                fixture_graph_sparse(),
                fixture_nightly_never_unscheduled(),
            ),
            "graph tables missing",
        );
        let v = serde_json::to_value(&env).expect("serialize");
        assert_eq!(v["graph"]["error"], "graph tables missing");
        assert_eq!(v["daemon"]["state"], "Stopped");
        assert!(v["doctor"].get("status").is_some());
        assert!(
            v["nightly"].get("last_run").is_some() || v["nightly"].get("error").is_some(),
            "nightly section present"
        );
        assert!(v["nightly"].get("last_run").is_some());
    }

    #[test]
    fn status_nightly_human__never_and_not_scheduled() {
        let nightly = NightlySection::ok(None, Some(false), None);
        let line = format_status_nightly_line(&nightly);
        assert!(
            line.contains("last=never"),
            "expected last=never; got {line}"
        );
        assert!(
            line.contains("scheduled=No"),
            "expected scheduled=No; got {line}"
        );
        let v = serde_json::to_value(&nightly).expect("serialize");
        assert!(v["last_run"].is_null());
        assert_eq!(v["scheduled"], false);
    }

    #[test]
    fn format_status_human__stopped__reuses_daemon_next() {
        let env = envelope_from_parts(
            false,
            fixture_doctor_ok(),
            fixture_graph_sparse(),
            fixture_nightly_never_unscheduled(),
        );
        let human = format_status_human(&env);
        let expected = status_next_line(false).expect("Stopped has next");
        assert!(
            human.lines().any(|l| l == expected),
            "human must contain exact daemon::status_next_line(false)={expected:?}; got:\n{human}"
        );
        assert_eq!(
            env.next_step.as_deref(),
            Some(NEXT_STEP_DAEMON_START),
            "JSON next_step is prefix-less const"
        );
        assert!(
            !human.contains(NEXT_STEP_DAEMON_START) || human.contains("next:"),
            "human uses prefixed form"
        );
    }

    #[test]
    fn format_status_human__running__no_next() {
        let env = envelope_from_parts(
            true,
            fixture_doctor_ok(),
            fixture_graph_sparse(),
            fixture_nightly_never_unscheduled(),
        );
        let human = format_status_human(&env);
        assert!(
            !human.contains("next:"),
            "Running must omit next:; got:\n{human}"
        );
        assert!(env.next_step.is_none());
    }

    #[test]
    fn format_status_human__includes_doctor_summary_no_nightly_banner() {
        let mut checks = vec![HealthCheck::warn(
            "graph_density",
            "sparse: edge/node ratio below floor",
            None,
        )];
        for i in 0..14 {
            checks.push(HealthCheck::ok_msg(format!("ok_{i}"), "ok"));
        }
        let report = DoctorReport {
            schema_version: 1,
            status: DoctorStatus::Degraded,
            vault_path: "C:\\tmp\\vault.db".into(),
            generated_at: "2026-08-29T00:00:00Z".into(),
            checks,
        };
        let summary = format_doctor_summary(&report);
        let doctor = slim_doctor_section(&report);
        let mut env = envelope_from_parts(
            true,
            doctor,
            fixture_graph_sparse(),
            fixture_nightly_never_unscheduled(),
        );
        env.doctor_summary = Some(summary.clone());
        let human = format_status_human(&env);
        let summary_trim = summary.trim_end_matches('\n');
        assert!(
            human.contains(summary_trim),
            "human must include format_doctor_summary verbatim;\nsummary=\n{summary_trim}\nhuman=\n{human}"
        );
        assert!(
            !human.contains("=== Nightly Status ==="),
            "must not print nightly banner"
        );
        assert!(!human.contains("LLM backend"), "no LLM backend line");
        assert!(!human.contains("probe="), "no probe= line");
    }

    #[test]
    fn format_status_graph_line__three_decimal_en() {
        let graph = fixture_graph_sparse();
        let line = format_status_graph_line(&graph);
        assert!(line.contains("E/N=0.416"), "three-decimal E/N; got {line}");
        assert!(
            !line.contains("0.415501"),
            "must not dump raw f64; got {line}"
        );
        let v = serde_json::to_value(&graph).expect("serialize");
        assert!(
            (v["edge_node_ratio"].as_f64().unwrap() - 0.41550127416247123).abs() < 1e-12,
            "JSON keeps raw f64"
        );
    }

    #[test]
    fn slim_doctor_section__attention_warn_only() {
        let report = DoctorReport {
            schema_version: 1,
            status: DoctorStatus::Degraded,
            vault_path: "/v".into(),
            generated_at: "t".into(),
            checks: vec![
                HealthCheck::ok_msg("a", "ok"),
                HealthCheck::warn("graph_density", "sparse", Some("rebuild".into())),
                HealthCheck::skip("integrity", "skipped"),
            ],
        };
        let slim = slim_doctor_section(&report);
        assert_eq!(slim.status.as_deref(), Some("degraded"));
        assert_eq!(slim.ok, Some(1));
        assert_eq!(slim.warn, Some(1));
        assert_eq!(slim.skip, Some(1));
        let att = slim.attention.as_ref().expect("attention");
        assert_eq!(att.len(), 1);
        assert_eq!(att[0].name, "graph_density");
        assert_eq!(att[0].remediation.as_deref(), Some("rebuild"));
    }
}
