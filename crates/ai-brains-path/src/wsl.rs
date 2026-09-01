use crate::errors::{PathError, Result};
use crate::windows::has_drive_prefix;

pub fn is_wsl_mount_path(input: &str) -> bool {
    input.to_lowercase().starts_with("/mnt/")
}

pub fn wsl_to_windows(input: &str) -> Result<String> {
    let trimmed = input.trim();
    let lower = trimmed.to_lowercase();
    let rest = if lower.starts_with("/mnt/") {
        &trimmed[5..]
    } else {
        return Err(PathError::MalformedWslPath(trimmed.to_string()));
    };

    let mut parts = rest.split('/');
    let drive = parts
        .next()
        .ok_or_else(|| PathError::MalformedWslPath(trimmed.to_string()))?;

    if drive.len() != 1 {
        return Err(PathError::MalformedWslPath(trimmed.to_string()));
    }

    let drive_char = drive
        .chars()
        .next()
        .ok_or_else(|| PathError::MalformedWslPath(trimmed.to_string()))?;

    if !drive_char.is_ascii_alphabetic() {
        return Err(PathError::MalformedWslPath(trimmed.to_string()));
    }

    let mut windows = format!("{}:\\", drive_char.to_ascii_uppercase());
    let remainder = parts.collect::<Vec<_>>().join("\\");
    if !remainder.is_empty() {
        windows.push_str(&remainder);
    }

    Ok(windows)
}

/// Inverse of [`wsl_to_windows`] for single-letter Windows drive paths.
///
/// `C:\dev\ai-brains` → `/mnt/c/dev/ai-brains`. `C:\` → `/mnt/c`.
/// Accepts `\` or `/` after the drive. UNC / relative / non-drive → error.
pub fn windows_drive_to_wsl_mount(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(PathError::EmptyInput);
    }
    let replaced = trimmed.replace('\\', "/");
    if !has_drive_prefix(&replaced) {
        return Err(PathError::RelativePath(trimmed.to_string()));
    }
    let drive_char = replaced
        .chars()
        .next()
        .ok_or_else(|| PathError::RelativePath(trimmed.to_string()))?
        .to_ascii_lowercase();
    if !drive_char.is_ascii_alphabetic() {
        return Err(PathError::RelativePath(trimmed.to_string()));
    }
    let rest = replaced[2..].trim_matches('/');
    if rest.is_empty() {
        Ok(format!("/mnt/{drive_char}"))
    } else {
        Ok(format!("/mnt/{drive_char}/{rest}"))
    }
}
