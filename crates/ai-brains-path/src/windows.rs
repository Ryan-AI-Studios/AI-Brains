use crate::errors::{PathError, Result};

pub fn strip_extended_length_prefix(input: &str) -> String {
    if let Some(rest) = input.strip_prefix("\\\\?\\UNC\\") {
        return format!("\\\\{}", rest);
    }

    if let Some(rest) = input.strip_prefix("\\\\?\\") {
        return rest.to_string();
    }

    input.to_string()
}

pub fn has_drive_prefix(input: &str) -> bool {
    let bytes = input.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

pub fn normalize_drive_path(input: &str) -> Result<String> {
    let replaced = strip_extended_length_prefix(input).replace('/', "\\");

    if !has_drive_prefix(&replaced) {
        return Err(PathError::RelativePath(input.to_string()));
    }

    let drive = replaced
        .chars()
        .next()
        .ok_or_else(|| PathError::RelativePath(input.to_string()))?
        .to_ascii_uppercase();

    let rest = replaced[2..]
        .split('\\')
        .filter(|segment| !segment.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("\\");

    if rest.is_empty() {
        Ok(format!("{drive}:\\"))
    } else {
        Ok(format!("{drive}:\\{rest}"))
    }
}
