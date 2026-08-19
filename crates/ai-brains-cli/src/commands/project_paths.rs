//! T254 — `project list-paths`, `project scan-roots`, and `project unregister-path`.
//!
//! Sibling of `project.rs` so the hotspot does not grow. Returns `Err` /
//! `fail_usage`; never calls `process::exit` (F37).

use crate::commands::project::display_label;
use crate::context::AppContext;
use ai_brains_store::QueryStore;
use serde::Serialize;
use std::collections::HashMap;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

pub(crate) const SCAN_CHILD_CAP: usize = 200;

#[derive(Debug, Clone)]
pub(crate) struct ScanChildResult {
    pub path: PathBuf,
    pub readable: bool,
    pub is_dir: bool,
    pub has_ledgerful: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ScanDiscovery {
    pub hits: Vec<PathBuf>,
    pub truncated: bool,
    pub unreadable_count: usize,
}

/// Cap + unreadable policy for `scan-roots` (F21). Pure: no filesystem I/O.
pub(crate) fn discover_scan_hits(
    scan_root: &Path,
    root_has_ledgerful: bool,
    children: impl IntoIterator<Item = ScanChildResult>,
    cap: usize,
) -> ScanDiscovery {
    let mut hits = Vec::new();
    if root_has_ledgerful {
        hits.push(scan_root.to_path_buf());
    }

    let mut truncated = false;
    let mut unreadable_count = 0;
    for (considered, child) in children.into_iter().enumerate() {
        if considered >= cap {
            truncated = true;
            break;
        }
        if !child.readable {
            unreadable_count += 1;
            continue;
        }
        if child.is_dir && child.has_ledgerful {
            hits.push(child.path);
        }
    }

    ScanDiscovery {
        hits,
        truncated,
        unreadable_count,
    }
}

// ---------------------------------------------------------------------------
// list-paths (F1 / F9–F12)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct ListPathsJson {
    api_version: String,
    paths: Vec<ListPathRow>,
}

#[derive(Debug, Serialize)]
struct ListPathRow {
    project_id: String,
    label: String,
    alias: String,
    normalized_path: String,
    exists: bool,
}

/// List every registered filesystem path alias (all roots, not first-path-only).
///
/// `--project` and `--shared-only` filter which rows appear; unfiltered JSON
/// keys stay T254 F10. `--shared-only` = owner appears ≥2 times in the full
/// alias list. Combined flags are an intersection.
pub fn list_paths(
    ctx: &AppContext,
    format: &str,
    project: Option<&str>,
    shared_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let use_json =
        crate::commands::format_resolve::is_json_output(format, std::io::stdout().is_terminal());

    let aliases = ctx.conn.list_path_aliases()?;
    let projects = ctx.conn.list_projects()?;
    let mut by_id: HashMap<String, (String, String)> = HashMap::new();
    for (id, name, alias, _) in projects {
        by_id.insert(id, (name, alias));
    }

    let mut owner_counts: HashMap<String, usize> = HashMap::new();
    for (project_id, _) in &aliases {
        *owner_counts.entry(project_id.to_string()).or_insert(0) += 1;
    }

    let filter_project = if let Some(pref) = project {
        Some(resolve_project_ref(ctx, pref)?.to_string())
    } else {
        None
    };

    let mut rows = Vec::with_capacity(aliases.len());
    for (project_id, normalized_path) in aliases {
        let id = project_id.to_string();
        if let Some(ref wanted) = filter_project
            && &id != wanted
        {
            continue;
        }
        if shared_only && owner_counts.get(&id).copied().unwrap_or(0) < 2 {
            continue;
        }
        let (name, alias) = by_id
            .get(&id)
            .cloned()
            .unwrap_or_else(|| (String::new(), String::new()));
        let label = display_label(&name, &alias, &id);
        let exists = Path::new(&normalized_path).exists();
        rows.push(ListPathRow {
            project_id: id,
            label,
            alias,
            normalized_path,
            exists,
        });
    }

    let filter_applied = project.is_some() || shared_only;

    if use_json {
        let envelope = ListPathsJson {
            api_version: "1".to_string(),
            paths: rows,
        };
        crate::commands::identity_warn::print_json_stdout(&envelope)?;
        return Ok(());
    }

    if rows.is_empty() {
        if filter_applied {
            println!("No path aliases match.");
        } else {
            println!("No path aliases registered.");
            println!("next: ai-brains project register-path <project_id|alias> <path>");
        }
        return Ok(());
    }

    println!(
        "{:<40} {:<20} {:<36} exists",
        "path", "project", "project_id"
    );
    for row in rows {
        let exists = if row.exists { "ok" } else { "missing" };
        println!(
            "{:<40} {:<20} {:<36} {exists}",
            row.normalized_path, row.label, row.project_id
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// scan-roots (F3 / F20–F23)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct ScanRootsJson {
    api_version: String,
    scan_root: String,
    truncated: bool,
    roots: Vec<ScanRootRow>,
}

#[derive(Debug, Serialize)]
struct ScanRootRow {
    path: String,
    registered_project_id: Option<String>,
    exists: bool,
    suggested: String,
}

/// Discover immediate child directories that contain `.ledgerful` (dry-run).
pub fn scan_roots(
    ctx: &AppContext,
    path: Option<&str>,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let use_json =
        crate::commands::format_resolve::is_json_output(format, std::io::stdout().is_terminal());

    let scan_root = match path {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        Some(_) => {
            return crate::commands::governed_common::fail_usage(
                "scan-roots path is empty; pass a directory or omit to use the current directory",
            );
        }
        None => std::env::current_dir()?,
    };

    if !scan_root.is_dir() {
        return Err(format!(
            "scan-roots path is not a directory: {}",
            scan_root.display()
        )
        .into());
    }

    let root_has_ledgerful = dir_has_ledgerful(&scan_root);
    let mut children = collect_scan_children(&scan_root)?;
    children.sort_by(|a, b| a.path.cmp(&b.path));

    let discovery = discover_scan_hits(&scan_root, root_has_ledgerful, children, SCAN_CHILD_CAP);
    if discovery.truncated {
        eprintln!("scan-roots: listing truncated at {SCAN_CHILD_CAP} children");
    }
    if discovery.unreadable_count > 0 {
        eprintln!(
            "scan-roots: skipped {} unreadable path(s)",
            discovery.unreadable_count
        );
    }

    let owners = alias_owner_map(ctx)?;
    let mut hits = discovery.hits;
    hits.sort();
    let roots = scan_rows_for_hits(&hits, &owners);

    if use_json {
        emit_scan_json(&scan_root, discovery.truncated, roots)?;
    } else {
        emit_scan_human(roots);
    }
    Ok(())
}

fn alias_owner_map(
    ctx: &AppContext,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let aliases = ctx.conn.list_path_aliases()?;
    let mut owners = HashMap::new();
    for (project_id, normalized) in aliases {
        owners.insert(normalized, project_id.to_string());
    }
    Ok(owners)
}

fn scan_rows_for_hits(hits: &[PathBuf], owners: &HashMap<String, String>) -> Vec<ScanRootRow> {
    let mut roots = Vec::with_capacity(hits.len());
    for hit in hits {
        let display = hit.to_string_lossy().into_owned();
        let key = ai_brains_path::normalize_for_location_compare(&display);
        let registered_project_id = owners.get(&key).cloned();
        let exists = hit.exists();
        let project_ref = registered_project_id
            .as_deref()
            .unwrap_or("<project-id-or-alias>");
        let suggested = format!("ai-brains project register-path {project_ref} {display}");
        roots.push(ScanRootRow {
            path: display,
            registered_project_id,
            exists,
            suggested,
        });
    }
    roots
}

fn emit_scan_json(
    scan_root: &Path,
    truncated: bool,
    roots: Vec<ScanRootRow>,
) -> Result<(), Box<dyn std::error::Error>> {
    let envelope = ScanRootsJson {
        api_version: "1".to_string(),
        scan_root: scan_root.to_string_lossy().into_owned(),
        truncated,
        roots,
    };
    crate::commands::identity_warn::print_json_stdout(&envelope)
}

fn emit_scan_human(roots: Vec<ScanRootRow>) {
    println!(
        "{:<40} {:<36} {:<8} suggested",
        "path", "registered_to", "disk"
    );
    if roots.is_empty() {
        println!("No .ledgerful roots found.");
        return;
    }
    for row in roots {
        let registered_to = row.registered_project_id.as_deref().unwrap_or("—");
        let disk = if row.exists { "ok" } else { "missing" };
        println!(
            "{:<40} {:<36} {:<8} {}",
            row.path, registered_to, disk, row.suggested
        );
    }
}

fn dir_has_ledgerful(dir: &Path) -> bool {
    dir.join(".ledgerful").exists()
}

fn collect_scan_children(
    scan_root: &Path,
) -> Result<Vec<ScanChildResult>, Box<dyn std::error::Error>> {
    let entries = std::fs::read_dir(scan_root).map_err(|e| {
        format!(
            "scan-roots: cannot read directory '{}': {e}",
            scan_root.display()
        )
    })?;

    let mut children = Vec::new();
    for (idx, entry) in entries.enumerate() {
        match entry {
            Ok(ent) => {
                let path = ent.path();
                let (readable, is_dir) = match std::fs::metadata(&path) {
                    Ok(meta) => (true, meta.is_dir()),
                    Err(_) => (false, false),
                };
                let has_ledgerful = readable && is_dir && dir_has_ledgerful(&path);
                children.push(ScanChildResult {
                    path,
                    readable,
                    is_dir,
                    has_ledgerful,
                });
            }
            Err(_) => {
                children.push(ScanChildResult {
                    path: scan_root.join(format!("<unreadable-{idx}>")),
                    readable: false,
                    is_dir: false,
                    has_ledgerful: false,
                });
            }
        }
    }
    Ok(children)
}

// ---------------------------------------------------------------------------
// unregister-path (F2 / F13–F17 / F35 / F37)
// ---------------------------------------------------------------------------

/// Unregister a filesystem path alias (compensating Removed event).
///
/// Missing path is idempotent exit 0. Optional `--project` must match the
/// current owner or the command returns `Err` (exit 1). Does not forget symbols.
pub fn unregister_path(
    ctx: &AppContext,
    path: &str,
    project_ref: Option<&str>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let normalized = ai_brains_path::normalize_for_location_compare(path);
    if normalized.is_empty() {
        return crate::commands::governed_common::fail_usage(
            "path normalized to empty; choose a non-empty filesystem path.",
        );
    }

    let owner = ctx.conn.find_path_alias_owner(&normalized)?;
    let Some(owner) = owner else {
        println!("Path alias '{normalized}' is not registered.");
        return Ok(());
    };

    if let Some(project_ref) = project_ref {
        let wanted = resolve_project_ref(ctx, project_ref)?;
        if wanted != owner {
            return Err(format!(
                "path alias '{normalized}' owner {owner} and --project {wanted} do not match"
            )
            .into());
        }
    }

    if dry_run {
        println!("would unregister path alias '{normalized}' from project {owner}");
        return Ok(());
    }

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    let writer = ai_brains_control_plane::StoreEventWriter::new(event_store);
    ai_brains_control_plane::unregister_path_alias(&writer, path, owner)
        .map_err(|e| format!("unregister path alias failed: {e}"))?;

    println!("Path alias '{normalized}' unregistered from project {owner}.");
    Ok(())
}

/// Resolve `project_ref` as UUID parse **or** human alias lookup.
pub(crate) fn resolve_project_ref(
    ctx: &AppContext,
    project_ref: &str,
) -> Result<ai_brains_core::ids::ProjectId, Box<dyn std::error::Error>> {
    use std::str::FromStr;

    if let Ok(pid) = ai_brains_core::ids::ProjectId::from_str(project_ref) {
        let projects = ctx.conn.list_projects()?;
        let id_str = pid.to_string();
        if projects.iter().any(|(p, _, _, _)| p == &id_str) {
            return Ok(pid);
        }
        return Err(format!("Project '{project_ref}' not found in vault.").into());
    }

    if let Some(pid) = ctx.conn.resolve_project_id_from_alias(project_ref)? {
        return Ok(pid);
    }

    Err(format!(
        "Project '{project_ref}' not found (not a valid project UUID and not a known alias)."
    )
    .into())
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    fn child(path: &str, readable: bool, is_dir: bool, has_ledgerful: bool) -> ScanChildResult {
        ScanChildResult {
            path: PathBuf::from(path),
            readable,
            is_dir,
            has_ledgerful,
        }
    }

    #[test]
    fn discover_scan_hits__over_cap__truncated_true() {
        let scan_root = Path::new(r"C:\scan");
        let children: Vec<ScanChildResult> = (0..201)
            .map(|i| child(&format!(r"C:\scan\child-{i:03}"), true, true, true))
            .collect();

        let discovery = discover_scan_hits(scan_root, true, children, SCAN_CHILD_CAP);
        assert!(
            discovery.truncated,
            "over-cap children must set truncated=true"
        );
        assert_eq!(
            discovery.hits.len(),
            SCAN_CHILD_CAP + 1,
            "hits == cap plus included root"
        );
        assert_eq!(discovery.unreadable_count, 0);
    }

    #[test]
    fn discover_scan_hits__unreadable_child__skipped_and_counted() {
        let scan_root = Path::new(r"C:\scan");
        let children = vec![
            child(r"C:\scan\hit", true, true, true),
            child(r"C:\scan\locked", false, true, true),
            child(r"C:\scan\plain", true, true, false),
            child(r"C:\scan\file.txt", true, false, false),
        ];

        let discovery = discover_scan_hits(scan_root, false, children, SCAN_CHILD_CAP);
        assert!(!discovery.truncated);
        assert_eq!(discovery.unreadable_count, 1);
        assert_eq!(discovery.hits.len(), 1);
        assert_eq!(discovery.hits[0], PathBuf::from(r"C:\scan\hit"));
    }
}
