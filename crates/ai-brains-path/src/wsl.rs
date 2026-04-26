use crate::errors::{PathError, Result};

pub fn is_wsl_mount_path(input: &str) -> bool {
    input.starts_with("/mnt/") || input.starts_with("/MNT/")
}

pub fn wsl_to_windows(input: &str) -> Result<String> {
    let trimmed = input.trim();
    let rest = trimmed
        .strip_prefix("/mnt/")
        .or_else(|| trimmed.strip_prefix("/MNT/"))
        .ok_or_else(|| PathError::MalformedWslPath(trimmed.to_string()))?;

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
