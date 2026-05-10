use std::collections::HashSet;

use crate::ansi::strip_ansi;
use crate::errors::Result;
use crate::privacy_filter::is_injectable_privacy;
use crate::sessions::active_sessions;
use crate::word_budget::{trim_to_word_budget, word_count};
use crate::GraphSearch;
use ai_brains_store::VaultConnection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightContext {
    pub text: String,
    pub word_count: usize,
}

pub fn build_preflight(
    conn: &VaultConnection,
    _graph: Option<&GraphSearch>,
    max_words: usize,
    project_id: Option<ai_brains_core::ids::ProjectId>,
) -> Result<PreflightContext> {
    let active = active_sessions(conn, project_id)?;
    let conn = conn.lock()?;

    let project_id_str: Option<String> = project_id.map(|p| p.to_string());

    let mut sections = Vec::new();

    // --- Onboarding & Safety Section (Max 15% of budget) ---
    let onboarding_budget = (max_words * 15) / 100;
    let mut safety_lines = Vec::new();
    let mut safety_ids: HashSet<String> = HashSet::new();

    let safety_sql = if project_id_str.is_some() {
        "SELECT m.memory_id, m.content FROM memory_projection m
         INNER JOIN session_projection s ON m.session_id = s.session_id
         WHERE (m.content LIKE '%CONSTRAINT:%' OR m.content LIKE '%INVARIANT:%' OR m.content LIKE '%HOTSPOT:%')
         AND m.status = 'pinned'
         AND s.project_id = ?1
         ORDER BY m.updated_at DESC LIMIT 10"
    } else {
        "SELECT memory_id, content FROM memory_projection
         WHERE (content LIKE '%CONSTRAINT:%' OR content LIKE '%INVARIANT:%' OR content LIKE '%HOTSPOT:%')
         AND status = 'pinned'
         ORDER BY updated_at DESC LIMIT 10"
    };

    let mut safety_stmt = conn.prepare(safety_sql)?;
    let mut safety_rows = if let Some(ref pid) = project_id_str {
        safety_stmt.query(rusqlite::params![pid])?
    } else {
        safety_stmt.query([])?
    };
    while let Some(row) = safety_rows.next()? {
        let memory_id: String = row.get(0)?;
        let content: String = row.get(1)?;
        safety_ids.insert(memory_id);
        safety_lines.push(strip_ansi(&content));
    }

    if !safety_lines.is_empty() {
        let safety_text = format!(
            "--- Repository Bearings & Safety ---\n{}",
            safety_lines.join("\n\n")
        );
        sections.push(trim_to_word_budget(&safety_text, onboarding_budget));
    }

    if !active.is_empty() {
        let mut session_texts = Vec::new();
        for session in active {
            let mut session_lines = vec![format!("--- Session: {} ---", session.session_id)];
            for turn in session.turns {
                session_lines.push(format!("{}: {}", turn.role.to_uppercase(), turn.content));
            }
            session_texts.push(session_lines.join("\n"));
        }
        sections.push(session_texts.join("\n\n"));
    }

    // --- General Memory Index (scoped to current project when project_id is known) ---
    let index_sql = if project_id_str.is_some() {
        "SELECT m.memory_id, m.content, m.privacy
         FROM memory_projection m
         INNER JOIN session_projection s ON m.session_id = s.session_id
         WHERE m.status = 'pinned'
         AND s.project_id = ?1
         ORDER BY m.updated_at DESC"
    } else {
        "SELECT memory_id, content, privacy
         FROM memory_projection
         WHERE status = 'pinned'
         ORDER BY updated_at DESC"
    };

    let mut stmt = conn.prepare(index_sql)?;
    let mut rows = if let Some(ref pid) = project_id_str {
        stmt.query(rusqlite::params![pid])?
    } else {
        stmt.query([])?
    };
    let mut collected = Vec::new();

    while let Some(row) = rows.next()? {
        let memory_id: String = row.get(0)?;
        let privacy: String = row.get(2)?;

        // Skip entries already shown in the safety section
        if safety_ids.contains(&memory_id) {
            continue;
        }

        if !is_injectable_privacy(&privacy) {
            continue;
        }

        let content: String = row.get(1)?;
        let content = strip_ansi(&content);
        let candidate = if collected.is_empty() {
            content.clone()
        } else {
            format!("{}\n\n{}", collected.join("\n\n"), content)
        };

        if word_count(&candidate) > max_words {
            break;
        }
        collected.push(content);
    }

    if !collected.is_empty() {
        // 1. Build the index section
        let mut index_lines = vec!["--- Memory Index (Briefing) ---".to_string()];
        for (i, content) in collected.iter().enumerate() {
            let first_line = content.lines().next().unwrap_or("Untitled Memory");
            let summary = if first_line.len() > 60 {
                format!("{}...", &first_line[..57])
            } else {
                first_line.to_string()
            };
            index_lines.push(format!("{}. {}", i + 1, summary));
        }
        let index_text = index_lines.join("\n");

        // 2. Build the detailed section (only the most recent memory)
        let mut detailed_text = String::new();
        if let Some(most_recent) = collected.first() {
            detailed_text = format!(
                "--- Most Recent Memory ---\n\n{}\n\n(Use 'recall' to fetch details for other index items)",
                most_recent
            );
        }

        // 3. Assemble with budget awareness
        let remaining_budget = max_words.saturating_sub(word_count(&sections.join("\n\n")));
        let full_text = format!("{}\n\n{}", index_text, detailed_text);

        if word_count(&full_text) <= remaining_budget {
            sections.push(full_text);
        } else if word_count(&index_text) <= remaining_budget {
            sections.push(index_text);
        } else {
            sections.push(trim_to_word_budget(&index_text, remaining_budget));
            sections.push("... [Index Truncated]".to_string());
        }
    }

    let text = trim_to_word_budget(&sections.join("\n\n"), max_words);
    Ok(PreflightContext {
        word_count: word_count(&text),
        text,
    })
}
