//! Resolve the in-force Approved decision for a term (T311).
//!
//! Governed query over `decision_projection` only. No events, pins, FTS, or
//! vault-wide `list_decisions(None)`.

use std::collections::HashSet;
use std::str::FromStr;

use serde::Serialize;
use time::format_description::well_known::Rfc3339;

use ai_brains_core::ids::DecisionId;

use crate::briefings::project::decision_valid_at;
use crate::errors::{ControlPlaneError, Result};
use crate::ports::{Clock, DecisionRow, GovernedQueryStore};

const MAX_SUPERSEDE_HOPS: usize = 32;
const RULING_STATE: &str = "in_force";

/// Frozen JSON object for `ai-brains decision in-force` (F4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InForceResponse {
    pub term: String,
    pub scope: String,
    pub ruling: Option<InForceRuling>,
    pub chain: Vec<InForceChainLink>,
}

/// Current ruling. `state` is always `"in_force"` when present (F9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InForceRuling {
    pub decision_id: String,
    pub title: String,
    pub statement: String,
    pub state: String,
    pub approver: String,
    pub updated_at: String,
}

/// Predecessor on the supersession walk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InForceChainLink {
    pub decision_id: String,
    pub status: String,
}

/// Resolve the in-force ruling for `term` inside `scope_key`.
///
/// Empty/whitespace `term` is [`ControlPlaneError::InvalidPayload`]. Cycle in
/// `superseded_by` is [`ControlPlaneError::InvalidTransition`].
pub fn resolve_in_force<Q, C>(
    query: &Q,
    clock: &C,
    scope_key: &str,
    term: &str,
) -> Result<InForceResponse>
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

    let rows = query.list_decisions(Some(scope_key), None)?;
    let empty = empty_response(term, scope_key);
    let Some(root) = select_root(&rows, term) else {
        return Ok(empty);
    };

    let (current, chain) = walk_chain(query, root.clone(), scope_key)?;
    let now = clock.now()?;
    let ruling = if current.state == "Approved" && decision_valid_at(&current, now) {
        Some(ruling_from_row(&current)?)
    } else {
        None
    };

    // Revoked root with no usable successor: keep chain empty (F7).
    let chain = if ruling.is_none() && current.id == root.id && current.state == "Revoked" {
        Vec::new()
    } else {
        chain
    };

    Ok(InForceResponse {
        term: term.to_string(),
        scope: scope_key.to_string(),
        ruling,
        chain,
    })
}

fn empty_response(term: &str, scope_key: &str) -> InForceResponse {
    InForceResponse {
        term: term.to_string(),
        scope: scope_key.to_string(),
        ruling: None,
        chain: Vec::new(),
    }
}

fn select_root<'a>(rows: &'a [DecisionRow], term: &str) -> Option<&'a DecisionRow> {
    let eligible: Vec<&DecisionRow> = rows
        .iter()
        .filter(|r| matches!(r.state.as_str(), "Approved" | "Superseded" | "Revoked"))
        .collect();
    let term_lower = term.to_ascii_lowercase();
    let mut matched: Vec<&DecisionRow> = eligible
        .iter()
        .copied()
        .filter(|r| title_term_prefix_exact(&r.title, &term_lower))
        .collect();
    if matched.is_empty() {
        matched = eligible
            .iter()
            .copied()
            .filter(|r| r.title.to_ascii_lowercase().contains(&term_lower))
            .collect();
    }
    if matched.is_empty() {
        matched = eligible
            .iter()
            .copied()
            .filter(|r| r.statement.to_ascii_lowercase().contains(&term_lower))
            .collect();
    }
    matched.sort_by(|a, b| {
        a.recorded_at
            .cmp(&b.recorded_at)
            .then_with(|| a.id.to_string().cmp(&b.id.to_string()))
    });
    matched.into_iter().next()
}

fn title_term_prefix_exact(title: &str, term_lower: &str) -> bool {
    let lower = title.trim().to_ascii_lowercase();
    match lower.strip_prefix("term:") {
        Some(rest) => rest.trim() == term_lower,
        None => false,
    }
}

fn walk_chain<Q: GovernedQueryStore>(
    query: &Q,
    mut current: DecisionRow,
    scope_key: &str,
) -> Result<(DecisionRow, Vec<InForceChainLink>)> {
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
        let Ok(next_id) = DecisionId::from_str(next_raw) else {
            break;
        };
        match query.get_decision(next_id)? {
            None => break,
            Some(next) => {
                if next.scope != scope_key {
                    break;
                }
                chain.push(InForceChainLink {
                    decision_id: current.id.to_string(),
                    status: format!("superseded_by:{next_raw}"),
                });
                current = next;
            }
        }
    }
    Ok((current, chain))
}

fn ruling_from_row(row: &DecisionRow) -> Result<InForceRuling> {
    let updated_at = row
        .updated_at
        .format(&Rfc3339)
        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
    Ok(InForceRuling {
        decision_id: row.id.to_string(),
        title: row.title.clone(),
        statement: row.statement.clone(),
        state: RULING_STATE.to_string(),
        approver: row.approver.clone().unwrap_or_default(),
        updated_at,
    })
}
