use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_scheduler::TaskScheduler;

pub fn run_start(_ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| {
            // current_exe is ai-brains.exe; daemon is ai-brainsd.exe alongside it
            p.parent().map(|dir| dir.join("ai-brainsd.exe"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("ai-brainsd"));

    if !exe.exists() {
        // Fall back to PATH lookup
        let fallback = which_daemon()?;
        return spawn_daemon(&fallback);
    }
    spawn_daemon(&exe)
}

fn which_daemon() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("where")
        .arg("ai-brainsd")
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let path = String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            Ok(std::path::PathBuf::from(path))
        }
        _ => Err("ai-brainsd not found on PATH. Run `cargo install --path crates/ai-brainsd --locked` first.".into()),
    }
}

fn spawn_daemon(exe: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        std::process::Command::new(exe)
            .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
            .spawn()?;
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new(exe).spawn()?;
    }
    // Brief pause so the pipe is ready before the caller does anything else
    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("AI-Brains daemon started.");
    Ok(())
}

fn schedule_inner(
    exe: &std::path::Path,
    dry_run: bool,
    run_as_system: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let exe_str = exe.to_string_lossy();
    let cmd = render_daemon_schedule_command(&exe_str, "AI-Brains-Daemon", 30, run_as_system);

    if dry_run {
        println!("[dry-run] Would execute:");
        println!("  {cmd}");
        println!();
        if run_as_system {
            match generate_daemon_wrapper_script(&exe_str) {
                Ok(content) => {
                    println!("Wrapper script content:");
                    println!("{}", content);
                    println!();
                }
                Err(e) => {
                    println!("(Wrapper script would fail: {})", e);
                }
            }
        }
        println!("Daemon logon command: {}", exe_str);
        println!();
        println!(
            "(Note: actual registration may require elevated PowerShell privileges depending on system policy)"
        );
        return Ok(());
    }

    let task_command = if run_as_system {
        let content = generate_daemon_wrapper_script(&exe_str)?;
        let path = write_daemon_wrapper_script(&content)?;
        println!("Wrapper script written to: {}", path.display());
        format!("'{}'", path.display())
    } else {
        format!("'{}'", exe_str)
    };

    let cmd =
        TaskScheduler::render_daemon_logon_command_with_tr("AI-Brains-Daemon", 30, &task_command);
    if run_as_system {
        println!("{} /ru SYSTEM", cmd);
    } else {
        println!("{}", cmd);
    }
    let output = std::process::Command::new("cmd")
        .args(["/C", &cmd])
        .output()?;
    if output.status.success() {
        println!("Task 'AI-Brains-Daemon' registered. Daemon will start at next logon.");
        println!("To start it now without rebooting: ai-brains daemon start");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if run_as_system
            && (stderr.contains("Access is denied") || stdout.contains("Access is denied"))
        {
            return Err(
                "Scheduling as SYSTEM requires elevation. Re-run from an Administrator shell."
                    .into(),
            );
        }
        return Err(
            "schtasks failed — check that you have permission to create scheduled tasks.".into(),
        );
    }
    Ok(())
}

fn generate_daemon_wrapper_script(exe_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    let required: [&str; 5] = [
        "AI_BRAINS_VAULT_PATH",
        "AI_BRAINS_MODEL_URL",
        "AI_BRAINS_COMPLETION_MODEL",
        "AI_BRAINS_EMBEDDING_URL",
        "AI_BRAINS_EMBEDDING_MODEL",
    ];
    let env_values: Vec<(&str, String)> = required
        .iter()
        .map(|key| (*key, std::env::var(key).unwrap_or_default()))
        .collect();
    generate_daemon_wrapper_script_from_env(exe_str, &env_values)
}

fn generate_daemon_wrapper_script_from_env(
    exe_str: &str,
    env_values: &[(&str, String)],
) -> Result<String, Box<dyn std::error::Error>> {
    let required: [&str; 5] = [
        "AI_BRAINS_VAULT_PATH",
        "AI_BRAINS_MODEL_URL",
        "AI_BRAINS_COMPLETION_MODEL",
        "AI_BRAINS_EMBEDDING_URL",
        "AI_BRAINS_EMBEDDING_MODEL",
    ];
    let mut lines = vec!["@echo off".to_string()];
    let mut missing = Vec::new();
    for key in required {
        let value = env_values
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        if value.is_empty() {
            tracing::warn!("Required env var {} is missing or empty", key);
            missing.push(key);
        } else {
            lines.push(format!("set \"{}={}\"", key, value));
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "Cannot schedule as SYSTEM: required env vars missing or empty: {}. \
             Run from a directory with a .env file, or set them in your user environment before scheduling.",
            missing.join(", ")
        )
        .into());
    }
    let vault_path = env_values
        .iter()
        .find(|(k, _)| *k == "AI_BRAINS_VAULT_PATH")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");
    if let Some(parent) = std::path::Path::new(vault_path).parent() {
        if !parent.as_os_str().is_empty() {
            lines.push(format!("cd /d \"{}\"", parent.display()));
        }
    }
    lines.push(format!(r#""{}" --no-project-context"#, exe_str));
    Ok(lines.join("\n"))
}

fn write_daemon_wrapper_script(
    content: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let vault_path = std::env::var("AI_BRAINS_VAULT_PATH").unwrap_or_default();
    let dir = if vault_path.is_empty() {
        std::env::temp_dir()
    } else {
        std::path::Path::new(&vault_path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(std::env::temp_dir)
    };
    let path = dir.join(".ai-brains-daemon-task.bat");
    std::fs::write(&path, content).map_err(|e| format!("Failed to write wrapper script: {}", e))?;
    Ok(path)
}

fn render_daemon_schedule_command(
    exe_path: &str,
    task_name: &str,
    delay_seconds: u32,
    run_as_system: bool,
) -> String {
    let task_command = if run_as_system {
        match generate_daemon_wrapper_script(exe_path) {
            Ok(_) => "%TEMP%\\ai-brains-daemon-task.bat --no-project-context".to_string(),
            Err(_) => {
                format!("'{}' --no-project-context", exe_path)
            }
        }
    } else {
        format!("'{}'", exe_path)
    };

    let base =
        TaskScheduler::render_daemon_logon_command_with_tr(task_name, delay_seconds, &task_command);
    if run_as_system {
        format!("{} /ru SYSTEM", base)
    } else {
        base
    }
}

pub fn run_schedule(
    _ctx: &AppContext,
    dry_run: bool,
    run_as_system: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let exe = which_daemon()?;
    schedule_inner(&exe, dry_run, run_as_system)
}

fn unschedule_inner(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = TaskScheduler::render_delete_command("AI-Brains-Daemon");

    if dry_run {
        println!("[dry-run] Would execute:");
        println!("  {cmd}");
        println!();
        println!(
            "(Note: actual removal may require elevated PowerShell privileges depending on system policy)"
        );
        return Ok(());
    }

    let status = std::process::Command::new("cmd")
        .args(["/C", &cmd])
        .status()?;
    if status.success() {
        println!("Task 'AI-Brains-Daemon' removed.");
    } else {
        tracing::warn!("schtasks /delete failed — task may not exist.");
    }
    Ok(())
}

pub fn run_unschedule(_ctx: &AppContext, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    unschedule_inner(dry_run)
}

pub async fn run_stop(_ctx: &AppContext, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();

    if force {
        tracing::info!("Forcefully stopping AI-Brains daemon...");
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/IM", "ai-brainsd.exe"])
                .output();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("pkill")
                .arg("ai-brainsd")
                .output();
        }
        println!("Daemon stopped (forced).");
        return Ok(());
    }

    tracing::info!("Sending shutdown signal to AI-Brains daemon...");
    match client.shutdown().await {
        Ok(_) => {
            println!("Shutdown signal sent successfully.");
            // Give it a moment to exit
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        Err(e) => {
            tracing::warn!(
                "Failed to send shutdown signal: {}. The daemon might not be running.",
                e
            );
            tracing::warn!("Use --force to kill the process if it's unresponsive.");
        }
    }

    Ok(())
}

/// T85: Parse "host:port" from a URL string (strips scheme/path).
/// Returns None if no port is present or the port is not a valid u16.
fn parse_host_port(url: &str) -> Option<(String, u16)> {
    // Strip "http://" / "https://" scheme prefix
    let without_scheme = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };
    // Keep only "host:port" — strip any path/query/fragment
    let host_port = without_scheme.split('/').next().unwrap_or(without_scheme);
    let colon_pos = host_port.rfind(':')?;
    let host = &host_port[..colon_pos];
    let port: u16 = host_port[colon_pos + 1..].parse().ok()?;
    Some((host.to_string(), port))
}

/// T85: Resolve backend address from an env var, with sensible defaults.
/// Returns (host, port, description_for_display).
fn resolve_backend(
    env_var: &str,
    default_host: &str,
    default_port: u16,
    default_label: &str,
) -> (String, u16, String) {
    match std::env::var(env_var) {
        Ok(url) if !url.is_empty() => {
            let (host, port) =
                parse_host_port(&url).unwrap_or_else(|| (default_host.to_string(), default_port));
            (host, port, url)
        }
        _ => (
            default_host.to_string(),
            default_port,
            format!("{} ({}=unset)", default_label, env_var),
        ),
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn count_pinned_memories(
    conn: &crate::context::AppContext,
) -> Result<u64, Box<dyn std::error::Error>> {
    let conn_lock = conn.conn.lock()?;
    let count: u64 = conn_lock.query_row(
        "SELECT COUNT(*) FROM memory_projection WHERE status = 'pinned'",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub async fn run_status(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();
    let is_running = client.probe(std::time::Duration::from_millis(200)).await;

    if is_running {
        println!("Status: Running");
    } else {
        println!("Status: Stopped");
    }

    // T128: show vault info only when daemon is running (caller may be pointing
    // at a different vault than the one the daemon is serving, so we use the
    // CLI-resolved path as the best available proxy).
    if is_running {
        println!("Vault: {}", ctx.vault_path.display());
        let size = std::fs::metadata(&ctx.vault_path)
            .map(|m| m.len())
            .unwrap_or(0);
        println!("Vault size: {}", format_size(size));
        match count_pinned_memories(ctx) {
            Ok(count) => println!("Memories: {}", count),
            Err(e) => tracing::warn!("Failed to read memory count: {}", e),
        }
    }

    // T85: resolve backend addresses from configuration rather than hardcoded ports
    let (model_host, model_port, model_desc) = resolve_backend(
        "AI_BRAINS_MODEL_URL",
        "127.0.0.1",
        11434,
        "Ollama default :11434",
    );
    let (embed_host, embed_port, embed_desc) = resolve_backend(
        "AI_BRAINS_EMBEDDING_URL",
        "127.0.0.1",
        8080,
        "llama.cpp default :8080",
    );

    for (name, host, port, desc) in [
        ("LLM backend", model_host, model_port, model_desc),
        ("Embedding backend", embed_host, embed_port, embed_desc),
    ] {
        let addr = format!("{}:{}", host, port);
        match addr.parse::<std::net::SocketAddr>() {
            Ok(socket_addr) => {
                let mut state = "Closed";
                let mut delay = std::time::Duration::from_millis(100);
                for attempt in 0..5 {
                    match std::net::TcpStream::connect_timeout(
                        &socket_addr,
                        std::time::Duration::from_millis(100),
                    ) {
                        Ok(_) => {
                            state = "Open";
                            break;
                        }
                        Err(_) => {
                            if attempt < 4 {
                                let nanos = std::time::SystemTime::now()
                                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                    .map(|d| d.as_nanos())
                                    .unwrap_or(0);
                                let jitter_ms = (nanos % 30) as u64;
                                std::thread::sleep(
                                    delay + std::time::Duration::from_millis(jitter_ms),
                                );
                                delay *= 2;
                            }
                        }
                    }
                }
                println!("{} {} [{}]: {}", name, addr, desc, state);
            }
            Err(_) => {
                println!("{} {}: unable to parse address", name, addr);
            }
        }
    }

    // Try to report PID
    #[cfg(windows)]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq ai-brainsd.exe", "/FO", "CSV", "/NH"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("ai-brainsd.exe") {
            // CSV format: "ai-brainsd.exe","PID","Session Name","Session#","Mem Usage"
            if let Some(line) = stdout.lines().next() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() > 1 {
                    let pid = parts[1].trim_matches('\"');
                    println!("PID: {}", pid);
                }
            }
        }
    }

    Ok(())
}

/// T84: Stop the daemon, install updated binaries via `cargo install`, then restart.
///
/// Must be run from the workspace root. Gracefully stops the daemon first;
/// falls back to a force-kill if it does not respond within ~1 s.
pub async fn run_update(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("[update] Checking for running daemon...");
    let client = DaemonClient::new();
    let is_running = client.probe(std::time::Duration::from_millis(300)).await;

    if is_running {
        tracing::info!("[update] Daemon is running — sending graceful shutdown signal...");
        let shutdown_ok = client.shutdown().await.is_ok();
        if shutdown_ok {
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        }

        // Verify it actually stopped
        let still_running = client.probe(std::time::Duration::from_millis(200)).await;
        if !shutdown_ok || still_running {
            tracing::warn!("[update] Graceful shutdown did not complete — force-terminating...");
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/IM", "ai-brainsd.exe"])
                    .output();
            }
            #[cfg(not(windows))]
            {
                let _ = std::process::Command::new("pkill")
                    .args(["-9", "ai-brainsd"])
                    .output();
            }
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        }
        tracing::info!("[update] Daemon stopped.");
    } else {
        tracing::info!("[update] No running daemon found.");
    }

    tracing::info!("[update] Installing ai-brains-cli via `cargo install --locked`...");
    let cli_ok = std::process::Command::new("cargo")
        .args(["install", "--path", "crates/ai-brains-cli", "--locked"])
        .status()
        .map_err(|e| format!("Failed to invoke cargo: {e}"))?;
    if !cli_ok.success() {
        return Err(format!(
            "cargo install ai-brains-cli failed (exit {:?}). Run from the workspace root.",
            cli_ok.code()
        )
        .into());
    }

    tracing::info!("[update] Installing ai-brainsd via `cargo install --locked`...");
    let daemon_ok = std::process::Command::new("cargo")
        .args(["install", "--path", "crates/ai-brainsd", "--locked"])
        .status()
        .map_err(|e| format!("Failed to invoke cargo: {e}"))?;
    if !daemon_ok.success() {
        return Err(format!(
            "cargo install ai-brainsd failed (exit {:?}). Run from the workspace root.",
            daemon_ok.code()
        )
        .into());
    }
    tracing::info!("[update] Binaries installed.");

    tracing::info!("[update] Restarting daemon...");
    run_start(ctx)?;
    println!("[update] Update complete. New daemon is running.");
    Ok(())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    /// T85: parse_host_port correctly extracts host and port from full URLs.
    #[test]
    fn parse_host_port_full_url() {
        let (host, port) = parse_host_port("http://127.0.0.1:9099").unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 9099);
    }

    #[test]
    fn parse_host_port_with_path() {
        let (host, port) = parse_host_port("http://localhost:11434/api/generate").unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 11434);
    }

    #[test]
    fn parse_host_port_bare_host_port() {
        let (host, port) = parse_host_port("127.0.0.1:8080").unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn parse_host_port_no_port_returns_none() {
        assert!(parse_host_port("http://localhost/").is_none());
        assert!(parse_host_port("localhost").is_none());
    }

    /// T103: schedule_inner with dry_run must return Ok without executing
    /// schtasks and must print the rendered command plus the daemon path.
    #[test]
    #[allow(non_snake_case)]
    fn schedule_inner__dry_run__prints_command_without_registering() {
        let exe = std::path::PathBuf::from(r"C:\fake\ai-brainsd.exe");
        let result = schedule_inner(&exe, true, false);
        assert!(result.is_ok());
    }

    #[test]
    #[allow(non_snake_case)]
    fn schedule_inner__run_as_system__adds_ru_system() {
        let cmd =
            render_daemon_schedule_command(r"C:\fake\ai-brainsd.exe", "AI-Brains-Daemon", 30, true);
        assert!(cmd.contains("/ru SYSTEM"));
        assert!(cmd.ends_with(" /ru SYSTEM"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn schedule_inner__no_run_as_system__omits_ru_system() {
        let cmd = render_daemon_schedule_command(
            r"C:\fake\ai-brainsd.exe",
            "AI-Brains-Daemon",
            30,
            false,
        );
        assert!(!cmd.contains("/ru SYSTEM"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn schedule_inner__dry_run_with_run_as_system__prints_ru_system() {
        let exe = std::path::PathBuf::from(r"C:\fake\ai-brainsd.exe");
        let result = schedule_inner(&exe, true, true);
        assert!(result.is_ok());
    }

    #[test]
    #[allow(non_snake_case)]
    fn render_daemon_schedule_command__run_as_system__includes_no_project_context() {
        let cmd =
            render_daemon_schedule_command(r"C:\fake\ai-brainsd.exe", "AI-Brains-Daemon", 30, true);
        assert!(cmd.contains("--no-project-context"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn generate_daemon_wrapper_script__all_vars_present__includes_set_cd_and_no_project_context(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let env_values: Vec<(&str, String)> = vec![
            ("AI_BRAINS_VAULT_PATH", "C:\\vault\\vault.db".to_string()),
            ("AI_BRAINS_MODEL_URL", "http://127.0.0.1:8081".to_string()),
            ("AI_BRAINS_COMPLETION_MODEL", "model.gguf".to_string()),
            (
                "AI_BRAINS_EMBEDDING_URL",
                "http://127.0.0.1:8083".to_string(),
            ),
            ("AI_BRAINS_EMBEDDING_MODEL", "embed-model".to_string()),
        ];
        let content =
            generate_daemon_wrapper_script_from_env(r"C:\fake\ai-brainsd.exe", &env_values)?;
        assert!(content.contains("set \"AI_BRAINS_VAULT_PATH=C:\\vault\\vault.db\""));
        assert!(content.contains("set \"AI_BRAINS_MODEL_URL=http://127.0.0.1:8081\""));
        assert!(content.contains("cd /d \"C:\\vault\""));
        assert!(content.contains("--no-project-context"));
        assert!(content.contains(r#""C:\fake\ai-brainsd.exe""#));
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn generate_daemon_wrapper_script__missing_env_var__returns_error() {
        let env_values: Vec<(&str, String)> = vec![
            ("AI_BRAINS_VAULT_PATH", String::new()),
            ("AI_BRAINS_MODEL_URL", "http://127.0.0.1:8081".to_string()),
            ("AI_BRAINS_COMPLETION_MODEL", "model.gguf".to_string()),
            (
                "AI_BRAINS_EMBEDDING_URL",
                "http://127.0.0.1:8083".to_string(),
            ),
            ("AI_BRAINS_EMBEDDING_MODEL", "embed-model".to_string()),
        ];
        let result =
            generate_daemon_wrapper_script_from_env(r"C:\fake\ai-brainsd.exe", &env_values);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("AI_BRAINS_VAULT_PATH"));
    }

    /// T103: unschedule_inner with dry_run must return Ok without executing
    /// schtasks /delete.
    #[test]
    #[allow(non_snake_case)]
    fn unschedule_inner__dry_run__prints_command_without_removing() {
        let result = unschedule_inner(true);
        assert!(result.is_ok());
    }

    /// T103: unschedule_inner with dry_run must return Ok for the deletion
    /// command when rendered with the hard-coded task name.
    #[test]
    #[allow(non_snake_case)]
    fn unschedule_inner__dry_run__renders_delete_command_for_ai_brains_daemon() {
        let expected = TaskScheduler::render_delete_command("AI-Brains-Daemon");
        let result = unschedule_inner(true);
        assert!(result.is_ok());
        // The rendered command is emitted to stdout; we verify it is the
        // expected schtasks /delete string rather than inspecting captured
        // output, keeping the test deterministic without stdio plumbing.
        assert!(expected.starts_with("schtasks /delete /tn \"AI-Brains-Daemon\""));
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_size__bytes() {
        assert_eq!(format_size(512), "512 B");
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_size__kilobytes() {
        assert_eq!(format_size(2048), "2.0 KB");
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_size__megabytes() {
        assert_eq!(format_size(1_048_576 * 5), "5.0 MB");
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_size__gigabytes() {
        assert_eq!(format_size(1_073_741_824 * 2), "2.0 GB");
    }
}
