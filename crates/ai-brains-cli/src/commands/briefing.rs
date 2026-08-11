//! Thin CLI surface for typed Project / Personal briefings (T152-P1-06 / T202 / T227).
//!
//! Format default (F4/F9): TTY + no `--format` → markdown; non-TTY → json; explicit wins.
//! Human aliases (`human|pretty|text|markdown|md`) → markdown; only `json` → JSON;
//! unknown → `fail_usage` exit 2 (F1–F3). Requires vault path + grants like governed preflight.

use crate::commands::governed_common::fail_usage;
use crate::context::AppContext;
use ai_brains_control_plane::{
    BudgetConfig, PersonalBriefingRequest, ProjectBriefingRequest, ScopeResolveInput, StorePorts,
    SystemClock, build_personal_briefing, build_project_briefing, make_principal,
    render_personal_markdown, render_project_markdown,
};
use ai_brains_core::ids::{PrincipalId, ProjectId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_store::SqliteEventStore;
use is_terminal::IsTerminal;

pub struct ProjectBriefingOptions {
    pub project_id: Option<ProjectId>,
    pub max_words: usize,
    pub dry_run: bool,
    pub format: Option<String>,
}

pub struct PersonalBriefingOptions {
    pub user_id: Option<UserId>,
    pub max_words: usize,
    pub dry_run: bool,
    pub format: Option<String>,
}

/// Type-safe briefing emit routing (T227 F28).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BriefingFormatKind {
    Markdown,
    Json,
}

/// Classify briefing `--format` (T227 F1–F4, F26).
///
/// Trim + lowercase before match. Unknown tokens return `Err` with accepted list
/// (caller maps to `fail_usage` exit 2).
fn classify_briefing_format(
    explicit: Option<&str>,
    is_tty: bool,
) -> Result<BriefingFormatKind, String> {
    match explicit.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        None => Ok(if is_tty {
            BriefingFormatKind::Markdown
        } else {
            BriefingFormatKind::Json
        }),
        Some("json") => Ok(BriefingFormatKind::Json),
        Some("markdown") | Some("md") | Some("human") | Some("pretty") | Some("text") => {
            Ok(BriefingFormatKind::Markdown)
        }
        Some(other) => Err(format!(
            "unknown --format '{other}' (accepted: human, pretty, text, markdown, md, json)"
        )),
    }
}

/// `ai-brains briefing project` — build a typed ProjectBriefingPacket.
pub fn run_project(
    ctx: &AppContext,
    options: ProjectBriefingOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let clock = SystemClock;
    let policy = ports.production_policy();
    let identity = ports.identity_store();
    let principal = cli_principal();
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let writer = if options.dry_run {
        None
    } else {
        Some(&ports.writer)
    };
    let packet = build_project_briefing(
        writer,
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal,
            resolve: ScopeResolveInput {
                cwd,
                explicit_project_id: options.project_id,
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig {
                max_words: options.max_words,
                ..BudgetConfig::default()
            },
            privacy: Privacy::LocalOnly,
            dry_run: options.dry_run,
            briefing_id: None,
            ledgerful: None,
        },
    )?;

    emit_output(
        options.format.as_deref(),
        || render_project_markdown(&packet),
        || serde_json::to_string_pretty(&packet),
    )
}

/// `ai-brains briefing personal` — build a Personal Continuity packet.
pub fn run_personal(
    ctx: &AppContext,
    options: PersonalBriefingOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let clock = SystemClock;
    let policy = ports.production_policy();
    let principal = cli_principal();
    let user_id = options.user_id.unwrap_or_else(|| {
        // Default: map principal id into a user id when possible; else nil sentinel.
        UserId::from_uuid(principal.id.as_uuid())
    });

    let writer = if options.dry_run {
        None
    } else {
        Some(&ports.writer)
    };
    let grant_store = ports.grant_store();
    // T152-FRESH3-P2-01: only Personal-scope grants for this user (never project grants).
    let personal_scope_key =
        ai_brains_control_plane::scope_identity_key(&ScopeRef::Personal(user_id));
    let packet = build_personal_briefing(
        writer,
        &ports.query,
        &clock,
        &policy,
        |p| {
            grant_store.list_applied_grants(
                p.id,
                &personal_scope_key,
                Some(&["ReadConclusions", "ReadDecisions"]),
            )
        },
        PersonalBriefingRequest {
            principal,
            user_id,
            budget: BudgetConfig {
                max_words: options.max_words,
                ..BudgetConfig::default()
            },
            privacy: Privacy::LocalOnly,
            dry_run: options.dry_run,
            briefing_id: None,
        },
    )?;

    emit_output(
        options.format.as_deref(),
        || render_personal_markdown(&packet),
        || serde_json::to_string_pretty(&packet),
    )
}

fn emit_output(
    format: Option<&str>,
    markdown: impl FnOnce() -> String,
    json: impl FnOnce() -> Result<String, serde_json::Error>,
) -> Result<(), Box<dyn std::error::Error>> {
    match classify_briefing_format(format, std::io::stdout().is_terminal()) {
        Ok(BriefingFormatKind::Markdown) => {
            println!("{}", markdown());
            Ok(())
        }
        Ok(BriefingFormatKind::Json) => {
            println!("{}", json()?);
            Ok(())
        }
        // F3/F25/F32: fail_usage → exit 2, zero stdout (no JSON pollution).
        Err(msg) => fail_usage(msg),
    }
}

/// Principal for CLI governed surfaces (shared with preflight conventions).
///
/// Override with `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID=<uuid>`. Principal must be
/// registered and hold appropriate read grants, or packets return denied/empty.
pub(crate) fn cli_principal() -> ai_brains_core::principal::Principal {
    if let Ok(raw) = std::env::var("AI_BRAINS_PREFLIGHT_PRINCIPAL_ID") {
        let trimmed = raw.trim();
        if let Ok(u) = uuid::Uuid::parse_str(trimmed) {
            return make_principal(PrincipalKind::Human, PrincipalId::from_uuid(u), "cli-human");
        }
    }
    make_principal(
        PrincipalKind::System,
        PrincipalId::from_uuid(uuid::Uuid::from_u128(
            0xA1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2,
        )),
        "cli-system",
    )
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn classify_briefing_format__explicit_json__returns_json() {
        assert_eq!(
            classify_briefing_format(Some("json"), true),
            Ok(BriefingFormatKind::Json)
        );
        assert_eq!(
            classify_briefing_format(Some("json"), false),
            Ok(BriefingFormatKind::Json)
        );
    }

    #[test]
    fn classify_briefing_format__explicit_markdown_aliases__returns_markdown() {
        for alias in ["markdown", "md", "human", "pretty", "text"] {
            assert_eq!(
                classify_briefing_format(Some(alias), true),
                Ok(BriefingFormatKind::Markdown),
                "alias {alias}"
            );
            assert_eq!(
                classify_briefing_format(Some(alias), false),
                Ok(BriefingFormatKind::Markdown),
                "alias {alias} non-tty"
            );
        }
    }

    #[test]
    fn classify_briefing_format__no_explicit_on_tty__returns_markdown() {
        assert_eq!(
            classify_briefing_format(None, true),
            Ok(BriefingFormatKind::Markdown)
        );
    }

    #[test]
    fn classify_briefing_format__no_explicit_not_tty__returns_json() {
        assert_eq!(
            classify_briefing_format(None, false),
            Ok(BriefingFormatKind::Json)
        );
    }

    #[test]
    fn classify_briefing_format__trim_and_case__returns_markdown() {
        // F26 / AC5b
        assert_eq!(
            classify_briefing_format(Some(" markdown"), true),
            Ok(BriefingFormatKind::Markdown)
        );
        assert_eq!(
            classify_briefing_format(Some("HUMAN"), false),
            Ok(BriefingFormatKind::Markdown)
        );
        assert_eq!(
            classify_briefing_format(Some("  Pretty  "), true),
            Ok(BriefingFormatKind::Markdown)
        );
    }

    #[test]
    fn classify_briefing_format__unknown__returns_err_with_accepted_list() {
        let err = classify_briefing_format(Some("banana"), true).expect_err("unknown");
        assert!(
            err.contains("unknown --format 'banana'"),
            "message must name token: {err}"
        );
        assert!(
            err.contains("human")
                && err.contains("pretty")
                && err.contains("text")
                && err.contains("markdown")
                && err.contains("md")
                && err.contains("json"),
            "message must list accepted values: {err}"
        );
    }
}
