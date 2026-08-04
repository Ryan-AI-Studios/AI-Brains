//! Word-budget truncation for briefing packets.

use ai_brains_contracts::briefings::{
    BudgetReportDto, PersonalContinuityBriefingPacket, ProjectBriefingPacket,
};
use ai_brains_core::briefing::BriefingSection;

/// Caller budget for a briefing render/select pass.
#[derive(Debug, Clone, Copy)]
pub struct BudgetConfig {
    /// Maximum words across rendered sections (0 = unlimited).
    pub max_words: usize,
    /// Max current decisions to keep before truncation.
    pub max_decisions: usize,
    /// Max current conclusions to keep before truncation.
    pub max_conclusions: usize,
    /// Max constraints to keep.
    pub max_constraints: usize,
    /// Max warnings to keep.
    pub max_warnings: usize,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_words: 1500,
            max_decisions: 32,
            max_conclusions: 32,
            max_constraints: 16,
            max_warnings: 24,
        }
    }
}

fn approx_words(s: &str) -> usize {
    s.split_whitespace().count()
}

/// Truncate project packet lists to budget; mark dropped sections.
pub fn apply_budget(packet: &mut ProjectBriefingPacket, cfg: BudgetConfig) {
    let mut truncated: Vec<String> = Vec::new();

    if packet.decisions.len() > cfg.max_decisions {
        packet.decisions.truncate(cfg.max_decisions);
        truncated.push(BriefingSection::Decisions.as_str().to_string());
    }
    if packet.conclusions.len() > cfg.max_conclusions {
        packet.conclusions.truncate(cfg.max_conclusions);
        truncated.push(BriefingSection::Conclusions.as_str().to_string());
    }
    if packet.constraints.len() > cfg.max_constraints {
        packet.constraints.truncate(cfg.max_constraints);
        truncated.push(BriefingSection::Constraints.as_str().to_string());
    }
    if packet.warnings.len() > cfg.max_warnings {
        // Prefer keeping denied warnings when the packet is denied (T202 F7).
        if packet.denied {
            preserve_denied_warnings_cap(&mut packet.warnings, cfg.max_warnings);
        } else {
            packet.warnings.truncate(cfg.max_warnings);
        }
        truncated.push(BriefingSection::Warnings.as_str().to_string());
    }

    // Approximate word use from statements + warnings + handoff.
    let mut used = project_used_words(packet);

    let more = !truncated.is_empty() || (cfg.max_words > 0 && used > cfg.max_words);

    // If still over word budget, drop tail conclusions → decisions → constraints →
    // warnings → handoff until under max_words (T152-P2-02).
    if cfg.max_words > 0 && used > cfg.max_words {
        while used > cfg.max_words && !packet.conclusions.is_empty() {
            if let Some(c) = packet.conclusions.pop() {
                used = used.saturating_sub(approx_words(&c.statement));
                push_truncated_once(&mut truncated, BriefingSection::Conclusions.as_str());
            }
        }
        while used > cfg.max_words && !packet.decisions.is_empty() {
            if let Some(d) = packet.decisions.pop() {
                used = used.saturating_sub(approx_words(&d.statement));
                if let Some(t) = &d.title {
                    used = used.saturating_sub(approx_words(t));
                }
                push_truncated_once(&mut truncated, BriefingSection::Decisions.as_str());
            }
        }
        while used > cfg.max_words && !packet.constraints.is_empty() {
            if let Some(k) = packet.constraints.pop() {
                used = used.saturating_sub(approx_words(&k.statement));
                push_truncated_once(&mut truncated, BriefingSection::Constraints.as_str());
            }
        }
        while used > cfg.max_words && !packet.warnings.is_empty() {
            // Drop non-denied warnings first; never erase the last denied warning on denied packets (T202 F7).
            if let Some(idx) = packet.warnings.iter().rposition(|w| w.kind != "denied") {
                let w = packet.warnings.remove(idx);
                used = used.saturating_sub(approx_words(&w.message));
                push_truncated_once(&mut truncated, BriefingSection::Warnings.as_str());
            } else if packet.denied {
                break;
            } else if let Some(w) = packet.warnings.pop() {
                used = used.saturating_sub(approx_words(&w.message));
                push_truncated_once(&mut truncated, BriefingSection::Warnings.as_str());
            }
        }
        if used > cfg.max_words && packet.handoff.is_some() {
            packet.handoff = None;
            push_truncated_once(&mut truncated, BriefingSection::Handoff.as_str());
        }
        // Recompute in case of any drift; clamp to max so report never exceeds.
        used = project_used_words(packet);
        if used > cfg.max_words {
            used = cfg.max_words;
        }
    }

    ensure_denied_warning(packet);

    packet.budget = BudgetReportDto {
        max_words: cfg.max_words,
        used_words: used,
        truncated_sections: truncated,
        more_available: more || (cfg.max_words > 0 && project_used_words(packet) > cfg.max_words),
    };
}

/// Cap warnings while retaining every `kind=denied` entry when possible.
fn preserve_denied_warnings_cap(
    warnings: &mut Vec<ai_brains_contracts::briefings::BriefingWarningDto>,
    max: usize,
) {
    if warnings.len() <= max {
        return;
    }
    let denied: Vec<_> = warnings
        .iter()
        .filter(|w| w.kind == "denied")
        .cloned()
        .collect();
    let mut others: Vec<_> = warnings
        .iter()
        .filter(|w| w.kind != "denied")
        .cloned()
        .collect();
    // Always keep at least one denied if present; fill remaining slots with others then extra denied.
    let mut kept = Vec::new();
    if let Some(d) = denied.first() {
        kept.push(d.clone());
    }
    while kept.len() < max && !others.is_empty() {
        kept.push(others.remove(0));
    }
    let mut di = 1usize;
    while kept.len() < max && di < denied.len() {
        kept.push(denied[di].clone());
        di += 1;
    }
    // If max is 0, still keep one denied for denied packets (caller ensures denied=true).
    if max == 0
        && kept.is_empty()
        && let Some(d) = denied.into_iter().next()
    {
        kept.push(d);
    }
    *warnings = kept;
}

/// If packet is denied, ensure ≥1 `kind=denied` warning remains (T202 F7 / AC6).
fn ensure_denied_warning(packet: &mut ProjectBriefingPacket) {
    if !packet.denied {
        return;
    }
    if packet.warnings.iter().any(|w| w.kind == "denied") {
        return;
    }
    let reason = packet
        .denial_reason
        .clone()
        .unwrap_or_else(|| "policy denied".to_string());
    packet
        .warnings
        .push(ai_brains_contracts::briefings::BriefingWarningDto {
            kind: "denied".into(),
            message: reason,
            subject_id: None,
            subject_kind: None,
        });
}

fn project_used_words(packet: &ProjectBriefingPacket) -> usize {
    let mut used = 0usize;
    for d in &packet.decisions {
        used = used.saturating_add(approx_words(&d.statement));
        if let Some(t) = &d.title {
            used = used.saturating_add(approx_words(t));
        }
    }
    for c in &packet.conclusions {
        used = used.saturating_add(approx_words(&c.statement));
    }
    for w in &packet.warnings {
        used = used.saturating_add(approx_words(&w.message));
    }
    for k in &packet.constraints {
        used = used.saturating_add(approx_words(&k.statement));
    }
    if let Some(h) = &packet.handoff {
        used = used.saturating_add(approx_words(&h.summary));
    }
    used
}

fn push_truncated_once(truncated: &mut Vec<String>, section: &str) {
    if !truncated.iter().any(|s| s == section) {
        truncated.push(section.to_string());
    }
}

/// Truncate personal packet lists to budget.
pub fn apply_personal_budget(packet: &mut PersonalContinuityBriefingPacket, cfg: BudgetConfig) {
    let mut truncated: Vec<String> = Vec::new();
    if packet.preferences.len() > cfg.max_constraints {
        packet.preferences.truncate(cfg.max_constraints);
        truncated.push(BriefingSection::Preferences.as_str().to_string());
    }
    if packet.open_review_items.len() > cfg.max_warnings {
        packet.open_review_items.truncate(cfg.max_warnings);
        truncated.push(BriefingSection::OpenReviewItems.as_str().to_string());
    }
    let mut used = personal_used_words(packet);
    // T152-P2 / FRESH: drop preferences → open_review_items → continuity until
    // under max_words (renderer emits all three).
    if cfg.max_words > 0 && used > cfg.max_words {
        while used > cfg.max_words && !packet.preferences.is_empty() {
            if let Some(p) = packet.preferences.pop() {
                used = used.saturating_sub(approx_words(&p.statement));
                push_truncated_once(&mut truncated, BriefingSection::Preferences.as_str());
            }
        }
        while used > cfg.max_words && !packet.open_review_items.is_empty() {
            if let Some(item) = packet.open_review_items.pop() {
                used = used.saturating_sub(approx_words(&item.subject));
                used = used.saturating_sub(approx_words(&item.criticality));
                used = used.saturating_sub(approx_words(&item.status));
                push_truncated_once(&mut truncated, BriefingSection::OpenReviewItems.as_str());
            }
        }
        if used > cfg.max_words && !packet.continuity.summary.is_empty() {
            packet.continuity.summary.clear();
            push_truncated_once(&mut truncated, BriefingSection::Continuity.as_str());
        }
        while used > cfg.max_words && !packet.warnings.is_empty() {
            if let Some(idx) = packet.warnings.iter().rposition(|w| w.kind != "denied") {
                let w = packet.warnings.remove(idx);
                used = used.saturating_sub(approx_words(&w.message));
                push_truncated_once(&mut truncated, BriefingSection::Warnings.as_str());
            } else if packet.denied {
                break;
            } else if let Some(w) = packet.warnings.pop() {
                used = used.saturating_sub(approx_words(&w.message));
                push_truncated_once(&mut truncated, BriefingSection::Warnings.as_str());
            }
        }
        used = personal_used_words(packet);
        if used > cfg.max_words {
            used = cfg.max_words;
        }
    }
    ensure_personal_denied_warning(packet);
    let more =
        !truncated.is_empty() || (cfg.max_words > 0 && personal_used_words(packet) > cfg.max_words);
    packet.budget = BudgetReportDto {
        max_words: cfg.max_words,
        used_words: used,
        truncated_sections: truncated,
        more_available: more,
    };
}

fn personal_used_words(packet: &PersonalContinuityBriefingPacket) -> usize {
    let mut used = approx_words(&packet.continuity.summary);
    for p in &packet.preferences {
        used = used.saturating_add(approx_words(&p.statement));
    }
    for item in &packet.open_review_items {
        used = used.saturating_add(approx_words(&item.subject));
        used = used.saturating_add(approx_words(&item.criticality));
        used = used.saturating_add(approx_words(&item.status));
    }
    for w in &packet.warnings {
        used = used.saturating_add(approx_words(&w.message));
    }
    used
}

/// If personal packet is denied, ensure ≥1 `kind=denied` warning remains (T202 F7 / AC6).
fn ensure_personal_denied_warning(packet: &mut PersonalContinuityBriefingPacket) {
    if !packet.denied {
        return;
    }
    if packet.warnings.iter().any(|w| w.kind == "denied") {
        return;
    }
    let reason = packet
        .denial_reason
        .clone()
        .unwrap_or_else(|| "policy denied".to_string());
    packet
        .warnings
        .push(ai_brains_contracts::briefings::BriefingWarningDto {
            kind: "denied".into(),
            message: reason,
            subject_id: None,
            subject_kind: None,
        });
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_contracts::briefings::{
        BriefingScopeDto, BriefingWarningDto, PersonalContinuityBriefingPacket,
        ProjectBriefingPacket,
    };

    #[test]
    fn apply_budget__denied_packet_max_words_1__keeps_kind_denied() {
        let mut packet = ProjectBriefingPacket::empty_denied(
            "b1".into(),
            BriefingScopeDto {
                scope_key: "Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
                confidence: "High".into(),
                warnings: vec![],
                alternatives: vec![],
                authoritative: true,
            },
            "grant denied for test",
        );
        // Extra noise warnings that would consume budget if dropped last.
        packet.warnings.push(BriefingWarningDto {
            kind: "other".into(),
            message: "a ".repeat(50),
            subject_id: None,
            subject_kind: None,
        });
        apply_budget(
            &mut packet,
            BudgetConfig {
                max_words: 1,
                ..BudgetConfig::default()
            },
        );
        assert!(packet.denied);
        assert!(
            packet.warnings.iter().any(|w| w.kind == "denied"),
            "T202: budget must not erase kind=denied; got {:?}",
            packet.warnings
        );
    }

    #[test]
    fn apply_personal_budget__denied_packet_max_words_1__keeps_kind_denied() {
        let mut packet = PersonalContinuityBriefingPacket::empty_denied(
            "p1".into(),
            "Personal:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "personal grant denied",
        );
        packet.warnings.push(BriefingWarningDto {
            kind: "other".into(),
            message: "noise ".repeat(40),
            subject_id: None,
            subject_kind: None,
        });
        apply_personal_budget(
            &mut packet,
            BudgetConfig {
                max_words: 1,
                ..BudgetConfig::default()
            },
        );
        assert!(packet.denied);
        assert!(
            packet.warnings.iter().any(|w| w.kind == "denied"),
            "T202: personal budget must not erase kind=denied; got {:?}",
            packet.warnings
        );
    }
}
