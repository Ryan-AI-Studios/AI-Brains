//! T265 — split compact preflight JSON `text` into additive `sections[]`.
//!
//! Pure split of already-assembled `context.text`. Does not re-query SQL or
//! apply PrettyCaps. Header match copies pretty `contains` / `starts_with`
//! plus Ledgerful / empty-repo ids (F5).

use ai_brains_contracts::preflight::{PreflightContextResponse, PreflightSection};

pub(crate) const SECTION_ID_SAFETY: &str = "safety";
pub(crate) const SECTION_ID_SESSION: &str = "session";
pub(crate) const SECTION_ID_INDEX: &str = "index";
pub(crate) const SECTION_ID_RECENT: &str = "recent";
pub(crate) const SECTION_ID_LEDGERFUL: &str = "ledgerful";
pub(crate) const SECTION_ID_EMPTY_REPO: &str = "empty_repo";
pub(crate) const SECTION_ID_GOVERNED: &str = "governed";
pub(crate) const SECTION_ID_OTHER: &str = "other";

/// Full-line legacy section header (copy of pretty `is_legacy_section_header`).
fn is_legacy_section_header(line: &str) -> bool {
    let t = line.trim();
    t.len() >= 7 && t.starts_with("---") && t.ends_with("---")
}

/// F5 header → closed-set id string (pretty table + Ledgerful / empty_repo).
fn classify_json_section_id(header: &str) -> &'static str {
    let t = header.trim();
    if t.contains("Repository Bearings") || t.contains("Bearings & Safety") {
        SECTION_ID_SAFETY
    } else if t.contains("Memory Index") {
        SECTION_ID_INDEX
    } else if t.contains("Most Recent Memories") {
        SECTION_ID_RECENT
    } else if t.starts_with("--- Session:") || t.starts_with("--- Session ") {
        SECTION_ID_SESSION
    } else if t.contains("Ledgerful Intelligence") {
        SECTION_ID_LEDGERFUL
    } else if t.contains("New Repository Detected") {
        SECTION_ID_EMPTY_REPO
    } else {
        SECTION_ID_OTHER
    }
}

/// Blank-line-separated item blocks (copy of pretty `split_item_blocks`).
fn split_item_blocks(lines: &[&str]) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            if !cur.is_empty() {
                blocks.push(cur.join("\n"));
                cur.clear();
            }
        } else {
            cur.push(*line);
        }
    }
    if !cur.is_empty() {
        blocks.push(cur.join("\n"));
    }
    blocks
}

/// Split already-assembled `context.text` into additive JSON sections (F4–F7).
pub(crate) fn split_preflight_sections(text: &str) -> Vec<PreflightSection> {
    let lines: Vec<&str> = text.lines().collect();
    let header_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| is_legacy_section_header(line).then_some(i))
        .collect();

    if header_indices.is_empty() {
        if text.contains("# Project Briefing (governed)") {
            return vec![PreflightSection {
                id: SECTION_ID_GOVERNED.to_string(),
                title: "Project Briefing (governed)".to_string(),
                items: vec![text.to_string()],
            }];
        }
        return Vec::new();
    }

    let mut sections = Vec::with_capacity(header_indices.len());
    for (idx, &start) in header_indices.iter().enumerate() {
        let end = header_indices.get(idx + 1).copied().unwrap_or(lines.len());
        let title = lines[start].trim().to_string();
        let id = classify_json_section_id(&title);
        let body = &lines[start + 1..end];
        let items: Vec<String> = split_item_blocks(body)
            .into_iter()
            .map(|block| block.trim().to_string())
            .filter(|block| !block.is_empty())
            .collect();
        sections.push(PreflightSection {
            id: id.to_string(),
            title,
            items,
        });
    }
    sections
}

/// Build the compact full-preflight JSON envelope (required keys + `sections`).
pub(crate) fn build_preflight_json(text: String, word_count: usize) -> PreflightContextResponse {
    let sections = split_preflight_sections(&text);
    PreflightContextResponse {
        text,
        word_count,
        sections,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    fn ids_of(sections: &[PreflightSection]) -> Vec<&str> {
        sections.iter().map(|s| s.id.as_str()).collect()
    }

    #[test]
    fn split_preflight_sections__legacy_headers__ids_in_order() {
        // AC3 — Bearings + one Session (newline-joined turns) + Index + Recent.
        let text = concat!(
            "--- Repository Bearings & Safety ---\n",
            "CONSTRAINT: stay compact\n",
            "\n",
            "--- Session: 11111111-1111-1111-1111-111111111111 ---\n",
            "USER: hello\n",
            "ASSISTANT: world\n",
            "\n",
            "--- Memory Index (Briefing) ---\n",
            "DECISION: one\n",
            "DECISION: two\n",
            "\n",
            "--- Most Recent Memories ---\n",
            "recent body\n",
        );
        let sections = split_preflight_sections(text);
        assert_eq!(
            ids_of(&sections),
            [
                SECTION_ID_SAFETY,
                SECTION_ID_SESSION,
                SECTION_ID_INDEX,
                SECTION_ID_RECENT
            ]
        );
        assert_eq!(sections[0].title, "--- Repository Bearings & Safety ---");
        assert!(
            sections[0]
                .items
                .iter()
                .any(|i| i.contains("CONSTRAINT: stay compact")),
            "safety items must include body; got {:?}",
            sections[0].items
        );
        assert_eq!(
            sections[1].items.len(),
            1,
            "F6: newline-joined session turns collapse to one item; got {:?}",
            sections[1].items
        );
        assert_eq!(
            sections[2].items.len(),
            1,
            "F6: newline-joined index lines collapse to one item; got {:?}",
            sections[2].items
        );
        assert!(
            !sections[3].items.is_empty(),
            "recent body must be non-empty; got {:?}",
            sections[3].items
        );
    }

    #[test]
    fn split_preflight_sections__two_sessions__two_section_rows() {
        // AC4
        let text = concat!(
            "--- Session: aaa ---\n",
            "turn a\n",
            "\n",
            "--- Session: bbb ---\n",
            "turn b\n",
        );
        let sections = split_preflight_sections(text);
        let session_ids: Vec<&str> = sections
            .iter()
            .filter(|s| s.id == SECTION_ID_SESSION)
            .map(|s| s.id.as_str())
            .collect();
        assert_eq!(
            session_ids,
            [SECTION_ID_SESSION, SECTION_ID_SESSION],
            "two Session headers → two session rows; got {sections:?}"
        );
        assert_eq!(sections[0].title, "--- Session: aaa ---");
        assert_eq!(sections[1].title, "--- Session: bbb ---");
    }

    #[test]
    fn split_preflight_sections__no_headers__empty() {
        // AC5
        let sections = split_preflight_sections("DECISION: one\nCONSTRAINT: two");
        assert!(
            sections.is_empty(),
            "no --- headers and no governed marker → []; got {sections:?}"
        );
    }

    #[test]
    fn split_preflight_sections__leading_preamble__discarded() {
        // AC5 additive — F6
        let text = concat!(
            "preamble\n",
            "--- Repository Bearings & Safety ---\n",
            "CONSTRAINT: x\n",
        );
        let sections = split_preflight_sections(text);
        assert_eq!(ids_of(&sections), [SECTION_ID_SAFETY]);
        assert!(
            sections.iter().all(|s| s.id != SECTION_ID_OTHER),
            "preamble must not become other; got {sections:?}"
        );
    }

    #[test]
    fn split_preflight_sections__governed_marker__one_section() {
        // AC6
        let text = "# Project Briefing (governed)\nApproved decision body";
        let sections = split_preflight_sections(text);
        assert_eq!(ids_of(&sections), [SECTION_ID_GOVERNED]);
        assert_eq!(sections[0].title, "Project Briefing (governed)");
        assert_eq!(sections[0].items, vec![text.to_string()]);
    }

    #[test]
    fn split_preflight_sections__ledgerful_and_other() {
        // AC15 — live retrieval header strings
        let text = concat!(
            "--- Ledgerful Intelligence ---\n",
            "Top Hotspots:\n",
            "\n",
            "--- Ledgerful Intelligence (Fallback - Contextual Unavailable) ---\n",
            "Top Hotspots:\n",
            "\n",
            "--- Ledgerful Intelligence (Contextual Risk) ---\n",
            "Top Impacted Hotspots for Current Scope:\n",
            "\n",
            "--- Foo Bar ---\n",
            "unknown body\n",
        );
        let sections = split_preflight_sections(text);
        assert_eq!(
            ids_of(&sections),
            [
                SECTION_ID_LEDGERFUL,
                SECTION_ID_LEDGERFUL,
                SECTION_ID_LEDGERFUL,
                SECTION_ID_OTHER
            ]
        );
        assert_eq!(sections[3].title, "--- Foo Bar ---");
    }

    #[test]
    fn preflight_context_response__n_minus_1_two_key__sections_default_empty() {
        // AC7
        let parsed: PreflightContextResponse =
            serde_json::from_str(r#"{"text":"DECISION: one","word_count":2}"#)
                .expect("n-1 two-key deserializes");
        assert!(
            parsed.sections.is_empty(),
            "#[serde(default)] sections; got {:?}",
            parsed.sections
        );
        assert_eq!(parsed.text, "DECISION: one");
        assert_eq!(parsed.word_count, 2);
    }
}
