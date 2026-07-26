//! Deterministic JSON + Markdown renderers for briefing packets (no LLM).

use ai_brains_contracts::briefings::{PersonalContinuityBriefingPacket, ProjectBriefingPacket};
use serde_json::Error as JsonError;

/// Serialize a project packet to JSON (pretty).
pub fn render_project_json(packet: &ProjectBriefingPacket) -> Result<String, JsonError> {
    serde_json::to_string_pretty(packet)
}

/// Serialize a personal packet to JSON (pretty).
pub fn render_personal_json(
    packet: &PersonalContinuityBriefingPacket,
) -> Result<String, JsonError> {
    serde_json::to_string_pretty(packet)
}

/// Deterministic Markdown render of a Project briefing packet.
pub fn render_project_markdown(packet: &ProjectBriefingPacket) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# Project Briefing".to_string());
    lines.push(String::new());
    lines.push(format!(
        "**Scope:** `{}` (confidence: {}, authoritative: {})",
        packet.scope.scope_key, packet.scope.confidence, packet.scope.authoritative
    ));
    if packet.denied {
        lines.push(String::new());
        lines.push(format!(
            "> **Denied:** {}",
            packet.denial_reason.as_deref().unwrap_or("policy denied")
        ));
    }
    if !packet.scope.warnings.is_empty() {
        lines.push(String::new());
        lines.push("## Scope warnings".to_string());
        for w in &packet.scope.warnings {
            lines.push(format!("- {w}"));
        }
    }

    if let Some(h) = &packet.handoff
        && !h.summary.is_empty()
    {
        lines.push(String::new());
        lines.push("## Handoff".to_string());
        lines.push(h.summary.clone());
    }

    lines.push(String::new());
    lines.push("## Decisions (current authority)".to_string());
    if packet.decisions.is_empty() {
        lines.push("_None_".to_string());
    } else {
        for d in &packet.decisions {
            let title = d.title.as_deref().unwrap_or("");
            if title.is_empty() {
                lines.push(format!("- **{}** [{}]: {}", d.id, d.state, d.statement));
            } else {
                lines.push(format!(
                    "- **{title}** (`{}`, {}) — {}",
                    d.id, d.state, d.statement
                ));
            }
            for h in &d.evidence_handles {
                lines.push(format!("  - evidence: `{}`", h.evidence_id));
            }
        }
    }

    lines.push(String::new());
    lines.push("## Conclusions (current authority)".to_string());
    if packet.conclusions.is_empty() {
        lines.push("_None_".to_string());
    } else {
        for c in &packet.conclusions {
            lines.push(format!("- **{}** [{}]: {}", c.id, c.state, c.statement));
            for h in &c.evidence_handles {
                lines.push(format!("  - evidence: `{}`", h.evidence_id));
            }
        }
    }

    if !packet.constraints.is_empty() {
        lines.push(String::new());
        lines.push("## Constraints".to_string());
        for k in &packet.constraints {
            lines.push(format!("- {}", k.statement));
        }
    }

    if !packet.warnings.is_empty() {
        lines.push(String::new());
        lines.push("## Warnings".to_string());
        for w in &packet.warnings {
            lines.push(format!("- **{}**: {}", w.kind, w.message));
        }
    }

    lines.push(String::new());
    lines.push("## Freshness".to_string());
    lines.push(format!(
        "sources={}, fresh={}, stale={}, unavailable={}, worst={}",
        packet.freshness.total_sources,
        packet.freshness.fresh_count,
        packet.freshness.stale_count,
        packet.freshness.unavailable_count,
        packet.freshness.worst_state
    ));

    if let Some(lf) = &packet.ledgerful {
        lines.push(String::new());
        lines.push("## Ledgerful".to_string());
        if lf.degraded {
            lines.push("_degraded / unavailable_".to_string());
        }
        for h in &lf.hotspots {
            lines.push(format!("- hotspot: {h}"));
        }
    }

    lines.push(String::new());
    lines.push(format!(
        "_Budget: used {} / max {} words; more_available={}_",
        packet.budget.used_words, packet.budget.max_words, packet.budget.more_available
    ));
    if !packet.budget.truncated_sections.is_empty() {
        lines.push(format!(
            "_Truncated sections: {}_",
            packet.budget.truncated_sections.join(", ")
        ));
    }

    lines.join("\n")
}

/// Deterministic Markdown render of a Personal continuity packet.
pub fn render_personal_markdown(packet: &PersonalContinuityBriefingPacket) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# Personal Continuity Briefing".to_string());
    lines.push(String::new());
    lines.push(format!("**Scope:** `{}`", packet.scope_key));
    if packet.denied {
        lines.push(format!(
            "> **Denied:** {}",
            packet.denial_reason.as_deref().unwrap_or("policy denied")
        ));
    }

    lines.push(String::new());
    lines.push("## Preferences".to_string());
    if packet.preferences.is_empty() {
        lines.push("_None_".to_string());
    } else {
        for p in &packet.preferences {
            lines.push(format!("- {}", p.statement));
        }
    }

    lines.push(String::new());
    lines.push("## Continuity".to_string());
    if packet.continuity.summary.is_empty() {
        lines.push("_None_".to_string());
    } else {
        lines.push(packet.continuity.summary.clone());
    }

    if !packet.open_review_items.is_empty() {
        lines.push(String::new());
        lines.push("## Open review items".to_string());
        for r in &packet.open_review_items {
            lines.push(format!(
                "- [{}] {} ({})",
                r.criticality, r.subject, r.status
            ));
        }
    }

    if !packet.grants_applied.is_empty() {
        lines.push(String::new());
        lines.push("## Grants applied".to_string());
        for g in &packet.grants_applied {
            lines.push(format!(
                "- `{}` {} / {} ({})",
                g.grant_id, g.capability, g.scope_key, g.privacy
            ));
        }
    }

    if !packet.warnings.is_empty() {
        lines.push(String::new());
        lines.push("## Warnings".to_string());
        for w in &packet.warnings {
            lines.push(format!("- **{}**: {}", w.kind, w.message));
        }
    }

    lines.join("\n")
}
