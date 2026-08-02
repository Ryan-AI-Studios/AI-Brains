//! `ai-brains migrate governed` — shadow replay + differential report (T168 / P9.2).
//!
//! - **M1** Dry-run default; `--confirm` for dest materialize + T167 apply.
//! - **M2** Classification / apply only via T167 (`classify_legacy` / `apply_legacy_import`).
//! - **M3/M6** Destination safety via `shadow::refuse_unsafe_destination`, plus
//!   multi-link (hardlink) dest refuse so confirm cannot R/W-open a dest that
//!   shares an inode with source/live (Codex R3).
//! - **M4/M11** Report has no plaintext bodies; CE honesty false.
//! - **M5** Never `migrate()` source; source open via pure RO
//!   `VaultConnection::open_read_intent`; fingerprint re-verify **after** all
//!   output writes (report + manifest).
//! - **M7** Live source refused unless `--allow-live-source`.
//! - **M8** `refuse_unsafe_report_path` for report location (incl. migrate-manifest,
//!   hardlink refuse when path exists).
//! - **M18** `refuse_unsafe_manifest_path`: sibling manifest must not collide with
//!   source/dest or be reparse/hardlink before any confirm write.
//! - **M15** Missing source → `NOT_FOUND` exit 4.
//! - **M16** Content-based `migrate_source_fingerprint`.
//! - **M17/M18** Re-apply: import-only when dest non-empty + matching manifest.
//! - **M19** `--source-key` / `--destination-key` with `--key` fallback (flag after subcommand OK).
//! - **M20** Envelope copy only (never projections).
//! - **M21** Batch copy 5000 + stderr progress ≥1000.
//! - **M22** Pass `read_all_events` order to classify (no re-sort).

use crate::artifact_security::{
    is_hardlink, is_reparse_or_symlink, refuse_if_hardlink, refuse_if_reparse,
};
use crate::commands::governed_common::{OutputFormat, fail_api, resolve_principal};
use crate::commands::shadow::{refuse_unsafe_destination, resolve_live_vault_path};
use ai_brains_contracts::response::ApiError;
use ai_brains_control_plane::{
    ApplyOpts, ImportActionKind, ImportMechanism, ImportOpts, ImportPlan, ImportReport, StorePorts,
    SystemClock, apply_legacy_import, classify_legacy, parse_scope_key,
};
use ai_brains_core::ids::PrincipalId;
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::SqlCipherKey;
use ai_brains_events::Envelope;
use ai_brains_path::paths_refer_to_same_location;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use uuid::Uuid;

const MIGRATE_MANIFEST_VERSION: u32 = 1;
const CONTENT_HASH_CAP: usize = 500;
const COPY_BATCH_SIZE: usize = 5000;
const PROGRESS_THRESHOLD: usize = 1000;
const DEFAULT_SQL_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

pub struct GovernedOptions {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub report: PathBuf,
    pub dry_run: bool,
    pub confirm: bool,
    pub default_scope: Option<String>,
    pub copy_events: bool,
    pub allow_live_source: bool,
    pub force_overwrite: bool,
    pub source_key: Option<String>,
    pub destination_key: Option<String>,
    pub key: Option<String>,
}

// ---------------------------------------------------------------------------
// Report types (schema v1 — types only, no domain logic)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct DifferentialReport {
    pub schema_version: u32,
    pub command: String,
    pub dry_run: bool,
    pub created_at: String,
    pub source_path: String,
    pub destination_path: String,
    pub source_fingerprint: String,
    pub live_vault_resolved: bool,
    pub plan_hash: String,
    pub report_hash: String,
    pub event_counts: EventCounts,
    pub classification: ClassificationTotals,
    pub unresolved: Vec<UnresolvedEntry>,
    pub privacy: PrivacySection,
    pub gaps: GapsSection,
    pub content_hashes: Vec<ContentHashEntry>,
    pub content_hashes_truncated: bool,
    pub replay_consistency: ReplayConsistency,
    pub ce_honesty: CeHonesty,
    pub rollback: RollbackSection,
    pub legacy_import_applied: bool,
    pub t167_plan_hash: String,
    pub manifest_written: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventCounts {
    pub source_events: u64,
    pub dest_events_before: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest_events_after: Option<u64>,
    pub would_copy_events: u64,
    pub would_import_appends: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassificationTotals {
    pub evidence: u64,
    pub conclusion_candidate: u64,
    pub decision_proposed: u64,
    pub review_opened: u64,
    pub skipped: u64,
    pub already_imported: u64,
    pub already_governed: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UnresolvedEntry {
    pub original_event_id: String,
    pub reason_code: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrivacySection {
    pub by_level: BTreeMap<String, u64>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GapsSection {
    pub forgotten_source: u64,
    pub missing_source: u64,
    pub missing_scope: u64,
    pub unknown_payload: u64,
    pub out_of_matrix: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ContentHashEntry {
    pub original_event_id: String,
    pub payload_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplayConsistency {
    pub mode: String,
    pub source_event_count: u64,
    pub plan_action_count: u64,
    pub ok: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CeHonesty {
    pub claims_cryptographic_erasure: bool,
    pub legacy_plaintext_limitation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RollbackSection {
    pub source_modified: bool,
    pub instructions: Vec<String>,
}

/// Hash view: all report fields except `created_at` and `report_hash` (M10).
#[derive(Debug, Clone, Serialize)]
struct ReportHashView {
    schema_version: u32,
    command: String,
    dry_run: bool,
    source_path: String,
    destination_path: String,
    source_fingerprint: String,
    live_vault_resolved: bool,
    plan_hash: String,
    event_counts: EventCounts,
    classification: ClassificationTotals,
    unresolved: Vec<UnresolvedEntry>,
    privacy: PrivacySection,
    gaps: GapsSection,
    content_hashes: Vec<ContentHashEntry>,
    content_hashes_truncated: bool,
    replay_consistency: ReplayConsistency,
    ce_honesty: CeHonesty,
    rollback: RollbackSection,
    legacy_import_applied: bool,
    t167_plan_hash: String,
    manifest_written: bool,
}

#[derive(Debug, Serialize)]
struct MigrateManifest {
    version: u32,
    source_path: String,
    dest_path: String,
    source_fingerprint: String,
    plan_hash: String,
    created_at: String,
}

// ---------------------------------------------------------------------------
// Public entry
// ---------------------------------------------------------------------------

pub fn run_governed(opts: GovernedOptions) -> Result<(), Box<dyn std::error::Error>> {
    if opts.dry_run && opts.confirm {
        return fail_invalid_payload(
            "cannot combine --dry-run and --confirm; omit both for dry-run default, or pass --confirm alone to apply",
        );
    }
    let is_dry_run = !opts.confirm;

    let live = resolve_live_vault_path();
    if live.is_none() {
        eprintln!(
            "note: no live vault resolved (AI_BRAINS_VAULT_PATH unset and ~/.ai-brains/.env \
             has no vault path); only source/destination same-path checks apply"
        );
    }

    // M3 / M6 — dest safety (reuse T147 helper; rewrite prefix for migrate messaging).
    refuse_unsafe_destination(&opts.source, &opts.destination, live.as_deref())
        .map_err(|e| path_refused(rewrite_refuse_prefix(e.to_string())))?;

    // Codex R3 — multi-link dest bypasses path-string equality (hardlink to source/live).
    // Skip when --force-overwrite: confirm deletes dest first then re-checks after wipe.
    if opts.destination.exists() && !opts.force_overwrite {
        refuse_hardlink_destination(&opts.destination)?;
    }

    // M7 — live source gate.
    if let Some(ref live_path) = live
        && paths_refer_to_same_location(&opts.source, live_path)
        && !opts.allow_live_source
    {
        return fail_path_refused(
            "refusing migrate: source equals the resolved live vault; use `shadow create` first \
             or pass --allow-live-source (destination still cannot equal live)",
        );
    }

    // M8 — report path safety (incl. refuse overwriting migrate-manifest).
    refuse_unsafe_report_path(&opts.report, &opts.source, &opts.destination)?;

    // M18 / Codex R3 — manifest sibling must not collide with source/dest or be reparse.
    refuse_unsafe_manifest_path(&opts.source, &opts.destination)?;

    // M15 — missing source is NOT_FOUND (exit 4), not a generic COMMAND_FAILED.
    if !opts.source.exists() {
        return fail_not_found(format!(
            "source vault does not exist: {}",
            opts.source.display()
        ));
    }

    let source_key = resolve_sql_key(opts.source_key.clone(), opts.key.clone());
    let dest_key = resolve_sql_key(opts.destination_key.clone(), opts.key.clone());

    // M5 / #12 — open source read-intent (no journal_mode mutation); never migrate source.
    let source_conn =
        VaultConnection::open_read_intent(&opts.source, &source_key).map_err(|e| {
            format!(
                "failed to open source vault at {} (check --source-key / --key): {e}",
                opts.source.display()
            )
        })?;
    let source_store = SqliteEventStore::new(source_conn);
    // M22 — preserve read_all_events order (occurred_at ASC, event_id ASC).
    let events = source_store.read_all_events().map_err(|e| {
        format!(
            "failed to read events from source vault {}: {e}",
            opts.source.display()
        )
    })?;

    let source_fp = migrate_source_fingerprint(&events);
    let source_bytes_before = file_len_or_zero(&opts.source);

    let default_scope = match opts.default_scope.as_deref() {
        Some(raw) => match parse_scope_key(raw) {
            Ok(s) => Some(s),
            Err(e) => {
                return fail_invalid_payload(format!("invalid --default-scope: {e}"));
            }
        },
        None => None,
    };

    // Confirm apply needs non-nil principal (T167); dry-run may use nil.
    let principal_id = if opts.confirm {
        resolve_principal(None).id
    } else {
        PrincipalId::from_uuid(Uuid::nil())
    };

    let import_opts = ImportOpts {
        dry_run: is_dry_run,
        include_truncated_summaries: false,
        default_scope,
        principal_id,
        command_id: Some(format!("migrate-governed-{}", Uuid::new_v4())),
    };

    // M2 — classify only via T167.
    let plan = classify_legacy(&events, &import_opts)
        .map_err(|e| format!("classify_legacy failed: {e}"))?;

    let mut dest_events_before: u64 = 0;
    let mut dest_events_after: Option<u64> = None;
    let mut would_copy_events: u64 = 0;
    let mut legacy_import_applied = false;
    let mut manifest_written = false;
    let mut did_copy = false;
    let mut apply_report: Option<ImportReport> = None;

    if is_dry_run {
        // Peek dest event count without migrating / writing / WAL-mutating.
        if opts.destination.exists() {
            match VaultConnection::open_read_intent(&opts.destination, &dest_key) {
                Ok(conn) => {
                    let store = SqliteEventStore::new(conn);
                    match store.read_all_events() {
                        Ok(ev) => dest_events_before = ev.len() as u64,
                        Err(e) => {
                            eprintln!(
                                "note: could not read destination event count ({}): {e}",
                                opts.destination.display()
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "note: could not open destination for event count ({}): {e}",
                        opts.destination.display()
                    );
                }
            }
        }
        would_copy_events = if dest_events_before == 0 && opts.copy_events {
            events.len() as u64
        } else {
            0
        };
        eprintln!(
            "note: source open uses read-intent (open_read_intent: no journal_mode pragma; never migrate source; T147 residual #12)"
        );
    } else {
        // Confirm path.
        run_confirm_materialize(
            &opts,
            &events,
            &plan,
            &source_fp,
            &dest_key,
            live.as_deref(),
            &mut dest_events_before,
            &mut dest_events_after,
            &mut would_copy_events,
            &mut legacy_import_applied,
            &mut manifest_written,
            &mut did_copy,
            &mut apply_report,
        )?;
    }

    // Dry-run: plan WouldAppend count. Confirm: applied this run (report honesty M13 / R1-01).
    let would_import_appends = match &apply_report {
        Some(ar) => ar.applied,
        None => plan
            .actions
            .iter()
            .filter(|a| a.mechanism == ImportMechanism::WouldAppend)
            .count() as u64,
    };

    // Build + write report first, then final source integrity (Codex R2 P1-02).
    // Integrity must run *after* report/manifest writes so a hardlinked report
    // path that truncated the source is detected even if early refuse was raced.
    let report = build_report(BuildReportArgs {
        is_dry_run,
        source: &opts.source,
        destination: &opts.destination,
        source_fingerprint: &source_fp,
        live_vault_resolved: live.is_some(),
        plan: &plan,
        events: &events,
        dest_events_before,
        dest_events_after,
        would_copy_events,
        would_import_appends,
        legacy_import_applied,
        manifest_written,
        // Provisional: final integrity below fails hard if source changed.
        source_modified: false,
        apply_report: apply_report.as_ref(),
    })?;

    write_report_file(&opts.report, &report)?;

    // M5 — re-verify source content fingerprint **after** all output writes.
    // Fail hard on mismatch (dry-run or confirm). Size check is secondary.
    verify_source_unmodified_after_writes(
        &opts.source,
        &source_key,
        &source_fp,
        source_bytes_before,
    )?;

    // Human one-liner summary on stdout.
    if is_dry_run {
        println!(
            "[dry-run] migrate governed: source_events={} plan_hash={} evidence={} unresolved={} would_copy={} would_import_appends={} report={}",
            report.event_counts.source_events,
            truncate_hash(&report.plan_hash),
            report.classification.evidence,
            report.unresolved.len(),
            report.event_counts.would_copy_events,
            report.event_counts.would_import_appends,
            opts.report.display()
        );
    } else {
        println!(
            "migrate governed: dest={} source_events={} dest_after={:?} copied={} import_applied={} plan_hash={} report={}",
            opts.destination.display(),
            report.event_counts.source_events,
            report.event_counts.dest_events_after,
            did_copy,
            legacy_import_applied,
            truncate_hash(&report.plan_hash),
            opts.report.display()
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Confirm materialize
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn run_confirm_materialize(
    opts: &GovernedOptions,
    events: &[Envelope],
    plan: &ImportPlan,
    source_fp: &str,
    dest_key: &SqlCipherKey,
    live: Option<&Path>,
    dest_events_before: &mut u64,
    dest_events_after: &mut Option<u64>,
    would_copy_events: &mut u64,
    legacy_import_applied: &mut bool,
    manifest_written: &mut bool,
    did_copy: &mut bool,
    apply_report_out: &mut Option<ImportReport>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Force-overwrite: wipe dest + sidecars + manifest, then treat as fresh.
    if opts.force_overwrite {
        delete_dest_artifacts(&opts.destination)?;
    }

    // Create parent dirs for dest.
    if let Some(parent) = opts.destination.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }

    // Soft TOCTOU re-check after mkdir / force-overwrite wipe.
    refuse_unsafe_destination(&opts.source, &opts.destination, live)
        .map_err(|e| path_refused(rewrite_refuse_prefix(e.to_string())))?;
    // Multi-link dest after wipe: still refuse if something re-linked the path.
    if opts.destination.exists() {
        refuse_hardlink_destination(&opts.destination)?;
    }
    // Manifest path may become unsafe if dest parent/layout changed (force-overwrite).
    refuse_unsafe_manifest_path(&opts.source, &opts.destination)?;

    let dest_conn = VaultConnection::open(&opts.destination, dest_key).map_err(|e| {
        format!(
            "failed to open destination vault at {} (check --destination-key / --key): {e}",
            opts.destination.display()
        )
    })?;
    // Dest-only migrate (M5 / M20).
    dest_conn.migrate()?;
    let dest_store = SqliteEventStore::new(dest_conn);
    let existing = dest_store.read_all_events()?;
    *dest_events_before = existing.len() as u64;

    let mut copy_events = opts.copy_events;
    if *dest_events_before > 0 {
        // M17 / M18 — re-apply requires matching manifest; import-only.
        let manifest_path = migrate_manifest_path(&opts.destination);
        if !manifest_path.exists() {
            return fail_path_refused(format!(
                "refusing migrate: destination is non-empty but migrate-manifest.json is missing at {}; \
                 use a fresh destination path or --force-overwrite",
                manifest_path.display()
            ));
        }
        let body = fs::read_to_string(&manifest_path)?;
        let existing_manifest: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            format!(
                "failed to parse migrate-manifest.json at {}: {e}",
                manifest_path.display()
            )
        })?;
        let mf_fp = existing_manifest
            .get("source_fingerprint")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if mf_fp != source_fp {
            return fail_path_refused(format!(
                "refusing migrate: destination migrate-manifest source_fingerprint mismatch \
                 (manifest={mf_fp}, current={source_fp}); use a fresh path or --force-overwrite"
            ));
        }
        copy_events = false; // M17 — import-only on re-apply
    }

    if copy_events && *dest_events_before == 0 {
        // M20 / M21 — envelope-only batch copy.
        copy_envelopes_batched(&dest_store, events)?;
        *did_copy = true;
        *would_copy_events = events.len() as u64;
    } else {
        *would_copy_events = 0;
    }

    // Apply probes dest for idempotency (already_imported); fold outcomes into report (R1-01).
    let ports = StorePorts::from_store(SqliteEventStore::new(dest_store.connection().clone()));
    let clock = SystemClock;
    let apply_report = apply_legacy_import(
        &ports.writer,
        &ports.query,
        &clock,
        plan,
        &ApplyOpts { confirm: true },
    )
    .map_err(|e| format!("apply_legacy_import failed: {e}"))?;

    *legacy_import_applied = apply_report.legacy_import_applied;
    *apply_report_out = Some(apply_report);

    let after = dest_store.read_all_events()?;
    *dest_events_after = Some(after.len() as u64);

    // M18 — mandatory migrate-manifest on confirm success.
    let created_at = created_at_rfc3339();
    let manifest = MigrateManifest {
        version: MIGRATE_MANIFEST_VERSION,
        source_path: opts.source.display().to_string(),
        dest_path: opts.destination.display().to_string(),
        source_fingerprint: source_fp.to_string(),
        plan_hash: plan.plan_hash.clone(),
        created_at,
    };
    let manifest_path = migrate_manifest_path(&opts.destination);
    refuse_hardlink_write_target(&manifest_path, "migrate-manifest")?;
    let body = format!("{}\n", serde_json::to_string_pretty(&manifest)?);
    write_operator_file_nofollow(&manifest_path, body.as_bytes(), "migrate-manifest")?;
    *manifest_written = true;

    Ok(())
}

fn copy_envelopes_batched(
    dest_store: &SqliteEventStore,
    events: &[Envelope],
) -> Result<(), Box<dyn std::error::Error>> {
    let total = events.len();
    let show_progress = total >= PROGRESS_THRESHOLD;
    let mut done = 0usize;
    for chunk in events.chunks(COPY_BATCH_SIZE) {
        dest_store.append_events(chunk)?;
        done += chunk.len();
        if show_progress {
            eprintln!("copied {done}/{total} events");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Fingerprint + report hash (M16 / M10)
// ---------------------------------------------------------------------------

/// Content-based source fingerprint: hex SHA-256 of sorted lines `event_id|payload_hash\n`.
pub fn migrate_source_fingerprint(events: &[Envelope]) -> String {
    let mut lines: Vec<String> = events
        .iter()
        .map(|e| format!("{}|{}\n", e.event_id, e.payload_hash))
        .collect();
    lines.sort();
    let mut hasher = Sha256::new();
    for line in &lines {
        hasher.update(line.as_bytes());
    }
    hex::encode(hasher.finalize())
}

/// Canonical report_hash (M10): SHA-256 of JSON excluding `created_at` and `report_hash`.
///
/// Sorts `unresolved` and `content_hashes` before hashing so order of those
/// collections does not affect the digest (canonical M10 / R1-05).
pub fn compute_report_hash(
    report: &DifferentialReport,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut unresolved = report.unresolved.clone();
    unresolved.sort_by(|a, b| {
        (&a.original_event_id, &a.reason_code).cmp(&(&b.original_event_id, &b.reason_code))
    });
    let mut content_hashes = report.content_hashes.clone();
    content_hashes.sort_by(|a, b| a.original_event_id.cmp(&b.original_event_id));

    let view = ReportHashView {
        schema_version: report.schema_version,
        command: report.command.clone(),
        dry_run: report.dry_run,
        source_path: report.source_path.clone(),
        destination_path: report.destination_path.clone(),
        source_fingerprint: report.source_fingerprint.clone(),
        live_vault_resolved: report.live_vault_resolved,
        plan_hash: report.plan_hash.clone(),
        event_counts: report.event_counts.clone(),
        classification: report.classification.clone(),
        unresolved,
        privacy: report.privacy.clone(),
        gaps: report.gaps.clone(),
        content_hashes,
        content_hashes_truncated: report.content_hashes_truncated,
        replay_consistency: report.replay_consistency.clone(),
        ce_honesty: report.ce_honesty.clone(),
        rollback: report.rollback.clone(),
        legacy_import_applied: report.legacy_import_applied,
        t167_plan_hash: report.t167_plan_hash.clone(),
        manifest_written: report.manifest_written,
    };
    let bytes =
        serde_json::to_vec(&view).map_err(|e| format!("report_hash serialization failed: {e}"))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

// ---------------------------------------------------------------------------
// Report build
// ---------------------------------------------------------------------------

struct BuildReportArgs<'a> {
    is_dry_run: bool,
    source: &'a Path,
    destination: &'a Path,
    source_fingerprint: &'a str,
    live_vault_resolved: bool,
    plan: &'a ImportPlan,
    events: &'a [Envelope],
    dest_events_before: u64,
    dest_events_after: Option<u64>,
    would_copy_events: u64,
    would_import_appends: u64,
    legacy_import_applied: bool,
    manifest_written: bool,
    /// True only when post-run source fingerprint/size diverges (M5); success path is false.
    source_modified: bool,
    /// Confirm-path T167 apply outcomes (dest-probed already_imported / applied).
    apply_report: Option<&'a ImportReport>,
}

fn build_report(
    args: BuildReportArgs<'_>,
) -> Result<DifferentialReport, Box<dyn std::error::Error>> {
    let event_kind_by_id: BTreeMap<Uuid, String> = args
        .events
        .iter()
        .map(|e| (e.event_id, e.event_type.to_string()))
        .collect();
    let payload_hash_by_id: BTreeMap<Uuid, String> = args
        .events
        .iter()
        .map(|e| (e.event_id, e.payload_hash.clone()))
        .collect();

    // Confirm: fold T167 apply counters (dest-probed). Dry-run: plan reason codes only.
    let already_imported = match args.apply_report {
        Some(ar) => ar.already_imported,
        None => args
            .plan
            .actions
            .iter()
            .filter(|a| a.reason_code == "already_imported")
            .count() as u64,
    };

    let classification = ClassificationTotals {
        evidence: args.plan.totals.evidence,
        conclusion_candidate: args.plan.totals.conclusion,
        decision_proposed: args.plan.totals.decision,
        review_opened: args.plan.totals.review,
        skipped: args.plan.totals.skipped,
        already_imported,
        already_governed: args.plan.totals.already_governed,
    };

    let mut unresolved: Vec<UnresolvedEntry> = args
        .plan
        .actions
        .iter()
        .filter(|a| a.kind == ImportActionKind::Unresolved)
        .map(|a| UnresolvedEntry {
            original_event_id: a.original_event_id.to_string(),
            reason_code: a.reason_code.clone(),
            kind: event_kind_by_id
                .get(&a.original_event_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string()),
        })
        .collect();
    unresolved.sort_by(|a, b| {
        (&a.original_event_id, &a.reason_code).cmp(&(&b.original_event_id, &b.reason_code))
    });

    let mut by_level: BTreeMap<String, u64> = BTreeMap::new();
    for a in &args.plan.actions {
        // Classified actions only (all plan rows are classified actions).
        let label = privacy_label(a.privacy).to_string();
        *by_level.entry(label).or_insert(0) += 1;
    }

    let mut gaps = GapsSection {
        forgotten_source: 0,
        missing_source: 0,
        missing_scope: 0,
        unknown_payload: 0,
        out_of_matrix: 0,
    };
    for a in &args.plan.actions {
        match a.reason_code.as_str() {
            "forgotten_source" => gaps.forgotten_source += 1,
            "missing_source" => gaps.missing_source += 1,
            "missing_scope" => gaps.missing_scope += 1,
            "unknown_payload" => gaps.unknown_payload += 1,
            "out_of_matrix" => gaps.out_of_matrix += 1,
            _ => {}
        }
    }

    // Content hashes: WouldAppend actions, sort by original_event_id, cap 500.
    let mut content_hashes: Vec<ContentHashEntry> = args
        .plan
        .actions
        .iter()
        .filter(|a| a.mechanism == ImportMechanism::WouldAppend)
        .map(|a| ContentHashEntry {
            original_event_id: a.original_event_id.to_string(),
            payload_hash: payload_hash_by_id
                .get(&a.original_event_id)
                .cloned()
                .unwrap_or_default(),
            derived_id: if a.derived_id.is_empty() {
                None
            } else {
                Some(a.derived_id.clone())
            },
        })
        .collect();
    content_hashes.sort_by(|a, b| a.original_event_id.cmp(&b.original_event_id));
    let content_hashes_truncated = content_hashes.len() > CONTENT_HASH_CAP;
    if content_hashes_truncated {
        content_hashes.truncate(CONTENT_HASH_CAP);
    }

    let source_event_count = args.events.len() as u64;
    let plan_action_count = args.plan.actions.len() as u64;
    let replay_ok = source_event_count > 0 || plan_action_count == 0;
    let mut warnings = Vec::new();
    if !replay_ok {
        warnings.push("empty source with non-empty plan (unexpected)".to_string());
    }

    let mut report = DifferentialReport {
        schema_version: 1,
        command: "migrate.governed".to_string(),
        dry_run: args.is_dry_run,
        created_at: created_at_rfc3339(),
        source_path: args.source.display().to_string(),
        destination_path: args.destination.display().to_string(),
        source_fingerprint: args.source_fingerprint.to_string(),
        live_vault_resolved: args.live_vault_resolved,
        plan_hash: args.plan.plan_hash.clone(),
        report_hash: String::new(),
        event_counts: EventCounts {
            source_events: source_event_count,
            dest_events_before: args.dest_events_before,
            dest_events_after: args.dest_events_after,
            would_copy_events: args.would_copy_events,
            would_import_appends: args.would_import_appends,
        },
        classification,
        unresolved,
        privacy: PrivacySection {
            by_level,
            note: "Imported entity privacy equals source envelope privacy (T167 L12); no authority upgrade."
                .to_string(),
        },
        gaps,
        content_hashes,
        content_hashes_truncated,
        replay_consistency: ReplayConsistency {
            mode: "count_digest_v1".to_string(),
            source_event_count,
            plan_action_count,
            ok: replay_ok,
            warnings,
        },
        ce_honesty: CeHonesty {
            claims_cryptographic_erasure: false,
            legacy_plaintext_limitation:
                "ADR-0016: copied legacy events remain non-CE.".to_string(),
        },
        rollback: RollbackSection {
            source_modified: args.source_modified,
            instructions: vec![
                "Do not point AI_BRAINS_VAULT_PATH at the destination until T170 dogfood passes."
                    .to_string(),
                "Discard destination vault and report to abort.".to_string(),
                "Live vault was not modified by this command.".to_string(),
                "Governed feature flags / dual-path remain under operator control (T170)."
                    .to_string(),
            ],
        },
        legacy_import_applied: args.legacy_import_applied,
        t167_plan_hash: args.plan.plan_hash.clone(),
        manifest_written: args.manifest_written,
    };

    report.report_hash = compute_report_hash(&report)?;
    Ok(report)
}

fn write_report_file(
    path: &Path,
    report: &DifferentialReport,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }
    // TOCTOU re-check: refuse hardlinked report targets before SOOT write.
    refuse_hardlink_write_target(path, "report")?;
    let body = format!("{}\n", serde_json::to_string_pretty(report)?);
    write_operator_file_nofollow(path, body.as_bytes(), "report")
}

/// T193 P1: operator report/manifest write via shared nofollow SOOT (Replace).
fn write_operator_file_nofollow(
    path: &Path,
    bytes: &[u8],
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| format!("{label} path has no parent directory: {}", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("{label} path missing UTF-8 file name: {}", path.display()))?;
    ai_brains_path::write_file_nofollow_under_parent_path(
        parent,
        file_name,
        bytes,
        ai_brains_path::CreateMode::Replace,
    )
    .map_err(|e| -> Box<dyn std::error::Error> {
        use ai_brains_path::CapOpenError;
        match e {
            CapOpenError::ReparseRefused(s) => format!(
                "refusing {label} write through reparse/symlink at {} ({s})",
                path.display()
            )
            .into(),
            CapOpenError::HardlinkRefused(s) => format!(
                "refusing {label} write through hardlink at {} ({s})",
                path.display()
            )
            .into(),
            other => format!("failed to write {label} {}: {other}", path.display()).into(),
        }
    })
}

// ---------------------------------------------------------------------------
// Safety helpers
// ---------------------------------------------------------------------------

/// M8 — refuse reparse/symlink/hardlink report path; refuse report == source, dest, or migrate-manifest.
pub fn refuse_unsafe_report_path(
    report: &Path,
    source: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if paths_refer_to_same_location(report, source) {
        return fail_path_refused("refusing migrate: report path equals the source vault file");
    }
    if paths_refer_to_same_location(report, destination) {
        return fail_path_refused(
            "refusing migrate: report path equals the destination vault file",
        );
    }
    // M18 honesty: report must not overwrite the mandatory migrate-manifest sibling.
    let manifest = migrate_manifest_path(destination);
    if paths_refer_to_same_location(report, &manifest) {
        return fail_path_refused(
            "refusing migrate: report path equals migrate-manifest.json location \
             (would overwrite mandatory migrate-manifest)",
        );
    }

    // Codex R5: reparse without `exists()` gate — dangling symlinks report
    // exists()==false but symlink_metadata still detects them.
    if let Err(msg) = refuse_if_reparse(report, is_reparse_or_symlink(report)?) {
        return fail_path_refused(format!("refusing migrate report path: {msg}"));
    }
    // Hardlinks cannot be dangling; only check multi-link when the path exists.
    if report.exists()
        && let Err(msg) = refuse_if_hardlink(report, is_hardlink(report)?)
    {
        return fail_path_refused(format!("refusing migrate report path: {msg}"));
    }
    if let Some(parent) = report.parent()
        && !parent.as_os_str().is_empty()
        && let Err(msg) = refuse_if_reparse(parent, is_reparse_or_symlink(parent)?)
    {
        return fail_path_refused(format!("refusing migrate report parent: {msg}"));
    }

    Ok(())
}

/// M18 / Codex R3 — refuse unsafe migrate-manifest sibling **before** confirm work.
///
/// Catches: source or dest named `migrate-manifest.json` (sibling path collides),
/// existing reparse/symlink at the manifest path, and multi-link manifest targets.
pub fn refuse_unsafe_manifest_path(
    source: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = migrate_manifest_path(destination);
    if paths_refer_to_same_location(&manifest, source) {
        return fail_path_refused(
            "refusing migrate: migrate-manifest.json path equals the source vault file \
             (source must not be named migrate-manifest.json next to destination)",
        );
    }
    if paths_refer_to_same_location(&manifest, destination) {
        return fail_path_refused(
            "refusing migrate: migrate-manifest.json path equals the destination vault file \
             (destination must not be named migrate-manifest.json)",
        );
    }
    // Codex R5: reparse without `exists()` gate (dangling symlink still detected).
    if let Err(msg) = refuse_if_reparse(&manifest, is_reparse_or_symlink(&manifest)?) {
        return fail_path_refused(format!("refusing migrate-manifest path: {msg}"));
    }
    // Hardlinks cannot be dangling; only check multi-link when the path exists.
    if manifest.exists()
        && let Err(msg) = refuse_if_hardlink(&manifest, is_hardlink(&manifest)?)
    {
        return fail_path_refused(format!("refusing migrate-manifest path: {msg}"));
    }
    Ok(())
}

/// Codex R3 / M5–M6 — refuse multi-link destination (hardlink to source/live or any nlink>1).
///
/// Path-string equality alone cannot detect hardlinks; confirm opens dest R/W and would
/// mutate the shared inode.
fn refuse_hardlink_destination(destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !destination.exists() {
        return Ok(());
    }
    if let Err(msg) = refuse_if_hardlink(destination, is_hardlink(destination)?) {
        return fail_path_refused(format!(
            "refusing migrate: destination is a hardlink (multi-link file); \
             writing would mutate shared data (source/live): {msg}"
        ));
    }
    Ok(())
}

/// Refuse writing through an existing hardlinked path (PATH_REFUSED).
fn refuse_hardlink_write_target(
    path: &Path,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(());
    }
    if let Err(msg) = refuse_if_hardlink(path, is_hardlink(path)?) {
        return fail_path_refused(format!("refusing migrate {label} path: {msg}"));
    }
    Ok(())
}

/// M5 final integrity: re-open source RO and compare content fingerprint + size.
///
/// Must run **after** report and migrate-manifest writes so corruption via a
/// hardlinked output path is still detected.
fn verify_source_unmodified_after_writes(
    source: &Path,
    source_key: &SqlCipherKey,
    source_fp_before: &str,
    source_bytes_before: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let re_conn = VaultConnection::open_read_intent(source, source_key).map_err(|e| {
        format!(
            "failed to re-open source vault for integrity check at {}: {e}",
            source.display()
        )
    })?;
    let re_store = SqliteEventStore::new(re_conn);
    let re_events = re_store.read_all_events().map_err(|e| {
        format!(
            "failed to re-read source events for integrity check at {}: {e}",
            source.display()
        )
    })?;
    let source_fp_after = migrate_source_fingerprint(&re_events);
    let source_bytes_after = file_len_or_zero(source);
    if source_fp_after != source_fp_before || source_bytes_after != source_bytes_before {
        return Err(format!(
            "source vault modified during migrate (fingerprint_before={source_fp_before}, \
             fingerprint_after={source_fp_after}, size_before={source_bytes_before}, \
             size_after={source_bytes_after}); this is a bug — aborting after output writes \
             (report/manifest may exist; do not trust them; discard destination)"
        )
        .into());
    }
    Ok(())
}

fn migrate_manifest_path(destination: &Path) -> PathBuf {
    match destination.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join("migrate-manifest.json"),
        _ => PathBuf::from("migrate-manifest.json"),
    }
}

fn delete_dest_artifacts(destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let candidates = [
        destination.to_path_buf(),
        PathBuf::from(format!("{}-wal", destination.display())),
        PathBuf::from(format!("{}-shm", destination.display())),
        migrate_manifest_path(destination),
    ];
    for p in candidates {
        if p.exists() {
            fs::remove_file(&p).map_err(|e| {
                format!(
                    "failed to remove {} during --force-overwrite: {e}",
                    p.display()
                )
            })?;
        }
    }
    Ok(())
}

fn resolve_sql_key(specific: Option<String>, fallback: Option<String>) -> SqlCipherKey {
    let key_str = specific
        .or(fallback)
        .unwrap_or_else(|| DEFAULT_SQL_KEY.to_string());
    SqlCipherKey::from_raw(key_str)
}

fn created_at_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn file_len_or_zero(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn privacy_label(p: Privacy) -> &'static str {
    match p {
        Privacy::CloudOk => "Public",
        Privacy::LocalOnly => "ProjectLocal",
        Privacy::NeverInject => "Private",
        Privacy::Sealed => "Sealed",
    }
}

fn truncate_hash(h: &str) -> String {
    const N: usize = 12;
    if h.len() <= N {
        h.to_string()
    } else {
        format!("{}…", &h[..N])
    }
}

/// Map T147 helper wording (`shadow create`) to migrate when reusing refuse_unsafe_destination.
fn rewrite_refuse_prefix(message: String) -> String {
    message.replace("shadow create", "migrate")
}

fn path_refused(message: String) -> Box<dyn std::error::Error> {
    // fail_api emits Human stderr `CODE: message` and returns Err(GovernedCliError).
    match fail_api(OutputFormat::Human, ApiError::new("PATH_REFUSED", message)) {
        Err(e) => e,
        Ok(()) => Box::new(std::io::Error::other("PATH_REFUSED fail_api returned Ok")),
    }
}

fn fail_path_refused(message: impl Into<String>) -> Result<(), Box<dyn std::error::Error>> {
    fail_api(
        OutputFormat::Human,
        ApiError::new("PATH_REFUSED", message.into()),
    )
}

fn fail_invalid_payload(message: impl Into<String>) -> Result<(), Box<dyn std::error::Error>> {
    fail_api(
        OutputFormat::Human,
        ApiError::new("INVALID_PAYLOAD", message.into()),
    )
}

fn fail_not_found(message: impl Into<String>) -> Result<(), Box<dyn std::error::Error>> {
    fail_api(
        OutputFormat::Human,
        ApiError::new("NOT_FOUND", message.into()),
    )
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_core::ids::MemoryId;
    use ai_brains_core::privacy::Privacy;
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::payload::MemoryPinnedPayload;
    use ai_brains_events::{Actor, AggregateType, Payload};
    use uuid::Uuid;

    fn pin_envelope(content: &str) -> Envelope {
        let memory_id = MemoryId::from_uuid(Uuid::from_u128(42));
        EventBuilder::new(
            AggregateType::Memory,
            memory_id.as_uuid(),
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::MemoryPinned(MemoryPinnedPayload {
            memory_id,
            content: content.into(),
            session_id: None,
            project_id: None,
            tx_id: None,
            rank: None,
            source_tag: None,
            query_text: None,
        }))
        .expect("build pin envelope")
    }

    fn sample_report() -> DifferentialReport {
        DifferentialReport {
            schema_version: 1,
            command: "migrate.governed".into(),
            dry_run: true,
            created_at: "2026-01-01T00:00:00Z".into(),
            source_path: "/tmp/source.db".into(),
            destination_path: "/tmp/dest.db".into(),
            source_fingerprint: "abc".into(),
            live_vault_resolved: false,
            plan_hash: "plan123".into(),
            report_hash: String::new(),
            event_counts: EventCounts {
                source_events: 1,
                dest_events_before: 0,
                dest_events_after: None,
                would_copy_events: 1,
                would_import_appends: 0,
            },
            classification: ClassificationTotals {
                evidence: 0,
                conclusion_candidate: 0,
                decision_proposed: 0,
                review_opened: 0,
                skipped: 0,
                already_imported: 0,
                already_governed: 0,
            },
            unresolved: vec![
                UnresolvedEntry {
                    original_event_id: "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb".into(),
                    reason_code: "missing_scope".into(),
                    kind: "MemoryPinned".into(),
                },
                UnresolvedEntry {
                    original_event_id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
                    reason_code: "unknown_payload".into(),
                    kind: "Unknown".into(),
                },
            ],
            privacy: PrivacySection {
                by_level: BTreeMap::from([("ProjectLocal".into(), 1)]),
                note: "note".into(),
            },
            gaps: GapsSection {
                forgotten_source: 0,
                missing_source: 0,
                missing_scope: 1,
                unknown_payload: 1,
                out_of_matrix: 0,
            },
            content_hashes: vec![],
            content_hashes_truncated: false,
            replay_consistency: ReplayConsistency {
                mode: "count_digest_v1".into(),
                source_event_count: 1,
                plan_action_count: 1,
                ok: true,
                warnings: vec![],
            },
            ce_honesty: CeHonesty {
                claims_cryptographic_erasure: false,
                legacy_plaintext_limitation: "ADR-0016".into(),
            },
            rollback: RollbackSection {
                source_modified: false,
                instructions: vec!["a".into()],
            },
            legacy_import_applied: false,
            t167_plan_hash: "plan123".into(),
            manifest_written: false,
        }
    }

    #[test]
    fn migrate_source_fingerprint__stable_across_reorder() {
        let e1 = pin_envelope("one");
        let e2 = {
            let memory_id = MemoryId::from_uuid(Uuid::from_u128(99));
            EventBuilder::new(
                AggregateType::Memory,
                memory_id.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::MemoryPinned(MemoryPinnedPayload {
                memory_id,
                content: "two".into(),
                session_id: None,
                project_id: None,
                tx_id: None,
                rank: None,
                source_tag: None,
                query_text: None,
            }))
            .expect("e2")
        };
        let a = migrate_source_fingerprint(&[e1.clone(), e2.clone()]);
        let b = migrate_source_fingerprint(&[e2, e1]);
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn report_hash__same_input_same_hash() {
        let r1 = sample_report();
        let h1 = compute_report_hash(&r1).expect("h1");
        let h2 = compute_report_hash(&r1).expect("h2");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn report_hash__reordered_unresolved_same_hash() {
        // compute_report_hash sorts unresolved/content_hashes — order independence without pre-sort.
        let mut r1 = sample_report();
        let mut r2 = sample_report();
        r1.unresolved
            .sort_by(|a, b| a.original_event_id.cmp(&b.original_event_id));
        r2.unresolved
            .sort_by(|a, b| b.original_event_id.cmp(&a.original_event_id));
        assert_ne!(
            r1.unresolved.first().map(|u| u.original_event_id.as_str()),
            r2.unresolved.first().map(|u| u.original_event_id.as_str()),
            "precondition: vectors must start in different order"
        );
        let h1 = compute_report_hash(&r1).expect("h1");
        let h2 = compute_report_hash(&r2).expect("h2");
        assert_eq!(h1, h2);
    }

    #[test]
    fn report_hash__excludes_created_at() {
        let mut r1 = sample_report();
        let mut r2 = sample_report();
        r1.created_at = "2020-01-01T00:00:00Z".into();
        r2.created_at = "2099-12-31T23:59:59Z".into();
        let h1 = compute_report_hash(&r1).expect("h1");
        let h2 = compute_report_hash(&r2).expect("h2");
        assert_eq!(h1, h2);
    }

    #[test]
    fn build_report__empty_source_nonempty_plan__replay_ok_false() {
        use ai_brains_control_plane::{
            ImportAction, ImportActionKind, ImportMechanism, ImportTotals,
        };
        use ai_brains_core::ids::PrincipalId;
        use ai_brains_core::privacy::Privacy;

        let action = ImportAction {
            kind: ImportActionKind::Evidence,
            original_event_id: Uuid::from_u128(1),
            derived_id: "ev-1".into(),
            reason_code: "legacy_pin".into(),
            mechanism: ImportMechanism::WouldAppend,
            source_tag: None,
            unsupported: None,
            content: None,
            title: None,
            statement: None,
            privacy: Privacy::LocalOnly,
            scope_key: None,
            evidence_ids: Vec::new(),
            related_decision_id: None,
            original_memory_id: None,
            session_id: None,
        };
        let plan = ImportPlan {
            actions: vec![action],
            totals: ImportTotals {
                evidence: 1,
                ..ImportTotals::default()
            },
            plan_hash: "plan-test".into(),
            principal_id: PrincipalId::from_uuid(Uuid::nil()),
            command_id: None,
            dry_run: true,
        };
        let report = build_report(BuildReportArgs {
            is_dry_run: true,
            source: Path::new("/tmp/source.db"),
            destination: Path::new("/tmp/dest.db"),
            source_fingerprint: "fp",
            live_vault_resolved: false,
            plan: &plan,
            events: &[],
            dest_events_before: 0,
            dest_events_after: None,
            would_copy_events: 0,
            would_import_appends: 1,
            legacy_import_applied: false,
            manifest_written: false,
            source_modified: false,
            apply_report: None,
        })
        .expect("build_report");
        assert!(
            !report.replay_consistency.ok,
            "empty source + non-empty plan must set replay_consistency.ok false"
        );
        assert!(
            report
                .replay_consistency
                .warnings
                .iter()
                .any(|w| w.contains("empty source")),
            "expected empty-source warning, got {:?}",
            report.replay_consistency.warnings
        );
    }

    #[test]
    fn refuse_unsafe_report_path__equals_source__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("dest.db");
        fs::write(&source, b"x").expect("source");
        let err = refuse_unsafe_report_path(&source, &source, &dest).expect_err("must refuse");
        assert!(
            err.to_string().contains("report path equals the source"),
            "got: {err}"
        );
    }

    #[test]
    fn refuse_unsafe_report_path__equals_dest__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("dest.db");
        fs::write(&dest, b"x").expect("dest");
        let err = refuse_unsafe_report_path(&dest, &source, &dest).expect_err("must refuse");
        assert!(
            err.to_string()
                .contains("report path equals the destination"),
            "got: {err}"
        );
    }

    #[test]
    fn refuse_unsafe_report_path__equals_migrate_manifest__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("dest").join("dest.db");
        let report = migrate_manifest_path(&dest);
        let err = refuse_unsafe_report_path(&report, &source, &dest).expect_err("must refuse");
        assert!(err.to_string().contains("migrate-manifest"), "got: {err}");
    }

    /// Report path hardlinked to source (or any multi-link inode) must refuse
    /// before `File::create` can truncate shared data (Codex R2 P1-02).
    #[test]
    fn refuse_unsafe_report_path__hardlink_to_source__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("dest.db");
        let report = dir.path().join("report.json");
        fs::write(&source, b"vault-bytes").expect("source");
        fs::hard_link(&source, &report).expect("hardlink report -> source");
        assert!(
            is_hardlink(&report).expect("nlink"),
            "precondition: report must be hardlink"
        );
        let err = refuse_unsafe_report_path(&report, &source, &dest).expect_err("must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains("hardlink") || msg.contains("link count") || msg.contains("PATH_REFUSED"),
            "expected hardlink refuse, got: {msg}"
        );
    }

    #[test]
    fn refuse_hardlink_write_target__hardlinked_manifest__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let manifest = dir.path().join("migrate-manifest.json");
        fs::write(&source, b"vault-bytes").expect("source");
        fs::hard_link(&source, &manifest).expect("hardlink manifest -> source");
        let err = refuse_hardlink_write_target(&manifest, "migrate-manifest")
            .expect_err("must refuse hardlinked manifest");
        let msg = err.to_string();
        assert!(
            msg.contains("hardlink") || msg.contains("link count") || msg.contains("PATH_REFUSED"),
            "expected hardlink refuse, got: {msg}"
        );
    }

    /// Dest hardlinked to source must refuse (Codex R3) — path equality alone is insufficient.
    #[test]
    fn refuse_hardlink_destination__hardlink_to_source__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("dest.db");
        fs::write(&source, b"vault-bytes").expect("source");
        fs::hard_link(&source, &dest).expect("hardlink dest -> source");
        assert!(
            is_hardlink(&dest).expect("nlink"),
            "precondition: dest must be hardlink"
        );
        let err = refuse_hardlink_destination(&dest).expect_err("must refuse hardlinked dest");
        let msg = err.to_string();
        assert!(
            msg.contains("hardlink") || msg.contains("link count") || msg.contains("PATH_REFUSED"),
            "expected hardlink dest refuse, got: {msg}"
        );
    }

    #[test]
    fn refuse_hardlink_destination__regular_file__ok() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dest = dir.path().join("dest.db");
        fs::write(&dest, b"vault-bytes").expect("dest");
        refuse_hardlink_destination(&dest).expect("single-link dest must be allowed");
    }

    #[test]
    fn refuse_hardlink_destination__missing__ok() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dest = dir.path().join("missing.db");
        refuse_hardlink_destination(&dest).expect("missing dest must be allowed");
    }

    /// Source named migrate-manifest.json next to dest → sibling path == source (Codex R3).
    #[test]
    fn refuse_unsafe_manifest_path__equals_source__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("migrate-manifest.json");
        let dest = dir.path().join("vault.db");
        fs::write(&source, b"x").expect("source");
        let err = refuse_unsafe_manifest_path(&source, &dest).expect_err("must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains("source") && msg.contains("migrate-manifest"),
            "got: {msg}"
        );
    }

    /// Dest named migrate-manifest.json → manifest path == dest vault file (Codex R3).
    #[test]
    fn refuse_unsafe_manifest_path__equals_destination__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("migrate-manifest.json");
        fs::write(&dest, b"x").expect("dest");
        let err = refuse_unsafe_manifest_path(&source, &dest).expect_err("must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains("destination") && msg.contains("migrate-manifest"),
            "got: {msg}"
        );
    }

    #[test]
    fn refuse_unsafe_manifest_path__normal_sibling__ok() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("dest").join("vault.db");
        fs::write(&source, b"x").expect("source");
        refuse_unsafe_manifest_path(&source, &dest).expect("normal paths must pass");
    }

    #[test]
    fn refuse_unsafe_manifest_path__hardlinked_manifest__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("dest").join("vault.db");
        let manifest = migrate_manifest_path(&dest);
        fs::create_dir_all(dest.parent().expect("parent")).expect("mkdir");
        fs::write(&source, b"vault-bytes").expect("source");
        fs::hard_link(&source, &manifest).expect("hardlink manifest -> source");
        let err = refuse_unsafe_manifest_path(&source, &dest).expect_err("must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains("hardlink") || msg.contains("link count") || msg.contains("PATH_REFUSED"),
            "expected hardlink refuse, got: {msg}"
        );
    }

    /// Create a file symlink whose target does not exist (dangling).
    /// Returns `None` when the OS denies symlink creation (Windows without Developer Mode).
    fn try_dangling_file_symlink(link: &Path) -> Option<()> {
        let missing_target = link.with_extension("missing-target-does-not-exist");
        #[cfg(windows)]
        let created = std::os::windows::fs::symlink_file(&missing_target, link);
        #[cfg(not(windows))]
        let created = std::os::unix::fs::symlink(&missing_target, link);
        match created {
            Ok(()) => {
                // Precondition: dangling → exists() false, reparse true.
                assert!(
                    !link.exists(),
                    "precondition: dangling symlink must have exists()==false"
                );
                assert!(
                    is_reparse_or_symlink(link).expect("symlink_metadata"),
                    "precondition: dangling symlink must be detected as reparse"
                );
                Some(())
            }
            Err(e) => {
                eprintln!(
                    "skipping dangling-symlink unit test (symlink create failed: {e}; \
                     needs Developer Mode or elevation on Windows)"
                );
                None
            }
        }
    }

    /// Codex R5 — dangling symlink report path must refuse without exists() gate.
    #[test]
    fn refuse_unsafe_report_path__dangling_symlink__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("dest.db");
        let report = dir.path().join("report.json");
        fs::write(&source, b"x").expect("source");
        if try_dangling_file_symlink(&report).is_none() {
            return;
        }
        let err = refuse_unsafe_report_path(&report, &source, &dest).expect_err("must refuse");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("reparse")
                || msg.contains("symlink")
                || msg.contains("junction")
                || msg.contains("path_refused"),
            "expected reparse refuse for dangling report symlink, got: {err}"
        );
    }

    /// Codex R5 — dangling symlink at migrate-manifest sibling must refuse.
    #[test]
    fn refuse_unsafe_manifest_path__dangling_symlink__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("dest").join("vault.db");
        let manifest = migrate_manifest_path(&dest);
        fs::create_dir_all(dest.parent().expect("parent")).expect("mkdir");
        fs::write(&source, b"x").expect("source");
        if try_dangling_file_symlink(&manifest).is_none() {
            return;
        }
        let err = refuse_unsafe_manifest_path(&source, &dest).expect_err("must refuse");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("reparse")
                || msg.contains("symlink")
                || msg.contains("junction")
                || msg.contains("path_refused"),
            "expected reparse refuse for dangling manifest symlink, got: {err}"
        );
    }
}
