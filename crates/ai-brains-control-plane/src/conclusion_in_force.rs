//! Resolve the in-force Active|Confirmed conclusion for a term (T323).
//!
//! Governed query over `conclusion_projection` only. No events, pins, FTS,
//! `--as-of`, or vault-wide `list_conclusions_by_scope_state(None, …)`.

use std::collections::HashSet;
use std::str::FromStr;

use serde::Serialize;
use time::format_description::well_known::Rfc3339;

use ai_brains_core::ids::ConclusionId;

use crate::briefings::project::conclusion_valid_at;
use crate::errors::{ControlPlaneError, Result};
use crate::ports::{Clock, ConclusionRow, GovernedQueryStore};

const MAX_SUPERSEDE_HOPS: usize = 32;
const RULING_STATE: &str = "in_force";

/// Frozen JSON object for `ai-brains conclusion in-force` (F4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConclusionInForceResponse {
    pub term: String,
    pub scope: String,
    pub ruling: Option<ConclusionInForceRuling>,
    pub chain: Vec<ConclusionInForceChainLink>,
}

/// Current ruling. `state` is always `"in_force"` when present (F9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConclusionInForceRuling {
    pub conclusion_id: String,
    pub statement: String,
    pub state: String,
    pub updated_at: String,
}

/// Predecessor on the supersession walk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConclusionInForceChainLink {
    pub conclusion_id: String,
    pub status: String,
}

/// Resolve the in-force ruling for `term` inside `scope_key` at **now** (T323).
///
/// Empty/whitespace `term` is [`ControlPlaneError::InvalidPayload`]. Cycle in
/// `superseded_by` is [`ControlPlaneError::InvalidTransition`].
pub fn resolve_conclusion_in_force<Q, C>(
    query: &Q,
    clock: &C,
    scope_key: &str,
    term: &str,
) -> Result<ConclusionInForceResponse>
where
    Q: GovernedQueryStore,
    C: Clock,
{
    let term = term.trim();
    if term.is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "term must be non-empty".into(),
        ));
    }

    let rows = query.list_conclusions_by_scope_state(Some(scope_key), None)?;
    let empty = empty_response(term, scope_key);
    let Some(root) = select_root(&rows, term) else {
        return Ok(empty);
    };

    let (current, chain) = walk_chain(query, root.clone(), scope_key)?;
    let at = clock.now()?;
    let ruling = ruling_now(&current, at)?;

    // Rejected root with no usable successor: keep chain empty (F7).
    let chain = if ruling.is_none() && current.id == root.id && current.state == "Rejected" {
        Vec::new()
    } else {
        chain
    };

    Ok(ConclusionInForceResponse {
        term: term.to_string(),
        scope: scope_key.to_string(),
        ruling,
        chain,
    })
}

fn ruling_now(
    current: &ConclusionRow,
    at: time::OffsetDateTime,
) -> Result<Option<ConclusionInForceRuling>> {
    let ok = matches!(current.state.as_str(), "Active" | "Confirmed")
        && conclusion_valid_at(current, at);
    if ok {
        Ok(Some(ruling_from_row(current)?))
    } else {
        Ok(None)
    }
}

fn empty_response(term: &str, scope_key: &str) -> ConclusionInForceResponse {
    ConclusionInForceResponse {
        term: term.to_string(),
        scope: scope_key.to_string(),
        ruling: None,
        chain: Vec::new(),
    }
}

fn select_root<'a>(rows: &'a [ConclusionRow], term: &str) -> Option<&'a ConclusionRow> {
    let eligible: Vec<&ConclusionRow> = rows
        .iter()
        .filter(|r| {
            matches!(
                r.state.as_str(),
                "Active" | "Confirmed" | "Superseded" | "Rejected"
            )
        })
        .collect();
    let term_lower = term.to_ascii_lowercase();
    let mut matched: Vec<&ConclusionRow> = eligible
        .iter()
        .copied()
        .filter(|r| statement_term_prefix_exact(&r.statement, &term_lower))
        .collect();
    if matched.is_empty() {
        matched = eligible
            .iter()
            .copied()
            .filter(|r| r.statement.to_ascii_lowercase().contains(&term_lower))
            .collect();
    }
    matched.sort_by(|a, b| {
        a.valid_from
            .cmp(&b.valid_from)
            .then_with(|| a.id.to_string().cmp(&b.id.to_string()))
    });
    matched.into_iter().next()
}

fn statement_term_prefix_exact(statement: &str, term_lower: &str) -> bool {
    let lower = statement.trim().to_ascii_lowercase();
    match lower.strip_prefix("term:") {
        Some(rest) => rest.trim() == term_lower,
        None => false,
    }
}

fn walk_chain<Q: GovernedQueryStore>(
    query: &Q,
    mut current: ConclusionRow,
    scope_key: &str,
) -> Result<(ConclusionRow, Vec<ConclusionInForceChainLink>)> {
    let mut chain = Vec::new();
    let mut visited = HashSet::new();
    visited.insert(current.id.to_string());

    for _ in 0..MAX_SUPERSEDE_HOPS {
        let Some(next_raw) = current
            .superseded_by
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        else {
            break;
        };
        if !visited.insert(next_raw.to_string()) {
            return Err(ControlPlaneError::InvalidTransition(format!(
                "supersession cycle at {next_raw}"
            )));
        }
        let Ok(next_id) = ConclusionId::from_str(next_raw) else {
            break;
        };
        match query.get_conclusion(next_id)? {
            None => break,
            Some(next) => {
                if next.scope != scope_key {
                    break;
                }
                chain.push(ConclusionInForceChainLink {
                    conclusion_id: current.id.to_string(),
                    status: format!("superseded_by:{next_raw}"),
                });
                current = next;
            }
        }
    }
    Ok((current, chain))
}

fn ruling_from_row(row: &ConclusionRow) -> Result<ConclusionInForceRuling> {
    let updated_at = row
        .updated_at
        .format(&Rfc3339)
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
    Ok(ConclusionInForceRuling {
        conclusion_id: row.id.to_string(),
        statement: row.statement.clone(),
        state: RULING_STATE.to_string(),
        updated_at,
    })
}
