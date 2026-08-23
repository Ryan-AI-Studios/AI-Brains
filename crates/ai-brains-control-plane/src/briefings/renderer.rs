//! Deterministic JSON + Markdown renderers for briefing packets (no LLM).
//!
//! Shared by CLI `briefing` and governed preflight (`ai-brains-retrieval`) —
//! next-step footers flow into both (T227 F29).

use ai_brains_contracts::briefings::{PersonalContinuityBriefingPacket, ProjectBriefingPacket};
use serde_json::Error as JsonError;

/// Bootstrap next-step after Denied (T227 F10 / T280 F2) — markdown footer.
///
/// Equals JSON `denial_hint` SHORT (no required `--scope …`). T275 grant-wall
/// still follows this line. HINT (POLICY_DENIED envelope) is a separate family.
pub const BRIEFING_DENIED_NEXT_STEP: &str = BRIEFING_DENIED_DENIAL_HINT;

/// JSON `denial_hint` short SOOT (T241 F7/F14) — must contain `policy bootstrap`.
pub const BRIEFING_DENIED_DENIAL_HINT: &str =
    "next: run `ai-brains policy bootstrap --dry-run` then `ai-brains policy bootstrap`";

/// Denied project grant-wall (T275 F2) — not an empty vault.
///
/// One line, 88 chars, ≤140. Must contain `recall`. Does not replace bootstrap next-step.
pub const BRIEFING_DENIED_GRANT_WALL: &str =
    "This is a grant wall, not an empty vault. Pins remain via `ai-brains recall` / `search`.";

/// Denied project empty Decisions/Conclusions body (T275 F1). Allowed-empty stays `_None_`.
pub const BRIEFING_DENIED_HIDDEN: &str = "_(hidden until discovery grants)_";

/// Empty allowed project authority notice (T227 F8 / F17).
pub const BRIEFING_EMPTY_AUTHORITY_NOTICE: &str =
    "_No current authority (decisions/conclusions empty)._";

/// Empty allowed project next-step (T227 F8 / F17; T263 F2 / F29).
///
/// Vault pins are not governed authority. Daily decisions: `recall` / `search`.
/// One line, ≤140 chars (preflight footer).
pub const BRIEFING_EMPTY_AUTHORITY_NEXT_STEP: &str =
    "next: `ai-brains recall` / `search` for vault pins; typed Approved needs propose + approve";

/// Granted-empty vault-pin heading (T288 F2 / F30). Not an Approved authority section.
pub const BRIEFING_VAULT_PINS_HEADING: &str = "## Vault pins (not Approved)";

/// Display-only vault inventory overlay (T288 F11). Not a packet DTO field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultPinStanza {
    pub count: u64,
    pub previews: Vec<String>,
}

/// Personal deny markdown next-step (T263 F4) — unused/optional, not a required bootstrap.
pub const BRIEFING_PERSONAL_DENIED_NEXT_STEP: &str =
    "next: Personal continuity is optional; daily decisions: `ai-brains recall` / `search`";

/// Personal deny JSON `denial_hint` (T263 F4 / F23). Must contain `recall`, not `policy bootstrap`.
pub const BRIEFING_PERSONAL_DENIED_DENIAL_HINT: &str =
    "Personal continuity is optional; daily decisions: `ai-brains recall` / `search`";

/// Denied Personal empty Preferences/Continuity body (T289 F2).
///
/// Allowed-empty stays `_None_`. One line, ≤140 chars. Not a grant wall.
pub const BRIEFING_PERSONAL_DENIED_BODY: &str = "_(optional continuity; not a missing vault)_";

/// Empty personal continuity notice (T227 F9 / F17).
pub const BRIEFING_EMPTY_CONTINUITY_NOTICE: &str = "_No personal continuity yet._";

/// Empty personal continuity next-step (T227 F9 / F17). No synthetic summary.
pub const BRIEFING_EMPTY_CONTINUITY_NEXT_STEP: &str = "next: continuity synthesis is deferred; Confirmed personal preferences appear when ReadConclusions grants and Confirmed conclusions exist on Personal scope";

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
    render_project_markdown_with_vault_pins(packet, None)
}

/// Same as [`render_project_markdown`] plus an optional vault-pin stanza (T288).
///
/// `None` ≡ today's render (governed preflight keeps this path).
pub fn render_project_markdown_with_vault_pins(
    packet: &ProjectBriefingPacket,
    vault_pins: Option<&VaultPinStanza>,
) -> String {
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
        // F10/F29: next-step immediately after Denied so preflight word budget keeps it.
        lines.push(String::new());
        lines.push(BRIEFING_DENIED_NEXT_STEP.to_string());
        // T275 F1/F2: grant-wall after bootstrap next, before Decisions.
        lines.push(String::new());
        lines.push(BRIEFING_DENIED_GRANT_WALL.to_string());
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
        lines.push(empty_section_placeholder(packet.denied).to_string());
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
        lines.push(empty_section_placeholder(packet.denied).to_string());
    } else {
        for c in &packet.conclusions {
            lines.push(format!("- **{}** [{}]: {}", c.id, c.state, c.statement));
            for h in &c.evidence_handles {
                lines.push(format!("  - evidence: `{}`", h.evidence_id));
            }
        }
    }

    // F8/F27: empty authority only when allowed (never when denied).
    if !packet.denied && packet.decisions.is_empty() && packet.conclusions.is_empty() {
        lines.push(String::new());
        lines.push(BRIEFING_EMPTY_AUTHORITY_NOTICE.to_string());
        lines.push(BRIEFING_EMPTY_AUTHORITY_NEXT_STEP.to_string());
        if let Some(stanza) = vault_pins {
            lines.push(String::new());
            lines.push(BRIEFING_VAULT_PINS_HEADING.to_string());
            lines.push(format!("Pinned: {}", stanza.count));
            if stanza.previews.is_empty() {
                if stanza.count > 0 {
                    lines.push(
                        "_No leading-line DECISION/CONSTRAINT samples in this scope._".to_string(),
                    );
                }
            } else {
                for preview in stanza.previews.iter().take(3) {
                    lines.push(format!("- {preview}"));
                }
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

/// Denied empty authority uses the hidden placeholder; allowed empty keeps `_None_` (T275 AC6).
fn empty_section_placeholder(denied: bool) -> &'static str {
    if denied {
        BRIEFING_DENIED_HIDDEN
    } else {
        "_None_"
    }
}

/// Denied Personal empty prefs/continuity is optional continuity, not `_None_` (T289 F2).
///
/// Private — do not share `empty_section_placeholder` (T275 F35 / T289 F3).
fn personal_empty_section_placeholder(denied: bool) -> &'static str {
    if denied {
        BRIEFING_PERSONAL_DENIED_BODY
    } else {
        "_None_"
    }
}

/// Deterministic Markdown render of a Personal continuity packet.
pub fn render_personal_markdown(packet: &PersonalContinuityBriefingPacket) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# Personal Continuity Briefing".to_string());
    lines.push(String::new());
    lines.push(format!("**Scope:** `{}`", packet.scope_key));
    if packet.denied {
        // F30: blank line before Denied (parity with project renderer).
        lines.push(String::new());
        lines.push(format!(
            "> **Denied:** {}",
            packet.denial_reason.as_deref().unwrap_or("policy denied")
        ));
        // T263 F4: Personal deny names recall (not repository bootstrap).
        lines.push(String::new());
        lines.push(BRIEFING_PERSONAL_DENIED_NEXT_STEP.to_string());
    }

    lines.push(String::new());
    lines.push("## Preferences".to_string());
    if packet.preferences.is_empty() {
        lines.push(personal_empty_section_placeholder(packet.denied).to_string());
    } else {
        for p in &packet.preferences {
            lines.push(format!("- {}", p.statement));
        }
    }

    lines.push(String::new());
    lines.push("## Continuity".to_string());
    if packet.continuity.summary.is_empty() {
        lines.push(personal_empty_section_placeholder(packet.denied).to_string());
    } else {
        lines.push(packet.continuity.summary.clone());
    }

    // F9/F27: empty continuity honesty only when allowed (never when denied).
    if !packet.denied && packet.continuity.summary.is_empty() {
        lines.push(String::new());
        lines.push(BRIEFING_EMPTY_CONTINUITY_NOTICE.to_string());
        lines.push(BRIEFING_EMPTY_CONTINUITY_NEXT_STEP.to_string());
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

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_contracts::briefings::{
        BriefingScopeDto, BudgetReportDto, ContinuitySummaryDto, FreshnessSummaryDto,
        PersonalContinuityBriefingPacket, ProjectBriefingPacket,
    };

    fn empty_project(denied: bool) -> ProjectBriefingPacket {
        ProjectBriefingPacket {
            api_version: "1".into(),
            briefing_id: "b1".into(),
            kind: "Project".into(),
            scope: BriefingScopeDto {
                scope_key: "Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
                confidence: "High".into(),
                warnings: Vec::new(),
                alternatives: Vec::new(),
                authoritative: true,
            },
            handoff: None,
            decisions: Vec::new(),
            conclusions: Vec::new(),
            constraints: Vec::new(),
            warnings: if denied {
                vec![ai_brains_contracts::briefings::BriefingWarningDto {
                    kind: "denied".into(),
                    message: "no grant".into(),
                    subject_id: None,
                    subject_kind: None,
                }]
            } else {
                Vec::new()
            },
            freshness: FreshnessSummaryDto {
                total_sources: 0,
                fresh_count: 0,
                stale_count: 0,
                unavailable_count: 0,
                worst_state: "Unknown".into(),
            },
            ledgerful: None,
            evidence_handles: Vec::new(),
            budget: BudgetReportDto {
                max_words: 1500,
                used_words: 0,
                truncated_sections: Vec::new(),
                more_available: false,
            },
            generated_at: None,
            denied,
            denial_reason: if denied {
                Some("ReadDecisions/ReadConclusions denied".into())
            } else {
                None
            },
            denial_hint: if denied {
                Some(BRIEFING_DENIED_DENIAL_HINT.into())
            } else {
                None
            },
        }
    }

    fn empty_personal(denied: bool) -> PersonalContinuityBriefingPacket {
        PersonalContinuityBriefingPacket {
            api_version: "1".into(),
            briefing_id: "b1".into(),
            kind: "Personal".into(),
            scope_key: "Personal:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
            preferences: Vec::new(),
            continuity: ContinuitySummaryDto {
                summary: String::new(),
                thread_handles: Vec::new(),
            },
            open_review_items: Vec::new(),
            grants_applied: Vec::new(),
            warnings: if denied {
                vec![ai_brains_contracts::briefings::BriefingWarningDto {
                    kind: "denied".into(),
                    message: "denied".into(),
                    subject_id: None,
                    subject_kind: None,
                }]
            } else {
                Vec::new()
            },
            budget: BudgetReportDto {
                max_words: 800,
                used_words: 0,
                truncated_sections: Vec::new(),
                more_available: false,
            },
            generated_at: None,
            denied,
            denial_reason: if denied {
                Some("Personal scope read denied without grant".into())
            } else {
                None
            },
            denial_hint: if denied {
                Some(BRIEFING_PERSONAL_DENIED_DENIAL_HINT.into())
            } else {
                None
            },
        }
    }

    #[test]
    fn briefing_empty_authority_next_step__contains_recall_not_seed_approved() {
        // T263 AC1 / F2
        assert!(
            BRIEFING_EMPTY_AUTHORITY_NEXT_STEP.contains("recall"),
            "empty_authority next must name recall; got {BRIEFING_EMPTY_AUTHORITY_NEXT_STEP}"
        );
        assert!(
            !BRIEFING_EMPTY_AUTHORITY_NEXT_STEP.starts_with("seed an Approved"),
            "must not lead with seed-Approved; got {BRIEFING_EMPTY_AUTHORITY_NEXT_STEP}"
        );
        assert!(
            !BRIEFING_EMPTY_AUTHORITY_NEXT_STEP.starts_with("next: seed an Approved"),
            "must not lead with seed-Approved; got {BRIEFING_EMPTY_AUTHORITY_NEXT_STEP}"
        );
    }

    #[test]
    fn briefing_empty_authority_next_step__one_line_at_most_140_chars() {
        // T263 AC14 / F29
        assert!(
            !BRIEFING_EMPTY_AUTHORITY_NEXT_STEP.contains('\n'),
            "empty_authority next must be one line; got {BRIEFING_EMPTY_AUTHORITY_NEXT_STEP:?}"
        );
        let n = BRIEFING_EMPTY_AUTHORITY_NEXT_STEP.chars().count();
        assert!(
            n <= 140,
            "empty_authority next must be <=140 chars (got {n}): {BRIEFING_EMPTY_AUTHORITY_NEXT_STEP}"
        );
    }

    #[test]
    fn render_project_markdown_with_vault_pins__some__inserts_after_empty_authority() {
        // T288 AC3
        let empty_allowed = empty_project(false);
        let stanza = VaultPinStanza {
            count: 12,
            previews: vec!["DECISION: x".into()],
        };
        let with = render_project_markdown_with_vault_pins(&empty_allowed, Some(&stanza));
        let without = render_project_markdown(&empty_allowed);
        assert!(
            with.contains(BRIEFING_VAULT_PINS_HEADING),
            "Some stanza must emit heading; got {with}"
        );
        assert!(
            with.contains("Pinned: 12"),
            "Some stanza must emit inventory count; got {with}"
        );
        assert!(
            with.contains("- DECISION: x"),
            "Some stanza must emit preview; got {with}"
        );
        let next_pos = with
            .find(BRIEFING_EMPTY_AUTHORITY_NEXT_STEP)
            .expect("empty_authority next");
        let heading_pos = with
            .find(BRIEFING_VAULT_PINS_HEADING)
            .expect("vault pins heading");
        let freshness_pos = with.find("## Freshness").expect("freshness");
        assert!(
            next_pos < heading_pos && heading_pos < freshness_pos,
            "stanza must sit after empty_authority next and before Freshness; got {with}"
        );
        assert!(
            with.contains("_None_"),
            "Decisions body must stay _None_; got {with}"
        );
        assert!(
            !without.contains(BRIEFING_VAULT_PINS_HEADING),
            "None/default render must omit heading; got {without}"
        );
    }

    #[test]
    fn render_project_markdown__allowed_empty__names_recall() {
        // T263 AC1
        let md = render_project_markdown(&empty_project(false));
        assert!(
            md.contains(BRIEFING_EMPTY_AUTHORITY_NEXT_STEP),
            "allowed empty must emit new next-step: {md}"
        );
        assert!(
            md.contains("recall"),
            "allowed empty markdown must name recall: {md}"
        );
        assert!(
            !md.contains("seed an Approved"),
            "allowed empty must not keep old seed-Approved lead-in: {md}"
        );
    }

    #[test]
    fn render_personal_markdown__denied__no_none_placeholder() {
        // T289 AC1 — required red on today's `_None_` branches.
        let md = render_personal_markdown(&empty_personal(true));
        assert!(
            !md.contains("_None_"),
            "denied personal must not look like empty preferences: {md}"
        );
        assert!(
            md.contains(BRIEFING_PERSONAL_DENIED_BODY),
            "denied empty sections use optional-continuity body: {md}"
        );
        assert!(
            md.contains(&format!("## Preferences\n{BRIEFING_PERSONAL_DENIED_BODY}")),
            "Preferences must use denied body: {md}"
        );
        assert!(
            md.contains(&format!("## Continuity\n{BRIEFING_PERSONAL_DENIED_BODY}")),
            "Continuity must use denied body: {md}"
        );
        assert!(
            md.contains(BRIEFING_PERSONAL_DENIED_NEXT_STEP),
            "personal deny next-step frozen: {md}"
        );
        assert!(
            md.contains("recall"),
            "personal deny must name recall: {md}"
        );
        assert!(
            !md.contains("policy bootstrap"),
            "personal deny must not contain policy bootstrap: {md}"
        );
        assert!(
            !md.contains(BRIEFING_DENIED_GRANT_WALL),
            "personal deny must not contain project grant-wall: {md}"
        );
        assert!(
            !md.contains(BRIEFING_DENIED_HIDDEN),
            "personal deny must not contain project hidden placeholder: {md}"
        );
        assert!(
            !md.contains(BRIEFING_DENIED_NEXT_STEP),
            "personal deny must not reuse repository bootstrap next: {md}"
        );
    }

    #[test]
    fn briefing_personal_denied_body__exact_optional_one_line() {
        // T289 AC4
        assert_eq!(
            BRIEFING_PERSONAL_DENIED_BODY,
            "_(optional continuity; not a missing vault)_"
        );
        assert!(
            !BRIEFING_PERSONAL_DENIED_BODY.contains('\n'),
            "body must be one line; got {BRIEFING_PERSONAL_DENIED_BODY:?}"
        );
        let n = BRIEFING_PERSONAL_DENIED_BODY.chars().count();
        assert!(n <= 140, "body must be <=140 chars (got {n})");
        assert!(
            BRIEFING_PERSONAL_DENIED_BODY.contains("optional"),
            "body must contain optional; got {BRIEFING_PERSONAL_DENIED_BODY:?}"
        );
        assert!(
            !BRIEFING_PERSONAL_DENIED_BODY.contains("_None_"),
            "const must not contain _None_; got {BRIEFING_PERSONAL_DENIED_BODY:?}"
        );
        assert!(
            !BRIEFING_PERSONAL_DENIED_BODY.contains("policy bootstrap"),
            "const must not contain policy bootstrap; got {BRIEFING_PERSONAL_DENIED_BODY:?}"
        );
    }

    #[test]
    fn personal_empty_denied__json__no_new_keys() {
        // T289 AC12 — packet keys frozen; no T288 overlay extras.
        let packet = PersonalContinuityBriefingPacket::empty_denied(
            "b1".into(),
            "Personal:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "denied",
        );
        let value = serde_json::to_value(&packet).expect("serialize empty_denied");
        let obj = value.as_object().expect("object");
        assert!(
            !obj.contains_key("vault_pin_count") && !obj.contains_key("vault_pin_previews"),
            "T288 overlay keys must not appear on Personal JSON: {value}"
        );
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "api_version",
                "briefing_id",
                "budget",
                "continuity",
                "denial_reason",
                "denied",
                "grants_applied",
                "kind",
                "open_review_items",
                "preferences",
                "scope_key",
                "warnings",
            ]
        );
        assert_eq!(value["preferences"], serde_json::json!([]));
        assert_eq!(value["continuity"]["summary"], "");
        assert_eq!(value["denied"], true);
        assert!(
            value.get("denial_hint").is_none(),
            "empty_denied leaves denial_hint omitted: {value}"
        );
    }

    #[test]
    fn render_personal_markdown__denied__names_recall_not_personal_bootstrap() {
        // T263 AC3 / F4
        let md = render_personal_markdown(&empty_personal(true));
        assert!(
            md.contains("recall"),
            "personal deny must name recall: {md}"
        );
        assert!(
            !md.contains("policy bootstrap"),
            "personal deny must not lead with policy bootstrap: {md}"
        );
        assert!(
            !md.contains(BRIEFING_DENIED_NEXT_STEP),
            "personal deny must not reuse repository bootstrap next: {md}"
        );
        // T275 AC16 / F35 — project grant-wall consts must not leak into Personal deny.
        assert!(
            !md.contains(BRIEFING_DENIED_GRANT_WALL),
            "personal deny must not contain project grant-wall: {md}"
        );
        assert!(
            !md.contains(BRIEFING_DENIED_HIDDEN),
            "personal deny must not contain project hidden placeholder: {md}"
        );
    }

    #[test]
    fn render_project_markdown__denied__no_none_placeholder() {
        // T275 AC1
        let md = render_project_markdown(&empty_project(true));
        assert!(
            !md.contains("_None_"),
            "denied project must not look like an empty vault: {md}"
        );
        assert!(
            md.contains(BRIEFING_DENIED_GRANT_WALL),
            "denied markdown must emit grant-wall: {md}"
        );
        assert!(
            BRIEFING_DENIED_GRANT_WALL.contains("recall"),
            "grant-wall must name recall"
        );
        assert!(md.contains("> **Denied:**"), "denied blockquote: {md}");
        assert!(
            md.contains("policy bootstrap"),
            "bootstrap stays primary next-step: {md}"
        );
        assert!(
            md.contains(BRIEFING_DENIED_HIDDEN),
            "denied empty sections use hidden placeholder: {md}"
        );
    }

    #[test]
    fn briefing_denied_grant_wall__88_chars_order_before_decisions() {
        // T275 AC2 / F2 / F29 — renderer order, not a preflight budget hermetic.
        assert!(
            !BRIEFING_DENIED_GRANT_WALL.contains('\n'),
            "grant-wall must be one line; got {BRIEFING_DENIED_GRANT_WALL:?}"
        );
        let n = BRIEFING_DENIED_GRANT_WALL.chars().count();
        assert_eq!(n, 88, "frozen GRANT_WALL must be 88 chars (got {n})");
        assert!(n <= 140, "grant-wall must be <=140 chars (got {n})");
        let md = render_project_markdown(&empty_project(true));
        let next_pos = md.find(BRIEFING_DENIED_NEXT_STEP).expect("next-step pos");
        let wall_pos = md.find(BRIEFING_DENIED_GRANT_WALL).expect("grant-wall pos");
        let decisions_pos = md
            .find("## Decisions (current authority)")
            .expect("decisions pos");
        assert!(
            next_pos < wall_pos,
            "grant-wall must follow bootstrap next: {md}"
        );
        assert!(
            wall_pos < decisions_pos,
            "grant-wall must precede ## Decisions: {md}"
        );
    }

    #[test]
    fn render_project_markdown__allowed_empty__keeps_none_not_grant_wall() {
        // T275 AC6 — grant-wall / hidden are denied-only.
        let md = render_project_markdown(&empty_project(false));
        assert!(
            md.contains("_None_"),
            "allowed empty still emits _None_: {md}"
        );
        assert!(
            md.contains(BRIEFING_EMPTY_AUTHORITY_NOTICE),
            "allowed empty still emits empty_authority: {md}"
        );
        assert!(
            !md.contains(BRIEFING_DENIED_GRANT_WALL),
            "grant-wall is denied-only: {md}"
        );
        assert!(
            !md.contains(BRIEFING_DENIED_HIDDEN),
            "hidden placeholder is denied-only: {md}"
        );
    }

    #[test]
    fn render_project_markdown__allowed_empty__emits_empty_authority_next_step() {
        // AC7
        let md = render_project_markdown(&empty_project(false));
        assert!(
            md.contains(BRIEFING_EMPTY_AUTHORITY_NOTICE),
            "empty authority notice: {md}"
        );
        assert!(
            md.contains(BRIEFING_EMPTY_AUTHORITY_NEXT_STEP),
            "empty authority next-step: {md}"
        );
        assert!(
            !md.contains(BRIEFING_DENIED_NEXT_STEP),
            "allowed empty must not get deny next-step: {md}"
        );
        assert!(!md.contains("**Denied:**"), "must not show Denied: {md}");
    }

    /// T280 AC4 / F2 — markdown next equals SHORT; order Denied → next → grant-wall → Decisions.
    #[test]
    fn render_project_markdown__denied__next_step_omits_scope_ellipsis() {
        assert_eq!(BRIEFING_DENIED_NEXT_STEP, BRIEFING_DENIED_DENIAL_HINT);
        assert!(
            !BRIEFING_DENIED_NEXT_STEP.contains("--scope …"),
            "markdown next must not require --scope ellipsis; got {BRIEFING_DENIED_NEXT_STEP}"
        );
        assert!(
            !BRIEFING_DENIED_DENIAL_HINT.contains("--scope …"),
            "JSON denial_hint must not require --scope ellipsis; got {BRIEFING_DENIED_DENIAL_HINT}"
        );
        let md = render_project_markdown(&empty_project(true));
        let denied_pos = md.find("> **Denied:**").expect("denied pos");
        let next_pos = md.find(BRIEFING_DENIED_NEXT_STEP).expect("next-step pos");
        let wall_pos = md.find(BRIEFING_DENIED_GRANT_WALL).expect("grant-wall pos");
        let decisions_pos = md
            .find("## Decisions (current authority)")
            .expect("decisions pos");
        assert!(denied_pos < next_pos, "Denied must precede next-step: {md}");
        assert!(
            next_pos < wall_pos,
            "next-step must precede grant-wall: {md}"
        );
        assert!(
            wall_pos < decisions_pos,
            "grant-wall must precede ## Decisions: {md}"
        );
        assert!(
            md.contains(BRIEFING_DENIED_GRANT_WALL),
            "T275 grant-wall stays: {md}"
        );
    }

    #[test]
    fn render_project_markdown__denied__bootstrap_next_step_no_empty_authority() {
        // AC7 + AC9
        let md = render_project_markdown(&empty_project(true));
        assert!(md.contains("> **Denied:**"), "denied blockquote: {md}");
        assert!(
            md.contains(BRIEFING_DENIED_NEXT_STEP),
            "deny next-step: {md}"
        );
        assert!(
            md.contains("policy bootstrap"),
            "bootstrap token for preflight budget (F29): {md}"
        );
        assert!(
            !md.contains(BRIEFING_EMPTY_AUTHORITY_NOTICE),
            "denied must not emit empty_authority: {md}"
        );
        assert!(
            !md.contains(BRIEFING_EMPTY_AUTHORITY_NEXT_STEP),
            "denied must not emit empty_authority next-step: {md}"
        );
        // Next-step appears before Decisions so word budget keeps it (F29).
        let deny_pos = md.find(BRIEFING_DENIED_NEXT_STEP).expect("next-step pos");
        let decisions_pos = md
            .find("## Decisions (current authority)")
            .expect("decisions pos");
        assert!(
            deny_pos < decisions_pos,
            "deny next-step must precede decisions for budget survival"
        );
    }

    #[test]
    fn render_personal_markdown__allowed_empty__emits_empty_continuity_next_step() {
        // AC8
        let md = render_personal_markdown(&empty_personal(false));
        assert!(
            md.contains(BRIEFING_EMPTY_CONTINUITY_NOTICE),
            "empty continuity notice: {md}"
        );
        assert!(
            md.contains(BRIEFING_EMPTY_CONTINUITY_NEXT_STEP),
            "empty continuity next-step: {md}"
        );
        assert!(
            !md.contains(BRIEFING_DENIED_NEXT_STEP),
            "allowed empty must not get deny next-step: {md}"
        );
        // No synthetic continuity fill. T289 AC5: both empty sections stay `_None_`.
        assert!(
            md.contains("## Preferences\n_None_"),
            "preferences stay _None_ when allowed-empty: {md}"
        );
        assert!(
            md.contains("## Continuity\n_None_"),
            "continuity stays _None_: {md}"
        );
    }

    #[test]
    fn render_personal_markdown__denied__blank_line_before_denied_and_recall() {
        // AC9 + AC9b / F30
        let md = render_personal_markdown(&empty_personal(true));
        assert!(md.contains("**Scope:**"), "scope line present: {md}");
        // Blank line between Scope and Denied: "Scope…\n\n> **Denied:**"
        assert!(
            md.contains("`\n\n> **Denied:**") || md.contains("\n\n> **Denied:**"),
            "blank line before Denied required (F30): {md}"
        );
        assert!(
            md.contains(BRIEFING_PERSONAL_DENIED_NEXT_STEP),
            "personal deny next-step: {md}"
        );
        assert!(
            !md.contains(BRIEFING_DENIED_NEXT_STEP),
            "personal deny must not reuse repository bootstrap next: {md}"
        );
        assert!(
            !md.contains(BRIEFING_EMPTY_CONTINUITY_NOTICE),
            "denied must not emit empty_continuity: {md}"
        );
        assert!(
            !md.contains(BRIEFING_EMPTY_CONTINUITY_NEXT_STEP),
            "denied must not emit empty_continuity next-step: {md}"
        );
    }
}
