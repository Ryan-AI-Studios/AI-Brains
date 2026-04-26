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
    graph: Option<&GraphSearch>,
    max_words: usize,
) -> Result<PreflightContext> {
    let active = active_sessions(conn)?;
    let conn = conn.lock()?;
    let mut stmt = conn.prepare(
        "SELECT content, privacy
         FROM memory_projection
         WHERE status = 'pinned'
         ORDER BY updated_at DESC",
    )?;
    let mut rows = stmt.query([])?;
    let mut collected = Vec::new();

    while let Some(row) = rows.next()? {
        let privacy: String = row.get(1)?;
        if !is_injectable_privacy(&privacy) {
            continue;
        }

        let content: String = row.get(0)?;
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

    let mut sections = Vec::new();
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
    if !collected.is_empty() {
        sections.push(format!(
            "--- Pinned Memories ---\n\n{}",
            collected.join("\n\n")
        ));
    }

    let text = trim_to_word_budget(&sections.join("\n\n"), max_words);
    Ok(PreflightContext {
        word_count: word_count(&text),
        text,
    })
}
