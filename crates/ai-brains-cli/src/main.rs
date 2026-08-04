mod artifact_security;
mod commands;
mod context;
mod daemon_client;
mod daemon_probe;
mod elevation;
mod help_ia;
mod key_resolve;
mod live_graph;

/// JSON Schema for `ai-bbrains agy-hook --payload`. Bundled at compile time
/// so `--schema` works regardless of cwd. The source-of-truth file lives at
/// `Docs/schemas/agy-hook-payload.json`; changes there must be mirrored here.
const SCHEMA_AGY_HOOK: &str = include_str!("../../../Docs/schemas/agy-hook-payload.json");

/// JSON Schema for the NDJSON records consumed by `ai-bbrains sync pull --from-file`.
/// Source-of-truth at `Docs/schemas/sync-pull-record.json`.
const SCHEMA_SYNC_PULL: &str = include_str!("../../../Docs/schemas/sync-pull-record.json");

/// Print an embedded JSON Schema to stdout and exit 0. The schemas are
/// included at compile time so the binary is self-contained.
fn print_schema(schema: &str, _title: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Pretty-print so users can read it directly. The audit required that
    // the output be valid JSON (consumers can pipe to jq).
    let parsed: serde_json::Value = serde_json::from_str(schema)
        .map_err(|e| format!("Embedded schema is not valid JSON: {}", e))?;
    println!("{}", serde_json::to_string_pretty(&parsed)?);
    Ok(())
}

use crate::context::AppContext;
use ai_brains_core::ids::{ProjectId, SessionId};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    #[test]
    #[allow(non_snake_case)]
    fn log_format_prescan__minimal__recognized() {
        let args = ["--log-format", "minimal"];
        let format = args
            .windows(2)
            .find(|w| w[0] == "--log-format")
            .map(|w| w[1].to_string())
            .unwrap_or_else(|| "compact".to_string());
        assert_eq!(format, "minimal");
    }
}

#[derive(Parser)]
#[command(name = "ai-brains")]
#[command(version)]
#[command(about = "AI-Brains CLI", long_about = None)]
#[command(after_long_help = help_ia::ROOT_AFTER_LONG_HELP)]
#[command(after_help = help_ia::ROOT_AFTER_HELP_TIP)]
struct Cli {
    /// Boxed so Windows debug stacks can parse the large clap `Commands` enum (T192).
    #[command(subcommand)]
    command: Box<Commands>,

    /// Path to the vault database
    #[arg(long, env = "AI_BRAINS_VAULT_PATH", help_heading = "Global options")]
    vault_path: Option<PathBuf>,

    /// Hex-encoded key for the vault (or dummy)
    #[arg(long, env = "AI_BRAINS_KEY", help_heading = "Global options")]
    key: Option<String>,

    /// Skip auto-discovery of project/session from .env. When set, the CLI
    /// will not clear inherited `AI_BRAINS_PROJECT_ID` / `AI_BRAINS_SESSION_ID`
    /// env vars or load a project-local `.env` file. Use this in CI, hooks,
    /// or any non-interactive flow where the caller has already configured
    /// the env vars explicitly.
    #[arg(long, global = true, help_heading = "Global options")]
    no_project_context: bool,

    /// Tracing output format: compact (default), full, json, minimal, or off
    #[arg(
        long,
        global = true,
        default_value = "compact",
        help_heading = "Global options"
    )]
    log_format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new vault
    #[command(display_order = 0)]
    Init {
        /// Re-initialize even when the vault already contains data
        #[arg(long)]
        force: bool,
    },
    /// Ingest a conversation turn (reads JSON from stdin)
    #[command(display_order = 50)]
    Ingest {
        /// Preview what would be ingested without writing to the vault
        #[arg(long)]
        dry_run: bool,
    },
    /// Recall memories based on a query
    #[command(display_order = 10)]
    Recall {
        /// Query string, or `-` to read from stdin
        query: String,
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        #[arg(long = "session")]
        session_id: Option<SessionId>,
        /// Optional partial/short session ID prefix to resolve against the vault.
        /// Conflicts with --session-last.
        #[arg(long, conflicts_with = "session_last")]
        session_prefix: Option<String>,
        /// Output format: 'json' or 'pretty' (default: pretty on TTY, json otherwise)
        #[arg(long)]
        format: Option<String>,
        /// Use semantic (embedding) search alongside FTS5
        #[arg(long)]
        semantic: bool,
        /// Score boost added to graph-neighbor hits (default 0.1)
        #[arg(long, default_value_t = 0.1)]
        graph_boost: f64,
        /// Hop depth for graph expansion (reserved; currently only depth=1)
        #[arg(long, default_value_t = 1)]
        graph_hop_depth: usize,
        /// Suppress non-fatal warnings (e.g., bridge-failed notices when
        /// the cwd is not a git repository). Useful for non-interactive
        /// scripts and CI runs.
        #[arg(long)]
        quiet: bool,
        /// Skip the Ledgerful bridge query and use only local vault FTS5 +
        /// semantic search. Guarantees vault memories appear in results.
        #[arg(long)]
        no_bridge: bool,
        /// Search across all projects, ignoring AI_BRAINS_PROJECT_ID
        #[arg(long)]
        global: bool,
        /// Use the most recent active session for recall.
        #[arg(long, conflicts_with = "session_id", conflicts_with = "session_prefix")]
        session_last: bool,
    },
    /// Generate preflight context for an LLM
    #[command(display_order = 11)]
    Preflight {
        #[arg(short, long, default_value_t = 1500)]
        max_words: usize,
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        /// Output human-readable text instead of JSON
        #[arg(long)]
        pretty: bool,
        /// Output format: 'json' or 'human'
        #[arg(long)]
        format: Option<String>,
        /// Comma-separated target file/directory paths for contextual risk analysis
        #[arg(long, env = "AI_BRAINS_SCOPE", value_delimiter = ',')]
        scope: Vec<String>,
        /// Output a concise statistical summary instead of full text
        #[arg(short, long)]
        summary: bool,
        /// Aggregate context across ALL projects (ignores project_id filter)
        #[arg(long)]
        global: bool,
        /// Read options from stdin as JSON `{"scope":[...],"max_words":N}` instead of CLI flags
        #[arg(long)]
        stdin: bool,
    },
    /// Run nightly intelligence sweep
    #[command(display_order = 26)]
    Nightly {
        /// Schedule this as a Windows scheduled task
        #[arg(long)]
        schedule: bool,
        /// Remove the Windows scheduled task
        #[arg(long)]
        unschedule: bool,
        /// Start time for the scheduled task (e.g. "03:00")
        #[arg(long, default_value = "03:00")]
        start_time: String,
        /// Show read-only status of the last nightly run and pending work
        #[arg(long, conflicts_with = "schedule", conflicts_with = "unschedule")]
        status: bool,
        /// Skip the Antigravity session import. Use this on isolated, CI,
        /// or per-project vaults to prevent cross-vault contamination
        /// from the user's real Antigravity history.
        #[arg(long)]
        skip_import: bool,
        /// Schedule the task to run as SYSTEM (no login required). Requires elevation.
        #[arg(long)]
        run_as_system: bool,
        /// Preview the scheduling command without registering the task
        #[arg(long)]
        dry_run: bool,
    },
    /// Create a timestamped backup of the vault
    #[command(display_order = 20)]
    Backup {
        #[command(subcommand)]
        command: Option<BackupCommands>,
        /// Preview what would happen without creating the backup file.
        /// Only applies when no subcommand is given (defaults to create).
        #[arg(long)]
        dry_run: bool,
    },
    /// Recovery kit export (operator offline key recovery)
    #[command(display_order = 21)]
    Recovery {
        #[command(subcommand)]
        command: RecoveryCommands,
    },
    /// Read-only operator health report (vault / cipher / backup / recoverability / daemon)
    #[command(
        display_order = 12,
        after_help = "Read-only: no migrate, no vault/backups create, no secrets on stdout. Does not replace RECOVERY-DRILLS. Offline kit residual without --kit-path is operator responsibility. Daemon probe = our IPC only. --backup-max-age uses Nd/Nh/Nw. No --passphrase argv.\nKey bootstrap: set --key or AI_BRAINS_KEY as x'<64 hex>' (see Docs/INSTALL.md). Missing key → vault_open skipped; wrong key → vault_open fail.\nExamples:\n  ai-brains doctor\n  ai-brains doctor --json\n  ai-brains doctor --kit-path ./kit.json --passphrase-file ./pw.txt\n  ai-brains doctor --fail-on-degraded --backup-max-age 14d --full"
    )]
    Doctor {
        /// Output format: human (default) or json
        #[arg(long, default_value = "human", value_parser = ["human", "json"])]
        format: String,
        /// Force JSON output (overrides --format)
        #[arg(long)]
        json: bool,
        /// Exit 1 when overall status is degraded (default exit 0 for degraded)
        #[arg(long)]
        fail_on_degraded: bool,
        /// Offline RecoveryKit path to unlock and compare to vault key
        #[arg(long)]
        kit_path: Option<PathBuf>,
        /// Passphrase file for --kit-path unlock (no --passphrase argv)
        #[arg(long)]
        passphrase_file: Option<PathBuf>,
        /// Max age for newest backup (Nd/Nh/Nw; default 7d)
        #[arg(long, default_value = "7d")]
        backup_max_age: String,
        /// Run PRAGMA integrity_check (slow path)
        #[arg(long)]
        full: bool,
    },
    /// [dangerous] Forget a specific memory (soft delete)
    #[command(display_order = 40)]
    Forget {
        /// Memory ID to forget
        #[arg(long)]
        memory_id: Option<String>,
        /// Search for memories by content match
        #[arg(long = "match")]
        match_query: Option<String>,
        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,
        /// List all forgotten memories
        #[arg(long)]
        list_forgotten: bool,
        /// Restore a forgotten memory
        #[arg(long)]
        restore: Option<String>,
        /// Preview what would be forgotten without modifying the vault
        #[arg(long)]
        dry_run: bool,
    },
    /// Stop an active session
    #[command(display_order = 16)]
    StopSession {
        /// Session ID to stop
        session_id: String,
    },
    /// Initialize or refresh the project context (writes local .env)
    #[command(display_order = 15)]
    Context {
        /// Force a fresh project ID even if one is detected
        #[arg(long)]
        new_project: bool,
        /// Force a new session ID, replacing the existing one
        #[arg(long)]
        new_session: bool,
        /// Show current context without modifying anything
        #[arg(long)]
        show: bool,
        /// Optional Ledgerful transaction ID to link this context to
        #[arg(long, env = "LEDGERFUL_TX_ID")]
        tx_id: Option<String>,
    },
    /// Pin a high-level decision or constraint directly to the vault
    #[command(display_order = 14)]
    Pin {
        /// The content to pin (e.g., "DECISION: Switched to SQLite")
        content: Option<String>,
        /// The role to associate with this pin (default: assistant)
        #[arg(long, default_value = "assistant")]
        role: String,
        /// Privacy level (default: LocalOnly)
        #[arg(long, default_value = "LocalOnly")]
        privacy: String,
        /// Read content from stdin instead of positional arg
        #[arg(long)]
        stdin: bool,
        /// Tags to categorize this memory (repeatable)
        #[arg(long = "tag", short = 't')]
        tags: Vec<String>,
        /// Optional Ledgerful transaction ID to link this pin to
        #[arg(long, env = "LEDGERFUL_TX_ID")]
        tx_id: Option<String>,
        /// Preview what would be pinned without writing to the vault
        #[arg(long)]
        dry_run: bool,
    },
    /// Manage repository safety signals
    #[command(display_order = 27)]
    Safety {
        #[command(subcommand)]
        command: SafetyCommands,
    },
    /// Sync structured records from external tools (Ledgerful)
    #[command(display_order = 53)]
    Sync {
        #[command(subcommand)]
        command: SyncCommands,
    },
    /// Import Antigravity conversation logs into the vault
    #[command(display_order = 51)]
    AntigravityImport {
        /// Only import sessions modified within the last N days
        #[arg(short, long, default_value_t = 30)]
        days: usize,
    },
    /// Process an Antigravity CLI (agy) hook payload
    #[command(display_order = 52)]
    AgyHook {
        /// The JSON payload from agy
        #[arg(long)]
        payload: Option<String>,
        /// Print the JSON Schema for the expected `--payload` shape and exit.
        /// The schema is also at `Docs/schemas/agy-hook-payload.json`.
        #[arg(long)]
        schema: bool,
    },
    /// Manage the AI-Brains daemon process
    #[command(display_order = 17)]
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
    /// Manage projects and resolve aliases
    #[command(display_order = 13)]
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    /// Graph operations
    #[cfg(feature = "graph")]
    #[command(display_order = 57)]
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
    /// Graph operations (requires --features graph)
    #[cfg(not(feature = "graph"))]
    #[command(display_order = 57)]
    Graph {
        #[command(subcommand)]
        command: GraphCommands,

        /// Trailing arguments accepted when the graph feature is not enabled
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Create a shadow vault copy for safe dogfood evaluation
    #[command(display_order = 54)]
    Shadow {
        #[command(subcommand)]
        command: ShadowCommands,
    },
    /// Governed migrate: classify legacy events, optional dest materialize, differential report (T168)
    ///
    /// Defaults to dry-run (report only). Pass `--confirm` to materialize destination + apply T167 import.
    /// Destination safety reuses T147 shadow refusals (live vault / parent / reparse). Source is never
    /// migrated. Report has no plaintext bodies.
    #[command(
        display_order = 58,
        after_help = "Examples:\n  ai-brains migrate governed --source ./src.db --destination ./dest.db --report ./report.json\n  ai-brains migrate governed --source ./src.db --destination ./dest.db --report ./report.json --confirm"
    )]
    Migrate {
        #[command(subcommand)]
        command: Box<MigrateCommands>,
    },
    /// Evaluate governed-memory trust scenarios (T169). Hermetic tempfile vaults only.
    ///
    /// Exit: 0 hard pass; 1 internal/path refuse; 6 invalid payload; 7 hard-gate fail.
    /// Soft metric misses do not fail unless `--strict-soft`. Never mutates live vault.
    #[command(
        display_order = 55,
        after_help = "Examples:\n  ai-brains evaluate governed --fixtures fixtures/governed-memory/scenarios\n  ai-brains evaluate governed --fixtures ./scenarios --report ./evaluate-report.json"
    )]
    Evaluate {
        #[command(subcommand)]
        command: EvaluateCommands,
    },
    /// Dogfood helpers (T170): pure-serde compare of governed briefing vs legacy preflight.
    ///
    /// Never opens a vault. Never mutates live. Use with `--vault-path` capture inputs only (D26).
    #[command(
        display_order = 56,
        after_help = "Examples:\n  ai-brains dogfood compare --governed packet.json --legacy preflight.json --out dogfood-compare.json --stage B"
    )]
    Dogfood {
        #[command(subcommand)]
        command: DogfoodCommands,
    },
    /// Build typed Project / Personal briefing packets (T152)
    ///
    /// Empty-state contract: denied/unresolved scopes return a packet with
    /// `denied=true` or empty authority sections + warnings. Default format is
    /// markdown on TTY and json otherwise (`--format` wins).
    /// Principal: `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` or well-known System principal
    /// (must be registered + granted). See `AI_BRAINS_GOVERNED_BRIEFING` for preflight.
    #[command(
        display_order = 31,
        after_help = "Examples:\n  ai-brains briefing project --format json --max-words 1500 --project-id <uuid>\n  ai-brains briefing personal --format json\n  # or set AI_BRAINS_PROJECT_ID for project briefing"
    )]
    Briefing {
        #[command(subcommand)]
        command: BriefingCommands,
    },
    /// Governed progressive query, handle expand, and query-trace retrieval (T152)
    #[command(
        display_order = 32,
        after_help = "Examples:\n  ai-brains query progressive \"why was graph backend replaced?\" --project-id <uuid>\n  ai-brains query expand <handle-id> --project-id <uuid>\n  ai-brains query trace <trace-id>\n  # or set AI_BRAINS_PROJECT_ID"
    )]
    Query {
        #[command(subcommand)]
        command: GovernedQueryCommands,
    },
    /// Resolve the active governed scope (T160 / #20)
    ///
    /// Always surfaces authoritative, confidence, warnings, and alternatives.
    #[command(
        display_order = 30,
        after_help = "Examples:\n  ai-brains scope resolve --format json"
    )]
    Scope {
        #[command(subcommand)]
        command: ScopeCommands,
    },
    /// Evidence discovery and handle previews (T160 / T203)
    #[command(
        display_order = 33,
        after_help = "Examples:\n  ai-brains evidence list --scope Repository:<uuid>\n  ai-brains evidence list --format json\n  ai-brains evidence show <id> --scope Repository:<uuid> --format json"
    )]
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommands,
    },
    /// Source registry discovery and inspect (T160 / T203)
    #[command(
        display_order = 34,
        after_help = "Examples:\n  ai-brains source list --scope Repository:<uuid>\n  ai-brains source list --format json\n  ai-brains source show <id> --scope Repository:<uuid>"
    )]
    Source {
        #[command(subcommand)]
        command: SourceCommands,
    },
    /// Propose conclusions (T160)
    #[command(
        display_order = 37,
        after_help = "Examples:\n  ai-brains conclusion propose --claim \"...\" --evidence <id> --scope Repository:<uuid>"
    )]
    Conclusion {
        #[command(subcommand)]
        command: ConclusionCommands,
    },
    /// Propose decisions (T160)
    #[command(
        display_order = 38,
        after_help = "Examples:\n  ai-brains decision propose --statement \"...\" --scope Repository:<uuid>"
    )]
    Decision {
        #[command(subcommand)]
        command: DecisionCommands,
    },
    /// Review queue list / resolve (T160 / T203 soft-default scope)
    #[command(
        display_order = 35,
        after_help = "Examples:\n  ai-brains review list --scope Repository:<uuid>\n  ai-brains review list --format json\n  ai-brains review resolve <id> --resolution approved --scope Repository:<uuid>"
    )]
    Review {
        #[command(subcommand)]
        command: ReviewCommands,
    },
    /// Policy grant inspection (read-only, T160)
    #[command(
        display_order = 36,
        after_help = "Examples:\n  ai-brains policy show --scope Repository:<uuid>\n  ai-brains policy check --capability ProposeConclusion --scope Repository:<uuid>"
    )]
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },
    /// [dangerous] Erasure tickets + content-envelope wipe (daemon-required) (T160/T165)
    #[command(
        display_order = 41,
        after_help = "Examples:\n  ai-brains erasure request --id <id> --scope Repository:<uuid> --format json\n  ai-brains erasure wipe --content-key-id <uuid> --scope Repository:<uuid> --confirm"
    )]
    Erasure {
        #[command(subcommand)]
        command: ErasureCommands,
    },
    /// Class-based retention plan/apply (T166 / P8.4)
    #[command(
        display_order = 23,
        after_help = "Examples:\n  ai-brains retention plan --format json\n  ai-brains retention apply --confirm --format json\n  ai-brains retention apply --confirm --scope Repository:<uuid> --format json\nHonesty: projection delete ≠ CE; CE reuses erasure wipe path for envelope classes only; CE apply requires --scope."
    )]
    Retention {
        #[command(subcommand)]
        command: RetentionCommands,
    },
    /// Multi-device enrollment (T176 / ADR-0018). Optional; not PQ; not remote wipe; not metadata-private.
    /// Does **not** repurpose `sync` (Ledgerful) or `safety sync` (hotspot pin).
    #[command(
        display_order = 24,
        after_help = "Examples:\n  ai-brains device bootstrap\n  ai-brains device list\n  ai-brains device fingerprint\n  ai-brains device package-export --out peer.bin\n  ai-brains device enroll --package peer.bin --yes\n  ai-brains device revoke <device-id>\nHonesty: multi-device is optional; classical ECC only (not PQ); ACK ≠ wipe proof; padding ≠ metadata privacy."
    )]
    Device {
        #[command(subcommand)]
        command: DeviceCommands,
    },
    /// Multi-device replication status / cursors / push / pull (T177 file fake relay only).
    #[command(
        display_order = 25,
        after_help = "Examples:\n  ai-brains replicate status\n  ai-brains replicate cursors\n  ai-brains replicate push --fake-relay ./relay\n  ai-brains replicate pull --fake-relay ./relay\nEnv: AI_BRAINS_SYNC_FAKE_RELAY_PATH\nNo `replicate sync` alias — run push then pull.\nHonesty: optional multi-device; not PQ; not remote wipe; not metadata-private."
    )]
    Replicate {
        #[command(subcommand)]
        command: ReplicateCommands,
    },
    /// Vault operator tools (T187): plain→SQLCipher encrypt via sqlcipher_export
    #[command(
        display_order = 22,
        after_help = "Examples:\n  ai-brains vault encrypt --vault-path ./plain.db --dry-run\n  ai-brains vault encrypt --vault-path ./plain.db --destination ./enc.db --key \"x'...'\"\n  ai-brains vault encrypt --vault-path ./plain.db --confirm --key \"x'...'\"\nHonesty: not FIPS; not NIST Purge; Online Backup is not used for plain→encrypt."
    )]
    Vault {
        #[command(subcommand)]
        command: VaultCommands,
    },
}

#[derive(Subcommand, Clone)]
enum VaultCommands {
    /// [dangerous] Convert a plaintext SQLite vault to SQLCipher page encryption (sqlcipher_export).
    Encrypt {
        /// Source plaintext vault (defaults to --vault-path / AI_BRAINS_VAULT_PATH)
        #[arg(long)]
        source: Option<PathBuf>,
        /// Destination encrypted path (non-destructive). Conflicts with silent default when omitted.
        #[arg(long)]
        destination: Option<PathBuf>,
        /// Replace source in place after export (moves plain aside to *.bak-plain). Required for in-place.
        #[arg(long)]
        confirm: bool,
        /// Preview only; never write (default when neither --destination nor --confirm)
        #[arg(long)]
        dry_run: bool,
    },
    /// [dangerous] Rotate vault DataKey (KEK) + SQLCipher page key (T189 / ADR-0020).
    #[command(
        after_help = "Safety (non-overridable):\n  - Daemon up → mutating rotate hard-fails (stop daemon first)\n  - --overwrite-kit only overwrites the kit file; never overrides daemon or backup gates\n  - Primary path: crash-safe sqlcipher_export; --accept-rekey-risk enables in-place PRAGMA rekey\n  - Mandatory --kit-output RecoveryKit for the NEW key; verify unlock before retiring old kits\nExamples:\n  ai-brains vault rotate-datakey --dry-run\n  ai-brains vault rotate-datakey --confirm --kit-output ./kit-new.json --passphrase-file ./pw.txt --i-have-backup \"I have a backup\"\nHonesty: multi-device peers need their own ceremony; peer wraps untouched; not NIST Purge of offline backups."
    )]
    RotateDatakey {
        /// Preview living wrap count + device-private 0|1; no mutation
        #[arg(long)]
        dry_run: bool,
        /// Required for non-dry-run apply
        #[arg(long)]
        confirm: bool,
        /// Require a recent verified backup (default true). `--require-backup=false` alone does not bypass; use `--i-have-backup "I have a backup"`.
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        require_backup: bool,
        /// Exact phrase bypass for backup gate: `I have a backup` (sets backup_bypassed on event)
        #[arg(long)]
        i_have_backup: Option<String>,
        /// Path for NEW RecoveryKit JSON (required on success)
        #[arg(long)]
        kit_output: Option<PathBuf>,
        /// Passphrase file for kit (or TTY double-entry)
        #[arg(long)]
        passphrase_file: Option<PathBuf>,
        /// Allow overwriting existing kit file only
        #[arg(long)]
        overwrite_kit: bool,
        /// Opt-in in-place PRAGMA rekey (not crash-safe; snapshot + auto-restore)
        #[arg(long)]
        accept_rekey_risk: bool,
        /// Print NEW SqlCipher key to stdout (default off)
        #[arg(long)]
        print_key: bool,
        /// Backup directory for gate (default: sibling `backups/` of vault)
        #[arg(long)]
        backup_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "First device: bootstrap. Peers: package-export on new machine → enroll on enrolled vault (OOB fingerprint)."
)]
enum DeviceCommands {
    /// First-device local enroll (status=local, self enrolled_by). Fails if any active/local exists.
    Bootstrap,
    /// Print dual-key fingerprint (R24 hyphen groups; --raw for plain hex)
    Fingerprint {
        /// Emit raw lowercase hex without hyphens
        #[arg(long)]
        raw: bool,
    },
    /// List enrolled devices (active + local)
    List,
    /// Generate keys + write enrollment package (new machine; does not enroll into a peer vault)
    PackageExport {
        /// Output path for the enrollment package bytes (public only by default)
        #[arg(long)]
        out: PathBuf,
        /// Optional path for OS-protected private seeds (Windows: DPAPI). Never raw seed files.
        #[arg(long)]
        write_private_key: Option<PathBuf>,
    },
    /// Enroll a peer from package on an already-enrolled vault (confirm fingerprint OOB)
    Enroll {
        /// Path to enrollment package from package-export
        #[arg(long)]
        package: PathBuf,
        /// Skip interactive yes confirmation (still prints fingerprint)
        #[arg(long)]
        yes: bool,
    },
    /// Revoke + permanently tombstone a device; delete peer wraps for recipient (R23)
    Revoke {
        /// Device id (UUID) to revoke
        device_id: String,
    },
}

#[derive(Subcommand, Clone)]
enum ReplicateCommands {
    /// Local cursors, gap/blocked state, enrolled count; relay file path or not configured
    Status {
        /// Explicit file fake relay directory (or set AI_BRAINS_SYNC_FAKE_RELAY_PATH)
        #[arg(long)]
        fake_relay: Option<PathBuf>,
        /// Emit JSON status
        #[arg(long)]
        format: Option<String>,
        /// Minimal output (relay line only)
        #[arg(long)]
        quiet: bool,
    },
    /// Dump replication_cursor rows
    Cursors {
        /// Emit JSON
        #[arg(long)]
        format: Option<String>,
    },
    /// Push pending envelopes to an explicitly configured file fake relay (no sockets)
    Push {
        /// Explicit file fake relay directory (or set AI_BRAINS_SYNC_FAKE_RELAY_PATH)
        #[arg(long)]
        fake_relay: Option<PathBuf>,
        /// Output format: text (default) or json
        #[arg(long)]
        format: Option<String>,
        /// Suppress success chatter (text mode only)
        #[arg(long)]
        quiet: bool,
    },
    /// Pull peer envelopes from an explicitly configured file fake relay (no sockets)
    Pull {
        /// Explicit file fake relay directory (or set AI_BRAINS_SYNC_FAKE_RELAY_PATH)
        #[arg(long)]
        fake_relay: Option<PathBuf>,
        /// Output format: text (default) or json
        #[arg(long)]
        format: Option<String>,
        /// Suppress success chatter (text mode only)
        #[arg(long)]
        quiet: bool,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains briefing project --format json --max-words 1500 --project-id <uuid>\n  ai-brains briefing personal --format json\n  # or set AI_BRAINS_PROJECT_ID for project briefing"
)]
enum BriefingCommands {
    /// Build a Project Briefing packet (policy → lifecycle → authority)
    #[command(
        after_help = "Examples:\n  ai-brains briefing project --format json --max-words 1500 --project-id <uuid>\n  # or set AI_BRAINS_PROJECT_ID"
    )]
    Project {
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        #[arg(short, long, default_value_t = 1500)]
        max_words: usize,
        /// Skip BriefingGenerated event / cache write (default: true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        dry_run: bool,
        /// Output format: `json` or `markdown` (default: markdown on TTY, json otherwise)
        #[arg(long)]
        format: Option<String>,
    },
    /// Build a Personal Continuity Briefing packet
    #[command(after_help = "Examples:\n  ai-brains briefing personal --format json")]
    Personal {
        /// Personal user id (defaults to principal UUID mapping)
        #[arg(long)]
        user_id: Option<String>,
        #[arg(short, long, default_value_t = 800)]
        max_words: usize,
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        dry_run: bool,
        /// Output format: `json` or `markdown` (default: markdown on TTY, json otherwise)
        #[arg(long)]
        format: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains query progressive \"why was graph backend replaced?\" --project-id <uuid>\n  ai-brains query expand <handle-id> --project-id <uuid>\n  ai-brains query trace <trace-id>\n  # or set AI_BRAINS_PROJECT_ID"
)]
enum GovernedQueryCommands {
    /// Run a governed progressive query (JSON ProgressiveQueryResponse)
    #[command(
        after_help = "Examples:\n  ai-brains query progressive \"why was graph backend replaced?\" --project-id <uuid>\n  # or set AI_BRAINS_PROJECT_ID"
    )]
    Progressive {
        /// Query text
        query: String,
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        #[arg(short, long, default_value_t = 16)]
        limit: usize,
        /// Skip QueryTraceRecorded event (default: true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        dry_run: bool,
    },
    /// Expand an evidence / conclusion / decision handle to a bounded preview
    #[command(
        after_help = "Examples:\n  ai-brains query expand <handle-id> --project-id <uuid>\n  # or set AI_BRAINS_PROJECT_ID"
    )]
    Expand {
        /// Handle id (evidence UUID, conclusion id, or decision id)
        handle_id: String,
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        #[arg(long, default_value_t = 512)]
        max_chars: usize,
    },
    /// Fetch a governed query trace by id (null when missing or unauthorized)
    #[command(after_help = "Examples:\n  ai-brains query trace <trace-id>")]
    Trace { trace_id: String },
}

#[derive(Subcommand, Clone)]
#[command(after_help = "Examples:\n  ai-brains scope resolve --format json")]
enum ScopeCommands {
    /// Resolve the active governed scope for the working context
    #[command(after_help = "Examples:\n  ai-brains scope resolve --format json")]
    Resolve {
        /// Output format: json (default) | human | markdown
        #[arg(long, default_value = "json")]
        format: Option<String>,
        /// Working directory hint (defaults to cwd)
        #[arg(long)]
        cwd: Option<String>,
        /// Explicit repository project id
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        /// Force Personal scope (never auto-selected otherwise)
        #[arg(long)]
        force_personal: bool,
        /// Personal user id when --force-personal
        #[arg(long)]
        personal_user_id: Option<String>,
        /// Force in-process control-plane path
        #[arg(long)]
        local: bool,
        /// Prefer daemon named-pipe path
        #[arg(long)]
        daemon: bool,
        /// Require daemon; exit 5 if unavailable
        #[arg(long)]
        require_daemon: bool,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains evidence list --scope Repository:<uuid>\n  ai-brains evidence list --format json\n  ai-brains evidence search --query keyword --scope Repository:<uuid>\n  ai-brains evidence show <id> --scope Repository:<uuid> --format json"
)]
enum EvidenceCommands {
    /// List evidence for a scope (optional FTS --query)
    #[command(
        after_help = "Examples:\n  ai-brains evidence list --scope Repository:<uuid>\n  ai-brains evidence list --format json\n  ai-brains evidence list --query keyword --scope Repository:<uuid>"
    )]
    List {
        /// Scope identity key; soft-filled from authoritative context when omitted
        #[arg(long)]
        scope: Option<String>,
        /// Optional FTS query over evidence summary
        #[arg(long)]
        query: Option<String>,
        /// Max items (default 50, max 200)
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
    /// Search evidence (requires --query; same handler as list)
    #[command(
        after_help = "Examples:\n  ai-brains evidence search --query keyword --scope Repository:<uuid>\n  ai-brains evidence search --query keyword --format json"
    )]
    Search {
        /// Scope identity key; soft-filled from authoritative context when omitted
        #[arg(long)]
        scope: Option<String>,
        /// FTS query over evidence summary (required)
        #[arg(long)]
        query: String,
        /// Max items (default 50, max 200)
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
    /// Show a bounded evidence / handle preview
    #[command(
        after_help = "Examples:\n  ai-brains evidence show <id> --scope Repository:<uuid> --format json\n  ai-brains evidence show <id> --format json"
    )]
    Show {
        /// Evidence or handle id
        id: String,
        /// Scope identity key; soft-filled from authoritative context when omitted
        #[arg(long)]
        scope: Option<String>,
        /// Output format: json (default) | human | markdown
        #[arg(long, default_value = "json")]
        format: Option<String>,
        /// Max characters in preview body
        #[arg(long, default_value_t = 512)]
        max_chars: usize,
        /// Principal UUID override (or AI_BRAINS_PREFLIGHT_PRINCIPAL_ID)
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains source list --scope Repository:<uuid>\n  ai-brains source list --format json\n  ai-brains source show <id> --scope Repository:<uuid>"
)]
enum SourceCommands {
    /// List registered sources for a scope
    #[command(
        after_help = "Examples:\n  ai-brains source list --scope Repository:<uuid>\n  ai-brains source list --format json"
    )]
    List {
        /// Scope identity key; soft-filled from authoritative context when omitted
        #[arg(long)]
        scope: Option<String>,
        /// Max items (default 50, max 200)
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
    /// Show a registered source by id
    #[command(
        after_help = "Examples:\n  ai-brains source show <id> --scope Repository:<uuid>\n  ai-brains source show <id> --format json"
    )]
    Show {
        /// Source id
        id: String,
        /// Scope identity key; soft-filled from authoritative context when omitted
        #[arg(long)]
        scope: Option<String>,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains conclusion propose --claim \"...\" --evidence <id> --scope Repository:<uuid>"
)]
enum ConclusionCommands {
    /// Propose a conclusion (daemon preferred; local if daemon down before send or --local)
    #[command(
        after_help = "Examples:\n  ai-brains conclusion propose --claim \"...\" --evidence <id> --scope Repository:<uuid>"
    )]
    Propose {
        /// Claim / statement text
        #[arg(long = "claim", visible_alias = "statement")]
        claim: String,
        /// Supporting evidence ids (repeatable)
        #[arg(long = "evidence")]
        evidence: Vec<String>,
        /// Scope identity key (required), e.g. Repository:<uuid>
        #[arg(long)]
        scope: String,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        /// Idempotency key (auto-generated UUID if omitted)
        #[arg(long = "command-id")]
        command_id: Option<String>,
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains decision propose --statement \"...\" --scope Repository:<uuid>"
)]
enum DecisionCommands {
    /// Propose a decision (daemon preferred; local if daemon down before send or --local)
    #[command(
        after_help = "Examples:\n  ai-brains decision propose --statement \"...\" --scope Repository:<uuid>"
    )]
    Propose {
        /// Decision statement
        #[arg(long)]
        statement: String,
        /// Optional title (defaults to "Decision")
        #[arg(long)]
        title: Option<String>,
        /// Supporting conclusion ids (repeatable)
        #[arg(long = "conclusion")]
        conclusions: Vec<String>,
        /// Supporting evidence ids (repeatable)
        #[arg(long = "evidence")]
        evidence: Vec<String>,
        /// Scope identity key (required)
        #[arg(long)]
        scope: String,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long = "command-id")]
        command_id: Option<String>,
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains review list --scope Repository:<uuid>\n  ai-brains review list --format json\n  ai-brains review resolve <id> --resolution approved --scope Repository:<uuid>"
)]
enum ReviewCommands {
    /// List open review items (E1: items: [] when empty)
    #[command(
        after_help = "Examples:\n  ai-brains review list --scope Repository:<uuid>\n  ai-brains review list --format json"
    )]
    List {
        /// Scope identity key; soft-filled from authoritative context when omitted
        #[arg(long)]
        scope: Option<String>,
        /// Optional status filter (e.g. Open)
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
    /// Resolve a review item (prefer Human principal; System may get APPROVAL_REQUIRED)
    #[command(
        after_help = "Examples:\n  ai-brains review resolve <id> --resolution approved --scope Repository:<uuid>"
    )]
    Resolve {
        /// Review item id
        id: String,
        /// Resolution: approved | dismissed | deferred | ...
        #[arg(long)]
        resolution: String,
        /// Governing scope identity key (required)
        #[arg(long)]
        scope: String,
        /// Optional note appended to resolution
        #[arg(long)]
        note: Option<String>,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long = "command-id")]
        command_id: Option<String>,
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains policy show --scope Repository:<uuid>\n  ai-brains policy check --capability ProposeConclusion --scope Repository:<uuid>"
)]
enum PolicyCommands {
    /// List applied grants for principal + scope (read-only)
    #[command(after_help = "Examples:\n  ai-brains policy show --scope Repository:<uuid>")]
    Show {
        /// Scope identity key (required)
        #[arg(long)]
        scope: String,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
    },
    /// Dry-run capability allow check
    #[command(
        after_help = "Examples:\n  ai-brains policy check --capability ProposeConclusion --scope Repository:<uuid>"
    )]
    Check {
        /// Capability name (e.g. ProposeConclusion)
        #[arg(long)]
        capability: String,
        /// Scope identity key
        #[arg(long)]
        scope: String,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains erasure request --id <id> --scope Repository:<uuid> --format json\n  ai-brains erasure wipe --content-key-id <uuid> --scope Repository:<uuid> --dry-run\n  ai-brains erasure wipe --content-key-id <uuid> --scope Repository:<uuid> --confirm"
)]
enum ErasureCommands {
    /// Request an erasure ticket (daemon-required; never claims CE wipe)
    #[command(
        after_help = "Examples:\n  ai-brains erasure request --id <id> --scope Repository:<uuid> --format json\nNote: ticket ≠ cryptographic erase. Use `erasure wipe` for envelope-backed CE."
    )]
    Request {
        /// Target record / aggregate ids (repeatable)
        #[arg(long = "id", required = true)]
        ids: Vec<String>,
        /// Human-readable reason
        #[arg(long)]
        reason: Option<String>,
        /// Scope identity key (required)
        #[arg(long)]
        scope: String,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long = "command-id")]
        command_id: Option<String>,
        /// Rejected: erasure is daemon-only
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
    /// [dangerous] Cryptographic erase envelope-backed content (daemon-required; dry-run default)
    #[command(
        after_help = "Honesty:\n  - CE only for content_key_store envelope-backed keys (NOT_ENVELOPE_BACKED otherwise)\n  - Not NIST Purge/Destroy; not physical media sanitization (WAL TRUNCATE is not Purge)\n  - Pre-erase backups/exports remain decryptable if restored\n  - Ticket path and soft forget are not cryptographic erasure\n  - SQLCipher vault lock is not per-item CE\nExamples:\n  ai-brains erasure wipe --content-key-id <uuid> --scope Repository:<uuid>\n  ai-brains erasure wipe --content-key-id <uuid> --scope Repository:<uuid> --confirm"
    )]
    Wipe {
        /// Content key id (UUID) to cryptographically erase
        #[arg(long = "content-key-id", required = true)]
        content_key_id: String,
        /// Scope identity key (required)
        #[arg(long, required = true)]
        scope: String,
        /// Optional ops reason (no secrets)
        #[arg(long)]
        reason: Option<String>,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long = "command-id")]
        command_id: Option<String>,
        /// Plan only (default when --confirm is absent). No wrap destroy / events / purge.
        #[arg(long = "dry-run", action = clap::ArgAction::SetTrue)]
        dry_run: bool,
        /// Execute wipe (E9). Without this flag the command is dry-run only.
        #[arg(long = "confirm", action = clap::ArgAction::SetTrue)]
        confirm: bool,
        /// Rejected: wipe is daemon-only
        #[arg(long)]
        local: bool,
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        require_daemon: bool,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains retention plan --format json\n  ai-brains retention apply --confirm\n  ai-brains retention apply --confirm --scope Repository:<uuid>\nNightly: AI_BRAINS_RETENTION_APPLY_CE only logs intent; CE is CLI+daemon+confirm+scope only."
)]
enum RetentionCommands {
    /// Dry-run class matrix report (no disposal)
    #[command(after_help = "Examples:\n  ai-brains retention plan --format json")]
    Plan {
        #[arg(long, default_value = "json")]
        format: Option<String>,
    },
    /// [dangerous] Apply retention plan (requires --confirm; CE via daemon T165 wipe)
    #[command(
        after_help = "Honesty:\n  - Default refuse without --confirm\n  - Legacy projection delete is not CE (local)\n  - Envelope CE requires daemon + wipe_content_envelope only (T165)\n  - CE candidates require explicit --scope (Repository:<uuid> / Personal:<uuid>); no random default\n  - Projection-only apply may run without daemon or --scope\n  - Not NIST Purge; pre-erase backups residual\nExamples:\n  ai-brains retention apply --confirm --format json\n  ai-brains retention apply --confirm --scope Repository:<uuid> --format json"
    )]
    Apply {
        #[arg(long, default_value = "json")]
        format: Option<String>,
        /// Execute disposal (required). Without this flag the command refuses.
        #[arg(long = "confirm", action = clap::ArgAction::SetTrue)]
        confirm: bool,
        /// Explicit plan-only (conflicts with --confirm)
        #[arg(long = "dry-run", action = clap::ArgAction::SetTrue)]
        dry_run: bool,
        #[arg(long = "command-id")]
        command_id: Option<String>,
        /// Scope for CE wipe policy path (required when plan has CE candidates)
        #[arg(long)]
        scope: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
enum ShadowCommands {
    /// Create a new shadow vault from a source vault
    Create {
        /// Path to the source vault database
        #[arg(long)]
        source: PathBuf,
        /// Path for the new destination vault (must not exist)
        #[arg(long)]
        destination: PathBuf,
        /// Explicitly enable turn-content redaction (default behavior)
        #[arg(long = "redact-turn-content", action = clap::ArgAction::SetTrue)]
        redact_turn_content: bool,
        /// Preserve turn content when creating the shadow vault
        #[arg(long = "no-redact-turn-content", action = clap::ArgAction::SetTrue)]
        no_redact_turn_content: bool,
        /// Preview refusals and plan without writing any files
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum DogfoodCommands {
    /// Build dogfood-compare.json from governed packet + legacy preflight JSON
    Compare {
        /// Path to ProjectBriefingPacket JSON (from `briefing project --format json`)
        #[arg(long)]
        governed: PathBuf,
        /// Path to PreflightContextResponse JSON (from `preflight --format json`, flag off)
        #[arg(long)]
        legacy: PathBuf,
        /// Output path for dogfood-compare.json
        #[arg(long)]
        out: PathBuf,
        /// Allow overwriting an existing --out file (never vaults)
        #[arg(long = "allow-out-overwrite", default_value_t = false)]
        allow_out_overwrite: bool,
        /// Stage label: B (synthetic) or C (shadow dogfood)
        #[arg(long)]
        stage: Option<String>,
        /// Optional T169 evaluate-report.json (Stage B seed + report_hash)
        #[arg(long = "evaluate-report")]
        evaluate_report: Option<PathBuf>,
        /// Optional migrate-report.json path (recorded in paths; not opened)
        #[arg(long = "migrate-report")]
        migrate_report: Option<PathBuf>,
        /// Shadow vault path (recorded in paths; not opened)
        #[arg(long)]
        shadow: Option<PathBuf>,
        /// Migrated vault path (recorded in paths; not opened)
        #[arg(long)]
        migrated: Option<PathBuf>,
        /// Live vault path for integrity section (not opened)
        #[arg(long = "live-vault")]
        live_vault: Option<PathBuf>,
        /// D24 live vault SHA-256 before dogfood
        #[arg(long = "sha256-pre")]
        sha256_pre: Option<String>,
        /// D24 live vault SHA-256 after dogfood
        #[arg(long = "sha256-post")]
        sha256_post: Option<String>,
        /// T169 evaluate exit code
        #[arg(long = "t169-exit")]
        t169_exit: Option<i32>,
        /// T169 report_hash
        #[arg(long = "t169-report-hash")]
        t169_report_hash: Option<String>,
        /// T169 hard_gates_passed (optional override)
        #[arg(long = "t169-hard-gates-passed")]
        t169_hard_gates_passed: Option<bool>,
    },
}

#[derive(Subcommand)]
enum EvaluateCommands {
    /// Run versioned governed-memory scenario corpus + hard/soft metrics
    Governed {
        /// Directory of scenario JSON files (default: fixtures/governed-memory/scenarios)
        #[arg(long, default_value = "fixtures/governed-memory/scenarios")]
        fixtures: PathBuf,
        /// Optional path to write evaluate-report.json (stdout always gets JSON too)
        #[arg(long)]
        report: Option<PathBuf>,
        /// Filter to one or more scenario ids (default: all)
        #[arg(long = "scenario")]
        scenario: Vec<String>,
        /// Soft metric failures → exit 7
        #[arg(long)]
        strict_soft: bool,
        /// Deferred scenarios count as hard fail
        #[arg(long)]
        require_all_active: bool,
        /// Allow overwriting an existing report file
        #[arg(long)]
        allow_report_overwrite: bool,
    },
}

#[derive(Subcommand)]
enum MigrateCommands {
    /// [dangerous] Classify legacy events via T167; write differential report; --confirm materializes destination
    Governed {
        /// Path to the source vault database (never migrated)
        #[arg(long)]
        source: PathBuf,
        /// Path for the destination vault (refused if live / inside live parent)
        #[arg(long)]
        destination: PathBuf,
        /// Path for the differential report JSON
        #[arg(long)]
        report: PathBuf,
        /// Explicit dry-run (default when --confirm is absent)
        #[arg(long)]
        dry_run: bool,
        /// Materialize destination + apply T167 import
        #[arg(long)]
        confirm: bool,
        /// Fallback scope when events lack project_id (T167 L19). Form: Repository:<uuid>|Personal:<uuid>|Workspace:<uuid>
        #[arg(long)]
        default_scope: Option<String>,
        /// Copy source envelopes when dest is empty (default true on first materialize)
        #[arg(long = "copy-events", action = clap::ArgAction::SetTrue)]
        copy_events: bool,
        /// Skip envelope copy even on fresh dest (import-only)
        #[arg(long = "no-copy-events", action = clap::ArgAction::SetTrue)]
        no_copy_events: bool,
        /// Permit source == live vault (still refuses dest == live)
        #[arg(long)]
        allow_live_source: bool,
        /// With --confirm: delete existing dest vault + migrate-manifest and recreate
        #[arg(long)]
        force_overwrite: bool,
        /// SQLCipher product key for the source vault (`x'<64 hex>'`; falls back to --key / AI_BRAINS_KEY; missing → VAULT_KEY_MISSING)
        #[arg(long)]
        source_key: Option<String>,
        /// SQLCipher product key for the destination vault (`x'<64 hex>'`; falls back to --key / AI_BRAINS_KEY; missing → VAULT_KEY_MISSING)
        #[arg(long)]
        destination_key: Option<String>,
        /// Shared SQLCipher key when --source-key / --destination-key omitted (also root CLI --key / AI_BRAINS_KEY; no silent zero)
        /// (also accepted as a root CLI flag before `migrate`; this places it after `governed`)
        #[arg(long)]
        key: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
pub enum GraphCommands {
    /// Rebuild graph from all events
    Rebuild,
    /// Show 1-hop graph neighbors of a memory
    Neighbors { memory_id: String },
    /// Show recursive SYNTHESIZED_FROM hierarchy of a memory
    Hierarchy { memory_id: String },
    /// Show all memories in a session via graph edges
    Session { session_id: String },
    /// Show current graph health: node/edge counts
    Update,
}

#[derive(Subcommand, Clone)]
pub enum ProjectCommands {
    /// List all projects in the vault
    List,
    /// Resolve an alias to a project ID
    Resolve {
        /// Project alias to resolve (positional)
        alias_positional: Option<String>,
        /// Project alias to resolve via --alias flag
        #[arg(long = "alias", conflicts_with = "alias_positional")]
        alias: Option<String>,
    },
    /// Auto-detect project from current git repository (fallback: .env AI_BRAINS_PROJECT_ID)
    Detect {
        /// Output as shell export statement
        #[arg(long)]
        export: bool,
    },
    /// Set a human-readable alias for a project
    SetAlias {
        /// Project UUID (from `project list`)
        project_id: String,
        /// Alias name (e.g. "ai-brains", "my-app")
        alias: String,
    },
}

#[derive(Subcommand, Clone)]
pub enum DaemonCommands {
    /// Start the daemon in the background
    Start,
    /// Show the status of the running daemon
    Status,
    /// Register a Windows Task Scheduler logon task to auto-start the daemon
    Schedule {
        /// Preview the schtasks command without registering the task
        #[arg(long)]
        dry_run: bool,
        /// Schedule the task to run as SYSTEM (no login required). Requires elevation.
        #[arg(long)]
        run_as_system: bool,
    },
    /// Remove the Task Scheduler logon task
    Unschedule {
        /// Preview the schtasks /delete command without executing it
        #[arg(long)]
        dry_run: bool,
    },
    /// [dangerous] Install the daemon as a Windows service (requires elevation)
    Install {
        /// Preview the sc.exe commands without executing them
        #[arg(long)]
        dry_run: bool,
    },
    /// [dangerous] Uninstall the Windows service (requires elevation)
    Uninstall {
        /// Preview the sc.exe command without executing it
        #[arg(long)]
        dry_run: bool,
    },
    /// Stop the running daemon gracefully
    Stop {
        /// Forcefully terminate the process if it doesn't respond to shutdown signal
        #[arg(long, short)]
        force: bool,
    },
    /// [dangerous] Stop daemon, install updated binaries, then restart (run from workspace root)
    Update,
}

#[derive(Subcommand, Clone)]
pub enum RecoveryCommands {
    /// Write a RecoveryKit JSON file (passphrase-wrapped DataKey; never prints kit JSON)
    Export {
        /// Destination path for the RecoveryKit JSON file
        #[arg(long)]
        output: PathBuf,
        /// Read passphrase from a regular file (max 8 KiB). Prefer over interactive TTY.
        /// Trailing single newline is stripped. Min length 8 bytes after trim.
        #[arg(long)]
        passphrase_file: Option<PathBuf>,
        /// Validate passphrase source and print would-write path; no file, no event
        #[arg(long)]
        dry_run: bool,
        /// Overwrite output if it already exists
        #[arg(long, short, visible_alias = "overwrite")]
        force: bool,
    },
}

#[derive(Subcommand, Clone)]
pub enum BackupCommands {
    /// Create a timestamped backup (default)
    Create {
        /// Custom output directory for the backup
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// After a successful backup, prune old backups keeping only the N
        /// most recent (including the new one). Default: 10.
        #[arg(long, conflicts_with = "no_prune")]
        keep: Option<usize>,
        /// Disable pruning after creating the backup
        #[arg(long, conflicts_with = "keep")]
        no_prune: bool,
        /// Preview what would happen without creating the backup file
        #[arg(long)]
        dry_run: bool,
    },
    /// Restore vault from a backup file
    Restore {
        /// Path to the backup file
        path: PathBuf,
        /// Skip the interactive confirmation prompt
        #[arg(long, short)]
        force: bool,
        /// Verify the backup's integrity and print the plan, but do not
        /// overwrite the destination vault
        #[arg(long)]
        dry_run: bool,
    },
    /// Delete old backups according to a retention policy
    Prune {
        /// Keep the N most recent backups (default: 10)
        #[arg(long, default_value_t = 10)]
        keep: usize,
        /// Delete backups older than this duration (e.g. 30d, 12h, 2w)
        #[arg(long)]
        older_than: Option<String>,
        /// List the files that would be deleted without actually deleting them
        #[arg(long)]
        dry_run: bool,
    },
    /// List all backups with their metadata
    List {
        /// Suppress WARN-level tracing output for backup metadata read failures.
        #[arg(long)]
        quiet: bool,
    },
    /// Verify the integrity of backup files
    Verify {
        /// Path to a single backup file to verify
        path: Option<PathBuf>,
        /// Run a full integrity_check instead of the default quick_check
        #[arg(long)]
        full: bool,
        /// Output format: 'json' or 'pretty' (default: pretty)
        #[arg(long)]
        format: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
pub enum SyncCommands {
    /// Pull records from an NDJSON file
    Pull {
        /// Path to the NDJSON file
        #[arg(long)]
        from_file: Option<PathBuf>,
        /// Export hotspot data from Ledgerful
        #[arg(long)]
        hotspots: bool,
        /// Export ledger delta data from Ledgerful
        #[arg(long)]
        ledger: bool,
        /// Suppress Ledgerful error messages
        #[arg(long, short)]
        quiet: bool,
        /// Print the JSON Schema for the expected NDJSON record shape and exit.
        /// The schema is also at `Docs/schemas/sync-pull-record.json`.
        #[arg(long)]
        schema: bool,
    },
    /// Push current context to Ledgerful
    Push {
        /// Include impact context
        #[arg(long)]
        with_impact: bool,
        /// Include verification context
        #[arg(long)]
        with_verify: bool,
        /// Suppress Ledgerful error messages
        #[arg(long, short)]
        quiet: bool,
    },
    /// Unified query across AI-Brains and Ledgerful
    Query {
        /// The query string
        query: String,
        /// Output format (pretty, text, ndjson)
        #[arg(long)]
        format: Option<String>,
        /// Suppress daemon-down error messages
        #[arg(long, short)]
        quiet: bool,
        /// Search across all projects, ignoring AI_BRAINS_PROJECT_ID
        #[arg(long)]
        global: bool,
        /// Skip the Ledgerful bridge query and use only local vault recall.
        #[arg(long)]
        no_bridge: bool,
    },
}

#[derive(Subcommand, Clone)]
pub enum SafetyCommands {
    /// Synchronize Ledgerful hotspots into the AI-Brains vault
    Sync {
        /// Limit the number of hotspots to ingest
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
        /// Preview what would be synced without pinning
        #[arg(long)]
        dry_run: bool,
    },
}

/// T86: Read a plain-text query from stdin until EOF.
/// Returns an error if stdin is a terminal (avoids hanging in interactive shells).
fn read_query_from_stdin() -> Result<String, Box<dyn std::error::Error>> {
    use is_terminal::IsTerminal;
    use std::io::Read;
    if std::io::stdin().is_terminal() {
        return Err(
            "stdin is a terminal — pipe or redirect input when using `-` as the query.".into(),
        );
    }
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("Failed to read from stdin: {e}"))?;
    let query = buf.trim().to_string();
    if query.is_empty() {
        return Err("Query read from stdin is empty.".into());
    }
    Ok(query)
}

/// T86: Read a JSON object from stdin until EOF.
/// Returns an error if stdin is a terminal.
fn read_json_from_stdin() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use is_terminal::IsTerminal;
    use std::io::Read;
    if std::io::stdin().is_terminal() {
        return Err("stdin is a terminal — pipe JSON input when using --stdin.".into());
    }
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("Failed to read from stdin: {e}"))?;
    let value: serde_json::Value = serde_json::from_str(buf.trim())
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    Ok(value)
}

fn should_warn_project_context_override(args: &[String]) -> bool {
    args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "preflight"
                | "recall"
                | "sync"
                | "pin"
                | "forget"
                | "nightly"
                | "context"
                | "project"
                | "safety"
                | "antigravity-import"
                | "briefing"
                | "query"
        )
    })
}

/// User home for global dotenv / gap-fill paths.
///
/// Prefer `USERPROFILE` then `HOME` (non-empty trim) before `dirs::home_dir()`.
/// Required for hermetic empty-home isolation: dirs 6 on Windows uses
/// `SHGetKnownFolderPath` and does not honor a redirected `USERPROFILE`.
fn resolve_user_home_for_dotenv() -> Option<std::path::PathBuf> {
    for key in ["USERPROFILE", "HOME"] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(std::path::PathBuf::from(trimmed));
            }
        }
    }
    dirs::home_dir()
}

fn apply_local_project_context_env(path: &std::path::Path, warn_on_override: bool) {
    let entries = match dotenvy::from_path_iter(path) {
        Ok(entries) => entries,
        Err(err) => {
            tracing::warn!("Failed to parse local .env for project context: {}", err);
            return;
        }
    };

    for entry in entries {
        let (key, value) = match entry {
            Ok(entry) => entry,
            Err(err) => {
                tracing::warn!("Skipping malformed local .env entry: {}", err);
                continue;
            }
        };

        if key != "AI_BRAINS_PROJECT_ID" && key != "AI_BRAINS_SESSION_ID" {
            continue;
        }

        if warn_on_override {
            if let Ok(existing) = std::env::var(&key)
                && existing != value
            {
                eprintln!(
                    "Warning: local .env {} overrides inherited shell value {}.",
                    key, existing
                );
            }
        } else if let Ok(existing) = std::env::var(&key)
            && existing != value
        {
            tracing::debug!(
                "local .env {} overrides inherited shell value for this command",
                key
            );
        }

        // SAFETY: single-threaded CLI startup before worker threads; process env
        // is intentionally mutated for project-context loading.
        unsafe {
            std::env::set_var(key, value);
        }
    }
}

fn main() {
    // Windows PE main-thread stack is often ~1 MiB; clap `Commands` + async
    // frames exceed that in debug builds once Doctor (T192) landed. Spawn a
    // worker with a larger stack. RUST_MIN_STACK only affects non-main threads.
    #[cfg(windows)]
    {
        const STACK: usize = 16 * 1024 * 1024;
        let result = std::thread::Builder::new()
            .name("ai-brains-main".into())
            .stack_size(STACK)
            .spawn(main_inner)
            .unwrap_or_else(|e| {
                eprintln!("Failed to spawn main worker thread: {e}");
                std::process::exit(1);
            })
            .join();
        match result {
            Ok(()) => {}
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
    #[cfg(not(windows))]
    main_inner();
}

fn main_inner() {
    // T197 F1/F27/F29: silence SQLCipher hmac flood before any vault open.
    ai_brains_store::sqlcipher_log_policy::install();

    let args: Vec<String> = std::env::args().collect();
    // UAC elevated child: restore env + cwd handoff from the non-elevated parent
    // before any .env / project-context logic (parent may have already loaded .env).
    crate::elevation::load_elevate_env_handoff();

    // Parse the CLI first so we can read the global --no-project-context
    // flag before doing any env-var manipulation. We re-parse below; clap
    // is cheap and this keeps the env-var logic close to its trigger.
    let no_project_context = args.iter().any(|a| a == "--no-project-context");
    let warn_on_project_context_override = should_warn_project_context_override(&args);

    // Pre-scan for --log-format so the tracing subscriber can be initialized
    // with the requested format before clap is fully parsed.
    let log_format = args
        .windows(2)
        .find(|w| w[0] == "--log-format")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "compact".to_string());

    // Project .env fills env gaps without overriding shell vars.
    // If no local .env exists, we clear project-specific env vars to prevent
    // stale inheritance from other projects in the same shell session.
    // T80: --no-project-context skips *project* discovery only so CI/hooks can
    // supply IDs explicitly. User-global ~/.ai-brains/.env still merges for gaps
    // (KEY / VAULT_PATH / models) unless the process already set those vars.
    if !no_project_context {
        let project_env = std::path::Path::new(".env");
        if !project_env.exists() {
            // SAFETY: single-threaded CLI startup before worker threads; process env
            // is intentionally mutated to clear stale project context.
            unsafe {
                std::env::remove_var("AI_BRAINS_PROJECT_ID");
                std::env::remove_var("AI_BRAINS_SESSION_ID");
            }
        } else {
            dotenvy::dotenv().ok();
            apply_local_project_context_env(project_env, warn_on_project_context_override);
        }
    }

    // Always merge user-global ~/.ai-brains/.env for gaps (KEY, VAULT_PATH, models).
    // dotenvy does not override vars already set by the shell or project `.env`.
    // Runs even with --no-project-context so vault key/path work in CI-style flags
    // without forcing secrets onto the command line. Previously gated on
    // AI_BRAINS_VAULT_PATH unset only (skipped KEY when path was already present).
    // Soft-fail parse errors (file absent is fine); never from_path_override.
    //
    // Home resolution (T205 F11/F22): prefer USERPROFILE then HOME so hermetic tests
    // and operators can redirect home. dirs 6 on Windows uses Known Folder API and
    // ignores USERPROFILE — same pattern as backup.rs retention sentinel.
    if let Some(mut home) = resolve_user_home_for_dotenv() {
        home.push(".ai-brains");
        home.push(".env");
        if home.exists()
            && let Err(err) = dotenvy::from_path(&home)
        {
            // Subscriber may not be installed yet; warn is best-effort.
            tracing::warn!(
                path = %home.display(),
                error = %err,
                "failed to load global ~/.ai-brains/.env (gaps not filled from file)"
            );
        }
    }

    let default_filter = tracing_subscriber::EnvFilter::new(
        "warn,ai_brains=info,ai_brains_cli=info,ai_brains_brain=info",
    );
    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or(default_filter);

    match log_format.as_str() {
        "off" => {
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::new("off"))
                .init();
        }
        "json" => {
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(env_filter)
                .init();
        }
        "full" => {
            tracing_subscriber::fmt().with_env_filter(env_filter).init();
        }
        "minimal" => {
            tracing_subscriber::fmt()
                .compact()
                .with_target(false)
                .without_time()
                .with_env_filter(env_filter)
                .init();
        }
        _ => {
            tracing_subscriber::fmt()
                .compact()
                .with_target(false)
                .with_env_filter(env_filter)
                .init();
        }
    }

    // Set up a basic signal handler for graceful interruption
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Failed to initialize Tokio runtime: {}", e);
            std::process::exit(1);
        }
    };

    // Parse outside the async future so the huge `Commands` enum does not bloat the
    // Tokio state machine (Windows debug stacks are tight; T168 Migrate tipped it over).
    let cli = Cli::parse();

    // Sync vault-path-free commands: handle before AppContext / async runtime.
    // Includes schema printers and the non-graph stub so clean Linux CI hosts
    // without AI_BRAINS_VAULT_PATH still work (T179).
    if is_vault_path_free(cli.command.as_ref()) {
        handle_cli_result(run_sync_path_free(cli));
        return;
    }

    runtime.block_on(async {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!("\nInterrupted by user. Exiting...");
                std::process::exit(130);
            }
            res = async {
                run(cli).await
            } => {
                handle_cli_result(res);
            }
        }
    });
}

/// Commands that must not require `--vault-path` / `AI_BRAINS_VAULT_PATH`.
fn is_vault_path_free(command: &Commands) -> bool {
    match command {
        Commands::Shadow { .. }
        | Commands::Migrate { .. }
        | Commands::Evaluate { .. }
        | Commands::Dogfood { .. } => true,
        // Encrypt may use --source; rotate-datakey needs vault path + async daemon probe.
        Commands::Vault {
            command: VaultCommands::Encrypt { .. },
        } => true,
        Commands::Vault {
            command: VaultCommands::RotateDatakey { .. },
        } => false,
        Commands::AgyHook { schema: true, .. } => true,
        Commands::Sync {
            command: SyncCommands::Pull { schema: true, .. },
        } => true,
        #[cfg(not(feature = "graph"))]
        Commands::Graph { .. } => true,
        _ => false,
    }
}

fn handle_cli_result(res: Result<(), Box<dyn std::error::Error>>) {
    match res {
        Ok(()) => {
            // Elevated UAC child: leave a success marker the parent can print
            // (elevated console is hidden / flashes closed). Commands may
            // already have written a richer message — do not overwrite.
            if crate::elevation::is_elevated() && !crate::elevation::elevate_result_path().exists()
            {
                crate::elevation::write_elevate_success_log(
                    "Elevated command completed successfully.",
                );
            }
        }
        Err(err) => {
            if crate::elevation::is_elevated() {
                crate::elevation::write_elevate_error_log(&err.to_string());
            }
            // Governed surface (T160): structured exit codes; payload already emitted.
            if let Some(g) = err.downcast_ref::<commands::governed_common::GovernedCliError>() {
                if !g.emitted {
                    eprintln!("{}", g.message);
                }
                std::process::exit(g.exit_code);
            }
            use crate::key_resolve::{
                KeyResolveError, VAULT_LOCKED_JSON_CODE, key_resolve_json_code,
                vault_locked_message,
            };
            use ai_brains_contracts::response::{ApiError, ApiResult};
            use ai_brains_store::StoreError;

            // T197 F8: map key resolve + vault locked to dedicated JSON codes.
            let (code, message) = if let Some(e) = err.downcast_ref::<KeyResolveError>() {
                (key_resolve_json_code(e), e.to_string())
            } else if let Some(StoreError::VaultLocked(detail)) = err.downcast_ref::<StoreError>() {
                (VAULT_LOCKED_JSON_CODE, vault_locked_message(detail))
            } else {
                let s = err.to_string();
                // Fallback string-family match when error was stringified mid-path.
                if s.starts_with("Vault key missing:") {
                    ("VAULT_KEY_MISSING", s)
                } else if s.starts_with("Vault key invalid format:") {
                    ("VAULT_KEY_FORMAT", s)
                } else if s.starts_with("Vault key refused:") {
                    ("VAULT_KEY_ZERO", s)
                } else if s.contains("Vault is locked")
                    || s.contains("Key verification failed")
                    || s.starts_with("Vault locked:")
                {
                    (VAULT_LOCKED_JSON_CODE, vault_locked_message(&s))
                } else {
                    ("COMMAND_FAILED", s)
                }
            };
            let api_error = ApiError::new(code, message);
            let result = ApiResult::<serde_json::Value>::error(api_error);
            if let Ok(json) = serde_json::to_string(&result) {
                eprintln!("{}", json);
            } else {
                eprintln!("Error: {err}");
            }
            std::process::exit(1);
        }
    }
}

/// Vault-path-free commands: no AppContext (shadow/migrate/evaluate/dogfood,
/// schema printers, non-graph stub).
fn run_sync_path_free(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match *cli.command {
        Commands::AgyHook { schema: true, .. } => {
            print_schema(SCHEMA_AGY_HOOK, "AI-Brains agy-hook payload")
        }
        Commands::Sync {
            command: SyncCommands::Pull { schema: true, .. },
        } => print_schema(SCHEMA_SYNC_PULL, "AI-Brains sync pull NDJSON record"),
        #[cfg(not(feature = "graph"))]
        Commands::Graph { .. } => {
            println!(
                "{}: The graph subcommand requires a --features graph build.",
                commands::governed_common::FEATURE_UNAVAILABLE
            );
            println!(
                "Reinstall with: cargo install --path crates/ai-brains-cli --locked --features graph"
            );
            std::process::exit(commands::governed_common::exit_code_feature_unavailable());
        }
        Commands::Shadow { command } => match command {
            ShadowCommands::Create {
                source,
                destination,
                redact_turn_content,
                no_redact_turn_content,
                dry_run,
            } => {
                let _ = redact_turn_content;
                let redact = !no_redact_turn_content;
                commands::shadow::run_create(source, destination, redact, dry_run, cli.key)
            }
        },
        Commands::Migrate { command } => match *command {
            MigrateCommands::Governed {
                source,
                destination,
                report,
                dry_run,
                confirm,
                default_scope,
                copy_events,
                no_copy_events,
                allow_live_source,
                force_overwrite,
                source_key,
                destination_key,
                key,
            } => {
                let _ = copy_events;
                let copy = !no_copy_events;
                // Shared key: governed `--key` (after subcommand) then root `--key`.
                // Per-side inside run_governed: source_key → shared → AI_BRAINS_KEY → Missing
                // (no silent zero; T197 SOOT).
                let shared_key = key.or(cli.key);
                commands::migrate::run_governed(commands::migrate::GovernedOptions {
                    source,
                    destination,
                    report,
                    dry_run,
                    confirm,
                    default_scope,
                    copy_events: copy,
                    allow_live_source,
                    force_overwrite,
                    source_key,
                    destination_key,
                    key: shared_key,
                })
            }
        },
        Commands::Evaluate { command } => match command {
            EvaluateCommands::Governed {
                fixtures,
                report,
                scenario,
                strict_soft,
                require_all_active,
                allow_report_overwrite,
            } => commands::evaluate::run_governed(commands::evaluate::GovernedEvaluateOptions {
                fixtures,
                report,
                scenario,
                strict_soft,
                require_all_active,
                allow_report_overwrite,
                vault_path: cli.vault_path,
            }),
        },
        Commands::Dogfood { command } => match command {
            DogfoodCommands::Compare {
                governed,
                legacy,
                out,
                allow_out_overwrite,
                stage,
                evaluate_report,
                migrate_report,
                shadow,
                migrated,
                live_vault,
                sha256_pre,
                sha256_post,
                t169_exit,
                t169_report_hash,
                t169_hard_gates_passed,
            } => commands::dogfood::run_compare(commands::dogfood::DogfoodCompareOptions {
                governed,
                legacy,
                out,
                stage,
                evaluate_report,
                migrate_report,
                shadow,
                migrated,
                live_vault,
                sha256_pre,
                sha256_post,
                t169_exit,
                t169_report_hash,
                t169_hard_gates_passed,
                allow_out_overwrite,
            }),
        },
        Commands::Vault { command } => match command {
            VaultCommands::Encrypt {
                source,
                destination,
                confirm,
                dry_run,
            } => {
                let source = source.or(cli.vault_path).ok_or(
                    "vault encrypt requires --source or --vault-path / AI_BRAINS_VAULT_PATH",
                )?;
                commands::vault::run_encrypt(commands::vault::EncryptCliOptions {
                    source,
                    destination,
                    key: cli.key,
                    confirm,
                    dry_run,
                })
            }
            VaultCommands::RotateDatakey { .. } => {
                unreachable!("vault rotate-datakey is not vault-path-free; handled in async run()")
            }
        },
        _ => unreachable!("run_sync_path_free only for vault-path-free commands"),
    }
}

/// T197 F19: `init` with no key generates a non-zero random product key once.
fn run_init(
    vault_path: Option<PathBuf>,
    key: Option<String>,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::key_resolve::{KeyResolveError, resolve_operator_sqlcipher_key};
    use ai_brains_crypto::{DataKey, SqlCipherKey};
    use zeroize::Zeroizing;

    let path = vault_path.ok_or("Vault path is required (--vault-path or AI_BRAINS_VAULT_PATH)")?;

    // Zeroizing keeps the one-time stdout bootstrap copy off the free-list plaintext.
    let (sql_key, generated_material): (SqlCipherKey, Option<Zeroizing<String>>) =
        match resolve_operator_sqlcipher_key(key) {
            Ok(k) => (k, None),
            Err(KeyResolveError::Missing) => {
                // Generate non-zero random key (regenerate if theoretically all-zero).
                let mut data = DataKey::generate();
                let mut sql = SqlCipherKey::from_data_key(&data);
                if sql.is_zero() {
                    data = DataKey::generate();
                    sql = SqlCipherKey::from_data_key(&data);
                }
                let material = Zeroizing::new(sql.expose_secret().to_string());
                (sql, Some(material))
            }
            Err(e) => return Err(e.into()),
        };

    let ctx = AppContext::from_resolved_key(path, sql_key)?;
    let print_key = generated_material.as_ref().map(|z| z.as_str());
    commands::init::run(&ctx, force, print_key)
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // T197 F19: init generates a non-zero key when none provided (no silent zero).
    if let Commands::Init { force } = cli.command.as_ref() {
        return run_init(cli.vault_path.clone(), cli.key.clone(), *force);
    }

    // T188 F16b: recovery export must not call AppContext::from_cli (always migrate()).
    // Special-case before vault open+migrate so kit export works while daemon is up.
    if let Commands::Recovery { command } = cli.command.as_ref() {
        return match command {
            RecoveryCommands::Export {
                output,
                passphrase_file,
                dry_run,
                force,
            } => {
                let vault_path = cli
                    .vault_path
                    .clone()
                    .ok_or("Vault path is required (--vault-path or AI_BRAINS_VAULT_PATH)")?;
                commands::recovery::run_export(commands::recovery::ExportOptions {
                    vault_path,
                    key: cli.key.clone(),
                    output: output.clone(),
                    passphrase_file: passphrase_file.clone(),
                    dry_run: *dry_run,
                    force: *force,
                })
                .await
            }
        };
    }

    // T192: doctor is read-only open_read_intent only — never AppContext::from_cli (migrate).
    if let Commands::Doctor {
        format,
        json,
        fail_on_degraded,
        kit_path,
        passphrase_file,
        backup_max_age,
        full,
    } = cli.command.as_ref()
    {
        let vault_path = cli
            .vault_path
            .clone()
            .ok_or("Vault path is required (--vault-path or AI_BRAINS_VAULT_PATH)")?;
        return commands::doctor::run(commands::doctor::DoctorOptions {
            vault_path,
            key: cli.key.clone(),
            format: format.clone(),
            json: *json,
            fail_on_degraded: *fail_on_degraded,
            kit_path: kit_path.clone(),
            passphrase_file: passphrase_file.clone(),
            backup_max_age: backup_max_age.clone(),
            full: *full,
        })
        .await;
    }

    // T199: daemon status is liveness IPC only — no AppContext / key / vault open.
    if let Commands::Daemon {
        command: DaemonCommands::Status,
    } = cli.command.as_ref()
    {
        return commands::daemon::run_status(commands::daemon::StatusOptions {
            vault_path: cli.vault_path.clone(),
            key: cli.key.clone(),
        })
        .await;
    }

    // T189: rotate-datakey mutates outside AppContext (daemon probe + no migrate race).
    if let Commands::Vault {
        command:
            VaultCommands::RotateDatakey {
                dry_run,
                confirm,
                require_backup,
                i_have_backup,
                kit_output,
                passphrase_file,
                overwrite_kit,
                accept_rekey_risk,
                print_key,
                backup_dir,
            },
    } = cli.command.as_ref()
    {
        let vault_path = cli
            .vault_path
            .clone()
            .ok_or("vault rotate-datakey requires --vault-path / AI_BRAINS_VAULT_PATH")?;
        return commands::vault::run_rotate_datakey(commands::vault::RotateDatakeyOptions {
            vault_path,
            key: cli.key.clone(),
            dry_run: *dry_run,
            confirm: *confirm,
            require_backup: *require_backup,
            i_have_backup: i_have_backup.clone(),
            kit_output: kit_output.clone(),
            passphrase_file: passphrase_file.clone(),
            overwrite_kit: *overwrite_kit,
            accept_rekey_risk: *accept_rekey_risk,
            print_key: *print_key,
            backup_dir: backup_dir.clone(),
        })
        .await;
    }

    let ctx = AppContext::from_cli(cli.vault_path.clone(), cli.key.clone())?;
    match cli.command.as_ref() {
        Commands::Shadow { .. } => unreachable!("shadow handled in run_sync_path_free"),
        Commands::Migrate { .. } => unreachable!("migrate handled in run_sync_path_free"),
        Commands::Evaluate { .. } => unreachable!("evaluate handled in run_sync_path_free"),
        Commands::Dogfood { .. } => unreachable!("dogfood handled in run_sync_path_free"),
        Commands::Vault {
            command: VaultCommands::Encrypt { .. },
        } => unreachable!("vault encrypt handled in run_sync_path_free"),
        Commands::Vault {
            command: VaultCommands::RotateDatakey { .. },
        } => unreachable!("vault rotate-datakey handled before AppContext"),
        Commands::Recovery { .. } => unreachable!("recovery handled before AppContext"),
        Commands::Doctor { .. } => unreachable!("doctor handled before AppContext"),
        Commands::Init { .. } => unreachable!("init handled before AppContext"),
        Commands::Briefing { command } => match command {
            BriefingCommands::Project {
                project_id,
                max_words,
                dry_run,
                format,
            } => commands::briefing::run_project(
                &ctx,
                commands::briefing::ProjectBriefingOptions {
                    project_id: *project_id,
                    max_words: *max_words,
                    dry_run: *dry_run,
                    format: format.clone(),
                },
            ),
            BriefingCommands::Personal {
                user_id,
                max_words,
                dry_run,
                format,
            } => {
                let uid = match user_id {
                    Some(raw) => Some(ai_brains_core::ids::UserId::from_str(raw)?),
                    None => None,
                };
                commands::briefing::run_personal(
                    &ctx,
                    commands::briefing::PersonalBriefingOptions {
                        user_id: uid,
                        max_words: *max_words,
                        dry_run: *dry_run,
                        format: format.clone(),
                    },
                )
            }
        },
        Commands::Query { command } => match command {
            GovernedQueryCommands::Progressive {
                query,
                project_id,
                limit,
                dry_run,
            } => commands::governed_query::run_progressive(
                &ctx,
                commands::governed_query::ProgressiveQueryOptions {
                    query: query.clone(),
                    project_id: *project_id,
                    limit: *limit,
                    dry_run: *dry_run,
                },
            ),
            GovernedQueryCommands::Expand {
                handle_id,
                project_id,
                max_chars,
            } => commands::governed_query::run_expand(
                &ctx,
                commands::governed_query::ExpandHandleOptions {
                    handle_id: handle_id.clone(),
                    project_id: *project_id,
                    max_chars: *max_chars,
                },
            ),
            GovernedQueryCommands::Trace { trace_id } => commands::governed_query::run_trace(
                &ctx,
                commands::governed_query::TraceOptions {
                    trace_id: trace_id.clone(),
                },
            ),
        },
        Commands::Scope { command } => match command {
            ScopeCommands::Resolve {
                format,
                cwd,
                project_id,
                force_personal,
                personal_user_id,
                local,
                daemon,
                require_daemon,
            } => {
                commands::scope::run_resolve(
                    &ctx,
                    commands::scope::ResolveOptions {
                        format: format.clone(),
                        cwd: cwd.clone(),
                        project_id: *project_id,
                        force_personal: *force_personal,
                        personal_user_id: personal_user_id.clone(),
                        local: *local,
                        daemon: *daemon,
                        require_daemon: *require_daemon,
                    },
                )
                .await
            }
        },
        Commands::Evidence { command } => match command {
            EvidenceCommands::List {
                scope,
                query,
                limit,
                format,
                principal_id,
                local,
                daemon,
                require_daemon,
            } => {
                commands::evidence::run_list(
                    &ctx,
                    commands::evidence::ListOptions {
                        scope: scope.clone(),
                        query: query.clone(),
                        limit: *limit,
                        format: format.clone(),
                        principal_id: principal_id.clone(),
                        local: *local,
                        daemon: *daemon,
                        require_daemon: *require_daemon,
                    },
                )
                .await
            }
            EvidenceCommands::Search {
                scope,
                query,
                limit,
                format,
                principal_id,
                local,
                daemon,
                require_daemon,
            } => {
                commands::evidence::run_list(
                    &ctx,
                    commands::evidence::ListOptions {
                        scope: scope.clone(),
                        query: Some(query.clone()),
                        limit: *limit,
                        format: format.clone(),
                        principal_id: principal_id.clone(),
                        local: *local,
                        daemon: *daemon,
                        require_daemon: *require_daemon,
                    },
                )
                .await
            }
            EvidenceCommands::Show {
                id,
                scope,
                format,
                max_chars,
                principal_id,
                local,
                daemon,
                require_daemon,
            } => {
                commands::evidence::run_show(
                    &ctx,
                    commands::evidence::ShowOptions {
                        id: id.clone(),
                        scope: scope.clone(),
                        format: format.clone(),
                        max_chars: *max_chars,
                        principal_id: principal_id.clone(),
                        local: *local,
                        daemon: *daemon,
                        require_daemon: *require_daemon,
                    },
                )
                .await
            }
        },
        Commands::Source { command } => match command {
            SourceCommands::List {
                scope,
                limit,
                format,
                principal_id,
                local,
                daemon,
                require_daemon,
            } => {
                commands::source::run_list(
                    &ctx,
                    commands::source::ListOptions {
                        scope: scope.clone(),
                        limit: *limit,
                        format: format.clone(),
                        principal_id: principal_id.clone(),
                        local: *local,
                        daemon: *daemon,
                        require_daemon: *require_daemon,
                    },
                )
                .await
            }
            SourceCommands::Show {
                id,
                scope,
                format,
                principal_id,
                local,
                daemon,
                require_daemon,
            } => {
                commands::source::run_show(
                    &ctx,
                    commands::source::ShowOptions {
                        id: id.clone(),
                        scope: scope.clone(),
                        format: format.clone(),
                        principal_id: principal_id.clone(),
                        local: *local,
                        daemon: *daemon,
                        require_daemon: *require_daemon,
                    },
                )
                .await
            }
        },
        Commands::Conclusion { command } => match command {
            ConclusionCommands::Propose {
                claim,
                evidence,
                scope,
                format,
                principal_id,
                command_id,
                local,
                daemon,
                require_daemon,
            } => {
                commands::conclusion::run_propose(
                    &ctx,
                    commands::conclusion::ProposeOptions {
                        statement: claim.clone(),
                        evidence: evidence.clone(),
                        scope: scope.clone(),
                        format: format.clone(),
                        principal_id: principal_id.clone(),
                        command_id: command_id.clone(),
                        local: *local,
                        daemon: *daemon,
                        require_daemon: *require_daemon,
                    },
                )
                .await
            }
        },
        Commands::Decision { command } => match command {
            DecisionCommands::Propose {
                statement,
                title,
                conclusions,
                evidence,
                scope,
                format,
                principal_id,
                command_id,
                local,
                daemon,
                require_daemon,
            } => {
                commands::decision::run_propose(
                    &ctx,
                    commands::decision::ProposeOptions {
                        statement: statement.clone(),
                        title: title.clone(),
                        conclusions: conclusions.clone(),
                        evidence: evidence.clone(),
                        scope: scope.clone(),
                        format: format.clone(),
                        principal_id: principal_id.clone(),
                        command_id: command_id.clone(),
                        local: *local,
                        daemon: *daemon,
                        require_daemon: *require_daemon,
                    },
                )
                .await
            }
        },
        Commands::Review { command } => match command {
            ReviewCommands::List {
                scope,
                status,
                format,
                principal_id,
                local,
                daemon,
                require_daemon,
            } => {
                commands::review::run_list(
                    &ctx,
                    commands::review::ListOptions {
                        scope: scope.clone(),
                        status: status.clone(),
                        format: format.clone(),
                        principal_id: principal_id.clone(),
                        local: *local,
                        daemon: *daemon,
                        require_daemon: *require_daemon,
                    },
                )
                .await
            }
            ReviewCommands::Resolve {
                id,
                resolution,
                scope,
                note,
                format,
                principal_id,
                command_id,
                local,
                daemon,
                require_daemon,
            } => {
                commands::review::run_resolve(
                    &ctx,
                    commands::review::ResolveOptions {
                        id: id.clone(),
                        resolution: resolution.clone(),
                        scope: scope.clone(),
                        note: note.clone(),
                        format: format.clone(),
                        principal_id: principal_id.clone(),
                        command_id: command_id.clone(),
                        local: *local,
                        daemon: *daemon,
                        require_daemon: *require_daemon,
                    },
                )
                .await
            }
        },
        Commands::Policy { command } => match command {
            PolicyCommands::Show {
                scope,
                format,
                principal_id,
            } => commands::policy_cmd::run_show(
                &ctx,
                commands::policy_cmd::ShowOptions {
                    scope: scope.clone(),
                    format: format.clone(),
                    principal_id: principal_id.clone(),
                },
            ),
            PolicyCommands::Check {
                capability,
                scope,
                format,
                principal_id,
            } => commands::policy_cmd::run_check(
                &ctx,
                commands::policy_cmd::CheckOptions {
                    capability: capability.clone(),
                    scope: scope.clone(),
                    format: format.clone(),
                    principal_id: principal_id.clone(),
                },
            ),
        },
        Commands::Erasure { command } => match command {
            ErasureCommands::Request {
                ids,
                reason,
                scope,
                format,
                principal_id,
                command_id,
                local,
                daemon,
                require_daemon,
            } => {
                let _ = &ctx;
                commands::erasure::run_request(commands::erasure::RequestOptions {
                    ids: ids.clone(),
                    reason: reason.clone(),
                    scope: scope.clone(),
                    format: format.clone(),
                    principal_id: principal_id.clone(),
                    command_id: command_id.clone(),
                    local: *local,
                    daemon: *daemon,
                    require_daemon: *require_daemon,
                })
                .await
            }
            ErasureCommands::Wipe {
                content_key_id,
                scope,
                reason,
                format,
                principal_id,
                command_id,
                dry_run,
                confirm,
                local,
                daemon,
                require_daemon,
            } => {
                let _ = &ctx;
                commands::erasure::run_wipe(commands::erasure::WipeOptions {
                    content_key_id: content_key_id.clone(),
                    scope: scope.clone(),
                    reason: reason.clone(),
                    format: format.clone(),
                    principal_id: principal_id.clone(),
                    command_id: command_id.clone(),
                    dry_run: *dry_run,
                    confirm: *confirm,
                    local: *local,
                    daemon: *daemon,
                    require_daemon: *require_daemon,
                })
                .await
            }
        },
        Commands::Retention { command } => match command {
            RetentionCommands::Plan { format } => commands::retention::run_plan(
                &ctx,
                commands::retention::PlanOptions {
                    format: format.clone(),
                },
            ),
            RetentionCommands::Apply {
                format,
                confirm,
                dry_run,
                command_id,
                scope,
                principal_id,
            } => commands::retention::run_apply(
                &ctx,
                commands::retention::ApplyOptions {
                    format: format.clone(),
                    confirm: *confirm,
                    dry_run: *dry_run,
                    command_id: command_id.clone(),
                    scope: scope.clone(),
                    principal_id: principal_id.clone(),
                },
            ),
        },
        Commands::Ingest { dry_run } => commands::ingest::run(&ctx, *dry_run),
        Commands::Recall {
            query,
            limit,
            project_id,
            session_id,
            session_prefix,
            format,
            semantic,
            graph_boost,
            graph_hop_depth,
            quiet,
            no_bridge,
            global,
            session_last,
        } => {
            // T86: `-` as the query reads the query string from stdin until EOF
            let effective_query = if query == "-" {
                read_query_from_stdin()?
            } else {
                query.clone()
            };
            // T112: --global searches across all projects and sessions;
            // default is project-scoped with no session filter.
            let (effective_project_id, effective_session_id) = if *global {
                (None, None)
            } else {
                (*project_id, *session_id)
            };
            commands::recall::run(
                &ctx,
                commands::recall::RecallRunOptions {
                    query: effective_query,
                    limit: *limit,
                    project_id: effective_project_id,
                    session_id: effective_session_id,
                    session_last: *session_last,
                    session_prefix: session_prefix.clone(),
                    format: format.clone(),
                    semantic: *semantic,
                    graph_boost: *graph_boost,
                    graph_hop_depth: *graph_hop_depth,
                    quiet: *quiet,
                    no_bridge: *no_bridge,
                    global: *global,
                },
            )
        }
        Commands::Preflight {
            max_words,
            project_id,
            pretty,
            format,
            scope,
            summary,
            global,
            stdin: use_stdin,
        } => {
            // T86: --stdin reads a JSON object {"max_words":N,"scope":[...]} from stdin
            let (effective_max_words, effective_scope) = if *use_stdin {
                let json_input = read_json_from_stdin()?;
                let mw = json_input["max_words"]
                    .as_u64()
                    .map(|n| n as usize)
                    .unwrap_or(*max_words);
                let sc = json_input["scope"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(|| scope.clone());
                (mw, sc)
            } else {
                (*max_words, scope.clone())
            };
            commands::preflight::run(
                &ctx,
                commands::preflight::PreflightRunOptions {
                    max_words: effective_max_words,
                    project_id: *project_id,
                    pretty: *pretty,
                    format: format.clone(),
                    scope: effective_scope,
                    summary: *summary,
                    global: *global,
                },
            )
        }
        Commands::Nightly {
            schedule,
            unschedule,
            start_time,
            status,
            skip_import,
            run_as_system,
            dry_run,
        } => {
            commands::nightly::run(
                &ctx,
                *schedule,
                *unschedule,
                start_time.clone(),
                *status,
                *skip_import,
                *run_as_system,
                *dry_run,
            )
            .await
        }
        Commands::Backup { command, dry_run } => match command {
            Some(BackupCommands::Restore {
                path,
                force,
                dry_run,
            }) => commands::backup::run_restore(&ctx, path.clone(), *force, *dry_run).await,
            Some(BackupCommands::Create {
                output_dir,
                keep,
                no_prune,
                dry_run,
            }) => {
                let effective_keep = if *no_prune { None } else { keep.or(Some(10)) };
                let is_default_retention = !*no_prune && keep.is_none();
                commands::backup::run_create(
                    &ctx,
                    output_dir.clone(),
                    effective_keep,
                    *dry_run,
                    is_default_retention,
                )
            }
            Some(BackupCommands::Prune {
                keep,
                older_than,
                dry_run,
            }) => commands::backup::run_prune(&ctx, *keep, older_than.clone(), *dry_run),
            Some(BackupCommands::List { quiet }) => commands::backup::run_list(&ctx, *quiet),
            Some(BackupCommands::Verify { path, full, format }) => {
                commands::backup::run_verify(&ctx, path.clone(), *full, format.clone())
            }
            None => commands::backup::run_create(&ctx, None, Some(10), *dry_run, true),
        },
        Commands::Forget {
            memory_id,
            match_query,
            force,
            list_forgotten,
            restore,
            dry_run,
        } => commands::forget::run(
            &ctx,
            memory_id.clone(),
            match_query.clone(),
            *force,
            *list_forgotten,
            restore.clone(),
            *dry_run,
        ),
        Commands::StopSession { session_id } => {
            commands::stop_session::run(&ctx, session_id.clone())
        }
        Commands::Context {
            new_project,
            new_session,
            show,
            tx_id,
        } => commands::context::run(&ctx, *new_project, *new_session, *show, tx_id.clone()),
        Commands::Pin {
            content,
            role,
            privacy,
            stdin,
            tags,
            tx_id,
            dry_run,
        } => {
            if *stdin {
                commands::pin::run_stdin(
                    &ctx,
                    role.clone(),
                    privacy.clone(),
                    tags.clone(),
                    tx_id.clone(),
                    *dry_run,
                )
            } else if let Some(c) = content {
                commands::pin::run(
                    &ctx,
                    c.clone(),
                    role.clone(),
                    privacy.clone(),
                    tags.clone(),
                    tx_id.clone(),
                    *dry_run,
                )
            } else {
                Err("Either provide content as a positional argument or use --stdin to read from stdin.".into())
            }
        }
        Commands::Device { command } => match command {
            DeviceCommands::Bootstrap => commands::device::run_bootstrap(&ctx),
            DeviceCommands::Fingerprint { raw } => commands::device::run_fingerprint(&ctx, *raw),
            DeviceCommands::List => commands::device::run_list(&ctx),
            DeviceCommands::PackageExport {
                out,
                write_private_key,
            } => commands::device::run_package_export(out.clone(), write_private_key.clone()),
            DeviceCommands::Enroll { package, yes } => {
                commands::device::run_enroll(&ctx, package.clone(), *yes)
            }
            DeviceCommands::Revoke { device_id } => commands::device::run_revoke(&ctx, device_id),
        },
        Commands::Replicate { command } => match command {
            ReplicateCommands::Status {
                fake_relay,
                format,
                quiet,
            } => {
                let format_json = format.as_deref() == Some("json");
                commands::replicate::run_status(&ctx, fake_relay.clone(), format_json, *quiet)
            }
            ReplicateCommands::Cursors { format } => {
                let format_json = format.as_deref() == Some("json");
                commands::replicate::run_cursors(&ctx, format_json)
            }
            ReplicateCommands::Push {
                fake_relay,
                format,
                quiet,
            } => {
                let format_json = format.as_deref() == Some("json");
                commands::replicate::run_push(&ctx, fake_relay.clone(), format_json, *quiet)
            }
            ReplicateCommands::Pull {
                fake_relay,
                format,
                quiet,
            } => {
                let format_json = format.as_deref() == Some("json");
                commands::replicate::run_pull(&ctx, fake_relay.clone(), format_json, *quiet)
            }
        },
        Commands::Safety { command } => match command {
            SafetyCommands::Sync { limit, dry_run } => {
                commands::safety::run(&ctx, *limit, *dry_run)
            }
        },
        Commands::Sync { command } => match command {
            SyncCommands::Pull {
                from_file,
                hotspots,
                ledger,
                quiet,
                schema,
            } => {
                if *schema {
                    print_schema(SCHEMA_SYNC_PULL, "AI-Brains sync pull NDJSON record")
                } else {
                    commands::sync::run_pull(&ctx, from_file.clone(), *hotspots, *ledger, *quiet)
                }
            }
            SyncCommands::Push {
                with_impact,
                with_verify,
                quiet,
            } => commands::sync::run_push(&ctx, *with_impact, *with_verify, *quiet),
            SyncCommands::Query {
                query,
                format,
                quiet,
                global,
                no_bridge,
            } => {
                commands::sync::run_query(
                    &ctx,
                    query.clone(),
                    format.clone(),
                    *quiet,
                    *global,
                    *no_bridge,
                )
                .await
            }
        },
        Commands::AntigravityImport { days } => commands::antigravity_import::run(&ctx, *days),
        Commands::AgyHook { payload, schema } => {
            if *schema {
                print_schema(SCHEMA_AGY_HOOK, "AI-Brains agy-hook payload")
            } else if let Some(p) = payload {
                commands::agy_hook::run(&ctx, p)
            } else {
                Err(
                    "Either provide --payload <json> or use --schema to print the payload schema."
                        .into(),
                )
            }
        }
        Commands::Daemon { command } => match command {
            DaemonCommands::Start => commands::daemon::run_start(&ctx),
            DaemonCommands::Status => {
                unreachable!("status handled before AppContext")
            }
            DaemonCommands::Schedule {
                dry_run,
                run_as_system,
            } => commands::daemon::run_schedule(&ctx, *dry_run, *run_as_system),
            DaemonCommands::Unschedule { dry_run } => {
                commands::daemon::run_unschedule(&ctx, *dry_run)
            }
            DaemonCommands::Install { dry_run } => commands::daemon::run_install(&ctx, *dry_run),
            DaemonCommands::Uninstall { dry_run } => {
                commands::daemon::run_uninstall(&ctx, *dry_run)
            }
            DaemonCommands::Stop { force } => commands::daemon::run_stop(&ctx, *force).await,
            DaemonCommands::Update => commands::daemon::run_update(&ctx).await,
        },
        Commands::Project { command } => match command {
            ProjectCommands::List => commands::project::list(&ctx),
            ProjectCommands::Resolve {
                alias_positional,
                alias,
            } => commands::project::resolve(&ctx, alias_positional.clone(), alias.clone()),
            ProjectCommands::Detect { export } => commands::project::detect(&ctx, *export),
            ProjectCommands::SetAlias { project_id, alias } => {
                commands::project::set_alias(&ctx, project_id, alias)
            }
        },
        #[cfg(feature = "graph")]
        Commands::Graph { command, .. } => match command {
            GraphCommands::Rebuild => commands::graph::rebuild(&ctx),
            GraphCommands::Neighbors { memory_id } => commands::graph::neighbors(&ctx, memory_id),
            GraphCommands::Hierarchy { memory_id } => commands::graph::hierarchy(&ctx, memory_id),
            GraphCommands::Session { session_id } => commands::graph::session(&ctx, session_id),
            GraphCommands::Update => commands::graph::update(&ctx),
        },
        #[cfg(not(feature = "graph"))]
        Commands::Graph { .. } => {
            println!(
                "{}: The graph subcommand requires a --features graph build.",
                commands::governed_common::FEATURE_UNAVAILABLE
            );
            println!(
                "Reinstall with: cargo install --path crates/ai-brains-cli --locked --features graph"
            );
            std::process::exit(commands::governed_common::exit_code_feature_unavailable());
        }
    }
}
