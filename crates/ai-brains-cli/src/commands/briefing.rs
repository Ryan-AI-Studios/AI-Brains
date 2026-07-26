//! Thin CLI surface for typed Project / Personal briefings (T152-P1-06).
//!
//! Defaults to dry-run JSON on stdout. Requires vault path + grants like governed preflight.

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
    let fmt = format.unwrap_or("json");
    if fmt.eq_ignore_ascii_case("markdown") || fmt.eq_ignore_ascii_case("md") {
        println!("{}", markdown());
    } else {
        println!("{}", json()?);
    }
    Ok(())
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
