pub fn is_unc_path(input: &str) -> bool {
    input.starts_with("\\\\") || input.starts_with("//")
}

pub fn normalize_unc(input: &str) -> String {
    let without_prefix = input
        .strip_prefix("\\\\?\\UNC\\")
        .map(|rest| format!("\\\\{}", rest))
        .unwrap_or_else(|| input.to_string());

    let replaced = without_prefix.replace('/', "\\");
    let body = replaced
        .trim_start_matches('\\')
        .split('\\')
        .filter(|segment| !segment.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("\\");

    format!("\\\\{}", body)
}
