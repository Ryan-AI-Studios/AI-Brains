//! Scenario 9 — windows_wsl_repo_alias.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

use super::SeedOutcome;
use super::common::{agent, register, resolve_for_project, stable_project};
use crate::adapters::StorePorts;
use crate::errors::Result;
use crate::grants::register_path_alias;
use crate::scope_resolver::{ScopeResolveInput, resolve_scope};
use crate::sources::scope_identity_key;

pub fn seed(ports: &StorePorts, params: &BTreeMap<String, Value>) -> Result<SeedOutcome> {
    let project_id = stable_project("path-alias");
    let agent_p = agent("path-alias-agent");
    register(ports, &agent_p)?;

    let win = params
        .get("win_path")
        .and_then(|v| v.as_str())
        .unwrap_or(r"C:\Dev\EvalProject")
        .to_string();
    let wsl = params
        .get("wsl_path")
        .and_then(|v| v.as_str())
        .unwrap_or("/mnt/c/Dev/EvalProject")
        .to_string();

    register_path_alias(&ports.writer, &win, project_id)?;
    register_path_alias(&ports.writer, &wsl, project_id)?;

    let identity = ports.identity_store();
    let win_input = ScopeResolveInput {
        cwd: PathBuf::from(&win),
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
        git_metadata: None,
    };
    let wsl_input = ScopeResolveInput {
        cwd: PathBuf::from(&wsl),
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
        git_metadata: None,
    };

    let a = resolve_scope(&win_input, &identity)?;
    let b = resolve_scope(&wsl_input, &identity)?;
    let key_a = scope_identity_key(&a.scope);
    let key_b = scope_identity_key(&b.scope);

    Ok(SeedOutcome {
        principal: agent_p,
        project_id,
        resolve: resolve_for_project(project_id),
        scope_keys: Some((key_a, key_b)),
        claim_ids: vec![],
        require_citations: false,
        expect_denied: true,
        ..SeedOutcome::default()
    })
}
