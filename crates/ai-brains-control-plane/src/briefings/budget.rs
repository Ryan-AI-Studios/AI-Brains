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
        packet.warnings.truncate(cfg.max_warnings);
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
            if let Some(w) = packet.warnings.pop() {
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

    packet.budget = BudgetReportDto {
        max_words: cfg.max_words,
        used_words: used,
        truncated_sections: truncated,
        more_available: more || (cfg.max_words > 0 && project_used_words(packet) > cfg.max_words),
    };
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
            if let Some(w) = packet.warnings.pop() {
                used = used.saturating_sub(approx_words(&w.message));
                push_truncated_once(&mut truncated, BriefingSection::Warnings.as_str());
            }
        }
        used = personal_used_words(packet);
        if used > cfg.max_words {
            used = cfg.max_words;
        }
    }
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
