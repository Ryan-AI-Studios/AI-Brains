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

pub fn run_schedule(_ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let exe = which_daemon()?;
    let exe_str = exe.to_string_lossy();
    let cmd = TaskScheduler::render_daemon_logon_command(&exe_str, "AI-Brains-Daemon", 30);
    println!("Registering Task Scheduler logon task...");
    println!("  {cmd}");
    let status = std::process::Command::new("cmd")
        .args(["/C", &cmd])
        .status()?;
    if status.success() {
        println!("Task 'AI-Brains-Daemon' registered. Daemon will start at next logon.");
        println!("To start it now without rebooting: ai-brains daemon start");
    } else {
        return Err(
            "schtasks failed — check that you have permission to create scheduled tasks.".into(),
        );
    }
    Ok(())
}

pub fn run_unschedule(_ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = TaskScheduler::render_delete_command("AI-Brains-Daemon");
    let status = std::process::Command::new("cmd")
        .args(["/C", &cmd])
        .status()?;
    if status.success() {
        println!("Task 'AI-Brains-Daemon' removed.");
    } else {
        eprintln!("schtasks /delete failed — task may not exist.");
    }
    Ok(())
}

pub async fn run_stop(_ctx: &AppContext, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();

    if force {
        eprintln!("Forcefully stopping AI-Brains daemon...");
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

    eprintln!("Sending shutdown signal to AI-Brains daemon...");
    match client.shutdown().await {
        Ok(_) => {
            println!("Shutdown signal sent successfully.");
            // Give it a moment to exit
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        Err(e) => {
            eprintln!(
                "Failed to send shutdown signal: {}. The daemon might not be running.",
                e
            );
            eprintln!("Use --force to kill the process if it's unresponsive.");
        }
    }

    Ok(())
}

pub async fn run_status(_ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();
    let is_running = client.probe(std::time::Duration::from_millis(200)).await;

    if is_running {
        println!("Status: Running");
    } else {
        println!("Status: Stopped");
    }

    for port in [8081, 8083] {
        let addr = format!("127.0.0.1:{}", port);
        if let Ok(socket_addr) = addr.parse() {
            match std::net::TcpStream::connect_timeout(
                &socket_addr,
                std::time::Duration::from_millis(100),
            ) {
                Ok(_) => println!("Port {}: Open", port),
                Err(_) => println!("Port {}: Closed", port),
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
