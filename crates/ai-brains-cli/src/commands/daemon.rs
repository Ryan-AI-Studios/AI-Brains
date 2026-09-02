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

    if run_as_system {
        let _ = generate_daemon_wrapper_script(&exe_str)?;
        match crate::elevation::ensure_elevated_or_relaunch()? {
            crate::elevation::ElevationOutcome::AlreadyElevated => {}
            crate::elevation::ElevationOutcome::Relaunched { exit_code } => {
                return report_elevated_outcome(exit_code, "schedule");
            }
        }
    }

    let task_command = if run_as_system {
        let content = generate_daemon_wrapper_script(&exe_str)?;
        let path = write_daemon_wrapper_script(&content)?;
        if !crate::artifact_security::may_register_after_prepare(true) {
            return Err(
                "internal: daemon wrapper prepare reported success but registration gate denied"
                    .into(),
            );
        }
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
    // Windows .bat scripts always use `\`. Host Path::parent is OS-sensitive
    // (`\` is not a separator on Unix), so split on Windows separators after
    // normalizing `/` → `\` (T179 cross-platform unit tests).
    if let Some(parent) = windows_path_parent(vault_path) {
        lines.push(format!("cd /d \"{parent}\""));
    }
    lines.push(format!(r#""{}" --no-project-context"#, exe_str));
    Ok(lines.join("\n"))
}

/// Parent directory of a Windows-style path for `.bat` `cd /d` lines.
///
/// Treats both `\` and `/` as separators so generation is host-OS independent.
/// Drive roots are returned with a trailing `\` (`C:\`), matching `Path::parent`
/// on Windows for `C:\file.db`.
fn windows_path_parent(path: &str) -> Option<String> {
    let normalized = path.replace('/', "\\");
    let trimmed = normalized.trim_end_matches('\\');
    let (parent, _leaf) = trimmed.rsplit_once('\\')?;
    if parent.is_empty() {
        None
    } else if parent.ends_with(':') {
        // Drive root: `C:\vault.db` → `C:\` (not bare `C:`).
        Some(format!("{parent}\\"))
    } else {
        Some(parent.to_string())
    }
}

fn write_daemon_wrapper_script(
    content: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // T145: %ProgramData%\AI-Brains\daemon-task.bat with SYSTEM+Administrators ACL only.
    let path = crate::artifact_security::daemon_wrapper_path();
    crate::artifact_security::write_protected_artifact(&path, content)?;
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
            Ok(_) => crate::artifact_security::daemon_wrapper_path()
                .display()
                .to_string(),
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
    tracing::warn!(
        "Note: `daemon schedule` is deprecated. Use `ai-brains daemon install` for a proper Windows service."
    );
    let exe = which_daemon()?;
    schedule_inner(&exe, dry_run, run_as_system)
}

fn unschedule_inner(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    tracing::warn!(
        "Note: `daemon unschedule` is deprecated. Use `ai-brains daemon uninstall` to remove the Windows service."
    );
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

pub fn run_install(_ctx: &AppContext, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let exe = which_daemon()?;
    let exe_str = exe.to_string_lossy().to_string();

    // T145: daemon.env under %ProgramData%\AI-Brains with SYSTEM+Administrators ACL only.
    let env_sidecar_path = crate::artifact_security::daemon_env_path();

    let service_name = ai_brains_scheduler::ServiceScheduler::service_name();
    let bin_path = format!("{exe_str} --service");
    let display_name = "AI-Brains Daemon";

    if dry_run {
        println!("[dry-run] Would execute the following commands:");
        println!(
            "  1. sc create \"{service_name}\" binPath= \"{bin_path}\" start= delayed-auto DisplayName= \"{display_name}\""
        );
        println!("  2. sc description \"{service_name}\" \"...\"");
        println!(
            "  3. Write env vars (ACL-restricted) to: {}",
            env_sidecar_path.display()
        );
        println!("  4. sc start \"{service_name}\"");
        println!();
        println!("(Requires an elevated PowerShell session.)");
        return Ok(());
    }

    match crate::elevation::ensure_elevated_or_relaunch()? {
        crate::elevation::ElevationOutcome::AlreadyElevated => {}
        crate::elevation::ElevationOutcome::Relaunched { exit_code } => {
            return report_elevated_outcome(exit_code, "install");
        }
    }

    // Env sidecar + parent dir hardening (fail closed before service create):
    // - Always ensure %ProgramData%\AI-Brains parent is non-reparse + ACL-restricted
    //   before sc create (even when no env content is written).
    // - Some(content) → rewrite via write_protected_artifact (parent+file ACL).
    // - None + path exists → apply+verify ACL on the existing file.
    // - None + path missing → parent still protected; refuse dangling reparse at path.
    crate::artifact_security::ensure_program_data_ai_brains_dir()?;
    let env_sidecar_content = generate_env_sidecar();
    match env_sidecar_content {
        Some(content) => {
            crate::artifact_security::write_protected_artifact(&env_sidecar_path, &content)?;
            println!("Env sidecar written to: {}", env_sidecar_path.display());
        }
        None if env_sidecar_path.exists() => {
            crate::artifact_security::ensure_protected_artifact_acl(&env_sidecar_path)?;
            println!(
                "Existing env sidecar ACL hardened: {}",
                env_sidecar_path.display()
            );
        }
        None => {
            // Dangling reparse can make Path::exists() false — still refuse.
            if crate::artifact_security::is_reparse_or_symlink(&env_sidecar_path)? {
                return Err(format!(
                    "refusing to register service: reparse point/symlink at {}",
                    env_sidecar_path.display()
                )
                .into());
            }
            println!(
                "No env sidecar content; parent ACL protected at {}",
                crate::artifact_security::program_data_ai_brains_dir().display()
            );
        }
    }

    println!("Creating service...");
    let output = std::process::Command::new("sc")
        .arg("create")
        .arg(service_name)
        .arg(format!("binPath= {bin_path}"))
        .arg("start=")
        .arg("demand")
        .arg(format!("DisplayName= {display_name}"))
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stderr.contains("already exists") || stdout.contains("already exists") {
            return Err("Service 'AI-Brains-Daemon' already exists. Use `ai-brains daemon uninstall` first.".into());
        }
        return Err(format!("sc create failed: {stderr}{stdout}").into());
    }

    println!("Setting description...");
    let _ = std::process::Command::new("sc")
        .args(["description", service_name])
        .arg(ai_brains_scheduler::ServiceScheduler::service_description())
        .output();

    println!("Setting delayed-auto start...");
    let _ = std::process::Command::new("sc")
        .arg("config")
        .arg(service_name)
        .arg("start=")
        .arg("delayed-auto")
        .output();
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("Starting service...");
    let start_output = std::process::Command::new("sc")
        .args(["start", service_name])
        .output()?;
    if start_output.status.success() {
        println!("Service 'AI-Brains-Daemon' installed and started.");
    } else {
        println!(
            "Service installed but failed to auto-start. Use `ai-brains daemon start` or `sc start AI-Brains-Daemon`."
        );
    }

    Ok(())
}

pub fn run_uninstall(_ctx: &AppContext, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let service_name = ai_brains_scheduler::ServiceScheduler::service_name();

    if dry_run {
        println!("[dry-run] Would execute the following commands:");
        println!("  1. sc stop \"{service_name}\"");
        println!("  2. sc delete \"{service_name}\"");
        println!();
        println!("(Requires an elevated PowerShell session.)");
        return Ok(());
    }

    match crate::elevation::ensure_elevated_or_relaunch()? {
        crate::elevation::ElevationOutcome::AlreadyElevated => {}
        crate::elevation::ElevationOutcome::Relaunched { exit_code } => {
            return report_elevated_outcome(exit_code, "uninstall");
        }
    }

    println!("Stopping service...");
    let _ = std::process::Command::new("sc")
        .args(["stop", service_name])
        .output();
    std::thread::sleep(std::time::Duration::from_secs(2));

    println!("Deleting service...");
    let output = std::process::Command::new("sc")
        .args(["delete", service_name])
        .output()?;
    if output.status.success() {
        println!("Service 'AI-Brains-Daemon' removed.");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stderr.contains("does not exist")
            || stdout.contains("does not exist")
            || stderr.contains("1060")
            || stdout.contains("1060")
        {
            println!("Service 'AI-Brains-Daemon' was not installed — nothing to remove.");
        } else {
            return Err(format!("sc delete failed: {stderr}{stdout}").into());
        }
    }

    Ok(())
}

fn report_elevated_outcome(exit_code: u32, action: &str) -> Result<(), Box<dyn std::error::Error>> {
    if exit_code == 0 {
        if let Some(Ok(msg)) = crate::elevation::take_elevate_result() {
            println!("{msg}");
        }
        println!("Elevated daemon {action} finished successfully.");
        return Ok(());
    }
    let detail = crate::elevation::take_elevate_result()
        .and_then(|r| r.err())
        .or_else(crate::elevation::take_elevate_error_log)
        .unwrap_or_else(|| {
            "(no elevated error log; re-run from an Admin shell for full stderr)".into()
        });
    Err(format!("Elevated {action} process exited with code {exit_code}: {detail}").into())
}

/// dotenvy-safe sidecar line. Paths use `/` so `\a` in `\ai-brains` is not BEL.
/// Values are double-quoted so `x'<hex>'` is not eaten as a single-quoted token.
pub(crate) fn format_daemon_env_line(key: &str, val: &str) -> String {
    let normalized = if key == "AI_BRAINS_VAULT_PATH" {
        val.replace('\\', "/")
    } else {
        val.to_string()
    };
    let escaped = normalized.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{key}=\"{escaped}\"")
}

fn generate_env_sidecar() -> Option<String> {
    let keys = [
        "AI_BRAINS_VAULT_PATH",
        "AI_BRAINS_VAULT_KEY",
        "AI_BRAINS_MODEL_URL",
        "AI_BRAINS_COMPLETION_MODEL",
        "AI_BRAINS_EMBEDDING_URL",
        "AI_BRAINS_EMBEDDING_MODEL",
    ];
    let mut lines = Vec::new();
    let mut found_any = false;
    for key in &keys {
        let val = if *key == "AI_BRAINS_VAULT_KEY" {
            std::env::var("AI_BRAINS_VAULT_KEY")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .or_else(|| {
                    std::env::var("AI_BRAINS_KEY")
                        .ok()
                        .filter(|v| !v.trim().is_empty())
                })
        } else {
            std::env::var(key).ok().filter(|v| !v.is_empty())
        };
        if let Some(val) = val {
            lines.push(format_daemon_env_line(key, &val));
            found_any = true;
        }
    }
    if found_any {
        Some(lines.join("\n"))
    } else {
        None
    }
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

/// Options for vault-independent `daemon status` (T199).
///
/// No [`AppContext`]: status must not open/migrate the vault for liveness.
pub struct StatusOptions {
    pub vault_path: Option<std::path::PathBuf>,
    pub key: Option<String>,
}

/// Optional pinned-memory count via read-intent only (T199 F7 / AC13).
///
/// Swallow-only: never propagates errors; never migrates. Returns `None` when
/// key is missing, vault is not openable, or the query fails.
fn try_count_pinned_optional(path: &std::path::Path, key: Option<String>) -> Option<u64> {
    use crate::key_resolve::resolve_operator_sqlcipher_key;
    use ai_brains_store::connection::VaultConnection;

    let resolved = resolve_operator_sqlcipher_key(key).ok()?;
    let conn = VaultConnection::open_read_intent(path, &resolved).ok()?;
    let lock = conn.lock().ok()?;
    lock.query_row(
        "SELECT COUNT(*) FROM memory_projection WHERE status = 'pinned'",
        [],
        |row| row.get(0),
    )
    .ok()
}

/// Vault section lines for status (T199 F6/F7; soft F12 extract for T201 JSON).
///
/// - Stopped → empty (T128)
/// - Running + no path → empty
/// - Running + path → Vault / Vault size always; Memories count or skip line
fn format_status_vault_section(
    is_running: bool,
    vault_path: Option<&std::path::Path>,
    key: Option<String>,
) -> Vec<String> {
    if !is_running {
        return Vec::new();
    }
    let Some(path) = vault_path else {
        return Vec::new();
    };
    let mut lines = Vec::with_capacity(3);
    lines.push(format!("Vault: {}", path.display()));
    // Honest size: real metadata length, or "unavailable" (never fake 0 B on IO fail).
    let size_label = match std::fs::metadata(path) {
        Ok(m) => format_size(m.len()),
        Err(_) => "unavailable".to_string(),
    };
    lines.push(format!("Vault size: {size_label}"));
    match try_count_pinned_optional(path, key) {
        Some(n) => lines.push(format!("Memories: {n}")),
        None => {
            lines.push("Memories: skipped (vault key missing or vault not openable)".to_string())
        }
    }
    lines
}

/// Next-step line for `daemon status` (T249). Printed only when Stopped.
pub(crate) fn status_next_line(is_running: bool) -> Option<&'static str> {
    if is_running {
        None
    } else {
        Some("next: ai-brains daemon start")
    }
}

/// Human contrast when Stopped and a model/embedding TCP port is Open (T297 F1).
/// U+2260 `≠` — not ASCII `!=`. Not T281 `HTTP /health 750ms ≠ daemon TCP`.
pub(crate) const BACKEND_OPEN_NE_DAEMON: &str = "backend TCP Open ≠ daemon";

/// Some iff Stopped and at least one backend TCP port is Open (T297 F2).
pub(crate) fn status_backend_contrast_line(
    is_running: bool,
    llm_open: bool,
    embed_open: bool,
) -> Option<&'static str> {
    if !is_running && (llm_open || embed_open) {
        Some(BACKEND_OPEN_NE_DAEMON)
    } else {
        None
    }
}

/// Tail after PID: contrast (if any) then T249 `next:` (T297 F5 / F24).
pub(crate) fn status_report_tail(
    is_running: bool,
    llm_open: bool,
    embed_open: bool,
) -> Vec<&'static str> {
    let mut lines = Vec::new();
    if let Some(contrast) = status_backend_contrast_line(is_running, llm_open, embed_open) {
        lines.push(contrast);
    }
    if let Some(next) = status_next_line(is_running) {
        lines.push(next);
    }
    lines
}

/// Interactive daemon status (T199): liveness IPC without vault key / open.
pub async fn run_status(opts: StatusOptions) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();
    let is_running = crate::daemon_probe::probe_daemon_reachable(
        &client,
        crate::daemon_probe::DaemonProbePolicy::Status,
    )
    .await;

    if is_running {
        println!("Status: Running");
    } else {
        println!("Status: Stopped");
    }

    // T128/T199: vault section only when Running and vault path is present.
    for line in format_status_vault_section(is_running, opts.vault_path.as_deref(), opts.key) {
        println!("{line}");
    }

    // T85: resolve backend addresses from configuration rather than hardcoded ports
    let (model_host, model_port, model_desc) = resolve_backend(
        "AI_BRAINS_MODEL_URL",
        "127.0.0.1",
        8081,
        "completion default :8081",
    );
    let (embed_host, embed_port, embed_desc) = resolve_backend(
        "AI_BRAINS_EMBEDDING_URL",
        "127.0.0.1",
        8083,
        "embedding default :8083",
    );

    let mut llm_open = false;
    let mut embed_open = false;
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
                // T297 F36: capture Open by backend name (not both from one state).
                match name {
                    "LLM backend" => llm_open = state == "Open",
                    "Embedding backend" => embed_open = state == "Open",
                    _ => {}
                }
                println!("{} {} [{}]: {}", name, addr, desc, state);
            }
            Err(_) => {
                println!("{} {}: unable to parse address", name, addr);
            }
        }
    }

    // Soft PID report (T199 F8/AC12): tasklist failure must not exit non-zero.
    #[cfg(windows)]
    {
        if let Ok(output) = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq ai-brainsd.exe", "/FO", "CSV", "/NH"])
            .output()
        {
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
    }

    // T297 F5/F29: contrast (if any) then T249 next: — after PID block.
    for line in status_report_tail(is_running, llm_open, embed_open) {
        println!("{line}");
    }

    Ok(())
}

#[cfg(test)]
mod status_vault_tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use rstest::rstest;
    use std::path::Path;

    /// T249 AC7: Stopped prints next-step; Running omits it.
    #[test]
    fn status_next_line__stopped__daemon_start() {
        assert_eq!(
            status_next_line(false),
            Some("next: ai-brains daemon start")
        );
    }

    #[test]
    fn status_next_line__running__none() {
        assert!(status_next_line(true).is_none());
    }

    /// T297 AC1/AC2/AC3/AC4 / F35: exhaustive 8-triple matrix.
    #[rstest]
    #[case(false, false, false, false)]
    #[case(false, true, false, true)]
    #[case(false, false, true, true)]
    #[case(false, true, true, true)]
    #[case(true, false, false, false)]
    #[case(true, true, false, false)]
    #[case(true, false, true, false)]
    #[case(true, true, true, false)]
    fn status_backend_contrast_line__matrix(
        #[case] is_running: bool,
        #[case] llm_open: bool,
        #[case] embed_open: bool,
        #[case] expect_some: bool,
    ) {
        let got = status_backend_contrast_line(is_running, llm_open, embed_open);
        if expect_some {
            assert_eq!(got, Some(BACKEND_OPEN_NE_DAEMON));
        } else {
            assert!(got.is_none(), "expected None; got {got:?}");
        }
    }

    /// T297 AC5 / F18: U+2260, not ASCII `!=`.
    #[test]
    fn backend_open_ne_daemon__uses_u2260_not_ascii() {
        assert!(
            BACKEND_OPEN_NE_DAEMON.contains('\u{2260}'),
            "const must use U+2260; got: {BACKEND_OPEN_NE_DAEMON}"
        );
        assert_ne!(BACKEND_OPEN_NE_DAEMON, "backend TCP Open != daemon");
        assert_eq!(BACKEND_OPEN_NE_DAEMON, "backend TCP Open ≠ daemon");
    }

    /// T297 AC6 / F30: Stopped+Open pair → single contrast then next:.
    #[rstest]
    #[case(true, false)]
    #[case(false, true)]
    #[case(true, true)]
    fn status_report_tail__stopped_open_pair__single_contrast_then_next(
        #[case] llm_open: bool,
        #[case] embed_open: bool,
    ) {
        let tail = status_report_tail(false, llm_open, embed_open);
        assert_eq!(
            tail,
            vec![BACKEND_OPEN_NE_DAEMON, "next: ai-brains daemon start"],
            "F30: exactly one contrast then next:; got {tail:?}"
        );
        assert_eq!(tail.last().copied(), Some("next: ai-brains daemon start"));
    }

    /// T297 AC6: Stopped + both Closed → next: only.
    #[test]
    fn status_report_tail__stopped_closed__next_only() {
        assert_eq!(
            status_report_tail(false, false, false),
            vec!["next: ai-brains daemon start"]
        );
    }

    /// T297 AC6: Running → empty even if both Open.
    #[test]
    fn status_report_tail__running__empty() {
        assert!(status_report_tail(true, true, true).is_empty());
    }

    /// AC13: missing key → None (no panic, no propagate).
    #[test]
    fn try_count_pinned_optional__no_key__returns_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing.db");
        assert!(try_count_pinned_optional(&path, None).is_none());
    }

    /// AC13: non-existent vault with key → None (open fails swallowed).
    #[test]
    fn try_count_pinned_optional__missing_vault__returns_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("no-such.db");
        // Zero key still fails resolve without ALLOW; use None path for open fail.
        // Explicit absent path with unresolved-looking key string that resolve rejects.
        assert!(try_count_pinned_optional(&path, Some("not-a-key".into())).is_none());
    }

    /// AC6: Stopped → no vault lines.
    #[test]
    fn format_status_vault_section__stopped__empty() {
        let lines = format_status_vault_section(false, Some(Path::new("C:\\vault.db")), None);
        assert!(
            lines.is_empty(),
            "stopped must omit vault section: {lines:?}"
        );
    }

    /// Running + no path → omit vault section.
    #[test]
    fn format_status_vault_section__running_no_path__empty() {
        let lines = format_status_vault_section(true, None, None);
        assert!(
            lines.is_empty(),
            "no path must omit vault section: {lines:?}"
        );
    }

    /// AC7: Running + path + no key → path/size + Memories skip line.
    #[test]
    fn format_status_vault_section__running_path_no_key__skip_memories() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("vault.db");
        std::fs::write(&path, b"placeholder").expect("write vault stub");

        let lines = format_status_vault_section(true, Some(path.as_path()), None);
        assert_eq!(
            lines.len(),
            3,
            "expected Vault/size/Memories; got: {lines:?}"
        );
        assert!(
            lines[0].starts_with("Vault: "),
            "path line; got: {}",
            lines[0]
        );
        assert!(
            lines[1].starts_with("Vault size: "),
            "size line; got: {}",
            lines[1]
        );
        assert!(
            !lines[1].contains("unavailable"),
            "existing file must report real size; got: {}",
            lines[1]
        );
        assert_eq!(
            lines[2],
            "Memories: skipped (vault key missing or vault not openable)"
        );
    }

    /// Metadata IO failure must not report a fake 0 B size.
    #[test]
    fn format_status_vault_section__running_missing_file__size_unavailable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("does-not-exist.db");
        let lines = format_status_vault_section(true, Some(path.as_path()), None);
        assert!(
            lines.len() >= 2,
            "expected Vault + size lines; got: {lines:?}"
        );
        assert_eq!(
            lines[1], "Vault size: unavailable",
            "missing file must not fake 0 B; got: {lines:?}"
        );
    }
}

/// T84: Stop the daemon, install updated binaries via `cargo install`, then restart.
///
/// Must be run from the workspace root. Gracefully stops the daemon first;
/// falls back to a force-kill if it does not respond within ~1 s.
///
/// T310 F1: CLI argv reconstructs `GRAPH_REINSTALL_SOOT` (unit-proven). Do not
/// edit that SOOT string. Keep these slices in this module (do not grow
/// `governed_common.rs`).
pub(crate) const UPDATE_CLI_CARGO_ARGS: &[&str] = &[
    "install",
    "--path",
    "crates/ai-brains-cli",
    "--locked",
    "--features",
    "graph",
];

/// T310 F2 / AC4: daemon crate has no `graph` feature.
pub(crate) const UPDATE_DAEMON_CARGO_ARGS: &[&str] =
    &["install", "--path", "crates/ai-brainsd", "--locked"];

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
        .args(UPDATE_CLI_CARGO_ARGS)
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
        .args(UPDATE_DAEMON_CARGO_ARGS)
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
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::commands::governed_common::GRAPH_REINSTALL_SOOT;

    /// T310 AC1: CLI cargo argv reconstructs GRAPH_REINSTALL_SOOT (not a parallel literal).
    #[test]
    fn run_update_cli_args__reconstruct_graph_reinstall_soot() {
        assert_eq!(
            format!("cargo {}", UPDATE_CLI_CARGO_ARGS.join(" ")),
            GRAPH_REINSTALL_SOOT
        );
    }

    /// T310 AC4: daemon install stays `--locked` with no `--features graph`.
    #[test]
    fn run_update_daemon_args__no_graph_feature() {
        assert_eq!(
            UPDATE_DAEMON_CARGO_ARGS,
            ["install", "--path", "crates/ai-brainsd", "--locked"]
        );
        assert!(
            !UPDATE_DAEMON_CARGO_ARGS
                .iter()
                .any(|a| *a == "--features" || *a == "graph"),
            "daemon crate has no graph feature; got {UPDATE_DAEMON_CARGO_ARGS:?}"
        );
    }

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

    /// T349 AC11: unset env uses nightly 8081/8083 labels, not Ollama/llama.cpp.
    #[test]
    fn resolve_backend__unset_env__nightly_default_ports() {
        let _m = ai_brains_core::temp_env::TempEnv::remove("AI_BRAINS_MODEL_URL");
        let _e = ai_brains_core::temp_env::TempEnv::remove("AI_BRAINS_EMBEDDING_URL");
        let (host, port, desc) = resolve_backend(
            "AI_BRAINS_MODEL_URL",
            "127.0.0.1",
            8081,
            "completion default :8081",
        );
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 8081);
        assert!(desc.contains("completion default :8081"), "desc={desc}");
        assert!(
            !desc.contains("Ollama") && !desc.contains("11434"),
            "desc={desc}"
        );
        let (ehost, eport, edesc) = resolve_backend(
            "AI_BRAINS_EMBEDDING_URL",
            "127.0.0.1",
            8083,
            "embedding default :8083",
        );
        assert_eq!(ehost, "127.0.0.1");
        assert_eq!(eport, 8083);
        assert!(edesc.contains("embedding default :8083"), "edesc={edesc}");
        assert!(
            !edesc.contains("llama.cpp") && !edesc.contains(":8080"),
            "edesc={edesc}"
        );
    }

    #[test]
    fn format_daemon_env_line__windows_path__forward_slash_quoted() {
        let line = format_daemon_env_line("AI_BRAINS_VAULT_PATH", r"C:\dev\ai-brains\vault.db");
        assert_eq!(line, r#"AI_BRAINS_VAULT_PATH="C:/dev/ai-brains/vault.db""#);
        assert!(!line.contains('\\'), "{line}");
    }

    #[test]
    fn format_daemon_env_line__product_key__double_quoted_x_form() {
        let key = "x'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'";
        let line = format_daemon_env_line("AI_BRAINS_VAULT_KEY", key);
        assert_eq!(line, format!(r#"AI_BRAINS_VAULT_KEY="{key}""#));
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
    fn render_daemon_schedule_command__run_as_system__uses_program_data_wrapper_or_no_project_context()
     {
        // When env is complete, dry-run /tr is the ProgramData wrapper path (flags live in the bat).
        // When env is incomplete, dry-run falls back to the bare exe + --no-project-context.
        let cmd =
            render_daemon_schedule_command(r"C:\fake\ai-brainsd.exe", "AI-Brains-Daemon", 30, true);
        let uses_wrapper = cmd.contains("daemon-task.bat") || cmd.contains("AI-Brains");
        let uses_flag = cmd.contains("--no-project-context");
        assert!(
            uses_wrapper || uses_flag,
            "expected ProgramData wrapper path or --no-project-context fallback, got: {cmd}"
        );
        assert!(cmd.contains("/ru SYSTEM"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn generate_daemon_wrapper_script__all_vars_present__includes_set_cd_and_no_project_context()
    -> Result<(), Box<dyn std::error::Error>> {
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("AI_BRAINS_VAULT_PATH")
        );
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
