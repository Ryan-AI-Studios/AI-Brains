mod artifact_security;
mod commands;
mod context;
mod daemon_client;
mod daemon_probe;
mod elevation;
mod env_warn;
mod env_warn_session;
mod graph_density;
mod harness;
mod help_ia;
mod key_resolve;
mod live_graph;
mod verify_report;

/// JSON Schema for `ai-bbrains agy-hook --payload`. Bundled at compile time
/// so `--schema` works regardless of cwd. The source-of-truth file lives at
/// `Docs/schemas/agy-hook-payload.json`; changes there must be mirrored here.
const SCHEMA_AGY_HOOK: &str = include_str!("../../../Docs/schemas/agy-hook-payload.json");

/// JSON Schema for `ai-brains grok-hook --payload`. Source-of-truth at
/// `Docs/schemas/grok-hook-payload.json`.
const SCHEMA_GROK_HOOK: &str = include_str!("../../../Docs/schemas/grok-hook-payload.json");

/// JSON Schema for `ai-brains opencode-hook --payload`. Source-of-truth at
/// `Docs/schemas/opencode-hook-payload.json`.
const SCHEMA_OPENCODE_HOOK: &str = include_str!("../../../Docs/schemas/opencode-hook-payload.json");

/// JSON Schema for `ai-brains claude-hook --payload`. Source-of-truth at
/// `Docs/schemas/claude-hook-payload.json`.
const SCHEMA_CLAUDE_HOOK: &str = include_str!("../../../Docs/schemas/claude-hook-payload.json");

/// JSON Schema for `ai-brains codex-hook --payload`. Source-of-truth at
/// `Docs/schemas/codex-hook-payload.json`.
const SCHEMA_CODEX_HOOK: &str = include_str!("../../../Docs/schemas/codex-hook-payload.json");

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

/// Default tracing EnvFilter when `RUST_LOG` is unset (T118 scoped + T208 graph).
///
/// `ai_brains_graph=warn` is required defense-in-depth: EnvFilter prefix-matches
/// `ai_brains=info` onto `ai_brains_graph`, which would otherwise enable graph-crate
/// INFO. Operators re-enable Cozo lifecycle with `RUST_LOG=ai_brains_graph=debug`.
const DEFAULT_ENV_FILTER: &str =
    "warn,ai_brains=info,ai_brains_cli=info,ai_brains_brain=info,ai_brains_graph=warn";

#[cfg(test)]
mod tests {
    use super::DEFAULT_ENV_FILTER;
    use clap::Parser;

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

    /// T208 AC7: default filter must pin `ai_brains_graph=warn` (F8).
    #[test]
    #[allow(non_snake_case)]
    fn default_env_filter__contains_ai_brains_graph_warn() {
        assert!(
            DEFAULT_ENV_FILTER.contains("ai_brains_graph=warn"),
            "DEFAULT_ENV_FILTER must include ai_brains_graph=warn; got: {DEFAULT_ENV_FILTER}"
        );
    }

    /// T247 AC7/AC13: `--quick` requires `--status` (clap exit 2; no vault work).
    #[test]
    #[allow(non_snake_case)]
    fn nightly_quick__without_status__clap_requires_status() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from(["ai-brains", "nightly", "--quick"]) {
            Ok(_) => panic!("expected clap to reject --quick without --status"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    /// T247 AC7: `--quick` conflicts with `--schedule` / `--unschedule`.
    #[test]
    #[allow(non_snake_case)]
    fn nightly_quick__with_schedule__clap_conflicts() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "nightly",
            "--status",
            "--quick",
            "--schedule",
        ]) {
            Ok(_) => panic!("expected clap to reject --quick with --schedule"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    /// T247 F1: `--status --quick` is a valid parse (no runtime `if !status && quick`).
    #[test]
    #[allow(non_snake_case)]
    fn nightly_quick__with_status__parses() {
        let cli = match super::Cli::try_parse_from(["ai-brains", "nightly", "--status", "--quick"])
        {
            Ok(c) => c,
            Err(e) => panic!("expected --status --quick to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Nightly { status, quick, .. } => {
                assert!(status);
                assert!(quick);
            }
            _ => panic!("expected Commands::Nightly"),
        }
    }

    /// T255 AC2: `--format` requires `--status`.
    #[test]
    #[allow(non_snake_case)]
    fn nightly_format__without_status__clap_requires_status() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from(["ai-brains", "nightly", "--format", "json"]) {
            Ok(_) => panic!("expected clap to reject --format without --status"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    /// T255 AC2: unknown `--format` is clap InvalidValue (exit 2).
    #[test]
    #[allow(non_snake_case)]
    fn nightly_status__format_xml__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "nightly",
            "--status",
            "--format",
            "xml",
        ]) {
            Ok(_) => panic!("expected clap to reject --format xml"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T255 AC2 / T249 AC16: `--format` tokens are case-sensitive (`Pretty` is not `pretty`).
    #[test]
    #[allow(non_snake_case)]
    fn nightly_status__format_Pretty__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "nightly",
            "--status",
            "--format",
            "Pretty",
        ]) {
            Ok(_) => panic!("expected clap to reject --format Pretty"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T255 AC2 / T249 AC16: `--format` tokens are case-sensitive (`JSON` is not `json`).
    #[test]
    #[allow(non_snake_case)]
    fn nightly_status__format_JSON__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "nightly",
            "--status",
            "--format",
            "JSON",
        ]) {
            Ok(_) => panic!("expected clap to reject --format JSON"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T255 F2: omitted `--format` defaults to `human` (pipes stay human).
    #[test]
    #[allow(non_snake_case)]
    fn nightly_status__default_format__human() {
        let cli = match super::Cli::try_parse_from(["ai-brains", "nightly", "--status"]) {
            Ok(c) => c,
            Err(e) => panic!("expected nightly --status to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Nightly { status, format, .. } => {
                assert!(status);
                assert_eq!(format, "human");
            }
            _ => panic!("expected Commands::Nightly"),
        }
    }

    /// T246 AC12: unknown `--format` is clap InvalidValue (exit 2), not resolve passthrough.
    #[test]
    #[allow(non_snake_case)]
    fn graph_neighbors__format_xml__clap_invalid_value() {
        use clap::Parser;
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "graph",
            "neighbors",
            "--format",
            "xml",
            "id",
        ]) {
            Ok(_) => panic!("expected clap to reject --format xml"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T290 AC10: list after_help names copy-paste recall + Pinned.
    #[test]
    #[allow(non_snake_case)]
    fn evidence_list__help__names_pinned_and_recall_query() {
        use clap::Parser;
        let err = match super::Cli::try_parse_from(["ai-brains", "evidence", "list", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string().to_lowercase();
        assert!(
            help.contains("pinned") && help.contains("what did we decide") && help.contains("none"),
            "AC10: evidence list after_help names Pinned + recall query + (none); got: {help}"
        );
    }

    /// T290 AC10: progressive after_help names operator query + Pinned (not ellipsis-only).
    #[test]
    #[allow(non_snake_case)]
    fn query_progressive__help__names_operator_query_and_pinned() {
        use clap::Parser;
        let err = match super::Cli::try_parse_from(["ai-brains", "query", "progressive", "--help"])
        {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string().to_lowercase();
        assert!(
            help.contains("pinned")
                && (help.contains("operator query") || help.contains("ellipsis")),
            "AC10: progressive after_help names Pinned + operator query; got: {help}"
        );
    }

    /// T292 AC6: `--format JSON` is clap InvalidValue (not `OutputFormat::parse`).
    #[test]
    #[allow(non_snake_case)]
    fn policy_check__format_JSON__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "policy",
            "check",
            "--capability",
            "ReadEvidence",
            "--format",
            "JSON",
        ]) {
            Ok(_) => panic!("expected clap to reject --format JSON"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T292 AC6: `--format Pretty` is clap InvalidValue.
    #[test]
    #[allow(non_snake_case)]
    fn policy_check__format_Pretty__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "policy",
            "check",
            "--capability",
            "ReadEvidence",
            "--format",
            "Pretty",
        ]) {
            Ok(_) => panic!("expected clap to reject --format Pretty"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T292 AC6: `--format json` parses.
    #[test]
    #[allow(non_snake_case)]
    fn policy_check__format_json__parses() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "policy",
            "check",
            "--capability",
            "ReadEvidence",
            "--format",
            "json",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected policy check --format json to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Policy {
                command: super::PolicyCommands::Check { format, .. },
            } => assert_eq!(format, "json"),
            _ => panic!("expected Policy::Check"),
        }
    }

    /// T292 AC6: `--format pretty` parses.
    #[test]
    #[allow(non_snake_case)]
    fn policy_check__format_pretty__parses() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "policy",
            "check",
            "--capability",
            "ReadEvidence",
            "--format",
            "pretty",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected policy check --format pretty to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Policy {
                command: super::PolicyCommands::Check { format, .. },
            } => assert_eq!(format, "pretty"),
            _ => panic!("expected Policy::Check"),
        }
    }

    /// T292 AC6: omitted `--format` defaults to `auto`.
    #[test]
    #[allow(non_snake_case)]
    fn policy_check__default_format__auto() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "policy",
            "check",
            "--capability",
            "ReadEvidence",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected policy check with no --format to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Policy {
                command: super::PolicyCommands::Check { format, .. },
            } => assert_eq!(format, "auto"),
            _ => panic!("expected Policy::Check"),
        }
    }

    /// T292 AC8: `policy show --help` still defaults format to json (Family D).
    #[test]
    #[allow(non_snake_case)]
    fn policy_show__help__default_format_json() {
        let err = match super::Cli::try_parse_from(["ai-brains", "policy", "show", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("default: json") || help.contains("[default: json]"),
            "AC8: policy show --help must keep default json; got: {help}"
        );
    }

    /// T292 AC8: `policy bootstrap --help` still defaults format to json (Family D).
    #[test]
    #[allow(non_snake_case)]
    fn policy_bootstrap__help__default_format_json() {
        let err = match super::Cli::try_parse_from(["ai-brains", "policy", "bootstrap", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("default: json") || help.contains("[default: json]"),
            "AC8: policy bootstrap --help must keep default json; got: {help}"
        );
    }

    /// T292 AC9/F29: `policy check --help` names auto/TTY and catalog block matches CAPABILITY_CATALOG.
    #[test]
    #[allow(non_snake_case)]
    fn policy_check__help__names_auto_tty_and_catalog() {
        let err = match super::Cli::try_parse_from(["ai-brains", "policy", "check", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        let lower = help.to_lowercase();
        assert!(
            lower.contains("auto") && (lower.contains("tty") || lower.contains("pipe")),
            "AC9: help must name auto + TTY/pipe; got: {help}"
        );
        assert!(
            !lower.contains("json-only") && !lower.contains("json only"),
            "AC9: help must not claim JSON-only; got: {help}"
        );
        // F29: after_help catalog block must stay byte-stable with CAPABILITY_CATALOG.
        let catalog_block = crate::commands::governed_common::CAPABILITY_CATALOG
            .iter()
            .map(|line| format!("  {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            help.contains(&catalog_block),
            "AC9/F29: help catalog block must match CAPABILITY_CATALOG byte-for-byte; missing:\n{catalog_block}\nhelp:\n{help}"
        );
    }

    /// T291 AC7: `--format JSON` is clap InvalidValue (not `OutputFormat::parse`).
    #[test]
    #[allow(non_snake_case)]
    fn query_trace__format_JSON__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "query",
            "trace",
            "x",
            "--format",
            "JSON",
        ]) {
            Ok(_) => panic!("expected clap to reject --format JSON"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T291 AC7: `--format json` parses (default envelope/DTO).
    #[test]
    #[allow(non_snake_case)]
    fn query_trace__format_json__parses() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "query",
            "trace",
            "x",
            "--format",
            "json",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected query trace --format json to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Query {
                command: super::GovernedQueryCommands::Trace { format, .. },
            } => assert_eq!(format, "json"),
            _ => panic!("expected Query::Trace"),
        }
    }

    /// T314 AC1: bare `--dry-run` on progressive parses as true.
    #[test]
    #[allow(non_snake_case)]
    fn query_progressive__dry_run_bare__parses_true() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "query",
            "progressive",
            "q",
            "--dry-run",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected progressive bare --dry-run to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Query {
                command: super::GovernedQueryCommands::Progressive { dry_run, .. },
            } => assert!(dry_run, "bare --dry-run must be true"),
            _ => panic!("expected Query::Progressive"),
        }
    }

    /// T314 AC2: `--dry-run false` / `true` / omitted on progressive.
    #[test]
    #[allow(non_snake_case)]
    fn query_progressive__dry_run_false__parses_false() {
        let false_cli = match super::Cli::try_parse_from([
            "ai-brains",
            "query",
            "progressive",
            "q",
            "--dry-run",
            "false",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected --dry-run false to parse: {e}"),
        };
        match *false_cli.command {
            super::Commands::Query {
                command: super::GovernedQueryCommands::Progressive { dry_run, .. },
            } => assert!(!dry_run, "--dry-run false must be false"),
            _ => panic!("expected Query::Progressive"),
        }

        let true_cli = match super::Cli::try_parse_from([
            "ai-brains",
            "query",
            "progressive",
            "q",
            "--dry-run",
            "true",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected --dry-run true to parse: {e}"),
        };
        match *true_cli.command {
            super::Commands::Query {
                command: super::GovernedQueryCommands::Progressive { dry_run, .. },
            } => assert!(dry_run, "--dry-run true must be true"),
            _ => panic!("expected Query::Progressive"),
        }

        let omitted = match super::Cli::try_parse_from(["ai-brains", "query", "progressive", "q"]) {
            Ok(c) => c,
            Err(e) => panic!("expected omitted --dry-run to parse: {e}"),
        };
        match *omitted.command {
            super::Commands::Query {
                command: super::GovernedQueryCommands::Progressive { dry_run, .. },
            } => assert!(dry_run, "omitted --dry-run must default true"),
            _ => panic!("expected Query::Progressive"),
        }
    }

    /// T314 AC3: bare `--dry-run` on briefing project parses as true.
    #[test]
    #[allow(non_snake_case)]
    fn briefing_project__dry_run_bare__parses_true() {
        let cli =
            match super::Cli::try_parse_from(["ai-brains", "briefing", "project", "--dry-run"]) {
                Ok(c) => c,
                Err(e) => panic!("expected briefing project bare --dry-run to parse: {e}"),
            };
        match *cli.command {
            super::Commands::Briefing {
                command: super::BriefingCommands::Project { dry_run, .. },
            } => assert!(dry_run, "bare --dry-run must be true"),
            _ => panic!("expected Briefing::Project"),
        }
    }

    /// T314 AC3: bare `--dry-run` on briefing personal parses as true.
    #[test]
    #[allow(non_snake_case)]
    fn briefing_personal__dry_run_bare__parses_true() {
        let cli =
            match super::Cli::try_parse_from(["ai-brains", "briefing", "personal", "--dry-run"]) {
                Ok(c) => c,
                Err(e) => panic!("expected briefing personal bare --dry-run to parse: {e}"),
            };
        match *cli.command {
            super::Commands::Briefing {
                command: super::BriefingCommands::Personal { dry_run, .. },
            } => assert!(dry_run, "bare --dry-run must be true"),
            _ => panic!("expected Briefing::Personal"),
        }
    }

    /// T314 AC4: expand `--format json` / human / default json.
    #[test]
    #[allow(non_snake_case)]
    fn query_expand__format_json__parses() {
        let json_cli = match super::Cli::try_parse_from([
            "ai-brains",
            "query",
            "expand",
            "x",
            "--format",
            "json",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected expand --format json to parse: {e}"),
        };
        match *json_cli.command {
            super::Commands::Query {
                command: super::GovernedQueryCommands::Expand { format, .. },
            } => assert_eq!(format, "json"),
            _ => panic!("expected Query::Expand"),
        }

        let human_cli = match super::Cli::try_parse_from([
            "ai-brains",
            "query",
            "expand",
            "x",
            "--format",
            "human",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected expand --format human to parse: {e}"),
        };
        match *human_cli.command {
            super::Commands::Query {
                command: super::GovernedQueryCommands::Expand { format, .. },
            } => assert_eq!(format, "human"),
            _ => panic!("expected Query::Expand"),
        }

        let default_cli = match super::Cli::try_parse_from(["ai-brains", "query", "expand", "x"]) {
            Ok(c) => c,
            Err(e) => panic!("expected expand default format to parse: {e}"),
        };
        match *default_cli.command {
            super::Commands::Query {
                command: super::GovernedQueryCommands::Expand { format, .. },
            } => assert_eq!(format, "json", "omitted --format must default json"),
            _ => panic!("expected Query::Expand"),
        }
    }

    /// T320 AC2: `status --format xml` is clap InvalidValue (exit 2).
    #[test]
    #[allow(non_snake_case)]
    fn status__format_xml__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from(["ai-brains", "status", "--format", "xml"]) {
            Ok(_) => panic!("expected clap to reject --format xml"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T320 AC2: `status --format JSON` is clap InvalidValue (case-sensitive).
    #[test]
    #[allow(non_snake_case)]
    fn status__format_JSON__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from(["ai-brains", "status", "--format", "JSON"]) {
            Ok(_) => panic!("expected clap to reject --format JSON"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T320 AC2: `daemon status` still parses (nested; not stolen by top-level status).
    #[test]
    #[allow(non_snake_case)]
    fn daemon_status__still_parses_alongside_top_level_status() {
        let cli = match super::Cli::try_parse_from(["ai-brains", "daemon", "status"]) {
            Ok(c) => c,
            Err(e) => panic!("expected daemon status to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Daemon {
                command: super::DaemonCommands::Status,
            } => {}
            _ => panic!("expected Daemon::Status"),
        }
        let status = match super::Cli::try_parse_from(["ai-brains", "status"]) {
            Ok(c) => c,
            Err(e) => panic!("expected top-level status to parse: {e}"),
        };
        match *status.command {
            super::Commands::Status { format } => assert_eq!(format, "auto"),
            _ => panic!("expected Commands::Status"),
        }
    }

    /// T314 AC5: expand `--format JSON` is clap InvalidValue.
    #[test]
    #[allow(non_snake_case)]
    fn query_expand__format_JSON__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "query",
            "expand",
            "x",
            "--format",
            "JSON",
        ]) {
            Ok(_) => panic!("expected clap to reject --format JSON"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T314 AC5: expand `--format xml` is clap InvalidValue.
    #[test]
    #[allow(non_snake_case)]
    fn query_expand__format_xml__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "query",
            "expand",
            "x",
            "--format",
            "xml",
        ]) {
            Ok(_) => panic!("expected clap to reject --format xml"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T314 AC6: scan-roots `--dry-run` parses (no-op alias).
    #[test]
    #[allow(non_snake_case)]
    fn scan_roots__dry_run__parses() {
        let cli =
            match super::Cli::try_parse_from(["ai-brains", "project", "scan-roots", "--dry-run"]) {
                Ok(c) => c,
                Err(e) => panic!("expected scan-roots --dry-run to parse: {e}"),
            };
        match *cli.command {
            super::Commands::Project {
                command: super::ProjectCommands::ScanRoots { dry_run, .. },
            } => assert!(dry_run, "--dry-run SetTrue must be true"),
            _ => panic!("expected ProjectCommands::ScanRoots"),
        }
    }

    /// T314 AC7 / F6: progressive still rejects `--format` (T290 F10).
    #[test]
    #[allow(non_snake_case)]
    fn query_progressive__format_json__unexpected_argument() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "query",
            "progressive",
            "q",
            "--format",
            "json",
        ]) {
            Ok(_) => panic!("expected progressive --format to fail clap parse"),
            Err(e) => e,
        };
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "AC7: progressive --format must be UnknownArgument (not a malformed value_parser)"
        );
        let rendered = err.to_string();
        assert!(
            rendered.contains("--format") || rendered.contains("unexpected"),
            "AC7: error should name --format; got: {rendered}"
        );
    }

    /// T291 AC10: `query trace --help` must not retain the scalar-null contract.
    #[test]
    #[allow(non_snake_case)]
    fn query_trace__help__names_envelope_not_json_token_null() {
        let err = match super::Cli::try_parse_from(["ai-brains", "query", "trace", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            !help.contains("JSON token null"),
            "AC10: help must not retain scalar-null contract; got: {help}"
        );
        let lower = help.to_lowercase();
        assert!(
            lower.contains("found")
                && lower.contains("next")
                && lower.contains("query progressive")
                && lower.contains("dry-run"),
            "AC10: help must name envelope + progressive persist; got: {help}"
        );
    }

    /// T270 AC12: `retention plan --help` names inventory / none_auto.
    #[test]
    #[allow(non_snake_case)]
    fn retention_plan__help__names_inventory_or_none_auto() {
        let err = match super::Cli::try_parse_from(["ai-brains", "retention", "plan", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("none_auto") || help.contains("inventory"),
            "AC12: after_help names none_auto or inventory; got: {help}"
        );
    }

    /// T248 AC10: unknown `--format` is clap InvalidValue (exit 2), not silent JSON.
    #[test]
    #[allow(non_snake_case)]
    fn retention_plan__format_xml__clap_invalid_value() {
        use clap::Parser;
        use clap::error::ErrorKind;
        let err =
            match super::Cli::try_parse_from(["ai-brains", "retention", "plan", "--format", "xml"])
            {
                Ok(_) => panic!("expected clap to reject --format xml"),
                Err(e) => e,
            };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T249 AC5: unknown `--format` is clap InvalidValue (exit 2), not silent JSON.
    #[test]
    #[allow(non_snake_case)]
    fn scope_resolve__format_xml__clap_invalid_value() {
        use clap::Parser;
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "scope",
            "resolve",
            "--format",
            "xml",
        ]) {
            Ok(_) => panic!("expected clap to reject --format xml"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T249 AC16: `--format` tokens are case-sensitive (`JSON` is not `json`).
    #[test]
    #[allow(non_snake_case)]
    fn scope_resolve__format_JSON__clap_invalid_value() {
        use clap::Parser;
        use clap::error::ErrorKind;
        let err =
            match super::Cli::try_parse_from(["ai-brains", "scope", "resolve", "--format", "JSON"])
            {
                Ok(_) => panic!("expected clap to reject --format JSON"),
                Err(e) => e,
            };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T249 AC16: `--format` tokens are case-sensitive (`Pretty` is not `pretty`).
    #[test]
    #[allow(non_snake_case)]
    fn scope_resolve__format_Pretty__clap_invalid_value() {
        use clap::Parser;
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "scope",
            "resolve",
            "--format",
            "Pretty",
        ]) {
            Ok(_) => panic!("expected clap to reject --format Pretty"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T250: `--compact` is a boolean flag on Preflight (not a `--format` token).
    #[test]
    #[allow(non_snake_case)]
    fn preflight__compact_pretty__parses() {
        use clap::Parser;
        let cli =
            match super::Cli::try_parse_from(["ai-brains", "preflight", "--pretty", "--compact"]) {
                Ok(c) => c,
                Err(e) => panic!("expected preflight --pretty --compact to parse: {e}"),
            };
        match *cli.command {
            super::Commands::Preflight {
                compact, pretty, ..
            } => {
                assert!(compact);
                assert!(pretty);
            }
            _ => panic!("expected Commands::Preflight"),
        }
    }

    /// T249 F1: omitted `--format` must default to `auto` (not the pre-T249 `json`).
    #[test]
    #[allow(non_snake_case)]
    fn scope_resolve__default_format__auto() {
        use clap::Parser;
        let cli = match super::Cli::try_parse_from(["ai-brains", "scope", "resolve"]) {
            Ok(c) => c,
            Err(e) => panic!("expected scope resolve with no --format to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Scope {
                command: super::ScopeCommands::Resolve { format, .. },
            } => {
                assert_eq!(format, "auto");
            }
            _ => panic!("expected Scope::Resolve"),
        }
    }

    /// T266 AC2: `--format pretty` parses on list-paths (not InvalidValue).
    #[test]
    #[allow(non_snake_case)]
    fn list_paths__format_pretty__parses() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "list-paths",
            "--format",
            "pretty",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected list-paths --format pretty to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Project {
                command: super::ProjectCommands::ListPaths { format, .. },
            } => {
                assert_eq!(format, "pretty");
            }
            _ => panic!("expected ProjectCommands::ListPaths"),
        }
    }

    /// T266 AC2: unknown `--format` is clap InvalidValue (exit 2).
    #[test]
    #[allow(non_snake_case)]
    fn list_paths__format_xml__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "list-paths",
            "--format",
            "xml",
        ]) {
            Ok(_) => panic!("expected clap to reject --format xml"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T266 AC2: `--format` tokens are case-sensitive (`JSON` is not `json`).
    #[test]
    #[allow(non_snake_case)]
    fn list_paths__format_JSON__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "list-paths",
            "--format",
            "JSON",
        ]) {
            Ok(_) => panic!("expected clap to reject --format JSON"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T266 AC2: `--format` tokens are case-sensitive (`Pretty` is not `pretty`).
    #[test]
    #[allow(non_snake_case)]
    fn list_paths__format_Pretty__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "list-paths",
            "--format",
            "Pretty",
        ]) {
            Ok(_) => panic!("expected clap to reject --format Pretty"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T266 AC2: `--format pretty` parses on scan-roots (not InvalidValue).
    #[test]
    #[allow(non_snake_case)]
    fn scan_roots__format_pretty__parses() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "scan-roots",
            "--format",
            "pretty",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected scan-roots --format pretty to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Project {
                command: super::ProjectCommands::ScanRoots { format, .. },
            } => {
                assert_eq!(format, "pretty");
            }
            _ => panic!("expected ProjectCommands::ScanRoots"),
        }
    }

    /// T266 AC2: unknown `--format` is clap InvalidValue on scan-roots.
    #[test]
    #[allow(non_snake_case)]
    fn scan_roots__format_xml__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "scan-roots",
            "--format",
            "xml",
        ]) {
            Ok(_) => panic!("expected clap to reject --format xml"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T266 AC2: `--format JSON` is clap InvalidValue on scan-roots.
    #[test]
    #[allow(non_snake_case)]
    fn scan_roots__format_JSON__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "scan-roots",
            "--format",
            "JSON",
        ]) {
            Ok(_) => panic!("expected clap to reject --format JSON"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T266 AC2: `--format Pretty` is clap InvalidValue on scan-roots.
    #[test]
    #[allow(non_snake_case)]
    fn scan_roots__format_Pretty__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "scan-roots",
            "--format",
            "Pretty",
        ]) {
            Ok(_) => panic!("expected clap to reject --format Pretty"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T268 AC1: `--root DIR PATH` is clap ArgumentConflict (exit 2).
    #[test]
    #[allow(non_snake_case)]
    fn scan_roots__root_and_path__clap_argument_conflict() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "scan-roots",
            "--root",
            r"C:\dev",
            r"C:\other",
        ]) {
            Ok(_) => panic!("expected clap to reject --root with positional PATH"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    /// T268 AC1: `PATH --root DIR` is the same XOR.
    #[test]
    #[allow(non_snake_case)]
    fn scan_roots__path_then_root__clap_argument_conflict() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "scan-roots",
            r"C:\other",
            "--root",
            r"C:\dev",
        ]) {
            Ok(_) => panic!("expected clap to reject positional PATH with --root"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    /// T268 F1: `--root DIR` alone parses into the named field.
    #[test]
    #[allow(non_snake_case)]
    fn scan_roots__root_flag__parses() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "scan-roots",
            "--root",
            r"C:\dev",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected scan-roots --root to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Project {
                command: super::ProjectCommands::ScanRoots { root, path, .. },
            } => {
                assert_eq!(root.as_deref(), Some(r"C:\dev"));
                assert!(path.is_none(), "positional stays None when --root is set");
            }
            _ => panic!("expected ProjectCommands::ScanRoots"),
        }
    }

    /// T273 AC7: POSIX `--` makes `--limit` the query string (layer 1).
    #[test]
    #[allow(non_snake_case)]
    fn sync_query__posix_end_of_options__limit_is_query() {
        let cli = match super::Cli::try_parse_from(["ai-brains", "sync", "query", "--", "--limit"])
        {
            Ok(c) => c,
            Err(e) => panic!("expected sync query -- --limit to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Sync {
                command: super::SyncCommands::Query { query, .. },
            } => {
                assert_eq!(query, "--limit");
            }
            _ => panic!("expected SyncCommands::Query"),
        }
    }

    /// T273 AC8 / F22: bare `--limit` is still the vault cap (empty option value).
    ///
    /// clap 4.6.1 reports this as `InvalidValue` (EmptyValue was folded in clap 4).
    /// T247 `MissingRequiredArgument` is a `requires` relationship, not this case.
    #[test]
    #[allow(non_snake_case)]
    fn sync_query__bare_limit_flag__still_requires_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from(["ai-brains", "sync", "query", "--limit"]) {
            Ok(_) => panic!("expected clap to reject --limit without a value"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
        let msg = err.to_string();
        assert!(
            msg.contains("--limit <LIMIT>"),
            "AC8: vault --limit still stands; got: {msg}"
        );
    }

    /// T279 F30: after_help names Safety live hotspots + dry-run empty.
    #[test]
    #[allow(non_snake_case)]
    fn preflight__help__names_session_safety_hotspots() {
        let err = match super::Cli::try_parse_from(["ai-brains", "preflight", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("hotspots") && help.contains("safety sync --dry-run"),
            "F30: after_help names live hotspots and dry-run empty; got: {help}"
        );
    }

    /// T278 F30: after_help names session PREVIEW caption shape.
    #[test]
    #[allow(non_snake_case)]
    fn graph__help__names_session_preview_caption() {
        let err = match super::Cli::try_parse_from(["ai-brains", "graph", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("memories") && help.contains("first line"),
            "F30: after_help names session PREVIEW caption; got: {help}"
        );
    }

    /// T293 AC10: after_help dual-truth — human prefer-fills authority; JSON order unchanged.
    #[test]
    #[allow(non_snake_case)]
    fn graph__help__names_prefer_authority_and_json_order() {
        let err = match super::Cli::try_parse_from(["ai-brains", "graph", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("prefer-fills") || help.contains("authority"),
            "AC10: after_help names human prefer-fills authority; got: {help}"
        );
        assert!(
            help.contains("JSON order unchanged")
                || (help.contains("direction") && help.contains("label")),
            "AC10: after_help names JSON order unchanged; got: {help}"
        );
        assert!(
            help.contains("--format json"),
            "AC10: catalog/examples still include --format json; got: {help}"
        );
        assert!(
            help.contains("memories") && help.contains("first line"),
            "AC10: session PREVIEW sentence retained; got: {help}"
        );
    }

    /// T269 AC6: after_help names Nightly heading, 267009/SCHED, 750 ms, TCP, and `/health`.
    #[test]
    #[allow(non_snake_case)]
    fn nightly__help__names_nightly_heading_and_probe_budget() {
        let err = match super::Cli::try_parse_from(["ai-brains", "nightly", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("AI-Brains-Nightly"),
            "AC6: after_help names AI-Brains-Nightly; got: {help}"
        );
        assert!(
            help.contains("267009") || help.contains("SCHED_S_TASK_RUNNING"),
            "AC6: after_help names 267009 or SCHED_S_TASK_RUNNING; got: {help}"
        );
        assert!(
            help.contains("750"),
            "AC6: after_help names 750 ms budget; got: {help}"
        );
        assert!(
            help.contains("TCP"),
            "AC6: after_help names TCP vs HTTP; got: {help}"
        );
        assert!(
            help.contains("/health"),
            "AC6: after_help names /health; got: {help}"
        );
    }

    /// T297 AC7: Status after_help names TCP connect + model process.
    #[test]
    #[allow(non_snake_case)]
    fn daemon__help__status_names_backend_tcp() {
        let err = match super::Cli::try_parse_from(["ai-brains", "daemon", "status", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("TCP connect"),
            "AC7: after_help names TCP connect; got: {help}"
        );
        assert!(
            help.contains("model process"),
            "AC7: after_help names model process; got: {help}"
        );
    }

    /// T297 AC7: unknown Status flags stay clap exit 2 (no `--format`).
    #[test]
    #[allow(non_snake_case)]
    fn daemon_status__unknown_format_flag__clap_exit_2() {
        let err =
            match super::Cli::try_parse_from(["ai-brains", "daemon", "status", "--format", "json"])
            {
                Ok(_) => panic!("expected unknown --format to fail clap parse"),
                Err(e) => e,
            };
        let kind = err.kind();
        assert!(
            matches!(
                kind,
                clap::error::ErrorKind::UnknownArgument | clap::error::ErrorKind::InvalidValue
            ),
            "AC7: unknown flag must be clap usage class; got {kind:?}"
        );
        // clap DisplayHelp/usage exits map to EXIT_USAGE=2 at main; kind proves refuse.
        let rendered = err.to_string();
        assert!(
            rendered.contains("--format") || rendered.contains("unexpected"),
            "AC7: error should name the unknown flag; got: {rendered}"
        );
    }

    /// T296 AC6: after_help names Router 267014 / SCHED_S_TASK_TERMINATED as success (not Nightly).
    #[test]
    #[allow(non_snake_case)]
    fn nightly__help__names_router_267014_success() {
        let err = match super::Cli::try_parse_from(["ai-brains", "nightly", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("267014") || help.contains("SCHED_S_TASK_TERMINATED"),
            "AC6: after_help names 267014 or SCHED_S_TASK_TERMINATED; got: {help}"
        );
        let lower = help.to_ascii_lowercase();
        assert!(
            lower.contains("success") && lower.contains("not nightly"),
            "AC6: after_help must say success and not Nightly Last Result; got: {help}"
        );
    }

    /// T273 AC12: after_help names POSIX `-- --limit` and contrasts vault `--limit 10`.
    #[test]
    #[allow(non_snake_case)]
    fn sync_query__help__contrasts_needle_and_vault_limit() {
        let err = match super::Cli::try_parse_from(["ai-brains", "sync", "query", "--help"]) {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("sync query -- --limit"),
            "AC12: after_help names POSIX -- --limit; got: {help}"
        );
        assert!(
            help.contains("--limit 10"),
            "AC12: after_help contrasts vault --limit 10; got: {help}"
        );
    }

    /// T268 AC13: after_help names `--root` and both C:\\dev examples.
    #[test]
    #[allow(non_snake_case)]
    fn scan_roots__help__names_root_and_positional() {
        let err = match super::Cli::try_parse_from(["ai-brains", "project", "scan-roots", "--help"])
        {
            Ok(_) => panic!("expected --help to be DisplayHelp"),
            Err(e) => e,
        };
        let help = err.to_string();
        assert!(
            help.contains("--root"),
            "AC13: after_help names --root; got: {help}"
        );
        assert!(
            help.contains(r"C:\dev") || help.contains("C:\\dev"),
            "AC13: after_help keeps C:\\dev examples; got: {help}"
        );
    }

    /// T266 AC7: `--format pretty` parses on whoami.
    #[test]
    #[allow(non_snake_case)]
    fn whoami__format_pretty__parses() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "whoami",
            "--format",
            "pretty",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected whoami --format pretty to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Project {
                command: super::ProjectCommands::Whoami { format },
            } => {
                assert_eq!(format, "pretty");
            }
            _ => panic!("expected ProjectCommands::Whoami"),
        }
    }

    /// T266 AC7: `--format JSON` is clap InvalidValue on whoami.
    #[test]
    #[allow(non_snake_case)]
    fn whoami__format_JSON__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "whoami",
            "--format",
            "JSON",
        ]) {
            Ok(_) => panic!("expected clap to reject --format JSON"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T266 AC7: `--format Pretty` is clap InvalidValue on whoami.
    #[test]
    #[allow(non_snake_case)]
    fn whoami__format_Pretty__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "whoami",
            "--format",
            "Pretty",
        ]) {
            Ok(_) => panic!("expected clap to reject --format Pretty"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T266 AC7: `--format pretty` parses on adopt-path.
    #[test]
    #[allow(non_snake_case)]
    fn adopt_path__format_pretty__parses() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "adopt-path",
            "--format",
            "pretty",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected adopt-path --format pretty to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Project {
                command: super::ProjectCommands::AdoptPath { format, .. },
            } => {
                assert_eq!(format, "pretty");
            }
            _ => panic!("expected ProjectCommands::AdoptPath"),
        }
    }

    /// T266 AC7: `--format JSON` is clap InvalidValue on adopt-path.
    #[test]
    #[allow(non_snake_case)]
    fn adopt_path__format_JSON__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "adopt-path",
            "--format",
            "JSON",
        ]) {
            Ok(_) => panic!("expected clap to reject --format JSON"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T266 AC7: `--format Pretty` is clap InvalidValue on adopt-path.
    #[test]
    #[allow(non_snake_case)]
    fn adopt_path__format_Pretty__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "adopt-path",
            "--format",
            "Pretty",
        ]) {
            Ok(_) => panic!("expected clap to reject --format Pretty"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T266 AC7: `--format pretty` parses on rebind-path (dummy path + `--to`).
    #[test]
    #[allow(non_snake_case)]
    fn rebind_path__format_pretty__parses() {
        let cli = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "rebind-path",
            r"C:\dev\dummy",
            "--to",
            "dest",
            "--format",
            "pretty",
        ]) {
            Ok(c) => c,
            Err(e) => panic!("expected rebind-path --format pretty to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Project {
                command: super::ProjectCommands::RebindPath { format, .. },
            } => {
                assert_eq!(format, "pretty");
            }
            _ => panic!("expected ProjectCommands::RebindPath"),
        }
    }

    /// T266 AC7: `--format JSON` is clap InvalidValue on rebind-path.
    #[test]
    #[allow(non_snake_case)]
    fn rebind_path__format_JSON__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "rebind-path",
            r"C:\dev\dummy",
            "--to",
            "dest",
            "--format",
            "JSON",
        ]) {
            Ok(_) => panic!("expected clap to reject --format JSON"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T266 AC7: `--format Pretty` is clap InvalidValue on rebind-path.
    #[test]
    #[allow(non_snake_case)]
    fn rebind_path__format_Pretty__clap_invalid_value() {
        use clap::error::ErrorKind;
        let err = match super::Cli::try_parse_from([
            "ai-brains",
            "project",
            "rebind-path",
            r"C:\dev\dummy",
            "--to",
            "dest",
            "--format",
            "Pretty",
        ]) {
            Ok(_) => panic!("expected clap to reject --format Pretty"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    /// T251 F1: first-class `device status` (not a List alias).
    #[test]
    #[allow(non_snake_case)]
    fn device_status__parses() {
        use clap::Parser;
        let cli = match super::Cli::try_parse_from(["ai-brains", "device", "status"]) {
            Ok(c) => c,
            Err(e) => panic!("expected device status to parse: {e}"),
        };
        match *cli.command {
            super::Commands::Device {
                command: super::DeviceCommands::Status,
            } => {}
            _ => panic!("expected DeviceCommands::Status"),
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn claude_hook_schema__include_str__draft_2020_12() {
        let parsed: serde_json::Value = serde_json::from_str(super::SCHEMA_CLAUDE_HOOK)
            .unwrap_or_else(|e| panic!("valid JSON: {e}"));
        assert_eq!(
            parsed["$schema"].as_str(),
            Some("https://json-schema.org/draft/2020-12/schema")
        );
        assert_eq!(parsed["additionalProperties"], false);
        assert_eq!(
            parsed["title"].as_str(),
            Some("AI-Brains claude-hook payload")
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn codex_hook_schema__include_str__draft_2020_12() {
        let parsed: serde_json::Value = serde_json::from_str(super::SCHEMA_CODEX_HOOK)
            .unwrap_or_else(|e| panic!("valid JSON: {e}"));
        assert_eq!(
            parsed["$schema"].as_str(),
            Some("https://json-schema.org/draft/2020-12/schema")
        );
        assert_eq!(parsed["additionalProperties"], false);
        assert_eq!(
            parsed["title"].as_str(),
            Some("AI-Brains codex-hook payload")
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn claude_and_codex_hook__schema_flag__parses() {
        for args in [
            vec!["ai-brains", "claude-hook", "--schema"],
            vec!["ai-brains", "codex-hook", "--schema"],
            vec!["ai-brains", "claude-import", "--days", "7", "--dry-run"],
            vec!["ai-brains", "codex-import", "--force"],
        ] {
            super::Cli::try_parse_from(&args)
                .unwrap_or_else(|e| panic!("expected {args:?} to parse: {e}"));
        }
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
    #[arg(
        long,
        env = "AI_BRAINS_KEY",
        hide_env_values = true,
        help_heading = "Global options"
    )]
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
    #[command(
        display_order = 50,
        after_help = "Pipe a JSON turn on stdin. Empty or TTY stdin exits 2 with a usage example.\nExample payload:\n{\n  \"session_id\": \"00000000-0000-0000-0000-000000000001\",\n  \"project_id\": \"00000000-0000-0000-0000-000000000000\",\n  \"harness_id\": \"00000000-0000-0000-0000-000000000002\",\n  \"turn_id\": \"00000000-0000-0000-0000-000000000003\",\n  \"role\": \"user\",\n  \"content\": \"hello\",\n  \"privacy\": \"CloudOk\"\n}\n  echo '{\"session_id\":\"00000000-0000-0000-0000-000000000001\",\"project_id\":\"00000000-0000-0000-0000-000000000000\",\"harness_id\":\"00000000-0000-0000-0000-000000000002\",\"turn_id\":\"00000000-0000-0000-0000-000000000003\",\"role\":\"user\",\"content\":\"hello\",\"privacy\":\"CloudOk\"}' | ai-brains ingest --dry-run"
    )]
    Ingest {
        /// Preview what would be ingested without writing to the vault
        #[arg(long)]
        dry_run: bool,
    },
    /// Vault-first search (pretty on TTY, JSON when piped). Alias: `search`.
    /// For vault + Ledgerful ledger: `sync query`. Governed conclusions/decisions: `query progressive`.
    #[command(visible_alias = "search", display_order = 10)]
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
        /// Output format: json | pretty | text (default: pretty on TTY, json otherwise)
        #[arg(long)]
        format: Option<String>,
        /// Use semantic (embedding) search alongside FTS5
        #[arg(long)]
        semantic: bool,
        /// One-shot cosine floor for `--semantic`. When set, **replaces** both
        /// the hybrid-arm default (0.55 / `AI_BRAINS_SEMANTIC_MIN_SCORE`) and the
        /// semantic-only default (0.60 / `AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE`) with
        /// this value — not `max()`. Omit to keep dual-floor defaults (T218 F2b/F39).
        #[arg(long = "min-score")]
        min_score: Option<f64>,
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
        /// Include T70 code-symbol stubs in the mix (default: exclude).
        #[arg(long)]
        symbols: bool,
    },
    /// Generate preflight context for an LLM
    #[command(
        display_order = 11,
        after_help = "Default --pretty caps Session and Recent display lines at 140 characters. Safety is not line-capped on default pretty; only --compact first-line-caps Safety (100).\nSafety is live Ledgerful hotspots (project-scoped) or leading CONSTRAINT/INVARIANT/HOTSPOT pins, not session dumps; empty names `ai-brains safety sync --dry-run`.\nJSON and --summary ignore --compact.\nFull `--format json` is compact `{text, word_count, sections}` (T265). `--summary --format json` stays the T220 pretty envelope.\nExamples:\n  ai-brains preflight --pretty\n  ai-brains preflight --pretty --compact\n  ai-brains preflight --format json\n  ai-brains preflight --summary"
    )]
    Preflight {
        #[arg(short, long, default_value_t = 1500)]
        max_words: usize,
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        /// Output human-readable text instead of JSON
        #[arg(long)]
        pretty: bool,
        /// Output format: human | json | pretty.
        /// With `--summary`, `--format json` (case-insensitive) emits a pretty machine
        /// envelope; other values stay on the human summary path. Full (non-summary)
        /// preflight is compact `{text, word_count, sections}` (T180 required keys + T265
        /// additive `sections`). `--summary --format json` stays T220.
        #[arg(long)]
        format: Option<String>,
        /// Tighter pretty item/line caps (human/pretty only). JSON and `--summary` ignore this.
        #[arg(long)]
        compact: bool,
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
        /// Never prompt to install harness capture hooks (T235)
        #[arg(long)]
        no_hook_prompt: bool,
        /// Install ready harness hooks without interactive prompt (T235)
        #[arg(long)]
        install_hooks: bool,
    },
    /// Run nightly intelligence sweep
    #[command(
        display_order = 26,
        after_help = "Default --format is human; pipes stay human (do not silently switch to JSON).\nScripts: pass --format json.\nNightly Last Result is AI-Brains-Nightly. Router 267009 is SCHED_S_TASK_RUNNING (success; ONLOGON keep-alive).\nRouter 267014 is SCHED_S_TASK_TERMINATED (success; last run ended), not Nightly Last Result.\nprobe=timeout is HTTP /health within 750 ms. daemon status Open is TCP connect.\nExamples:\n  ai-brains nightly --status\n  ai-brains nightly --status --format json\n  ai-brains nightly --status --quick --format json"
    )]
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
        /// Skip HTTP probes (requires --status)
        #[arg(long, requires = "status", conflicts_with_all = ["schedule", "unschedule"])]
        quick: bool,
        /// Output format for --status (default human; pipes stay human)
        #[arg(
            long,
            default_value = "human",
            requires = "status",
            conflicts_with_all = ["schedule", "unschedule"],
            value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]
        )]
        format: String,
        /// Skip all harness session importers (AGY, Grok, OpenCode). Use on
        /// isolated, CI, SYSTEM-scheduled, or per-project vaults to prevent
        /// cross-vault contamination from real harness history.
        #[arg(long)]
        skip_import: bool,
        /// Skip only Antigravity (agy) batch import during nightly
        #[arg(long)]
        skip_import_agy: bool,
        /// Skip only Grok Build batch import during nightly
        #[arg(long)]
        skip_import_grok: bool,
        /// Skip only OpenCode batch import during nightly
        #[arg(long)]
        skip_import_opencode: bool,
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
        after_help = "Read-only: no migrate, no vault/backups create, no secrets on stdout. Does not replace RECOVERY-DRILLS. Offline kit residual without --kit-path is operator responsibility. Daemon probe = our IPC only. --backup-max-age uses Nd/Nh/Nw. No --passphrase argv.\nKey bootstrap: set --key or AI_BRAINS_KEY as x'<64 hex>' (see Docs/INSTALL.md). Missing key → vault_open skipped; wrong key → vault_open fail.\nExamples:\n  ai-brains doctor\n  ai-brains doctor --summary\n  ai-brains doctor --json\n  ai-brains doctor --kit-path ./kit.json --passphrase-file ./pw.txt\n  ai-brains doctor --fail-on-degraded --backup-max-age 14d --full"
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
        /// Compact human summary (warn+fail only). JSON still emits the full report.
        #[arg(long)]
        summary: bool,
    },
    /// Unified vault glance (daemon + doctor attention + graph density + nightly last-run)
    #[command(
        display_order = 12,
        after_help = "In-process compose of four probes. Does not replace `doctor` / `nightly --status` / `daemon status` / `graph update`.\nNever starts the daemon; never rebuilds the graph; no HTTP probes; no daemon TCP retries.\nFail-open per section; exit 0 for degraded / Stopped / sparse / never.\nExamples:\n  ai-brains status\n  ai-brains status --format json"
    )]
    Status {
        /// Output format: auto (TTY human / pipe json) or explicit human|json aliases
        #[arg(
            long,
            default_value = "auto",
            value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]
        )]
        format: String,
    },
    /// [dangerous] Forget a specific memory (soft delete)
    #[command(
        display_order = 40,
        after_help = "Read inventory (not dangerous):\n  ai-brains memory list\n  ai-brains memory list --status forgotten\n  ai-brains forget --list-forgotten --limit 5\nList-forgotten shares the memory list backend (limit default 50, max 200; Scope + --global/--format/--tag).\nEmpty list-forgotten prints Pinned: N (same as memory list --summary) then next: ai-brains memory list.\nSoft-forget is not CE wipe / not NIST Purge — use restore to reverse."
    )]
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
        /// List forgotten memories (read-only; same backend as `memory list --status forgotten`)
        #[arg(long)]
        list_forgotten: bool,
        /// Restore a forgotten memory
        #[arg(long)]
        restore: Option<String>,
        /// Preview what would be forgotten without modifying the vault
        #[arg(long)]
        dry_run: bool,
        /// Aggregate list across ALL projects (list-forgotten only)
        #[arg(long)]
        global: bool,
        /// Max rows for --list-forgotten (default 50, max 200)
        #[arg(short = 'l', long)]
        limit: Option<usize>,
        /// Output format for --list-forgotten: human (default) or json
        #[arg(long, default_value = "human", value_parser = ["human", "json"])]
        format: String,
        /// Filter list-forgotten by content TAGS: token (heuristic)
        #[arg(long)]
        tag: Option<String>,
        /// Project scope for list-forgotten (env AI_BRAINS_PROJECT_ID)
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
    },
    /// List pinned/forgotten memories (inventory skim; read-only)
    #[command(display_order = 18)]
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },
    /// Stop an active session
    #[command(display_order = 16)]
    StopSession {
        /// Session ID to stop
        session_id: String,
    },
    /// Initialize or refresh the project context (first-init writes local .env; already-initialized ensures vault)
    #[command(
        display_order = 15,
        after_help = "First-init (no .env) writes local .env with PROJECT_ID / SESSION_ID / HARNESS_ID.\nAlready-initialized (session present, no --new-project/--new-session) ensures those .env IDs exist in the open vault and does not rewrite .env.\n--show never writes .env or ensures vault."
    )]
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
        /// Skip the 5-minute quiescence window (import even if file was modified recently)
        #[arg(long, default_value_t = false)]
        force: bool,
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
    /// Process a Grok Build hook payload (Stop / SessionEnd → chat_history)
    #[command(display_order = 52)]
    GrokHook {
        /// The JSON payload from the Grok capture wrapper
        #[arg(long)]
        payload: Option<String>,
        /// Print the JSON Schema for the expected `--payload` shape and exit.
        /// The schema is also at `Docs/schemas/grok-hook-payload.json`.
        #[arg(long)]
        schema: bool,
    },
    /// Import Grok Build chat_history sessions into the vault
    #[command(display_order = 51)]
    GrokImport {
        /// Only import sessions modified within the last N days
        #[arg(short, long, default_value_t = 30)]
        days: usize,
        /// Skip the 5-minute quiescence window (import even if file was modified recently)
        #[arg(long, default_value_t = false)]
        force: bool,
        /// Discover and report what would be imported without writing to the vault
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Process an OpenCode plugin hook payload (session.idle → message-only)
    #[command(display_order = 54)]
    OpencodeHook {
        /// The JSON payload from the OpenCode capture plugin
        #[arg(long)]
        payload: Option<String>,
        /// Print the JSON Schema for the expected `--payload` shape and exit.
        /// The schema is also at `Docs/schemas/opencode-hook-payload.json`.
        #[arg(long)]
        schema: bool,
    },
    /// Import OpenCode sessions via `opencode session list` + `export` (never opencode.db)
    #[command(display_order = 53)]
    OpencodeImport {
        /// Only import sessions updated within the last N days
        #[arg(short, long, default_value_t = 7)]
        days: usize,
        /// Ignore watermark and reprocess sessions in the days window
        #[arg(long, default_value_t = false)]
        force: bool,
        /// Discover and report what would be imported without writing to the vault
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Max sessions to list/process (OpenCode list default cap is 100)
        #[arg(long, default_value_t = 100)]
        max_sessions: usize,
    },
    /// Process a Claude Code hook payload (UserPromptSubmit / Stop / SessionEnd)
    #[command(display_order = 56)]
    ClaudeHook {
        /// The JSON payload from the Claude capture wrapper
        #[arg(long)]
        payload: Option<String>,
        /// Print the JSON Schema for the expected `--payload` shape and exit.
        /// The schema is also at `Docs/schemas/claude-hook-payload.json`.
        #[arg(long)]
        schema: bool,
    },
    /// Import Claude Code project JSONL sessions into the vault
    #[command(display_order = 55)]
    ClaudeImport {
        /// Only import sessions modified within the last N days
        #[arg(short, long, default_value_t = 30)]
        days: usize,
        /// Skip the 5-minute quiescence window (import even if file was modified recently)
        #[arg(long, default_value_t = false)]
        force: bool,
        /// Discover and report what would be imported without writing to the vault
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Process a Codex CLI hook payload (UserPromptSubmit / Stop)
    #[command(display_order = 58)]
    CodexHook {
        /// The JSON payload from the Codex capture wrapper
        #[arg(long)]
        payload: Option<String>,
        /// Print the JSON Schema for the expected `--payload` shape and exit.
        /// The schema is also at `Docs/schemas/codex-hook-payload.json`.
        #[arg(long)]
        schema: bool,
    },
    /// Import Codex rollout JSONL sessions into the vault (fail-open)
    #[command(display_order = 57)]
    CodexImport {
        /// Only import sessions modified within the last N days
        #[arg(short, long, default_value_t = 30)]
        days: usize,
        /// Skip the 5-minute quiescence window (import even if file was modified recently)
        #[arg(long, default_value_t = false)]
        force: bool,
        /// Discover and report what would be imported without writing to the vault
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Detect and install harness capture hooks (user-global, message-only)
    #[command(
        display_order = 52,
        after_help = "Examples:\n  ai-brains harness status\n  ai-brains harness status --format json\n  ai-brains harness install --harness agy --dry-run\n  ai-brains harness install --harness all-ready --dry-run\n  ai-brains harness install --harness agy --yes\n  ai-brains harness uninstall --harness agy --yes\n  ai-brains harness reset-decline --harness all"
    )]
    Harness {
        #[command(subcommand)]
        command: HarnessCommands,
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
        after_help = "Examples:\n  ai-brains briefing project --format human --max-words 1500 --project-id <uuid>\n  ai-brains briefing project --format json --project-id <uuid>\n  ai-brains briefing personal --format human\n  ai-brains briefing personal --format json\n  # or set AI_BRAINS_PROJECT_ID for project briefing"
    )]
    Briefing {
        #[command(subcommand)]
        command: BriefingCommands,
    },
    /// Governed progressive query, handle expand, and query-trace retrieval (T152)
    #[command(
        display_order = 32,
        after_help = "Progressive searches Approved decisions + Confirmed/Active conclusions, not vault FTS. Vault-first: `recall` / `search`. Vault + ledger: `sync query`.\n`query trace` missing/unauthorized prints a JSON envelope (`found: false` + `next_step` copy-paste `query progressive … --dry-run false`) and exits 0; `--format human` prints two lines.\nExamples:\n  ai-brains query progressive \"why was graph backend replaced?\" --project-id <uuid>\n  ai-brains query progressive \"what did we decide\" --dry-run\n  ai-brains query progressive \"what did we decide\" --dry-run false\n  ai-brains query expand <handle-id> --project-id <uuid> --format json\n  ai-brains query expand <handle-id> --project-id <uuid> --format human\n  ai-brains query trace <trace-id>\n  # or set AI_BRAINS_PROJECT_ID"
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
        after_help = "Examples:\n  ai-brains scope resolve\n  ai-brains scope resolve --format json"
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
    /// Policy grant inspection + discovery bootstrap (T160/T210)
    #[command(
        display_order = 36,
        after_help = "Examples:\n  ai-brains policy bootstrap --scope Repository:<uuid>\n  ai-brains policy bootstrap   # omit --scope when project context is authoritative\n  ai-brains policy show --scope Repository:<uuid>\n  ai-brains policy show   # omit --scope when project context is authoritative\n  ai-brains policy check --capability ProposeConclusion --scope Repository:<uuid>\n  ai-brains policy check --capability ReadEvidence   # omit --scope when authoritative; default auto = TTY human / pipe JSON\n  ai-brains policy check --capability ReadEvidence --format human\n  ai-brains policy check --capability ReadEvidence --format json"
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
        after_help = "Examples:\n  ai-brains retention plan\n  ai-brains retention plan --format json\n  ai-brains retention apply --confirm --format json\n  ai-brains retention apply --confirm --scope Repository:<uuid> --format json\nHonesty: projection delete ≠ CE; CE reuses erasure wipe path for envelope classes only; CE apply requires --scope."
    )]
    Retention {
        #[command(subcommand)]
        command: RetentionCommands,
    },
    /// Multi-device enrollment (T176 / ADR-0018). Optional; not PQ; not remote wipe; not metadata-private.
    /// Does **not** repurpose `sync` (Ledgerful) or `safety sync` (hotspot pin).
    #[command(
        display_order = 24,
        after_help = "Examples:\n  ai-brains device bootstrap\n  ai-brains device list\n  ai-brains device status\n  ai-brains device fingerprint\n  ai-brains device package-export --out peer.bin\n  ai-brains device enroll --package peer.bin --yes\n  ai-brains device revoke <device-id>\nHonesty: multi-device is optional; classical ECC only (not PQ); ACK ≠ wipe proof; padding ≠ metadata privacy."
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
    /// Enrolled roster + this-machine + local-only honesty + pointer to `replicate status`
    Status,
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
    after_help = "Examples:\n  ai-brains briefing project --format human --max-words 1500 --project-id <uuid>\n  ai-brains briefing project --format json --project-id <uuid>\n  ai-brains briefing personal --format human\n  ai-brains briefing personal --format json\n  # or set AI_BRAINS_PROJECT_ID for project briefing"
)]
enum BriefingCommands {
    /// Build a Project Briefing packet (policy → lifecycle → authority)
    #[command(
        after_help = "Human granted-empty prefer-fills a Vault pins (not Approved) stanza; CLI JSON adds vault_pin_count / vault_pin_previews; authority arrays stay empty.\nExamples:\n  ai-brains briefing project --format human --max-words 1500 --project-id <uuid>\n  ai-brains briefing project --format json --project-id <uuid>\n  ai-brains briefing project --dry-run --project-id <uuid>\n  ai-brains briefing project --dry-run false --project-id <uuid>\n  # or set AI_BRAINS_PROJECT_ID"
    )]
    Project {
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        #[arg(short, long, default_value_t = 1500)]
        max_words: usize,
        /// Skip BriefingGenerated event / cache write (default: true)
        /// T314 F1 — optional-value dry-run (default true). Bare `--dry-run` and `--dry-run false` both parse.
        #[arg(
            long,
            default_value_t = true,
            num_args = 0..=1,
            default_missing_value = "true",
            action = clap::ArgAction::Set
        )]
        dry_run: bool,
        /// Output format: human, pretty, text, markdown, md, or json (default: markdown on TTY, json otherwise)
        #[arg(long)]
        format: Option<String>,
    },
    /// Build a Personal Continuity Briefing packet
    #[command(
        after_help = "Denied human uses an optional-continuity body (not `_None_` empty preferences). JSON `denied: true` keeps empty arrays.\nExamples:\n  ai-brains briefing personal --format human\n  ai-brains briefing personal --format json\n  ai-brains briefing personal --dry-run\n  ai-brains briefing personal --dry-run false"
    )]
    Personal {
        /// Personal user id (defaults to principal UUID mapping)
        #[arg(long)]
        user_id: Option<String>,
        #[arg(short, long, default_value_t = 800)]
        max_words: usize,
        /// T314 F1 — optional-value dry-run (default true). Bare `--dry-run` and `--dry-run false` both parse.
        #[arg(
            long,
            default_value_t = true,
            num_args = 0..=1,
            default_missing_value = "true",
            action = clap::ArgAction::Set
        )]
        dry_run: bool,
        /// Output format: human, pretty, text, markdown, md, or json (default: markdown on TTY, json otherwise)
        #[arg(long)]
        format: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Progressive searches Approved decisions + Confirmed/Active conclusions, not vault FTS. Vault-first: `recall` / `search`. Vault + ledger: `sync query`.\n`query trace` missing/unauthorized prints a JSON envelope (`found: false` + `next_step` copy-paste `query progressive … --dry-run false`) and exits 0; `--format human` prints two lines.\nExamples:\n  ai-brains query progressive \"why was graph backend replaced?\" --project-id <uuid>\n  ai-brains query progressive \"what did we decide\" --dry-run\n  ai-brains query progressive \"what did we decide\" --dry-run false\n  ai-brains query expand <handle-id> --project-id <uuid> --format json\n  ai-brains query expand <handle-id> --project-id <uuid> --format human\n  ai-brains query trace <trace-id>\n  # or set AI_BRAINS_PROJECT_ID"
)]
enum GovernedQueryCommands {
    /// Run a governed progressive query (JSON ProgressiveQueryResponse)
    #[command(
        after_help = "Granted-empty next_step is copy-paste `recall` of the operator query plus `(Pinned: N)` when COUNT succeeds (not the U+2026 ellipsis).\nProgressive searches Approved decisions + Confirmed/Active conclusions, not vault FTS. Vault-first: `recall` / `search`. Vault + ledger: `sync query`.\nExamples:\n  ai-brains query progressive \"why was graph backend replaced?\" --project-id <uuid>\n  ai-brains query progressive \"what did we decide\" --dry-run\n  ai-brains query progressive \"what did we decide\" --dry-run false\n  # or set AI_BRAINS_PROJECT_ID"
    )]
    Progressive {
        /// Query text
        query: String,
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        #[arg(short, long, default_value_t = 16)]
        limit: usize,
        /// Skip QueryTraceRecorded event (default: true)
        /// T314 F1 — optional-value dry-run (default true). Bare `--dry-run` and `--dry-run false` both parse.
        #[arg(
            long,
            default_value_t = true,
            num_args = 0..=1,
            default_missing_value = "true",
            action = clap::ArgAction::Set
        )]
        dry_run: bool,
    },
    /// Expand an evidence / conclusion / decision handle to a bounded preview
    #[command(
        after_help = "Default --format json emits HandlePreviewDto + applied_scope. `--format human` prints kind then preview (two lines for Unknown/Denied).\nUnknown that is a vault memory_id (not a governed handle) names the namespace and adds a third human line / optional JSON next_step pointing at `ai-brains recall \"what did we decide\"`.\nExamples:\n  ai-brains query expand <handle-id> --project-id <uuid> --format json\n  ai-brains query expand <handle-id> --project-id <uuid> --format human\n  # or set AI_BRAINS_PROJECT_ID"
    )]
    Expand {
        /// Handle id (evidence UUID, conclusion id, or decision id)
        handle_id: String,
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        #[arg(long, default_value_t = 512)]
        max_chars: usize,
        /// Output format: json (default DTO), pretty/human/text/markdown/md (kind+preview), auto (TTY human / pipe json)
        #[arg(
            long,
            default_value = "json",
            value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]
        )]
        format: String,
    },
    /// Fetch a governed query trace by id (envelope when missing or unauthorized)
    #[command(
        after_help = "Missing or unauthorized traces print a JSON envelope (`found: false` + `next_step` copy-paste `query progressive … --dry-run false`) and exit 0, or two human lines with `--format human`. Found traces stay QueryTraceDto JSON. Vault-first: `recall` / `search`.\nExamples:\n  ai-brains query trace <trace-id>\n  ai-brains query trace <trace-id> --format human"
    )]
    Trace {
        trace_id: String,
        /// Output format: json (default envelope/DTO), pretty/human/text/markdown/md (missing two-line), auto (TTY human / pipe json)
        #[arg(
            long,
            default_value = "json",
            value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]
        )]
        format: String,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains scope resolve\n  ai-brains scope resolve --format json"
)]
enum ScopeCommands {
    /// Resolve the active governed scope for the working context
    #[command(
        after_help = "Examples:\n  ai-brains scope resolve\n  ai-brains scope resolve --format json"
    )]
    Resolve {
        /// Output format: auto (TTY human / pipe json), pretty/human/text/markdown/md, or json
        #[arg(
            long,
            default_value = "auto",
            value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]
        )]
        format: String,
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
        after_help = "Granted-empty JSON next_step is copy-paste `recall \"what did we decide\"` plus `(Pinned: N)` when local COUNT succeeds; human prints that line after `(none)`.\nExamples:\n  ai-brains evidence list --scope Repository:<uuid>\n  ai-brains evidence list --format json\n  ai-brains evidence list --query keyword --scope Repository:<uuid>"
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
        after_help = "A vault memory_id pasted here is named (not shown as evidence) and points at `ai-brains recall \"what did we decide\"`.\nExamples:\n  ai-brains evidence show <id> --scope Repository:<uuid> --format json\n  ai-brains evidence show <id> --format json"
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
        after_help = "Granted-empty JSON next_step is copy-paste `recall \"what did we decide\"` plus `(Pinned: N)` when local COUNT succeeds; human prints that line after `(none)`.\nExamples:\n  ai-brains source list --scope Repository:<uuid>\n  ai-brains source list --format json"
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
        after_help = "A vault memory_id pasted here stays NOT_FOUND (exit 4) and names the other namespace via details.hint / stderr hint.\nExamples:\n  ai-brains source show <id> --scope Repository:<uuid>\n  ai-brains source show <id> --format json"
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
    after_help = "Examples:\n  ai-brains decision propose --statement \"...\" --scope Repository:<uuid>\n  ai-brains decision in-force workspace_id"
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
    /// Resolve the in-force Approved decision for a term (local projection)
    #[command(
        after_help = "Examples:\n  ai-brains decision in-force workspace_id\n  ai-brains decision in-force workspace_id --format json"
    )]
    InForce {
        /// Term to resolve (e.g. workspace_id)
        #[arg(value_name = "TERM")]
        term: String,
        /// Scope identity key; soft-filled from authoritative context when omitted
        #[arg(long)]
        scope: Option<String>,
        #[arg(
            long,
            default_value = "json",
            value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]
        )]
        format: String,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
#[command(
    after_help = "Examples:\n  ai-brains review list --scope Repository:<uuid>\n  ai-brains review list --format json\n  ai-brains review resolve <id> --resolution approved --scope Repository:<uuid>"
)]
enum ReviewCommands {
    /// List open review items (E1: items: [] when empty)
    #[command(
        after_help = "Granted-empty JSON next_step is copy-paste `recall \"what did we decide\"` plus `(Pinned: N)` when local COUNT succeeds; human prints that line after `(none)`.\nExamples:\n  ai-brains review list --scope Repository:<uuid>\n  ai-brains review list --format json"
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
    after_help = "Examples:\n  ai-brains policy bootstrap --scope Repository:<uuid>\n  ai-brains policy bootstrap   # omit --scope when project context is authoritative\n  ai-brains policy show --scope Repository:<uuid>\n  ai-brains policy show   # omit --scope when project context is authoritative\n  ai-brains policy check --capability ProposeConclusion --scope Repository:<uuid>\n  ai-brains policy check --capability ReadEvidence   # omit --scope when authoritative; default auto = TTY human / pipe JSON\n  ai-brains policy check --capability ReadEvidence --format human\n  ai-brains policy check --capability ReadEvidence --format json"
)]
enum PolicyCommands {
    /// List applied grants for principal + scope (read-only)
    #[command(
        after_help = "Examples:\n  ai-brains policy show --scope Repository:<uuid>\n  ai-brains policy show   # omit --scope when project context is authoritative"
    )]
    Show {
        /// Scope identity key (optional — soft-resolves when authoritative)
        #[arg(long)]
        scope: Option<String>,
        #[arg(long, default_value = "json")]
        format: Option<String>,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
    },
    /// Dry-run capability allow check
    // after_help catalog must stay in sync with governed_common::CAPABILITY_CATALOG (T241 F6b).
    #[command(
        after_help = "Examples:\n  ai-brains policy check --capability ProposeConclusion --scope Repository:<uuid>\n  ai-brains policy check --capability ReadEvidence   # omit --scope when authoritative; default auto = TTY human / pipe JSON\n  ai-brains policy check --capability ReadEvidence --format human\n  ai-brains policy check --capability ReadEvidence --format json\n\nValid capabilities (discovery first):\n  ReadEvidence (discovery)\n  ReadConclusions (discovery)\n  ReadDecisions (discovery)\n  ApproveConclusion\n  ApproveDecision\n  Erase\n  Export\n  ProposeConclusion\n  ProposeDecision"
    )]
    Check {
        /// Capability name (e.g. ProposeConclusion, ReadEvidence). Required at runtime (catalog on omit).
        #[arg(long)]
        capability: Option<String>,
        /// Scope identity key (optional — soft-resolves when authoritative)
        #[arg(long)]
        scope: Option<String>,
        /// Output format: auto (TTY human / pipe json), pretty/human/text/markdown/md, or json
        #[arg(
            long,
            default_value = "auto",
            value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]
        )]
        format: String,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
    },
    /// Register principal (if needed) and issue discovery-class grants on a scope (T210)
    #[command(
        after_help = "Examples:\n  ai-brains policy bootstrap --scope Repository:<uuid>\n  ai-brains policy bootstrap --scope Repository:<uuid> --dry-run\n  ai-brains policy bootstrap   # omit --scope when project context is authoritative\n  ai-brains policy show --scope Repository:<uuid>   # inspect after bootstrap\nIssues exactly ReadEvidence, ReadConclusions, ReadDecisions (Privacy::LocalOnly). Idempotent."
    )]
    Bootstrap {
        /// Scope identity key (optional — soft-resolves when authoritative)
        #[arg(long)]
        scope: Option<String>,
        /// Report plan without appending register/issue events
        #[arg(long, short = 'n')]
        dry_run: bool,
        #[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]
        principal_id: Option<String>,
        #[arg(long, default_value = "json")]
        format: Option<String>,
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
    after_help = "Examples:\n  ai-brains retention plan\n  ai-brains retention plan --format json\n  ai-brains retention apply --confirm\n  ai-brains retention apply --confirm --scope Repository:<uuid>\nNightly: AI_BRAINS_RETENTION_APPLY_CE only logs intent; CE is CLI+daemon+confirm+scope only."
)]
enum RetentionCommands {
    /// Dry-run class matrix report (no disposal)
    #[command(
        after_help = "Examples:\n  ai-brains retention plan\n  ai-brains retention plan --format json\nmemory_legacy is inventory (none_auto); pins held; plan does not forget.\nWork lists dispose identities even when the class's dominant mechanism is held."
    )]
    Plan {
        /// Output format: auto (TTY human / pipe json), pretty/human/text/markdown/md, or json
        #[arg(
            long,
            default_value = "auto",
            value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]
        )]
        format: String,
    },
    /// [dangerous] Apply retention plan (requires --confirm; CE via daemon T165 wipe)
    #[command(
        after_help = "Honesty:\n  - Default refuse without --confirm\n  - Legacy projection delete is not CE (local)\n  - Envelope CE requires daemon + wipe_content_envelope only (T165)\n  - CE candidates require explicit --scope (Repository:<uuid> / Personal:<uuid>); no random default\n  - Projection-only apply may run without daemon or --scope\n  - Not NIST Purge; pre-erase backups residual\nExamples:\n  ai-brains retention apply --confirm --format json\n  ai-brains retention apply --confirm --format human\n  ai-brains retention apply --confirm --scope Repository:<uuid> --format json"
    )]
    Apply {
        /// Output format: json (default pretty-JSON), auto (same as json), or human/pretty/text/md
        #[arg(
            long,
            default_value = "json",
            value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]
        )]
        format: String,
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
enum HarnessCommands {
    /// Show which harnesses are installed on this machine and wiring status
    Status {
        /// Output format: human (default) or json
        #[arg(long, default_value = "human")]
        format: String,
    },
    /// Install message-only capture hooks (user-global). All five ready: grok, agy, opencode, claude, codex.
    Install {
        /// Harness id: grok | agy | opencode | claude | codex | all | all-ready
        #[arg(long)]
        harness: Option<String>,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
        /// Print planned paths/snippets; zero writes
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove only AI-Brains managed hook markers / wrapper scripts
    Uninstall {
        /// Harness id: grok | agy | opencode | claude | codex | all | all-ready
        #[arg(long)]
        harness: Option<String>,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
        /// Print planned removals; zero writes
        #[arg(long)]
        dry_run: bool,
    },
    /// Clear decline stamps so preflight may offer install again
    ResetDecline {
        /// Harness id or `all` (default: all)
        #[arg(long)]
        harness: Option<String>,
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
#[command(
    after_help = "Examples:\n  ai-brains graph neighbors <memory-id>\n  ai-brains graph neighbors <memory-id> --format json\nTTY/auto prints a table; --format json is compact (keys unchanged).\nSession PREVIEW is {n} memories · first line.\nHuman prefer-fills authority 1-hop (DECISION/CONSTRAINT/INVARIANT/HOTSPOT); JSON order unchanged (direction→label→id).\nHuman table caps RECALLS at 3 and prints +N more RECALLS; JSON lists all 1-hop. Hierarchy leaf pretty may add next: ai-brains nightly --status."
)]
pub enum GraphCommands {
    /// Rebuild graph from all events
    #[command(
        after_help = "Daemon must be Stopped before a mutating rebuild (`ai-brains daemon stop` or `sc stop AI-Brains-Daemon`).\nPrefer `graph rebuild --dry-run` first — prints current density without DELETE.\nTyped-lineage floor 0.50 may still report sparse after a full replay (honest; not a floor lie).\nStdout is the density report (same labeled lines / JSON keys as `graph update`).\nRemediator string stays exact `ai-brains graph rebuild` (no --confirm).\nLarge vaults may take minutes; progress is tracing on stderr, not a stdout spinner.\nExamples:\n  ai-brains graph rebuild --dry-run\n  ai-brains graph rebuild\n  ai-brains graph rebuild --format json"
    )]
    Rebuild {
        /// Preview density + event COUNT; do not DELETE or replay
        #[arg(long)]
        dry_run: bool,
        /// Output format: human (default) or json (same keys as `graph update`)
        #[arg(long, default_value = "human", value_parser = ["human", "json"])]
        format: String,
    },
    /// Show 1-hop graph neighbors of a memory
    Neighbors {
        memory_id: String,
        /// Output format: auto (TTY pretty / pipe json), pretty/human/text, or json
        #[arg(long, default_value = "auto", value_parser = ["auto", "pretty", "human", "text", "json"])]
        format: String,
        /// Max rows (pretty default 50, max 200; JSON unlimited unless set)
        #[arg(short = 'l', long)]
        limit: Option<usize>,
    },
    /// Show recursive SYNTHESIZED_FROM hierarchy of a memory
    Hierarchy {
        memory_id: String,
        /// Output format: auto (TTY pretty / pipe json), pretty/human/text, or json
        #[arg(long, default_value = "auto", value_parser = ["auto", "pretty", "human", "text", "json"])]
        format: String,
        /// Max rows (pretty default 50, max 200; JSON unlimited unless set)
        #[arg(short = 'l', long)]
        limit: Option<usize>,
    },
    /// Show all memories in a session via graph edges
    Session {
        session_id: String,
        /// Output format: auto (TTY pretty / pipe json), pretty/human/text, or json
        #[arg(long, default_value = "auto", value_parser = ["auto", "pretty", "human", "text", "json"])]
        format: String,
        /// Max rows (pretty default 50, max 200; JSON unlimited unless set)
        #[arg(short = 'l', long)]
        limit: Option<usize>,
    },
    /// Show current graph health: node/edge counts
    Update {
        /// Output format: json (default pretty-JSON), auto (same as json), or human
        #[arg(long, default_value = "json", value_parser = ["json", "auto", "human"])]
        format: String,
    },
}

/// T216 memory inventory subcommands.
#[derive(Subcommand, Clone)]
pub enum MemoryCommands {
    /// List pinned or forgotten memories (inventory skim; read-only)
    #[command(
        after_help = "Examples:\n  ai-brains memory list\n  ai-brains memory list --status forgotten --limit 5\n  ai-brains memory list --summary\n  ai-brains memory list --summary --global\n  ai-brains memory list --format json --limit 3\n  ai-brains memory list --tag architecture\nDefault status=pinned. --summary always shows Pinned + Forgotten (ignores --status/--limit; --tag filters counts).\nHuman pinned prefer-fills leading-line authority; JSON order unchanged (recency).\nEmpty --status forgotten prints Pinned: N + next: ai-brains memory list (same as forget --list-forgotten).\nTags are content-prefix heuristic (TAGS: first line), not a schema column.\nSoft-forget list/restore is not CE wipe / not NIST Purge."
    )]
    List {
        /// Status filter: pinned (default) or forgotten
        #[arg(long, default_value = "pinned")]
        status: String,
        /// Max rows (default 50, max 200)
        #[arg(short = 'l', long)]
        limit: Option<usize>,
        /// List across all projects
        #[arg(long)]
        global: bool,
        /// Output format: human (default) or json
        #[arg(long, default_value = "human", value_parser = ["human", "json"])]
        format: String,
        /// Counts mode: Pinned + Forgotten (and by-project under --global)
        #[arg(long)]
        summary: bool,
        /// Filter by content TAGS: token (case-insensitive exact)
        #[arg(long)]
        tag: Option<String>,
        /// Project scope (env AI_BRAINS_PROJECT_ID); required unless --global
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
    },
}

#[derive(Subcommand, Clone)]
pub enum ProjectCommands {
    /// List all projects in the vault (label-first; set-alias nudge on stderr)
    #[command(
        after_help = "Examples:\n  ai-brains project list\n  ai-brains project list --format json\n  ai-brains project set-alias <uuid> my-project\nColumns (human): label | project_id | memories | last_activity | path\nlast_activity = last memory-projection mutation (pin/forget/ingest), not chat-only.\npath is a registered repo path alias when present; never invented (— / null).\nUnaliased projects: a set-alias example is printed on stderr (not stdout).\nThe example prefers the cwd path-owner; the cwd git slug is used only for that owner.\nhuman table puts the cwd path-owner first; JSON order unchanged"
    )]
    List {
        /// Output format: human (default table) or json
        #[arg(long, default_value = "human", value_parser = ["human", "json"])]
        format: String,
    },
    /// Resolve an alias to a project ID
    Resolve {
        /// Project alias to resolve (positional)
        alias_positional: Option<String>,
        /// Project alias to resolve via --alias flag
        #[arg(long = "alias", conflicts_with = "alias_positional")]
        alias: Option<String>,
    },
    /// Auto-detect project: path alias (toplevel/cwd) → git slug → .env PROJECT_ID
    #[command(
        after_help = "Precedence (F5):\n  1. Path alias of git toplevel (else cwd)\n  2. Git slug exact-first vault match\n  3. AI_BRAINS_PROJECT_ID if present in vault\n  4. Miss exit 1\nPath always wins over a unique git slug hit (stderr notes the slug project).\n--export includes source=path_alias|git_slug|env. set-alias is a label; register-path is the disk root."
    )]
    Detect {
        /// Output as shell export statement
        #[arg(long)]
        export: bool,
    },
    /// Set a human-readable alias for a project
    #[command(
        after_help = "Examples:\n  ai-brains project list\n  ai-brains project set-alias <uuid> my-project\n  ai-brains project list --format json\nTip: `project list` prints a copy-paste set-alias example on stderr when aliases are missing."
    )]
    SetAlias {
        /// Project UUID (from `project list`)
        project_id: String,
        /// Alias name (e.g. "ai-brains", "my-app")
        alias: String,
    },
    /// Register a filesystem path alias for multi-root nightly bridge (T233)
    #[command(
        after_help = "Examples:\n  ai-brains project register-path <uuid> C:\\dev\\AI-Brains\n  ai-brains project register-path my-alias C:\\dev\\ledgerful\n  ai-brains project register-path <uuid> /mnt/c/dev/AI-Brains\nset-alias is a human label; register-path is a disk root for Phase-2 Ledgerful bridge.\nSame normalized path may only belong to one project (conflict exit 1).\nCorrection: ai-brains project unregister-path <path>"
    )]
    RegisterPath {
        /// Project UUID or human alias (from `project list` / `set-alias`)
        project_ref: String,
        /// Filesystem path to register (Windows or WSL form; normalized for compare)
        path: String,
    },
    /// Show all project identity signals (env / path alias / git detect)
    #[command(
        after_help = "Shows effective daily Scope, shell vs .env PROJECT_ID, path-alias owner, and detect result.\nDoes not rewrite PROJECT_ID. Detect order: path_alias → git_slug → env.\nExamples:\n  ai-brains project whoami\n  ai-brains project whoami --format json\n  ai-brains --no-project-context project whoami --format json"
    )]
    Whoami {
        /// Output format: auto (TTY=human / pipe=JSON), pretty|human|text|markdown|md (human), or json
        #[arg(long, default_value = "auto", value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"])]
        format: String,
    },
    /// Bind daily Scope to the path-alias owner of this repo (print-only by default)
    #[command(
        after_help = "Default is print-only (does not write .env). Never silent auto-switch (T240 F2).\nWrite requires both --write-env and --yes. Touches only AI_BRAINS_PROJECT_ID.\ncontext initializes / rotates; it is not adopt-path.\nExamples:\n  ai-brains project adopt-path --format human\n  ai-brains project adopt-path --write-env --yes"
    )]
    AdoptPath {
        /// Rewrite cwd .env AI_BRAINS_PROJECT_ID (requires --yes)
        #[arg(long)]
        write_env: bool,
        /// Confirm the .env write (requires --write-env)
        #[arg(long, requires = "write_env")]
        yes: bool,
        /// Output format: auto (TTY=human / pipe=JSON), pretty|human|text|markdown|md (human), or json
        #[arg(long, default_value = "auto", value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"])]
        format: String,
    },
    /// List every registered filesystem path alias (all roots, not just project-list first path)
    #[command(
        after_help = "Default --format auto = TTY table / pipe JSON. Agents that want a table pass --format human. Scripts pass --format json.\nExamples:\n  ai-brains project list-paths\n  ai-brains project list-paths --format human\n  ai-brains project list-paths --format json\n  ai-brains project list-paths --project <id|alias>\n  ai-brains project list-paths --shared-only\nproject list still shows only the first path per project. This command lists all roots.\n--shared-only keeps owners that appear on two or more roots. Combined with --project is an intersection.\nEmpty filter prints 'No path aliases match.' (exit 0)."
    )]
    ListPaths {
        /// Filter to one project UUID or alias
        #[arg(long)]
        project: Option<String>,
        /// Keep only owners that appear on two or more registered paths
        #[arg(long)]
        shared_only: bool,
        /// Output format: auto (TTY=human / pipe=JSON), pretty|human|text|markdown|md (human), or json
        #[arg(long, default_value = "auto", value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"])]
        format: String,
    },
    /// Move one path alias to another existing project (print-only by default)
    #[command(
        after_help = "Default is print-only (does not append events). Write requires both --write and --yes.\nDoes not move historical memories. Does not write .env (use adopt-path for daily Scope).\nDoes not mint the dest project — run `ai-brains context` in that repo first (already-initialized context upserts that dest without rewriting .env).\nExamples:\n  ai-brains project rebind-path C:\\dev\\crawlx --to <dest-uuid> --format human\n  ai-brains project rebind-path C:\\dev\\crawlx --to <dest-uuid> --write --yes"
    )]
    RebindPath {
        /// Filesystem path to rebind (Windows or WSL form; normalized for compare)
        path: String,
        /// Destination project UUID or alias (must already exist)
        #[arg(long)]
        to: String,
        /// Append Removed+Added events (requires --yes)
        #[arg(long)]
        write: bool,
        /// Confirm the write (requires --write)
        #[arg(long, requires = "write")]
        yes: bool,
        /// Output format: auto (TTY=human / pipe=JSON), pretty|human|text|markdown|md (human), or json
        #[arg(long, default_value = "auto", value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"])]
        format: String,
    },
    /// Discover immediate child directories that contain .ledgerful (dry-run; never writes)
    #[command(
        after_help = "Default --format auto = TTY table / pipe JSON. Agents that want a table pass --format human. Scripts pass --format json.\nDry-run only. Never appends events. Never writes .env. Never auto-registers.\n`--dry-run` is accepted (already dry-run-only).\nA hit is a directory that contains a .ledgerful child. .changeguard alone is not a hit.\n`--root DIR` is a named alias of the positional path (not both). Default is cwd — not the parent.\nAlready-registered hits list the owner; suggested is empty (human —). Use unregister-path / rebind-path to move a bind.\nWhen the implicit-cwd scan has no unregistered hits, human output may print `next: ai-brains project scan-roots --root <git-parent>`.\nExamples:\n  ai-brains project scan-roots\n  ai-brains project scan-roots C:\\dev\n  ai-brains project scan-roots --root C:\\dev\n  ai-brains project scan-roots --dry-run\n  ai-brains project scan-roots --format human\n  ai-brains project scan-roots --format json"
    )]
    ScanRoots {
        /// Directory to scan (default: cwd). Immediate children only.
        path: Option<String>,
        /// Named alias of PATH (conflicts with positional)
        #[arg(long, value_name = "DIR", conflicts_with = "path")]
        root: Option<String>,
        /// Output format: auto (TTY=human / pipe=JSON), pretty|human|text|markdown|md (human), or json
        #[arg(long, default_value = "auto", value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"])]
        format: String,
        /// Accepted no-op alias (command is already dry-run-only; T314 F11)
        #[arg(long)]
        dry_run: bool,
    },
    /// Unregister a filesystem path alias (compensating Removed event; does not forget symbols)
    #[command(
        after_help = "Path is unique. Missing path is idempotent exit 0.\nDoes not delete MemoryPinned / ledgerful:symbol history.\nExamples:\n  ai-brains project unregister-path C:\\dev\\AI-Brains\n  ai-brains project unregister-path --dry-run C:\\dev\\AI-Brains\n  ai-brains project unregister-path --project my-alias C:\\dev\\AI-Brains"
    )]
    UnregisterPath {
        /// Filesystem path to unregister (Windows or WSL form; normalized for compare)
        path: String,
        /// Optional project UUID or alias; if owner ≠ this ref, exit 1
        #[arg(long)]
        project: Option<String>,
        /// Print what would happen; do not append an event
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Clone)]
pub enum DaemonCommands {
    /// Start the daemon in the background
    Start,
    /// Show the status of the running daemon
    #[command(
        after_help = "LLM/Embedding Open is TCP connect to the model process, not the AI-Brains daemon."
    )]
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
    #[command(
        after_help = "Default --keep 10 prunes older vault-*.db.bak by timestamp, not class.\nA residual fleet ((unreadable key) / (legacy plain) / (no core tables)) is kept only with --no-prune.\nDoctor backup_recent only lists the vault sibling backups/ directory — omit --output-dir when the goal is doctor-ok.\nAfter AI_BRAINS_KEY change, old .bak stay KeyMismatch; create a new snapshot.\nExamples:\n  ai-brains --no-project-context backup create --dry-run --no-prune\n  ai-brains --no-project-context backup create --no-prune"
    )]
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
        /// Suppress summary and per-file metadata WARNs (table tokens still apply).
        #[arg(long)]
        quiet: bool,
        /// Per-file detail for non-readable backups (legacy plain / key mismatch / corrupt).
        #[arg(long)]
        verbose: bool,
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
        /// Full per-file OK/FAIL stream (no summary).
        #[arg(long)]
        verbose: bool,
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
    /// Unified query across AI-Brains vault and Ledgerful ledger
    ///
    /// Human vault + ledger pane (default format is always pretty). Agent/JSON path is `recall` (TTY pretty / non-TTY json).
    /// Vault-only search: `recall` / `search`. Governed conclusions/decisions: `query progressive`.
    #[command(
        after_help = "Dash-leading needles need POSIX `--` so clap does not treat them as flags.\n  ai-brains sync query -- --limit     searches for the text `--limit`\n  ai-brains sync query \"text\" --limit 10   sets the vault hit cap to 10\nFlags (--quiet, --no-bridge, --limit N) go before `--`.\nExamples:\n  ai-brains sync query -- --limit\n  ai-brains sync query --quiet -- --limit\n  ai-brains sync query --no-bridge -- --limit"
    )]
    Query {
        /// The query string
        query: String,
        /// Output format (pretty, text, ndjson). Default is always pretty (intentional human-first; agents use `recall` for JSON).
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
        /// Max vault hits after re-rank (default 5; T211 F14/F27).
        #[arg(short = 'l', long, default_value_t = 5)]
        limit: usize,
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
    use std::io::IsTerminal;
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
    // T261 F2: piped trim-empty becomes `""` (then recall_full F1). TTY refuse stays.
    Ok(buf.trim().to_string())
}

/// T86: Read a JSON object from stdin until EOF.
/// Returns an error if stdin is a terminal.
fn read_json_from_stdin() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use std::io::IsTerminal;
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

/// Apply local `.env` force-set for PROJECT_ID/SESSION_ID and emit override policy.
///
/// Returns a **deferred** collapsed debug body when the emit path is Debug
/// (session-only, quiet, seen-fingerprint, or non-warn command). Caller must log
/// it with `tracing::debug!` **after** the tracing subscriber is installed — this
/// function runs before subscriber init (T223 Codex R1).
///
/// Stderr warnings are emitted immediately (eprintln does not need tracing).
/// T242: cross-process session quiet via atomic marker under user home cache.
fn apply_local_project_context_env(
    path: &std::path::Path,
    warn_on_override: bool,
) -> Option<String> {
    use env_warn::{
        EnvOverrideEmit, EnvOverrideFingerprint, EnvWarnPolicy, FORCE_ENV_WARN_KEY, PROJECT_ID_KEY,
        SESSION_ID_KEY, classify_env_overrides, compute_fingerprint_hex, decide_env_override_emit,
        env_warn_truthy, format_override_body, override_body_from_stderr_line,
    };
    use env_warn_session::{MarkerClaim, try_claim_marker};
    use std::sync::atomic::AtomicBool;

    /// Defensive once-per-process belt so a re-entered apply never double-eprintln.
    static ENV_OVERRIDE_STDERR_EMITTED: AtomicBool = AtomicBool::new(false);

    let entries = match dotenvy::from_path_iter(path) {
        Ok(entries) => entries,
        Err(err) => {
            // Runs before subscriber init; dropped unless deferred path used later.
            tracing::warn!("Failed to parse local .env for project context: {}", err);
            return None;
        }
    };

    // Collect differ-gate pairs first, then force-set. Stable order PROJECT then SESSION
    // (do not rely on .env file key order). T223: emit policy only — set_var still always.
    // T242: also retain `.env` values by key match for fingerprint (F4/F24).
    let mut project_override: Option<(String, String)> = None;
    let mut session_override: Option<(String, String)> = None;
    let mut new_project: Option<String> = None;
    let mut new_session: Option<String> = None;
    let mut to_set: Vec<(String, String)> = Vec::new();

    for entry in entries {
        let (key, value) = match entry {
            Ok(entry) => entry,
            Err(err) => {
                tracing::warn!("Skipping malformed local .env entry: {}", err);
                continue;
            }
        };

        if key != PROJECT_ID_KEY && key != SESSION_ID_KEY {
            continue;
        }

        if let Ok(existing) = std::env::var(&key)
            && existing != value
        {
            if key == PROJECT_ID_KEY {
                project_override = Some((key.clone(), existing));
            } else {
                session_override = Some((key.clone(), existing));
            }
        }

        if key == PROJECT_ID_KEY {
            new_project = Some(value.clone());
        } else {
            new_session = Some(value.clone());
        }

        to_set.push((key, value));
    }

    // Always force-set local IDs (F1 precedence frozen).
    for (key, value) in to_set {
        // SAFETY: single-threaded CLI startup before worker threads; process env
        // is intentionally mutated for project-context loading.
        unsafe {
            std::env::set_var(key, value);
        }
    }

    let mut overrides: Vec<(&str, &str)> = Vec::new();
    if let Some((ref k, ref old)) = project_override {
        overrides.push((k.as_str(), old.as_str()));
    }
    if let Some((ref k, ref old)) = session_override {
        overrides.push((k.as_str(), old.as_str()));
    }
    if overrides.is_empty() {
        return None;
    }

    // Quiet / force from process env at apply time (shell or project `.env` already
    // gap-filled). Global `~/.ai-brains/.env` loads after this function — quiet only
    // there is too late.
    let quiet = env_warn_truthy(std::env::var(env_warn::QUIET_ENV_WARN_KEY).ok().as_deref());
    let force = env_warn_truthy(std::env::var(FORCE_ENV_WARN_KEY).ok().as_deref());

    // Non-warn commands: always deferred Debug; never claim markers (F3/F30).
    if !warn_on_override {
        return Some(format_override_body(&overrides));
    }

    let classified = classify_env_overrides(&overrides);
    let decided = decide_env_override_emit(classified, EnvWarnPolicy { quiet, force });

    match decided {
        None => None,
        Some(EnvOverrideEmit::Debug(body)) => Some(body),
        Some(EnvOverrideEmit::Stderr(line)) => {
            if force {
                emit_env_override_stderr_once(&ENV_OVERRIDE_STDERR_EMITTED, &line);
                return None;
            }

            // !force Stderr candidate: atomic marker claim (F3/F5).
            // F4: fingerprint cwd = location-normalized **absolute** `.env` parent.
            // `main_inner` passes relative `Path::new(".env")` whose `.parent()` is
            // empty — resolve against process cwd so different projects do not share
            // an empty-cwd fingerprint (T242 internal R1 P1).
            let env_abs = if path.is_absolute() {
                path.to_path_buf()
            } else {
                match std::env::current_dir() {
                    Ok(cwd) => cwd.join(path),
                    Err(_) => path.to_path_buf(),
                }
            };
            let parent = env_abs.parent().unwrap_or(env_abs.as_path());
            let raw_parent = parent.to_string_lossy();
            let normalized_cwd = ai_brains_path::normalize_for_location_compare(&raw_parent);
            let fingerprint = compute_fingerprint_hex(&EnvOverrideFingerprint {
                normalized_cwd: &normalized_cwd,
                old_shell_project: project_override.as_ref().map(|(_, old)| old.as_str()),
                old_shell_session: session_override.as_ref().map(|(_, old)| old.as_str()),
                new_env_project: new_project.as_deref(),
                new_env_session: new_session.as_deref(),
            });

            let Some(home) = resolve_user_home_for_dotenv() else {
                emit_env_override_stderr_once(&ENV_OVERRIDE_STDERR_EMITTED, &line);
                return None;
            };

            match try_claim_marker(&home, &fingerprint) {
                MarkerClaim::Claimed | MarkerClaim::IoFail => {
                    emit_env_override_stderr_once(&ENV_OVERRIDE_STDERR_EMITTED, &line);
                    None
                }
                MarkerClaim::Exists => {
                    // Seen fingerprint → demote Debug body (F31 strip Warning prefix).
                    Some(override_body_from_stderr_line(&line))
                }
            }
        }
    }
}

/// F9 once-per-process belt: at most one stderr override warning per process.
fn emit_env_override_stderr_once(flag: &std::sync::atomic::AtomicBool, line: &str) {
    use std::sync::atomic::Ordering;
    // swap returns previous: false → first emit; true → already emitted, skip.
    if flag.swap(true, Ordering::SeqCst) {
        return;
    }
    eprintln!("{line}");
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

    // T240 L9: capture shell PROJECT_ID before any project-context force-set / clear
    // so `project whoami` can report shell_project_id when it differs from .env.
    {
        let shell = std::env::var("AI_BRAINS_PROJECT_ID")
            .ok()
            .filter(|s| !s.is_empty());
        commands::project::record_shell_project_id(shell);
    }

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
    // Deferred override-debug body (session-only / quiet / non-warn): emit after
    // tracing subscriber init so RUST_LOG=debug can observe collapsed F3 SOOT (T223).
    let mut deferred_env_override_debug: Option<String> = None;

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
            // Gap-fill only (non-override): shell wins for general keys. PROJECT_ID /
            // SESSION_ID are force-set below in apply_local_project_context_env so local
            // project context beats a stale shell. Global ~/.ai-brains/.env merges after
            // apply — quiet for override warnings must be shell or project `.env` (T223 M1).
            dotenvy::dotenv().ok();
            deferred_env_override_debug =
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

    // T208: `ai_brains_graph=warn` overrides prefix-match from `ai_brains=info`
    // so graph-crate lifecycle INFO cannot leak on the default CLI filter.
    // Escape hatch: `RUST_LOG=ai_brains_graph=debug` (Cozo init is debug after F2).
    let default_filter = tracing_subscriber::EnvFilter::new(DEFAULT_ENV_FILTER);
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

    // T223: deferred collapsed override debug (session-only / quiet / non-warn).
    if let Some(body) = deferred_env_override_debug {
        tracing::debug!("{body}");
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
        | Commands::Dogfood { .. }
        | Commands::Harness { .. } => true,
        // Encrypt may use --source; rotate-datakey needs vault path + async daemon probe.
        Commands::Vault {
            command: VaultCommands::Encrypt { .. },
        } => true,
        Commands::Vault {
            command: VaultCommands::RotateDatakey { .. },
        } => false,
        Commands::AgyHook { schema: true, .. } => true,
        Commands::GrokHook { schema: true, .. } => true,
        Commands::OpencodeHook { schema: true, .. } => true,
        Commands::ClaudeHook { schema: true, .. } => true,
        Commands::CodexHook { schema: true, .. } => true,
        Commands::Sync {
            command: SyncCommands::Pull { schema: true, .. },
        } => true,
        #[cfg(not(feature = "graph"))]
        Commands::Graph { .. } => true,
        _ => false,
    }
}

fn handle_cli_result(res: Result<(), Box<dyn std::error::Error>>) {
    commands::identity_warn::flush_identity_mismatch_warn();
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
/// harness, schema printers, non-graph stub).
fn run_sync_path_free(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match *cli.command {
        Commands::Harness { command } => match command {
            HarnessCommands::Status { format } => {
                commands::harness::run_status(commands::harness::HarnessStatusOptions { format })
            }
            HarnessCommands::Install {
                harness,
                yes,
                dry_run,
            } => commands::harness::run_install(commands::harness::HarnessInstallOptions {
                harness,
                yes,
                dry_run,
            }),
            HarnessCommands::Uninstall {
                harness,
                yes,
                dry_run,
            } => commands::harness::run_uninstall(commands::harness::HarnessUninstallOptions {
                harness,
                yes,
                dry_run,
            }),
            HarnessCommands::ResetDecline { harness } => commands::harness::run_reset_decline(
                commands::harness::HarnessResetDeclineOptions { harness },
            ),
        },
        Commands::AgyHook { schema: true, .. } => {
            print_schema(SCHEMA_AGY_HOOK, "AI-Brains agy-hook payload")
        }
        Commands::GrokHook { schema: true, .. } => {
            print_schema(SCHEMA_GROK_HOOK, "AI-Brains grok-hook payload")
        }
        Commands::OpencodeHook { schema: true, .. } => {
            print_schema(SCHEMA_OPENCODE_HOOK, "AI-Brains opencode-hook payload")
        }
        Commands::ClaudeHook { schema: true, .. } => {
            print_schema(SCHEMA_CLAUDE_HOOK, "AI-Brains claude-hook payload")
        }
        Commands::CodexHook { schema: true, .. } => {
            print_schema(SCHEMA_CODEX_HOOK, "AI-Brains codex-hook payload")
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
                "Reinstall with: {}",
                commands::governed_common::GRAPH_REINSTALL_SOOT
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
        summary,
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
            summary: *summary,
        })
        .await;
    }

    // T320: unified status glance — Status IPC + doctor/graph/nightly compose; no AppContext.
    if let Commands::Status { format } = cli.command.as_ref() {
        let vault_path = cli
            .vault_path
            .clone()
            .ok_or("Vault path is required (--vault-path or AI_BRAINS_VAULT_PATH)")?;
        return commands::status::run(commands::status::StatusOptions {
            vault_path,
            key: cli.key.clone(),
            format: format.clone(),
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
    // T240 F3 / T257 F6: record mismatch; flush after the command (handle_cli_result).
    commands::identity_warn::record_identity_mismatch(&ctx);
    match cli.command.as_ref() {
        Commands::Shadow { .. } => unreachable!("shadow handled in run_sync_path_free"),
        Commands::Migrate { .. } => unreachable!("migrate handled in run_sync_path_free"),
        Commands::Evaluate { .. } => unreachable!("evaluate handled in run_sync_path_free"),
        Commands::Dogfood { .. } => unreachable!("dogfood handled in run_sync_path_free"),
        Commands::Harness { .. } => unreachable!("harness handled in run_sync_path_free"),
        Commands::Vault {
            command: VaultCommands::Encrypt { .. },
        } => unreachable!("vault encrypt handled in run_sync_path_free"),
        Commands::Vault {
            command: VaultCommands::RotateDatakey { .. },
        } => unreachable!("vault rotate-datakey handled before AppContext"),
        Commands::Recovery { .. } => unreachable!("recovery handled before AppContext"),
        Commands::Doctor { .. } => unreachable!("doctor handled before AppContext"),
        Commands::Status { .. } => unreachable!("status handled before AppContext"),
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
                format,
            } => commands::governed_query::run_expand(
                &ctx,
                commands::governed_query::ExpandHandleOptions {
                    handle_id: handle_id.clone(),
                    project_id: *project_id,
                    max_chars: *max_chars,
                    format: format.clone(),
                },
            ),
            GovernedQueryCommands::Trace { trace_id, format } => {
                commands::governed_query::run_trace(
                    &ctx,
                    commands::governed_query::TraceOptions {
                        trace_id: trace_id.clone(),
                        format: format.clone(),
                    },
                )
            }
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
            DecisionCommands::InForce {
                term,
                scope,
                format,
                principal_id,
            } => commands::decision::run_in_force(
                &ctx,
                commands::decision::InForceOptions {
                    term: term.clone(),
                    scope: scope.clone(),
                    format: format.clone(),
                    principal_id: principal_id.clone(),
                },
            ),
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
            PolicyCommands::Bootstrap {
                scope,
                dry_run,
                principal_id,
                format,
            } => commands::policy_cmd::run_bootstrap(
                &ctx,
                commands::policy_cmd::BootstrapOptions {
                    scope: scope.clone(),
                    dry_run: *dry_run,
                    principal_id: principal_id.clone(),
                    format: format.clone(),
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
            min_score,
            graph_boost,
            graph_hop_depth,
            quiet,
            no_bridge,
            global,
            session_last,
            symbols,
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
                    min_score: *min_score,
                    symbols: *symbols,
                    preferred_project_id: if *global { *project_id } else { None },
                },
            )
        }
        Commands::Preflight {
            max_words,
            project_id,
            pretty,
            format,
            compact,
            scope,
            summary,
            global,
            stdin: use_stdin,
            no_hook_prompt,
            install_hooks,
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
            // T214 F3: mirror recall — when --global, clear project_id for label + filter parity.
            let effective_project_id = if *global { None } else { *project_id };
            commands::preflight::run(
                &ctx,
                commands::preflight::PreflightRunOptions {
                    max_words: effective_max_words,
                    project_id: effective_project_id,
                    pretty: *pretty,
                    format: format.clone(),
                    compact: *compact,
                    scope: effective_scope,
                    summary: *summary,
                    global: *global,
                    no_hook_prompt: *no_hook_prompt,
                    install_hooks: *install_hooks,
                    stdin_mode: *use_stdin,
                },
            )
        }
        Commands::Nightly {
            schedule,
            unschedule,
            start_time,
            status,
            quick,
            format,
            skip_import,
            skip_import_agy,
            skip_import_grok,
            skip_import_opencode,
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
                *skip_import_agy,
                *skip_import_grok,
                *skip_import_opencode,
                *run_as_system,
                *dry_run,
                *quick,
                format.clone(),
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
            Some(BackupCommands::List { quiet, verbose }) => {
                use ai_brains_brain::ListMode;
                commands::backup::run_list(&ctx, ListMode::from_flags(*quiet, *verbose))
            }
            Some(BackupCommands::Verify {
                path,
                full,
                format,
                verbose,
            }) => commands::backup::run_verify(&ctx, path.clone(), *full, format.clone(), *verbose),
            None => commands::backup::run_create(&ctx, None, Some(10), *dry_run, true),
        },
        Commands::Forget {
            memory_id,
            match_query,
            force,
            list_forgotten,
            restore,
            dry_run,
            global,
            limit,
            format,
            tag,
            project_id,
        } => {
            let effective_project_id = if *global { None } else { *project_id };
            commands::forget::run(
                &ctx,
                memory_id.clone(),
                match_query.clone(),
                *force,
                *list_forgotten,
                restore.clone(),
                *dry_run,
                *global,
                *limit,
                format.clone(),
                tag.clone(),
                effective_project_id,
            )
        }
        Commands::Memory { command } => match command {
            MemoryCommands::List {
                status,
                limit,
                global,
                format,
                summary,
                tag,
                project_id,
            } => {
                let effective_project_id = if *global { None } else { *project_id };
                commands::memory::run_list(
                    &ctx,
                    commands::memory::MemoryListOptions {
                        status: status.clone(),
                        limit: *limit,
                        global: *global,
                        format: format.clone(),
                        summary: *summary,
                        tag: tag.clone(),
                        project_id: effective_project_id,
                    },
                )
            }
        },
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
            DeviceCommands::Status => commands::device::run_status(&ctx),
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
                limit,
            } => {
                commands::sync::run_query(
                    &ctx,
                    query.clone(),
                    format.clone(),
                    *quiet,
                    *global,
                    *no_bridge,
                    *limit,
                )
                .await
            }
        },
        Commands::AntigravityImport { days, force } => {
            commands::antigravity_import::run(&ctx, *days, *force)
        }
        Commands::GrokImport {
            days,
            force,
            dry_run,
        } => commands::grok_import::run(&ctx, *days, *force, *dry_run),
        Commands::OpencodeImport {
            days,
            force,
            dry_run,
            max_sessions,
        } => commands::opencode_import::run(&ctx, *days, *force, *dry_run, *max_sessions),
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
        Commands::GrokHook { payload, schema } => {
            if *schema {
                print_schema(SCHEMA_GROK_HOOK, "AI-Brains grok-hook payload")
            } else if let Some(p) = payload {
                commands::grok_hook::run(&ctx, p)
            } else {
                Err(
                    "Either provide --payload <json> or use --schema to print the payload schema."
                        .into(),
                )
            }
        }
        Commands::OpencodeHook { payload, schema } => {
            if *schema {
                print_schema(SCHEMA_OPENCODE_HOOK, "AI-Brains opencode-hook payload")
            } else if let Some(p) = payload {
                commands::opencode_hook::run(&ctx, p)
            } else {
                Err(
                    "Either provide --payload <json> or use --schema to print the payload schema."
                        .into(),
                )
            }
        }
        Commands::ClaudeImport {
            days,
            force,
            dry_run,
        } => commands::claude_import::run(&ctx, *days, *force, *dry_run),
        Commands::CodexImport {
            days,
            force,
            dry_run,
        } => commands::codex_import::run(&ctx, *days, *force, *dry_run),
        Commands::ClaudeHook { payload, schema } => {
            if *schema {
                print_schema(SCHEMA_CLAUDE_HOOK, "AI-Brains claude-hook payload")
            } else if let Some(p) = payload {
                commands::claude_hook::run(&ctx, p)
            } else {
                Err(
                    "Either provide --payload <json> or use --schema to print the payload schema."
                        .into(),
                )
            }
        }
        Commands::CodexHook { payload, schema } => {
            if *schema {
                print_schema(SCHEMA_CODEX_HOOK, "AI-Brains codex-hook payload")
            } else if let Some(p) = payload {
                commands::codex_hook::run(&ctx, p)
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
            ProjectCommands::List { format } => commands::project::list(&ctx, format),
            ProjectCommands::Resolve {
                alias_positional,
                alias,
            } => commands::project::resolve(&ctx, alias_positional.clone(), alias.clone()),
            ProjectCommands::Detect { export } => commands::project::detect(&ctx, *export),
            ProjectCommands::SetAlias { project_id, alias } => {
                commands::project::set_alias(&ctx, project_id, alias)
            }
            ProjectCommands::RegisterPath { project_ref, path } => {
                commands::project::register_path(&ctx, project_ref, path)
            }
            ProjectCommands::Whoami { format } => commands::project::whoami(&ctx, format),
            ProjectCommands::AdoptPath {
                write_env,
                yes,
                format,
            } => commands::project_adopt::run(&ctx, *write_env, *yes, format),
            ProjectCommands::ListPaths {
                project,
                shared_only,
                format,
            } => {
                commands::project_paths::list_paths(&ctx, format, project.as_deref(), *shared_only)
            }
            ProjectCommands::RebindPath {
                path,
                to,
                write,
                yes,
                format,
            } => commands::project_rebind::run(&ctx, path, to, *write, *yes, format),
            ProjectCommands::ScanRoots {
                path,
                root,
                format,
                dry_run: _,
            } => commands::project_paths::scan_roots(
                &ctx,
                root.as_deref().or(path.as_deref()),
                format,
            ),
            ProjectCommands::UnregisterPath {
                path,
                project,
                dry_run,
            } => commands::project_paths::unregister_path(&ctx, path, project.as_deref(), *dry_run),
        },
        #[cfg(feature = "graph")]
        Commands::Graph { command, .. } => match command {
            GraphCommands::Rebuild { dry_run, format } => {
                commands::graph::rebuild(&ctx, *dry_run, format).await
            }
            GraphCommands::Neighbors {
                memory_id,
                format,
                limit,
            } => commands::graph::neighbors(&ctx, memory_id, format, *limit),
            GraphCommands::Hierarchy {
                memory_id,
                format,
                limit,
            } => commands::graph::hierarchy(&ctx, memory_id, format, *limit),
            GraphCommands::Session {
                session_id,
                format,
                limit,
            } => commands::graph::session(&ctx, session_id, format, *limit),
            GraphCommands::Update { format } => commands::graph::update(&ctx, format),
        },
        #[cfg(not(feature = "graph"))]
        Commands::Graph { .. } => {
            println!(
                "{}: The graph subcommand requires a --features graph build.",
                commands::governed_common::FEATURE_UNAVAILABLE
            );
            println!(
                "Reinstall with: {}",
                commands::governed_common::GRAPH_REINSTALL_SOOT
            );
            std::process::exit(commands::governed_common::exit_code_feature_unavailable());
        }
    }
}
