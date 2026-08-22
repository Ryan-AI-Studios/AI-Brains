//! T276 — attach T264-class project tags to `--global` pretty recall hits.

use super::preflight_pretty::{unique_project_id_for_tag, upgrade_global_tag};
use ai_brains_retrieval::RecallHit;
use ai_brains_store::{QueryStore, VaultConnection};
use std::collections::HashMap;

/// One optional leading tag per hit (`[8hex]` or upgraded `display_label`).
pub(crate) fn tags_for_hits(
    conn: &VaultConnection,
    hits: &[RecallHit],
) -> Result<Vec<Option<String>>, Box<dyn std::error::Error>> {
    let listed = conn.list_projects()?;
    let mut names: HashMap<String, (String, String)> = HashMap::new();
    let mut ids: Vec<String> = Vec::new();
    for (id, name, alias, _) in listed {
        names.insert(id.clone(), (name, alias));
        ids.push(id);
    }
    for hit in hits {
        if let Some(pid) = hit.project_id.as_deref()
            && !pid.is_empty()
            && !ids.iter().any(|existing| existing == pid)
        {
            ids.push(pid.to_string());
        }
    }

    let mut tags = Vec::with_capacity(hits.len());
    for hit in hits {
        tags.push(tag_for_project_id(hit.project_id.as_deref(), &names, &ids));
    }
    Ok(tags)
}

fn tag_for_project_id(
    project_id: Option<&str>,
    names: &HashMap<String, (String, String)>,
    ids: &[String],
) -> Option<String> {
    let id = project_id.filter(|s| !s.is_empty())?;
    if id.len() < 8 {
        return Some("[unknown]".to_string());
    }
    let tag8 = &id[..8];
    let raw = format!("[{tag8}]");
    let unique = unique_project_id_for_tag(ids.iter().map(String::as_str), tag8);
    if unique.as_deref() != Some(id) {
        return Some(raw);
    }
    match names.get(id) {
        Some((name, alias)) => Some(upgrade_global_tag(&raw, Some((id, name, alias)))),
        None => Some(raw),
    }
}
