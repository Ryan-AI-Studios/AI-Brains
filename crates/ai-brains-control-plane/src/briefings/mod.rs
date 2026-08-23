//! Deterministic Project / Personal briefings (T152).
//!
//! v1 is **projection select + render** — no LLM.

pub mod budget;
pub mod personal;
pub mod project;
pub mod renderer;

pub use budget::{BudgetConfig, apply_budget, apply_personal_budget};
pub use personal::{PersonalBriefingRequest, build_personal_briefing};
pub use project::{BRIEFING_POLICY_VERSION, ProjectBriefingRequest, build_project_briefing};
pub use renderer::{
    VaultPinStanza, render_personal_markdown, render_project_json, render_project_markdown,
    render_project_markdown_with_vault_pins,
};
