mod commands;
mod context;
mod daemon_client;
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
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the vault database
    #[arg(long, env = "AI_BRAINS_VAULT_PATH")]
    vault_path: Option<PathBuf>,

    /// Hex-encoded key for the vault (or dummy)
    #[arg(long, env = "AI_BRAINS_KEY")]
    key: Option<String>,

    /// Skip auto-discovery of project/session from .env. When set, the CLI
    /// will not clear inherited `AI_BRAINS_PROJECT_ID` / `AI_BRAINS_SESSION_ID`
    /// env vars or load a project-local `.env` file. Use this in CI, hooks,
    /// or any non-interactive flow where the caller has already configured
    /// the env vars explicitly.
    #[arg(long, global = true)]
    no_project_context: bool,

    /// Tracing output format: compact (default), full, json, minimal, or off
    #[arg(long, global = true, default_value = "compact")]
    log_format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new vault
    Init {
        /// Re-initialize even when the vault already contains data
        #[arg(long)]
        force: bool,
    },
    /// Ingest a conversation turn (reads JSON from stdin)
    Ingest {
        /// Preview what would be ingested without writing to the vault
        #[arg(long)]
        dry_run: bool,
    },
    /// Recall memories based on a query
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
        /// Skip the ChangeGuard bridge query and use only local vault FTS5 +
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
    },
    /// Create a timestamped backup of the vault
    Backup {
        #[command(subcommand)]
        command: Option<BackupCommands>,
        /// Preview what would happen without creating the backup file.
        /// Only applies when no subcommand is given (defaults to create).
        #[arg(long)]
        dry_run: bool,
    },
    /// Forget a specific memory (soft delete)
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
    StopSession {
        /// Session ID to stop
        session_id: String,
    },
    /// Initialize or refresh the project context (writes local .env)
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
        /// Optional ChangeGuard transaction ID to link this context to
        #[arg(long, env = "CHANGEGUARD_TX_ID")]
        tx_id: Option<String>,
    },
    /// Pin a high-level decision or constraint directly to the vault
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
        /// Optional ChangeGuard transaction ID to link this pin to
        #[arg(long, env = "CHANGEGUARD_TX_ID")]
        tx_id: Option<String>,
        /// Preview what would be pinned without writing to the vault
        #[arg(long)]
        dry_run: bool,
    },
    /// Manage repository safety signals
    Safety {
        #[command(subcommand)]
        command: SafetyCommands,
    },
    /// Sync structured records from external tools (ChangeGuard)
    Sync {
        #[command(subcommand)]
        command: SyncCommands,
    },
    /// Import Antigravity conversation logs into the vault
    AntigravityImport {
        /// Only import sessions modified within the last N days
        #[arg(short, long, default_value_t = 30)]
        days: usize,
    },
    /// Process an Antigravity CLI (agy) hook payload
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
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
    /// Manage projects and resolve aliases
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    /// Graph operations
    #[cfg(feature = "graph")]
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
    /// Graph operations (requires --features graph)
    #[cfg(not(feature = "graph"))]
    Graph {
        #[command(subcommand)]
        command: GraphCommands,

        /// Trailing arguments accepted when the graph feature is not enabled
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        args: Vec<String>,
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
    /// Stop the running daemon gracefully
    Stop {
        /// Forcefully terminate the process if it doesn't respond to shutdown signal
        #[arg(long, short)]
        force: bool,
    },
    /// Stop daemon, install updated binaries, then restart (run from workspace root)
    Update,
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
        /// Export hotspot data from ChangeGuard
        #[arg(long)]
        hotspots: bool,
        /// Export ledger delta data from ChangeGuard
        #[arg(long)]
        ledger: bool,
        /// Suppress ChangeGuard error messages
        #[arg(long, short)]
        quiet: bool,
        /// Print the JSON Schema for the expected NDJSON record shape and exit.
        /// The schema is also at `Docs/schemas/sync-pull-record.json`.
        #[arg(long)]
        schema: bool,
    },
    /// Push current context to ChangeGuard
    Push {
        /// Include impact context
        #[arg(long)]
        with_impact: bool,
        /// Include verification context
        #[arg(long)]
        with_verify: bool,
        /// Suppress ChangeGuard error messages
        #[arg(long, short)]
        quiet: bool,
    },
    /// Unified query across AI-Brains and ChangeGuard
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
        /// Skip the ChangeGuard bridge query and use only local vault recall.
        #[arg(long)]
        no_bridge: bool,
    },
}

#[derive(Subcommand, Clone)]
pub enum SafetyCommands {
    /// Synchronize ChangeGuard hotspots into the AI-Brains vault
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
        )
    })
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
            if let Ok(existing) = std::env::var(&key) {
                if existing != value {
                    eprintln!(
                        "Warning: local .env {} overrides inherited shell value {}.",
                        key, existing
                    );
                }
            }
        } else if let Ok(existing) = std::env::var(&key) {
            if existing != value {
                tracing::debug!(
                    "local .env {} overrides inherited shell value for this command",
                    key
                );
            }
        }

        std::env::set_var(key, value);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
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
    // T80: --no-project-context disables this whole block so that CI, hooks,
    // and any non-interactive caller can supply env vars explicitly.
    if !no_project_context {
        let project_env = std::path::Path::new(".env");
        if !project_env.exists() {
            std::env::remove_var("AI_BRAINS_PROJECT_ID");
            std::env::remove_var("AI_BRAINS_SESSION_ID");
        } else {
            dotenvy::dotenv().ok();
            apply_local_project_context_env(project_env, warn_on_project_context_override);
        }

        // Fallback to global config in ~/.ai-brains/.env if AI_BRAINS_VAULT_PATH not set yet
        if std::env::var("AI_BRAINS_VAULT_PATH").is_err() {
            if let Some(mut home) = dirs::home_dir() {
                home.push(".ai-brains");
                home.push(".env");
                if home.exists() {
                    dotenvy::from_path(home).ok();
                }
            }
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

    runtime.block_on(async {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!("\nInterrupted by user. Exiting...");
                std::process::exit(130);
            }
            res = async {
                let cli = Cli::parse();
                run(cli).await
            } => {
                if let Err(err) = res {
                    use ai_brains_contracts::response::{ApiError, ApiResult};
                    let api_error = ApiError::new("COMMAND_FAILED", err.to_string());
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
    });
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = AppContext::from_cli(cli.vault_path.clone(), cli.key.clone())?;
    match &cli.command {
        Commands::Init { force } => commands::init::run(&ctx, *force),
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
        } => {
            commands::nightly::run(
                &ctx,
                *schedule,
                *unschedule,
                start_time.clone(),
                *status,
                *skip_import,
                *run_as_system,
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
            DaemonCommands::Status => commands::daemon::run_status(&ctx).await,
            DaemonCommands::Schedule {
                dry_run,
                run_as_system,
            } => commands::daemon::run_schedule(&ctx, *dry_run, *run_as_system),
            DaemonCommands::Unschedule { dry_run } => {
                commands::daemon::run_unschedule(&ctx, *dry_run)
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
            println!("The graph subcommand requires a --features graph build.");
            println!("Reinstall with: cargo install --path crates/ai-brains-cli --locked --features graph");
            Ok(())
        }
    }
}
