use ai_brains_core::ids::{HarnessId, ProjectId, SessionId};
use ai_brains_path::{extract_project_id_from_ledgerful, find_ledgerful_dir};

/// F1: leftover stdout prefix (27 chars including trailing space).
pub(crate) const SHELL_LEFTOVER_PREFIX: &str = "";
/// F1: leftover stdout suffix (17 chars including leading space).
pub(crate) const SHELL_LEFTOVER_SUFFIX: &str = "";
/// F3: file dump replacement for `AI_BRAINS_KEY`.
pub(crate) const SHOW_REDACTED_KEY: &str = "";
/// Daemon/elevation alias: `AI_BRAINS_VAULT_KEY` is live (`ai-brainsd` vault_key.rs, CLI elevation.rs, daemon.rs daemon.env). F36.
pub(crate) const SHOW_REDACTED_VAULT_KEY: &str = "";

pub(crate) fn format_shell_leftover_line(_id: &str) -> String {
    String::new()
}

pub(crate) fn leftover_shell_vs_file(_shell: Option<&str>, _file: Option<&str>) -> Option<String> {
    None
}

pub(crate) fn file_project_id_from_env_text(_content: &str) -> Option<&str> {
    None
}

pub(crate) fn map_show_env_line(_line: &str) -> Option<String> {
    None
}

pub fn run(
    ctx: &crate::context::AppContext,
    new_project: bool,
    new_session: bool,
    show: bool,
    tx_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown-project");

    let env_path = current_dir.join(".env");

    if show {
        if env_path.exists() {
            let content = std::fs::read_to_string(&env_path)?;
            println!("--- Current Context ---");
            for line in content.lines() {
                if line.starts_with("AI_BRAINS_") {
                    println!("{}", line);
                }
            }
            println!("Repository: {}", current_dir.display());
        } else {
            println!(
                "No .env file found in {}. Run 'ai-brains context' to initialize.",
                current_dir.display()
            );
        }
        return Ok(());
    }

    // Auto-discovery from Ledgerful
    let ledgerful_dir = find_ledgerful_dir(&current_dir);
    let discovered_project_id = ledgerful_dir
        .as_ref()
        .and_then(|dir| extract_project_id_from_ledgerful(dir))
        .and_then(|id_str| id_str.parse::<ProjectId>().ok());

    let project_id = if new_project {
        ProjectId::new()
    } else if let Some(id) = discovered_project_id {
        println!("Auto-discovered project ID from .ledgerful: {}", id);
        id
    } else {
        // Deterministic project ID based on the canonical directory path
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&current_dir.to_string_lossy().to_lowercase(), &mut hasher);
        let hash = std::hash::Hasher::finish(&hasher);
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&hash.to_be_bytes());
        ProjectId::from_uuid(uuid::Uuid::from_bytes(bytes))
    };

    let existing_project = if env_path.exists() {
        let existing = std::fs::read_to_string(&env_path)?;
        existing
            .lines()
            .find(|l| l.starts_with("AI_BRAINS_PROJECT_ID"))
            .and_then(|l| l.split('=').nth(1))
            .map(|s| s.to_string())
    } else {
        None
    };

    let existing_session = if env_path.exists() {
        let existing = std::fs::read_to_string(&env_path)?;
        existing
            .lines()
            .find(|l| l.starts_with("AI_BRAINS_SESSION_ID"))
            .and_then(|l| l.split('=').nth(1))
            .map(|s| s.to_string())
    } else {
        None
    };

    if let Some(ref sid) = existing_session {
        // T82: --new-project forces re-initialization even when a session
        // already exists. The early-return below is skipped in that case,
        // and we fall through to assign a fresh project_id and session_id.
        if !new_session && !new_project {
            println!(
                "Context is already initialized for project: {}",
                project_name
            );
            if let Some(ref pid) = existing_project {
                println!("Project ID: {}", pid);
            } else {
                println!("Project ID: {}", project_id);
            }
            println!("Session ID: {}", sid);
            return Ok(());
        }
        if new_project {
            if let Some(ref pid) = existing_project {
                println!("Rotating project ID from {} to fresh UUID.", pid);
            } else {
                println!("Rotating to fresh project ID.");
            }
        }
        if new_session || new_project {
            println!("Replacing existing session: {}", sid);
        }
    }

    let session_id = SessionId::new();
    let harness_id = HarnessId::new();
    let privacy = ai_brains_core::privacy::Privacy::LocalOnly;

    // Ensure project/session exists in the vault (idempotent)
    let mut sink = crate::context::StoreSink {
        store: ai_brains_store::SqliteEventStore::new((*ctx.conn).clone()),
        last_error: None,
        #[cfg(feature = "graph")]
        graph_hook: Some(crate::live_graph::LiveGraphHook::new(
            std::sync::Arc::clone(&ctx.conn),
        )),
    };
    let service = ai_brains_capture::CaptureService::new();
    let capture_context = ai_brains_capture::CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    ctx.ensure_project_and_session_exists(
        &mut sink,
        &service,
        &capture_context,
        project_id,
        session_id,
        harness_id,
        privacy,
    )?;

    let mut env_content = format!(
        "AI_BRAINS_PROJECT_ID={}\nAI_BRAINS_SESSION_ID={}\nAI_BRAINS_HARNESS_ID={}\n",
        project_id, session_id, harness_id
    );

    if let Some(id) = tx_id {
        env_content.push_str(&format!("LEDGERFUL_TX_ID={}\n", id));
    }

    let mut final_content = if env_path.exists() {
        let existing = std::fs::read_to_string(&env_path)?;
        existing
            .lines()
            .filter(|l| {
                !l.starts_with("AI_BRAINS_PROJECT_ID")
                    && !l.starts_with("AI_BRAINS_SESSION_ID")
                    && !l.starts_with("AI_BRAINS_HARNESS_ID")
                    && !l.starts_with("LEDGERFUL_TX_ID")
                    && !l.starts_with("CHANGEGUARD_TX_ID")
            })
            .collect::<Vec<&str>>()
            .join("\n")
    } else {
        String::new()
    };

    if !final_content.is_empty() && !final_content.ends_with('\n') {
        final_content.push('\n');
    }
    final_content.push_str(&env_content);

    std::fs::write(&env_path, final_content)?;

    println!("Context initialized for project: {}", project_name);
    println!("Project ID: {}", project_id);
    println!("Session ID: {}", session_id);
    println!("Local .env updated successfully.");

    // Auto-trigger sync pull to ingest initial signals (hotspots/ledger)
    if !show && let Err(e) = crate::commands::sync::run_pull(ctx, None, true, true, false) {
        tracing::warn!("Auto-triggering sync pull failed: {}", e);
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    const FROZEN_UUID: &str = "7d97a456-f2f4-43ea-1f13-211af684ad37";
    const FILE_UUID: &str = "3581317d-601e-44f7-ab84-fde90aa12d3c";
    const SAME_UUID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

    #[test]
    fn format_shell_leftover_line__known_uuid__frozen_80() {
        assert_eq!(SHELL_LEFTOVER_PREFIX, "shell leftover PROJECT_ID: ");
        assert_eq!(SHELL_LEFTOVER_PREFIX.chars().count(), 27);
        assert_eq!(SHELL_LEFTOVER_SUFFIX, " (.env overrides)");
        assert_eq!(SHELL_LEFTOVER_SUFFIX.chars().count(), 17);
        assert_eq!(FROZEN_UUID.chars().count(), 36);
        let line = format_shell_leftover_line(FROZEN_UUID);
        assert_eq!(line.chars().count(), 80);
        assert!(
            line.starts_with(SHELL_LEFTOVER_PREFIX),
            "must start with prefix; got {line:?}"
        );
        assert!(
            line.ends_with(SHELL_LEFTOVER_SUFFIX),
            "must end with suffix; got {line:?}"
        );
        assert!(
            !line.starts_with("Warning:"),
            "leftover must not start with Warning:; got {line:?}"
        );
        assert_eq!(
            line,
            "shell leftover PROJECT_ID: 7d97a456-f2f4-43ea-1f13-211af684ad37 (.env overrides)"
        );
    }

    #[test]
    fn leftover_shell_vs_file__differ__some() {
        assert_eq!(
            leftover_shell_vs_file(Some(FROZEN_UUID), Some(FILE_UUID)),
            Some(format_shell_leftover_line(FROZEN_UUID))
        );
    }

    #[rstest::rstest]
    #[case(Some(SAME_UUID), Some(SAME_UUID))]
    #[case(None, Some(SAME_UUID))]
    #[case(Some(SAME_UUID), None)]
    #[case(Some(""), Some(SAME_UUID))]
    #[case(Some(SAME_UUID), Some(""))]
    #[case(None, None)]
    fn leftover_shell_vs_file__same_or_missing__none(
        #[case] shell: Option<&str>,
        #[case] file: Option<&str>,
    ) {
        assert_eq!(leftover_shell_vs_file(shell, file), None);
    }

    #[test]
    fn file_project_id_from_env_text__padded_value__trimmed() {
        let content = format!("AI_BRAINS_PROJECT_ID=  {FILE_UUID}  \n");
        assert_eq!(file_project_id_from_env_text(&content), Some(FILE_UUID));
    }

    #[test]
    fn map_show_env_line__key__redacted() {
        assert_eq!(
            map_show_env_line(
                "AI_BRAINS_KEY=x'deadbeefcafebabe0123456789abcdefdeadbeefcafebabe0123456789abcdef'"
            ),
            Some("AI_BRAINS_KEY=(redacted)".to_string())
        );
    }

    #[test]
    fn map_show_env_line__vault_key__redacted() {
        assert_eq!(
            map_show_env_line(
                "AI_BRAINS_VAULT_KEY=x'deadbeefcafebabe0123456789abcdefdeadbeefcafebabe0123456789abcdef'"
            ),
            Some("AI_BRAINS_VAULT_KEY=(redacted)".to_string())
        );
    }

    #[test]
    fn map_show_env_line__bare_key_names__redacted() {
        assert_eq!(
            map_show_env_line("AI_BRAINS_KEY"),
            Some("AI_BRAINS_KEY=(redacted)".to_string())
        );
        assert_eq!(
            map_show_env_line("AI_BRAINS_VAULT_KEY"),
            Some("AI_BRAINS_VAULT_KEY=(redacted)".to_string())
        );
    }

    #[test]
    fn map_show_env_line__project_id__passthrough() {
        let line = "AI_BRAINS_PROJECT_ID=3581317d-601e-44f7-ab84-fde90aa12d3c";
        assert_eq!(map_show_env_line(line), Some(line.to_string()));
    }

    #[test]
    fn map_show_env_line__comment_and_ledgerful__skip() {
        assert_eq!(map_show_env_line("# comment"), None);
        assert_eq!(map_show_env_line("LEDGERFUL_TX_ID=abc"), None);
    }

    #[test]
    fn map_show_env_line__keyring_and_vault_key_path__passthrough() {
        assert_eq!(
            map_show_env_line("AI_BRAINS_KEYRING=foo"),
            Some("AI_BRAINS_KEYRING=foo".to_string())
        );
        assert_eq!(
            map_show_env_line("AI_BRAINS_VAULT_KEY_PATH=/x"),
            Some("AI_BRAINS_VAULT_KEY_PATH=/x".to_string())
        );
    }
}
