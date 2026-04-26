#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    std::env::temp_dir().join(format!("ai-brains-git-{name}-{nanos}"))
}

pub fn run_git(path: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(args).current_dir(path).output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}

pub fn init_repo(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = unique_temp_dir(name);
    fs::create_dir_all(&root)?;
    run_git(&root, &["init"])?;
    run_git(&root, &["config", "user.name", "AI Brains Test"])?;
    run_git(&root, &["config", "user.email", "tests@example.com"])?;
    Ok(root)
}

pub fn commit_file(
    root: &Path,
    relative_path: &str,
    content: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    run_git(root, &["add", "."])?;
    run_git(root, &["commit", "-m", message])?;
    Ok(())
}
