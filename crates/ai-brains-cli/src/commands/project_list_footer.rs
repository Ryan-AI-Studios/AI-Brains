//! T267 — `project list` unaliased footer pick + suggestion (F3 / F3b / F10).
//!
//! `pick_unaliased_footer_target` and `footer_alias_suggestion` are pure.
//! The print wrapper does I/O (cwd, git identity, path-alias lookup).

use crate::commands::project::{
    collect_git_identity, resolve_path_alias_for_location, sanitize_alias_suggestion,
};
use crate::context::AppContext;
use ai_brains_store::{ProjectListDetail, QueryStore};
use std::collections::HashMap;

/// One unaliased list row plus its registered-path count (not first-path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FooterCandidate {
    pub project_id: String,
    pub path: Option<String>,
    pub path_count: usize,
}

/// F3 pick among unaliased rows (already memory-DESC).
pub(crate) fn pick_unaliased_footer_target<'a>(
    unaliased: &'a [FooterCandidate],
    cwd_owner: Option<&str>,
) -> Option<&'a FooterCandidate> {
    if unaliased.is_empty() {
        return None;
    }
    if let Some(owner) = cwd_owner
        && let Some(hit) = unaliased.iter().find(|c| c.project_id == owner)
    {
        return Some(hit);
    }
    if let Some(hit) = unaliased.iter().find(|c| c.path_count == 1) {
        return Some(hit);
    }
    if let Some(hit) = unaliased.iter().find(|c| c.path_count == 0) {
        return Some(hit);
    }
    Some(&unaliased[0])
}

/// Last non-empty path component; skip empty and drive-only (`C:` / `C:\`).
fn last_path_component(path: &str) -> Option<&str> {
    let trimmed = path.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        return None;
    }
    let bytes = trimmed.as_bytes();
    if bytes.len() == 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return None;
    }
    trimmed.rsplit(['/', '\\']).find(|s| !s.is_empty())
}

/// F3b: cwd slug only when the picked id is the cwd path-owner.
pub(crate) fn footer_alias_suggestion(
    target_id: &str,
    cwd_owner: Option<&str>,
    cwd_slug: Option<&str>,
    target_path: Option<&str>,
) -> String {
    if cwd_owner == Some(target_id)
        && let Some(slug) = cwd_slug
    {
        let cleaned = sanitize_alias_suggestion(slug);
        if !cleaned.is_empty() {
            return cleaned;
        }
    }
    if let Some(path) = target_path
        && let Some(comp) = last_path_component(path)
    {
        let cleaned = sanitize_alias_suggestion(comp);
        if !cleaned.is_empty() {
            return cleaned;
        }
    }
    "my-project".to_string()
}

pub(crate) fn print_unaliased_footer(
    ctx: &AppContext,
    projects: &[ProjectListDetail],
) -> Result<(), Box<dyn std::error::Error>> {
    let unaliased: Vec<&ProjectListDetail> =
        projects.iter().filter(|p| p.alias.is_empty()).collect();
    if unaliased.is_empty() {
        return Ok(());
    }

    let aliases = ctx.conn.list_path_aliases()?;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (pid, _path) in &aliases {
        *counts.entry(pid.to_string()).or_insert(0) += 1;
    }

    let candidates: Vec<FooterCandidate> = unaliased
        .iter()
        .map(|p| FooterCandidate {
            project_id: p.project_id.clone(),
            path: p.path.clone(),
            path_count: counts.get(&p.project_id).copied().unwrap_or(0),
        })
        .collect();

    // Git / cwd probes are best-effort (pre-T267). A missing `git` binary must
    // not fail `project list` — F3b falls back to path basename / `my-project`.
    let (cwd_owner, slug) = match std::env::current_dir() {
        Ok(cwd) => {
            let git = collect_git_identity(&cwd).unwrap_or_default();
            let owner = resolve_path_alias_for_location(ctx.conn.as_ref(), &cwd, &git)?;
            (owner, git.slug)
        }
        Err(_) => (None, None),
    };
    let Some(target) = pick_unaliased_footer_target(&candidates, cwd_owner.as_deref()) else {
        return Ok(());
    };
    let suggestion = footer_alias_suggestion(
        &target.project_id,
        cwd_owner.as_deref(),
        slug.as_deref(),
        target.path.as_deref(),
    );

    eprintln!("{} project(s) have no alias.", unaliased.len());
    eprintln!(
        "Example: ai-brains project set-alias {} {}",
        target.project_id, suggestion
    );
    Ok(())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    fn cand(id: &str, path: Option<&str>, path_count: usize) -> FooterCandidate {
        FooterCandidate {
            project_id: id.to_string(),
            path: path.map(str::to_string),
            path_count,
        }
    }

    #[test]
    fn pick_unaliased_footer_target__cwd_owner_unaliased__wins() {
        let leftover = cand(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            Some(r"C:\dev\crawlx"),
            11,
        );
        let owner = cand(
            "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
            Some(r"C:\dev\AI-Brains"),
            1,
        );
        let rows = [leftover, owner];
        let hit = pick_unaliased_footer_target(&rows, Some("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"))
            .expect("cwd owner present");
        assert_eq!(
            hit.project_id, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
            "F3(1): unaliased cwd path-owner wins over leftover"
        );
    }

    #[test]
    fn pick_unaliased_footer_target__single_path_before_orphan() {
        let leftover = cand(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            Some(r"C:\dev\crawlx"),
            11,
        );
        let single = cand(
            "cccccccc-cccc-cccc-cccc-cccccccccccc",
            Some(r"C:\dev\only"),
            1,
        );
        let orphan = cand("dddddddd-dddd-dddd-dddd-dddddddddddd", None, 0);
        let rows = [leftover, single, orphan];
        let hit = pick_unaliased_footer_target(&rows, None).expect("candidate");
        assert_eq!(
            hit.project_id, "cccccccc-cccc-cccc-cccc-cccccccccccc",
            "F3(2): first path_count==1 beats leftover and orphan"
        );
    }

    #[test]
    fn pick_unaliased_footer_target__orphan_before_last_resort() {
        let leftover = cand(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            Some(r"C:\dev\crawlx"),
            11,
        );
        let orphan = cand("dddddddd-dddd-dddd-dddd-dddddddddddd", None, 0);
        let rows = [leftover, orphan];
        let hit = pick_unaliased_footer_target(&rows, None).expect("candidate");
        assert_eq!(
            hit.project_id, "dddddddd-dddd-dddd-dddd-dddddddddddd",
            "F3(3): orphan (path_count==0) beats leftover last-resort"
        );
    }

    #[test]
    fn footer_alias_suggestion__non_owner__not_cwd_slug() {
        let got = footer_alias_suggestion(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            Some("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
            Some("AI-Brains"),
            Some(r"C:\dev\crawlx"),
        );
        assert_eq!(
            got, "crawlx",
            "F3b: non-owner must use path basename, not cwd slug; got {got}"
        );
    }

    #[test]
    fn footer_alias_suggestion__cwd_owner__uses_slug() {
        let got = footer_alias_suggestion(
            "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
            Some("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
            Some("AI-Brains"),
            Some(r"C:\dev\AI-Brains"),
        );
        assert_eq!(got, "AI-Brains", "F3b: cwd owner uses sanitized slug");
    }

    #[test]
    fn footer_alias_suggestion__no_path__my_project() {
        let got = footer_alias_suggestion(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            Some("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
            Some("AI-Brains"),
            None,
        );
        assert_eq!(got, "my-project", "F3b: no path falls back to my-project");
    }
}
