use crate::context::AppContext;
use crate::graph_density::{
    GatherResult, GraphDensitySnapshot, assess_graph_density, gather_density_snapshot,
};
use ai_brains_control_plane::clamp_list_limit;
use ai_brains_graph::{GraphRebuilder, GraphSearch, GraphVault, NeighborHit};
use ai_brains_retrieval::{PinKind, classify_pin_kind};
use ai_brains_store::{QueryStore, SqliteEventStore};
use rusqlite::OptionalExtension;
use serde::Serialize;
use std::io::IsTerminal;

const PRETTY_NEXT: &str = "next: ai-brains graph update";

#[derive(Serialize)]
struct NeighborsOutput<'a> {
    memory_id: &'a str,
    neighbors: Vec<NeighborHit>,
}

#[derive(Serialize)]
struct HierarchyOutput<'a> {
    root: &'a str,
    synthesized_from: Vec<String>,
}

#[derive(Serialize)]
struct SessionOutput<'a> {
    session_id: &'a str,
    memories: Vec<String>,
}

/// Graph health report (`graph update`). T213 expands density fields; keeps `note` (M3).
#[derive(Serialize)]
struct GraphHealthOutput {
    nodes: i64,
    edges: i64,
    pinned_memories: i64,
    memory_nodes: i64,
    edge_node_ratio: f64,
    /// `ok` | `warn` | `skip` (never `fail` on this field).
    density: &'static str,
    /// `live` | `sparse` | `empty`
    status: &'static str,
    note: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    remediation: Option<String>,
}

/// One pretty-neighbors row (kind/preview are pretty-only).
#[derive(Debug, Clone)]
pub(crate) struct PrettyNeighborRow {
    pub direction: String,
    pub label: String,
    pub external_id: String,
    pub kind: String,
    pub preview: String,
}

pub(crate) fn resolve_graph_format(explicit: &str, is_tty: bool) -> &'static str {
    match explicit {
        "pretty" | "human" | "text" => "pretty",
        "json" => "json",
        "auto" if is_tty => "pretty",
        "auto" => "json",
        // Clap value_parser rejects unknowns; fail-closed compact for scripts.
        _ => "json",
    }
}

/// Maps `memory_exists` onto F1. Never `?` this Result on graph reads (F18 / AC19).
pub(crate) fn vault_memory_present<E: std::fmt::Display>(result: Result<bool, E>) -> bool {
    match result {
        Ok(true) => true,
        Ok(false) => false,
        Err(err) => {
            tracing::warn!(error = %err, "memory_exists failed; treating id as unknown");
            false
        }
    }
}

/// Session missing-node: vault has the id if it is a memory **or** a session
/// (CX1-P2 — `memory_exists` alone lies for `graph session <session_id>`).
fn session_projection_present(ctx: &AppContext, session_id: &str) -> Result<bool, String> {
    let conn = ctx
        .conn
        .lock()
        .map_err(|e| format!("Failed to lock vault: {e}"))?;
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM session_projection WHERE session_id = ?1",
            [session_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("session_projection lookup failed: {e}"))?;
    Ok(count > 0)
}

fn vault_has_session_or_memory(ctx: &AppContext, id: &str) -> bool {
    vault_memory_present(ctx.conn.memory_exists(id))
        || vault_memory_present(session_projection_present(ctx, id))
}

pub(crate) fn pretty_no_graph_node(id: &str, vault_has_memory: bool) -> String {
    if vault_has_memory {
        format!("No graph node for {id}.\nnext: ai-brains graph rebuild")
    } else {
        format!("No graph node for {id} (not a vault memory id).")
    }
}

pub(crate) fn pretty_no_memory_node(id: &str) -> String {
    format!("No memory node for {id}.\n{PRETTY_NEXT}")
}

pub(crate) fn pretty_no_session_node(id: &str) -> String {
    format!("No session node for {id}.\n{PRETTY_NEXT}")
}

pub(crate) fn pretty_no_neighbors(id: &str) -> String {
    format!("No neighbors for {id}.")
}

pub(crate) fn pretty_hierarchy_leaf() -> String {
    "No SYNTHESIZED_FROM children (leaf).\nnext: ai-brains nightly --status".to_string()
}

/// T317 F1: human-only RECALLS display cap (after T293 prefer-authority).
pub(crate) const RECALLS_PRETTY_CAP: usize = 3;

/// Keep all non-`RECALLS` rows; keep the first `RECALLS_PRETTY_CAP` `RECALLS` in order.
/// Returns `(kept, recalls_hidden)`. Label match is exact `"RECALLS"`.
pub(crate) fn cap_recalls_pretty_rows(
    rows: &[PrettyNeighborRow],
) -> (Vec<PrettyNeighborRow>, usize) {
    let mut kept = Vec::with_capacity(rows.len());
    let mut recalls_kept = 0usize;
    let mut recalls_hidden = 0usize;
    for row in rows {
        if row.label == "RECALLS" {
            if recalls_kept < RECALLS_PRETTY_CAP {
                kept.push(row.clone());
                recalls_kept += 1;
            } else {
                recalls_hidden += 1;
            }
        } else {
            kept.push(row.clone());
        }
    }
    (kept, recalls_hidden)
}

pub(crate) fn pretty_session_empty() -> String {
    "No memories in this session via graph edges.".to_string()
}

fn dir_display(direction: &str) -> &str {
    match direction {
        "incoming" => "in",
        "outgoing" => "out",
        other => other,
    }
}

pub(crate) fn sort_neighbor_hits(hits: &mut [NeighborHit]) {
    hits.sort_by(|a, b| {
        a.direction
            .cmp(&b.direction)
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.external_id.cmp(&b.external_id))
    });
}

fn apply_limit(len: usize, limit: usize) -> (usize, Option<usize>) {
    if len > limit {
        (limit, Some(len - limit))
    } else {
        (len, None)
    }
}

pub(crate) fn format_neighbors_pretty(
    memory_id: &str,
    rows: &[PrettyNeighborRow],
    limit: usize,
    full_hop_count: usize,
    recalls_hidden: usize,
) -> String {
    let (shown, more) = apply_limit(rows.len(), limit);
    let mut out = format!("Neighbors of {memory_id} ({full_hop_count})\n");
    out.push_str(&format!(
        "{:<3} {:<16} {:<36} {:<14} {}\n",
        "DIR", "LABEL", "ID", "KIND", "PREVIEW"
    ));
    for row in rows.iter().take(shown) {
        out.push_str(&format!(
            "{:<3} {:<16} {:<36} {:<14} {}\n",
            dir_display(&row.direction),
            row.label,
            row.external_id,
            row.kind,
            row.preview
        ));
    }
    if let Some(n) = more {
        out.push_str(&format!("… and {n} more\n"));
    }
    if recalls_hidden > 0 {
        out.push_str(&format!("+{recalls_hidden} more RECALLS\n"));
    }
    out
}

pub(crate) fn format_neighbors_json(
    memory_id: &str,
    neighbors: &[NeighborHit],
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&NeighborsOutput {
        memory_id,
        neighbors: neighbors.to_vec(),
    })
}

pub(crate) fn format_hierarchy_pretty(root: &str, rows: &[(String, i64)], limit: usize) -> String {
    let total = rows.len();
    let (shown, more) = apply_limit(total, limit);
    let mut out = format!("Hierarchy of {root} ({total})\n");
    for (id, depth) in rows.iter().take(shown) {
        let indent = " ".repeat(2 * (*depth).max(0) as usize);
        out.push_str(&format!("{indent}{id}\n"));
    }
    if let Some(n) = more {
        out.push_str(&format!("… and {n} more\n"));
    }
    out
}

pub(crate) fn format_session_pretty(
    session_id: &str,
    rows: &[(String, String)],
    limit: usize,
) -> String {
    let total = rows.len();
    let (shown, more) = apply_limit(total, limit);
    let mut out = format!("Memories in session {session_id} ({total})\n");
    for (id, preview) in rows.iter().take(shown) {
        out.push_str(&format!("{id} {preview}\n"));
    }
    if let Some(n) = more {
        out.push_str(&format!("… and {n} more\n"));
    }
    out
}

fn json_take(limit: Option<usize>) -> usize {
    if limit.is_some() {
        clamp_list_limit(limit)
    } else {
        usize::MAX
    }
}

fn memory_preview(ctx: &AppContext, memory_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let conn = ctx
        .conn
        .lock()
        .map_err(|e| format!("Failed to lock vault: {e}"))?;
    let content: Option<String> = conn
        .query_row(
            "SELECT content FROM memory_projection WHERE memory_id = ?1 LIMIT 1",
            [memory_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Failed to load memory preview: {e}"))?;
    Ok(content.map_or_else(String::new, |c| {
        crate::commands::memory::preview_line(&c, 80)
    }))
}

/// T278 F2: `{n} memories` plus optional ` · first` (trim-empty first → no dot), cap 80.
pub(crate) fn format_session_neighbor_preview(n: usize, first_preview: &str) -> String {
    let mut caption = format!("{n} memories");
    if !first_preview.trim().is_empty() {
        caption.push_str(" · ");
        caption.push_str(first_preview);
    }
    crate::commands::display_text::truncate_preview_chars(&caption, 80)
}

/// T278 F34: skip items whose trim is empty; return the first remaining as-is.
pub(crate) fn pick_first_nonempty(previews: &[String]) -> Option<String> {
    previews
        .iter()
        .find(|preview| !preview.trim().is_empty())
        .cloned()
}

/// T278 F33: session PREVIEW I/O. Never `Result` — fail-open to `"0 memories"` / `"{n} memories"`.
fn session_neighbor_caption(
    ctx: &AppContext,
    searcher: &GraphSearch<'_>,
    session_id: &str,
) -> String {
    let mut ids = match searcher.get_session_memories(session_id) {
        Ok(ids) => ids,
        Err(err) => {
            tracing::warn!(
                error = %err,
                session_id,
                "session neighbor caption: get_session_memories failed"
            );
            return format_session_neighbor_preview(0, "");
        }
    };
    ids.sort();
    let n = ids.len();
    let mut previews = Vec::with_capacity(n);
    for id in &ids {
        match memory_preview(ctx, id) {
            Ok(preview) => previews.push(preview),
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    memory_id = %id,
                    session_id,
                    "session neighbor caption: memory_preview failed"
                );
                previews.push(String::new());
            }
        }
    }
    let first = pick_first_nonempty(&previews).unwrap_or_default();
    format_session_neighbor_preview(n, &first)
}

fn pretty_neighbor_rows(
    ctx: &AppContext,
    searcher: &GraphSearch<'_>,
    hits: &[NeighborHit],
) -> Result<Vec<PrettyNeighborRow>, Box<dyn std::error::Error>> {
    let mut rows = Vec::with_capacity(hits.len());
    for hit in hits {
        let kind = searcher.node_kind(&hit.external_id)?.unwrap_or_default();
        let preview = if kind == "memory" {
            memory_preview(ctx, &hit.external_id)?
        } else if kind == "session" {
            session_neighbor_caption(ctx, searcher, &hit.external_id)
        } else {
            String::new()
        };
        rows.push(PrettyNeighborRow {
            direction: hit.direction.clone(),
            label: hit.label.clone(),
            external_id: hit.external_id.clone(),
            kind,
            preview,
        });
    }
    Ok(rows)
}

/// T293 F4: strip T278 `{n} memories · ` once (exact space-dot-space). Dots in the
/// remainder stay. No separator / empty remainder → `""` (Other).
pub(crate) fn session_caption_body(preview: &str) -> &str {
    match preview.split_once(" · ") {
        Some((_, rest)) => rest.trim(),
        None => "",
    }
}

/// T293 F1 ranks: 0 memory authority, 1 session authority, 2 other memory, 3 other.
pub(crate) fn neighbor_authority_rank(row: &PrettyNeighborRow) -> u8 {
    let authority = if row.kind == "memory" {
        classify_pin_kind(&row.preview) != PinKind::Other
    } else if row.kind == "session" {
        let body = session_caption_body(&row.preview);
        !body.is_empty() && classify_pin_kind(body) != PinKind::Other
    } else {
        false
    };
    match (row.kind.as_str(), authority) {
        ("memory", true) => 0,
        ("session", true) => 1,
        ("memory", false) => 2,
        _ => 3,
    }
}

/// T293 F1: stable prefer-authority reorder. Same length; no drops. Within-tier
/// order stays the F9 direction→label→id order via `(rank, original_index)`.
pub(crate) fn prefer_authority_neighbor_rows(rows: &mut [PrettyNeighborRow]) {
    let mut indexed: Vec<(usize, PrettyNeighborRow)> = rows.iter().cloned().enumerate().collect();
    indexed.sort_by_key(|(idx, row)| (neighbor_authority_rank(row), *idx));
    for (dst, (_, row)) in rows.iter_mut().zip(indexed) {
        *dst = row;
    }
}

fn emit_graph_health_human(report: &GraphHealthOutput) {
    println!("status: {}", report.status);
    println!("density: {}", report.density);
    println!("nodes: {}", report.nodes);
    println!("edges: {}", report.edges);
    println!("pinned_memories: {}", report.pinned_memories);
    println!("memory_nodes: {}", report.memory_nodes);
    println!("edge_node_ratio: {}", report.edge_node_ratio);
    println!("note: {}", report.note);
    if let Some(remediation) = &report.remediation {
        println!("remediation: {remediation}");
    }
}

fn emit_graph_health(
    report: &GraphHealthOutput,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if format == "human" {
        emit_graph_health_human(report);
    } else {
        crate::commands::identity_warn::print_json_stdout(report)?;
    }
    Ok(())
}

/// Shared health builder for `graph update` and `graph rebuild` (T300 F27).
fn graph_health_report(ctx: &AppContext) -> Result<GraphHealthOutput, Box<dyn std::error::Error>> {
    let conn = ctx
        .conn
        .lock()
        .map_err(|e| format!("Failed to lock vault: {}", e))?;

    let gather = gather_density_snapshot(&conn)
        .map_err(|e| format!("Failed to gather graph density: {e}"))?;
    graph_health_from_gather(gather)
}

fn graph_health_from_gather(
    gather: GatherResult,
) -> Result<GraphHealthOutput, Box<dyn std::error::Error>> {
    let (snap, pinned_for_json, memory_for_json) = match gather {
        GatherResult::TablesMissing => {
            return Err(
                "Failed to count graph nodes/edges: graph tables missing (run migrate / init)"
                    .into(),
            );
        }
        GatherResult::PinnedCountFailed {
            nodes,
            edges,
            memory_nodes,
        } => {
            // T326 red: still invents pinned=0 then assesses (same as glance).
            let s = GraphDensitySnapshot {
                nodes,
                edges,
                pinned_memories: 0,
                memory_nodes,
            };
            let mem_json = memory_nodes.unwrap_or(0);
            (s, 0_i64, mem_json)
        }
        GatherResult::Ok(s) => {
            let mem_json = s.memory_nodes.unwrap_or(0);
            let pinned = s.pinned_memories;
            (s, pinned, mem_json)
        }
    };

    let assessment = assess_graph_density(&snap);

    Ok(GraphHealthOutput {
        nodes: snap.nodes,
        edges: snap.edges,
        pinned_memories: pinned_for_json,
        memory_nodes: memory_for_json,
        edge_node_ratio: assessment.edge_node_ratio,
        density: assessment.density,
        status: assessment.status,
        note: assessment.note,
        remediation: assessment.remediation,
    })
}

/// T188 substring classes for mutating rebuild blocked by a live daemon (T300 F7/F26).
pub(crate) fn rebuild_daemon_busy_message() -> String {
    "Cannot rebuild graph: daemon is running and holds the vault open. \
     Stop it first with `ai-brains daemon stop`, or if installed as a Windows \
     service: `sc stop AI-Brains-Daemon` (service hosts `ai-brainsd`)."
        .to_string()
}

const REBUILD_DRY_RUN_DAEMON_NOTICE: &str = "NOTICE: live rebuild will fail while the daemon is running. \
     Stop with `ai-brains daemon stop` or `sc stop AI-Brains-Daemon` before a real rebuild.";

fn rebuild_dry_run_line(event_count: Option<i64>) -> String {
    match event_count {
        Some(n) => format!(
            "[dry-run] would DELETE graph_node/graph_edge then replay {n} events; no mutation."
        ),
        None => "[dry-run] would DELETE graph_node/graph_edge then replay events; no mutation."
            .to_string(),
    }
}

fn count_events_fail_open(ctx: &AppContext) -> Option<i64> {
    let conn = ctx.conn.lock().ok()?;
    conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .ok()
}

/// Injectable rebuild core (T300 F26). Production probes then calls this.
pub(crate) fn rebuild_with_daemon_state(
    ctx: &AppContext,
    dry_run: bool,
    format: &str,
    daemon_up: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dry_run && daemon_up {
        return Err(rebuild_daemon_busy_message().into());
    }

    if dry_run {
        let report = graph_health_report(ctx)?;
        emit_graph_health(&report, format)?;
        if format == "human" {
            if daemon_up {
                println!("{REBUILD_DRY_RUN_DAEMON_NOTICE}");
            }
            let n = count_events_fail_open(ctx);
            println!("{}", rebuild_dry_run_line(n));
        }
        return Ok(());
    }

    tracing::info!("[graph] Starting graph rebuild...");
    let event_store = SqliteEventStore::new((*ctx.conn).clone());
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let rebuilder = GraphRebuilder::new(&graph_vault, &event_store);
    rebuilder.rebuild()?;
    tracing::info!("[graph] Rebuild complete.");

    let report = graph_health_report(ctx)?;
    emit_graph_health(&report, format)?;
    Ok(())
}

pub async fn rebuild(
    ctx: &AppContext,
    dry_run: bool,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = crate::daemon_client::DaemonClient::new();
    let daemon_up = crate::commands::backup::probe_restore_daemon_busy(&client).await;
    rebuild_with_daemon_state(ctx, dry_run, format, daemon_up)
}

pub fn neighbors(
    ctx: &AppContext,
    memory_id: &str,
    format: &str,
    limit: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let resolved = resolve_graph_format(format, std::io::stdout().is_terminal());
    if resolved != "pretty" {
        crate::commands::identity_warn::note_machine_stdout();
    }
    let kind = searcher.node_kind(memory_id)?;

    if kind.is_none() {
        if resolved == "pretty" {
            let present = vault_memory_present(ctx.conn.memory_exists(memory_id));
            println!("{}", pretty_no_graph_node(memory_id, present));
        } else {
            println!("{}", format_neighbors_json(memory_id, &[])?);
        }
        return Ok(());
    }

    let mut hits = searcher.get_neighbors(memory_id)?;
    sort_neighbor_hits(&mut hits);

    if resolved == "pretty" {
        if hits.is_empty() {
            println!("{}", pretty_no_neighbors(memory_id));
            return Ok(());
        }
        let mut rows = pretty_neighbor_rows(ctx, &searcher, &hits)?;
        prefer_authority_neighbor_rows(&mut rows);
        let full_hop_count = rows.len();
        let (kept, recalls_hidden) = cap_recalls_pretty_rows(&rows);
        print!(
            "{}",
            format_neighbors_pretty(
                memory_id,
                &kept,
                clamp_list_limit(limit),
                full_hop_count,
                recalls_hidden,
            )
        );
        return Ok(());
    }

    let take = json_take(limit);
    if hits.len() > take {
        hits.truncate(take);
    }
    println!("{}", format_neighbors_json(memory_id, &hits)?);
    Ok(())
}

pub fn hierarchy(
    ctx: &AppContext,
    memory_id: &str,
    format: &str,
    limit: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let resolved = resolve_graph_format(format, std::io::stdout().is_terminal());
    if resolved != "pretty" {
        crate::commands::identity_warn::note_machine_stdout();
    }
    let kind = searcher.node_kind(memory_id)?;

    match kind.as_deref() {
        None => {
            if resolved == "pretty" {
                let present = vault_memory_present(ctx.conn.memory_exists(memory_id));
                println!("{}", pretty_no_graph_node(memory_id, present));
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&HierarchyOutput {
                        root: memory_id,
                        synthesized_from: Vec::new(),
                    })?
                );
            }
            return Ok(());
        }
        Some("memory") => {}
        Some(_) => {
            if resolved == "pretty" {
                println!("{}", pretty_no_memory_node(memory_id));
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&HierarchyOutput {
                        root: memory_id,
                        synthesized_from: Vec::new(),
                    })?
                );
            }
            return Ok(());
        }
    }

    if resolved == "pretty" {
        let rows = searcher.get_synthesized_hierarchy_with_depth(memory_id)?;
        if rows.is_empty() {
            println!("{}", pretty_hierarchy_leaf());
            return Ok(());
        }
        print!(
            "{}",
            format_hierarchy_pretty(memory_id, &rows, clamp_list_limit(limit))
        );
        return Ok(());
    }

    let mut synthesized = searcher.get_synthesized_hierarchy(memory_id)?;
    synthesized.sort();
    let take = json_take(limit);
    if synthesized.len() > take {
        synthesized.truncate(take);
    }
    println!(
        "{}",
        serde_json::to_string(&HierarchyOutput {
            root: memory_id,
            synthesized_from: synthesized,
        })?
    );
    Ok(())
}

pub fn session(
    ctx: &AppContext,
    session_id: &str,
    format: &str,
    limit: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let resolved = resolve_graph_format(format, std::io::stdout().is_terminal());
    if resolved != "pretty" {
        crate::commands::identity_warn::note_machine_stdout();
    }
    let kind = searcher.node_kind(session_id)?;

    match kind.as_deref() {
        None => {
            if resolved == "pretty" {
                let present = vault_has_session_or_memory(ctx, session_id);
                println!("{}", pretty_no_graph_node(session_id, present));
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&SessionOutput {
                        session_id,
                        memories: Vec::new(),
                    })?
                );
            }
            return Ok(());
        }
        Some("session") => {}
        Some(_) => {
            if resolved == "pretty" {
                println!("{}", pretty_no_session_node(session_id));
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&SessionOutput {
                        session_id,
                        memories: Vec::new(),
                    })?
                );
            }
            return Ok(());
        }
    }

    let mut memories = searcher.get_session_memories(session_id)?;
    memories.sort();

    if resolved == "pretty" {
        if memories.is_empty() {
            println!("{}", pretty_session_empty());
            return Ok(());
        }
        let mut rows = Vec::with_capacity(memories.len());
        for id in &memories {
            let preview = memory_preview(ctx, id)?;
            rows.push((id.clone(), preview));
        }
        print!(
            "{}",
            format_session_pretty(session_id, &rows, clamp_list_limit(limit))
        );
        return Ok(());
    }

    let take = json_take(limit);
    if memories.len() > take {
        memories.truncate(take);
    }
    println!(
        "{}",
        serde_json::to_string(&SessionOutput {
            session_id,
            memories,
        })?
    );
    Ok(())
}

pub fn update(ctx: &AppContext, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let report = graph_health_report(ctx)?;
    emit_graph_health(&report, format)
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::graph_density::{GraphDensitySnapshot, assess_graph_density_with};
    use rstest::rstest;

    #[test]
    fn graph_health_from_gather__pinned_count_failed__err() {
        use crate::graph_density::PINNED_COUNT_FAILED_MSG;

        let r = graph_health_from_gather(GatherResult::PinnedCountFailed {
            nodes: 0,
            edges: 0,
            memory_nodes: Some(0),
        });
        match r {
            Err(e) => {
                let s = e.to_string();
                assert!(
                    s.contains("cannot assess empty_lag without pins"),
                    "Err display must contain skip body; got {s}"
                );
                assert!(
                    s.contains(PINNED_COUNT_FAILED_MSG) || s.contains("cannot assess empty_lag"),
                    "got {s}"
                );
            }
            Ok(out) => panic!(
                "must not fake pinned_memories={} status={}",
                out.pinned_memories, out.status
            ),
        }
    }

    /// T213 AC8: success JSON shape includes expanded density fields + note.
    #[test]
    fn graph_health_output__serde_keys__include_density_fields() {
        let snap = GraphDensitySnapshot {
            nodes: 10,
            edges: 5,
            pinned_memories: 5,
            memory_nodes: Some(5),
        };
        let a = assess_graph_density_with(&snap, true);
        assert_eq!(a.status, "live");
        assert_eq!(a.density, "ok");

        let report = GraphHealthOutput {
            nodes: snap.nodes,
            edges: snap.edges,
            pinned_memories: snap.pinned_memories,
            memory_nodes: snap.memory_nodes.unwrap_or(0),
            edge_node_ratio: a.edge_node_ratio,
            density: a.density,
            status: a.status,
            note: a.note,
            remediation: a.remediation,
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
                .expect("json");
        for key in [
            "nodes",
            "edges",
            "pinned_memories",
            "memory_nodes",
            "edge_node_ratio",
            "density",
            "status",
            "note",
        ] {
            assert!(v.get(key).is_some(), "missing key {key} in {v}");
        }
        assert!(v.get("remediation").is_none(), "ok path omits remediation");
        assert!(matches!(
            v["status"].as_str(),
            Some("live" | "sparse" | "empty")
        ));
        assert!(matches!(
            v["density"].as_str(),
            Some("ok" | "warn" | "skip")
        ));
    }

    /// T213 AC9 / T308: sparse fixture maps status/density; remediator key omitted (no rebuild loop).
    #[test]
    fn graph_health_output__sparse_fixture__status_sparse_omits_remediation() {
        let snap = GraphDensitySnapshot {
            nodes: 1304,
            edges: 95,
            pinned_memories: 8398,
            memory_nodes: Some(500),
        };
        let a = assess_graph_density_with(&snap, true);
        assert!(
            a.remediation.is_none(),
            "sparse graph-on remediator must be None"
        );
        let report = GraphHealthOutput {
            nodes: snap.nodes,
            edges: snap.edges,
            pinned_memories: snap.pinned_memories,
            memory_nodes: snap.memory_nodes.unwrap_or(0),
            edge_node_ratio: a.edge_node_ratio,
            density: a.density,
            status: a.status,
            note: a.note,
            remediation: a.remediation,
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
                .expect("json");
        assert_eq!(v["status"], "sparse");
        assert_eq!(v["density"], "warn");
        assert!(
            v.get("remediation").is_none(),
            "sparse graph-on JSON must omit remediation key: {}",
            v
        );
        assert!(!v["note"].as_str().unwrap_or("").is_empty());
        assert!(v.get("nodes").is_some());
        assert!(v.get("edges").is_some());
        assert!(v.get("pinned_memories").is_some());
        assert!(v.get("memory_nodes").is_some());
        assert!(v.get("edge_node_ratio").is_some());
        assert!(v.get("note").is_some());
    }

    #[test]
    fn resolve_graph_format__auto_tty__pretty() {
        assert_eq!(resolve_graph_format("auto", true), "pretty");
    }

    #[test]
    fn resolve_graph_format__auto_pipe__json() {
        assert_eq!(resolve_graph_format("auto", false), "json");
    }

    #[test]
    fn resolve_graph_format__pretty_human_text__pretty_regardless_of_tty() {
        for token in ["pretty", "human", "text"] {
            assert_eq!(resolve_graph_format(token, true), "pretty", "{token} tty");
            assert_eq!(resolve_graph_format(token, false), "pretty", "{token} pipe");
        }
    }

    #[test]
    fn resolve_graph_format__json__json_regardless_of_tty() {
        assert_eq!(resolve_graph_format("json", true), "json");
        assert_eq!(resolve_graph_format("json", false), "json");
    }

    /// T300 AC10: busy message carries T188 substring classes (rstest cases).
    #[rstest]
    #[case::daemon_running("daemon is running")]
    #[case::daemon_stop("ai-brains daemon stop")]
    #[case::service_stop("sc stop")]
    fn rebuild_daemon_busy_message__contains_t188_substring(#[case] needle: &str) {
        let msg = rebuild_daemon_busy_message();
        assert!(msg.contains(needle), "missing {needle}: {msg}");
        assert!(
            msg.contains("AI-Brains-Daemon"),
            "missing service name: {msg}"
        );
    }

    #[test]
    fn rebuild_dry_run_line__with_count__includes_n() {
        let line = rebuild_dry_run_line(Some(42));
        assert!(line.contains("[dry-run]"), "{line}");
        assert!(line.contains("42"), "{line}");
        assert!(line.contains("no mutation"), "{line}");
    }

    #[test]
    fn rebuild_dry_run_daemon_notice__contains_stop_guidance() {
        assert!(REBUILD_DRY_RUN_DAEMON_NOTICE.contains("daemon"));
        assert!(REBUILD_DRY_RUN_DAEMON_NOTICE.contains("ai-brains daemon stop"));
        assert!(REBUILD_DRY_RUN_DAEMON_NOTICE.contains("sc stop AI-Brains-Daemon"));
    }

    const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";

    fn count_graph_nodes(ctx: &AppContext) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = ctx
            .conn
            .lock()
            .map_err(|e| format!("Failed to lock vault: {e}"))?;
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM graph_node", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count graph_node: {e}"))?;
        Ok(n)
    }

    fn make_ctx(vault: std::path::PathBuf) -> (AppContext, ai_brains_core::temp_env::TempEnv) {
        use ai_brains_crypto::SqlCipherKey;
        use ai_brains_store::connection::VaultConnection;
        use std::sync::Arc;

        let allow = ai_brains_core::temp_env::TempEnv::set(
            ai_brains_store::connection::ALLOW_ZERO_KEY_ENV,
            "1",
        );
        let key = SqlCipherKey::from_raw(ZERO_KEY.to_string());
        let conn = VaultConnection::open(&vault, &key).expect("open vault");
        conn.migrate().expect("migrate");
        (
            AppContext {
                vault_path: vault,
                _key: key,
                conn: Arc::new(conn),
            },
            allow,
        )
    }

    fn seed_pin(ctx: &AppContext) -> String {
        use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
        use ai_brains_core::privacy::Privacy;
        use ai_brains_events::constructors::EventBuilder;
        use ai_brains_events::{
            Actor, AggregateType, MemoryPinnedPayload, Payload, ProjectRegisteredPayload,
            SessionStartedPayload,
        };
        use ai_brains_store::EventStore;

        let store = SqliteEventStore::new((*ctx.conn).clone());
        let project_id = ProjectId::new();
        let session_id = SessionId::new();
        let memory_id = MemoryId::new();

        let project_env = EventBuilder::new(
            AggregateType::Project,
            project_id.as_uuid(),
            Actor::System,
            Privacy::CloudOk,
        )
        .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
            project_id,
            name: "t300-rebuild".to_string(),
            tx_id: None,
        }))
        .expect("project event");
        store.append_event(&project_env).expect("append project");

        let session_env = EventBuilder::new(
            AggregateType::Session,
            session_id.as_uuid(),
            Actor::System,
            Privacy::CloudOk,
        )
        .build(Payload::SessionStarted(SessionStartedPayload {
            session_id,
            project_id,
            tx_id: None,
        }))
        .expect("session event");
        store.append_event(&session_env).expect("append session");

        let pin_env = EventBuilder::new(
            AggregateType::Memory,
            memory_id.as_uuid(),
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::MemoryPinned(MemoryPinnedPayload {
            memory_id,
            content: "DECISION: T300 rebuild keeps pin node.".to_string(),
            session_id: Some(session_id),
            project_id: Some(project_id),
            tx_id: None,
            rank: Some(1),
            source_tag: Some("pin".to_string()),
            query_text: None,
        }))
        .expect("pin event");
        store.append_event(&pin_env).expect("append pin");

        let graph_vault = GraphVault::new((*ctx.conn).clone());
        let rebuilder = GraphRebuilder::new(&graph_vault, &store);
        rebuilder.rebuild().expect("initial project");
        memory_id.to_string()
    }

    /// T300 AC3: daemon-up mutate fail-closes before DELETE.
    #[test]
    fn rebuild_with_daemon_state__daemon_up_mutate__err() {
        let dir = tempfile::tempdir().unwrap();
        let (ctx, _allow) = make_ctx(dir.path().join("vault.db"));
        let memory_id = seed_pin(&ctx);
        let before = count_graph_nodes(&ctx).expect("count before");
        assert!(before > 0, "seed must create nodes");

        let err = rebuild_with_daemon_state(&ctx, false, "human", true)
            .expect_err("daemon-up mutate must Err");
        let msg = err.to_string();
        assert!(msg.contains("daemon is running"), "{msg}");
        assert!(msg.contains("ai-brains daemon stop"), "{msg}");
        assert!(
            msg.contains("sc stop") && msg.contains("AI-Brains-Daemon"),
            "{msg}"
        );
        let after = count_graph_nodes(&ctx).expect("count after");
        assert_eq!(before, after, "must not DELETE when daemon up");

        let graph_vault = GraphVault::new((*ctx.conn).clone());
        let searcher = GraphSearch::new(&graph_vault);
        let hits = searcher.get_neighbors(&memory_id).expect("neighbors");
        assert!(
            hits.iter().any(|h| h.label == "RECALLS"),
            "pin RECALLS must survive blocked rebuild; got {hits:?}"
        );
    }

    /// T300 AC4: daemon-up dry-run Ok (NOTICE + dry-run lines are human helpers).
    #[test]
    fn rebuild_with_daemon_state__daemon_up_dry_run__ok() {
        let dir = tempfile::tempdir().unwrap();
        let (ctx, _allow) = make_ctx(dir.path().join("vault.db"));
        let _memory_id = seed_pin(&ctx);
        let before = count_graph_nodes(&ctx).expect("count before");

        rebuild_with_daemon_state(&ctx, true, "human", true).expect("dry-run ok");
        let after = count_graph_nodes(&ctx).expect("count after");
        assert_eq!(before, after, "dry-run must not DELETE");
        assert!(REBUILD_DRY_RUN_DAEMON_NOTICE.contains("daemon"));
        assert!(rebuild_dry_run_line(Some(1)).contains("[dry-run]"));
    }

    /// T300 AC2 / AC10 case 3: daemon-down mutate emits health; pin RECALLS stay; status matches update.
    #[test]
    fn rebuild_with_daemon_state__daemon_down_mutate__prints_density_keeps_pin() {
        let dir = tempfile::tempdir().unwrap();
        let (ctx, _allow) = make_ctx(dir.path().join("vault.db"));
        let memory_id = seed_pin(&ctx);

        rebuild_with_daemon_state(&ctx, false, "human", false).expect("mutate ok");
        let after = graph_health_report(&ctx).expect("health after");
        assert!(
            matches!(after.status, "live" | "sparse" | "empty"),
            "status={}",
            after.status
        );
        assert!(after.nodes >= 0);
        assert!(after.edges >= 0);

        let update_report = graph_health_report(&ctx).expect("update health");
        assert_eq!(
            after.status, update_report.status,
            "rebuild health must match update builder"
        );

        let graph_vault = GraphVault::new((*ctx.conn).clone());
        let searcher = GraphSearch::new(&graph_vault);
        let hits = searcher.get_neighbors(&memory_id).expect("neighbors");
        assert!(
            hits.iter()
                .any(|h| h.direction == "incoming" && h.label == "RECALLS"),
            "T262 RECALLS must remain after rebuild; got {hits:?}"
        );
    }

    /// T300 AC5: JSON mutate health keys frozen (no next_step / events_replayed).
    #[test]
    fn rebuild_with_daemon_state__format_json__health_keys() {
        let dir = tempfile::tempdir().unwrap();
        let (ctx, _allow) = make_ctx(dir.path().join("vault.db"));
        let _memory_id = seed_pin(&ctx);

        rebuild_with_daemon_state(&ctx, false, "json", false).expect("json mutate");
        let report = graph_health_report(&ctx).expect("report");
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&report).expect("ser")).expect("parse");
        for key in [
            "nodes",
            "edges",
            "pinned_memories",
            "memory_nodes",
            "edge_node_ratio",
            "density",
            "status",
            "note",
        ] {
            assert!(v.get(key).is_some(), "missing {key} in {v}");
        }
        assert!(v.get("next_step").is_none(), "no next_step: {v}");
        assert!(
            v.get("events_replayed").is_none(),
            "no events_replayed: {v}"
        );
        assert!(matches!(
            v["status"].as_str(),
            Some("live" | "sparse" | "empty")
        ));
        assert!(matches!(
            v["density"].as_str(),
            Some("ok" | "warn" | "skip")
        ));
    }

    fn fixture_neighbor_rows() -> Vec<PrettyNeighborRow> {
        vec![
            PrettyNeighborRow {
                direction: "incoming".into(),
                label: "RECALLS".into(),
                external_id: "3b4e95b8-a011-48a8-b5ea-72e36c6a2458".into(),
                kind: "session".into(),
                preview: "2 memories · pin text".into(),
            },
            PrettyNeighborRow {
                direction: "outgoing".into(),
                label: "SYNTHESIZED_FROM".into(),
                external_id: "5a0e0a71-1ee7-445b-84a9-aa06fe499c2e".into(),
                kind: "memory".into(),
                preview: "child preview".into(),
            },
        ]
    }

    #[test]
    fn format_neighbors_pretty__incoming_and_outgoing__header_in_out_kinds() {
        let text = format_neighbors_pretty("root-id", &fixture_neighbor_rows(), 50, 2, 0);
        assert!(text.starts_with("Neighbors of root-id (2)"));
        assert!(text.contains("DIR"));
        assert!(text.contains("LABEL"));
        assert!(text.contains("KIND"));
        assert!(text.contains("PREVIEW"));
        assert!(text.contains("in "));
        assert!(text.contains("out"));
        assert!(text.contains("RECALLS"));
        assert!(text.contains("SYNTHESIZED_FROM"));
        assert!(text.contains("3b4e95b8-a011-48a8-b5ea-72e36c6a2458"));
        assert!(text.contains("5a0e0a71-1ee7-445b-84a9-aa06fe499c2e"));
        assert!(text.contains("session"));
        assert!(text.contains("memory"));
        assert!(!text.contains("incoming"));
        assert!(!text.contains("outgoing"));
    }

    #[test]
    fn format_neighbors_json__fixture__keeps_incoming_outgoing_external_id() {
        let hits = vec![
            NeighborHit {
                external_id: "3b4e95b8-a011-48a8-b5ea-72e36c6a2458".into(),
                label: "RECALLS".into(),
                direction: "incoming".into(),
            },
            NeighborHit {
                external_id: "5a0e0a71-1ee7-445b-84a9-aa06fe499c2e".into(),
                label: "SYNTHESIZED_FROM".into(),
                direction: "outgoing".into(),
            },
        ];
        let raw = format_neighbors_json("root-id", &hits).expect("json");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(v["memory_id"], "root-id");
        let obj_keys: Vec<&str> = v
            .as_object()
            .expect("obj")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(obj_keys, vec!["memory_id", "neighbors"]);
        let n0 = &v["neighbors"][0];
        assert_eq!(n0["direction"], "incoming");
        assert_eq!(n0["external_id"], "3b4e95b8-a011-48a8-b5ea-72e36c6a2458");
        assert_eq!(n0["label"], "RECALLS");
        let n1 = &v["neighbors"][1];
        assert_eq!(n1["direction"], "outgoing");
        let hit_keys: std::collections::BTreeSet<&str> = n0
            .as_object()
            .expect("hit")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            hit_keys,
            std::collections::BTreeSet::from(["external_id", "label", "direction"])
        );
    }

    #[test]
    fn format_session_neighbor_preview__zero_and_blank__zero_memories_no_dot() {
        assert_eq!(format_session_neighbor_preview(0, ""), "0 memories");
        let ws = format_session_neighbor_preview(2, "   ");
        assert_eq!(ws, "2 memories");
        assert!(!ws.contains(" · "));
    }

    #[test]
    fn format_session_neighbor_preview__count_and_first__dot_and_cap_80() {
        let one = format_session_neighbor_preview(1, "preview");
        assert!(one.contains("1 memories"), "got {one:?}");
        assert!(one.contains("preview"), "got {one:?}");
        assert!(one.contains(" · "), "got {one:?}");
        let three = format_session_neighbor_preview(3, "hello");
        assert!(three.contains("3 memories"), "got {three:?}");
        assert!(three.contains("hello"), "got {three:?}");
        assert!(three.contains(" · "), "got {three:?}");
        let long = "a".repeat(200);
        let capped = format_session_neighbor_preview(1, &long);
        assert!(
            capped.chars().count() <= 80,
            "ASCII cap chars={} got {capped:?}",
            capped.chars().count()
        );
        assert!(capped.ends_with('…'), "got {capped:?}");
        let cjk = "日本語テストプレビュー境界値チェック用の長い行です".repeat(8);
        let cjk_out = format_session_neighbor_preview(1, &cjk);
        assert!(
            cjk_out.chars().count() <= 80,
            "CJK cap chars={} got {cjk_out:?}",
            cjk_out.chars().count()
        );
        assert!(cjk_out.ends_with('…'), "got {cjk_out:?}");
    }

    #[test]
    fn format_neighbors_pretty__session_recalls__preview_shows_memories() {
        let mut rows = fixture_neighbor_rows();
        rows[0].preview = format_session_neighbor_preview(2, "pin text");
        let text = format_neighbors_pretty("root-id", &rows, 50, 2, 0);
        assert!(text.contains("DIR"));
        assert!(text.contains("in "));
        assert!(text.contains("RECALLS"));
        assert!(text.contains("session"));
        assert!(
            text.contains("2 memories"),
            "session PREVIEW must show count; got: {text}"
        );
        let raw = format_neighbors_json(
            "root-id",
            &[NeighborHit {
                external_id: "3b4e95b8-a011-48a8-b5ea-72e36c6a2458".into(),
                label: "RECALLS".into(),
                direction: "incoming".into(),
            }],
        )
        .expect("json");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(
            v.as_object()
                .expect("obj")
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["memory_id", "neighbors"]
        );
        let hit_keys: std::collections::BTreeSet<&str> = v["neighbors"][0]
            .as_object()
            .expect("hit")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            hit_keys,
            std::collections::BTreeSet::from(["external_id", "label", "direction"])
        );
        assert_eq!(v["neighbors"][0]["direction"], "incoming");
        assert_eq!(
            v["neighbors"][0]["external_id"],
            "3b4e95b8-a011-48a8-b5ea-72e36c6a2458"
        );
    }

    #[test]
    fn pick_first_nonempty__blank_then_hello__some_hello() {
        let blank = [String::new(), "  ".into(), "hello".into()];
        let pick = pick_first_nonempty(&blank);
        assert_eq!(pick.as_deref(), Some("hello"));
        assert_eq!(
            pick_first_nonempty(&[String::from("pin")]).as_deref(),
            Some("pin")
        );
        assert_eq!(pick_first_nonempty(&["".into(), "   ".into()]), None);
        assert_eq!(pick_first_nonempty(&[]), None);
        let caption = format_session_neighbor_preview(3, pick.as_deref().unwrap_or(""));
        assert!(caption.contains("3 memories"), "got {caption:?}");
        assert!(caption.contains("hello"), "got {caption:?}");
        assert!(caption.contains(" · "), "got {caption:?}");
    }

    #[test]
    fn pretty_no_graph_node__vault_memory__next_rebuild__ac8() {
        let text = pretty_no_graph_node("abc", true);
        assert_eq!(
            text,
            "No graph node for abc.\nnext: ai-brains graph rebuild"
        );
        assert!(text.contains("graph rebuild"));
        assert!(!text.contains("graph update"));
    }

    #[test]
    fn pretty_no_graph_node__unknown_id__no_rebuild__ac8() {
        let text = pretty_no_graph_node("abc", false);
        assert_eq!(text, "No graph node for abc (not a vault memory id).");
        assert!(text.contains("not a vault memory id"));
        assert!(!text.contains("rebuild"));
        assert!(!text.contains("update"));
        assert!(!text.contains("next:"));
    }

    #[test]
    fn vault_memory_present__query_err__false_unknown_graph_copy__ac19() {
        assert!(!vault_memory_present(Err("locked")));
        let text = pretty_no_graph_node("abc", vault_memory_present(Err("locked")));
        assert!(text.contains("not a vault memory id"));
        assert!(!text.contains("rebuild"));
        assert!(!text.contains("update"));
    }

    #[test]
    fn pretty_hierarchy_leaf__no_graph_update_or_rebuild__ac9() {
        assert!(!pretty_hierarchy_leaf().contains("graph update"));
        assert!(!pretty_hierarchy_leaf().contains("graph rebuild"));
        assert!(!pretty_no_neighbors("abc").contains("graph update"));
        assert!(!pretty_no_neighbors("abc").contains("graph rebuild"));
        assert!(!pretty_session_empty().contains("graph update"));
        assert!(!pretty_session_empty().contains("graph rebuild"));
    }

    #[test]
    fn empty_pretty__json_keys_frozen__ac10() {
        let neighbors = format_neighbors_json("abc", &[]).expect("json");
        let v: serde_json::Value = serde_json::from_str(&neighbors).expect("parse");
        let keys: Vec<&str> = v
            .as_object()
            .expect("obj")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, vec!["memory_id", "neighbors"]);
        assert_eq!(v["neighbors"], serde_json::json!([]));

        let hierarchy = serde_json::to_string(&HierarchyOutput {
            root: "abc",
            synthesized_from: Vec::new(),
        })
        .expect("hier");
        let h: serde_json::Value = serde_json::from_str(&hierarchy).expect("parse hier");
        let h_keys: Vec<&str> = h
            .as_object()
            .expect("obj")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(h_keys, vec!["root", "synthesized_from"]);
        assert_eq!(h["synthesized_from"], serde_json::json!([]));
    }

    #[test]
    fn format_hierarchy_pretty__depth_1_and_2__indent_2_and_4_no_box() {
        let rows = vec![("child-a".into(), 1_i64), ("child-b".into(), 2_i64)];
        let text = format_hierarchy_pretty("root", &rows, 50);
        assert!(text.contains("  child-a"));
        assert!(text.contains("    child-b"));
        assert!(!text.contains('└'));
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[1], "  child-a");
        assert_eq!(lines[2], "    child-b");
    }

    #[test]
    fn format_neighbors_pretty__51_rows_default_limit__50_lines_and_1_more() {
        let rows: Vec<PrettyNeighborRow> = (0..51)
            .map(|i| PrettyNeighborRow {
                direction: "incoming".into(),
                label: "RECALLS".into(),
                external_id: format!("id-{i:02}"),
                kind: "session".into(),
                preview: String::new(),
            })
            .collect();
        let limit = clamp_list_limit(None);
        let text = format_neighbors_pretty("root", &rows, limit, 51, 0);
        let data_lines: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("in ") || l.starts_with("out"))
            .collect();
        assert_eq!(data_lines.len(), 50);
        assert!(text.contains("… and 1 more"));
        assert!(!text.contains("more RECALLS"));
    }

    /// T317 AC1: 11 RECALLS → 3 kept, hidden 8.
    #[test]
    fn cap_recalls_pretty_rows__eleven_recalls__keeps_three_hidden_eight() {
        let rows: Vec<PrettyNeighborRow> = (0..11)
            .map(|i| pretty_row("session", &format!("s-{i:02}"), "1 memories · ## Objective"))
            .collect();
        let (kept, hidden) = cap_recalls_pretty_rows(&rows);
        assert_eq!(kept.len(), 3, "kept={kept:?}");
        assert_eq!(hidden, 8);
        assert!(kept.iter().all(|r| r.label == "RECALLS"));
        assert_eq!(kept[0].external_id, "s-00");
        assert_eq!(kept[2].external_id, "s-02");
    }

    /// T317 AC2: at-or-under cap unchanged; 4 → keep 3 hide 1.
    #[rstest::rstest]
    #[case::zero(0usize, 0usize, 0usize)]
    #[case::one(1, 1, 0)]
    #[case::three(3, 3, 0)]
    #[case::four(4, 3, 1)]
    fn cap_recalls_pretty_rows__at_or_under_cap__unchanged(
        #[case] n: usize,
        #[case] expect_kept: usize,
        #[case] expect_hidden: usize,
    ) {
        let rows: Vec<PrettyNeighborRow> = (0..n)
            .map(|i| pretty_row("session", &format!("s-{i}"), "1 memories · dump"))
            .collect();
        let (kept, hidden) = cap_recalls_pretty_rows(&rows);
        assert_eq!(kept.len(), expect_kept);
        assert_eq!(hidden, expect_hidden);
        assert_eq!(kept.len() + hidden, n);
    }

    /// T317 AC3: mixed labels keep all non-RECALLS + 3 RECALLS.
    #[test]
    fn cap_recalls_pretty_rows__mixed_labels__keeps_all_non_recalls() {
        let mut rows = vec![
            PrettyNeighborRow {
                direction: "outgoing".into(),
                label: "SYNTHESIZED_FROM".into(),
                external_id: "synth-a".into(),
                kind: "memory".into(),
                preview: "a".into(),
            },
            PrettyNeighborRow {
                direction: "outgoing".into(),
                label: "SYNTHESIZED_FROM".into(),
                external_id: "synth-b".into(),
                kind: "memory".into(),
                preview: "b".into(),
            },
        ];
        for i in 0..5 {
            rows.push(pretty_row(
                "session",
                &format!("r-{i}"),
                "1 memories · ## Objective",
            ));
        }
        let (kept, hidden) = cap_recalls_pretty_rows(&rows);
        assert_eq!(kept.len(), 5, "2 non-RECALLS + 3 RECALLS; kept={kept:?}");
        assert_eq!(hidden, 2);
        assert!(kept.iter().any(|r| r.external_id == "synth-a"));
        assert!(kept.iter().any(|r| r.external_id == "synth-b"));
        let recalls: Vec<&str> = kept
            .iter()
            .filter(|r| r.label == "RECALLS")
            .map(|r| r.external_id.as_str())
            .collect();
        assert_eq!(recalls, vec!["r-0", "r-1", "r-2"]);
    }

    /// T317 AC4: after prefer, authority RECALLS is among the 3 kept.
    #[test]
    fn cap_recalls_pretty_rows__authority_recalls__kept_before_dumps() {
        let mut rows = vec![
            pretty_row("session", "dump-0", "1 memories · ## Objective"),
            pretty_row("session", "dump-1", "1 memories · ## Objective"),
            pretty_row("session", "dump-2", "1 memories · ## Objective"),
            pretty_row("session", "dump-3", "1 memories · ## Objective"),
            pretty_row("session", "auth-s", "1 memories · DECISION: keep me"),
        ];
        prefer_authority_neighbor_rows(&mut rows);
        assert_eq!(rows[0].external_id, "auth-s");
        let (kept, hidden) = cap_recalls_pretty_rows(&rows);
        assert_eq!(kept.len(), 3);
        assert_eq!(hidden, 2);
        assert_eq!(kept[0].external_id, "auth-s");
        assert!(kept.iter().any(|r| r.external_id == "auth-s"));
    }

    /// T317 AC5: header uses full hop count; 3 data rows; RECALLS footer.
    #[test]
    fn format_neighbors_pretty__recalls_hidden__header_total_and_footer() {
        let rows: Vec<PrettyNeighborRow> = (0..3)
            .map(|i| pretty_row("session", &format!("s-{i}"), "1 memories · dump"))
            .collect();
        let text = format_neighbors_pretty("root", &rows, 50, 11, 8);
        assert!(
            text.starts_with("Neighbors of root (11)"),
            "header must use full_hop_count; got: {text}"
        );
        let data_lines: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("in ") || l.starts_with("out"))
            .collect();
        assert_eq!(data_lines.len(), 3);
        assert!(
            text.contains("+8 more RECALLS"),
            "missing RECALLS footer; got: {text}"
        );
        assert!(!text.contains("+8 more RECALLS."));
    }

    /// T317 AC6: recalls_hidden=0 → no RECALLS footer; header uses full_hop_count.
    #[test]
    fn format_neighbors_pretty__no_hidden__no_recalls_footer() {
        let rows = fixture_neighbor_rows();
        let text = format_neighbors_pretty("root-id", &rows, 50, 2, 0);
        assert!(text.starts_with("Neighbors of root-id (2)"));
        assert!(!text.contains("more RECALLS"));
    }

    /// T317 AC7: leaf two lines; nightly --status next-step.
    #[test]
    fn pretty_hierarchy_leaf__nightly_status_next() {
        let text = pretty_hierarchy_leaf();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "expected two lines; got: {text:?}");
        assert_eq!(lines[0], "No SYNTHESIZED_FROM children (leaf).");
        assert_eq!(lines[1], "next: ai-brains nightly --status");
    }

    /// T317 AC17: --limit footer then RECALLS footer.
    #[test]
    fn format_neighbors_pretty__limit_and_recalls_hidden__two_footers() {
        let rows: Vec<PrettyNeighborRow> = (0..4)
            .map(|i| pretty_row("session", &format!("s-{i}"), "1 memories · dump"))
            .collect();
        let text = format_neighbors_pretty("root", &rows, 2, 10, 7);
        let limit_pos = text.find("… and 2 more").unwrap_or_else(|| {
            panic!("limit footer missing: {text}");
        });
        let recalls_pos = text.find("+7 more RECALLS").unwrap_or_else(|| {
            panic!("RECALLS footer missing: {text}");
        });
        assert!(
            limit_pos < recalls_pos,
            "limit line must precede RECALLS footer; got: {text}"
        );
    }

    #[test]
    fn sort_neighbor_hits__incoming_before_outgoing_then_label_then_id() {
        let mut hits = vec![
            NeighborHit {
                external_id: "z".into(),
                label: "B".into(),
                direction: "outgoing".into(),
            },
            NeighborHit {
                external_id: "b".into(),
                label: "A".into(),
                direction: "incoming".into(),
            },
            NeighborHit {
                external_id: "a".into(),
                label: "B".into(),
                direction: "incoming".into(),
            },
            NeighborHit {
                external_id: "a".into(),
                label: "A".into(),
                direction: "incoming".into(),
            },
        ];
        sort_neighbor_hits(&mut hits);
        let keys: Vec<(&str, &str, &str)> = hits
            .iter()
            .map(|h| {
                (
                    h.direction.as_str(),
                    h.label.as_str(),
                    h.external_id.as_str(),
                )
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                ("incoming", "A", "a"),
                ("incoming", "A", "b"),
                ("incoming", "B", "a"),
                ("outgoing", "B", "z"),
            ]
        );
    }

    fn pretty_row(kind: &str, id: &str, preview: &str) -> PrettyNeighborRow {
        PrettyNeighborRow {
            direction: "incoming".into(),
            label: "RECALLS".into(),
            external_id: id.into(),
            kind: kind.into(),
            preview: preview.into(),
        }
    }

    /// T293 AC1: dump session then Decision memory → memory first; dump still present.
    #[test]
    fn prefer_authority_neighbor_rows__dump_then_decision_memory__memory_first() {
        let mut rows = vec![
            pretty_row("session", "dump-session", "1 memories · ## Objective"),
            pretty_row("memory", "auth-memory", "DECISION: keep authority first"),
        ];
        prefer_authority_neighbor_rows(&mut rows);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].external_id, "auth-memory");
        assert_eq!(rows[1].external_id, "dump-session");
    }

    /// T293 AC13: exact ` · ` strip; dots in remainder stay; no Decision on prefix.
    #[test]
    fn session_caption_body__memories_dot_decision__strips_prefix() {
        assert_eq!(
            session_caption_body("5 memories · DECISION: x"),
            "DECISION: x"
        );
        assert_eq!(session_caption_body("2 memories"), "");
        assert_eq!(session_caption_body("2 memories ·    "), "");
        assert_eq!(
            session_caption_body("1 memories · 1.2.3 dump"),
            "1.2.3 dump"
        );
        assert_eq!(
            classify_pin_kind(session_caption_body("5 memories · DECISION: x")),
            PinKind::Decision
        );
        assert_eq!(
            classify_pin_kind(session_caption_body("2 memories")),
            PinKind::Other
        );
    }

    /// T293 AC2: rank helper cases including four-tier mixed exact order.
    #[rstest::rstest]
    #[case::overlap_stable(
        vec![
            pretty_row("session", "dump-a", "1 memories · ## Objective"),
            pretty_row("session", "dump-b", "1 memories · # Review of Track"),
            pretty_row("memory", "auth-m", "DECISION: keep"),
        ],
        vec!["auth-m", "dump-a", "dump-b"]
    )]
    #[case::session_authority(
        vec![
            pretty_row("session", "dump", "1 memories · ## Objective"),
            pretty_row("session", "auth-s", "1 memories · DECISION: x"),
        ],
        vec!["auth-s", "dump"]
    )]
    #[case::chrome_only(
        vec![
            pretty_row("session", "a", "1 memories · ## Objective"),
            pretty_row("session", "b", "1 memories · ## Objective"),
        ],
        vec!["a", "b"]
    )]
    #[case::hotspot_memory(
        vec![
            pretty_row("session", "dump", "1 memories · ## Objective"),
            pretty_row("memory", "hot", "HOTSPOT: brittle"),
        ],
        vec!["hot", "dump"]
    )]
    #[case::invariant_session(
        vec![
            pretty_row("session", "dump", "1 memories · ## Objective"),
            pretty_row("session", "inv", "1 memories · INVARIANT: stay"),
        ],
        vec!["inv", "dump"]
    )]
    #[case::four_tier_mixed(
        vec![
            pretty_row("session", "other-s", "1 memories · ## Objective"),
            pretty_row("memory", "other-m", "plain note"),
            pretty_row("session", "auth-s", "1 memories · DECISION: s"),
            pretty_row("memory", "auth-m", "DECISION: m"),
        ],
        vec!["auth-m", "auth-s", "other-m", "other-s"]
    )]
    fn prefer_authority_neighbor_rows__cases__expected_order(
        #[case] mut rows: Vec<PrettyNeighborRow>,
        #[case] expected_ids: Vec<&str>,
    ) {
        prefer_authority_neighbor_rows(&mut rows);
        let ids: Vec<&str> = rows.iter().map(|r| r.external_id.as_str()).collect();
        assert_eq!(ids, expected_ids);
    }
}
